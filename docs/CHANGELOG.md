# Changelog

All notable changes to Capsule are documented in this file.

## v2.4.0

### Features
* **Desktop Bar Mode**: Introduced a native desktop Bar display mode alongside Island mode, featuring modular dockless widgets, convex geometry, attached panels (Dashboard, Calendar, Media, Tray, Power), and a centered Spotlight launcher.
* **Interactive Widget Customizer**: Added a visual drag-and-drop palette in Settings to arrange and manage bar widgets across Left, Center, and Right zones.
* **KWin Wayland Support**: Added full compatibility with the KWin compositor, standalone session execution, and overlay lockscreen fallback with wallpaper blur.
* **Flat Design 2.0 & Gaussian Blur**: Applied a unified Flat Design 2.0 visual system with layered surface cards, native sliders, and hardware-accelerated Gaussian backdrop blur for modal overlays.

### Improvements
* **Display Mode Architecture**: Decoupled UI modes into configurable Island and Bar modules with dedicated settings and IPC controls.
* **Rendering & Input Optimization**: Eliminated compositor glass shader overhead on panel layers, implemented lock-free system tray reads, and refined keyboard focus to avoid intercepting compositor shortcuts.
* **Compositor Window Rules**: Added automatic rules to exclude Capsule from task switchers, pagers, and window close shortcuts in KWin.

### Bug Fixes
* **Window Close Handling**: Fixed issue where window close shortcuts would close the panel in KWin.
