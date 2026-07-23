use super::{IpcCommand, decode_command, encode_command, get_socket_path};
use gpui::{App, Entity};
use std::collections::VecDeque;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::net::UnixListener as TokioUnixListener;

static IPC_QUEUE: OnceLock<Arc<Mutex<VecDeque<IpcCommand>>>> = OnceLock::new();

fn get_ipc_queue() -> &'static Arc<Mutex<VecDeque<IpcCommand>>> {
    IPC_QUEUE.get_or_init(|| Arc::new(Mutex::new(VecDeque::new())))
}

pub fn push_ipc_command(cmd: IpcCommand) {
    if let Ok(mut q) = get_ipc_queue().lock() {
        q.push_back(cmd);
    }
}

pub fn pop_ipc_command() -> Option<IpcCommand> {
    if let Ok(mut q) = get_ipc_queue().lock() {
        q.pop_front()
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct IpcMessage {
    pub id: u64,
    pub command: IpcCommand,
}

enum AcquireResult {
    Primary(IpcSubscriber),
    Secondary,
    NotRunning,
    Error(String),
}

/// Subscriber for IPC messages from other instances based on gpui-shell design.
pub struct IpcSubscriber {
    listener: Option<TokioUnixListener>,
    socket_path: PathBuf,
    pub initial_command: Option<IpcCommand>,
}

impl std::fmt::Debug for IpcSubscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IpcSubscriber")
            .field("socket_path", &self.socket_path)
            .field("initial_command", &self.initial_command)
            .finish()
    }
}

impl IpcSubscriber {
    /// Initialize IPC single-instance handling.
    ///
    /// Returns `Some(IpcSubscriber)` when this process should continue as
    /// the primary instance, otherwise `None`.
    pub fn init(cmd_arg: Option<&str>) -> Option<IpcSubscriber> {
        match Self::acquire(cmd_arg) {
            AcquireResult::Primary(subscriber) => Some(subscriber),
            AcquireResult::Secondary => None,
            AcquireResult::NotRunning => {
                eprintln!("Error: Capsule daemon is not running. Start it first with 'Capsule'.");
                std::process::exit(1);
            }
            AcquireResult::Error(err) => {
                crate::log_error!("IPC", "IPC service error: {err}. Retrying...");
                match Self::acquire(None) {
                    AcquireResult::Primary(subscriber) => Some(subscriber),
                    AcquireResult::Secondary => None,
                    AcquireResult::NotRunning => {
                        eprintln!(
                            "Error: Capsule daemon is not running. Start it first with 'Capsule'."
                        );
                        std::process::exit(1);
                    }
                    AcquireResult::Error(retry_err) => {
                        crate::log_error!(
                            "IPC",
                            "Failed to acquire IPC service on retry: {retry_err}"
                        );
                        None
                    }
                }
            }
        }
    }

    fn acquire(cmd_arg: Option<&str>) -> AcquireResult {
        let path = get_socket_path();
        let command = cmd_arg.and_then(decode_command);

        // Try to connect to existing instance (fast, synchronous path)
        if let Ok(mut stream) = UnixStream::connect(&path) {
            if let Some(ref cmd) = command {
                let _ = stream.set_write_timeout(Some(std::time::Duration::from_millis(100)));
                let payload = format!("{}\n", encode_command(cmd));
                if let Err(e) = stream.write_all(payload.as_bytes()) {
                    crate::log_error!("IPC", "Failed to send message to existing instance: {e}");
                    return AcquireResult::Error(format!(
                        "Failed to signal existing instance: {e}"
                    ));
                }
                let _ = stream.flush();
                let _ = stream.shutdown(std::net::Shutdown::Write);
                crate::log_info!(
                    "IPC",
                    "Successfully signaled existing instance with command: {:?}",
                    cmd
                );
            }
            return AcquireResult::Secondary;
        }

        // If a command was specified but no daemon is running, do NOT auto-start the daemon.
        if cmd_arg.is_some() {
            return AcquireResult::NotRunning;
        }

        // No existing instance and no command, become primary
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                crate::log_warn!("IPC", "Failed to remove stale socket: {e}");
            }
        }

        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(e) => return AcquireResult::Error(format!("Failed to bind socket: {e}")),
        };

        if let Err(e) = listener.set_nonblocking(true) {
            return AcquireResult::Error(format!("Failed to set socket non-blocking: {e}"));
        }

        let tokio_listener = match TokioUnixListener::from_std(listener) {
            Ok(l) => l,
            Err(e) => return AcquireResult::Error(format!("Failed to create tokio listener: {e}")),
        };

        crate::log_info!("IPC", "Prepared as primary instance, socket at {:?}", path);

        AcquireResult::Primary(IpcSubscriber {
            listener: Some(tokio_listener),
            socket_path: path,
            initial_command: command,
        })
    }

    pub fn start_listener(&mut self) {
        if let Some(listener) = self.listener.take() {
            let path_clone = self.socket_path.clone();
            tokio::spawn(async move {
                accept_loop(listener, path_clone).await;
            });
        }
    }

    pub fn start<T: 'static>(
        mut self,
        _cx: &mut App,
        _target_entity: Entity<T>,
        _handler: fn(&mut T, IpcCommand, &mut gpui::Context<T>),
    ) {
        if let Some(cmd) = self.initial_command.take() {
            push_ipc_command(cmd);
        }
        self.start_listener();
    }
}

async fn accept_loop(listener: TokioUnixListener, socket_path: PathBuf) {
    use std::sync::atomic::AtomicU64;
    static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

    crate::log_info!("IPC", "Socket listener started at {:?}", socket_path);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    handle_connection(stream, &REQUEST_COUNTER).await;
                });
            }
            Err(e) => {
                crate::log_error!("IPC", "Failed to accept IPC connection: {e}");
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        }
    }
}

async fn handle_connection(stream: tokio::net::UnixStream, counter: &std::sync::atomic::AtomicU64) {
    use tokio::io::{AsyncBufReadExt, BufReader};

    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    let read_result = tokio::time::timeout(
        std::time::Duration::from_millis(50),
        reader.read_line(&mut line),
    )
    .await;

    let payload = match read_result {
        Ok(Ok(_)) => line,
        Ok(Err(e)) => {
            crate::log_warn!("IPC", "Error reading from socket: {e}");
            String::new()
        }
        Err(_) => {
            crate::log_warn!("IPC", "Timeout reading from socket");
            String::new()
        }
    };

    if let Some(command) = decode_command(&payload) {
        let request_id = counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        crate::log_info!(
            "IPC",
            "Received & queued command #{request_id}: {:?}",
            command
        );
        push_ipc_command(command);
    }
}
