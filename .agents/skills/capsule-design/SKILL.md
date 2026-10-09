---
name: capsule-design
description: Design and refine Capsule's native GPUI modules, widgets, controls and animations with its solid, rounded, theme-aware desktop style. Ground changes in new_capsule and preserve real system interactions.
---

# Capsule Design

## Design direction

Capsule uses solid rounded surfaces, spacious cards, pill controls and clear typography. Reference mockups establish proportions, alignment and hierarchy. Their colors, branding and example data are not specifications. Apply the active theme and the user's latest layout corrections.

Work in `crates/app/src/new_capsule/`. Legacy `capsule/` code is a reference for behavior that has not migrated. Preserve each module's purpose: the compact Default pill, launcher grid, Clipboard and Shelf galleries, finite appearance carousels, recording strip and focused Polkit form. Do not impose one Dashboard layout on every module.

## Before changing the UI

- Inspect module state, widgets, theme helpers and a nearby implementation. Use the installed codebase graph and verify relevant coverage; read missed or stale source directly.
- Reuse actual services and interaction paths. Styling must not replace integrations with placeholders or require dependencies without a concrete reason.
- For requested design exploration, provide meaningfully different compositions. Recoloring the same layout does not provide a different design.
- Preserve local edits. Explicit user choices take precedence over examples in this skill.

## Theme and surfaces

- Use `new_capsule/widgets/style.rs` for opaque background, surface, hover, selected, border, raised and on-accent colors. Reuse the configured font and radius.
- Keep grouping surfaces opaque; desktop text must not show through ordinary cards.
- Use foreground for content and muted foreground for metadata. Active controls, selection and slider fill use the theme accent; accent-backed text uses `style::on_accent`.
- Avoid glass, decorative gradients, glow and blurred panels. The music player's existing blurred artwork is an intentional exception.
- Differentiate groups through surface tone and spacing. Borders identify selection, focus or a necessary boundary rather than every nested group.
- Theme previews, wallpapers, album artwork and application-owned tray icons retain their source colors.

## Geometry and hierarchy

- Read each module's constants before choosing dimensions. Adjust groups independently; do not scale the entire interface uniformly.
- Shared card radii are currently 16px and 12px for inner surfaces. Outer Capsule rounding follows configuration.
- Keep padding, gaps and columns aligned. Circles suit prominent status icons and playback controls; utility actions can be pills or rounded rectangles.
- Ellipsize long titles and paths. Wrap and scroll descriptions, authentication messages and notifications inside their own content area.
- Match module dimensions to actual padding, gaps and child heights. Fixed rows need minimum heights and protection from unintended flex shrinking.
- Avoid decorative titles, explanatory filler and oversized empty areas.

## Dashboard

Current references: 560px width, 268px connectivity column, 8px column gap, 260px music column. Height is 562px with tray items and 514px without them. These dimensions do not apply to other modules.

- Keep one Home composition. Connection, calendar, audio and tray details use the existing satellites instead of replacing Home with secondary views.
- Header: date/time left and actual Settings action right. No Capsule branding, decorative title, time divider or top-right close button.
- Connectivity: coherent cards with prominent circular icons, real Ethernet/Wi-Fi state, readable labels and muted secondary text. Preserve separate toggles and detail actions; nested actions stop propagation.
- Music: real rounded artwork background, title and artist without album, pause at the opposite side of the same row. Center the header/progress group vertically and give the header extra horizontal padding. Previous and next sit at opposite ends of the bar. No timestamps, player-selector badge or media satellite.
- Sound: one 80px section with a 32px track and real volume/output selection. There is no Display section. Paint accent fill as a complete rounded track clipped to the volume fraction, preserving the endpoint shape at low volume. Dragging uses the same bounds. Do not create a shrinking rounded blob or add a second thumb.
- Notifications: bounded 150px section with natural-height rows, minimum row height and scrolling. Long bodies wrap without clipping; preserve notification actions and dismissal.
- Tray: actual icons at the bottom with balanced space above/below. Remove the footer and its gap when empty. Preserve application-owned icons and supported interactions.
- Themes, Wallpapers and Lock remain available through IPC. Do not restore their removed Dashboard shortcuts or a bottom Settings button.

## Other modules

