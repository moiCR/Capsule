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
INSTALL_DIR="${HOME}/.local/bin"
DESKTOP_DIR="${HOME}/.local/share/applications"

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

info "Detecting AUR helper..."
AUR_HELPER=""

if command -v paru &>/dev/null; then
    AUR_HELPER="paru"
elif command -v yay &>/dev/null; then
    AUR_HELPER="yay"
fi

if [[ -z "$AUR_HELPER" ]]; then
    error "Neither 'paru' nor 'yay' was found on your system."
    error "Please install paru or yay and run this installer again."
    exit 1
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
)

TO_INSTALL=()

info "Checking system dependencies..."
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
            exit 1
        fi
    fi
else
    echo -e "----------------------------------------"
    info "All dependencies are already installed!"
fi
