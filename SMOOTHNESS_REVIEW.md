# Smoothness and laptop performance review

2026-11-10, on `tryout/smooth` (cut from `tryout/particles` at `b6c9c8a8`).
Done in a cloud session: nothing here was seen running. Every fix was
built, linted and unit-tested for the Windows target (see "Checks run").
`TODO.md` "Not checked live yet" 14-19 has the live checks.

Findings are ranked by expected effect on the laptop. "Stand-in" means the
WM's own click-through window that draws a moving window (as in
`STAND_INS_PLAN.md`).

## Summary

| # | Finding | Status |
|---|---|---|
| 1 | Frames were evaluated at uneven times | fixed `0ab906a6`, `d8598e3f`, `25cffc9f` |
| 2 | A retargeted move jumped up to half a frame of travel | fixed `4f63c742` |
| 3 | Every move and resize ended with a frame-long stall | fixed `4da212d7` |
| 4 | Wallpaper surfaces re-baked and duplicated with two displays | fixed `be26317b` |
| 5 | A closed overview kept every card's pixels | fixed `a6a2bf33` |
| 6 | Unknown which GPU the backdrop device is on | logged `84b0b949` |
| 7 | Wallpaper surfaces are full resolution | proposed |
| 8 | The timer thread's scheduling boost is a Defender suspect | proposed |
| 9 | One frame of wait between key press and first motion | proposed |
| 10 | Second frame of every animation steps unevenly | proposed |
| 11 | Overview wallpaper surfaces held while closed | proposed |
| 12 | Moves and resizes restart from rest when retargeted | proposed (design) |
| 13-17 | Smaller items | proposed |

## Fixed

### 1. Frames were evaluated at uneven times

- **Where**: `packages/wm-platform/src/frame_clock.rs:56` (new),
  `packages/wm/src/animation/manager.rs:1283` (fed by the timer thread),
  `:2220` (read), `packages/wm/src/animation/scroll_view.rs:679`,
  `packages/wm-platform/src/native_overview.rs:1268`.
- **What**: an animation's position each frame was computed for the time
  the timer thread *woke* for the vblank, plus half a period. A wake lands
  a varying while after its vblank, later on a loaded machine, while
  frames are shown a steady period apart. So motion advanced unevenly: at
  10 px/ms (a 2560 px workspace slide peaks above that) one wake 2 ms late
  is a 20 px stutter, with no frame dropped. Three places were worse:
  - Scrolls installed no vsync waiter (moves, resizes, opens, closes and
    switches did). They were paced by `DwmFlush`, i.e. the primary
    monitor, and evaluated at `Instant::now()` on the WM thread, which
    also varies with the up to 4 ms of queue catch-up before each tick.
  - The overview stepped its springs by wall-clock time between wakes.
  - The waiter's period was only ever refined downward: one late wake
    followed by one on time shortened it for the waiter's life, and a
    panel running below its mode's rate (dynamic refresh rate) was never
    followed.
- **Fix**: `FrameClock` estimates the vblank times: it steps by its
  period, and phase and period lean slightly toward each wake. A lone
  late wake is left out; an early one restarts it (a wake can't precede
  its vblank); a rate change is measured afresh. Fed after either wait,
  so `DwmFlush` pacing gets it too. Scrolls install their monitor's
  waiter. The overview steps by the same clock.
- **Evidence** (unit test `steps_evenly_through_late_wakes`, wakes
  0-1.5 ms late, one in 17 another 3 ms late, 60 Hz):

  | Frame-to-frame step error | Raw wakes | Clock |
  |---|---|---|
  | worst | 4355 us | 350 us |
  | rms | 1168 us | 85 us |

  The real wake jitter on either machine is not measured; the profiler
  doesn't record it.
- **Expected win**: steadier motion in every animation, most in scrolls
  and on the laptop (memory pressure delays wakes). No change in frame
  count or WM-thread time.
- **Risk**: medium. Timing code that only a live run can judge. A stalled
  timer falls back to the wall clock after 100 ms without a wake.

### 2. A retargeted move jumped up to half a frame of travel

- **Where**: `manager.rs:2462` and the block after `update_target`,
  `scroll_view.rs:440`, `packages/wm/src/animation/state.rs` (`retargeted`,
  `progress_per_ms`).
- **What**: a move retargeted mid-way (a second key press while columns
  still scroll) started from where the old one was at `Instant::now()`,
  with its clock started on the next tick, one period back. Frames are
  evaluated half a period ahead of each wake, so that sample was up to
  half a frame of travel behind or ahead of what was last drawn: the
  first frame after every retarget stepped 0.5 to 1.5 times as far as
  its neighbours. A second retarget within 8 ms of the first also read a
  speed of zero.
