---
name: capsule-design
description: Design and refine Capsule and Orbit's GPUI interface using the project's minimalist style. Use for visual changes, modules, widgets, orbs, satellites, controls, spacing, typography, states, and animations. Ground decisions in existing implementations, theme, and configuration; avoid unnecessary persistent UI or decoration.
---

# Capsule and Orbit Design

## Direction

Capsule is a contextual Wayland shell. Its interface should occupy only the space and attention needed for the current task.

**Fewer simultaneous elements, clear hierarchy, and detail on demand.** Minimalism does not mean making everything tiny, hiding essential actions, or replacing clear labels with ambiguous icons.

- Start with the closest existing module or widget. Improve consistency before introducing another visual language.
- Each visible element must communicate useful state, enable a necessary action, or aid understanding. Omit elements whose removal harms none of these.
- A design request does not authorize new features, configuration options, panels, or dependencies.
- Do not turn Capsule Corp. inspiration into themed decoration. Avoid anime motifs, neon, or sci-fi styling unless explicitly requested.
- These rules guide new changes; they do not authorize redesigning unrelated modules.

## Before Designing

1. Define the task: what users need to see or do, when the interface appears, and when it is no longer needed.
2. Consult the project graph and verify coverage and freshness according to repository rules. Read the affected surface's current code and a nearby reference implementation. If the graph is insufficient, inspect files directly and disclose the limitation.
3. Inspect the container, widget, states, and theme/configuration sources. Do not design from the README alone.
4. Choose the smallest surface that solves the task using the distinctions below.
5. Preserve existing structure and gestures unless the requested change affects them. Do not ask users to resolve decisions already established by the code.

Paths below are relative to the repository root, not the skill directory. Documented values are observed references, not universal tokens or substitutes for configuration. Verify them before reuse.

## Surface Responsibilities

### Capsule: Primary Interaction

- Use the existing container to present the active module. Do not draw another framed capsule inside it.
- Size the interface for the necessary information and controls. A brief action can fit in one row; search needs input and results, not a welcome page.
- Preserve dimension and corner-radius morphing. Do not force every module into the same large rectangle.
- Use the launcher as a reference for compact lists and the recording bar for single-row status and controls.
- Complex modules such as the dashboard may group controls. Their density should not carry over to simple interactions.

### Orbit and Orbs: Contextual Presence

Orbit manages layout, visibility, and motion. Orb widgets render indicators and connect interactions.

- An orb represents an active state worth quick access, not an installed feature. Do not add a permanent orb for every module.
- The current pattern shows Shelf when it contains items and Recording while it is not stopped. Capsule allows them in `Default` mode with no active drag target; opening them activates the corresponding module.
- Preserve this policy unless the task requires changing it. Do not keep orbs around every module as decoration.
- Keep each orb circular with one symbol or indicator. Add a badge only for operational information, such as Shelf item count.
- Do not add permanent titles, descriptions, timers, action menus, or legends to orbs. Details belong in the module they open.
- Reuse `ORB_SIZE` and Orbit geometry. Do not calculate independent positions in each widget or create another manager.
- Preserve active orbs' lateral stability, gap closing, and continuity when reversing transitions. Do not reorder them on every state update.
- Respect `interactive` and input regions when changing visibility. Hidden elements must not capture clicks, and transparent window areas must not block desktop input.
- Preserve specialized interactions, such as dropping files onto Shelf, without extending them to all orbs.

### Satellites: Supporting Detail

- Distinguish circular status orbs from panels in `satellites/`.
- Reuse existing satellites and their manager for supporting details that follow this pattern. Do not create a parallel window or popup system.
- Opening a detail view must not expose unrelated controls. Preserve placement, available-space, and dismissal rules.

## Visual Language

### Color and Surfaces

- Use the existing `Theme` colors according to their meaning: `background()`, `background_alt()`, `surface()`, `foreground()`, `foreground_muted()`, `accent()`, `red()`, and `green()`.
- Use `background` for the base and `surface` with appropriate opacity for grouping and feedback. Do not assume the theme is always dark.
- Reserve `accent` for selection, primary actions, active states, or brief feedback. Do not accent every border, icon, and background simultaneously.
- Use `foreground` for primary text and `foreground_muted` for secondary text. Keep useful content readable.
- Active controls may use an accent background, as connectivity pills do. Preserve this functional signal.
- Reuse the container's border and shadow. Build hierarchy through spacing and alignment before adding internal backgrounds, frames, or nested cards.
- Do not introduce gradients, glow, glassmorphism, extra blur, or arbitrary colors merely to make the design look modern.
- Current recording widgets contain fixed colors, and the Shelf badge uses fixed white. These are observed exceptions, not a palette to copy. Prefer available semantic colors without inventing theme methods or expanding its schema beyond scope.

### Shape, Scale, and Spacing

- Preserve the existing shape family: adaptive container, pills for compact controls, circles for icon actions, and softly rounded list rows.
- Respect container parameters and available configuration. Do not hardcode another global radius, font, spacing, or duration inside a widget.
- Group through alignment, proximity, and whitespace. Add an internal background only when it distinguishes an actual control or functional group.
- Keep density compact and readable. Do not fill unused space with extra content or shrink interactive areas for a minimalist appearance.

Scale references to verify in code:

