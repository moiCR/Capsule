use crate::compositor::{Compositor, CompositorBackend};
use std::process::Command;

pub struct DebianCompositorBackend;

impl Default for DebianCompositorBackend {
    fn default() -> Self {
        Self
    }
}

impl DebianCompositorBackend {
    pub fn build_apt_compositor_command(compositor: Compositor, is_ubuntu: bool) -> Command {
        let script = match compositor {
            Compositor::Hyprland => {
                if is_ubuntu {
                    concat!(
                        "set -e && export DEBIAN_FRONTEND=noninteractive && ",
                        "apt-get update -y && ",
                        "if apt-cache show hyprland >/dev/null 2>&1; then ",
                        "  echo '==> Installing Hyprland from Ubuntu repositories...' && ",
                        "  apt-get install -y hyprland xdg-desktop-portal-hyprland; ",
                        "else ",
                        "  echo '==> Hyprland not in default APT release, building from source...' && ",
                        "  apt-get install -y git build-essential cmake meson ninja-build pkg-config ",
                        "  libwayland-dev wayland-protocols libdrm-dev libxkbcommon-dev libinput-dev ",
                        "  libpixman-1-dev libseat-dev libgbm-dev libegl1-mesa-dev libgles2-mesa-dev ",
                        "  libudev-dev libdisplay-info-dev libliftoff-dev libxcb-dri3-dev libxcb-present-dev ",
                        "  libxcb-composite0-dev libxcb-render-util0-dev libxcb-res0-dev libxcb-icccm4-dev ",
                        "  libxcb-ewmh-dev libxcb-xinput-dev libxcb-errors-dev xwayland glslang-tools ",
                        "  libtomlplusplus-dev libzip-dev librsvg2-dev libcairo2-dev libpango1.0-dev && ",
                        "  BUILD_DIR=$(mktemp -d /tmp/capsule-hyprland-XXXXXX) && ",
                        "  trap 'rm -rf \"$BUILD_DIR\"' EXIT && ",
                        "  git clone --recursive --depth 1 https://github.com/hyprwm/Hyprland.git \"$BUILD_DIR/Hyprland\" && ",
                        "  cmake --no-warn-unused-cli -DCMAKE_BUILD_TYPE:STRING=Release -DCMAKE_INSTALL_PREFIX:STRING=/usr ",
                        "    -S \"$BUILD_DIR/Hyprland\" -B \"$BUILD_DIR/Hyprland/build\" -G Ninja && ",
                        "  cmake --build \"$BUILD_DIR/Hyprland/build\" --config Release --target all && ",
                        "  cmake --install \"$BUILD_DIR/Hyprland/build\"; ",
                        "fi"
                    )
                } else {
                    concat!(
                        "set -e && export DEBIAN_FRONTEND=noninteractive && ",
                        "apt-get update -y && ",
                        "if apt-cache show hyprland >/dev/null 2>&1; then ",
                        "  echo '==> Installing Hyprland from Debian repositories...' && ",
                        "  apt-get install -y hyprland xdg-desktop-portal-hyprland; ",
                        "else ",
                        "  echo '==> Hyprland not in current Debian APT suite, building from source...' && ",
                        "  apt-get install -y git build-essential cmake meson ninja-build pkg-config ",
                        "  libwayland-dev wayland-protocols libdrm-dev libxkbcommon-dev libinput-dev ",
                        "  libpixman-1-dev libseat-dev libgbm-dev libegl1-mesa-dev libgles2-mesa-dev ",
                        "  libudev-dev libdisplay-info-dev libliftoff-dev libxcb-dri3-dev libxcb-present-dev ",
                        "  libxcb-composite0-dev libxcb-render-util0-dev libxcb-res0-dev libxcb-icccm4-dev ",
                        "  libxcb-ewmh-dev libxcb-xinput-dev libxcb-errors-dev xwayland glslang-tools ",
                        "  libtomlplusplus-dev libzip-dev librsvg2-dev libcairo2-dev libpango1.0-dev && ",
                        "  BUILD_DIR=$(mktemp -d /tmp/capsule-hyprland-XXXXXX) && ",
                        "  trap 'rm -rf \"$BUILD_DIR\"' EXIT && ",
                        "  git clone --recursive --depth 1 https://github.com/hyprwm/Hyprland.git \"$BUILD_DIR/Hyprland\" && ",
                        "  cmake --no-warn-unused-cli -DCMAKE_BUILD_TYPE:STRING=Release -DCMAKE_INSTALL_PREFIX:STRING=/usr ",
                        "    -S \"$BUILD_DIR/Hyprland\" -B \"$BUILD_DIR/Hyprland/build\" -G Ninja && ",
                        "  cmake --build \"$BUILD_DIR/Hyprland/build\" --config Release --target all && ",
                        "  cmake --install \"$BUILD_DIR/Hyprland/build\"; ",
                        "fi"
                    )
                }
            }
            Compositor::Niri => concat!(
                "set -e && export DEBIAN_FRONTEND=noninteractive && ",
                "apt-get update -y && ",
                "if apt-cache show niri >/dev/null 2>&1; then ",
                "  echo '==> Installing Niri from APT repositories...' && ",
                "  apt-get install -y niri; ",
                "else ",
                "  echo '==> Installing build dependencies and compiling Niri from source...' && ",
                "  apt-get install -y gcc clang pkg-config libudev-dev libgbm-dev libxkbcommon-dev ",
                "  libegl1-mesa-dev libwayland-dev libinput-dev libdbus-1-dev libsystemd-dev ",
                "  libseat-dev libpipewire-0.3-dev libpango1.0-dev libdisplay-info-dev git curl ",
                "  build-essential cargo rustc && ",
                "  BUILD_DIR=$(mktemp -d /tmp/capsule-niri-XXXXXX) && ",
                "  trap 'rm -rf \"$BUILD_DIR\"' EXIT && ",
                "  git clone --depth 1 https://github.com/YaLTeR/niri.git \"$BUILD_DIR/niri\" && ",
                "  cargo build --release --manifest-path \"$BUILD_DIR/niri/Cargo.toml\" && ",
                "  install -Dm755 \"$BUILD_DIR/niri/target/release/niri\" /usr/local/bin/niri && ",
                "  install -Dm755 \"$BUILD_DIR/niri/resources/niri-session\" /usr/local/bin/niri-session && ",
                "  install -Dm644 \"$BUILD_DIR/niri/resources/niri.desktop\" /usr/share/wayland-sessions/niri.desktop; ",
                "fi"
            ),
            Compositor::KWin => concat!(
                "set -e && export DEBIAN_FRONTEND=noninteractive && ",
                "echo '==> Installing KWin Wayland from repositories...' && ",
                "apt-get update -y && ",
                "apt-get install -y kwin-wayland"
            ),
            Compositor::MangoWm => concat!(
                "set -e && export DEBIAN_FRONTEND=noninteractive && ",
                "echo '==> Installing MangoWM build dependencies via APT...' && ",
                "apt-get update -y && ",
                "apt-get install -y meson ninja-build git build-essential pkg-config libwayland-dev ",
                "wayland-protocols libinput-dev libxkbcommon-dev libpixman-1-dev libdrm-dev ",
                "libpcre2-dev libpango1.0-dev libjson-c-dev xwayland libxcb-render0-dev ",
                "libxcb-shape0-dev libxcb-xfixes0-dev libxcb-icccm4-dev libxcb-composite0-dev ",
                "libxcb-res0-dev libxcb-ewmh-dev libseat-dev hwdata libdisplay-info-dev ",
                "libliftoff-dev libegl1-mesa-dev libgles2-mesa-dev libgbm-dev libvulkan-dev glslang-tools && ",
                "BUILD_DIR=$(mktemp -d /tmp/capsule-mango-XXXXXX) && ",
                "trap 'rm -rf \"$BUILD_DIR\"' EXIT && ",
                "echo '==> Compiling wlroots for MangoWM...' && ",
                "git clone -b 0.18.2 --depth 1 https://gitlab.freedesktop.org/wlroots/wlroots.git \"$BUILD_DIR/wlroots\" && ",
                "meson setup \"$BUILD_DIR/wlroots/build\" \"$BUILD_DIR/wlroots\" -Dprefix=/usr && ",
                "ninja -C \"$BUILD_DIR/wlroots/build\" install && ",
                "echo '==> Compiling scenefx for MangoWM...' && ",
                "git clone -b 0.4 --depth 1 https://github.com/wlrfx/scenefx.git \"$BUILD_DIR/scenefx\" && ",
                "meson setup \"$BUILD_DIR/scenefx/build\" \"$BUILD_DIR/scenefx\" -Dprefix=/usr && ",
                "ninja -C \"$BUILD_DIR/scenefx/build\" install && ",
                "echo '==> Compiling and installing MangoWM...' && ",
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

impl CompositorBackend for DebianCompositorBackend {
    fn distro_name(&self) -> &str {
        "Debian"
    }

    fn build_compositor_install_command(&self, compositor: Compositor) -> Command {
        Self::build_apt_compositor_command(compositor, false)
    }
}