- **Fix**: sampled at the time frames are evaluated at, and the new
  movement's clock starts there. A movement that needs a new stand-in
  still starts its clock on its first tick.
- **Evidence**: unit test `retarget_on_the_frame_clock_keeps_steps_even`
  (steps within 2 px across a retarget at the same speed).
- **Expected win**: no kink per key press in fast keyboard scrolling.
- **Risk**: low to medium.

### 3. Every move and resize ended with a frame-long stall

- **Where**: `manager.rs:2057` (removed block).
- **What**: on the tick a finished move's stand-in was dropped, the WM
  thread moved the window's overlays behind the real window, then blocked
  on `DwmFlush`: up to 16.7 ms at 60 Hz with nothing else animating and
  no key handled. The move it waited for changed nothing: the loop a few
  lines above makes the same call with the same anchor on every tick of
  the fade-out, at least one composition earlier. For a window in a stack
  it even drew the ring without its tab bar for that frame.
- **Evidence**: both calls are `defer_rect(rect, window handle)`;
  `overlay_z_anchor` returns the window's own handle.
- **Expected win**: one frame less stall per move, resize and open; one
  dropped frame less when another animation overlaps the landing.
- **Risk**: low to medium. The check is the end of a move of a window
  with a backdrop (`TODO.md` 16).

### 4. Wallpaper surfaces re-baked and duplicated with two displays

- **Where**: `packages/wm-platform/src/platform_impl/windows/wallpaper_surface.rs:195`.
- **What**: the cache of baked wallpaper surfaces held 4. One display uses
  up to 5 keys (focused, unfocused, urgent style; the overview's plain and
  blurred image), two displays with the overview opened once use 8.
  Evicting a key in use frees nothing (brushes keep the surface alive),
  but the next request bakes a second copy: the file decoded and blurred
  again on the composition thread, and both copies held. Requests come
  with every new window, every focus fade between two styles
  (`composition.rs:574`), every display change.
- **Fix**: 5 per display.

  | Displays | Keys in use | Cached before | Cached after |
  |---|---|---|---|
  | 1 | 4-5 | 4 | 5 |
  | 2 | 8-10 | 4 | 10 |

- **Expected win**: laptop with an external monitor: no bake (tens of ms
  on the composition thread; not measured) on focus changes and new
  windows, and no duplicate surfaces (15 MB each at 2560x1440).
- **Risk**: low. Cost: after a config reload that changes blur or colour
  grading, up to one more stale surface per display stays until pushed
  out.

### 5. A closed overview kept every card's pixels

- **Where**: `packages/wm-platform/src/native_overview/screen.rs:320`,
  `packages/wm-platform/src/overview_thumbnails.rs:119`.
- **What**: each card and tab bar picture kept its bitmap after the
  overview closed, though cards are always redrawn on the next open.
- **Held before, per display** (sizes worked out from the layout code,
  not measured; "x2" assumes DWM keeps a copy of a layered window's
  bitmap):

  | Layout | Pictures | Bytes | x2 |
  |---|---|---|---|
  | activities, 5 workspaces, 2560x1440 | 5 x 352x224 | 1.6 MB | 3.2 MB |
  | stack, 5 workspaces | 5 x 1212x738 | 17.9 MB | 35.8 MB |
  | one strip four displays wide | 4649x738 | 13.7 MB | 27.4 MB |

  Upper bound per picture: 8.4 Mpx, 33.5 MB (`overview_chrome.rs:141`).
- **Fix**: pixels freed on close, windows kept.
- **Expected win**: that memory back at idle, and no paging it back in as
  the overview opens.
- **Risk**: low.

### 6. Which GPU the backdrop device is on

- **Where**: `packages/wm-platform/src/platform_impl/windows/graphics_device.rs:152`.
- **What**: `D3D11CreateDevice` is called with no adapter (`:125`), so
  Windows chooses. The device only bakes wallpaper surfaces; nothing is
  presented through it per frame, so a mismatch costs a copy per bake,
  not per frame. A device on the discrete GPU would keep it powered.
- **Fix**: logged as the device is created: `INFO` when no other adapter
  drives a display, `WARN` (so it reaches `errors.log`) naming displays
  and adapters when one does. Nothing else changes.
- **Next step**: read `errors.log` on the laptop, with and without the
  external monitor (`TODO.md` 19). Choosing the adapter needs
  `IDXGIFactory6::EnumAdapterByGpuPreference`, a new API: proposal only.

## Proposed

### 7. Bake wallpaper surfaces at reduced resolution

- **Where**: `wallpaper_surface.rs:600` (`bake`), `:318` (`crop_brush`).
- **Evidence**: one BGRA surface per display per style, full size:
  14.7 MB at 2560x1440. Two window styles plus the overview's two is
  59 MB per display, in system RAM on an integrated GPU. The module's own
  doc calls this "the obvious lever". At half size per axis a blurred
  surface is 1/4 the bytes (44 MB saved per display), and every backdrop
  pixel the compositor draws reads a quarter of the texture memory.
