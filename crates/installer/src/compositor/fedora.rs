use crate::compositor::{Compositor, CompositorBackend};
use std::process::Command;

pub struct FedoraCompositorBackend;

impl Default for FedoraCompositorBackend {
    fn default() -> Self {
        Self
    }
}

impl CompositorBackend for FedoraCompositorBackend {
    fn distro_name(&self) -> &str {
        "Fedora"
    }

    fn build_compositor_install_command(&self, compositor: Compositor) -> Command {
        let script = match compositor {
            Compositor::Hyprland => concat!(
                "set -e && ",
                "echo '==> Enabling solopasha/hyprland COPR and installing Hyprland...' && ",
                "dnf copr enable -y solopasha/hyprland && ",
                "dnf install -y hyprland xdg-desktop-portal-hyprland"
            ),
            Compositor::Niri => concat!(
                "set -e && ",
                "echo '==> Enabling yalter/niri COPR and installing Niri...' && ",
                "dnf copr enable -y yalter/niri && ",
                "dnf install -y niri xwayland-satellite"
            ),
            Compositor::MangoWm => concat!(
                "set -e && ",
                "echo '==> Installing MangoWM via Fedora COPR (with source fallback)...' && ",
                "if dnf copr enable -y dennemann/MangoWC && dnf install -y mangowc; then exit 0; fi && ",
                "echo '==> COPR build unavailable for current release, compiling MangoWM from source...' && ",
                "dnf install -y meson ninja-build gcc gcc-c++ git pkgconf-pkg-config wayland-devel ",
                "wayland-protocols-devel libinput-devel libxkbcommon-devel pixman-devel libdrm-devel ",
                "pcre2-devel pango-devel json-c-devel xorg-x11-server-Xwayland-devel libxcb-devel ",
                "libseat-devel hwdata-devel libdisplay-info-devel libliftoff-devel mesa-libEGL-devel ",
                "mesa-libGLES-devel vulkan-loader-devel glslang && ",
                "BUILD_DIR=$(mktemp -d /tmp/capsule-mango-XXXXXX) && ",
                "trap 'rm -rf \"$BUILD_DIR\"' EXIT && ",
                "git clone -b 0.18.2 --depth 1 https://gitlab.freedesktop.org/wlroots/wlroots.git \"$BUILD_DIR/wlroots\" && ",
                "meson setup \"$BUILD_DIR/wlroots/build\" \"$BUILD_DIR/wlroots\" -Dprefix=/usr && ",
                "ninja -C \"$BUILD_DIR/wlroots/build\" install && ",
                "git clone -b 0.4 --depth 1 https://github.com/wlrfx/scenefx.git \"$BUILD_DIR/scenefx\" && ",
                "meson setup \"$BUILD_DIR/scenefx/build\" \"$BUILD_DIR/scenefx\" -Dprefix=/usr && ",
                "ninja -C \"$BUILD_DIR/scenefx/build\" install && ",
                "git clone --depth 1 https://github.com/mangowm/mango.git \"$BUILD_DIR/mango\" && ",
                "meson setup \"$BUILD_DIR/mango/build\" \"$BUILD_DIR/mango\" -Dprefix=/usr && ",
                "ninja -C \"$BUILD_DIR/mango/build\" install"
            ),
        };

        let mut cmd = Command::new("sudo");
        cmd.args(["-n", "sh", "-c", script]);
        cmd
    }
}
