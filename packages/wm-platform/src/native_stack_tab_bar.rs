use std::{
  sync::OnceLock,
  time::{Duration, Instant},
};

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{
      COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM,
    },
    Graphics::Gdi::{
      CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC,
      DeleteObject, DrawTextW, SelectObject, SetBkMode, SetTextColor,
      AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
      BLENDFUNCTION, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
      DEFAULT_CHARSET, DIB_RGB_COLORS, DT_END_ELLIPSIS, DT_LEFT,
      DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, FW_NORMAL, FW_SEMIBOLD, HDC,
      HFONT, OUT_DEFAULT_PRECIS, TRANSPARENT,
    },
    UI::{
      Controls::WM_MOUSELEAVE,
      Input::KeyboardAndMouse::{
        ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE,
        TRACKMOUSEEVENT,
      },
      WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
        DestroyMenu, DestroyWindow, DrawIconEx, GetCursorPos,
        GetSystemMetrics, GetWindowLongPtrW, KillTimer, LoadCursorW,
        PostMessageW, RegisterClassW, SetForegroundWindow, SetTimer,
        SetWindowLongPtrW, SetWindowPos, ShowWindow, TrackPopupMenu,
        UpdateLayeredWindow, CREATESTRUCTW, DI_NORMAL, GWLP_USERDATA,
        IDC_ARROW, MA_NOACTIVATE, MF_STRING, SM_CXDRAG, SM_CYDRAG,
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSENDCHANGING, SWP_NOSIZE,
        SW_HIDE, SW_SHOWNOACTIVATE, TPM_NONOTIFY, TPM_RETURNCMD,
        TPM_RIGHTBUTTON, ULW_ALPHA, WM_APP, WM_CAPTURECHANGED, WM_CLOSE,
        WM_CREATE, WM_DESTROY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONUP,
        WM_MOUSEACTIVATE, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONUP,
        WM_TIMER, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_POPUP,
      },
    },
  },
};

use crate::{
  tab_icons,
  tab_layout::{TabAction, TabHit, TabLayout, TabLayoutParams, TabRect},
  tab_paint::{Canvas, Rgba},
  window_class, Color, Dispatcher, Rect,
};

/// Posted with a `Box<TabFrame>` in `WPARAM` to show the bar with new
/// contents.
const WM_UPDATE_TABS: u32 = WM_APP + 1;

/// Posted to hide the bar.
const WM_HIDE_TABS: u32 = WM_APP + 2;

/// Posted by the icon thread once an icon is cached.
const WM_ICON_READY: u32 = WM_APP + 3;

const PILL_TIMER_ID: usize = 1;

/// How long the active tab's highlight takes to slide to a new tab.
const PILL_SLIDE: Duration = Duration::from_millis(150);

const MENU_CLOSE: usize = 1;
const MENU_DETACH: usize = 2;

static CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// A tab of the bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TabInfo {
  /// Title shown on the tab.
  pub title: String,

  /// Handle of the tab's window, for its icon.
  pub hwnd: isize,
}

/// When tabs show a close button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabCloseMode {
  /// On the active and the hovered tab.
  Hover,
  Always,
  Never,
}

/// Look of a tab bar, in physical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct TabBarStyle {
  pub background: Color,
  /// Opacity of the whole bar.
  pub opacity: u8,
  pub active_background: Color,
  pub hover_background: Color,
  pub inactive_background: Color,
  pub text: Color,
  pub inactive_text: Color,
  pub font_family: String,
  pub font_size: i32,
  pub corner_radius: i32,
  pub min_tab_width: i32,
  /// 0 lets tabs share the whole bar.
  pub max_tab_width: i32,
  pub show_icons: bool,
  pub show_numbers: bool,
  pub close_button: TabCloseMode,
  /// Scale factor of the bar's monitor, for the size thresholds.
  pub scale_factor: f32,
}

