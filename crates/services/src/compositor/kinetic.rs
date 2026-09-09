use crate::compositor::{Compositor, WorkspaceInfo};

#[derive(Clone, Default)]
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

    fn get_workspace(&self) -> Option<WorkspaceInfo> {
        Some(WorkspaceInfo::default())
    }
}
