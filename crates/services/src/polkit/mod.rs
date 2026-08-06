use anyhow::{Context, Result};

use std::collections::{HashMap, VecDeque};

use std::os::unix::fs::PermissionsExt;

use std::process::Stdio;

use std::sync::{Arc, Mutex, OnceLock};

use tokio::process::Command;

use zbus::zvariant::{OwnedValue, Value};

use zbus::{connection, interface};

fn is_setuid(path: &std::path::Path) -> bool {
    if let Ok(metadata) = std::fs::metadata(path) {
        let mode = metadata.permissions().mode();

        return (mode & 0o4000) != 0;
    }

    false
}

#[derive(Debug, Clone)]

pub struct PolkitAuthRequest {
    pub action_id: String,

    pub message: String,

    pub icon_name: String,

    pub user_name: String,

    pub cookie: String,

    pub uid: u32,

    pub identity_kind: String,

    pub identity_details: HashMap<String, Value<'static>>,
}

pub type PolkitPendingAuth = (
    PolkitAuthRequest,
    tokio::sync::oneshot::Sender<Result<(), String>>,
);

static POLKIT_QUEUE: OnceLock<Arc<Mutex<VecDeque<PolkitPendingAuth>>>> = OnceLock::new();

static POLKIT_AGENT_CONN: OnceLock<Arc<Mutex<Option<zbus::Connection>>>> = OnceLock::new();

fn polkit_queue() -> &'static Arc<Mutex<VecDeque<PolkitPendingAuth>>> {
    POLKIT_QUEUE.get_or_init(|| Arc::new(Mutex::new(VecDeque::new())))
}

fn polkit_agent_conn() -> &'static Arc<Mutex<Option<zbus::Connection>>> {
    POLKIT_AGENT_CONN.get_or_init(|| Arc::new(Mutex::new(None)))
}

fn set_agent_connection(conn: Option<zbus::Connection>) {
    if let Ok(mut lock) = polkit_agent_conn().lock() {
        *lock = conn;
    }
}

pub fn push_polkit_request(
    req: PolkitAuthRequest,

    responder: tokio::sync::oneshot::Sender<Result<(), String>>,
) {
    if let Ok(mut queue) = polkit_queue().lock() {
        queue.push_back((req, responder));
    }
}

pub fn pop_polkit_request() -> Option<PolkitPendingAuth> {
    if let Ok(mut queue) = polkit_queue().lock() {
        queue.pop_front()
    } else {
        None
    }
}

pub fn cancel_polkit_request(cookie: &str) -> bool {
    if let Ok(mut queue) = polkit_queue().lock() {
        if let Some(pos) = queue.iter().position(|(req, _)| req.cookie == cookie) {
            if let Some((_, responder)) = queue.remove(pos) {
                let _ = responder.send(Err("Cancelled by Polkit Authority".to_string()));

                return true;
            }
        }
    }

    false
}

#[derive(Clone, Default)]

pub struct PolkitService;

impl PolkitService {
    pub fn new() -> Self {
        start_polkit_agent();

        Self
    }

    pub fn pop_request(&self) -> Option<PolkitPendingAuth> {
        pop_polkit_request()
    }

    pub async fn authenticate(&self, user_name: &str, cookie: &str, password: &str) -> bool {
        authenticate_user(user_name, cookie, password).await.is_ok()
    }
}

pub struct PolkitAgentServer;

#[interface(name = "org.freedesktop.PolicyKit1.AuthenticationAgent")]

