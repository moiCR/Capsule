# Changelog

All notable changes to Capsule are documented in this file.

## v2.4.0

### Features
* **KWin Wayland Compositor Support**: Native backend integration detecting KWin (`KWIN_RUNNING=1` / `XDG_CURRENT_DESKTOP=KDE`), virtual desktop tracking (`kdotool`), focused window tracking, output refresh rate polling (`kscreen-doctor`), and native KDE session logout.
* **Standalone KWin + Capsule Session**: Automated session launcher and installer (`/usr/local/bin/capsule-session-kwin` and `/usr/share/wayland-sessions/kwin-capsule.desktop`) to run Capsule as a lightweight desktop environment powered by KWin without `plasmashell`.
* **Resilient LayerShell Lockscreen**: Fallback lockscreen implementation using `LayerShell` on `Layer::Overlay` with cached wallpaper blur (`WallpaperService::get_blurred_wallpaper()`) on compositors lacking `ext-session-lock-v1` or `wlr-screencopy` (such as KWin).
* **Plasma Wallpaper Auto-Detection**: Detects active wallpaper from `plasma-org.kde.plasma.desktop-appletsrc` or scans `/usr/share/wallpapers/` to ensure smooth desktop overview and window switcher (Alt+Tab) rendering without black buffers.

### Improvements
* **Passive Keyboard Interactivity**: Capsule main panel defaults to `KeyboardInteractivity::None` and unfocuses (`window.blur()`), dynamically requesting `OnDemand` or `Exclusive` focus only when interactive modules are active (Launcher, Polkit, Clipboard, Emoji). Prevents stealing compositor focus or breaking window shortcuts when clicking panel items.
* **Explicit Daemon Lifecycle**: Configured `cx.set_quit_mode(gpui::QuitMode::Explicit)` and surface watchdog to prevent compositor unmap events from terminating the Capsule process.
* **KWin Window Rules Automation**: Added automated rule generation in `setup-kwin-session.sh` targeting Capsule windows to prevent them from appearing in KWin's task switcher, pager, or taskbar.

### Bug Fixes
* **Compositor Close Signal Interception**: Updated Wayland layer-surface event handling in `gpui-capsule` to respect `on_window_should_close()`, and configured panel to intercept `Super+Q` / `Alt+F4` close events by smoothly dismissing active modules instead of unmapping or destroying the panel.
* **Python ConfigParser Compatibility**: Fixed `kwinrulesrc` parser handling in setup script using `RawConfigParser(strict=False)` with case preservation to prevent duplicate option errors on Python 3.14.

## v2.3.1

### Features
* **Native GTK & Qt Theming Engine**: Fully native desktop theme synchronization for GTK (GTK3, GTK4, Libadwaita) and Qt (Qt5, Qt6, KDE) applications directly in Rust, eliminating external helper utilities like `nwg-look`.
* **Modular Multi-Distro Installer (`DependsManager`)**: Added trait-based package management supporting **Arch-based** (`pacman`/`paru`/`yay`), **Debian** (`apt`/`dpkg`), **Ubuntu** (`apt`/`dpkg`), and **Fedora** (`dnf`/`rpm`) distributions with automatic `/etc/os-release` detection, native package mappings, and automated installation for `adw-gtk-theme` and `gpu-screen-recorder`.
* **Multi-Compositor Installer (`CompositorManager`)**: Added an interactive compositor selection step allowing users to choose and automatically install **Hyprland**, **Niri**, or **MangoWM** across Arch, Fedora (COPR), Debian, and Ubuntu (APT/source build).

### Improvements
* **Universal Windowed Installer**: Converted `capsule-installer` from a Wayland `LayerShell` overlay into a standard desktop window so it runs seamlessly across any desktop environment or window manager.
* **Non-Destructive Configuration Merging**: Safely merges theme keys into `kdeglobals`, `dolphinrc`, and GTK `settings.ini` without wiping out existing user preferences (such as custom fonts, icon themes, and cursor sizes).
* **Live XSettings Daemon Sync**: Added native support for generating `~/.config/xsettingsd/xsettingsd.conf` and dispatching `SIGHUP` reload signals, enabling instant hot-reloading for GTK3 applications without aggressive desktop portal restarts.
* **Streamlined Installer Dependencies**: Removed `nwg-look` from installer dependencies, reducing installation overhead and moving to fully self-contained desktop configuration.

### Bug Fixes
* **KDE & Dolphin Color Palette Persistence**: Corrected the invalid `[Base]` header in `~/.local/share/color-schemes/Capsule.colors` to `[General]` so KDE Frameworks 6 can index the color scheme, injected `[UiSettings]` into `kdeglobals` and `dolphinrc`, and replaced no-op `kreadconfig6` calls with `kwriteconfig6 --notify` so Dolphin and other KDE applications reliably load and live-reload the Capsule palette instead of defaulting to Breeze.
* **Alacritty Config Hook Syntax**: Fixed the Alacritty template hook injection to properly write `import = ["theme.toml"]` under the `[general]` TOML section and added self-healing migration for existing template files.