/// Everything shown by a tab bar, posted to its thread as a whole.
#[derive(Clone, Debug, PartialEq)]
pub struct TabFrame {
  pub rect: Rect,
  pub tabs: Vec<TabInfo>,
  pub active_index: usize,
  /// The stack's active window. The bar is kept directly behind it in
  /// z-order, so it is covered by whatever covers the window.
  pub anchor: isize,
  pub style: TabBarStyle,
}

/// A tab being dragged with the left button.
struct Drag {
  index: usize,
  start: (i32, i32),
  current: (i32, i32),
  is_moving: bool,
}

/// The active highlight sliding from one tab to another.
struct PillSlide {
  from: TabRect,
  started: Instant,
}

/// State of a bar, owned by its window on the event-loop thread.
struct BarState {
  frame: Option<TabFrame>,
  layout: TabLayout,
  hover: TabHit,
  is_tracking_leave: bool,
  pressed_close: Option<usize>,
  drag: Option<Drag>,
  pill_slide: Option<PillSlide>,
  /// Highlight drawn last, where a slide starts from.
  last_pill: Option<TabRect>,
  on_action: Box<dyn Fn(TabAction) + Send + 'static>,
}

/// A stack's tab bar: a layered popup on the event-loop thread, which
/// handles its input and draws it.
///
/// The WM thread only ever posts to it, since a synchronous call could
/// deadlock with the event loop waiting on the WM.
pub struct NativeStackTabBar {
  hwnd: isize,
  /// Frame last posted, to skip posting identical ones every animation
  /// tick. `None` while hidden.
  last_frame: Option<TabFrame>,
}

// SAFETY: Only the raw handle is kept, used to post messages, which is
// allowed from any thread.
unsafe impl Send for NativeStackTabBar {}

impl NativeStackTabBar {
  /// Creates a hidden tab bar. `on_action` is called on the event-loop
  /// thread for each user action.
  pub fn create(
    dispatcher: &Dispatcher,
    on_action: Box<dyn Fn(TabAction) + Send + 'static>,
  ) -> crate::Result<Self> {
    let state = Box::new(BarState {
      frame: None,
      layout: TabLayout::new(&TabLayoutParams {
        width: 0,
        height: 0,
        tab_count: 0,
        active_index: 0,
        min_tab_width: 0,
        max_tab_width: 0,
        icon_only_width: 0,
        close_min_width: 0,
        show_icons: false,
      }),
      hover: TabHit::Empty,
      is_tracking_leave: false,
      pressed_close: None,
      drag: None,
      pill_slide: None,
      last_pill: None,
      on_action,
    });

    // Passed as an integer so the closure is `Send`.
    let state_ptr = Box::into_raw(state) as usize;

    let hwnd =
      dispatcher.dispatch_sync(move || -> crate::Result<isize> {
        ensure_class_registered();

        // SAFETY: The class is registered above. `state_ptr` is owned by
        // the window from here on and freed in `WM_DESTROY`.
        let hwnd = unsafe {
          CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            w!("GlazeWM_TabBar"),
            w!(""),
            WS_POPUP,
            0,
            0,
            0,
            0,
            None,
            None,
            None,
            Some(state_ptr as *const std::ffi::c_void),
          )
        };

        if hwnd.0 == 0 {
          // SAFETY: Creation failed, so nothing else took ownership.
          unsafe { drop(Box::from_raw(state_ptr as *mut BarState)) };
          return Err(crate::Error::Platform(
            "Failed to create tab bar window.".to_string(),
          ));
        }

        Ok(hwnd.0)
      })??;

    Ok(Self {
      hwnd,
      last_frame: None,
    })
  }

  /// Shows the bar with `frame`. `restack` puts it back behind its anchor
  /// even if nothing else changed, e.g. after a focus change.
  pub fn update(&mut self, frame: TabFrame, restack: bool) {
    if !restack && self.last_frame.as_ref() == Some(&frame) {
      return;
    }

    self.last_frame = Some(frame.clone());
    let frame = Box::into_raw(Box::new(frame)) as usize;

    // SAFETY: Ownership of the frame passes to the window procedure, which
    // frees it. If the post fails the window is gone and it leaks.
    unsafe {
      let _ = PostMessageW(
        HWND(self.hwnd),
        WM_UPDATE_TABS,
        WPARAM(frame),
        LPARAM(isize::from(restack)),
      );
    }
  }

  /// Hides the bar.
  pub fn hide(&mut self) {
    if self.last_frame.take().is_none() {
      return;
    }

    // SAFETY: Posting to a destroyed window just fails.
    unsafe {
      let _ =
        PostMessageW(HWND(self.hwnd), WM_HIDE_TABS, WPARAM(0), LPARAM(0));
    }
  }
}