impl PolkitAgentServer {
    async fn begin_authentication(
        &self,

        action_id: String,

        message: String,

        icon_name: String,

        details: HashMap<String, String>,

        cookie: String,

        identities: Vec<(String, HashMap<String, Value<'_>>)>,
    ) -> zbus::fdo::Result<()> {
        let _ = details;

        let user_name = std::env::var("USER").unwrap_or_else(|_| "root".to_string());

        let current_uid = unsafe {
            unsafe extern "C" {

                fn getuid() -> u32;

            }

            getuid()
        };

        let mut identity_kind = "unix-user".to_string();

        let mut identity_details: HashMap<String, Value<'static>> = HashMap::new();

        identity_details.insert("uid".to_string(), Value::from(current_uid));

        if let Some(first_ident) = identities.first() {
            identity_kind = first_ident.0.clone();

            identity_details.clear();

            for (k, v) in &first_ident.1 {
                if let Ok(owned) = OwnedValue::try_from(v.clone()) {
                    identity_details.insert(k.clone(), Value::from(owned));
                }
            }
        }

        let req = PolkitAuthRequest {
            action_id: action_id.clone(),
            message: message.clone(),
            icon_name,
            user_name,
            cookie: cookie.clone(),
            uid: current_uid,
            identity_kind,
            identity_details,
        };

        crate::log_info!(
            "POLKIT",
            "Polkit auth request received: action='{action_id}', msg='{message}', cookie='{cookie}'"
        );

        let (tx, rx) = tokio::sync::oneshot::channel();
        push_polkit_request(req, tx);

        match tokio::time::timeout(std::time::Duration::from_secs(120), rx).await {
            Ok(Ok(Ok(()))) => {
                crate::log_info!(
                    "POLKIT",
                    "User authentication successfully authorized by helper for cookie='{cookie}'"
                );
                Ok(())
            }
            _ => {
                crate::log_warn!(
                    "POLKIT",
                    "Polkit authentication failed, cancelled by user, or timed out for cookie='{cookie}'!"
                );
                Err(zbus::fdo::Error::Failed(
                    "Authentication failed or cancelled".to_string(),
                ))
            }
        }
    }

    async fn cancel_authentication(&self, cookie: String) -> zbus::fdo::Result<()> {
        crate::log_info!(
            "POLKIT",
            "Polkit Authority requested CancelAuthentication for cookie='{cookie}'"
        );
        cancel_polkit_request(&cookie);
        Ok(())
    }
}

pub fn start_polkit_agent() {
    static STARTED: OnceLock<()> = OnceLock::new();
    if STARTED.set(()).is_err() {
        return;
    }

    tokio::spawn(async move {
        loop {
            crate::log_info!(
                "POLKIT",
                "Registering Polkit AuthenticationAgent on System Bus..."
            );

            let server = PolkitAgentServer;

            if let Err(err) = register_agent(server).await {
                crate::log_warn!(
                    "POLKIT",
                    "Polkit Agent error: {err:?}. Re-registering in 4s..."
                );
            } else {
                crate::log_warn!(
                    "POLKIT",
                    "Polkit Agent loop exited unexpectedly. Re-registering in 4s..."
                );
            }

            set_agent_connection(None);

            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        }
    });
}

async fn register_agent(server: PolkitAgentServer) -> Result<()> {
    let system_conn = connection::Builder::system()?
        .serve_at("/org/freedesktop/PolicyKit1/AuthenticationAgent", server)?
        .build()
        .await?;

    set_agent_connection(Some(system_conn.clone()));

    let locale = std::env::var("LANG").unwrap_or_else(|_| "es_ES.UTF-8".to_string());

    let object_path = "/org/freedesktop/PolicyKit1/AuthenticationAgent";

    let session_id = std::env::var("XDG_SESSION_ID")
        .or_else(|_| std::env::var("XDG_SESSION_COOKIE"))
        .unwrap_or_else(|_| "2".to_string());

    let mut session_details: HashMap<String, Value> = HashMap::new();

    session_details.insert("session-id".to_string(), Value::from(session_id));

    let reg_result = system_conn
        .call_method(
            Some("org.freedesktop.PolicyKit1"),
            "/org/freedesktop/PolicyKit1/Authority",
            Some("org.freedesktop.PolicyKit1.Authority"),
            "RegisterAuthenticationAgent",
            &(
                ("unix-session", session_details),
                locale.as_str(),
                object_path,
            ),
        )
        .await;

    if let Err(err) = reg_result {
        crate::log_warn!(
            "POLKIT",
            "unix-session registration failed ({err:?}), attempting unix-process fallback..."
        );

        let pid = std::process::id();

        let mut process_details: HashMap<String, Value> = HashMap::new();

        process_details.insert("pid".to_string(), Value::from(pid));

        process_details.insert("start-time".to_string(), Value::from(0u64));

        system_conn
            .call_method(
                Some("org.freedesktop.PolicyKit1"),
                "/org/freedesktop/PolicyKit1/Authority",
                Some("org.freedesktop.PolicyKit1.Authority"),
                "RegisterAuthenticationAgent",
                &(
                    ("unix-process", process_details),
                    locale.as_str(),
                    object_path,
                ),
            )
            .await
            .context("Failed to register Polkit agent with Authority via unix-process")?;
    }

    crate::log_info!(
        "POLKIT",
        "Successfully registered Polkit AuthenticationAgent on System Bus with PolicyKit1.Authority"
    );

    // Keep service active permanently until system connection is closed
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
    loop {
        interval.tick().await;
        if system_conn.is_closed() {
            break;
        }
    }

    Err(anyhow::anyhow!("System D-Bus connection closed"))
}

async fn run_helper_process(
    path: &str,
    args: &[&str],
    stdin_data: &str,
    timeout_secs: u64,
) -> Result<std::process::Output, String> {
    let mut cmd = Command::new(path);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("No se pudo iniciar {path}: {e}"))?;

