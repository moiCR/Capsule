## What's New in v0.2.1

### Features & UI Improvements
- **Media Player Redesign**: Modern compact single-row layout (Dynamic Island style) with a 44px square album art thumbnail and an integrated player status badge (Spotify/Music).
- **SystemTray Performance & Responsiveness**: Migrated tray DBus loop timeouts and timers to native `tokio::time` primitives with ultra-low timeouts (50ms), allowing instant detection when apps open or close.
- **Notifications Widget Refinement**: Cleaned notification card layout, removed redundant header text, and added subtle horizontal dividers between notifications.

### Fixes
- Fixed background image corner clipping bug in GPUI.
- Fixed hover highlight overflow in notifications widget.
- Fixed password bypass in lockscreen (escape key).
