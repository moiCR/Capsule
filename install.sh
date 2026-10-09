#!/usr/bin/env bash
set -euo pipefail

# Capsule Installer Bootstrap
# Downloads the TUI installer binary to /tmp and runs it

REPO="moiCR/Capsule"
INSTALLER_BIN="/tmp/capsule-installer"
TEMP_DIR="/tmp/capsule-installer-bootstrap-$$"

if [[ $EUID -eq 0 ]]; then
    echo "Error: Please run this script as your regular user, not root or sudo." >&2
    echo "Sudo privileges will be requested by the installer when needed." >&2
    exit 1
fi

BETA_MODE=false
for arg in "$@"; do
    case "$arg" in
        --beta) BETA_MODE=true ;;
    esac
done

ARCH=$(uname -m)
case "$ARCH" in
    x86_64)          TARGET="x86_64-unknown-linux-gnu" ;;
    aarch64 | arm64) TARGET="aarch64-unknown-linux-gnu" ;;
    *)               echo "Error: Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

SCRIPT_DIR=""
if [[ -n "${BASH_SOURCE[0]:-}" ]]; then
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
fi

# If running within repository with local build, prefer local binary
if [[ -n "$SCRIPT_DIR" && -f "$SCRIPT_DIR/Cargo.toml" ]]; then
    if [[ -x "$SCRIPT_DIR/target/release/capsule-installer" ]]; then
        exec "$SCRIPT_DIR/target/release/capsule-installer" "$@"
    elif [[ -x "$SCRIPT_DIR/target/debug/capsule-installer" ]]; then
        exec "$SCRIPT_DIR/target/debug/capsule-installer" "$@"
    fi
fi

echo "==> Fetching Capsule TUI installer for $ARCH ($TARGET)..."

mkdir -p "$TEMP_DIR"
trap 'rm -rf "$TEMP_DIR"' EXIT

LATEST_TAG=""
if command -v curl &>/dev/null; then
    if [[ "$BETA_MODE" == true ]]; then
        LATEST_TAG=$(curl -sSL -m 10 "https://api.github.com/repos/${REPO}/releases?per_page=50" 2>/dev/null | awk '
            /^  \{$/ { tag = "" }
            /"tag_name":[[:space:]]*"/ {
                line = $0
                sub(/^.*"tag_name":[[:space:]]*"/, "", line)
                sub(/".*$/, "", line)
                tag = line
            }
            /"prerelease":[[:space:]]*true/ && tag ~ /-beta/ { print tag; exit }
        ' || echo "")
    else
        LATEST_TAG=$(curl -sSL -m 5 "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/' || echo "")
    fi
fi

if [[ -n "$LATEST_TAG" ]]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/capsule-installer-${TARGET}.tar.gz"
else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/capsule-installer-${TARGET}.tar.gz"
fi

DOWNLOADED=false
if curl -sSL --fail "$DOWNLOAD_URL" -o "$TEMP_DIR/installer.tar.gz" 2>/dev/null; then
    if tar -xzf "$TEMP_DIR/installer.tar.gz" -C "$TEMP_DIR" 2>/dev/null; then
        if [[ -f "$TEMP_DIR/capsule-installer" ]]; then
            cp "$TEMP_DIR/capsule-installer" "$INSTALLER_BIN"
            DOWNLOADED=true
        fi
    fi
fi

if [[ "$DOWNLOADED" == false ]]; then
    # Fallback to direct raw binary if published without tar
    DIRECT_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG:-latest}/capsule-installer-${TARGET}"
    if curl -sSL --fail "$DIRECT_URL" -o "$INSTALLER_BIN" 2>/dev/null; then
        DOWNLOADED=true
    fi
fi

if [[ "$DOWNLOADED" == false ]]; then
    # If no prebuilt installer is published yet on GitHub, try building locally via cargo if available
    if command -v cargo &>/dev/null && [[ -n "$SCRIPT_DIR" && -f "$SCRIPT_DIR/Cargo.toml" ]]; then
        echo "==> Building installer locally via cargo..."
        cargo build --release -p capsule-installer
        if [[ -x "$SCRIPT_DIR/target/release/capsule-installer" ]]; then
            exec "$SCRIPT_DIR/target/release/capsule-installer" "$@"
        fi
    fi

    echo "Error: Failed to download Capsule installer binary from GitHub Releases ($DOWNLOAD_URL)." >&2
    exit 1
fi

chmod +x "$INSTALLER_BIN"
exec "$INSTALLER_BIN" "$@"
