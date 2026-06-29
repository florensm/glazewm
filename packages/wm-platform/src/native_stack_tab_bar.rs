use std::{
  sync::OnceLock,
  time::{Duration, Instant},
};

use windows::{
  core::w,
  Win32::{
    Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{
      BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW,
      EndPaint, FillRect, GetStockObject, HBRUSH, InvalidateRect,
      NULL_PEN, RoundRect, SelectObject, SetBkMode, SetTextColor,
      CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_QUALITY, DT_END_ELLIPSIS,
      DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, FW_NORMAL, OUT_DEFAULT_PRECIS,
      PAINTSTRUCT, TRANSPARENT,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, DrawIconEx,
      GetClassLongPtrW, GetSystemMetrics, GetWindowLongPtrW, KillTimer,
      LoadCursorW, PostMessageW, RegisterClassW, SetTimer, SetWindowLongPtrW,
      SetWindowPos, ShowWindow, CREATESTRUCTW, DI_NORMAL, GCLP_HICONSM,
      GWLP_USERDATA, HICON, IDC_ARROW, SM_CXSMICON, SW_HIDE, SWP_NOACTIVATE,
      SWP_SHOWWINDOW, WM_APP, WM_CLOSE, WM_CREATE, WM_DESTROY, WM_ERASEBKGND,
      WM_LBUTTONDOWN, WM_PAINT, WM_TIMER, WNDCLASSW, WS_EX_NOACTIVATE,
      WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
    },
  },
};

use crate::{Color, Dispatcher, Rect};

/// Custom message used to update tab state from the tokio thread.
///
/// `WPARAM` carries a heap-allocated `Box<TabUpdate>` that must be
/// recovered with `Box::from_raw`. `LPARAM` is unused.
const WM_UPDATE_TABS: u32 = WM_APP + 1;

/// Timer ID for the sliding active-indicator animation.
const INDICATOR_TIMER_ID: usize = 1;

/// Duration of the active-indicator slide animation.
const INDICATOR_ANIM_DURATION: Duration = Duration::from_millis(180);

static TAB_BAR_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// Information about a single tab in the tab bar.
pub struct TabInfo {
  /// Display title for the tab.
  pub title: String,

  /// Handle (`HWND`) of the managed window, used to fetch its icon.
  pub hwnd: isize,
}

/// Color and style scheme for the stack tab bar.
#[derive(Clone, PartialEq)]
pub struct TabBarColors {
  pub background: Color,
  pub active: Color,
  pub inactive: Color,
  pub text: Color,
  /// Color of the active-tab slide indicator bar.
  pub indicator: Color,
  /// Height of the indicator bar in pixels (0 = disabled).
  pub indicator_height: i32,
  /// Corner radius of tab backgrounds in pixels (0 = square).
  pub border_radius: i32,
  /// Width of separator lines between tabs in pixels (0 = disabled).
  pub separator_width: i32,
  /// Color of separator lines between tabs.
  pub separator: Color,
}

/// Payload sent via `WM_UPDATE_TABS` to update the tab bar from any thread.
struct TabUpdate {
  tabs: Vec<TabInfo>,
  active_index: usize,
  rect: Rect,
  colors: TabBarColors,
}

/// Per-window state stored in `GWLP_USERDATA`.
struct TabBarState {
  tabs: Vec<TabInfo>,
  active_index: usize,
  rect: Rect,
  colors: TabBarColors,
  on_click: Box<dyn Fn(usize) + Send + 'static>,
  /// Current x position of the indicator bar, in pixels relative to the
  /// tab bar client area. Interpolated between tabs during animation.
  indicator_cur_x: f32,
  /// Target x position of the indicator bar after the active tab changes.
  indicator_target_x: f32,
  /// Starting x position at the beginning of the current animation.
  indicator_from_x: f32,
  /// Wall-clock instant when the current indicator animation began.
  indicator_anim_start: Option<Instant>,
}

