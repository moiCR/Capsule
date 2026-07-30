> [!WARNING]
> **Early Development**
> This project is in an early stage of development, which means that it currently only supports Hyprland and Arch Linux distributions.

# Capsule Shell
A Wayland desktop shell built with GPUI (The Zed UI framework). It's inspired by the Capsule Corp. capsules in Dragon Ball


## Scope
Capsule is designed to be lighter and faster than shells created with Quickshell. It's what you might call an all-in-one solution, since the shell is based on a Capsule / Dynamic Island that expands and changes its content depending on the, This eliminates the need for external apps such as rofi/walker, waybar, etc.

## Showcase


## Features
### Capsule & Widgets
- **Dynamic Island Architecture**: Context-aware expanding bar with fluid transitions between Idle, Compact, Expanded, and Module states.
- **Header Status Bar**: Workspaces indicator, active window title, system indicators, volume, media player, language switcher, quick settings, calendar, and power menu.
- **Interactive Widgets**: Media player with album art & playback controls, volume output selector, notification center with actions, interactive calendar, and quick system controls.
- **Custom Design System & Themes**: Built-in glassmorphism aesthetics, dynamic color tokens, custom theme creation (`create_theme`), and live theme switcher (`select_theme`).

### Satellites
- **Volume Satellite**: Quick audio output device selection and master volume control.
- **Wi-Fi Satellite**: Real-time Wi-Fi network scanning, signal strength indicators, and connection status.
- **Bluetooth Satellite**: Nearby device discovery, paired device management, and connection toggle.
- **Tray Satellite**: System tray integration for background application icons and menus.
- **Language Satellite**: On-the-fly system language switcher supporting dynamic `.toml` translation variants (`es`, `en`, and custom files).
- **Calendar Satellite**: Monthly calendar view with quick navigation.
- **Power Satellite**: Instant system power options (Lock, Logout, Suspend, Reboot, Shutdown).

### Launcher
- **App Launcher**: Ultra-fast desktop application search and launch via `freedesktop` `.desktop` entries.
- **Calculator**: Real-time mathematical expression evaluation directly inside the search bar.
- **Clipboard History**: Dedicated clipboard manager for searching history, previews, and instant copying.
- **Theme Creator & Switcher**: Dedicated modules to customize visual tokens or switch themes live.
- **Wallpaper Switcher**: Interactive wallpaper browser and switcher integrated with `awww`.

### System Integration
- **Polkit Authentication Agent**: Integrated `org.freedesktop.PolicyKit1.AuthenticationAgent` D-Bus service with native password prompt dialogs.
- **Multi-language Support (i18n)**: System-wide localization system saved in `~/.config/capsule/languages/`, automatically applying system locales (`LANG`, `LANGUAGE`).
- **Hyprland Native Integration**: Built via `hyprland-rs` for IPC events, workspace tracking, and active window monitoring on Wayland.
- **High-Performance GPUI Engine**: Powered by Zed's GPU-accelerated UI framework for smooth 60+ FPS animations and minimal CPU/memory usage.

## Installation
```bash
curl -fsSL https://raw.githubusercontent.com/moiCR/Capsule/master/install.sh | bash
```

## Special Thanks
- [zed](https://github.com/zed-industries/zed): for creating gpui and making it open source
- [tide-island](https://github.com/enhaoswen/Tide-island): for the inspiration
- [saneAspect](https://github.com/enhaoswen/Tide-island): for the inspiration (Your course is way too expensive, bro.)
- [Akira Toriyama](https://en.wikipedia.org/wiki/Akira_Toriyama): for creating Dragon Ball, inspiring millions around the world, and giving me a childhood I'll never forget. 