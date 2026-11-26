# OpenCode v2 CLI research for SLC

## Source and scope

This study uses the public OpenCode **v2.3.2** CLI source at
`c0ad18b402eb2827fc1ee71c171b1177b3fc02d1` and the OpenTUI release and
runtime documentation. The OpenCode lockstep package catalog declares
OpenTUI 0.8.4 for both core and Solid. The supplied 610×259 image is a
Syllaslop visual reference. It was evaluated separately from upstream source.
Findings about implementation come from a sparse checkout of the pinned
release. No timings were inferred from watching the supplied still image.

- [OpenCode v2.3.2 release](https://github.com/anomalyco/opencode/releases/tag/v2.3.2)
- [Release package catalog](https://github.com/anomalyco/opencode/blob/v2.3.2/package.json)
- [Official v2 terminal changelog](https://opencode.ai/changelog?area=terminal)

## Composition and focus

| Evidence in v2.3.2 | What it means for SLC |
| --- | --- |
| The home route uses flexible space above and below a centered content group. The prompt has a 75-cell maximum, with tighter side padding on narrow terminals. [Home layout](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/routes/home.tsx#L81-L107) | Cap the column width and position the idle command as one group. At 1024×768 and 800×600, whitespace should come from layout rather than filler elements. |
| The home screen contains its logo and prompt. Update notifications and form overlays have explicit visibility conditions. A separate footer slot exists for the upstream app. [Home route](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/routes/home.tsx#L81-L124) | Keep one Syllaslop wordmark and the input in the idle state. Omit default metadata, rails and a permanent shortcut strip. |
| The running session uses a scrollbox for the transcript and a distinct composer. Routing binds a prompt reference for each context. [Session layout](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/routes/session/index.tsx#L1527-L1663) | Let a result occupy space only after a command. Keep SLC's prompt above normal command output and give requested modal views a single return hint. |
| The composer restores draft text and cursor offsets by route. When a modal opens, the background input becomes non-focusable and reclaims focus when the modal closes. [Prompt focus and draft](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/component/prompt/index.tsx#L678-L755) | Keep the existing SLC editor and modal dispatch; animation moves pixels and never owns focus. Esc should expose the same draft immediately. |
| OpenCode can retain the reading position or jump to the most recent result on submission. Cursor shape and color settings sit in CLI config. [TUI settings](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/config/index.tsx) | Preserve SLC's existing history and scroll semantics. Use one visible caret and one context-specific hint. |

The reference image's dominant tone is RGB (12, 13, 17), found in about
97.5% of its pixels. Its spatial emphasis comes from the restrained center
group and large quiet areas. The previous SLC screenshot used its darkest
background in about 24.0% of pixels because it filled most of the frame
with three raised bands. The redesign uses the reference's backdrop tone
and removes the large colored surfaces entirely. The reference still
contains repeated branding, an outlined field and corner metadata.
The user's explicit requirements call for omitting those elements.

## Motion and rendering

| Upstream mechanism | What SLC takes from it |
| --- | --- |
| `ui/animation.ts` has eased tweens and a damped spring. It starts one 16 ms timer when work appears, takes new targets from current values, and stops the timer when all tasks settle. [Animation engine](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/ui/animation.ts) | Use a few short integer tweens. Start them at view changes and stop scheduling frames when they reach their targets. SLC needs no continuous frame loop or spring. |
| The prompt reveals agent/model metadata with a one-time fade. `FadeInText` uses a separate 200 ms sweep that drops its live render request on completion. Session tabs use local tweens around 180–250 ms for glow, dimming or label changes. [Prompt metadata](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/component/prompt/index.tsx#L1589-L1605) · [200 ms fade](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/component/fade-in-text.tsx) · [Tab transitions](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/component/session-tabs.tsx#L140-L224) | Reveal a new result and give a re-anchored prompt a tiny settling movement. Retain the general duration range. Use uniform opacity and an original composition rather than a swept replica. |
| The normal spinner advances every 80 ms; a busy composer variant uses 40 ms. A static indicator replaces the spinner with animations disabled. [Spinner](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/component/spinner.tsx) · [Busy composer](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/component/prompt/index.tsx#L1919-L1945) | SLC commands finish synchronously. There is no spinner, pulsing status, artificial delay or blinking glyph to advertise work that does not exist. |
| The CLI passes a 60 fps target into OpenTUI. The spinner renderable uses the shared animation clock, separately from a persistent live render request. OpenTUI's 0.8 notes distinguish dirty rendering from repainting an entire frame. [Renderer setup](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/app.tsx#L248-L264) · [Spinner renderable](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/component/spinner-renderable.ts) · [OpenTUI 0.8 notes](https://opentui.com/docs/releases/0.8.0/) | Use temporary redraws around affected text bands, with no screen paint after settling. Scale down the cadence because SLC writes pixels directly rather than submitting terminal cell diffs. |
| The command palette toggles `animations`; the toggle snaps active tween values to the endpoint. [Animation setting](https://github.com/anomalyco/opencode/blob/v2.3.2/packages/tui/src/app.tsx#L1230-L1246) | Expose `motion off` and `motion on` through SLC's existing registry. Input and output remain usable if calibration fails or motion is off. |

The source shows small event-linked effects. It does not establish that
OpenCode slides its entire home screen into its transcript or animates the
idle command line continuously. SLC's selected reflow is an original
response to guest framebuffer space. Related accessibility guidance also
supports an off switch for interaction-triggered motion.
[W3C animation from interactions](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html).

## Composition comparison

| Candidate | Idle and result geometry | Decision |
| --- | --- | --- |
| Fixed center | The same central command group remains in place when results appear. Output flows immediately below it. The input stays under the user's eye at all times. | Closest to the reference's centered gesture. The fixed vertical anchor leaves substantially less room for wrapped output and command lists at 800×600. |
| Center to work column | Idle is one quiet group. A result creates a higher, centered reading column with a left-aligned input and output. Clear returns to the idle group. | Selected. The idle screen preserves the reference's emphasis and the active view gains useful rows. Reflow snaps to the new anchor; the local movement around that anchor is deliberately small. |

Both compositions use the same near-black backdrop and typography. They
change how results take up space. Neither uses the predecessor's hero, panel,
texture, border, activity strip or pinned footer.

The fixed-center comparison was also built as a temporary 800×600 QEMU
guest, using the same renderer and command handlers with a single layout
change. The fixed center has space for nine activity rows at default
scale. The selected layout has space for eighteen. The archive includes
both guests' output after the same command sequence, plus the selected
layout at both resolutions.