/// A GDI-painted tab bar overlay for a `StackContainer`.
///
/// The window is a `WS_POPUP | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE` overlay
/// created on the event-loop thread. Tab state is updated from the tokio
/// thread via `PostMessageW(WM_UPDATE_TABS)` (fire-and-forget). Click events
/// are routed back through the `on_click` closure, which is expected to send
/// a message on a tokio channel.
///
/// `update` deduplicates posts by caching the last-applied rect, active
/// index, tab keys, and colors. During animations `platform_sync` runs every
/// frame, but the tab bar content does not change, so redundant GDI repaints
/// are skipped until something actually differs.
///
/// The active-tab indicator slides smoothly between tabs using a `WM_TIMER`
/// driven animation internal to the WNDPROC.
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeStackTabBar {
  hwnd: isize,
  /// Rect from the last posted `WM_UPDATE_TABS`, or `None` after `hide()`.
  ///
  /// Reset to `None` by `hide()` so the next `update()` always re-posts even
  /// when rect and tab state are unchanged (needed to re-show the window after
  /// a workspace-switch hide).
  last_rect: Option<Rect>,
  /// Active-tab index from the last post.
  last_active_index: usize,
  /// Per-tab keys from the last post: `(hwnd, title)`.
  last_tab_keys: Vec<(isize, String)>,
  /// Colors from the last post, used to detect config-reload changes.
  last_colors: Option<TabBarColors>,
}

// SAFETY: `hwnd` is a valid Win32 window handle that can be passed between
// threads. All WNDPROC processing happens on the event-loop thread. We only
// store the raw handle value here so that we can post messages to it.
unsafe impl Send for NativeStackTabBar {}

impl NativeStackTabBar {
  /// Creates a new tab bar window positioned at the top of `rect`.
  ///
  /// The window is created synchronously on the event-loop thread via
  /// `dispatcher.dispatch_sync()`. The `on_click` closure is called with
  /// the zero-based tab index whenever the user clicks a tab.
  pub fn create(
    dispatcher: &Dispatcher,
    rect: &Rect,
    tabs: Vec<TabInfo>,
    active_index: usize,
    colors: TabBarColors,
    on_click: Box<dyn Fn(usize) + Send + 'static>,
  ) -> crate::Result<Self> {
    // Snapshot the creation state for the dedup cache so the first `update`
    // call with identical content does not post a redundant `WM_UPDATE_TABS`.
    let initial_rect = rect.clone();
    let initial_tab_keys: Vec<(isize, String)> =
      tabs.iter().map(|t| (t.hwnd, t.title.clone())).collect();
    let initial_colors = colors.clone();

    let tab_width = if !tabs.is_empty() {
      rect.width() / tabs.len() as i32
    } else {
      0
    };
    let initial_indicator_x =
      active_index as f32 * tab_width as f32;

    let state = Box::new(TabBarState {
      tabs,
      active_index,
      rect: rect.clone(),
      colors,
      on_click,
      indicator_cur_x: initial_indicator_x,
      indicator_target_x: initial_indicator_x,
      indicator_from_x: initial_indicator_x,
      indicator_anim_start: None,
    });

    // Transmit the pointer as a plain `usize` so the closure is `Send`.
    let state_ptr_val = Box::into_raw(state) as usize;
    let rect = rect.clone();

    let hwnd = dispatcher.dispatch_sync(move || -> crate::Result<isize> {
      ensure_class_registered();

      let state_ptr = state_ptr_val as *mut TabBarState;

      // SAFETY: Creating a valid top-level popup window with known-good
      // parameters. `state_ptr` is valid for the duration of the window's
      // lifetime and is freed in `WM_DESTROY`.
      let handle = unsafe {
        CreateWindowExW(
          WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
          w!("GlazeWM_TabBar"),
          w!(""),
          WS_POPUP | WS_VISIBLE,
          rect.left,
          rect.top,
          rect.width(),
          rect.height(),
          None,
          None,
          GetModuleHandleW(None).unwrap_or_default(),
          Some(state_ptr.cast()),
        )
      };

      if handle.0 == 0 {
        // Window creation failed — free state to avoid a leak.
        // SAFETY: We just allocated this pointer and creation failed, so
        // no other code has taken ownership.
        unsafe { drop(Box::from_raw(state_ptr)) };
        return Err(crate::Error::Platform(
          "Failed to create tab bar window.".to_string(),
        ));
      }

      Ok(handle.0)
    })??;

    Ok(Self {
      hwnd,
      last_rect: Some(initial_rect),
      last_active_index: active_index,
      last_tab_keys: initial_tab_keys,
      last_colors: Some(initial_colors),
    })
  }

