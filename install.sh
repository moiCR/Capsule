#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

APP_NAME="capsule"
GITHUB_REPO="moiCR/Capsule"
DOTFILES_REPO="https://github.com/moiCR/Capsule-Plus.git"
INSTALL_DIR="/usr/local/bin"
DESKTOP_DIR="/usr/share/applications"

REAL_USER="${SUDO_USER:-$USER}"
USER_HOME=$(getent passwd "$REAL_USER" | cut -d: -f6 2>/dev/null || echo "$HOME")
CONFIG_DIR="${USER_HOME}/.config/capsule"

SUDO=""
if [[ $EUID -ne 0 ]]; then
    if command -v sudo &>/dev/null; then
        SUDO="sudo"
    fi
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

ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
    aarch64 | arm64) TARGET="aarch64-unknown-linux-gnu" ;;
    *)       error "Unsupported architecture: $ARCH" ;;
esac

info "Detecting package manager..."
PKG_MANAGER=""
AUR_HELPER=""
DNF_CMD=""

if command -v paru &>/dev/null; then
    AUR_HELPER="paru"
    PKG_MANAGER="aur"
    info "Found AUR helper: paru"
elif command -v yay &>/dev/null; then
    AUR_HELPER="yay"
    PKG_MANAGER="aur"
    info "Found AUR helper: yay"
elif command -v dnf &>/dev/null || command -v dnf5 &>/dev/null; then
    PKG_MANAGER="dnf"
    if command -v dnf5 &>/dev/null; then
        DNF_CMD="dnf5"
    else
        DNF_CMD="dnf"
    fi
    info "Found package manager: ${DNF_CMD}"
fi

ARCH_LINUX_DEPENDENCIES=(
    "vulkan-icd-loader"
    "libxkbcommon"
    "wayland"
    "qt5ct"
    "qt6ct"
    "gsettings-desktop-schemas"
    "zenity"
    "dconf"
    "nwg-look"
    "awww"
    "xdg-desktop-portal"
    "xdg-desktop-portal-gtk"
    "adw-gtk-theme"
    "git"
    "curl"
    "tar"
)

FEDORA_DEPENDENCIES=(
    "vulkan-loader"
    "libxkbcommon"
    "wayland-devel"
    "qt5ct"
    "qt6ct"
    "gsettings-desktop-schemas"
    "zenity"
    "dconf"
    "xdg-desktop-portal"
    "xdg-desktop-portal-gtk"
    "adw-gtk3-theme"
    "lz4-devel"
    "cargo"
    "git"
    "curl"
    "tar"
)

