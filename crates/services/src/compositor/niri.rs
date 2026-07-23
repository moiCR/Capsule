use crate::compositor::Compositor;
use niri_ipc::{Request, Response, socket::Socket};
use std::panic::catch_unwind;

pub struct Niri;

impl Niri {
    pub fn new() -> Self {
        Self
    }
}

impl Compositor for Niri {
    fn get_refresh_rate(&self) -> f64 {
        let res = catch_unwind(|| {
            if let Ok(mut socket) = Socket::connect() {
                if let Ok(Ok(Response::Outputs(outputs))) = socket.send(Request::Outputs) {
                    for (_name, output) in outputs {
                        if let Some(mode_idx) = output.current_mode {
                            if let Some(mode) = output.modes.get(mode_idx) {
                                let rate = (mode.refresh_rate as f64) / 1000.0;
                                if rate > 0.0 {
                                    return rate;
                                }
                            }
                        }
                    }
                }
            }
            60.0
        });
        res.unwrap_or(60.0)
    }
}