    let stdin_bytes = stdin_data.as_bytes().to_vec();
    let stdin_pipe = child.stdin.take();
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();

    let stdin_fut = async move {
        if let Some(mut in_p) = stdin_pipe {
            use tokio::io::AsyncWriteExt;
            let _ = in_p.write_all(&stdin_bytes).await;
            let _ = in_p.flush().await;
        }
    };

    let stdout_fut = async move {
        let mut buf = Vec::new();
        if let Some(mut out) = stdout_pipe {
            use tokio::io::AsyncReadExt;
            let _ = out.read_to_end(&mut buf).await;
        }
        buf
    };

    let stderr_fut = async move {
        let mut buf = Vec::new();
        if let Some(mut err) = stderr_pipe {
            use tokio::io::AsyncReadExt;
            let _ = err.read_to_end(&mut buf).await;
        }
        buf
    };

    let status_fut = async { child.wait().await };

    let wait_all = async move {
        let (_, stdout_buf, stderr_buf, status_res) =
            tokio::join!(stdin_fut, stdout_fut, stderr_fut, status_fut);
        match status_res {
            Ok(status) => Ok(std::process::Output {
                status,
                stdout: stdout_buf,
                stderr: stderr_buf,
            }),
            Err(e) => Err(format!("Error en ejecución: {e}")),
        }
    };

    match tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), wait_all).await {
        Ok(res) => res,
        Err(_) => {
            let _ = child.start_kill();
            Err("El proceso sobrepasó el tiempo límite.".to_string())
        }
    }
}

pub async fn authenticate_user(
    user_name: &str,
    cookie: &str,
    password: &str,
) -> Result<(), String> {
    let cookie = cookie.trim();
    let password = password.trim_end_matches(&['\r', '\n'][..]);

    if password.is_empty() {
        return Err("La contraseña no puede estar vacía.".to_string());
    }

    let helper_paths = [
        "/usr/lib/polkit-1/polkit-agent-helper-1",
        "/usr/libexec/polkit-agent-helper-1",
        "/usr/lib/policykit-1/polkit-agent-helper-1",
        "/usr/libexec/polkit-1/polkit-agent-helper-1",
    ];

    let helper = helper_paths
        .iter()
        .find(|path| std::path::Path::new(path).exists())
        .copied()
        .ok_or_else(|| {
            "No se encontró el ejecutable polkit-agent-helper-1 en el sistema.".to_string()
        })?;

    let helper_path = std::path::Path::new(helper);
    let has_setuid = is_setuid(helper_path);

    if has_setuid {
        crate::log_info!(
            "POLKIT",
            "Invoking setuid helper {} for user '{}'...",
            helper,
            user_name
        );

        let stdin_content = format!("{cookie}\n{password}\n");
        match run_helper_process(helper, &[user_name], &stdin_content, 15).await {
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let ok = output.status.success();
                crate::log_info!(
                    "POLKIT",
                    "polkit-agent-helper-1 result: success={ok}, code={:?}, stderr='{}'",
                    output.status.code(),
                    stderr.trim()
                );
                if ok {
                    return Ok(());
                }
                let msg = extract_stderr_message(&stderr);
                Err(msg)
            }
            Err(err) => Err(format!("Error en autenticación: {err}")),
        }
    } else {
        crate::log_info!(
            "POLKIT",
            "Helper {} lacks setuid bit. Running polkit-agent-helper-1 via sudo for user '{}'...",
            helper,
            user_name
        );

        let _ = run_helper_process("sudo", &["-k"], "", 2).await;

        let stdin_sudo_helper = format!("{password}\n{cookie}\n{password}\n");
        match run_helper_process("sudo", &["-S", helper, user_name], &stdin_sudo_helper, 15).await {
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let ok = output.status.success();
                crate::log_info!(
                    "POLKIT",
                    "polkit-agent-helper-1 via sudo result: success={ok}, code={:?}, stderr='{}'",
                    output.status.code(),
                    stderr.trim()
                );
                if ok {
                    return Ok(());
                }
                let msg = extract_stderr_message(&stderr);
                Err(msg)
            }
            Err(err) => Err(format!("Error en autenticación: {err}")),
        }
    }
}

fn extract_stderr_message(stderr: &str) -> String {
    let clean: Vec<&str> = stderr
        .lines()
        .map(|l| l.trim())
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with("[sudo]")
                && !l.contains("needs to be setuid")
                && !l.starts_with("polkit-agent-helper-1: pam_authenticate failed")
        })
        .collect();

    if clean.is_empty() {
        "Contraseña incorrecta. Inténtalo de nuevo.".to_string()
    } else {
        clean.join(" ")
    }
}
