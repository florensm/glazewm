# Stack Layout – Feature Todo

## Live thumbnails in the tab bar

Each tab in the stack tab bar should show a scaled-down live DWM thumbnail of
its window, similar to the Windows taskbar hover preview.

**Approach:**
- Register a `DwmRegisterThumbnail` for each tab's source window.
- Set `rcDestination` to the tab's available icon area (e.g. 40×30 px).
- Update properties on every `WM_UPDATE_TABS` paint so thumbnails track window
  content changes in real time.
- Unregister on tab removal and re-register on tab insertion.

**Considerations:**
- DWM thumbnails are per-registration, not cached — no extra memory cost beyond
  the thumbnail handle itself.
- Elevated/UWP windows may reject thumbnail registration; fall back to the
  existing icon in that case.
- The tab bar already batches `WM_UPDATE_TABS` posts (deduplication added in
  `perf: deduplicate tab bar WM_UPDATE_TABS posts during animation frames`), so
  thumbnail redraws are automatically rate-limited to the animation tick rate.
- Related infrastructure: `NativeSurrogate` in `wm-platform` shows the full
  DWM thumbnail pipeline — reuse `DwmRegisterThumbnail` /
  `DwmUpdateThumbnailProperties` directly from `native_stack_tab_bar.rs`.
