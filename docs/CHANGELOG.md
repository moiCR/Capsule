# Changelog

All notable changes to Capsule are documented in this file.

## v2.0.0

### Features
- Added a polkit integration so authentication prompts from supported applications can be handled directly by Capsule.
- Added a terminal-based (TUI) installer to guide users through the installation process without requiring a graphical interface.

### Improvements
- Improved CPU performance by reducing unnecessary work during normal operation.
- Improved animation performance to make transitions smoother and more responsive.
- Refined the user interface across all modules for a more consistent and polished experience.

### Bug Fixes
- Fixed clipboard integration when using `cliphist`. Previously, `cliphist` could return an error, causing Capsule to fall back to text-only clipboard handling; the integration now uses the expected clipboard history behavior.