# Changelog

All notable changes to Capsule are documented in this file.

## v1.1

### Features
- **Interactive Notifications & Inline Reply**: Notifications now dynamically expand on hover and adapt to their content size, featuring direct action button triggers and in-line quick replies without having to leave the capsule or open the target application.
- **Orbit System & Interactive Orbs**: Introduced the Orbit subsystem, providing floating satellite orbs flanking the capsule to surface background task status and provide quick-access controls.
- **Dedicated Record & Shelf Orbs**: Integrated interactive orbs into Orbit for real-time screen recording status (with quick stop and elapsed time indicators) and temporary file staging shelf management (with drag-and-drop targets and item count badges).
- **Satellite Surface Layer**: Refactored satellite overlays into an independent rendering surface (`SatelliteSurface`) with unified spring physics, smooth entrance/exit transitions, and decoupled input handling.

### Improvements
- **Media Player Card Redesign**: Modernized the dashboard media player widget with full-bleed album artwork backgrounds, centered track metadata and playback controls, and multi-player navigation indicators.
- **Smooth Track Change Transitions**: Added fluid crossfade animations between album artwork and subtle slide-and-fade typography transitions when changing tracks.
- **Orbit & Satellite Visibility Synchronization**: Synchronized Orbit visibility and satellite overlay bounds with primary capsule module states to prevent visual clipping and layout shifts.

### Bug Fixes
- **Album Art Aspect Ratio & Corner Clipping**: Fixed an issue where media player cards would inherit square image aspect ratios from album covers, causing bottom corners to render without proper rounded clipping.
- **Notification Lifecycle & Action Synchronization**: Resolved state desynchronization between background notification dismissals, action triggers, and active UI updates.
