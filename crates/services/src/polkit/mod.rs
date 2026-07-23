use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::sync::oneshot;
use zbus::zvariant::Value;
use zbus::{connection, interface};

#[derive(Debug, Clone)]
pub struct PolkitAuthRequest {
    pub action_id: String,
    pub message: String,
    pub icon_name: String,
    pub user_name: String,
    pub cookie: String,
}

pub struct PolkitAgentServer {
    request_tx: UnboundedSender<(PolkitAuthRequest, oneshot::Sender<Result<(), String>>)>,
}

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
        let _ = (details, identities);
        let user_name = std::env::var("USER").unwrap_or_else(|_| "root".to_string());

        let req = PolkitAuthRequest {
            action_id,
            message,
            icon_name,
            user_name,
            cookie,
        };

        let (tx, rx) = oneshot::channel();
        if self.request_tx.send((req, tx)).is_err() {
            return Err(zbus::fdo::Error::Failed("Agent unreachable".to_string()));
        }

        match rx.await {
            Ok(Ok(())) => Ok(()),
            _ => Err(zbus::fdo::Error::Failed(
                "Authentication failed".to_string(),
            )),
        }
    }
}

pub fn start_polkit_agent()
-> Result<UnboundedReceiver<(PolkitAuthRequest, oneshot::Sender<Result<(), String>>)>> {
    let (tx, rx) = unbounded_channel();

    let server = PolkitAgentServer { request_tx: tx };

    tokio::spawn(async move {
        if let Err(err) = register_agent(server).await {
            eprintln!("Polkit Agent warning: {err}");
        }
    });

    Ok(rx)
}

async fn register_agent(server: PolkitAgentServer) -> Result<()> {
    let _session_conn = connection::Builder::session()?
        .name("org.freedesktop.PolicyKit1.AuthenticationAgent")?
        .serve_at("/org/freedesktop/PolicyKit1/AuthenticationAgent", server)?
        .build()
        .await?;

    let system_conn = connection::Builder::system()?.build().await?;

    let mut session_details: HashMap<&str, Value> = HashMap::new();
    session_details.insert("session-id", Value::from("2"));

    let reply = system_conn
        .call_method(
            Some("org.freedesktop.PolicyKit1"),
            "/org/freedesktop/PolicyKit1/Authority",
            Some("org.freedesktop.PolicyKit1.Authority"),
            "RegisterAuthenticationAgent",
            &(
                ("unix-session", session_details),
                "es_ES.UTF-8",
                "/org/freedesktop/PolicyKit1/AuthenticationAgent",
            ),
        )
        .await
        .context("Failed to register Polkit agent with Authority")?;

    let _: () = reply
        .body()
        .deserialize()
        .context("Failed to parse reply body")?;

    Ok(())
}

pub async fn authenticate_user(password: &str) -> bool {
    let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());

    if let Ok(mut child) = Command::new("/usr/bin/unix_chkpwd")
        .arg(&user)
        .arg("nullok")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(password.as_bytes()).await;
            let _ = stdin.write_all(b"\0").await;
        }

        if let Ok(status) = child.wait().await {
            return status.success();
        }
    }

    false
}