impl Drop for NativeStackTabBar {
  fn drop(&mut self) {
    // SAFETY: Posting to a destroyed window just fails.
    unsafe {
      let _ =
        PostMessageW(HWND(self.hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
    }
  }
}

fn ensure_class_registered() {
  CLASS_REGISTERED.get_or_init(|| {
    let class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_TabBar"),
      lpfnWndProc: Some(wnd_proc),
      // SAFETY: `IDC_ARROW` is a system cursor.
      hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap_or_default(),
      ..Default::default()
    };

    // SAFETY: `class` is fully initialized with a static class name.
    unsafe { RegisterClassW(&raw const class) };
  });
}

impl BarState {
  fn relayout(&mut self) {
    let Some(frame) = &self.frame else {
      return;
    };

    let scale = |px: f32| {
      #[allow(clippy::cast_possible_truncation)]
      let scaled = (px * frame.style.scale_factor).round() as i32;
      scaled
    };

    self.layout = TabLayout::new(&TabLayoutParams {
      width: frame.rect.width(),
      height: frame.rect.height(),
      tab_count: frame.tabs.len(),
      active_index: frame.active_index,
      min_tab_width: frame.style.min_tab_width,
      max_tab_width: frame.style.max_tab_width,
      icon_only_width: scale(60.0),
      close_min_width: scale(80.0),
      show_icons: frame.style.show_icons,
    });
  }

  fn is_close_visible(&self, index: usize) -> bool {
    let Some(frame) = &self.frame else {
      return false;
    };

    match frame.style.close_button {
      TabCloseMode::Always => true,
      TabCloseMode::Never => false,
      TabCloseMode::Hover => {
        index == frame.active_index
          || matches!(
            self.hover,
            TabHit::Tab(hovered) | TabHit::Close(hovered) if hovered == index
          )
      }
    }
  }

  /// Order in which tabs are drawn: the dragged tab shown at the slot it
  /// would be dropped into.
  fn display_order(&self) -> Vec<usize> {
    let count = self.layout.slots.len();
    let mut order = (0..count).collect::<Vec<_>>();

    if let Some(drag) = self.drag.as_ref().filter(|drag| drag.is_moving) {
      let to = self.layout.drop_index(drag.current.0);
      if drag.index < count {
        let index = order.remove(drag.index);
        order.insert(to.min(count - 1), index);
      }
    }

    order
  }

  /// Highlight of the active tab, sliding if a slide is running.
  fn active_pill(&self) -> Option<TabRect> {
    let frame = self.frame.as_ref()?;
    let order = self.display_order();
    let position = order.iter().position(|i| *i == frame.active_index)?;
    let target = self.layout.slots.get(position)?.pill;

    let Some(slide) = &self.pill_slide else {
      return Some(target);
    };

    let progress = (slide.started.elapsed().as_secs_f32()
      / PILL_SLIDE.as_secs_f32())
    .min(1.0);
    let eased = 1.0 - (1.0 - progress).powi(3);

    let lerp = |from: i32, to: i32| {
      #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
      )]
      let value =
        (from as f32 + (to - from) as f32 * eased).round() as i32;
      value
    };

    Some(TabRect {
      left: lerp(slide.from.left, target.left),
      right: lerp(slide.from.right, target.right),
      ..target
    })
  }
}