| Element | Current reference | Use |
| --- | --- | --- |
| Orb | `ORB_SIZE = 26.0`; 13 px icon | Reuse the constant |
| Recording orb indicator | 8 px dot | Status without extra text |
| `IconButton` | 24 px control; default 13 px icon | Reusable compact action |
| Recording bar controls | 26 px control; 12 px icon | Activity-row reference |
| Launcher row | 12 px radius; 6 px vertical padding | Compact list |
| Launcher app text | 13 px semibold primary; 11 px secondary | Brief row hierarchy |
| Launcher list | 4 px gap | Result spacing |

Do not use small badge text sizes for general content or impose these measurements on surfaces with different needs.

### Typography, Icons, and Content

- Inherit the configured font through the theme/container. Geist is the default, not a font to hardcode in every widget.
- Prefer primary and secondary text levels. Use semibold or bold for focused emphasis, not every label.
- Reuse existing SVGs and preserve optical scale. Avoid decorative emoji, mixed icon families, or additional icon packages.
- Avoid headings that repeat what controls or content already explain. A recording bar does not need an additional “Recording” heading.
- Write short, concrete labels localized through the existing language service. Do not add help paragraphs by default.
- Preserve ellipsis, height limits, and scrolling where appropriate. Check long titles, paths, translations, and counters without arbitrarily enlarging the entire interface.
- Keep empty states quiet: a useful sentence and, where helpful, a muted icon. Avoid large illustrations, onboarding, or unrelated feature prompts.

## Interaction and Motion

- Distinguish selection, hover, pressed, and disabled states through restrained surface, border, or contrast changes. Do not move or scale every control on hover.
- Keep keyboard selection visible and consistent with pointer interaction. Preserve the module's Enter, Escape, focus, and navigation behavior.
- Do not make essential actions depend solely on hover or a new hidden gesture. Icons need clear meaning and accessible names where supported.
- Preserve necessary feedback and confirmations. Important states must remain understandable without color alone.
- Closing a view does not stop its activity. Preserve the distinction between closing the recording bar and stopping recording.
- Reuse Capsule transitions and Orbit/satellite curves. Read `animation_duration_ms` where used; do not assume every animation shares configuration.
- Use motion to explain appearance, retraction, or resizing. Avoid independent bounces, persistent pulses, and ornamental animation.
- Continue interrupted animations from the current visual state. Stop continuous repainting when no ongoing activity requires it.

## GPUI Composition

- Keep rendering declarative. Widgets present state and connect actions; they do not instantiate services or perform blocking work.
- Extend the relevant module in `crates/app/src/capsule/modules/` and widgets in `crates/app/src/capsule/widgets/`. Do not accumulate all new UI in `capsule.rs`.
- Reuse controls from `crates/ui/src/components/` where appropriate. Extract shared components for actual reuse, not to wrap every `div()`.
- Use `AppState` entities and global services. Do not duplicate theme, configuration, or Orbit managers.
- Follow project rules for background work with Tokio. Older implementations do not justify copying patterns that conflict with those rules.
- Preserve stable IDs, event propagation, focus, and input regions. Visual changes must not accidentally trigger parent-container actions.

## Implementation References

Read only references relevant to the task:

| Path | What to inspect |
| --- | --- |
| `crates/app/src/capsule/capsule.rs` | `sync_orbit_visibility`, `open_orb`, dimensions, configuration, focus, input regions |
| `crates/app/src/capsule/container/normal.rs` | Parameterized size, radius, font, background, border |
| `crates/app/src/capsule/orbit.rs` | `ORB_SIZE`, `Layout`, `Motion`, geometry, continuity/visibility tests |
| `crates/app/src/capsule/widgets/record/orb.rs` | Minimal recording/paused indicator and module access |
| `crates/app/src/capsule/widgets/shelf/orb.rs` | Icon, counter, drag feedback |
| `crates/app/src/capsule/widgets/record/bar.rs` | Status, duration, compact actions without extra heading |
| `crates/app/src/capsule/modules/record.rs` | Width by state/content; closing versus stopping |
| `crates/app/src/capsule/modules/launcher.rs` | Search/results composition, empty state, keyboard navigation |
| `crates/app/src/capsule/widgets/launcher/app_item.rs` | Row hierarchy, selection, ellipsis, feedback |
| `crates/app/src/capsule/widgets/dashboard/quick_settings.rs` | Functional grouping and active pills |
| `crates/app/src/capsule/satellites/mod.rs` | Supporting panels, lanes, bounds, animation |
| `crates/ui/src/components/button.rs` | Existing `IconButton` and `TextButton` |
| `crates/ui/src/theme/mod.rs` | Actual color and font APIs |

These references describe specific patterns, not a complete visual audit. If code and this guide diverge, verify the current implementation. Do not invent APIs or force outdated measurements.

## Final Review

Before completing a design, check:

- Does it solve the task without adding a persistent surface?
- Is the primary content clear without another title, card, or color?
- Does each orb remain an indicator/access point rather than a miniature module?
- Do details appear when needed and disappear without accidentally canceling activities?
- Are theme, configuration, density, and shared components respected?
- Is content readable across light/dark themes, long content, and supported scaling?
- Do keyboard, pointer, and relevant drag interactions work, including rapid opening/closing and multiple orbs?
- Do transparent areas pass input through, and do hidden elements stop capturing clicks?
- Has the change avoided unrelated features, configuration, and refactors?

For Rust changes, run `cargo fmt`, relevant tests, and Clippy according to scope and repository rules. Visually verify affected states when the shell can run. Otherwise, disclose that verification was code-only and identify remaining visual checks. A successful build is not visual validation.