if [[ "$PKG_MANAGER" == "aur" ]]; then
    TO_INSTALL=()
    info "Checking Arch Linux dependencies..."
    for pkg in "${ARCH_LINUX_DEPENDENCIES[@]}"; do
        if "$AUR_HELPER" -Qq "$pkg" &>/dev/null; then
            echo -e "  \e[1;32m✔ \e[0m $pkg \e[2minstalled\e[0m"
        else
            echo -e "  \e[1;33m➜ \e[0m $pkg \e[1mwill be installed\e[0m"
            TO_INSTALL+=("$pkg")
        fi
    done

    if [[ ${#TO_INSTALL[@]} -gt 0 ]]; then
        echo -e "----------------------------------------"
        info "Packages to install: ${#TO_INSTALL[@]}"

        confirm="y"
        if [[ -t 0 ]]; then
            read -rp "Do you want to proceed with the installation of dependencies? [Y/n] " confirm
        fi
        confirm=$(echo "$confirm" | tr '[:upper:]' '[:lower:]')

        if [[ "$confirm" == "n" || "$confirm" == "no" ]]; then
            info "Package installation skipped by user."
        else
            info "Starting installation using $AUR_HELPER..."
            if $AUR_HELPER -S --needed "${TO_INSTALL[@]}"; then
                success "All packages installed successfully!"
            else
                error "Installation failed. Please check the logs above."
            fi
        fi
    else
        echo -e "----------------------------------------"
        info "All dependencies are already installed!"
    fi
elif [[ "$PKG_MANAGER" == "dnf" ]]; then
    TO_INSTALL=()
    info "Checking Fedora dependencies..."
    for pkg in "${FEDORA_DEPENDENCIES[@]}"; do
        if rpm -q "$pkg" &>/dev/null; then
            echo -e "  \e[1;32m✔ \e[0m $pkg \e[2minstalled\e[0m"
        else
            echo -e "  \e[1;33m➜ \e[0m $pkg \e[1mwill be installed\e[0m"
            TO_INSTALL+=("$pkg")
        fi
    done

    if [[ ${#TO_INSTALL[@]} -gt 0 ]]; then
        echo -e "----------------------------------------"
        info "Packages to install: ${#TO_INSTALL[@]}"

        confirm="y"
        if [[ -t 0 ]]; then
            read -rp "Do you want to proceed with the installation of dependencies? [Y/n] " confirm
        fi
        confirm=$(echo "$confirm" | tr '[:upper:]' '[:lower:]')

        if [[ "$confirm" == "n" || "$confirm" == "no" ]]; then
            info "Package installation skipped by user."
        else
            info "Starting installation using $DNF_CMD..."
            if $SUDO "$DNF_CMD" install -y "${TO_INSTALL[@]}"; then
                success "All packages installed successfully!"
            else
                error "Installation failed. Please check the logs above."
            fi
        fi
    else
        echo -e "----------------------------------------"
        info "All dependencies are already installed!"
    fi
else
    warn "No supported package manager ('paru', 'yay', or 'dnf') was found on your system."
    warn "Package dependency checks will be skipped. Ensure required libraries are installed."
fi

# Ensure Wayland wallpaper daemon (awww / swww) is available
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
                $SUDO cp "$AWWW_TEMP_DIR/target/release/awww" "$INSTALL_DIR/awww"
                $SUDO cp "$AWWW_TEMP_DIR/target/release/awww-daemon" "$INSTALL_DIR/awww-daemon"
                $SUDO chmod +x "$INSTALL_DIR/awww" "$INSTALL_DIR/awww-daemon"
                success "'awww' wallpaper daemon installed successfully to $INSTALL_DIR!"
            fi
        fi
        rm -rf "$AWWW_TEMP_DIR"
    else
        warn "Cargo is not available to compile 'awww'. Please install cargo or 'awww' manually."
    fi
fi

# 1. Fetch Latest Release Binary or Compile from Source via Temporary Clone
echo -e "----------------------------------------"
info "Installing Capsule binary for $ARCH ($TARGET)..."
$SUDO mkdir -p "$INSTALL_DIR"
$SUDO mkdir -p "$DESKTOP_DIR"

BINARY_INSTALLED=false

RELEASE_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/capsule-${TARGET}.tar.gz"
info "Downloading latest binary release from ${RELEASE_URL}..."

TEMP_BIN_DIR=$(mktemp -d)
if curl -sSL --fail "$RELEASE_URL" -o "$TEMP_BIN_DIR/capsule.tar.gz"; then
    info "Extracting release asset..."
    if tar -xzf "$TEMP_BIN_DIR/capsule.tar.gz" -C "$TEMP_BIN_DIR"; then
        if [[ -f "$TEMP_BIN_DIR/capsule" ]]; then
            $SUDO cp "$TEMP_BIN_DIR/capsule" "$INSTALL_DIR/capsule"
            $SUDO chmod +x "$INSTALL_DIR/capsule"
            BINARY_INSTALLED=true
            success "Capsule binary installed from GitHub Release to $INSTALL_DIR/capsule"
        fi
    fi
fi
rm -rf "$TEMP_BIN_DIR"

# Fallback to cloning & compiling if pre-compiled release binary is unavailable
if [[ "$BINARY_INSTALLED" == false ]]; then
    warn "Pre-compiled release binary is unavailable."
    if command -v cargo &>/dev/null; then
        info "Cargo detected. Cloning ${GITHUB_REPO} to compile from source..."
        SRC_TEMP_DIR=$(mktemp -d)
        if git clone "https://github.com/${GITHUB_REPO}.git" "$SRC_TEMP_DIR"; then
            info "Compiling Capsule release binary via cargo..."
            (
                cd "$SRC_TEMP_DIR"
                cargo build --release
            )
            if [[ -f "$SRC_TEMP_DIR/target/release/capsule" ]]; then
                $SUDO cp "$SRC_TEMP_DIR/target/release/capsule" "$INSTALL_DIR/capsule"
                $SUDO chmod +x "$INSTALL_DIR/capsule"
                BINARY_INSTALLED=true
                success "Capsule binary compiled from source and installed to $INSTALL_DIR/capsule"
            elif [[ -f "$SRC_TEMP_DIR/target/release/Capsule" ]]; then
                $SUDO cp "$SRC_TEMP_DIR/target/release/Capsule" "$INSTALL_DIR/capsule"
                $SUDO chmod +x "$INSTALL_DIR/capsule"
                BINARY_INSTALLED=true
                success "Capsule binary compiled from source and installed to $INSTALL_DIR/capsule"
            fi
        fi
        rm -rf "$SRC_TEMP_DIR"
    fi
fi

if [[ "$BINARY_INSTALLED" == false ]]; then
    error "Failed to install Capsule binary! Pre-compiled release was not found and cargo build from source failed."
fi

# Desktop Entry
DESKTOP_FILE="$DESKTOP_DIR/capsule.desktop"
cat << EOF | $SUDO tee "$DESKTOP_FILE" > /dev/null
[Desktop Entry]
Name=Capsule
Comment=Dynamic Island Bar for Linux
Exec=$INSTALL_DIR/capsule
Icon=capsule
Terminal=false
Type=Application
NoDisplay=true
Categories=Utility;System;
EOF
success "Desktop entry created at $DESKTOP_FILE"

echo -e "----------------------------------------"
confirm_dotfiles="y"
if [[ -t 0 ]]; then
    read -rp "Do you want to install Capsule-Plus (dotfiles)? [Y/n] " confirm_dotfiles
fi

confirm_dotfiles=$(echo "$confirm_dotfiles" | tr '[:upper:]' '[:lower:]')
if [[ "$confirm_dotfiles" == "n" || "$confirm_dotfiles" == "no" ]]; then
    info "Dotfiles installation skipped by user."
else
    info "Installing Capsule-Plus dotfiles directly in $CONFIG_DIR..."

    if [[ -d "$CONFIG_DIR/.git" ]]; then
        info "Existing Capsule-Plus repository found at $CONFIG_DIR. Updating via git pull..."
        (cd "$CONFIG_DIR" && git pull) || warn "Git pull failed, proceeding with existing files."
    elif [[ -d "$CONFIG_DIR" && "$(ls -A "$CONFIG_DIR" 2>/dev/null)" ]]; then
        BACKUP_DIR="${CONFIG_DIR}.bak.$(date +%Y%m%d_%H%M%S)"
        info "Backing up existing $CONFIG_DIR to $BACKUP_DIR..."
        mv "$CONFIG_DIR" "$BACKUP_DIR"
        git clone "$DOTFILES_REPO" "$CONFIG_DIR"
    else
        mkdir -p "$(dirname "$CONFIG_DIR")"
        git clone "$DOTFILES_REPO" "$CONFIG_DIR"
    fi

    if [[ -f "$CONFIG_DIR/install.sh" ]]; then
        info "Executing Capsule-Plus installer script from $CONFIG_DIR..."
        chmod +x "$CONFIG_DIR/install.sh"
        (cd "$CONFIG_DIR" && ./install.sh) || warn "Capsule-Plus install.sh finished with warnings."
        success "Capsule-Plus dotfiles installed successfully in $CONFIG_DIR!"
    else
        warn "No install.sh script found in $CONFIG_DIR."
    fi
fi

echo -e "----------------------------------------"
success "Capsule installation completed successfully!"
info "You can start Capsule by running 'capsule' in your terminal."
