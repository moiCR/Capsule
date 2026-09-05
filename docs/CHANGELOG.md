## What's New in v0.3.1

### Features & UI Improvements
- **Capsule Style Customization (Normal & Concave)**: Added the `capsule_style` option (`Normal` and `Concave`), allowing users to choose between the classic floating rounded pill and a concave notch attached flush to the screen bezel.
- **Concave Container & Vector Wings**: Created SVG shoulder wings (`concave-wing-left`, `concave-wing-right` and border assets) for continuous curvature connecting to the monitor frame.
- **Decoupled Container Architecture**: Introduced the `CapsuleContainerRenderer` trait with dedicated renderers (`NormalContainer` and `ConcaveContainer`), separating container geometry and framing from capsule views.
- **Live Dynamic Offset & Margin Switching**: Moved vertical positioning to GPUI (`current_y`), enabling instant real-time transitions between Normal and Concave modes, as well as live `margin_top` changes without restarting the application.
- **Dynamic Exclusive Zone**: Compositor reserved space (`exclusive_zone`) now syncs live when changing capsule styles or dimensions.
- **UI Settings Selector**: Added an interactive style selector (`Normal` / `Cóncava`) in the Settings UI tab.

### Fixes and Minor Changes
- **Concave Seam Lines Fix**: Eliminated unwanted internal vertical border lines between the concave wings and the central pill body.
- **Settings Modal Isolation**: Ensured the Settings window always renders using `NormalContainer` as a centered rounded modal dialog without wings.
- **Top Bezel Alignment**: LayerShell surface is now permanently anchored to $y = 0.0$, eliminating the gap between the concave notch and the screen bezel.
- **Capsule Modules Consolidation**: Unified 12 separate capsule view fields into a centralized `CapsuleModules` struct (`crates/app/src/capsule/modules/mod.rs`).
- Fixed Ghostty reload app (the config was deleted on theme change).
- The refresh time for the launcher service has been reduced.
