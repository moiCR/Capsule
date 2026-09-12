#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

read_input() {
    local prompt="$1"
    local default="${2:-}"
    local reply=""
    if [ -t 0 ]; then
        read -rp "$prompt" reply
    elif [ -r /dev/tty ]; then
        read -rp "$prompt" reply < /dev/tty || reply="$default"
    else
        reply="$default"
    fi
    echo "${reply:-$default}"
}

APP_NAME="capsule"
GITHUB_REPO="moiCR/Capsule"
INSTALL_DIR="/usr/local/bin"
DESKTOP_DIR="/usr/share/applications"

if [[ $EUID -eq 0 ]]; then
    error "Please do not run this script as root or with sudo. Run it as your regular user; sudo privileges will be requested when needed."
fi

echo -e "${BLUE}"
cat << 'EOF'
  ____    _    ____  ____  _   _ _     _____ 
 / ___|  / \  |  _ \/ ___|| | | | |   | ____|
| |     / _ \ | |_) \___ \| | | | |   |  _|  
| |___ / ___ \|  __/ ___) | |_| | |___| |___ 
 \____/_/   \_\_|   |____/ \___/|_____|_____|
EOF
echo -e "${NC}"
info "Starting installation of ${APP_NAME}..."

info "Requesting sudo privileges for installation tasks..."
if ! sudo -v; then
    error "Failed to obtain sudo privileges. Aborting."
fi

while true; do sudo -n true; sleep 60; kill -0 "$$" || exit; done 2>/dev/null &
SUDO_KEEP_ALIVE_PID=$!
trap 'kill "$SUDO_KEEP_ALIVE_PID" 2>/dev/null || true' EXIT

ARCH=$(uname -m)
case "$ARCH" in
    x86_64)          TARGET="x86_64-unknown-linux-gnu" ;;
    aarch64 | arm64) TARGET="aarch64-unknown-linux-gnu" ;;
    *)               error "Unsupported architecture: $ARCH" ;;
esac

INSTALLED_BIN=""
CURRENT_VERSION=""

if command -v capsule &>/dev/null; then
    INSTALLED_BIN=$(command -v capsule)
elif [[ -f "$INSTALL_DIR/capsule" ]]; then
    INSTALLED_BIN="$INSTALL_DIR/capsule"
fi

if [[ -n "$INSTALLED_BIN" && -x "$INSTALLED_BIN" ]]; then
    RAW_VER=$("$INSTALLED_BIN" --version 2>/dev/null || echo "")
    CURRENT_VERSION=$(echo "$RAW_VER" | awk '{print $NF}')
    if [[ -n "$CURRENT_VERSION" ]]; then
        info "Found existing Capsule installation: version ${CURRENT_VERSION} at ${INSTALLED_BIN}"
    else
        info "Found existing Capsule installation at ${INSTALLED_BIN}"
    fi
fi

LATEST_VERSION=""
RELEASE_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/capsule-${TARGET}.tar.gz"

