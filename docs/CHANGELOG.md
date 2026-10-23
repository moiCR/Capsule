# Changelog

All notable changes to Capsule are documented in this file.

## v2.4.0

### Features
* **KWin Compatibility**: Added full compatibility with KWin Wayland compositor.
* **Standalone KWin Session**: Added standalone session support to run Capsule directly on top of KWin.
* **Lockscreen Fallback**: Added overlay lockscreen fallback with wallpaper blur for compositors without session-lock support.
* **Plasma Wallpaper Detection**: Added automatic detection for active KDE Plasma wallpaper.

### Improvements
* **Focus Management**: Improved keyboard focus handling to prevent stealing compositor shortcuts.
* **Window Rules**: Added automatic KWin rules to exclude Capsule from task switchers and pagers.

### Bug Fixes
* **Window Close Handling**: Fixed issue where window close shortcuts (`Alt+F4`, `Super+Q`) would close the panel.

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
