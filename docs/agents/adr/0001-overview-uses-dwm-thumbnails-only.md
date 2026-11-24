# ADR-0001: The overview shows windows with DWM thumbnails only

**Status:** accepted, 2026-11-24

## Decision

The overview draws windows with DWM thumbnails (`DwmRegisterThumbnail`)
and nothing else. No `Windows.Graphics.Capture`, screen grabs or
undocumented DWM calls (e.g. shared thumbnail visuals) in any build bound
for the work laptop.

## Why

- **Defender.** The WM already has a keyboard hook, synthesized input,
  cloaking of other apps' windows and auto-start. Adding screen capture
  puts "keyboard hook plus screen capture" in one binary, the profile of
  spyware that Defender's machine-learning verdict (`Bearfoos`) looks for.
  `dcomp-spike` may already have been flagged. A hit on the work laptop
  can also reach its IT team.
- **Latency.** Thumbnails are composed by DWM in the same pass as the
  window: no delay. Capture delivers its first frame a few composition
  cycles after it starts (`1bb99b8b` on `dcomp-effect-tryout`) and runs a
  frame behind after that, so it can't serve the first frames of an open.
- **Cost.** A thumbnail costs nothing while still; capture is a GPU copy
  per window per frame.
- **Screenshots** were ruled out too: stale, and periodic screen grabs read
  as screen scraping.

## What it gives up

Thumbnails are axis-aligned rectangles: crop, place, scale, opacity. No
rotation, perspective or bending, and no shaders over them. 3D looks
(a bent ring, flips, tilts) are faked with layout: position, size,
opacity and stacking order.

## Revisit when

A capture-based effect is wanted on the desktop PC only: keep it in a
tryout branch, out of test builds (add it to the strip list in
`CLAUDE.md`), and scan the full WM binary with it, not a spike, before
anything else.
