# Capsule
A Wayland desktop shell built with GPUI (the Zed UI framework), inspired by Dragon Ball's Capsule Corp.

## Scope
Capsule is an all-in-one desktop shell designed to be lightweight, responsive, and GPU-accelerated.

In a typical tiling Wayland setup, you end up gluing together 5 or 6 separate daemons (Waybar, Rofi, Dunst, Wlogout, clipboard managers, and shell scripts) through IPC and duplicate styling. Capsule unifies this into a single process built around a contextual Dynamic Island that smoothly morphs its state—from an idle flip-clock to active launchers, media controls, and quick settings—without spawning or destroying windows.

Because it is written in Rust and powered by GPUI, it renders directly through the GPU at 60+ FPS with minimal RAM consumption, avoiding the runtime overhead of WebViews, Electron, or QML/JS engines.

## Showcase
https://github.com/user-attachments/assets/09a61cdc-6e56-4841-a826-4097da17a6d0

## Features
### Dynamic Island & Shell
- **Context-Aware Morphing**: Smooth spring-animated transitions between Idle (flip-clock / minimal status), Compact bar, and Expanded module views.
- **Satellites & OSDs**: Non-intrusive floating overlays for volume, screen brightness, and media status that react to your compositor keybinds.
- **Shelf**: Built-in scratchpad for temporary file staging and drag-and-drop workflows.

### Launcher & Productivity
- **Application Launcher**: Fast fuzzy-search application runner.
- **Inline Calculator & Conversions**: Instant arithmetic evaluation (`15% of 2500`, `2^8`, `sin(45)`) and unit conversions directly in the search bar.
- **Clipboard & Snippets**: Integrated clipboard history manager (via `cliphist`) and custom text snippets.

### System Services & Control Center
- **Audio Control**: Native PipeWire / WirePlumber integration with per-app volume sliders and live audio sink switching.
- **Connectivity**: NetworkManager (Wi-Fi) and BlueZ (Bluetooth) toggles and device pairing.
- **Media (MPRIS)**: Track metadata, album artwork display, and playback control.
- **Session & Security**: Native Polkit authentication agent, screen recorder service, and power management menu.

## Theming & Templates
Capsule includes a declarative theming engine that automatically synchronizes your system colors across your terminals, compositor, shell, and file managers in real time without external template dependencies.

### 1. Creating Custom Themes
Themes are stored in `~/.config/capsule/themes/presets/<Name>.toml`. Any `.toml` file placed here is automatically discovered by Capsule and selectable via the quick settings menu:

```toml
mode = "Dark" # "Dark" | "Light"
font_family = "Geist"

[background_color]
hex = "#121210"

[background_color_alt]
hex = "#181818"

[surface_color]
hex = "#242424"

[foreground_color]
hex = "#FFFFFF"

[foreground_color_muted]
hex = "#AAAAAA"

[accent_color]
hex = "#007BFF"

[red_color]
hex = "#FF3B30"

[green_color]
hex = "#34C759"
```

### 2. Template Plugins
Templates live in `~/.config/capsule/templates/` (either as `<app>.toml` or `<app>/template.toml`). When a theme changes, Capsule renders all active templates and updates your target configs.

A template definition supports:
- **`target`**: Destination path (`~` and `$XDG_CONFIG_HOME` are automatically expanded).
- **`replace_section`**: For INI/TOML files where themes must live inside an existing file (e.g. `foot.ini`). Replaces only that section (e.g. `[colors-{{mode}}]`) while preserving non-color keys like `alpha`, `blur`, and custom keybindings.
- **`start_marker` / `end_marker`**: In-place replacement between specific comment markers for arbitrary files.
- **`hook`**: Automatically injects an `include` or `require` line into your main config if it's missing.
- **`reload`**: Shell command executed after writing the template.

#### Example: Terminal with In-Place Section Replacement (`foot.toml`)
```toml
name = "foot"
description = "Foot terminal theme"
enabled = true
target = "~/.config/foot/foot.ini"
replace_section = "[colors-{{mode}}]"

[template]
content = """
[colors-{{mode}}]
background={{background.strip}}
foreground={{foreground.strip}}
regular0={{palette.0.strip}}
regular1={{palette.1.strip}}
...
bright7={{palette.15.strip}}
"""
```

#### Example: Hook Injection (`hyprland.toml`)
```toml
name = "hyprland"
description = "Hyprland borders and colors"
enabled = true
target = "~/.config/hypr/capsule_colors.lua"
reload = "hyprctl reload"

[hook]
file = "~/.config/hypr/hyprland.lua"
contains = "require(\"capsule_colors\")"
inject = "colors = require(\"capsule_colors\")\n"

[template]
content = """
return {
    active_border = "rgb({{accent.strip}})",
    inactive_border = "rgb({{background_alt.strip}})",
    surface = "rgb({{surface.strip}})"
}
"""
```

### 3. Template Engine Syntax & Context
You can use the following variables inside any template:

| Variable | Format / Description | Example |
| :--- | :--- | :--- |
| `{{mode}}` | Current mode (`dark` or `light`) | `dark` |
| `{{font_family}}` | Theme font name | `Geist` |
| `{{<color>}}` | Standard 6-digit hex color with `#` | `#007BFF` |
| `{{<color>.strip}}` | Hex color without `#` (for Foot, Fish, Alacritty) | `007BFF` |
| `{{<color>.rgb}}` | Comma-separated RGB channels | `0, 123, 255` |
| `{{<color>.rgba}}` | CSS rgba string | `rgba(0, 123, 255, 1.0)` |
| `{{<color>.qt}}` | ARGB hex color for Qt | `#ff007bff` |
| `{{palette.0}}` .. `{{palette.15}}` | Full 16-color ANSI palette with modifiers | `{{palette.1.strip}}` |

Conditionals are also supported:
```ini
{% if is_dark %}
# Dark mode configuration
{% else %}
# Light mode configuration
{% endif %}
```

## Linux Support

![Arch Linux Based](https://img.shields.io/badge/Arch_Linux_Based-1793D1?style=for-the-badge&logo=archlinux&logoColor=white)

- **Distributions**: Currently targeted and optimized for **Arch Linux** and Arch-based distributions.
- **Compositors**: Native support for **Hyprland** and **Niri** (via direct IPC integrations).
- **Backend**: Wayland-only (utilizing `wlr-layer-shell`).
- **Core Integrations**: D-Bus (`org.mpris.MediaPlayer2`, `org.freedesktop.NetworkManager`, BlueZ), PipeWire, WirePlumber, Polkit.

## Installation
```bash
curl -fsSL https://raw.githubusercontent.com/moiCR/Capsule/master/install.sh | bash
```

Or build from source:
```bash
git clone https://github.com/moiCR/Capsule.git
cd Capsule
cargo build --release
```

## Special Thanks
- [zed](https://github.com/zed-industries/zed): for creating gpui and making it open source
- [tide-island](https://github.com/enhaoswen/Tide-island): for the inspiration
- [saneAspect](https://github.com/enhaoswen/Tide-island): for the inspiration (Your course is way too expensive, bro.)
- [Akira Toriyama](https://en.wikipedia.org/wiki/Akira_Toriyama): for creating Dragon Ball, inspiring millions around the world, and giving me a childhood I'll never forget.
