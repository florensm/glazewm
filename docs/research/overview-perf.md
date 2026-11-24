# Making the overview lighter

How the overview could use less CPU and GPU, and feel lighter, from a read
of `packages/wm-platform/src/overview/` and `dwm_thumbnail.rs`. A lot is
already done well:
- only the changed properties are sent to DWM (`Thumbnail::set`);
- cards are drawn two a frame (`native/pacing.rs`);
- only the workspaces in sight are spread;
- the thread runs under MMCSS;
- the loop sleeps when nothing moves.

So the wins left are mostly in how many things move, how long they move,
and what happens on open.

Measure each change on the tryout build on the desktop PC with
`GLAZEWM_PERF` (never on the laptop): frame time, missed frames
(`Overview missed a frame` in the log) and time from key press to first
frame.

## Where the cost is

Per window on the shown workspace, the overview keeps up to four DWM
thumbnails:
- its preview;
- the thumbnails of its border and backdrop overlays;
- its preview on its miniature in the strip.

Every frame of a zoom moves all of them, one
`DwmUpdateThumbnailProperties` each. Each is a separate surface DWM
composes, sampled from the full-size window. On top of that, the framed
backdrop resizes its rounded clip and the panel under its drop shadow on
every frame (`composition.rs` `set_layout`). All of it is GPU work in DWM
rather than in our process, which is why the overview's own CPU use looks
small while the machine still works hard.

## Ranked changes

1. **Stop on pixels, not on a fraction.** The zoom spring counts as
   settled within ε = 1e-4 *of the way* (`SpringParams::default`). For a
   window moving 600 px that is 0.06 px, far below anything visible. Its
   last ~80 ms are spent moving by less than half a pixel. Settling at
   half a physical pixel of the longest travel (ε ≈ 0.5 / travel) ends a
   `spring(1, 800)` zoom at about 335 ms instead of 415 ms. That's about
   20% fewer frames per open and close, with no visible difference. The
   card and window springs already do this (`Spring::new(.., 0.25)` in
   px).
   *Where:* `parsed_config.rs` `SpringParams`, or the overview's own
   zoom timing in `layout.rs`.
   *Risk:* none visible; a test that the final frame lands exactly on
   target.

2. **Register in stages on open.** On open, every thumbnail is registered
   before the first frame (`native/screen.rs` `register`): previews,
   overlays, every miniature's previews, cards, search pictures.
   Registration is a round trip to DWM each. Register the shown
   workspace's previews first (they're what zooms), then the strip,
   cards and other scenes over the next frames. The strip only fades in
   past 80% openness anyway, and cards are already drawn a few a frame.
   *Gain:* time to first frame on a workspace-heavy setup; the open
   feels instant.
   *Where:* `native/screen.rs` `register` split into a now part and a
   deferred part, like `DrawBudget`.

3. **Fewer surfaces in motion.** During the zoom, a window's border and
   backdrop overlays are two more thumbnails per window, moved in step.
   Options, lightest first:
   - fade them in at rest, as the chrome does;
   - or keep them only for the focused window;
   - or draw the border into the preview's frame picture.

   Halves the thumbnails moving per frame on a busy workspace.
   *Risk:* the open no longer looks exactly like the desktop at its
   first frame; try "fade in at rest" first.

4. **Move less, for less time.** The size recommendations in
   `overview-ux.md` (0.69× → 0.80×) also make the overview lighter:
   windows travel about a third less, so the same spring covers less
   ground. Fewer pixels change per frame, and the motion feels lighter
   for the same reason it costs less.

5. **Don't resize the backdrop every frame.** The framed backdrop sets
   its root and rounded clip size each frame, and a `DropShadow` under a
   visual whose size changes is re-rasterised. Scaling a fixed-size
   visual (`SetScale`) and adjusting only the corner radius would be
   cheaper. Measure before changing: this is a guess about DWM's
   internals.
   *Where:* `platform_impl/windows/composition.rs` `set_layout`.

6. **Keep card pictures between opens.** Cards, names and search are
   drawn with DirectWrite into layered windows each time they're needed.
   Keep the last pictures and redraw only those whose content (window
   set, titles, size) changed. Most opens then draw nothing.
   *Where:* `native/cards.rs`.

7. **A light preset.** One option that turns off what costs most for the
   least:
   - `style: fade` with a short curve;
   - no overlays in motion;
   - no stagger or particles.

   This is for laptops on battery. For the motion-sensitive it is also
   the calm option (see the research on dizziness).

## What makes it feel lighter

How light it feels is mostly about the motion, not the cost:

- **Respond on the next frame.** The first frame within one refresh of
  the key press matters more than the length of the animation. (2) helps
  most here.
- **One thing moving at a time.** The zoom, the slab's radius and
  shadow, the strip and the chrome all move at once today. Let the
  windows move; let everything else only fade.
- **Shorter distance, not shorter time.** A 0.80× zoom over 300 ms reads
  lighter than a 0.69× one over 250 ms.
- **No long tail.** A spring creeping in for its last 100 ms reads as
  sluggish even when it starts fast. (1) fixes that too.
- **Soft starts on interrupts.** Springs already keep their velocity
  when retargeted; keep it so for every animated value, including
  opacities.

Changes (2), (3) and (6) touch no new Windows API. (5) only uses
composition calls already in use. None would need the Defender call-out
in `CLAUDE.md`.