  /// Posts a tab-state update to the tab bar window.
  ///
  /// Uses `PostMessageW` (fire-and-forget) to avoid blocking the tokio
  /// thread, which could deadlock if the Win32 event-loop thread is itself
  /// waiting on a `SetWindowPos` for a managed application window.
  ///
  /// No-op when `rect`, `active_index`, `colors`, and the `(hwnd, title)`
  /// pairs of `tabs` are all identical to the last post. This prevents
  /// redundant GDI repaints during animation ticks where `platform_sync`
  /// runs every frame but the tab bar content has not changed.
  pub fn update(
    &mut self,
    rect: &Rect,
    tabs: Vec<TabInfo>,
    active_index: usize,
    colors: TabBarColors,
  ) {
    let new_tab_keys: Vec<(isize, String)> =
      tabs.iter().map(|t| (t.hwnd, t.title.clone())).collect();

    if self.last_rect.as_ref() == Some(rect)
      && self.last_active_index == active_index
      && self.last_tab_keys == new_tab_keys
      && self.last_colors.as_ref() == Some(&colors)
    {
      return;
    }

    self.last_rect = Some(rect.clone());
    self.last_active_index = active_index;
    self.last_tab_keys = new_tab_keys;
    self.last_colors = Some(colors.clone());

    let update = Box::new(TabUpdate {
      tabs,
      active_index,
      rect: rect.clone(),
      colors,
    });

    let ptr = Box::into_raw(update) as usize;

    // SAFETY: `self.hwnd` is a valid window handle. `PostMessageW` queues
    // the message without blocking; ownership of the `Box<TabUpdate>`
    // transfers to the WNDPROC, which recovers and frees it in
    // `WM_UPDATE_TABS`. If the post fails the pointer is leaked (window
    // is gone), which is acceptable as the app is shutting down anyway.
    unsafe {
      let _ = PostMessageW(
        HWND(self.hwnd),
        WM_UPDATE_TABS,
        WPARAM(ptr),
        LPARAM(0),
      );
    }
  }

  /// Hides the tab bar window without destroying it.
  ///
  /// Clears the cached rect so the next `update()` call always re-posts,
  /// causing `WM_UPDATE_TABS` to reposition and re-show the window even when
  /// the tab content has not changed since the hide.
  pub fn hide(&mut self) {
    self.last_rect = None;
    // SAFETY: `self.hwnd` is a valid window handle.
    unsafe {
      let _ = ShowWindow(HWND(self.hwnd), SW_HIDE);
    }
  }
}

