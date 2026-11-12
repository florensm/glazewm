You're on my desktop PC in the GlazeWM repo (C:\Users\googlers\glazewm), branch `tryout/stand-ins`. Read CLAUDE.md first and follow it. This is the tryout machine: benchmarks are OK, but no input synthesis, no screenshot loops, no dummy windows.

Context: last night's bench compared the new peek drawing against the old engine and `origin/main`. Read `C:\Users\googlers\.glzr\glazewm\bench-2026-11-12\bench-report.md` (start with "Bottom line" and "Verdict") and the README next to it. In short:

- Peeks (`default`) are much smoother on `resize`, and so is `morepeeks` on `float`: frame p90 roughly halved, interval p90 ~9–11 → 6–7 ms. We keep them.
- But peek modes answer the command that starts an animation 30–64% later than `nopeeks` and 54–91% later than `main`. On `resize` with 7 windows that is 130–160 vs 90–114 vs 76–85 ms.
- Each peek-mode `resize` burst has one 74–95 ms stall. In that frame the real windows are repositioned synchronously while peeks are up (`peek_frame` > `platform_sync` > `rd_apply` > `rp_swp`/`rp_visible`, then `uncloak`), blocked on slow apps: Explorer, Windows Terminal, the Settings/Store frames, Everything, ~10–14 ms each.
- Even with peeks off, the tryout build (`nopeeks`) answers commands 17–75% slower than `main` on move/float/resize/switch, while being as smooth.

Goal for today: make the peek path start as fast as the old one without losing its smoothness. Work in this order and measure each change. Don't guess.

1. Find why `nopeeks` answers slower than `origin/main`. Compare where time goes before the IPC answer: `platform_sync`, relayout, effects/overlays, `focus_change`, cloak, event queue waits. Use `perf.log` (`logs/` in the bench folder has both builds' reports). Bisect if profiling doesn't show it. Report the cause and a fix proposal before changing code.
2. Find what the peek path does before the IPC answer that the old path doesn't (peek setup, thumbnails, synchronous repositions). Can it move after the first frame?
3. Shorten or hide the hand-over stall. Options to weigh: reposition the real windows while the peeks still cover them, but early or spread over frames; don't wait on slow apps' message pumps (async reposition for windows behind peeks); keep peeks up until the real windows have landed. Explain the trade-offs and get my OK on an approach first.
4. Make `input->first frame` measurable on the peek path. Today only the old path calls `perf::note_animation_start` (`animation/manager.rs`), so peek modes show n/a and we rely on command p50.
5. Bench tooling: give `isolate_modes.ps1`/perf_bench the fixed tile order. A WM restart flips the tile order, so with an even number of modes each mode is locked to one order. See `bench-run.ps1 -TileOrder` in the bench folder. Also have perf_bench report interval p99/max.

Measure with the bench folder's `bench-run.ps1` and `-TileOrder WindowsTerminal,Notepad,chrome,Everything,explorer,ApplicationFrameHost,ApplicationFrameHost`, modes `nopeeks,default` (add `morepeeks` for float), 4 rounds, with and without `-CpuLoad 8`. Run a short probe before any long bench, and don't TaskStop a running bench (it kills the WM). Compare against last night's `results/fixed-*.txt` with `summarize.py`/`tails.py`.

Keep `morepeeks` behind its flag until 2 and 3 are done. Don't commit or push without asking. Call out any new Windows API in a change meant for the laptop test build (Defender).
