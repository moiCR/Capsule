use crate::compositor::Compositor;

#[derive(Clone)]
pub struct KineticWE;

impl KineticWE {
    pub fn new() -> Self {
        Self
    }
}

impl Compositor for KineticWE {
    fn get_refresh_rate(&self) -> f64 {
        60.0
    }
}