/// Draws the bar into a layered bitmap and shows it at its frame's rect.
///
/// # Safety
///
/// `hwnd` must be the bar's window, called on its thread.
unsafe fn render(hwnd: HWND, state: &mut BarState) {
  let Some(frame) = state.frame.clone() else {
    return;
  };

  let (width, height) = (frame.rect.width(), frame.rect.height());
  if width <= 0 || height <= 0 {
    return;
  }

  let info = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: u32::try_from(std::mem::size_of::<BITMAPINFOHEADER>())
        .unwrap_or_default(),
      biWidth: width,
      // Negative height: top-down rows.
      biHeight: -height,
      biPlanes: 1,
      biBitCount: 32,
      biCompression: BI_RGB.0,
      ..Default::default()
    },
    ..Default::default()
  };

  let mem_dc = CreateCompatibleDC(None);
  let mut bits = std::ptr::null_mut();
  let Ok(bitmap) = CreateDIBSection(
    mem_dc,
    &raw const info,
    DIB_RGB_COLORS,
    &raw mut bits,
    None,
    0,
  ) else {
    let _ = DeleteDC(mem_dc);
    return;
  };

  let old_bitmap = SelectObject(mem_dc, bitmap);
  let pixel_count = usize::try_from(width * height).unwrap_or(0);

  // SAFETY: The DIB section holds `width * height` 32-bit pixels, owned by
  // `bitmap` until it is deleted below.
  let pixels =
    std::slice::from_raw_parts_mut(bits.cast::<u32>(), pixel_count);
  paint_shapes(pixels, width, height, state, &frame);

  // GDI zeroes the alpha of every pixel it draws, so it is restored after
  // drawing text and icons, which only ever land on opaque parts.
  let alpha = pixels.iter().map(|p| p >> 24).collect::<Vec<_>>();
  paint_text_and_icons(mem_dc, hwnd, state, &frame);
  for (pixel, alpha) in pixels.iter_mut().zip(alpha) {
    *pixel = (*pixel & 0x00ff_ffff) | (alpha << 24);
  }

  paint_close_buttons(pixels, width, height, state, &frame);

  let blend = BLENDFUNCTION {
    BlendOp: u8::try_from(AC_SRC_OVER).unwrap_or_default(),
    BlendFlags: 0,
    SourceConstantAlpha: frame.style.opacity,
    AlphaFormat: u8::try_from(AC_SRC_ALPHA).unwrap_or_default(),
  };

  let position = POINT {
    x: frame.rect.x(),
    y: frame.rect.y(),
  };
  let size = SIZE {
    cx: width,
    cy: height,
  };
  let source = POINT { x: 0, y: 0 };

  if let Err(err) = UpdateLayeredWindow(
    hwnd,
    None,
    Some(&raw const position),
    Some(&raw const size),
    mem_dc,
    Some(&raw const source),
    COLORREF(0),
    Some(&raw const blend),
    ULW_ALPHA,
  ) {
    tracing::warn!("Failed to draw tab bar: {err}");
  }

  SelectObject(mem_dc, old_bitmap);
  let _ = DeleteObject(bitmap);
  let _ = DeleteDC(mem_dc);
}

fn rgba(color: Color) -> Rgba {
  Rgba {
    r: color.r,
    g: color.g,
    b: color.b,
    a: color.a,
  }
}

/// Draws the strip and the tab highlights.
fn paint_shapes(
  pixels: &mut [u32],
  width: i32,
  height: i32,
  state: &BarState,
  frame: &TabFrame,
) {
  pixels.fill(0);

  let style = &frame.style;
  let mut canvas = Canvas {
    pixels,
    width,
    height,
  };

  canvas.fill_rounded_rect(
    TabRect {
      left: 0,
      top: 0,
      right: width,
      bottom: height,
    },
    style.corner_radius,
    rgba(style.background),
  );

  let pill_radius = (style.corner_radius - 2).max(0);

  for (position, index) in state.display_order().into_iter().enumerate() {
    let Some(slot) = state.layout.slots.get(position) else {
      continue;
    };

    if index == frame.active_index {
      continue;
    }

    let is_hovered = matches!(
      state.hover,
      TabHit::Tab(hovered) | TabHit::Close(hovered) if hovered == index
    ) || state
      .drag
      .as_ref()
      .is_some_and(|drag| drag.is_moving && drag.index == index);

    let color = if is_hovered {
      style.hover_background
    } else {
      style.inactive_background
    };

    canvas.fill_rounded_rect(slot.pill, pill_radius, rgba(color));
  }

  if let Some(pill) = state.active_pill() {
    canvas.fill_rounded_rect(
      pill,
      pill_radius,
      rgba(style.active_background),
    );
  }
}

