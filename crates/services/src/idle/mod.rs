use crate::config::AppConfig;
use crate::ipc::{IpcCommand, push_ipc_command};
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub enum IdleEvent {
    IdleTimeoutTriggered,
}

#[derive(Clone)]
pub struct IdleService {
    tx: broadcast::Sender<IdleEvent>,
}

impl Default for IdleService {
    fn default() -> Self {
        Self::new()
    }
}

impl IdleService {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(16);
        let service = Self { tx };

        let svc_dbus = service.clone();
        tokio::spawn(async move {
            svc_dbus.start_dbus_lock_listener().await;
        });

        let svc_input = service.clone();
        tokio::spawn(async move {
            svc_input.start_physical_input_tracker().await;
        });

        service
    }

    pub fn subscribe(&self) -> broadcast::Receiver<IdleEvent> {
        self.tx.subscribe()
    }

    async fn start_physical_input_tracker(&self) {
        let config = AppConfig::load();
        let timeout_secs = config.lockscreen.idle_timeout;
        if timeout_secs == 0 {
            return;
        }

        let (tx_activity, mut rx_activity) = tokio::sync::mpsc::unbounded_channel::<()>();

        // 1. Spawn Hyprland socket2 listener for real-time keyboard/mouse/touch events
        tokio::spawn(start_hyprland_socket2_listener(tx_activity));

        let mut last_activity = std::time::Instant::now();
        let mut last_cursor_pos = String::new();

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            let mut has_activity = false;

            // Check activity events received from Hyprland IPC socket2
            while rx_activity.try_recv().is_ok() {
                has_activity = true;
            }

            // 2. Check hyprctl cursorpos for mouse movement fallback
            if let Ok(output) = std::process::Command::new("hyprctl")
                .arg("cursorpos")
                .output()
            {
                if output.status.success() {
                    let pos_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !pos_str.is_empty() {
                        if last_cursor_pos.is_empty() {
                            last_cursor_pos = pos_str;
                        } else if last_cursor_pos != pos_str {
                            last_cursor_pos = pos_str;
                            has_activity = true;
                        }
                    }
                }
            }

            // 3. Check active video players (YouTube in Firefox/Chrome/Brave, MPV, VLC, etc.)
            // NON-SPOTIFY MPRIS players currently playing prevent locking.
            // Spotify is EXCLUDED: Spotify playing music does NOT prevent locking.
            let is_video_playing = {
                let players = crate::mpris::MprisService::fetch_all_players().await;
                let has_mpris_video = players.iter().any(|p| {
                    if !p.is_playing {
                        return false;
                    }
                    let bus = p.bus_name.to_lowercase();
                    let name = p.player_name.to_lowercase();
                    !bus.contains("spotify") && !name.contains("spotify")
                });

                if has_mpris_video {
                    true
                } else {
                    // Fallback check with playerctl for browser video players (e.g. firefox.instance_...)
                    if let Ok(out) = std::process::Command::new("playerctl")
                        .args(["-a", "status"])
                        .output()
                    {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        if let Ok(list_out) =
                            std::process::Command::new("playerctl").arg("-l").output()
                        {
                            let list = String::from_utf8_lossy(&list_out.stdout);
                            let lines: Vec<&str> = list.lines().collect();
                            let statuses: Vec<&str> = stdout.lines().collect();

                            lines.iter().zip(statuses.iter()).any(|(player, status)| {
                                status.trim().eq_ignore_ascii_case("Playing")
                                    && !player.to_lowercase().contains("spotify")
                            })
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            };

            if has_activity || is_video_playing {
                last_activity = std::time::Instant::now();
            } else if last_activity.elapsed().as_secs() >= timeout_secs {
                eprintln!(
                    "[IdleService] Idle timeout reached ({}s). Triggering lockscreen.",
                    timeout_secs
                );
                // Trigger lock via Capsule's permanent IPC mechanism
                push_ipc_command(IpcCommand::Lock);
                let _ = self.tx.send(IdleEvent::IdleTimeoutTriggered);
                last_activity = std::time::Instant::now();
            }
        }
    }

    async fn start_dbus_lock_listener(&self) {
        if let Some(conn) = crate::dbus_util::get_shared_system_conn().await {
            let tx1 = self.tx.clone();
            let conn1 = conn.clone();
            tokio::spawn(async move {
                if let Ok(proxy) = zbus::Proxy::new(
                    &conn1,
                    "org.freedesktop.login1",
                    "/org/freedesktop/login1",
                    "org.freedesktop.login1.Manager",
                )
                .await
                {
                    if let Ok(mut stream) = proxy.receive_signal("PrepareForSleep").await {
                        use futures::StreamExt;
                        while let Some(msg) = stream.next().await {
                            if let Ok(going_to_sleep) = msg.body().deserialize::<bool>() {
                                if going_to_sleep {
                                    push_ipc_command(IpcCommand::Lock);
                                    let _ = tx1.send(IdleEvent::IdleTimeoutTriggered);
                                }
                            }
                        }
                    }
                }
            });

            let tx2 = self.tx.clone();
            tokio::spawn(async move {
                if let Ok(proxy) = zbus::Proxy::new(
                    &conn,
                    "org.freedesktop.login1",
                    "/org/freedesktop/login1/session/auto",
                    "org.freedesktop.login1.Session",
                )
                .await
                {
                    if let Ok(mut stream) = proxy.receive_signal("Lock").await {
                        use futures::StreamExt;
                        while let Some(_) = stream.next().await {
                            push_ipc_command(IpcCommand::Lock);
                            let _ = tx2.send(IdleEvent::IdleTimeoutTriggered);
                        }
                    }
                }
            });
        }
    }
}

async fn start_hyprland_socket2_listener(tx_activity: tokio::sync::mpsc::UnboundedSender<()>) {
    let signature = match std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(s) if !s.is_empty() => s,
        _ => return,
    };

    let socket_path = if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        format!("{runtime_dir}/hypr/{signature}/.socket2.sock")
    } else {
        format!("/tmp/hypr/{signature}/.socket2.sock")
    };

    if let Ok(stream) = tokio::net::UnixStream::connect(&socket_path).await {
        use tokio::io::AsyncBufReadExt;
        let mut reader = tokio::io::BufReader::new(stream).lines();
        while let Ok(Some(_line)) = reader.next_line().await {
            let _ = tx_activity.send(());
        }
    }
}
