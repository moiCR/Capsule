use crate::compositor::{Compositor, CompositorBackend};
use crate::system::{self, AurHelper};
use std::process::Command;

pub struct ArchCompositorBackend {
    helper: Option<AurHelper>,
}

impl Default for ArchCompositorBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchCompositorBackend {
    pub fn new() -> Self {
        Self {
            helper: system::detect_aur_helper(),
        }
    }

    fn pacman_or_helper_install(&self, packages: &[&str]) -> Command {
        match &self.helper {
            Some(AurHelper::Paru) => {
                let mut cmd = Command::new("paru");
                cmd.args(["-S", "--needed", "--noconfirm"]);
                cmd.args(packages);
                cmd
            }
            Some(AurHelper::Yay) => {
                let mut cmd = Command::new("yay");
                cmd.args(["-S", "--needed", "--noconfirm"]);
                cmd.args(packages);
                cmd
            }
            _ => {
                let mut cmd = Command::new("sudo");
                cmd.args(["-n", "pacman", "-S", "--needed", "--noconfirm"]);
                cmd.args(packages);
                cmd
            }
        }
    }
}

impl CompositorBackend for ArchCompositorBackend {
    fn distro_name(&self) -> &str {
        "Arch Linux"
    }

    fn build_compositor_install_command(&self, compositor: Compositor) -> Command {
        match compositor {
            Compositor::Hyprland => {
                self.pacman_or_helper_install(&["hyprland", "xdg-desktop-portal-hyprland"])
            }
            Compositor::Niri => self.pacman_or_helper_install(&["niri", "xwayland-satellite"]),
            Compositor::MangoWm => match &self.helper {
                Some(AurHelper::Paru) => {
                    let mut cmd = Command::new("paru");
                    cmd.args(["-S", "--needed", "--noconfirm", "mangowm-git"]);
                    cmd
                }
                Some(AurHelper::Yay) => {
                    let mut cmd = Command::new("yay");
                    cmd.args(["-S", "--needed", "--noconfirm", "mangowm-git"]);
                    cmd
                }
                _ => {
                    let script = concat!(
                        "set -e && ",
                        "pacman -S --needed --noconfirm base-devel git meson ninja wayland wayland-protocols ",
                        "libinput libxkbcommon pixman libdrm pcre2 pango json-c xorg-xwayland libxcb seatd ",
                        "hwdata libdisplay-info libliftoff vulkan-icd-loader glslang && ",
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
                    );
                    let mut cmd = Command::new("sudo");
                    cmd.args(["-n", "sh", "-c", script]);
                    cmd
                }
            },
        }
    }
}