/// Draws titles and icons with GDI.
///
/// # Safety
///
/// `dc` must have the bar's bitmap selected.
unsafe fn paint_text_and_icons(
  dc: HDC,
  hwnd: HWND,
  state: &BarState,
  frame: &TabFrame,
) {
  let style = &frame.style;
  let regular = create_font(style, false);
  let bold = create_font(style, true);
  let old_font = SelectObject(dc, regular);
  SetBkMode(dc, TRANSPARENT);

  for (position, index) in state.display_order().into_iter().enumerate() {
    let (Some(slot), Some(tab)) =
      (state.layout.slots.get(position), frame.tabs.get(index))
    else {
      continue;
    };

    let is_active = index == frame.active_index;

    if let Some(icon_rect) = slot.icon {
      if let Some(icon) =
        tab_icons::icon_for(tab.hwnd, hwnd, WM_ICON_READY)
      {
        let _ = DrawIconEx(
          dc,
          icon_rect.left,
          icon_rect.top,
          icon,
          icon_rect.width(),
          icon_rect.height(),
          0,
          None,
          DI_NORMAL,
        );
      }
    }

    if slot.text.width() <= 0 {
      continue;
    }

    let title = if style.show_numbers {
      format!("{}. {}", index + 1, tab.title)
    } else {
      tab.title.clone()
    };

    let color = if is_active {
      style.text
    } else {
      style.inactive_text
    };

    SelectObject(dc, if is_active { bold } else { regular });
    SetTextColor(dc, COLORREF(color.to_bgr()));

    let mut text = title.encode_utf16().collect::<Vec<_>>();
    let mut rect = RECT {
      left: slot.text.left,
      top: slot.text.top,
      right: slot.text.right,
      bottom: slot.text.bottom,
    };

    DrawTextW(
      dc,
      &mut text,
      &raw mut rect,
      DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
    );
  }

  SelectObject(dc, old_font);
  let _ = DeleteObject(regular);
  let _ = DeleteObject(bold);
}

/// Creates the tab title font, bold for the active tab.
///
/// # Safety
///
/// The returned font must be deleted by the caller.
unsafe fn create_font(style: &TabBarStyle, is_bold: bool) -> HFONT {
  let family = style
    .font_family
    .encode_utf16()
    .chain(std::iter::once(0))
    .collect::<Vec<_>>();

  let weight = if is_bold { FW_SEMIBOLD } else { FW_NORMAL };

  CreateFontW(
    // Negative: character height rather than cell height.
    -style.font_size.max(1),
    0,
    0,
    0,
    i32::try_from(weight.0).unwrap_or(400),
    0,
    0,
    0,
    u32::from(DEFAULT_CHARSET.0),
    u32::from(OUT_DEFAULT_PRECIS.0),
    u32::from(CLIP_DEFAULT_PRECIS.0),
    u32::from(CLEARTYPE_QUALITY.0),
    0,
    PCWSTR(family.as_ptr()),
  )
}

