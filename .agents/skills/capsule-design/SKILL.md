---
name: capsule-design
description: Design and refine Capsule's native GPUI modules, widgets, controls, and animations with its Flat Design 2.0 desktop style, Gaussian backdrop blur for modal overlays, and theme-aware surfaces.
---

# Capsule Design (Flat Design 2.0)

## Design Direction: Flat Design 2.0

Capsule uses **Flat Design 2.0** across both Island and Bar modes:
- **Layered solid surfaces**: Outer containers use `style::background(theme)` while inner cards, rows, and input pills use `style::surface(theme)` with subtle boundaries (`style::border(theme).opacity(0.35)`).
- **Bold accent active/selected cards**: Active connectivity pills, quick-action tiles, and selected Launcher result cards fill with solid `theme.accent()` and switch their foregrounds to `theme.on_accent()` (with `gpui::white().opacity(0.18)` for nested icon badges or action pills).
- **Generous rounded geometry**: Cards and panels use `rounded_2xl` (`16px`–`24px`), nested icon badges use `rounded_xl` (`12px`), and sliders, circular icon buttons, and status badges use `rounded_full`.
- **Gaussian blur for modal backdrops**: Modal popups (Spotlight Launcher, Polkit) and in-panel subview overlays (Dashboard Wi-Fi, Bluetooth, Audio Output/Input subviews) use GPUI's native **Gaussian backdrop blur** (`.backdrop_blur(px(...))` paired with a light surface tint `style::background(theme).opacity(0.08..0.18)`) rather than heavy dark/black backdrops.
- **Hover integrity in GPUI**: Because GPUI's `.overflow_hidden()` clips to an axis-aligned rectangular bounding box rather than rounded corner radii, never place unrounded `.hover(|s| s.bg(...))` children inside rounded cards. Always apply `.hover()` directly on the rounded card itself (using solid colors such as `solid_hover_accent` when active) or on explicitly rounded child buttons (`.rounded_full()` / `.rounded_xl()`).

## Standard Components & Controls

- **Sliders (`crates/ui/src/components/slider.rs`)**:
  - Always use `ui::components::Slider` paired with `ui::tracker::DimensionTracker` via `.track_bounds(&tracker)`.
  - Use `style::background(theme)` for `track_color` and `theme.accent()` for `active_color` (or `theme.foreground_muted()` when muted).
  - Never use raw neon colors (`theme.green()`, `theme.red()`) for audio/mic sliders or custom unmeasured track divs.
- **Inputs (`crates/ui/src/components/input.rs`)**:
  - Wrap `Input::from_state(...)` inside a Flat Design 2.0 `style::surface(theme)` card (`rounded_2xl` or `rounded_full`) with an accent icon badge or prefix icon.

## Bar Mode & SharedPanel Architecture (`crates/app/src/bar/`)

- **Dockless Bar Widgets (`crates/app/src/bar/widgets/`)**:
  - Widgets sit directly on the Bar without pill dock containers.
  - Workspaces use `pacman.svg` (active), `ghost.svg` (occupied), and `6px` dot (empty).
  - Clock shows only the time; Media shows only `Title — Artist`; Status shows network, audio, and battery only when a battery exists (`state.power.get_battery().is_some()`).
- **SharedPanel (`crates/app/src/bar/shared_panel/`)**:
  - Renders with concave top shoulders (`concave_panel_canvas`) seamlessly attached to the Bar, keeping inner content at fixed target dimensions (`anim.target_size`) during open/close animations to avoid Taffy layout thrashing.
  - `Calendar`: 2-column layout (`Notifications | Calendar`).
  - `Dashboard`: Top 2-column `Network` & `Bluetooth` pill cards, middle 6-tile quick actions box, bottom `Slider` cards (`Output Volume`, `Input Volume`, and conditional `Brightness`), plus animated Gaussian-blurred overlay subviews.
- **Spotlight Launcher (`crates/app/src/bar/popups/launcher.rs`)**:
  - Modern Clean Minimalist / Integrated Command Palette: seamless top search bar with inline icon, subtle divider line, transparent unselected rows, soft surface selected row with vertical accent indicator pill on the far left edge, and Gaussian backdrop blur.
- **Settings Bar Widget Editor (`crates/app/src/settings/views/bar.rs`)**:
  - Unified dark preview canvas containing a top horizontal row of visual mini-widget pills (dimmed when already placed in the bar, bright when available) above a live interactive Bar mockup with `Left`, `Center`, and `Right` drop/click zones.

## Performance & Rendering Rules

- **Pure `render()`**: Never call `window.set_input_region()`, `window.set_keyboard_interactivity()`, `window.focus()`, or `cx.notify()` inside `render()`. Synchronize window state outside `render()` (`Bar::sync_window_state`).
- **Keyboard Focus**: Only request `KeyboardInteractivity::Exclusive` when a text-entry popup or subview is active (`Launcher`, `Clipboard`, `Emoji`, `Polkit`, or Wi-Fi password input). Non-text panels (`Calendar`, `Dashboard` home, `Media`, `Tray`, `Power`) must use `KeyboardInteractivity::None` so active applications keep focus.
- **Lock-free UI reads**: Services read during `render()` (such as `SniHostService::items_arc()`) must use lock-free `ArcSwap` snapshots.
