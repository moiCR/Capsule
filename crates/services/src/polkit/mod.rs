use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
use std::os::unix::fs::PermissionsExt;
use std::process::Stdio;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::io::AsyncWriteExt;
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
static POLKIT_AGENT_CONN: OnceLock<zbus::Connection> = OnceLock::new();

fn polkit_queue() -> &'static Arc<Mutex<VecDeque<PolkitPendingAuth>>> {
    POLKIT_QUEUE.get_or_init(|| Arc::new(Mutex::new(VecDeque::new())))
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
            identity_kind: identity_kind.clone(),
            identity_details: identity_details.clone(),
        };

        crate::log_info!(
            "POLKIT",
            "Polkit auth request received: action='{action_id}', msg='{message}', cookie='{cookie}'"
        );

        let (tx, rx) = tokio::sync::oneshot::channel();
        push_polkit_request(req, tx);

        match rx.await {
            Ok(Ok(())) => {
                crate::log_info!(
                    "POLKIT",
                    "User authentication verified, sending AuthenticationAgentResponse2 to Authority..."
                );

                let agent_conn_opt = POLKIT_AGENT_CONN.get().cloned();
                if let Some(system_conn) = agent_conn_opt {
                    let response_res = system_conn
                        .call_method(
                            Some("org.freedesktop.PolicyKit1"),
                            "/org/freedesktop/PolicyKit1/Authority",
                            Some("org.freedesktop.PolicyKit1.Authority"),
                            "AuthenticationAgentResponse2",
                            &(
                                current_uid,
                                cookie.as_str(),
                                (identity_kind.as_str(), &identity_details),
                            ),
                        )
                        .await;

                    if let Err(err) = response_res {
                        crate::log_warn!(
                            "POLKIT",
                            "AuthenticationAgentResponse2 failed ({err:?}), falling back to AuthenticationAgentResponse..."
                        );
                        let fallback_res = system_conn
                            .call_method(
                                Some("org.freedesktop.PolicyKit1"),
                                "/org/freedesktop/PolicyKit1/Authority",
                                Some("org.freedesktop.PolicyKit1.Authority"),
                                "AuthenticationAgentResponse",
                                &(cookie.as_str(), (identity_kind.as_str(), &identity_details)),
                            )
                            .await;

                        if let Err(fb_err) = fallback_res {
                            crate::log_warn!(
                                "POLKIT",
                                "AuthenticationAgentResponse fallback also failed: {fb_err:?}"
                            );
                        } else {
                            crate::log_info!(
                                "POLKIT",
                                "Polkit Authority accepted AuthenticationAgentResponse fallback!"
                            );
                        }
                    } else {
                        crate::log_info!(
                            "POLKIT",
                            "Polkit Authority accepted AuthenticationAgentResponse2!"
                        );
                    }
                }

                Ok(())
            }
            _ => {
                crate::log_warn!(
                    "POLKIT",
                    "Polkit authentication failed or cancelled by user!"
                );
                Err(zbus::fdo::Error::Failed(
                    "Authentication failed".to_string(),
                ))
            }
        }
    }
}

pub fn start_polkit_agent() {
    tokio::spawn(async move {
        let server = PolkitAgentServer;
        if let Err(err) = register_agent(server).await {
            crate::log_warn!("POLKIT", "Polkit Agent warning: {err}");
        }
    });
}

async fn register_agent(server: PolkitAgentServer) -> Result<()> {
    let system_conn = connection::Builder::system()?
        .serve_at("/org/freedesktop/PolicyKit1/AuthenticationAgent", server)?
        .build()
        .await?;

    let _ = POLKIT_AGENT_CONN.set(system_conn.clone());

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

    std::future::pending::<()>().await;

    Ok(())
}

