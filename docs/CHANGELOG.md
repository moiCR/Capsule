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
* **Window Close Handling**: Fixed issue where window close shortcuts would close the panel in KWin.
