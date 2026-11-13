# Start-latency words, desktop bench, 2026-11-13

`tryout/start-latency` `c1470413` (= `834486dc` plus the script fixes below)
vs `origin/main` `cfd3cef9`. Ryzen 7 3700X, RTX 3070, 3440×1440 @ 174 Hz.

## Verdict

| word | verdict | deciding numbers (load 0 / CPU 8) |
| --- | --- | --- |
| `keepoverlays` | **make it the default** | `nopeeks` resize command p50 42.4 → 29.5 ms (−30% / −27%), first frame −23% / −20%, move command −18% / −14%; frame and interval columns unchanged. `nopeeks` resize answers within ~2 ms of `main` (27.6). |
| `asynchandback` | **make it the default**, after one feel test of a Notepad resize | Resize `[sync]` rows 120 → 0 (both loads); worst burst gap 170 → 49 ms / 193 → 56 ms; command, first frame, frame and interval p90 unchanged. |
| `asynclanding` | remove | `[sync]` rows stay; under CPU 8 Notepad still falls back to a 115 ms synchronous resize (worst gap 127 ms). Frame p90 +21% / +19%, interval p90 +26% at CPU 8. `asynchandback` does better at no cost. |
| `earlylanding` | remove | Interval p99 −32% / −31%, but frame p90 +64% / +149% and interval p90 5.82 → 11.50 ms at CPU 8: the motion's tail misses vblanks. Same in its combinations and under `morepeeks`. |
| `keeppeeks` | remove as it stands | `peek_new` only −2.3 / −1.5 ms (−13% / −7%), not near 0 from the second burst; resize command not separated; resize interval p99 +29% at CPU 8. Copies are rebuilt, so only the window and first thumbnail are saved. |

None of the peek words moves resize's answer: `default` stays at +120% command
and +172% first frame against `main`. The cost is in work the words don't
touch: `peek_new`, the take-over flush, `peek_cover`/`peek_handover`, and a
first `peek_frame` of 22–26 ms that has no sub-stages yet.

## Setup

