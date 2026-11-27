# References and licensing policy

SyllaslopOS is MIT-licensed (copyright kamensuper). Every file in this
repository must be compatible with that license.

## Slopix (predecessor)

- Source: https://github.com/kamensuper/slopix (studied as a read-only clone outside this repo).
- License: the first-party code is MIT (`LICENSE-FIRST-PARTY-MIT`). The
  combined kernel is distributed under GPLv3 because it links DoomGeneric
  and other third-party code (`THIRD-PARTY.md`).
- Use: we study its design and its mistakes. First-party Slopix code may be
  adapted under MIT, but only after we understand its assumptions, and the
  commit must say it was adapted. Never take code from Slopix's
  GPL/third-party parts (`third_party/`, `vendor/`, the Doom port).

## Syllable Desktop / AtheOS

- The main design reference for the overall architecture (kernel/server
  split, application server, messaging, the native GUI toolkit) and for
  where the desktop goes in the future.
- License: mostly GPL (kernel and most system components). Some libraries
  are LGPL.
- Use: **reimplement, never copy.** Read the docs and source to understand
  the concepts and interfaces, then write our own implementation. Do not
  paste code, transliterate it line by line, or reproduce non-trivial
  structures verbatim.

## Policy summary

1. Writing it ourselves is the default.
2. Outside code needs a license check first. MIT/BSD/ISC is acceptable with
   attribution. GPL/LGPL is not, unless the user decides to change the
   project license.
3. Record any adapted code here and in its commit message.

## GRUB build dependency

The ISO contains distribution GRUB BIOS binaries. They are used as external
bootloader components under GPL-3.0-or-later and are not part of the MIT kernel.
For the Debian GRUB build used in the verification record, the matching source
manifest and source archives are available from Debian's
[GRUB source directory](https://deb.debian.org/debian/pool/main/g/grub2/).

## Console design and asset research

The brief supplied written descriptions of OpenCode, OpenClaude and a dark
perforated sphere. The interface takes their hierarchy, restrained accent
and dot material cues. The layout, S mark and procedural dots are original.
The terminal programs below were studied for interaction and rendering
patterns. Their code and runtime dependencies were not incorporated.

| Reference | Applied idea and boundary |
| --- | --- |
| [OpenCode](https://github.com/anomalyco/opencode) and [OpenTUI](https://opentui.com/docs/releases/0.8.0/) | A focused entry surface with an incremental renderer. Direct framebuffer rectangles replace terminal cells and the host layout engine. |
| OpenClaude and the sphere, as described in the brief | Large identity typography, a restrained amber accent and a static dot-lit relief. The surface has no chat thread or animation. |
| [Bubble Tea](https://github.com/charmbracelet/bubbletea/tree/main/tutorials/basics) and [Lip Gloss](https://github.com/charmbracelet/lipgloss) | Device-independent updates and reusable semantic styles. Fixed arrays and RGB primitives fit bare metal. |
| [Textual](https://textual.textualize.io/guide/input/) and [Ratatui](https://docs.rs/ratatui/latest/ratatui/struct.Terminal.html) | Focus consumes keys before global command entry. Dirty rectangles take the role of cell diffs on an MMIO framebuffer. |
| [Helix](https://docs.helix-editor.com/keymap.html) and [Zellij](https://zellij.dev/documentation/keybindings) | Explicit view keyboard behavior and a return key. This early console has one active view. |
| [btop](https://github.com/aristocratos/btop), [Yazi](https://yazi-rs.github.io/docs/) and [Midnight Commander](https://github.com/MidnightCommander/mc) | Compact system readouts, selection and keyboard discovery. Process controls and filesystem panels depend on kernel services. |
| [Syllable](https://syllable.metaproject.frl/), [Haiku Interface Kit](https://www.haiku-os.org/docs/api/group__interface.html) and [Plan 9 rio](https://man.cat-v.org/plan_9/1/rio) | Workstation-style ownership, consistent actions and separation between display and application state. The firmware-backed kernel console is a bring-up stage toward a userspace application. |
| [Linux framebuffer format guide](https://docs.kernel.org/fb/api.html) and [QEMU QMP](https://www.qemu.org/docs/master/interop/qemu-qmp-ref.html) | Respect pitch and channel layout. Validate through guest input, serial output and the emulated VGA scanout. |

The glyph outlines in `kernel/graphics/assets/mono.outline` are extracted
from DejaVu Sans Mono 2.37. Their
[font license](../kernel/graphics/assets/DejaVu-LICENSE.txt) travels with
the asset. `python3 tools/bake-outlines.py [DejaVuSansMono.ttf]` regenerates
it with only the Python standard library. The asset is not a font file and
does not use the Bitstream or DejaVu names. Normal kernel builds use the
committed asset.
