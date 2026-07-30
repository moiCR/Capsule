pub mod hyprland;
pub mod kinetic;
pub mod niri;

use arc_swap::ArcSwap;
use std::sync::Arc;
use std::time::Duration;

pub trait Compositor: Send + Sync {
    fn get_refresh_rate(&self) -> f64;
}

#[derive(Clone)]
pub struct CompositorService {
    refresh_rate: Arc<ArcSwap<f64>>,
}

impl CompositorService {
    pub fn new() -> Self {
        let refresh_rate = Arc::new(ArcSwap::from_pointee(60.0));
        let service = Self { refresh_rate };

        let service_clone = service.clone();
        tokio::spawn(async move {
            service_clone.run_polling_loop().await;
        });

        service
    }

    pub fn get_refresh_rate(&self) -> f64 {
        **self.refresh_rate.load()
    }

    pub fn get_frame_duration(&self) -> Duration {
        let rate = self.get_refresh_rate().max(30.0);
        let micros = (1_000_000.0 / rate).round() as u64;
        Duration::from_micros(micros)
    }

    pub fn get_frame_duration_ms(&self) -> u64 {
        let rate = self.get_refresh_rate().max(30.0);
        (1000.0 / rate).round().max(1.0) as u64
    }

    async fn run_polling_loop(&self) {
        // Determine backend once, up front.
        let use_hyprland = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();
        let use_niri = !use_hyprland && std::env::var("NIRI_SOCKET").is_ok();

        loop {
            // Call get_refresh_rate in a blocking thread so that:
            //  1. Any panic (e.g. old hyprland crate IPC bug) stays in that thread.
            //  2. Blocking I/O doesn't starve the async runtime.
            let rate = tokio::task::spawn_blocking(move || {
                std::panic::catch_unwind(|| {
                    if use_hyprland {
                        hyprland::Hyprland::new().get_refresh_rate()
                    } else if use_niri {
                        niri::Niri::new().get_refresh_rate()
                    } else {
                        60.0
                    }
                })
                .unwrap_or(60.0)
            })
            .await
            .unwrap_or(60.0);

            if rate > 0.0 {
                let prev = **self.refresh_rate.load();
                if (prev - rate).abs() > 0.1 {
                    crate::log_info!(
                        "COMPOSITOR",
                        "Detected active monitor refresh rate: {rate:.2} Hz (frame duration: {:.2} ms)",
                        1000.0 / rate
                    );
                }
                self.refresh_rate.store(Arc::new(rate));
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

impl Default for CompositorService {
    fn default() -> Self {
        Self::new()
    }
}
