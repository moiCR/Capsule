
<div align="center">

[![Arch Linux](https://img.shields.io/badge/Arch_Linux-supported-1793d1?logo=archlinux&logoColor=white)](https://archlinux.org/)
[![NixOS](https://img.shields.io/badge/NixOS-supported-5277c3?logo=nixos&logoColor=white)](https://nixos.org/)
[![Debian](https://img.shields.io/badge/Debian-supported-5277c3?logo=debian&logoColor=white)](https://debian.org/)
[![ubuntu](https://img.shields.io/badge/Ubuntu-supported-5277c3?logo=ubuntu&logoColor=white)](https://ubuntu.org/)


[![Niri](https://img.shields.io/badge/Niri-supported-5e81ac?logo=wayland&logoColor=white)](https://github.com/YaLTeR/niri)
[![Hyprland](https://img.shields.io/badge/Hyprland-supported-58e1ff?logo=wayland&logoColor=white)](https://github.com/hyprwm/Hyprland)
[![MangoWM](https://img.shields.io/badge/MangoWM-supported-f2a65a?logo=wayland&logoColor=white)](https://github.com/DreamMaoMao/mangowc)

</div>

<div align="center">

# Capsule

A Wayland desktop shell built with GPUI (the Zed UI framework), inspired by Dragon Ball's Capsule Corp.
</div>



## Scope
Capsule is an all-in-one desktop shell designed to be lightweight, responsive, and GPU-accelerated.

In a typical tiling Wayland setup, you end up gluing together 5 or 6 separate daemons (Waybar, Rofi, Dunst, Wlogout, clipboard managers, and shell scripts) through IPC and duplicate styling. Capsule unifies this into a single process built around a contextual Dynamic Island that smoothly morphs its state—from an idle flip-clock to active launchers, media controls, and quick settings—without spawning or destroying windows.

Because it is written in Rust and powered by GPUI, it renders directly through the GPU at 60+ FPS with minimal RAM consumption, avoiding the runtime overhead of WebViews, Electron, or QML/JS engines.

## Showcase
https://github.com/user-attachments/assets/baf17a29-27d1-4f04-a97b-43b860c84d84

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

## Requirements
Capsule is a Wayland desktop shell and expects a real Wayland runtime, not a bare X11 setup.

- A supported Wayland compositor such as Niri, Hyprland, or MangoWM.
- PipeWire and WirePlumber enabled and running.
- The `wpctl`, `pactl`, and `parec` utilities available in `PATH`.
- At least one visible audio device (sink/source) exposed by the system.
- If you are testing inside a VM, you will usually need a virtual audio device or a host with real audio passthrough; otherwise the audio widgets will stay empty.

### Nix

The provided flake supports `x86_64-linux` and `aarch64-linux`.

```bash
nix run github:moiCR/Capsule
nix build github:moiCR/Capsule#Capsule
```

For a NixOS or Home Manager configuration, add this input to your flake:

```bash
inputs.capsule.url = "github:moiCR/Capsule";
inputs.capsule.inputs.nixpkgs.follows = "nixpkgs";
```

For NixOS:

```nix
environment.systemPackages = [ inputs.capsule.packages.${pkgs.system}.default ];
```

Or with Home Manager:

```nix
home.packages = [ inputs.capsule.packages.${pkgs.system}.default ];
```


## Special Thanks
- [zed](https://github.com/zed-industries/zed): for creating gpui and making it open source
- [tide-island](https://github.com/enhaoswen/Tide-island): for the inspiration
- [saneAspect](https://www.youtube.com/@saneAspect): for the inspiration (Your course is way too expensive, bro.)
- [Akira Toriyama](https://en.wikipedia.org/wiki/Akira_Toriyama): for creating Dragon Ball, inspiring millions around the world, and giving me a childhood I'll never forget.
