---
name: capsule-bar-design
description: Flat Design 2.0 standards and geometry guidelines for Capsule's native desktop Bar, its dockless widgets (Workspaces, Clock, Media, Status, Tray, Power), SharedPanel modules (Calendar, Dashboard, Media, Tray, Power), and Gaussian-blurred popups (Spotlight Launcher, Polkit).
---

# Capsule Bar Design (Flat Design 2.0)

## Core Philosophy

Capsule's Bar and its panels follow **Flat Design 2.0**:
- Clean, layered solid surfaces (`style::background(theme)` outer shells + `style::surface(theme)` elevated inner cards) paired with bold `theme.accent()` active/selected pill cards and `theme.on_accent()` foregrounds.
- Dockless Bar widgets that sit directly on the Bar surface without nested pill containers or micro-borders.
- Generous rounded geometry (`rounded_2xl` / `16px`–`24px` for cards and panels, `rounded_full` for sliders, badges, and circular icon buttons).
- Hover states applied directly on rounded card or circular button containers using solid colors (never unrounded rectangular children inside `overflow_hidden`, since GPUI's `overflow_hidden` clips to an axis-aligned rectangle).
- Modal overlays and popups (Spotlight Launcher, Polkit, Dashboard subviews) use **Gaussian backdrop blur** (`.backdrop_blur(px(...))` with a subtle surface tint) instead of heavy dark/black backdrops.
- All components remain localized using `AppState::language` with zero hardcoded user-facing strings.

## Geometry and Proportions

### Bar Container
- Bar height: `40px` (or configured height).
- Bar horizontal padding: `12px`, vertical padding: `4px`.
- Horizontal zone gap: `8px`.
- Optional Convex Mode (`bar_config.convex`): renders outward-curving `18px` convex ears at the bar's left and right screen edges and concave shoulders on attached `SharedPanel` modules.

## Dockless Bar Widgets

Widgets sit directly on the Bar without dock pill backgrounds or borders:

### 1. Workspaces (`crates/app/src/bar/widgets/workspaces.rs`)
- Active workspace: `pacman.svg` (`16px`) in `theme.yellow()`.
- Occupied workspace (with open apps): `ghost.svg` (`15px`) in workspace slot color (`accent`, `red`, `cyan`, `orange`, `green`, `purple`).
- Empty workspace: minimal `6px` dot in `theme.foreground_muted().opacity(0.45)`.

### 2. Clock (`crates/app/src/bar/widgets/clock.rs`)
- Displays **only the formatted time** (`13px`, `FontWeight::SEMIBOLD`, `theme.foreground()`), with no date or separator dot.
- Clicking toggles `SharedPanelModule::Calendar` (the 2-column `Notifications | Calendar` panel).

### 3. Media (`crates/app/src/bar/widgets/media.rs`)
- Displays **only the track title and artist** (`Título — Artista`), with no leading SVG icon and no inline play/pause button.
- Clicking toggles `SharedPanelModule::Media`.

### 4. Status (`crates/app/src/bar/widgets/status.rs`)
- Displays network status icon + audio volume status icon.
- Battery icon and percentage are rendered **only if a battery actually exists** (`state.power.get_battery().is_some()`), never on desktop systems without a battery.
- Clicking toggles `SharedPanelModule::Dashboard`.

### 5. Tray (`crates/app/src/bar/widgets/tray.rs`)
- Lock-free `ArcSwap` SNI item row (`size(26px)` circular hover targets, `16px` icons).
- Clicking an item opens `SharedPanelModule::Tray(idx)`.

### 6. Power (`crates/app/src/bar/widgets/power.rs`)
- Circular `28px` icon button (`power.svg`, `15px`) that tints `theme.red()` on hover and toggles `SharedPanelModule::Power`.

## SharedPanel Modules (`crates/app/src/bar/shared_panel/`)

- **Calendar & Notifications (`calendar.rs`)**: 2-column row (`Notifications | Calendar + Clock header`, `720px × 450px`) in Flat Design 2.0 cards.
- **Dashboard (`dashboard.rs`)**:
  - Top row: 2-column large pill cards (`Network` and `Bluetooth`, `h(72px)`, `rounded_2xl`) with solid `theme.accent()` active fill and a circular `rounded_full` chevron button (`chevron-right.svg`) to open the corresponding subview.
  - Middle container: rounded surface card (`rounded_2xl`) housing 6 quick-action icon tiles (`DND`, `PowerProfile`, `Audio Mute`, `Mic Mute`, `Lock`, `Settings`).
  - Bottom slider cards: individual `rounded_2xl` surface cards (`h(58px)`) for Output Volume, Input Mic Volume (with circular gear button to open device selection), and Brightness (only when a battery is present), built with `ui::components::Slider` and `DimensionTracker`.
  - **Overlay Subviews (`Wifi`, `Bluetooth`, `AudioOutput`, `AudioInput`)**: open as an animated modal card (`rounded_2xl`) on top of the Dashboard with a **Gaussian-blurred backdrop** (`.backdrop_blur(px(12.0 * progress)).bg(style::background(&theme).opacity(0.18 * progress))`).

## Popups & Spotlight Launcher (`crates/app/src/bar/popups/launcher.rs`)

- **Modern Clean Minimalist / Integrated Command Palette**:
  - Seamless integrated search header at top (no nested cards, clean naked search icon directly inline with text, `h(52px)`, `px(16px)`).
  - Subtle `1px` horizontal divider line (`bg(style::border(theme).opacity(0.16))`) separating the header and the results list.
  - Transparent unselected rows with subtle surface tint on hover (`style::surface(theme).opacity(0.35)`).
  - Selected row with soft, integrated surface fill (`style::surface(theme).opacity(0.85)`), subtle border, and a **vertical accent indicator pill** on the far left edge (`3.5px × 20px`, `rounded_full`, `theme.accent()`).
  - Icon squircle (`34px`, `rounded(8px)`, `bg(style::background(theme).opacity(0.55))`).
  - Crisp, uncluttered layout with no right-side buttons or chevrons.
- **Gaussian Blur Backdrop**:
  - Uses `.backdrop_blur(px(20.0 * progress)).bg(gpui::black().opacity(0.20 * progress))` for clean, immersive visual depth.
