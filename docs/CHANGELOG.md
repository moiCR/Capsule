# Changelog

All notable changes to Capsule are documented in this file.

## v2.0.0-b.1

### Features
- New architecture for Capsule, which includes a new experimental capsule (activated from the settings panel)
- New architecture for the satellite system (available only in the new capsule)

### Improvements
- CPU performance improvements (the issue of high CPU usage, which was consistently at 6.3%, has been resolved)
- Implementation of all modules into the new architecture has begun (Currently includes: Default, Launcher, Dashboard, Clipboard, Record, Themes, Wallpaper and Shelf—without Orbit).
- gpui-capsule evolved from a fork of gpui into gpui-ce

### Bug Fixes
- Fixed a clipboard bug that prevented items from being copied.
- The problem with the lock screen (it kept verifying continuously) has been fixed.