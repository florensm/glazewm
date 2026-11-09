# Prompt: smoothness and laptop performance pass

To use: start a new Claude Code session on this repo with Claude Fable 5.2 at
effort `high` (`xhigh` if you want it more thorough, at a much higher token cost),
and paste everything below the line.

---

I use Hyprland at home and want GlazeWM to feel as close to it as possible: every
move, resize, scroll, workspace switch and overview zoom runs at the display's full
frame rate, starts the instant I press the key, and never hitches, blinks or snaps.
This fork is already close. Find what still stands between this code and that, fix
what you can show is better, and leave the rest as a ranked plan with evidence.

## My two machines

- **Desktop**: RTX 3070. Runs `tryout/*` builds. Runs pretty well, could be smoother.
- **Work laptop**: hybrid graphics (an integrated GPU plus an RTX 3050 Ti Laptop
  GPU). System RAM sits around 90% used most of the time. Microsoft Defender is on.
  Runs `test/*` builds only.

The main goal is the laptop. If it felt like the desktop does now, I'd be happy.

What that means for the code:

- An integrated GPU has no memory of its own. Every texture, composition surface,
  thumbnail and swap chain the WM allocates comes out of RAM that is already at
  90%. So memory is a frame-time problem here, not only a footprint one: under
  pressure Windows trims working sets, and an animation that touches trimmed pages
  or allocates a surface as it starts will hitch.
- Integrated GPUs are short on fill rate and memory bandwidth. What costs is the
  pixels the compositor blends each frame: monitor-sized layers, blur, overlapping
  thumbnails, anything drawn twice or drawn at zero opacity.
- On hybrid laptops the internal panel is usually driven by the integrated GPU and
  external ports often by the discrete one. A D3D device on a different adapter
  from the one composing the output costs a cross-adapter copy per frame, and a
  device on the discrete GPU keeps it powered. Windows' per-app graphics preference
  changes which adapter a `D3D11CreateDevice` call with no adapter returns.
- Laptop panels may run at 120-165 Hz with dynamic refresh rate, drop to 60 Hz on
  battery, and sit next to a 60 Hz external monitor.

## Read first

`CLAUDE.md` (rules, code style, endpoint safety, build targets), `TODO.md` (state,
what is not checked live yet, the checks you can run off Windows),
`STAND_INS_PLAN.md`, and `packages/wm-platform/src/perf.rs` (the opt-in frame
profiler). The context in those files is hard-won. Where your findings contradict
it, say so with evidence instead of working around it.

## Leads

Where I'd look first. None is a known bug: drop any that reading or measurement
disproves, and add your own.

1. **WM-thread cost per frame.** Everything runs on one thread (the loop in
   `wm/src/main.rs`, `update_animations` in `animation/manager.rs`,
   `commands/general/platform_sync.rs`). Per-frame allocation, Win32 and DWM
   queries repeated per window per frame, work for windows that aren't moving, and
   the synchronous `wm_platform::dwm_flush()` calls, each of which can block for a
   frame.
2. **Frame pacing.** `DxgiVsyncWaiter` (`wm-platform/src/lib.rs`) and
   `predictive_vsync_now` (`manager.rs`): what happens when the refresh period goes
   up (the estimate is only refined downward), on mixed refresh rates, when the
   pacing monitor changes mid-animation, and when the output belongs to the other
   adapter.
3. **Compositor load.** Per moving window: its thumbnail, ring copy, backdrop crop,
   vignette, and both focus styles' copies in peeks (`overlay_copies.rs`,
   `backdrop_source.rs`, `window_peek.rs`, `workspace_surrogate.rs`). Blur and
   glow. Particle windows (`native_particles.rs`). Overview card pictures as wide
   as their strip. Which are composited while invisible, which are larger than
   what shows.
4. **Adapter choice.** `platform_impl/windows/graphics_device.rs` creates its
   device with no adapter. Compare that adapter with the one driving each output.
   Log the choice once at startup.
5. **Memory.** What the WM holds at idle and why: warm surrogates (kept 8 s),
   wallpaper and blurred backdrop surfaces per monitor and style
   (`wallpaper_surface.rs`), parallax images, overview pictures, unbounded caches.
   Leaks over hours of use: handles, GDI objects, composition visuals, thumbnails
   not unregistered on every path. What can be released at idle or when Windows
   signals low memory.
6. **Input-to-motion latency.** From key press to the first frame that moves:
   `catch_up_before_frame`, the biased `select!`, setup done before the first tick.
