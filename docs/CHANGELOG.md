# Changelog

All notable changes to Capsule are documented in this file.

## v2.3.1

### Features
* **Native GTK & Qt Theming Engine**: Fully native desktop theme synchronization for GTK (GTK3, GTK4, Libadwaita) and Qt (Qt5, Qt6, KDE) applications directly in Rust, eliminating external helper utilities like `nwg-look`.

### Improvements
* **Non-Destructive Configuration Merging**: Safely merges theme keys into `kdeglobals`, `dolphinrc`, and GTK `settings.ini` without wiping out existing user preferences (such as custom fonts, icon themes, and cursor sizes).
* **Live XSettings Daemon Sync**: Added native support for generating `~/.config/xsettingsd/xsettingsd.conf` and dispatching `SIGHUP` reload signals, enabling instant hot-reloading for GTK3 applications without aggressive desktop portal restarts.
* **Streamlined Installer Dependencies**: Removed `nwg-look` from `ARCH_DEPENDENCIES`, reducing installation overhead and moving to fully self-contained desktop configuration.

### Bug Fixes
* **KDE & Dolphin Color Palette Persistence**: Corrected the invalid `[Base]` header in `~/.local/share/color-schemes/Capsule.colors` to `[General]` so KDE Frameworks 6 can index the color scheme, injected `[UiSettings]` into `kdeglobals` and `dolphinrc`, and replaced no-op `kreadconfig6` calls with `kwriteconfig6 --notify` so Dolphin and other KDE applications reliably load and live-reload the Capsule palette instead of defaulting to Breeze.