impl Drop for NativeStackTabBar {
  fn drop(&mut self) {
    // Post WM_CLOSE so the event loop destroys the window and its state.
    // SAFETY: `self.hwnd` is a valid window handle.
    unsafe {
      let _ = PostMessageW(HWND(self.hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
    }
  }
}

fn ensure_class_registered() {
  TAB_BAR_CLASS_REGISTERED.get_or_init(|| {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_TabBar"),
      lpfnWndProc: Some(tab_bar_wnd_proc),
      // No background brush: WM_ERASEBKGND returns 1 to suppress erase,
      // and WM_PAINT draws the entire client area.
      hbrBackground: HBRUSH::default(),
      // SAFETY: IDC_ARROW is a valid system cursor resource.
      hCursor: unsafe {
        LoadCursorW(None, IDC_ARROW).unwrap_or_default()
      },
      ..Default::default()
    };

    // SAFETY: `wnd_class` is a properly initialized `WNDCLASSW` with a
    // static class name and a valid window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
}

/// Returns the pixel x offset of the left edge of the tab at `index`.
fn tab_x(index: usize, tab_width: i32) -> f32 {
  index as f32 * tab_width as f32
}

/// Applies a cubic ease-out to `t` (0.0–1.0).
fn ease_out_cubic(t: f32) -> f32 {
  let t = t.clamp(0.0, 1.0);
  1.0 - (1.0 - t).powi(3)
}

/// Paints the tab bar client area using GDI.
///
/// Draws the background, per-tab colored rectangles with optional rounded
/// corners and separators, process icons fetched from the managed window's
/// class, tab title text, and a sliding active-tab indicator bar.
unsafe fn paint_tab_bar(hwnd: HWND, state: &TabBarState) {
  let mut ps = PAINTSTRUCT::default();
  let hdc = BeginPaint(hwnd, &mut ps);

  let width = state.rect.width();
  let height = state.rect.height();
  let n_tabs = state.tabs.len();

  if n_tabs == 0 {
    EndPaint(hwnd, &ps);
    return;
  }

  // Draw the full bar background.
  let bg_brush = CreateSolidBrush(COLORREF(state.colors.background.to_bgr()));
  let full_rect = RECT {
    left: 0,
    top: 0,
    right: width,
    bottom: height,
  };
  FillRect(hdc, &full_rect, bg_brush);
  DeleteObject(bg_brush);

  SetBkMode(hdc, TRANSPARENT);
  SetTextColor(hdc, COLORREF(state.colors.text.to_bgr()));

  // Create a proportionally sized font that fits within the tab height.
  let font_height = -(height - 6).max(8);
  let font = CreateFontW(
    font_height,
    0,
    0,
    0,
    FW_NORMAL.0 as i32,
    0,
    0,
    0,
    DEFAULT_CHARSET.0 as u32,
    OUT_DEFAULT_PRECIS.0 as u32,
    CLIP_DEFAULT_PRECIS.0 as u32,
    DEFAULT_QUALITY.0 as u32,
    0,
    w!("Segoe UI"),
  );
  let old_font = SelectObject(hdc, font);

  let tab_width = width / n_tabs as i32;
  let border_radius = state.colors.border_radius;
  let sep_w = state.colors.separator_width;
  let ind_h = state.colors.indicator_height;

  // Horizontal inset applied on each side of a tab's background so that
  // rounded corners are visible between adjacent tabs. When border_radius
  // is 0 (flat tabs) the inset collapses to 0 and separator lines are
  // used for visual separation instead.
  let h_inset = if border_radius > 0 { 3 } else { 0 };

  // Pre-fetch a null pen so `RoundRect` has no visible outline.
  // SAFETY: NULL_PEN is a valid stock GDI object identifier.
  let null_pen = GetStockObject(NULL_PEN);

  for (i, tab) in state.tabs.iter().enumerate() {
    // `slot_x`/`slot_w` = the full allocated width for this tab (used for
    // click detection and indicator positioning).
    let slot_x = i as i32 * tab_width;
    let slot_w = if i == n_tabs - 1 {
      width - slot_x
    } else {
      tab_width
    };

    // `bg_x`/`bg_w` = the visible background rect after horizontal inset.
    let bg_x = slot_x + h_inset;
    let bg_w = (slot_w - h_inset * 2).max(0);

    // Draw per-tab background, either rounded or flat.
    let tab_color = if i == state.active_index {
      state.colors.active.to_bgr()
    } else {
      state.colors.inactive.to_bgr()
    };

    if bg_w > 0 {
      let tab_brush = CreateSolidBrush(COLORREF(tab_color));

      if border_radius > 0 {
        let old_pen = SelectObject(hdc, null_pen);
        let old_brush = SelectObject(hdc, tab_brush);
        // `RoundRect` corner ellipse diameter = radius * 2.
        let _ = RoundRect(
          hdc,
          bg_x,
          0,
          bg_x + bg_w,
          height,
          border_radius * 2,
          border_radius * 2,
        );
        SelectObject(hdc, old_pen);
        SelectObject(hdc, old_brush);
      } else {
        let tab_rect = RECT {
          left: bg_x,
          top: 0,
          right: bg_x + bg_w,
          bottom: height,
        };
        FillRect(hdc, &tab_rect, tab_brush);
      }

      DeleteObject(tab_brush);
    }

    // Draw separator on the right edge only for flat tabs (rounded tabs
    // use the inset gap for visual separation instead).
    if sep_w > 0 && border_radius == 0 && i < n_tabs - 1 {
      let sep_brush =
        CreateSolidBrush(COLORREF(state.colors.separator.to_bgr()));
      let sep_rect = RECT {
        left: slot_x + slot_w - sep_w,
        top: 0,
        right: slot_x + slot_w,
        bottom: height,
      };
      FillRect(hdc, &sep_rect, sep_brush);
      DeleteObject(sep_brush);
    }

    // Attempt to load the window's small icon.
    let icon_hwnd = HWND(tab.hwnd);
    // Use the system's small icon size (SM_CXSMICON, typically 16px) so
    // the icon is never upscaled from a smaller source bitmap.
    let icon_size = GetSystemMetrics(SM_CXSMICON).max(8);
    // `GetClassLongPtrW` reads directly from kernel-mode data — no
    // cross-process message required. `SendMessageW(WM_GETICON)` would
    // block the Win32 event loop while the target app processes the
    // message, freezing all window management until it responds.
    let hicon =
      HICON(GetClassLongPtrW(icon_hwnd, GCLP_HICONSM) as isize);

    let text_x = if hicon.0 != 0 {
      let icon_y = (height - icon_size) / 2;
      let _ = DrawIconEx(
        hdc,
        bg_x + 4,
        icon_y,
        hicon,
        icon_size,
        icon_size,
        0,
        None,
        DI_NORMAL,
      );
      bg_x + 4 + icon_size + 4
    } else {
      bg_x + 6
    };

    // Draw tab title text within the inset background bounds.
    let mut title_wide: Vec<u16> = tab.title.encode_utf16().collect();
    let mut text_rect = RECT {
      left: text_x,
      top: 0,
      right: bg_x + bg_w - 4,
      bottom: height,
    };
    DrawTextW(
      hdc,
      &mut title_wide,
      &mut text_rect,
      DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
    );
  }

  // Draw the sliding active-indicator bar across the full slot width so
  // it is visible regardless of the tab background inset.
  if ind_h > 0 {
    let ind_x = state.indicator_cur_x as i32;
    let ind_w = tab_width.min(width - ind_x);
    if ind_w > 0 {
      let ind_brush =
        CreateSolidBrush(COLORREF(state.colors.indicator.to_bgr()));
      let ind_rect = RECT {
        left: ind_x,
        top: height - ind_h,
        right: ind_x + ind_w,
        bottom: height,
      };
      FillRect(hdc, &ind_rect, ind_brush);
      DeleteObject(ind_brush);
    }
  }

  SelectObject(hdc, old_font);
  DeleteObject(font);
  EndPaint(hwnd, &ps);
}

/// Window procedure for the tab bar overlay window.
unsafe extern "system" fn tab_bar_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match msg {
    WM_CREATE => {
      // Store the `TabBarState` pointer passed via `lpCreateParams`.
      let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
      let state_ptr = create_struct.lpCreateParams;
      SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
      LRESULT(0)
    }
    WM_ERASEBKGND => {
      // Suppress default background erase — WM_PAINT covers the full area.
      LRESULT(1)
    }
    WM_PAINT => {
      let state_ptr =
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TabBarState;
      if state_ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
      }
      paint_tab_bar(hwnd, &*state_ptr);
      LRESULT(0)
    }
    WM_TIMER => {
      if wparam.0 != INDICATOR_TIMER_ID {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
      }
      let state_ptr =
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TabBarState;
      if state_ptr.is_null() {
        return LRESULT(0);
      }
      let state = &mut *state_ptr;

      if let Some(start) = state.indicator_anim_start {
        let elapsed = start.elapsed();
        let t = (elapsed.as_secs_f32()
          / INDICATOR_ANIM_DURATION.as_secs_f32())
        .clamp(0.0, 1.0);
        let p = ease_out_cubic(t);

        state.indicator_cur_x = state.indicator_from_x
          + (state.indicator_target_x - state.indicator_from_x) * p;

        if t >= 1.0 {
          state.indicator_cur_x = state.indicator_target_x;
          state.indicator_anim_start = None;
          let _ = KillTimer(hwnd, INDICATOR_TIMER_ID);
        }

        let _ = InvalidateRect(hwnd, None, false);
      } else {
        let _ = KillTimer(hwnd, INDICATOR_TIMER_ID);
      }

      LRESULT(0)
    }
    WM_LBUTTONDOWN => {
      let state_ptr =
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TabBarState;
      if !state_ptr.is_null() {
        let state = &*state_ptr;
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let x = (lparam.0 & 0xFFFF) as i16 as i32;
        let n_tabs = state.tabs.len();
        if n_tabs > 0 {
          let tab_width = state.rect.width() / n_tabs as i32;
          if tab_width > 0 {
            let index =
              ((x / tab_width) as usize).min(n_tabs.saturating_sub(1));
            (state.on_click)(index);
          }
        }
      }
      LRESULT(0)
    }
    WM_UPDATE_TABS => {
      // Recover the `TabUpdate` pointer from WPARAM.
      let update = Box::from_raw(wparam.0 as *mut TabUpdate);
      let state_ptr =
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TabBarState;
      if !state_ptr.is_null() {
        let state = &mut *state_ptr;

        let prev_active = state.active_index;
        let n_tabs = update.tabs.len();
        state.tabs = update.tabs;
        state.active_index = update.active_index;
        state.rect = update.rect;
        state.colors = update.colors;

        // Kick off a sliding indicator animation when the active tab changes.
        let tab_w = if n_tabs > 0 {
          state.rect.width() / n_tabs as i32
        } else {
          0
        };
        let new_target = tab_x(state.active_index, tab_w);

        if (state.indicator_target_x - new_target).abs() > 0.5 {
          // Start animating from the current visual position.
          state.indicator_from_x = state.indicator_cur_x;
          state.indicator_target_x = new_target;
          state.indicator_anim_start = Some(Instant::now());
          // Fire at ~60 fps; the timer stops itself when the animation ends.
          let _ = SetTimer(hwnd, INDICATOR_TIMER_ID, 16, None);
        } else if prev_active != state.active_index {
          // Active tab snapped to the same position — just update without animation.
          state.indicator_cur_x = new_target;
          state.indicator_target_x = new_target;
        }

        // Reposition and show in one call so the bar is never visible with
        // stale content at a new position (eliminates flicker on tab switch).
        let _ = SetWindowPos(
          hwnd,
          HWND(0),
          state.rect.left,
          state.rect.top,
          state.rect.width(),
          state.rect.height(),
          SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );

        // SAFETY: `hwnd` is valid and `None` means the full client area.
        let _ = InvalidateRect(hwnd, None, false);
      }
      LRESULT(0)
    }
    WM_CLOSE => {
      let _ = DestroyWindow(hwnd);
      LRESULT(0)
    }
    WM_DESTROY => {
      let state_ptr =
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut TabBarState;
      if !state_ptr.is_null() {
        // Kill any active animation timer before freeing state.
        let _ = KillTimer(hwnd, INDICATOR_TIMER_ID);
        // Zero GWLP_USERDATA before freeing to prevent use-after-free if
        // a stray message arrives before the window is fully gone.
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        // SAFETY: We own this allocation; it was created in `create()` and
        // is freed exactly here when the window is destroyed.
        drop(Box::from_raw(state_ptr));
      }
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
  }
}
