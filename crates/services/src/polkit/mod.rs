use anyhow::{Context, Result};
use std::collections::{HashMap, VecDeque};
use std::process::Stdio;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use zbus::zvariant::{OwnedValue, Value};
use zbus::{connection, interface};

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

    pub async fn authenticate(&self, password: &str) -> bool {
        authenticate_user(password).await
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

                if let Ok(builder) = connection::Builder::system() {
                    if let Ok(system_conn) = builder.build().await {
                        let response_res = system_conn
                            .call_method(
                                Some("org.freedesktop.PolicyKit1"),
                                "/org/freedesktop/PolicyKit1/Authority",
                                Some("org.freedesktop.PolicyKit1.Authority"),
                                "AuthenticationAgentResponse2",
                                &(
                                    current_uid,
                                    cookie.as_str(),
                                    (identity_kind.as_str(), identity_details),
                                ),
                            )
                            .await;

                        if let Err(err) = response_res {
                            crate::log_warn!(
                                "POLKIT",
                                "AuthenticationAgentResponse2 failed: {err:?}"
                            );
                        } else {
                            crate::log_info!(
                                "POLKIT",
                                "Polkit Authority accepted AuthenticationAgentResponse2!"
                            );
                        }
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

pub async fn authenticate_user(password: &str) -> bool {
    if password.is_empty() {
        return false;
    }

    // Invalidate sudo timestamp first
    let _ = Command::new("sudo").arg("-k").output().await;

    // Spawn sudo -S -v to validate user password against PAM
    let mut child = match Command::new("sudo")
        .arg("-S")
        .arg("-v")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(password.as_bytes()).await;
        let _ = stdin.write_all(b"\n").await;
        let _ = stdin.flush().await;
        drop(stdin);
    }

    if let Ok(output) = child.wait_with_output().await {
        let ok = output.status.success();
        crate::log_info!("POLKIT", "Password check result: success={ok}");
        return ok;
    }

    false
}
