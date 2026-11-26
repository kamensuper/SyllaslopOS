---
name: syllaslop-console
description: Change the native SLC command console, keyboard or COM1 behavior, theme, font, framebuffer rendering, clipping, damage tracking, and guest screenshots.
---

Read the specific console, input or graphics module and the matching section of `docs/CONSOLE.md`. Preserve the one-field idle design, immediate screen updates, on-demand views and VGA-free COM1 recovery.

- Keep input state and command execution aligned across PS/2 and COM1. A search selection stages text for editing. A separate submission executes it. Preserve bounded entry, history and filtering.
- Bound glyph and pixel writes against clipping, stride, pixel format and mapped extent. Check compact layouts and zoom. Measure damage bytes separately from timing.
- Use `make test-host` for pure behavior and `make test-console` for real QEMU keyboard, scanout and recovery. Inspect 1024×768 and 800×600 screenshots for visual changes.