/// Draws the close buttons' crosses, anti-aliased.
fn paint_close_buttons(
  pixels: &mut [u32],
  width: i32,
  height: i32,
  state: &BarState,
  frame: &TabFrame,
) {
  let mut canvas = Canvas {
    pixels,
    width,
    height,
  };

  for (position, index) in state.display_order().into_iter().enumerate() {
    let Some(close) =
      state.layout.slots.get(position).and_then(|slot| slot.close)
    else {
      continue;
    };

    if !state.is_close_visible(index) {
      continue;
    }

    if state.hover == TabHit::Close(index) {
      canvas.fill_rounded_rect(
        close,
        close.width() / 4,
        rgba(frame.style.hover_background),
      );
    }

    let color = if index == frame.active_index {
      frame.style.text
    } else {
      frame.style.inactive_text
    };

    #[allow(clippy::cast_precision_loss)]
    let (inset, left, top, right, bottom) = (
      close.width() as f32 * 0.3,
      close.left as f32,
      close.top as f32,
      close.right as f32,
      close.bottom as f32,
    );
    let thickness = (frame.style.scale_factor * 1.3).max(1.0);

    canvas.stroke_line(
      (left + inset, top + inset),
      (right - inset, bottom - inset),
      thickness,
      rgba(color),
    );
    canvas.stroke_line(
      (right - inset, top + inset),
      (left + inset, bottom - inset),
      thickness,
      rgba(color),
    );
  }
}

/// Signed client coordinates packed into a mouse message's `LPARAM`.
fn mouse_position(lparam: LPARAM) -> (i32, i32) {
  #[allow(clippy::cast_possible_truncation)]
  let (x, y) = (lparam.0 as i16, (lparam.0 >> 16) as i16);
  (i32::from(x), i32::from(y))
}

/// Applies a newly posted frame.
///
/// # Safety
///
/// `hwnd` must be the bar's window, called on its thread.
unsafe fn apply_frame(
  hwnd: HWND,
  state: &mut BarState,
  frame: TabFrame,
  restack: bool,
) {
  let was_visible = state.frame.is_some();
  let previous = state.frame.replace(frame);

  // Icons of windows no longer shown are dropped, and so are the icons of
  // windows whose title changed, since apps tend to change both together.
  if let (Some(previous), Some(current)) = (&previous, &state.frame) {
    for old in &previous.tabs {
      let is_unchanged = current
        .tabs
        .iter()
        .any(|new| new.hwnd == old.hwnd && new.title == old.title);
      if !is_unchanged {
        tab_icons::invalidate(old.hwnd);
      }
    }
  }

  let old_pill = state.last_pill;
  state.relayout();

  let active_changed = previous.as_ref().is_some_and(|previous| {
    state.frame.as_ref().is_some_and(|current| {
      previous.active_index != current.active_index
        || previous.tabs.len() != current.tabs.len()
    })
  });

  if was_visible && active_changed {
    if let Some(from) = old_pill {
      state.pill_slide = Some(PillSlide {
        from,
        started: Instant::now(),
      });
      SetTimer(hwnd, PILL_TIMER_ID, 16, None);
    }
  }

  render(hwnd, state);
  state.last_pill = state.active_pill();

  let anchor_changed = previous.as_ref().map(|p| p.anchor)
    != state.frame.as_ref().map(|f| f.anchor);

  if let Some(anchor) =
    state.frame.as_ref().map(|frame| HWND(frame.anchor))
  {
    if restack || anchor_changed || !was_visible {
      window_class::match_z_band(hwnd, anchor);
      let _ = SetWindowPos(
        hwnd,
        window_class::insert_after_point(anchor),
        0,
        0,
        0,
        0,
        SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOSENDCHANGING,
      );
    }
  }

  if !was_visible {
    ShowWindow(hwnd, SW_SHOWNOACTIVATE);
  }
}

/// Handles a left-button release: a click, the end of a drag, or a close
/// button press.
fn finish_left_click(state: &mut BarState, (x, y): (i32, i32)) {
  let pressed_close = state.pressed_close.take();
  let hit = state
    .layout
    .hit_test(x, y, |index| state.is_close_visible(index));

  if let Some(index) = pressed_close {
    if hit == TabHit::Close(index) {
      (state.on_action)(TabAction::Close(index));
    }
    return;
  }

  let Some(drag) = state.drag.take() else {
    return;
  };

  if !drag.is_moving {
    (state.on_action)(TabAction::Activate(drag.index));
    return;
  }

  let height = state.frame.as_ref().map_or(0, |f| f.rect.height());
  let is_torn_off = y < -height || y > height * 2;

  if is_torn_off {
    (state.on_action)(TabAction::Detach(drag.index));
    return;
  }

  let to = state.layout.drop_index(x);
  if to != drag.index {
    (state.on_action)(TabAction::Move {
      from: drag.index,
      to,
    });
  }
}

