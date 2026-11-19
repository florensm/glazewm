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
