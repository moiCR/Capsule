# Changelog

All notable changes to Capsule are documented in this file.

## v1.2

### Features
- **Redesigned Default Module**: Replaced legacy Idle module with a revamped Default module featuring modular widgets:
  - **Workspaces Widget**: Interactive workspace indicator supporting active status, Pacman dot animation, and special workspace transitions.
  - **Status Dock**: Integrated indicator displaying network connectivity, battery level and charging status, Do Not Disturb (DND), and active notification counts.
  - **Clock & Flip Clock**: Dual time presentation options with smooth minute flip animations.
  - **Media Dock**: Compact player widget featuring spinning vinyl disc animation and MPRIS playback controls.
- **Niri Compositor Support**: Added initial IPC integration and workspace tracking for the Niri Wayland compositor.
- **Standalone Settings Panel**: Decoupled settings from the capsule into an independent `Layer::Overlay` panel with spring physics (`apple_island_spring`), slide-up entrance, circular expansion, and reverse dismissal animation.
- **Wallpaper Blur Fallback**: Added automatic generation and caching of blurred wallpapers for the lockscreen and background surfaces.

### Improvements
- **Live Configuration Synchronization**: Subscribed Capsule directly to `ConfigService`, dynamically applying style, margin, radius, and exclusive zone adjustments in real time without restarts or mode switching.
- **Lockscreen Architecture**: Complete modularization into dedicated components (`auth_form`, `clock`, `lyrics`, `view`).
- **Real-time Lyrics Sync**: Direct synchronization between MPRIS playback position and LRCLIB cached lyrics without channel latency or UI freezing.
- **Hyprland Compositor Integration**: Enhanced event handling for special workspaces, workspace switching, and frame duration calculation.
- **Satellites & Panel Surfaces**: Refined satellite animations, surface framing geometry, and connecting curves.
- **Power & Battery Service**: Improved battery capacity detection and charging state listeners.
- **Assets**: Added vector icons for vinyl player, pacman workspace indicators, and user avatars.

### Bug Fixes
- **Concave / Normal Style Offset**: Resolved an issue where toggling between Concave and Normal capsule styles failed to reposition the capsule until another mode switch occurred.
- **Lockscreen PAM Authentication**: Offloaded PAM authentication to background worker threads, eliminating infinite "Verificando..." freezes and borrow collisions during unlock.
- **Unlock Panic Prevention**: Deferred panel closing (`cx.defer`) and properly sequenced window removal to prevent `cannot update while already being updated` panics.
- **GPUI Task Stability**: Eliminated fragile background channel loops and replaced them with robust GPUI executor timers, preventing silent loop crashes.
- **Dynamic Dimension Tracking**: Fixed dimension measurements and target size adjustments across capsule transitions and dashboard expansions.