/// Shows the context menu of tab `index` at the cursor.
///
/// # Safety
///
/// `hwnd` must be the bar's window, called on its thread.
unsafe fn show_context_menu(hwnd: HWND, state: &BarState, index: usize) {
  let Ok(menu) = CreatePopupMenu() else {
    return;
  };

  let _ = AppendMenuW(menu, MF_STRING, MENU_CLOSE, w!("Close"));
  let _ =
    AppendMenuW(menu, MF_STRING, MENU_DETACH, w!("Remove from stack"));

  let mut cursor = POINT::default();
  let _ = GetCursorPos(&raw mut cursor);

  // Required for the menu to close when clicking elsewhere.
  let _ = SetForegroundWindow(hwnd);

  let command = TrackPopupMenu(
    menu,
    TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_NONOTIFY,
    cursor.x,
    cursor.y,
    0,
    hwnd,
    None,
  );

  let _ = DestroyMenu(menu);

  match usize::try_from(command.0).unwrap_or(0) {
    MENU_CLOSE => (state.on_action)(TabAction::Close(index)),
    MENU_DETACH => (state.on_action)(TabAction::Detach(index)),
    _ => {}
  }
}

/// Tracks hover, or the tab being dragged.
///
/// # Safety
///
/// `hwnd` must be the bar's window, called on its thread.
unsafe fn on_mouse_move(
  hwnd: HWND,
  state: &mut BarState,
  position: (i32, i32),
) {
  if !state.is_tracking_leave {
    let mut track = TRACKMOUSEEVENT {
      cbSize: u32::try_from(std::mem::size_of::<TRACKMOUSEEVENT>())
        .unwrap_or_default(),
      dwFlags: TME_LEAVE,
      hwndTrack: hwnd,
      dwHoverTime: 0,
    };
    state.is_tracking_leave = TrackMouseEvent(&raw mut track).is_ok();
  }

  if let Some(drag) = &mut state.drag {
    drag.current = position;
    let threshold =
      (GetSystemMetrics(SM_CXDRAG), GetSystemMetrics(SM_CYDRAG));
    drag.is_moving |= (position.0 - drag.start.0).abs() > threshold.0
      || (position.1 - drag.start.1).abs() > threshold.1;

    if drag.is_moving {
      render(hwnd, state);
    }
    return;
  }

  let hover = state.layout.hit_test(position.0, position.1, |_| true);
  if hover != state.hover {
    state.hover = hover;
    render(hwnd, state);
  }
}

/// Starts a click or drag on a tab, or a press on a close button.
///
/// # Safety
///
/// `hwnd` must be the bar's window, called on its thread.
unsafe fn on_left_button_down(
  hwnd: HWND,
  state: &mut BarState,
  position: (i32, i32),
) {
  let hit = state.layout.hit_test(position.0, position.1, |index| {
    state.is_close_visible(index)
  });

  match hit {
    TabHit::Close(index) => state.pressed_close = Some(index),
    TabHit::Tab(index) => {
      state.drag = Some(Drag {
        index,
        start: position,
        current: position,
        is_moving: false,
      });
      SetCapture(hwnd);
    }
    TabHit::Empty => {}
  }
}

/// Advances the active highlight's slide.
///
/// # Safety
///
/// `hwnd` must be the bar's window, called on its thread.
unsafe fn on_pill_timer(hwnd: HWND, state: &mut BarState) {
  let is_done = state
    .pill_slide
    .as_ref()
    .is_none_or(|slide| slide.started.elapsed() >= PILL_SLIDE);

  if is_done {
    state.pill_slide = None;
    let _ = KillTimer(hwnd, PILL_TIMER_ID);
  }

  render(hwnd, state);
  state.last_pill = state.active_pill();
}