- **Why not done**: needs the brush scaled back up
  (`CompositionSurfaceBrush.Scale`, a new API), the crop and parallax
  offsets rescaled, and a live A/B for banding. As a config option, off
  by default, e.g. `backdrop.bake_scale: 0.5`. The overview's unblurred
  image must stay full size.
- **Risk**: medium (visible if wrong).

### 8. The timer thread's scheduling boost is a Defender suspect

- **Where**: `manager.rs:1231`, `packages/wm-platform/src/lib.rs`
  (`try_set_thread_mmcss`, `set_thread_priority_highest`).
- **Evidence**: the animation timer thread sets `THREAD_PRIORITY_HIGHEST`
  and registers with MMCSS, resolving `avrt.dll` by
  `LoadLibraryW` + `GetProcAddress`. This is existing code I left alone
  (your limit: no scheduling changes by any route). But resolving an API
  by name at run time is a well-known malware trait, like the reverted
  `SetProcessInformation`, and its purpose (less wake jitter) is now
  largely covered by finding 1.
- **Proposal**: on the desktop, A/B a build without both calls. If motion
  is as smooth, drop them from the laptop build first.
- **Risk**: low to try; your call.

### 9. One frame of wait between key press and first motion

- **Where**: `scroll_view.rs:717`, `platform_sync.rs:587`, `:1067`.
- **Evidence**: before a real window is cloaked its stand-in must be
  composited once, which is bought with a `DwmFlush` on the WM thread:
  0 to 1 frame, then the first tick. So key to motion is at least one
  frame plus stand-in creation. Hyprland has none of this, being the
  compositor.
- **Options**: (a) start moving on the flush's frame instead of after it
  (the stand-in is composited by the first tick anyway; cloak one tick
  later, as the teardown is already spread over ticks); (b) keep stand-ins
  of visible columns warm at rest (memory and compositor cost).
  Both change the take-over that `STAND_INS_PLAN.md` step 2 unifies, so
  they belong after it. Measure first: `input->first frame ms` in
  `perf_bench`.

### 10. Second frame of every animation steps unevenly

- **Where**: `manager.rs` `update_internal` (`clock_origin`), `:2220`.
- **Evidence**: the first tick comes at once, on the wall clock; the
  second is on the vblank clock. Depending on where in the refresh the
  key landed, the step between them is 0.5 to 1.5 periods of travel.
- **Proposal**: `DwmGetCompositionTimingInfo` gives the vblank time and
  period directly, so the first frame could be on the clock too. A new
  API: own commit, your call.

### 11. Overview wallpaper surfaces held while closed

- **Where**: `composition.rs:2145`, `native_overview/screen.rs` `close`.
- **Evidence**: two surfaces per display (29.5 MB at 2560x1440) stay
  after the first open, for the life of the WM.
- **Options**: release them on close and re-bake on open (costs a bake
  at open, against "starts the instant I press the key": measure the
  bake first); or with finding 7, the blurred one shrinks to a quarter.
  Releasing on a low-memory signal needs
  `CreateMemoryResourceNotification`, a new API.

### 12. Moves and resizes restart from rest when retargeted

- **Where**: `manager.rs:2489`.
- **Evidence**: only columns keep their speed across a retarget
  (`WindowAnimationState::retargeted`). Other moves and resizes keep
  their position (and since finding 2, exactly) but restart the easing
  curve, so with an ease-out each retarget is a burst of speed.
- **Why not done**: a feel decision. Hyprland itself restarts its bezier
  curves from the current value; keeping velocity is what niri's springs
  do. Extending `retargeted` to all moves is a few lines; it needs your
  A/B while holding a resize key.

### Smaller items

13. **Landing flush with transparency off** (`platform_sync.rs:1695`,
    `:1860`): any visible window that moved without a stand-in costs a
    `DwmFlush`, whether or not it uses the transparency effect. No gain
    with your config (transparency on); gate it on the effect.
14. **Landing one frame late** (`scroll_view.rs:281`, `state.rs`
    `is_complete`): completion is judged at the wall clock, frames half a
    period ahead, so the hand-back starts a tick after the last frame
    of motion. Latency only.
15. **Idle wake-ups** (`main.rs:327`): 10 per second so messages sent to
    the WM thread's windows are handled, plus one every 5 s. A few
    microseconds each. Removing them means moving those windows to a
    thread with a message loop. Not worth it alone.
16. **Per-tick allocation** (`manager.rs:1592`, `sync_overlays`): a set of
    urgent windows and a walk of all windows are built every tick, also
    with no workspace switch playing. Microseconds; `tick ms/frame` in
    `perf.log` would show if it matters.
