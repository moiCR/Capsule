## What's New in v0.3.0

### Features & UI Improvements
- **Settings Module & Panel**: Introduced a dedicated Settings panel (`CapsuleMode::Settings`) allowing live configuration of UI parameters, default applications, wallpapers, and lockscreen options.
- **Granular Config Modularization**: Split configuration into structured sub-configs (`UIConfig`, `LockScreenConfig`, and `DefaultsConfig`) with live reloading and full serialization.
- **Customizable Capsule Styling**: Dynamic border radius (`capsule_round`), widget badge rounding, satellite panel gaps, and configurable transition animation durations.
- **Dashboard Header Enhancements**: Integrated quick-settings navigation, updated action controls, and improved layout responsiveness.
- **Expanded IPC Commands**: Added IPC commands for opening settings, launching default browser, terminal, and editor, as well as triggering lock and quit actions.
- **Satellite Panels Cleanup**: Streamlined satellite mini-panels with dynamic spacing and removed redundant panel chips.

### Fixes
- Fixed UI freeze and event loop contention when starting media playback in idle mode.
- Fixed lockscreen component layout and clock alignment according to user configuration.
- Fixed wallpaper selector modal dropdown clipping and transition behavior.
