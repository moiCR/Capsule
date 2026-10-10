# Changelog

All notable changes to Capsule are documented in this file.

## v2.0.0

### Features
- Completely rewrote Capsule with a new architecture and redesigned its core systems and user experience.
- Redesigned the Settings experience with integrated navigation, search, and reusable setting controls.
- Added new localized strings for the updated Settings interface in English and Spanish.
- Added a terminal-based (TUI) installer to guide users through the installation process without requiring a graphical interface.
- Added a notification module with support for displaying, dismissing, and replying to notifications.
- Restructured the application runtime around Tokio and added event-loop tests.

### Improvements
- Improved Settings module state handling and window integration.
- Refined styling and layout primitives throughout the rewritten Capsule.
- Improved CPU performance by reducing unnecessary work during normal operation.
- Improved animation performance to make transitions smoother and more responsive.
- Refined the user interface across all modules for a more consistent and polished experience.
- Reduced CPU usage during normal operation.
- Rebuilt the satellite system as part of the Capsule rewrite.
- Migrated `gpui-capsule` from a fork of `gpui` to `gpui-ce` as part of the rewrite.

### Bug Fixes
### Bug Fixes
- Fixed clipboard integration when using `cliphist`. Previously, `cliphist` could return an error, causing Capsule to fall back to text-only clipboard handling; the integration now uses the expected clipboard history behavior.
- Fixed a clipboard issue that prevented items from being copied.
- Fixed the lock screen continuously rechecking authentication.