17. **Unshown backdrop style left visible at opacity 0**
    (`overlay_copies.rs:199`): after a focus fade the old style's copy
    stays visible at opacity 0. Whether DWM skips it is unknown;
    `perf_bench --dwm` on the desktop could tell.

## Ruled out

- Animations left running at idle: none. No composition animation loops;
  particle windows are destroyed by a timer; the overview thread blocks
  in `WaitMessage` when nothing moves.
- A particle pool held at idle: none. A burst is one window and up to 480
  sprites with no surfaces, at most 8 bursts.
- Pixels owned by border and backdrop overlay windows: none
  (`WS_EX_NOREDIRECTIONBITMAP`; brushes and shapes only).
- Parallax images: none; parallax is a brush offset on the same surface.
- Per-frame cross-adapter copies by the WM's device: none; it only bakes.
- DXGI factory cost per gesture: already cached (`lib.rs`).
- Stale vsync waiter across idle gaps: already cleared at idle.

## Not done

- **Leak audit** (handles, GDI objects, thumbnails over hours): the
  read-only sweep for it failed part-way (usage limit) and I did not
  redo it. Open.
- `STAND_INS_PLAN.md` step 1: left alone, as asked.

## Visible changes

None intended beyond steadier motion. In particular: animation curves,
durations and config keys are unchanged. Possible side effects to watch:
the half-frame lead of scrolls now counts from the vblank rather than the
WM thread's clock (a constant shift under half a frame).

## New Windows APIs

| Commit | API | Why |
|---|---|---|
| `84b0b949` | `IDXGIDevice::GetAdapter`, `IDXGIAdapter::GetDesc` | name the backdrop device's adapter |

Both are queries under the already enabled `Win32_Graphics_Dxgi` feature.
`be26317b` passes a new index (`SM_CMONITORS`) to `GetSystemMetrics`,
which was already called. No new dependency, crate feature, or scheduling
change. To rule `84b0b949` out after a quarantine, revert it alone:
nothing depends on it.

## Checks run

Off Windows, for `x86_64-pc-windows-gnu`:

- `cargo clippy --workspace --all-targets -- -D warnings`: clean.
- Tests under Wine, one thread: `wm` 260 passed; `wm-common` 23 passed;
  `wm-platform` 282 passed, 8 failed. The same 8 fail on `b6c9c8a8`
  (they need a real desktop, as `TODO.md` "Checks" says).

## Commands for the desktop

Before is `tryout/particles` (`b6c9c8a8`), after is `tryout/smooth`. For
each: exit GlazeWM, check out, `cargo build --release`, then:

```powershell
$env:GLAZEWM_PERF = "1"; .\target\release\glazewm.exe
```

```powershell
cargo run -p wm-cli --release --example perf_bench -- --scenario resize --target chrome --bursts 10 --label before
cargo run -p wm-cli --release --example perf_bench -- --scenario move --target chrome --bursts 10 --label before
```

Use `--label after` on `tryout/smooth`. Expect `dwm_flush calls (in
frames)` to drop by about one per burst (finding 3) and `frame p90 ms`
with it. Finding 1 changes when frames are evaluated, not how long they
take, so `perf.log` won't show it: judge it by eye, on a scroll.

## Docs

- [`DwmFlush`](https://learn.microsoft.com/windows/win32/api/dwmapi/nf-dwmapi-dwmflush)
- [`IDXGIOutput::WaitForVBlank`](https://learn.microsoft.com/windows/win32/api/dxgi/nf-dxgi-idxgioutput-waitforvblank)
- [`DwmGetCompositionTimingInfo`](https://learn.microsoft.com/windows/win32/api/dwmapi/nf-dwmapi-dwmgetcompositiontiminginfo)
- [`D3D11CreateDevice`](https://learn.microsoft.com/windows/win32/api/d3d11/nf-d3d11-d3d11createdevice)
  and [`EnumAdapterByGpuPreference`](https://learn.microsoft.com/windows/win32/api/dxgi1_6/nf-dxgi1_6-idxgifactory6-enumadapterbygpupreference)
- [`CompositionSurfaceBrush`](https://learn.microsoft.com/uwp/api/windows.ui.composition.compositionsurfacebrush)
- [`CreateMemoryResourceNotification`](https://learn.microsoft.com/windows/win32/api/memoryapi/nf-memoryapi-creatememoryresourcenotification)
- [Multimedia Class Scheduler Service](https://learn.microsoft.com/windows/win32/procthread/multimedia-class-scheduler-service)
- [Hyprland: animations](https://wiki.hypr.land/Configuring/Animations/)
  and [variables (`misc:vfr`)](https://wiki.hypr.land/Configuring/Variables/)