7. **Idle cost.** Timers that wake the thread when nothing moves (the 5 s cleanup,
   `SENT_MESSAGE_INTERVAL`, the overview's breathing glow). Hyprland's `misc:vfr`
   is the bar: nothing renders when nothing changes.
8. **Feel next to Hyprland.** A move retargeted mid-way keeps its current position
   and velocity (no restart, no kink). Move, resize, scroll, switch and overview
   use the same curve family. Nothing snaps at the start or end.

If measurement shows some effects are more than an integrated GPU can afford, a
config option that drops them on battery or battery saver
(`RegisterPowerSettingNotification`) is in scope: off by default, documented in the
sample config.

## How to measure

- In a cloud session you can't run the WM. Build, lint and test against the
  Windows target as `TODO.md` "Checks" describes. Where a win can be shown in a
  unit test (fewer allocations or calls, a bounded cache, a pacing calculation),
  show it there.
- Count what doesn't depend on hardware: thumbnails and windows per scenario,
  pixels composited per frame, bytes held at idle and at peak, Win32 calls per
  frame. Before/after tables of these are the evidence the laptop can't give.
- On the desktop I can run the profiler (`GLAZEWM_PERF=1`) and `perf_bench` (its
  doc comment has the commands). Give me the exact commands for the before and
  after runs you want; I run them, not you.
- On the laptop I only have what I can see: Task Manager (Details tab, memory and
  GPU memory columns), smoothness by eye, and `errors.log`. Never the profiler or
  `perf_bench` there.

## Hard limits

A laptop build was already quarantined by Defender as malware (`Bearfoos`, see
`CLAUDE.md`), and the laptop belongs to my employer. Follow `CLAUDE.md`'s
endpoint-safety and build-targets sections exactly. On top of that:

- The shipped crates (`wm`, `wm-platform`, `wm-common`, `wm-ipc-client`, `wm-cli`,
  `wm-watcher`) must not gain any of: enumerating or opening other processes,
  reading their memory, ETW sessions, loading non-system DLLs, hooks beyond the
  WinEvent and low-level hooks that exist, screen capture beyond what exists,
  synthesized input, new registry writes (so skip the `TODO.md` item that replaces
  `auto-launch` with a direct `Run` key write), network beyond the local IPC
  socket, spawning shells or scripts, or writing executable files. If the right
  fix needs one of these, don't write it: put it in the review as a proposal, with
  the risk.
- A new benchmark, stress scenario or automation script goes under an `examples/`
  directory and onto `CLAUDE.md`'s strip list in the same commit. Prefer extending
  `perf_bench` to adding a tool.
- No new dependency in a shipped crate unless the win is large; name it in the
  report.
- Don't start `STAND_INS_PLAN.md` step 1. It waits for my baseline recordings.

Approved, no need to ask: reading anything, building, linting, running and adding
tests, committing to your tryout branch and pushing it, updating `TODO.md`,
`CLAUDE.md`'s strip list and the review file, and cutting the test branch at the
end as described below.

## What to change

Fix what you can show is better and is safe to ship before a live check, or whose
live check is quick and clear. One commit per fix, with a prefix like the history
(`perf(scroll): ...`), so I can bisect a regression on the desktop. Restructure
code where that is the right fix: this fork is mine to improve and you'll keep
building on it. But what I see on screen and what config keys mean stay the same
unless the finding is about changing them; list every visible change in your
report. Code style, comments and errors per `CLAUDE.md`. No partial
implementations: a fix you can't finish goes in the review instead.

Leave for the plan anything that needs a design decision from me, a live A/B to
judge, or one of the hard limits.

Use subagents only for broad read-only sweeps of the code: each starts with no
context and has to re-read what it needs.

## Branches

Work on `tryout/smooth`, cut from `origin/tryout/particles` (if the session assigns
you a `tryout/` branch, use that name instead). Before you finish, clippy with
`-D warnings` and the tests from `TODO.md` "Checks" must be clean; then push. Then
cut `test/smooth` from it with `CLAUDE.md`'s recipe, strip commit included, and
push it. That push is approved. If you couldn't build for the Windows target,
don't cut the test branch, and say why.

## Deliverables

1. `SMOOTHNESS_REVIEW.md` at the repo root. Findings ranked by expected effect on
   the laptop. For each: what and where (`file:line`), evidence (counts,
   measurements, reasoning), expected win (ms per frame, MB, latency), risk, and
   status: fixed in `<sha>`, proposed, or ruled out. Keep ruled-out ones to a line;
   they save the next session time.
2. `TODO.md` brought up to date: what's done, and each new change under "Not
   checked live yet" with exact steps for the desktop and for the laptop, written
   like the existing entries.
3. A final message: the commits, the top items left, and what I should check
   first on the desktop, then on the laptop.

Before you start, say in a line what you're about to do. Brief updates while you
work help me follow along.

# Finishing work

Please avoid stopping while you still owe work I asked for. Be persistent and do
not stop unless needed. While status notes are welcome, as is clarifying with me
on open decisions, carry on with work that does not depend on my answer without
unnecessary stops. If you notice yourself inviting me to redirect you or offering
to wait, instead proceed on the next part of the task. You may meet ordinary
obstacles like errors, timeouts, locked files, empty results or failing tools.
Diagnose them first, and when they are not real blockers, work through them with
the access you have (wait and retry, fix the request, use another tool or source)
rather than stopping unnecessarily. If something is clearly a hard blocker, such
as the hard limits above, access that was intentionally withheld, or a safety
guardrail, leave it alone, say plainly what you found, and look for another safe
way to finish the task. Stopping before the task is complete is very rarely
merited. This does not override the need for confirmation on risky or destructive
actions.

# Communicating clearly

Write for an intelligent reader who has not seen your tool calls, your reasoning or
the files you opened. Put the key points first. Use plain words and the same word
for the same thing. Don't use terms you made up during the work; say what the
thing is. Keep it concise, as `CLAUDE.md` asks, but self-contained: the reader
should not need to dig for context to understand a line.