if command -v curl &>/dev/null; then
    LATEST_TAG=$(curl -sSL -m 5 "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/' || echo "")
    LATEST_VERSION="${LATEST_TAG#v}"
fi

if [[ -n "$CURRENT_VERSION" && -n "$LATEST_VERSION" && "$CURRENT_VERSION" == "$LATEST_VERSION" ]]; then
    warn "Capsule is already installed and up to date (version ${CURRENT_VERSION})."
    confirm=$(read_input "Do you want to reinstall Capsule ${CURRENT_VERSION}? [y/N] " "n")
    confirm=$(echo "$confirm" | tr '[:upper:]' '[:lower:]')
    if [[ "$confirm" != "y" && "$confirm" != "yes" ]]; then
        info "Installation skipped. Existing version kept."
        exit 0
    fi
fi

WAS_RUNNING=false
if pgrep -x capsule &>/dev/null || pgrep -x Capsule &>/dev/null; then
    WAS_RUNNING=true
    info "Detected running Capsule process. Stopping it before installation..."
    if command -v capsule &>/dev/null; then
        capsule quit 2>/dev/null || true
    fi
    for _ in {1..15}; do
        if ! pgrep -x capsule &>/dev/null && ! pgrep -x Capsule &>/dev/null; then
            break
        fi
        sleep 0.2
    done

    if pgrep -x capsule &>/dev/null || pgrep -x Capsule &>/dev/null; then
        pkill -x capsule 2>/dev/null || pkill -x Capsule 2>/dev/null || true
        sleep 0.5
    fi

    if pgrep -x capsule &>/dev/null || pgrep -x Capsule &>/dev/null; then
        pkill -9 -x capsule 2>/dev/null || pkill -9 -x Capsule 2>/dev/null || true
        sleep 0.2
    fi

    success "Previous Capsule process stopped."
fi

info "Detecting Arch Linux package manager..."
AUR_HELPER=""

if command -v paru &>/dev/null; then
    AUR_HELPER="paru"
    info "Found AUR helper: paru"
elif command -v yay &>/dev/null; then
    AUR_HELPER="yay"
    info "Found AUR helper: yay"
elif command -v pacman &>/dev/null; then
    warn "No AUR helper (paru/yay) detected. Falling back to pacman."
    warn "AUR packages may need manual installation."
else
    error "No supported Arch Linux package manager found. Capsule is currently optimized for Arch-based distributions."
fi

ARCH_LINUX_DEPENDENCIES=(
    "vulkan-icd-loader"
    "libxkbcommon"
    "wayland"
    "wireplumber"
    "brightnessctl"
    "cliphist"
    "wl-clipboard"
    "qt5ct"
    "qt6ct"
    "gsettings-desktop-schemas"
    "zenity"
    "dconf"
    "nwg-look"
    "xdg-desktop-portal"
    "xdg-desktop-portal-gtk"
    "adw-gtk-theme"
    "git"
    "curl"
    "tar"
    "gpu-screen-recorder"
)

if ! command -v awww &>/dev/null && ! command -v swww &>/dev/null; then
    if [[ -n "$AUR_HELPER" ]]; then
        ARCH_LINUX_DEPENDENCIES+=("awww")
    fi
fi

TO_INSTALL=()
info "Checking Arch Linux dependencies..."

check_pkg() {
    local pkg="$1"
    if [[ -n "$AUR_HELPER" ]]; then
        "$AUR_HELPER" -Qq "$pkg" &>/dev/null
    else
        pacman -Qq "$pkg" &>/dev/null
    fi
}

for pkg in "${ARCH_LINUX_DEPENDENCIES[@]}"; do
    if check_pkg "$pkg"; then
        echo -e "  \e[1;32m✔\e[0m $pkg \e[2minstalled\e[0m"
    else
        echo -e "  \e[1;33m➜\e[0m $pkg \e[1mwill be installed\e[0m"
        TO_INSTALL+=("$pkg")
    fi
done

if [[ ${#TO_INSTALL[@]} -gt 0 ]]; then
    echo -e "----------------------------------------"
    info "Packages to install: ${#TO_INSTALL[@]}"

    confirm=$(read_input "Do you want to proceed with the installation of dependencies? [Y/n] " "y")
    confirm=$(echo "$confirm" | tr '[:upper:]' '[:lower:]')

    if [[ "$confirm" == "n" || "$confirm" == "no" ]]; then
        info "Package installation skipped by user."
    else
        if [[ -n "$AUR_HELPER" ]]; then
            info "Starting installation using $AUR_HELPER..."
            if ! "$AUR_HELPER" -S --needed "${TO_INSTALL[@]}"; then
                error "Dependency installation failed. Please check the logs above."
            fi
        else
            info "Starting installation using pacman..."
            if ! sudo pacman -S --needed "${TO_INSTALL[@]}"; then
                error "Dependency installation failed. Please check the logs above."
            fi
        fi
        success "Dependencies installed successfully!"
    fi
else
    echo -e "----------------------------------------"
    info "All required dependencies are already installed!"
fi

if ! command -v awww &>/dev/null && ! command -v swww &>/dev/null; then
    echo -e "----------------------------------------"
    info "Neither 'awww' nor 'swww' wallpaper daemon was detected."
    if command -v cargo &>/dev/null; then
        info "Compiling and installing 'awww' wallpaper daemon from source..."
        AWWW_TEMP_DIR=$(mktemp -d)
        if git clone --depth 1 https://codeberg.org/LGFae/awww.git "$AWWW_TEMP_DIR"; then
            (
                cd "$AWWW_TEMP_DIR"
                cargo build --release
            )
            if [[ -f "$AWWW_TEMP_DIR/target/release/awww" && -f "$AWWW_TEMP_DIR/target/release/awww-daemon" ]]; then
                sudo cp "$AWWW_TEMP_DIR/target/release/awww" "$INSTALL_DIR/awww"
                sudo cp "$AWWW_TEMP_DIR/target/release/awww-daemon" "$INSTALL_DIR/awww-daemon"
                sudo chmod +x "$INSTALL_DIR/awww" "$INSTALL_DIR/awww-daemon"
                success "'awww' wallpaper daemon installed successfully to $INSTALL_DIR!"
            fi
        fi
        rm -rf "$AWWW_TEMP_DIR"
    else
        warn "Cargo is not available to compile 'awww'. You can install 'awww' or 'swww' later via your package manager."
    fi
fi

echo -e "----------------------------------------"
info "Installing Capsule binary for $ARCH ($TARGET)..."
sudo mkdir -p "$INSTALL_DIR"
sudo mkdir -p "$DESKTOP_DIR"

BINARY_INSTALLED=false

SCRIPT_DIR=""
if [[ -n "${BASH_SOURCE[0]:-}" ]]; then
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
fi

if [[ -n "$SCRIPT_DIR" && -f "$SCRIPT_DIR/Cargo.toml" && -f "$SCRIPT_DIR/crates/app/src/main.rs" ]]; then
    if [[ -f "$SCRIPT_DIR/target/release/capsule" ]]; then
        info "Found locally compiled release binary in workspace..."
        sudo cp "$SCRIPT_DIR/target/release/capsule" "$INSTALL_DIR/capsule"
        sudo chmod +x "$INSTALL_DIR/capsule"
        BINARY_INSTALLED=true
        success "Capsule binary installed from local build to $INSTALL_DIR/capsule"
    fi
fi

if [[ "$BINARY_INSTALLED" == false ]]; then
    info "Attempting to download release binary from ${RELEASE_URL}..."
    TEMP_BIN_DIR=$(mktemp -d)
    if curl -sSL --fail "$RELEASE_URL" -o "$TEMP_BIN_DIR/capsule.tar.gz" 2>/dev/null; then
        info "Extracting release asset..."
        if tar -xzf "$TEMP_BIN_DIR/capsule.tar.gz" -C "$TEMP_BIN_DIR"; then
            if [[ -f "$TEMP_BIN_DIR/capsule" ]]; then
                sudo cp "$TEMP_BIN_DIR/capsule" "$INSTALL_DIR/capsule"
                sudo chmod +x "$INSTALL_DIR/capsule"
                BINARY_INSTALLED=true
                success "Capsule binary installed from GitHub Release to $INSTALL_DIR/capsule"
            fi
        fi
    fi
    rm -rf "$TEMP_BIN_DIR"
fi

if [[ "$BINARY_INSTALLED" == false ]]; then
    warn "Pre-compiled release binary is unavailable."
    if command -v cargo &>/dev/null; then
        info "Cargo detected. Building Capsule from source..."
        if [[ -n "$SCRIPT_DIR" && -f "$SCRIPT_DIR/Cargo.toml" ]]; then
            (
                cd "$SCRIPT_DIR"
                cargo build --release
            )
            if [[ -f "$SCRIPT_DIR/target/release/capsule" ]]; then
                sudo cp "$SCRIPT_DIR/target/release/capsule" "$INSTALL_DIR/capsule"
                sudo chmod +x "$INSTALL_DIR/capsule"
                BINARY_INSTALLED=true
                success "Capsule binary compiled from local source and installed to $INSTALL_DIR/capsule"
            fi
        else
            SRC_TEMP_DIR=$(mktemp -d)
            if git clone "https://github.com/${GITHUB_REPO}.git" "$SRC_TEMP_DIR"; then
                info "Compiling Capsule release binary via cargo..."
                (
                    cd "$SRC_TEMP_DIR"
                    cargo build --release
                )
                if [[ -f "$SRC_TEMP_DIR/target/release/capsule" ]]; then
                    sudo cp "$SRC_TEMP_DIR/target/release/capsule" "$INSTALL_DIR/capsule"
                    sudo chmod +x "$INSTALL_DIR/capsule"
                    BINARY_INSTALLED=true
                    success "Capsule binary compiled from source and installed to $INSTALL_DIR/capsule"
                elif [[ -f "$SRC_TEMP_DIR/target/release/Capsule" ]]; then
                    sudo cp "$SRC_TEMP_DIR/target/release/Capsule" "$INSTALL_DIR/capsule"
                    sudo chmod +x "$INSTALL_DIR/capsule"
                    BINARY_INSTALLED=true
                    success "Capsule binary compiled from source and installed to $INSTALL_DIR/capsule"
                fi
            fi
            rm -rf "$SRC_TEMP_DIR"
        fi
    fi
fi

if [[ "$BINARY_INSTALLED" == false ]]; then
    error "Failed to install Capsule binary! Pre-compiled release was not found and cargo build from source failed."
fi

DESKTOP_FILE="$DESKTOP_DIR/capsule.desktop"
cat << EOF | sudo tee "$DESKTOP_FILE" > /dev/null
[Desktop Entry]
Name=Capsule
Comment=Dynamic Island Shell for Linux
Exec=$INSTALL_DIR/capsule
Icon=capsule
Terminal=false
Type=Application
Path=$HOME
NoDisplay=true
Categories=Utility;System;
EOF
sudo chmod 644 "$DESKTOP_FILE"
success "Desktop entry created at $DESKTOP_FILE"

echo -e "----------------------------------------"
success "Capsule installation completed successfully!"

START_CAPSULE=false
if [[ "$WAS_RUNNING" == true ]]; then
    START_CAPSULE=true
else
    start_confirm=$(read_input "Do you want to start Capsule now? [Y/n] " "y")
    start_confirm=$(echo "$start_confirm" | tr '[:upper:]' '[:lower:]')
    if [[ "$start_confirm" != "n" && "$start_confirm" != "no" ]]; then
        START_CAPSULE=true
    fi
fi

if [[ "$START_CAPSULE" == true ]]; then
    info "Starting Capsule from user home directory ($HOME)..."
    (
        cd "$HOME"
        export HOME="$HOME"
        if command -v gtk-launch &>/dev/null && [[ -f "$DESKTOP_FILE" ]]; then
            gtk-launch capsule.desktop >/dev/null 2>&1 || nohup "$INSTALL_DIR/capsule" >/dev/null 2>&1 &
        else
            nohup "$INSTALL_DIR/capsule" >/dev/null 2>&1 &
        fi
    )
    sleep 1
    if pgrep -x capsule &>/dev/null || pgrep -x Capsule &>/dev/null; then
        success "Capsule is now running in the background!"
    else
        warn "Capsule process did not start or exited. You can start it manually with 'capsule'."
    fi
fi

echo ""
echo -e "${CYAN}How to autostart Capsule on login:${NC}"
echo -e "  - In Hyprland:      Add ${YELLOW}exec-once = capsule${NC} to ~/.config/hypr/hyprland.conf"
echo -e "  - In Niri:          Add ${YELLOW}spawn-at-startup \"capsule\"${NC} to ~/.config/niri/config.kdl"
echo ""
