You're working on my GlazeWM fork (github.com/florensm/glazewm), a Windows tiling WM in Rust. Work on branch `tryout/start-latency`: `tryout/stand-ins` (`65a60de3`) plus one commit with last night's bench in `packages/wm-cli/examples/perf_bench/results/2026-11-12/`. Read `CLAUDE.md` first and follow it, then `STAND_INS_PLAN.md`, `ANIMATION_REVIEW.md` and `TODO.md` for how the peek ("stand-in") drawing came about. Then read the bench folder's `README.md`, `bench-report.md` ("Bottom line", "Reading the numbers", "Verdict") and `perf-excerpts.txt` (full profiler reports of one `resize` burst per build).

You're in a Linux cloud container. You can read, plan, write code, compile-check and push, but you can't run the WM or measure anything. I'll measure on my Windows desktop afterwards. So every behaviour change must be switchable at runtime, so I can A/B it there. Use the existing `GLAZEWM_ISOLATE` mechanism (`packages/wm-platform/src/isolation.rs`): one new word per change, off by default.

## What last night's bench found

Desktop: Ryzen 7 3700X, RTX 3070, 3440×1440 @ 174 Hz. 7 tiled windows, some slow to reposition (Explorer, Windows Terminal, Settings and Store `ApplicationFrameHost` frames, Everything, Notepad). Frame profiler on (`GLAZEWM_PERF=1`). `perf_bench` scenarios move/float/minimize/resize/switch, 10 bursts each, all four builds interleaved with a fixed tile order, 4 rounds, at load 0 and with 8 busy CPU threads. Modes: `main` = origin/main; `nopeeks` = old engine in this build; `default` = peek drawing; `morepeeks` = plus floating/fullscreen/live/cross-monitor on peeks.

- **Peeks are smoother; we keep them.** On `resize` (and `float` under `morepeeks`), frame p90 roughly halves (7–12 → 4–5.5 ms) and interval p90 drops from 8–11 to 6–7 ms (one vblank is 5.75 ms).
- **Peeks start later.** The IPC answer to the command that starts an animation (≈ input→first frame) comes 30–64% later than `nopeeks` and 54–91% later than `main`. `resize`: 130–160 ms vs 90–114 vs 76–85. `move`: 24–26 vs 18–19 vs 16–17. `float` under `morepeeks`: 138/163 vs 106/100 vs 74/91. Per-frame `tick` +15–55%, plus `peek_frame` 3.1–3.8 ms.
- **One long stall per peek-mode `resize` burst: 74–95 ms**, against several 40–65 ms gaps on the old engine. Its frame, from `perf.log`:
  - `tick` worst 88 ms; `peek_frame` worst 59 ms.
  - Inside it: `platform_sync` 52 ms > `redraw` > `redraw_loop` > `rd_apply` 42 ms.
  - Below that: `rp_swp` 28 ms, `rp_visible` 11 ms, `uncloak`.
  - Synchronous repositions per process: Explorer 13.8 ms, Windows Terminal 12.7, `ApplicationFrameHost` 10.4, Everything 10.0, chrome 4.0. Other windows already go async.
  - Likely the hand-over from peeks to the real windows at the end of the motion, but unconfirmed.
- **The build itself is slower even with peeks off.** `nopeeks` answers 17–75% slower than `main` on move/resize/switch (and float at load 0), with equal or better smoothness. `switch`: 54–65 ms vs 37–39.
- **`input->first frame` is not recorded on the peek path.** Only the old path calls `perf::note_animation_start` (`packages/wm/src/animation/manager.rs`), so peek modes can only be judged by command time.
- **Bench pitfall.** A WM restart flips the tile order between two states, so with an even number of modes each mode always gets the same order. I fixed it with a script that moves tiles into a fixed order over IPC after each restart (`bench-run.ps1 -TileOrder` in the bench folder). `perf_bench` itself doesn't do this, and it only reports interval p50/p90, not p99/max.

## Goal

Make the peek path start as fast as the old engine (ideally as fast as `main`) without losing its smoothness, and make the next bench able to prove it.

## How to work

1. **Plan first, with Fable (you).** Read the code paths involved:
   - The command → `platform_sync` → IPC answer path, old and peek.
   - `PeekView`, and the hand-over to the real windows.
   - What `origin/main..tryout/stand-ins` added to the pre-answer path. 431 commits, so diff the relevant files rather than reading the log.

   Then write `START_LATENCY_PLAN.md` at the repo root, in the style of `STAND_INS_PLAN.md`. Split the work into independent workstreams with **disjoint file ownership** where possible; where two touch the same file, say which goes first. Suggested workstreams; change them if the code says otherwise:
   - **A. Instrumentation.**
     - Record input→first frame on the peek path.
     - Add profiler stages that split the pre-answer time.
     - Give `perf_bench` interval p99/max columns, and give `isolate_modes.ps1` a fixed tile order.
     - `perf_bench` is already in CLAUDE.md's strip list. Any new script goes on that list in the same commit.
   - **B. Why `nopeeks` answers slower than `main`.**
     - Ranked hypotheses, each with the code it points at.
     - Fixes behind `GLAZEWM_ISOLATE` words where they change behaviour.
   - **C. Peek-path work done before the answer** (peek setup, thumbnails, synchronous repositions) that could move after the first frame. Behind a word.
   - **D. The hand-over stall.** Options include:
     - Reposition the real windows while the peeks still cover them, earlier or spread over frames.
     - Don't block on slow apps' message pumps.
     - Keep the peeks up until the real windows have landed.

     Give the trade-offs, and put each approach behind its own word.

   For each workstream, give: files, approach, risks, how to compile-check it here, and exactly what I should measure on the desktop (scenarios, modes, `GLAZEWM_ISOLATE` words, expected effect on command p50, interval p90, frame p90 and the longest gap). **Then stop and show me the plan. Don't start coding until I say go.**

2. **Execute with Opus agents, after my go.**
   - Build on `tryout/start-latency` (it exists, with only the bench commit on top of `tryout/stand-ins`).
   - Spawn one subagent per workstream with the Agent tool (`model: "opus"`, `isolation: "worktree"`). Run them in parallel where files don't overlap, in sequence where they do.
   - Each agent gets its workstream section of the plan plus `CLAUDE.md`, works and commits on its own branch, and reports what it changed and what it couldn't verify.
   - You review each result against the plan and CLAUDE.md's code and comment rules, merge the branches into `tryout/start-latency`, and resolve conflicts.

3. **Check, then push.**
   - Run `rustup target add x86_64-pc-windows-msvc`, then `cargo check --workspace --target x86_64-pc-windows-msvc` and clippy for that target.
   - Run `cargo test` for whatever builds on Linux.
   - If the Windows resource build step (`tauri-winres`) blocks the cross-check, try installing `llvm` for `llvm-rc`. If that fails, report it. Don't edit build scripts to get around it.
   - Push only `tryout/start-latency`. No `test/` branch, no other branch, nothing to `main`.
   - Finish with:
     - A summary of each change and its `GLAZEWM_ISOLATE` word.
     - Any new Windows API calls, which CLAUDE.md requires for Defender.
     - What is unverified.
     - The exact bench commands for me to run on the desktop.

Rules:
- No `.unwrap()`; use `anyhow` outside `wm-platform`, `crate::Result` inside it; `tracing` for logs.
- No `SetProcessInformation` (Defender).
- Don't change default behaviour. Everything new is off unless its `GLAZEWM_ISOLATE` word is set.
