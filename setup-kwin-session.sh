#!/usr/bin/env bash
set -e

echo "==> 1. Compilando Capsule en modo release..."
cargo build --release -p capsule

echo "==> 2. Instalando binario en /usr/local/bin/capsule..."
sudo cp target/release/capsule /usr/local/bin/capsule
sudo chmod 755 /usr/local/bin/capsule

echo "==> 3. Creando script de arranque /usr/local/bin/capsule-session-kwin..."
sudo rm -f /usr/local/bin/capsule-session-kwin
cat << 'EOF' > /tmp/capsule-session-kwin
#!/usr/bin/env bash
export XDG_CURRENT_DESKTOP="KDE:Capsule"
export XDG_SESSION_DESKTOP="KDE"
export KDE_FULL_SESSION=true
if [[ -z "${KDE_SESSION_VERSION:-}" ]]; then
    if command -v kwin_wayland &>/dev/null && kwin_wayland --version 2>&1 | grep -q " 6\."; then
        export KDE_SESSION_VERSION=6
    else
        export KDE_SESSION_VERSION=5
    fi
fi

if command -v dbus-update-activation-environment &>/dev/null; then
    dbus-update-activation-environment --systemd --all
fi

exec kwin_wayland --exit-with-session /usr/local/bin/capsule
EOF
sudo cp /tmp/capsule-session-kwin /usr/local/bin/capsule-session-kwin
sudo chmod 755 /usr/local/bin/capsule-session-kwin
rm -f /tmp/capsule-session-kwin

echo "==> 4. Creando entrada en /usr/share/wayland-sessions/kwin-capsule.desktop..."
sudo mkdir -p /usr/share/wayland-sessions
cat << 'EOF' > /tmp/kwin-capsule.desktop
[Desktop Entry]
Name=KWin + Capsule
Comment=KWin Wayland Compositor with Capsule Shell
Exec=/usr/local/bin/capsule-session-kwin
Type=Application
DesktopNames=KDE;Capsule;
EOF
sudo cp /tmp/kwin-capsule.desktop /usr/share/wayland-sessions/kwin-capsule.desktop
sudo chmod 644 /usr/share/wayland-sessions/kwin-capsule.desktop
rm -f /tmp/kwin-capsule.desktop

echo "==> 5. Configurando reglas de KWin (kwinrulesrc)..."
python3 - << 'EOF'
import configparser
import os

path = os.path.expanduser("~/.config/kwinrulesrc")
config = configparser.RawConfigParser(strict=False)
config.optionxform = str
if os.path.exists(path):
    config.read(path)

if not config.has_section("General"):
    config.add_section("General")

order = [x.strip() for x in config.get("General", "Order", fallback="").split(",") if x.strip()]
for rule in ["capsule", "capsule-lockscreen"]:
    if rule not in order:
        order.insert(0, rule)
config.set("General", "Order", ",".join(order))

if not config.has_section("capsule"):
    config.add_section("capsule")
config.set("capsule", "Description", "Capsule Panel")
config.set("capsule", "wmclass", "capsule-panel")
config.set("capsule", "wmclassmatch", "1")
config.set("capsule", "closeable", "false")
config.set("capsule", "closeablerule", "2")
config.set("capsule", "skipswitcher", "true")
config.set("capsule", "skipswitcherrule", "2")
config.set("capsule", "skiptaskbar", "true")
config.set("capsule", "skiptaskbarrule", "2")
config.set("capsule", "skippager", "true")
config.set("capsule", "skippagerrule", "2")

if not config.has_section("capsule-lockscreen"):
    config.add_section("capsule-lockscreen")
config.set("capsule-lockscreen", "Description", "Capsule Lockscreen")
config.set("capsule-lockscreen", "wmclass", "capsule-lockscreen")
config.set("capsule-lockscreen", "wmclassmatch", "1")
config.set("capsule-lockscreen", "closeable", "false")
config.set("capsule-lockscreen", "closeablerule", "2")
config.set("capsule-lockscreen", "skipswitcher", "true")
config.set("capsule-lockscreen", "skipswitcherrule", "2")
config.set("capsule-lockscreen", "skiptaskbar", "true")
config.set("capsule-lockscreen", "skiptaskbarrule", "2")
config.set("capsule-lockscreen", "skippager", "true")
config.set("capsule-lockscreen", "skippagerrule", "2")
config.set("capsule-lockscreen", "above", "true")
config.set("capsule-lockscreen", "aboverule", "2")

with open(path, "w") as f:
    config.write(f, space_around_delimiters=False)
EOF

if command -v qdbus6 &>/dev/null; then
    qdbus6 org.kde.KWin /KWin reconfigure 2>/dev/null || true
elif command -v qdbus &>/dev/null; then
    qdbus org.kde.KWin /KWin reconfigure 2>/dev/null || true
fi

echo ""
echo "=========================================================="
echo " [OK] Sesión 'KWin + Capsule' instalada exitosamente."
echo " Ya puedes cerrar sesión y seleccionarla en SDDM / greeter."
echo "=========================================================="
