use crate::compositor::debian::DebianCompositorBackend;
use crate::compositor::{Compositor, CompositorBackend};
use std::process::Command;

pub struct UbuntuCompositorBackend;

impl Default for UbuntuCompositorBackend {
    fn default() -> Self {
        Self
    }
}

impl CompositorBackend for UbuntuCompositorBackend {
    fn distro_name(&self) -> &str {
        "Ubuntu"
    }

    fn build_compositor_install_command(&self, compositor: Compositor) -> Command {
        DebianCompositorBackend::build_apt_compositor_command(compositor, true)
    }
}