- 6 tiles on workspace 1, the same in all 112 mode-rounds (logged after each
  restart): Notepad, WindowsTerminal (this session's), WindowsTerminal (new),
  chrome (Helium, the target, lowest handle), chrome (new), FPilot. AnyDesk
  minimized. Target `chrome`, so `move` swaps it with the new terminal and
  `resize` shrinks or grows all five others.
- The four programs were opened once for the whole session and closed at the
  end, rather than per run with `-Launch`, so `main` (whose runner has no
  `-Launch`) measured the same windows. `-Launch` itself was smoke-tested.
- 4 rounds × 10 bursts, modes interleaved per round, WM restarted per mode,
  `-TileOrder Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot`.
- CPU 8: 8 busy-loop jobs of 16 threads. Runs 07:38–08:31 local, no failures.
- `main` ran through `bench-run.ps1` (`main,nopeeks,default`), in
  `..\glazewm-baseline` with this branch's `perf_bench` source and
  `baseline-config.yaml` (user config unchanged since 2026-11-11).
- "change ... sep" = every round of one mode is on the same side of every
  round of the other. Only those are quoted as differences here. Full
  tables, with round ranges: `results/cmp-*.md`.

Differences from 2026-11-12:

- Window set: no UWP (`ApplicationFrameHost`) or Everything windows, and
  `explorer.exe` opens File Pilot here (the registered folder handler). So
  absolute stalls differ.
- Notepad's synchronous resize (~120 ms) only hits the first resize burst
  after each restart: the WM then moves it asynchronously for 30 s
  (`Synchronous move ... moving it asynchronously for 30s`). Medians show the
  other apps' stalls; the worst column shows Notepad's.
- `interval max (median burst)` equals `interval p99`: ≤63 intervals per burst.

## Script fixes (committed)

`c1470413`, `isolate_modes.ps1`:

- `glazewm.exe` is a GUI-subsystem app: `& $Cli ... *> $null` neither waits
  for it nor sets `$LASTEXITCODE`. `Wait-WmReady` returned at once on the 0
  `cargo build` left behind; `Close-Launched` never saw a failure. Now piped
  to `Out-Null`. `bench-run.ps1` here has the same fix (the 2026-11-12 copy
  has the bug).
- `-Launch explorer`: the window belongs to the file manager registered for
  folders (`FPilot` here), which was never found or closed. Now read from
  `HKCR\Directory\shell\open\command`.
- `wt -w new`, so a `windowingBehavior` setting can't open a tab in this
  session's terminal instead.

`cargo test --workspace`: 794 passed, 1 ignored, none failed (first run ever).
`perf_bench`'s own 2 tests pass.

## Per word (only rows whose rounds separate)

### `keepoverlays` vs `nopeeks`

| scenario | metric | `nopeeks` | `+keepoverlays` load 0 | CPU 8 |
| --- | --- | ---: | --- | --- |
| resize | command p50 ms | 42.36 / 42.22 | 29.48 (−30%) | 30.71 (−27%) |
| resize | command ms (WM side) | 42.00 / 41.80 | 29.10 (−31%) | 30.40 (−27%) |
| resize | input→first frame ms | 47.85 / 47.30 | 37.00 (−23%) | 38.02 (−20%) |
| resize | session_overlays outside ms | 16.00 / 17.90 | 6.85 (−57%) | 7.10 (−60%) |
| resize | cloak outside ms | 21.40 | 18.30 (−14%) | ~ |
| move | command p50 ms | 14.81 / 15.29 | 12.11 (−18%) | 13.13 (−14%) |
| move | input→first frame ms | 15.45 / 16.24 | 12.66 (−18%) | 14.09 (−13%) |
| move | session_overlays outside ms | 3.45 / 3.80 | 1.90 (−45%) | 1.90 (−50%) |

Frame p90, interval p90/p99: no separation (resize frame p90 +7% / +10%,
interval p99 +6% / +9%, rounds overlap).

From `perf.log`, resize, per-report medians, load 0 (`results/stage-medians.md`):

| stage (outside frames) | `main` | `nopeeks` | `+keepoverlays` |
| --- | ---: | ---: | ---: |
| `rd_frozen` | 3.5 | 7.1 | 3.6 |
| `session_overlays` | 7.7 | 14.8 | 6.1 |
| `ovl_place` | – | 12.1 | 3.8 |
| `cloak` | 14.5 | 20.7 | 16.5 |
| `platform_sync` | 29.8 | 42.7 | 31.2 |

So yes, `rd_frozen` and `session_overlays` drop to `main`'s level, as
"B: findings" 1 predicted. Not benched: its effect under `default`, where
only the old path's leftovers (e.g. floats) use the take-over.

### `asynchandback` vs `default` (resize)

No metric's round medians separate. What changes is in `perf.log`
(`results/sync-rows.md`):

| | `default` load 0 | `asynchandback` | `default` CPU 8 | `asynchandback` |
| --- | ---: | ---: | ---: | ---: |
| reports with `[sync]` rows | 39 / 39 | 0 / 37 | 39 / 39 | 0 / 38 |
| `[sync]` rows, worst ms by app | Notepad 122, FPilot 20, WT 15, chrome 9 | none | Notepad 124, chrome 23, FPilot 22, WT 6 | none |
| interval max, median report ms | 43.3 | 36.4 | 39.9 | 39.6 |
| interval max, worst report ms | 170.4 | 49.3 | 192.7 | 56.1 |
| interval p99 per round (median, range) | 45.65 (41.2–47.0) | 37.97 (35.9–41.5) | 46.36 (43.6–48.9) | 41.73 (38.4–45.3) |

Command p50 67.6 vs 71.7 ms (~), frame p90 3.75 vs 3.38 (~), interval p90
5.85 vs 5.93 (~). The remaining ~37–40 ms gap in its worst burst is the first
`peek_frame` (25 ms) and an in-frame pass (22 ms: `rp_visible` 8.7,
`dwm_flush` 8.2, `ovl_place` 6.7), not a hand-back.

### `asynclanding`, `earlylanding` vs `default` (resize)

| metric | `default` | `asynclanding` | `earlylanding` | `earlylanding+asynchandback+keeppeeks` |
| --- | ---: | --- | --- | --- |
| frame p90 ms | 3.38 / 4.00 | +21% / +19% | +64% / +149% | +37% / +167% |
| interval p90 ms | 5.93 / 5.82 | ~ / 7.36 (+26%) | ~ / 11.50 (+98%) | ~ / 12.09 (+108%) |
| interval p99 ms | 45.65 / 46.36 | ~ / 40.39 (−13%) | 30.84 (−32%) / 31.93 (−31%) | 30.37 (−33%) / 32.28 (−30%) |
| `[sync]` rows (load 0 / CPU 8) | 120 / 120 | 152 / 143, worst 12 / 115 ms | 152 / 148, worst 5 / 5 ms | 0 / 0 |
| worst report interval max ms | 170 / 193 | 50 / 127 | 37 / 36 | 37 / 37 |

Command and first frame: no separation (−4 to −8 ms at load 0, overlapping).
`earlylanding` gets the shortest landing gap because apps resize during the
motion's tail (from ~60% of a 25% shrink), but that tail then draws at 1–2
vblanks per frame under load. `asynclanding`'s sync rows are the hand-back's
`SetWindowPos` at a rect already reached (cheap), except when the 500 ms
`FILL_LANDING_WAIT` runs out (Notepad under CPU 8).

`morepeeks` agrees (`results/cmp-more.md`):
`morepeeks+earlylanding+asynchandback+keeppeeks` vs `morepeeks`: resize
interval p99 −42% / −35%, float −26% / −24%, but frame p90 +66% / +99% (resize)
and +129% / +132% (float), interval p90 ~10 ms (+56–79%).

### `keeppeeks` vs `default`

| scenario | metric | `default` | `keeppeeks` load 0 | CPU 8 |
| --- | --- | ---: | --- | --- |
| resize | peek_new outside ms | 17.70 / 19.80 | 15.40 (−13%) | 18.35 (−7%) |
| resize | cloak outside ms | 20.90 / 19.55 | 15.35 (−27%) | 17.70 (−9%) |
| resize | interval p99 ms | 45.65 / 46.36 | ~ | 59.95 (+29%) |
| move | frame p90 ms | 0.85 / 0.96 | 0.51 (−40%) | 0.48 (−49%) |
| move | command p50 ms | 19.24 / 20.46 | ~ | 19.95 (−2%) |

`peek_new` per report, resize, round 1, in burst order:

```
default    17.1 17.2 17.9 17.3 18.3 18.7 19.6 17.7 17.3 17.6
keeppeeks  18.2 16.5 15.5 14.1 15.2 14.6 15.3 15.2 16.1 15.4
```

Every round looks like this: −2 to −3 ms from the second burst on, never near 0.
Resize command p50 −7.7 ms at load 0 does not separate (`keeppeeks` rounds
59.2–68.2 ms, `default`'s 63.4–75.8). Same under `morepeeks`: `peek_new`
−12% / −12%, nothing else separates.

## `perf.log` questions

Full reports: `perf-excerpts.txt` (one resize burst per mode, round 1,
load 0, the longest interval max). Per-report medians: `results/stage-medians.md`.

**Where the ~56 ms with no stage went.** Nothing in `redraw` is unscoped
now. `default`, resize, per-report medians at load 0 (ms, outside frames):

| `redraw` 68.0 = | |
| --- | ---: |
| `redraw_loop` (of which `peek_new` 17.7) | 27.3 |
| `cloak` (of which `take_over` 13.5, `preposition` 0.2; `dwm_flush` 10.4 inside) | 19.1 |
| `peek_cover` | 10.3 |
| `peek_handover` | 10.7 |
| `peek_raise` | 1.6 |

So last night's ~56 ms was `peek_cover` + `peek_handover` + `peek_raise`
(~23 ms here with 6 non-UWP windows). `take_over` and `preposition` were
already inside `cloak`; `preposition` is ~0. Slowest input of the worst
burst: command 51.4 ms = `peek_new` ×6 17 ms, `peek_cover` 10.0, `cloak` 13.5
(flush 7.1, `take_over` ×6 6.4), `peek_handover` 4.3, `peek_raise` 1.2. Then
`peek_frame` 22.4 ms before the first frame (73.5 ms).

**Input→first frame on the peek path** (new today): resize 101 / 99 ms
`default` vs 48 / 47 `nopeeks` vs 32 / 37 `main`. Of that, first frame minus
command is ~26 ms on peeks vs ~5 on `nopeeks`: the first `peek_frame`, which
has no sub-stages. **Next lead.**

**Hand-over words, `[sync]` rows and the stall.** `asynchandback` removes
every `[sync]` row and the stall (worst 170 → 49 ms). `asynclanding` doesn't:
rows stay and Notepad still stalls 115 ms under CPU 8. `earlylanding` leaves
≤5 ms rows. The 74–95 ms of 2026-11-12 is 40–46 ms (median burst) and
170–193 ms (Notepad's first burst) on today's windows under `default`.

**`keeppeeks` and `peek_new`.** −2 to −3 ms from the second burst on, not
~0 (above).

**`keepoverlays`, `rd_frozen` and `session_overlays`.** 7.1 → 3.6 and 14.8 →
6.1 ms (above).

**Switch, `main` vs `nopeeks` (2026-11-12's `perf.log`, no new run).**
Command p50 36.7 vs 64.2 ms. One report spans several switches (`main`'s
median one has 11 passes), so per burst instead (sums over a run ÷ 10,
median of 4 rounds, `switch-perburst-2026-11-12.md`):

| stage (outside frames) | `main` | `nopeeks` | Δ |
| --- | ---: | ---: | ---: |
| `platform_sync` | 62.7 | 72.9 | +10.2 |
| `redraw` | 56.7 | 51.2 | −5.5 |
| `redraw_prep` | 16.3 | 36.8 | **+20.5** |
| `redraw_loop` | 24.6 | 11.8 | −12.8 |
| `platform_sync` − `redraw` | 6.0 | 21.7 | **+15.7** |

The extra is in `redraw_prep`, which holds the switch pre-pass
(`WorkspaceSurrogate::new` + `carry_backdrop` per window,
`park_switch_borders`, `cover_ring`, its flush), and in `platform_sync` outside
`redraw`, which was unscoped that night. It is `sync_tail` on this branch,
whose first part is `sync_focus` after the switch set-up. The reports are in
`switch-excerpts-2026-11-12.txt`. One `switch` run on this branch would
split it.

## On screen

Each word was started by hand with `GLAZEWM_PERF=1`, the keybound resize
(`resize --width +2%`, then −2%) and a move each way sent over IPC, as
keys don't pass over AnyDesk (`checks/`). For every word: no crash, every
window shown at its tile afterwards (IPC rects and one screenshot), and
errors.log only had the known warnings (`Failed to set focus`, sync-reposition
notices). Under default hand-over (seen in the `keeppeeks` check) Notepad's
first resize took 101.6 ms synchronously.

Then one timed still per word, on a config copy with 3 s move/resize motions
(`resize --width +25%`). One still can't show a frame-long artifact, so read
"not seen" as just that:

- `asynchandback`, 30 ms after the motion ended: every window at its new
  size, content reflowed. No window caught at its old size.
- `asynclanding`, same moment: nothing visible. Frames per burst equal
  `default`'s (60.5 vs 60.5), so no measurable lingering either.
- `earlylanding`, 80% into the shrink: the shrinking windows had already
  taken their new size (Terminal's text re-wrapped, FPilot's caption buttons
  at the new width), where `default` still shows them cut at the old one. No
  edge-colour strip visible, but these dark windows' edge colour matches
  their background.
- `keeppeeks`, 150 ms into the second motion (reused peeks): no doubled
  ring, no stale picture.
- `keepoverlays`, 150 ms into a `nopeeks` move: the target's ring in place
  and aligned. Plain `nopeeks` at the same moment showed only a grey edge,
  as finding B1 describes (overlays hidden for the take-over).

The first `nopeeks` stills (`checks/visual-nopeeks*`) are void: the 3 s
reorder after the restart was still running. `visual2-*` are the retakes.
The screenshots show this desktop and aren't committed; they're in
`~/.glzr/glazewm/bench-2026-11-13/screens/`.

## `main` vs this branch (`results/cmp-main.md`)

| scenario | metric | `main` | `nopeeks` | `default` |
| --- | --- | ---: | --- | --- |
| resize | command p50 ms | 27.55 / 30.66 | +45% / +42% | +120% / +123% |
| resize | input→first frame ms | 31.81 / 36.55 | +37% / +32% | +172% / +169% |
| resize | frame p90 ms | 4.24 / 4.63 | ~ | −23% / ~ |
| resize | interval p99 ms | 25.53 / 27.49 | −31% / −28% | +93% / +110% |
| move | command p50 ms | 13.35 / 13.21 | +19% / +17% | +45% / +57% |
| move | frame p90 ms | 1.32 / 1.42 | ~ / −4% | −33% / −33% |
| move | interval p99 ms | 7.37 / 9.00 | ~ | +46% / +36% |

Interval p90: equal everywhere (5.73–5.87 ms).
