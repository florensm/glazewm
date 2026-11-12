# Verdict

"Smoother" means interval p90 and/or frame p90. "Costlier" means
`tick` per frame. "Start" means command p50, the stand-in for
input→first frame. Percentages come from the controlled runs (fixed tile
order, 4 rounds), as load 0 / CPU 8, and are only given where the
rounds separate (≥5%). "Same" means no separation. The requested runs
(3 rounds, tile order flipping) agree on every verdict below, with
wider ranges. On `move` they found no difference at all, because there
the differences are a few ms and the position flip drowned them.

| scenario | `default` vs `main` | `default` vs `nopeeks` | `morepeeks` vs `main` | `morepeeks` vs `nopeeks` |
| --- | --- | --- | --- | --- |
| move | same smoothness (both at refresh; frame p90 −31/−38%), costlier, start +56/+58% | same smoothness (frame p90 −31/−32%), costlier, start +33/+35% | as `default` | as `default` |
| float | same (not on peeks); start +34% at load 0 | same | **smoother** (frame p90 −44% at load 0), costlier, start +85/+78% | **smoother** (−28% / −49% at CPU 8), costlier, start +30/+64% |
| minimize | same; start +38% at CPU 8 | **smoother** (frame p90 −33/−27%), slightly costlier | same; start +29% at CPU 8 | **smoother** (−34/−27%), slightly costlier |
| resize | **smoother** (interval p90 −28/−39%, frame p90 −51/−51%), costlier, start +78/+89% | **smoother** (frame p90 −45/−49%; interval −38% at CPU 8), costlier, start +50/+40% | as `default` (start +71/+91%) | as `default` (start +45/+41%) |
| switch | smoother frame p90 (−51/−48%), cheaper, start +48/+55%. Not peeks: a build difference | same | as `default` | same |

- **move**: every mode stays on the refresh rate (interval p90
  5.75–5.77 ms, longest gap 11–19 ms). The peek modes' frame p90 is
  lower (0.6–0.7 vs 1.0–1.2 ms). They spend more per frame and start
  later: 24–26 ms vs 18–19 (`nopeeks`) and 16–17 (`main`). At load 0
  they had 2 frames >2× budget per burst vs 0, but 1 vs 1 under CPU
  load, so that isn't robust.
- **float**: `default` doesn't draw floating↔tiling on peeks
  (`peek_frame` 0) and behaves like the old engine. `morepeeks` does,
  and its frame p90 stays at 3.7–6.0 ms in every round of every run
  set, while the old engine ranges 4–11 ms round to round. That is why
  only some comparisons separate. The cost is the steepest of all
  scenarios: commands 138/163 ms vs 106/100 (`nopeeks`) and 74/91
  (`main`), and a longest gap of 74–107 ms vs 52–66.
- **minimize** (the neighbours re-tile around chrome): `nopeeks` is the
  rough one (frame p90 5.1–5.7 ms). In the controlled runs `main`,
  `default` and `morepeeks` are all at 3.5–4.1 ms, so peeks fix a
  regression the old engine has in the new build rather than beating
  `main`.
- **resize**: the clearest result, beyond noise in all four run sets.
  Peek modes reach interval p90 6.0–7.0 ms and frame p90 4.2–5.4 ms,
  against 8.0–11.8 / 7.7–11.8 ms for `main`/`nopeeks`. The price is
  `tick` +17–44%, plus `peek_frame` 3.3–3.8 ms. Commands take
  130–160 ms, against 90–114 ms for `nopeeks` and 76–85 ms for `main`.
  The longest gap is one 74–95 ms stall, against several shorter ones
  (see Reading the numbers).
- **switch**: no peeks involved. The tryout build, in every mode, halves
  `main`'s frame p90 (1.7–2.2 vs 3.6–4.1 ms) and its longest gap (42–49
  vs 90–102 ms), and uses 20–28% less `tick`. It answers 46–75% later
  (54–65 vs 37–39 ms).
- **The build without peeks** (`nopeeks` vs `main`): smoothness is the
  same or better (`switch`), except `minimize` (frame p90 +55% at load
  0). Per-frame cost is 13–28% lower. Yet commands take 17–75% longer
  (`move` +17/+17%, `float` +42% at load 0, `resize` +18/+35%,
  `switch` +75/+57%). So part of the peek modes' start delay against
  `main` isn't peek drawing at all.

**Overall**: the new peek drawing is **smoother during the motion**
wherever it is used (`resize` always, `float` with `morepeeks`). The
margin is far beyond the noise: roughly half the frame p90, and interval
p90 from 1.4–2 vblanks to 1.05–1.2. It is **not faster**. It costs more
WM time per frame, and it moves the expensive work (repositioning the
real windows, blocked on slow apps) into one long stall instead of
spreading it over the animation. The command that starts the animation
is answered 30–64% later than with the old engine in the same build, and
54–91% later than `main`. On this desktop that is +40–47 ms on a
`resize` vs `nopeeks` and +54–76 ms vs `main`, with 7 windows, some slow
to reposition. 14–30 ms of the latter is the tryout build even with
peeks off. With fewer or faster windows the stall and the delay should
shrink. On the 60 Hz laptop a 75–95 ms stall is ~5 vblanks rather than
~15, but a slower CPU lengthens it, and the start delay is wall-clock
time either way.

Suggested next checks:

1. Run `perf_bench --dwm` (elevated) on `resize` for `nopeeks` vs
   `default`. It would show whether the 74–95 ms stall is visible on
   screen or falls at the hand-over after the motion.
2. Profile why `nopeeks` answers commands 17–75% slower than `main`.
   It isn't peek drawing, and it is the cheapest part of the start
   delay to win back.
