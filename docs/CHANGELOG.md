# Changelog

All notable changes to Capsule are documented in this file.

## v2.3.0

### Features
* **Custom Motion Profiles**: Choose how Capsule moves with multiple animation styles (Fluid, Cubic, Exponential, Back, and Spring), letting you pick between silky smooth transitions, snappy responses, or playful rebounds.
* **Elastic Motion Dynamics**: Added organic elasticity (*Squash & Stretch*) during expansions and contractions, giving Capsule and corner orbs a lifelike, dynamic feel as they resize.
* **Cascading Transitions**: Multiple panels and widgets now transition with a graceful staggered sequence instead of moving all at once, creating a more natural flow.
* **Reduced Motion Mode**: Added a dedicated accessibility option that disables spatial movement and morphing in favor of clean, instant fades, ideal for motion sensitivity or minimalist setups.
* **Fine-Tuned Motion Controls**: Personalize the exact feel of your desktop with adjustable sliders for spring bounciness and internal content transition speed.
* **Native Wallpaper Engine**: Fully built-in desktop wallpaper panel rendered natively at full screen on the lowest layer, eliminating the need for external daemons like `swww` or `awww`.
* **Smooth Wallpaper Transitions**: Seamless visual transitions when switching wallpapers, including Fade, Slide, Zoom, Wipe, and Instant modes.
* **Modern Wallpaper Settings**: A redesigned General card in Appearance settings featuring a large rounded preview of the active wallpaper, a quick file picker via the native desktop portal, and a dedicated transition selector.

### Improvements
* **Interactive Motion Preview**: Test and preview animation curves, bounciness, elasticity, and transitions in real time directly inside the Appearance settings before applying them.
* **Unified Desktop Motion**: Synchronized movement physics and timing across Capsule, corner orbs, preview cards, and satellite panels for a harmonious visual flow.
* **Simplified Theme Selector**: Streamlined the in-capsule appearance module to focus entirely on themes, removing redundant wallpaper browsing for a lighter, faster experience.
* **Settings Focus & IPC Isolation**: Opening settings now automatically closes any active capsule modules to default and blocks external IPC commands, ensuring your keyboard focus stays right where you need it.
* **Refined Staggered Spacing**: Fixed satellite gaps and decoupled screen corners so expanding one side leaves the opposite corner completely undisturbed.

### Bug Fixes
* **Wallpaper Memory Leak & Atlas Eviction**: Eliminated memory accumulation when switching wallpapers while preserving 100% native image quality. Paired GPUI GPU texture eviction (`cx.drop_image` on `WgpuAtlas`) with asset loader cleanup (`cx.remove_asset`), and limited glibc per-thread memory arenas (`mallopt(M_ARENA_MAX, 2)`) alongside `malloc_trim(0)` to keep RAM consumption strictly bounded under 200MB even with massive 4K/6K images.
* **Wallpaper Blur & Palette Color Optimization**: Fixed an RGBA-to-JPEG encoding failure in `WallpaperService::create_or_get_blur` that created 0-byte cache files, and optimized `dominant_color` extraction to sample downscaled blur thumbnails instead of re-decoding multi-megapixel images.
* **Settings Portal & Re-entrant Safety**: Resolved re-entrant borrow issues when launching the desktop portal file picker and kept settings closed upon selection to keep transitions smooth and unobstructed.
* **Media Player Seeking via Native MPRIS D-Bus**: Replaced external `playerctl` process calls with native D-Bus MPRIS `SetPosition` and `Seek` calls, eliminating errors when seeking or controlling players on systems without `playerctl`.


<div align="center">

## Preview

</div>

https://github.com/user-attachments/assets/baf17a29-27d1-4f04-a97b-43b860c84d84