/// Frees the bar's state as its window is destroyed.
///
/// # Safety
///
/// `state_ptr` must be the state allocated in `create`, freed only here.
unsafe fn on_destroy(hwnd: HWND, state_ptr: *mut BarState) {
  let _ = KillTimer(hwnd, PILL_TIMER_ID);

  // Cleared first, so no stray message reaches freed state.
  SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);

  let state = Box::from_raw(state_ptr);
  if let Some(frame) = &state.frame {
    for tab in &frame.tabs {
      tab_icons::invalidate(tab.hwnd);
    }
  }
}

/// Closes a tab on a middle click, or shows its menu on a right click.
///
/// # Safety
///
/// `hwnd` must be the bar's window, called on its thread.
unsafe fn on_tab_button_up(
  hwnd: HWND,
  state: &BarState,
  msg: u32,
  (x, y): (i32, i32),
) {
  let (TabHit::Tab(index) | TabHit::Close(index)) =
    state.layout.hit_test(x, y, |_| true)
  else {
    return;
  };

  if msg == WM_MBUTTONUP {
    (state.on_action)(TabAction::Close(index));
  } else {
    show_context_menu(hwnd, state, index);
  }
}

/// Window procedure of tab bars.
unsafe extern "system" fn wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  if msg == WM_CREATE {
    let create = &*(lparam.0 as *const CREATESTRUCTW);
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    return LRESULT(0);
  }

  let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut BarState;
  if state_ptr.is_null() {
    return DefWindowProcW(hwnd, msg, wparam, lparam);
  }

  // SAFETY: The state lives until `WM_DESTROY` and is only touched on
  // this thread.
  let state = &mut *state_ptr;

  match msg {
    WM_UPDATE_TABS => {
      // SAFETY: Posted by `update` with an owned, leaked `TabFrame`.
      let frame = Box::from_raw(wparam.0 as *mut TabFrame);
      apply_frame(hwnd, state, *frame, lparam.0 != 0);
      LRESULT(0)
    }
    WM_HIDE_TABS => {
      state.frame = None;
      state.drag = None;
      state.hover = TabHit::Empty;
      state.last_pill = None;
      ShowWindow(hwnd, SW_HIDE);
      LRESULT(0)
    }
    WM_ICON_READY => {
      render(hwnd, state);
      LRESULT(0)
    }
    WM_TIMER if wparam.0 == PILL_TIMER_ID => {
      on_pill_timer(hwnd, state);
      LRESULT(0)
    }
    WM_MOUSEACTIVATE => {
      LRESULT(isize::try_from(MA_NOACTIVATE).unwrap_or_default())
    }
    WM_MOUSEMOVE => {
      on_mouse_move(hwnd, state, mouse_position(lparam));
      LRESULT(0)
    }
    WM_MOUSELEAVE => {
      state.is_tracking_leave = false;
      if state.hover != TabHit::Empty && state.drag.is_none() {
        state.hover = TabHit::Empty;
        render(hwnd, state);
      }
      LRESULT(0)
    }
    WM_LBUTTONDOWN => {
      on_left_button_down(hwnd, state, mouse_position(lparam));
      LRESULT(0)
    }
    WM_LBUTTONUP => {
      let had_drag = state.drag.is_some();
      finish_left_click(state, mouse_position(lparam));
      if had_drag {
        let _ = ReleaseCapture();
      }
      render(hwnd, state);
      LRESULT(0)
    }
    WM_CAPTURECHANGED => {
      if state.drag.take().is_some() {
        render(hwnd, state);
      }
      LRESULT(0)
    }
    WM_MBUTTONUP | WM_RBUTTONUP => {
      on_tab_button_up(hwnd, state, msg, mouse_position(lparam));
      LRESULT(0)
    }
    WM_MOUSEWHEEL => {
      // The high word of `wparam` is the signed wheel delta.
      #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap
      )]
      let delta = (wparam.0 >> 16) as u16 as i16;
      (state.on_action)(TabAction::Cycle { prev: delta > 0 });
      LRESULT(0)
    }
    WM_CLOSE => {
      let _ = DestroyWindow(hwnd);
      LRESULT(0)
    }
    WM_DESTROY => {
      on_destroy(hwnd, state_ptr);
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
  }
}
