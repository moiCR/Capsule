# Changelog

All notable changes to Capsule are documented in this file.

## [v1.0]

### Features
- **GPU Screen Recorder Integration**: Added a screen recording module powered by `gpu-screen-recorder` as the recording backend, fully configurable through the Settings module.
- **Shelf Widget (Drag & Drop Staging)**: Introduced a temporary file staging shelf supporting drag-and-drop operations, quick copy actions, and automatic file-type icon resolution.
- **Inline Calculator & Unit Converter**: Integrated mathematical evaluation into the application launcher search bar powered by `fend-core`, supporting arithmetic, percentages, powers, and unit conversions.
- **Declarative Theme Template Engine**: Redesigned external application theming into a dynamic, user-configurable template engine (`~/.config/capsule/templates/`). Supports arbitrary applications with in-place section patching (`replace_section`), comment marker blocks (`start_marker` / `end_marker`), include hook injection, and real-time palette synchronization without external templating dependencies.
- **Wi-Fi & Bluetooth Satellites**: Added dedicated interactive satellite overlays for NetworkManager (Wi-Fi) and BlueZ (Bluetooth), featuring real-time device discovery, status monitoring, and connection controls.

### Improvements
- **Settings Module Redesign**: Overhauled the Settings UI with a streamlined layout, improved spacing, and clearer category organization.
- **Clipboard Snippets**: Added snippet management alongside clipboard history for quick access and insertion of frequently used text.
- **Default File Manager Setting**: Added an option in default applications settings to configure the preferred file manager.
- **Performance & Spring Bounds**: Fine-tuned satellite spring animations and bounds tracking for smoother, glitch-free transitions.

### Bug Fixes
- **Keyboard Focus Routing**: The shell now explicitly requests compositor keyboard focus during module and launcher transitions, resolving an issue where typing in the launcher would leak keystrokes to the underlying client window.