- Default remains a 250px workspace/clock/status pill with a minimum height of 40px. Preserve workspace spacing, accent states, special workspaces and centered time.
- Launcher preserves its native grid, real icons, search and keyboard navigation.
- Clipboard uses a 480px gallery with two columns and 144px cards. Search and tabs sit above the gallery. Two complete rows fit before scrolling; footer hints stay outside the scroll area. Binary images and local image paths show previews; selecting an image copies image data.
- Shelf preserves real file previews, copying and drag/drop.
- Themes and Wallpapers remain finite compact carousels with piano motion, rounded preview clipping and explicit apply. Bound wallpaper names inside thumbnails; the current limit is 12 Unicode characters plus an ellipsis. Theme preview backgrounds stay inside rounded selection bounds.
- Recording remains a compact action/status bar. Closing its view does not silently stop recording. Preserve pending, paused, stopping and error states.
- Polkit uses a compact 400×212px form: accent lock icon, authentication title and actual user, real message/action, masked password, bounded error area and Cancel/Authenticate. Long messages/errors scroll. Show the error area only when needed; it adds 40px to the module height. Keep normal and pending states compact without reserved empty bands. Real requests open the form automatically; queueing, authority cancellation, retries and completion use the existing authentication service.
- Satellites retain one panel per side, centered relative to Capsule, with measured height, a maximum and smooth transitions. Do not reintroduce legacy Orbit behavior.

## Interaction, animation and idle behavior

- Preserve keyboard navigation, focus, Escape/Enter and essential actions. Do not hide required actions behind hover.
- Use restrained hover, pressed, selected and disabled states. Do not scale or move every control on hover.
- Reuse configured animation durations. Interrupted transitions continue from the displayed state; finished animations stop requesting frames.
- Module resizing and content fades use GPUI next-frame callbacks. Same-size module changes still reveal their content. Capsule opts out of GPUI's inactive-window frame cap so returning to unfocused Default remains smooth; this must not introduce continuous idle rendering.
- Timer-driven animations use the normalized compositor refresh interval with a defensive 2ms minimum. Do not impose a 16ms floor on high-refresh displays.
- Rendering is declarative. Perform IO, service creation, task setup, focus changes and Wayland commits outside render. Geometry updates require cached diffs.
- Give interactive loop elements stable unique IDs. Keep GPUI's event loop outside Tokio `block_on`; D-Bus/socket work runs on the shared Tokio runtime and reaches entities through channels.
- Polkit is event-driven. Do not add periodic request/result polling, copy passwords to the clipboard or log them. Cancel outstanding authentication when the request expires, is cancelled or the entity closes.
- Preserve input regions and pass-through outside visible panels. Do not synchronize geometry from idle polling loops.

## Implementation references

| Path | Responsibility |
| --- | --- |
| `crates/app/src/new_capsule/widgets/style.rs` | Semantic opaque palette and controls |
| `crates/app/src/new_capsule/widgets/dashboard/` | Home, music, sound, notifications and tray |
| `crates/app/src/new_capsule/widgets/default/` | Compact workspace/clock/status pill |
| `crates/app/src/new_capsule/widgets/clipboard.rs` | Clipboard gallery and previews |
| `crates/app/src/new_capsule/widgets/appearance.rs` | Appearance carousels |
| `crates/app/src/new_capsule/widgets/polkit.rs` | Authentication form |
| `crates/app/src/new_capsule/module/polkit.rs` | Authentication state, requests and input |
| `crates/services/src/polkit/mod.rs` | Existing Polkit agent and helpers |
| `crates/app/src/new_capsule/window_state.rs` | Cached geometry and focus |
| `crates/app/src/new_capsule/animator.rs` | Size/content transition state |
| `crates/ui/src/theme/mod.rs` | Existing theme and font APIs |

## Verification

Run formatting, compilation and relevant behavior tests. Check long labels, empty/populated states, clipping, focus, keyboard input, interrupted transitions and both theme modes. Verify Polkit with a real authorization request and cancellation without inventing requests or collecting credentials.

Render the real application when a desktop session is available. Claim visual verification only for states inspected. Measure frame intervals and idle CPU when changing animation scheduling; compilation and screenshots do not establish FPS. Report remaining limitations plainly.
