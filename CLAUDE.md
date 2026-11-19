<project_overview>
GlazeWM is a window manager for macOS and Windows, written in Rust.

Crate structure:

- **wm** (bin): Main application, which implements the core window management logic. Install path on Windows: `C:\Program Files\glzr.io\glazewm.exe`
- **wm-cli** (bin, lib): CLI for interacting with the main application. Added to `$PATH` by default. Install path on Windows: `C:\Program Files\glzr.io\cli\glazewm.exe`
- **wm-common** (lib): Shared types, utilities, and constants used across other crates.
- **wm-ipc-client** (lib): WebSocket client library for IPC with the main application.
- **wm-platform** (lib): Wrappers over platform-specific APIs; other crates do not call Windows/macOS APIs directly.
- **wm-watcher** (Windows-only) (bin): Watchdog process ensuring proper cleanup when the main application exits. Install path on Windows: `C:\Program Files\glzr.io\glazewm-watcher.exe`

</project_overview>

<output_guidelines>

- Be extremely concise. Sacrifice grammar for the sake of conciseness.
- Do not leave partial or simplified implementations.
- The required quality standard is high. Low quality code will be rejected.
- Do not proceed with solutions that are hacky. Solutions must be robust, maintainable, and extendable. Ask guiding questions if uncertain about a solution.

</output_guidelines>

<code_style_guidelines>

- Avoid `.unwrap()` wherever possible.
- For error handling:
  - Use `crate::Error` and `crate::Result` within the `wm-platform` crate.
  - Use `anyhow` in all other crates.
- For logging, use `tracing` macros (e.g. `tracing::info!("...")`).

</code_style_guidelines>

<code_comment_guidelines>

Keep comments proportionate. Most doc comments should be a line or two; diffs that are mostly
comments are hard to review and make a small change look larger than it is.

The exception is real, hard-won context: a subtle invariant, a platform API quirk, or the
reason a non-obvious approach was chosen. That is worth more lines than the function itself,
because it cannot be recovered from the code. Length is only a problem when it restates what
the code already says.

- Document what is not obvious from the signature: intent, caveats, invariants, why. Never
  restate the code or paraphrase the function name.
- One-line summary for most functions. Only add more when there is a real caveat.
- No section headers, examples, or platform notes unless they earn their place. They are the
  exception, not the default.
- No comments on trivial or self-explanatory code, no block banners, no changelog or
  narration comments ("now we do X", "fixed to handle Y").
- Use punctuation mark at the end of all comments.
- If using unsafe features, include a "SAFETY: ..." comment.
- Wrap type names in backticks (e.g. `NativeMonitor`).

Full structure, for the rare item that needs it (every part after the summary is optional and
usually omitted):

```rs
/// <Concise summary of the function or type>
///
/// (optional) <Notable caveats for usage (kept brief)>
///
/// (optional) <Describe return value if ambiguous (e.g. "Returns a vector of `NativeMonitor`, sorted by their position from left-to-right.")>
///
/// (optional) # Example usage
///
/// <Code block with example usage>
///
/// (optional) # Platform-specific
///
/// <Bullet-point list of behavioral differences on macOS vs Windows>
pub fn my_function() { ... }
```

</code_comment_guidelines>

<test_guidelines>

- Use `#[cfg(test)]` for test modules.
- Write unit tests for core functionality.

</test_guidelines>


<endpoint_safety_guidelines>

Development machines may run endpoint security (e.g. Microsoft Defender), and bursty
automation is easily mistaken for malware. Keep verification manual and low-volume.

- Screenshots: at most one or two per verification step. Never loop, poll, or burst-capture
  the screen. Prefer logs, IPC state queries, and tests over visual capture.
- Windows: do not spawn swarms of test/dummy windows. A couple of real windows is enough to
  verify tiling behaviour.
- No rapid automated input synthesis (simulated keystrokes/clicks in a loop), no repeated
  process spawn/kill cycles, no bulk enumeration of other processes' windows beyond what the
  WM itself needs.
- Anything else that could read as screen-scraping, keylogging, injection, or persistence is
  out of scope. If a check seems to need it, stop and ask first.

</endpoint_safety_guidelines>

<build_targets>

Two kinds of build, by machine:

- **Tryout build** (`tryout/<name>` branch): desktop PC only. May carry benchmarks, profiling
  runs, and automation.
- **Test build** (`test/<name>` branch): work laptop, which runs Defender. Always cut fresh
  from `tryout/<name>`, with everything listed below stripped in one commit. Never develop on a
  test branch: change the tryout branch, then cut the test branch again.

Always stripped from a test build:

- `packages/wm-cli/examples/perf_bench/`, plus `wm-cli`'s `[dev-dependencies]` and
  `[target.'cfg(windows)'.dev-dependencies]`, which only it uses. It drives bursty IPC
  workspace switching and, with `--dwm`, enumerates processes, loads the `PresentMon` DLL at
  runtime and attaches to `dwm.exe`. Likely why a build got quarantined as `Bearfoos` malware.
- `packages/wm-platform/examples/uncloak_all.rs`: walks and un-cloaks every top-level window.
- `resources/scripts/package.ps1`: the installer packaging script, which only the release
  workflow runs. Not needed to build or run the WM.
- Any benchmark, stress scenario, or automation script added later. Add it to this list in the
  same change that adds it.

Removing them from the tree (not just not running them) matters: `cargo test` builds every
example into an `.exe` under `target/`. Never set `GLAZEWM_PERF` on the laptop; the profiler
compiled into `wm` stays inert without it.

Cutting a test build:

```bash
git checkout -B test/<name> origin/tryout/<name>
git rm -r packages/wm-cli/examples/perf_bench packages/wm-platform/examples/uncloak_all.rs \
  resources/scripts/package.ps1
# Remove `wm-cli`'s two dev-dependency tables, then let `cargo build` update `Cargo.lock`.
cargo build --release && cargo test --workspace
git commit -am "chore: strip benchmarks and automation for test build"
git push --force-with-lease -u origin test/<name>
```

</build_targets>