pub async fn authenticate_user(
    user_name: &str,
    cookie: &str,
    password: &str,
) -> Result<(), String> {
    if password.is_empty() {
        return Err("La contraseña no puede estar vacía.".to_string());
    }

    let helper_paths = [
        "/usr/lib/polkit-1/polkit-agent-helper-1",
        "/usr/libexec/polkit-agent-helper-1",
        "/usr/lib/policykit-1/polkit-agent-helper-1",
        "/usr/libexec/polkit-1/polkit-agent-helper-1",
    ];

    let mut helper_bin = None;
    for path in &helper_paths {
        if std::path::Path::new(path).exists() {
            helper_bin = Some(*path);
            break;
        }
    }

    if !cookie.is_empty() {
        if let Some(helper) = helper_bin {
            let helper_path = std::path::Path::new(helper);
            if !is_setuid(helper_path) {
                crate::log_info!(
                    "POLKIT",
                    "polkit-agent-helper-1 ({}) lacks setuid bit. Repairing permission via sudo chmod u+s...",
                    helper
                );

                if let Ok(mut sudo_chmod) = Command::new("sudo")
                    .arg("-S")
                    .arg("chmod")
                    .arg("u+s")
                    .arg(helper)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    if let Some(mut stdin) = sudo_chmod.stdin.take() {
                        let _ = stdin.write_all(password.as_bytes()).await;
                        let _ = stdin.write_all(b"\n").await;
                        let _ = stdin.flush().await;
                        drop(stdin);
                    }
                    let _ = sudo_chmod.wait_with_output().await;
                }
            }

            crate::log_info!(
                "POLKIT",
                "Invoking {} for user '{}' with active cookie...",
                helper,
                user_name
            );

            let child = Command::new(helper)
                .arg(user_name)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            if let Ok(mut c) = child {
                if let Some(mut stdin) = c.stdin.take() {
                    let _ = stdin.write_all(cookie.as_bytes()).await;
                    let _ = stdin.write_all(b"\n").await;
                    let _ = stdin.write_all(password.as_bytes()).await;
                    let _ = stdin.write_all(b"\n").await;
                    let _ = stdin.flush().await;
                    drop(stdin);
                }

                if let Ok(output) = c.wait_with_output().await {
                    let ok = output.status.success();
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    crate::log_info!(
                        "POLKIT",
                        "polkit-agent-helper-1 result: success={ok}, code={:?}, stderr='{}'",
                        output.status.code(),
                        stderr.trim()
                    );

                    if ok {
                        return Ok(());
                    }

                    let stderr_lower = stderr.to_lowercase();
                    if stderr_lower.contains("delay")
                        || stderr_lower.contains("lockout")
                        || stderr_lower.contains("try again")
                        || stderr_lower.contains("pruebe otra vez")
                    {
                        return Err(
                            "Contraseña incorrecta o límite de reintentos alcanzado. Por favor espera unos segundos..."
                                .to_string(),
                        );
                    }

                    return Err("Contraseña incorrecta. Inténtalo de nuevo.".to_string());
                }
            }
        }
    }

    // Fallback: Sudo PAM check
    let _ = Command::new("sudo").arg("-k").output().await;

    let mut child = match Command::new("sudo")
        .arg("-S")
        .arg("-v")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            crate::log_warn!("POLKIT", "Failed to spawn sudo -S -v: {e}");
            return Err("No se pudo iniciar la verificación PAM.".to_string());
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(password.as_bytes()).await;
        let _ = stdin.write_all(b"\n").await;
        let _ = stdin.flush().await;
        drop(stdin);
    }

    if let Ok(output) = child.wait_with_output().await {
        let ok = output.status.success();
        let stderr = String::from_utf8_lossy(&output.stderr);

        if ok {
            return Ok(());
        }

        let stderr_lower = stderr.to_lowercase();
        if stderr_lower.contains("delay")
            || stderr_lower.contains("lockout")
            || stderr_lower.contains("try again")
            || stderr_lower.contains("pruebe otra vez")
        {
            return Err(
                "Contraseña incorrecta o límite de reintentos alcanzado. Por favor espera unos segundos..."
                    .to_string(),
            );
        }

        return Err("Contraseña incorrecta. Inténtalo de nuevo.".to_string());
    }

    Err("Error inesperado en la verificación de credenciales.".to_string())
}
