use std::sync::OnceLock;

use windows::{
  core::{w, PCWSTR, PWSTR},
  Win32::{
    Foundation::{
      COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM,
    },
    Graphics::Gdi::{
      CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC,
      DeleteObject, DrawTextW, SelectObject, SetBkMode, SetTextColor,
      AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
      BLENDFUNCTION, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
      DEFAULT_CHARSET, DIB_RGB_COLORS, DT_CALCRECT, DT_END_ELLIPSIS,
      DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, FW_NORMAL,
      FW_SEMIBOLD, HDC, HFONT, OUT_DEFAULT_PRECIS, TRANSPARENT,
    },
    UI::{
      Controls::{
        InitCommonControlsEx, ICC_WIN95_CLASSES, INITCOMMONCONTROLSEX,
        NMHDR, NMTTDISPINFOW, TOOLTIPS_CLASSW, TTF_SUBCLASS, TTM_ADDTOOLW,
        TTM_DELTOOLW, TTM_POP, TTM_SETMAXTIPWIDTH, TTN_GETDISPINFOW,
        TTS_ALWAYSTIP, TTS_NOPREFIX, TTTOOLINFOW, WM_MOUSELEAVE,
      },
      Input::KeyboardAndMouse::{
        ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE,
        TRACKMOUSEEVENT,
      },
      WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
        DestroyMenu, DestroyWindow, DrawIconEx, GetCursorPos,
        GetSystemMetrics, GetWindow, GetWindowLongPtrW, LoadCursorW,
        PostMessageW, RegisterClassW, SendMessageW, SetForegroundWindow,
        SetWindowLongPtrW, SetWindowPos, ShowWindow, TrackPopupMenu,
        UpdateLayeredWindow, CREATESTRUCTW, DI_NORMAL, GWLP_USERDATA,
        GW_HWNDNEXT, GW_HWNDPREV, IDC_ARROW, MA_NOACTIVATE, MF_STRING,
        SM_CXDRAG, SM_CYDRAG, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOSENDCHANGING, SWP_NOSIZE, SW_HIDE, SW_SHOWNOACTIVATE,
        TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTBUTTON, ULW_ALPHA,
        WINDOW_STYLE, WM_APP, WM_CAPTURECHANGED, WM_CLOSE, WM_CREATE,
        WM_DESTROY, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONUP,
        WM_MOUSEACTIVATE, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NOTIFY,
        WM_RBUTTONUP, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
      },
    },
  },
};

use crate::{
  overlay_window::OverlayKind,
  paint::Canvas,
  tab_layout::{TabAction, TabHit, TabLayout, TabLayoutParams, TabRect},
  window_class, window_icons, Dispatcher, Rect, TabBarStyle, TabCloseMode,
  TabFrame,
};

/// Posted with a `Box<TabFrame>` in `WPARAM` to show the bar with new
/// contents.
const WM_UPDATE_TABS: u32 = WM_APP + 1;

/// Posted to hide the bar.
const WM_HIDE_TABS: u32 = WM_APP + 2;

/// Posted by the icon thread once an icon is cached.
const WM_ICON_READY: u32 = WM_APP + 3;

/// Posted to put the bar back behind its anchor if it was pushed away.
const WM_RESTACK_TABS: u32 = WM_APP + 4;

/// Bound on the walk over the overlays behind the anchor; a window has at
/// most a backdrop and a border.
const MAX_OVERLAY_WALK: usize = 8;

const MENU_CLOSE: usize = 1;
const MENU_DETACH: usize = 2;
const MENU_FLOAT: usize = 3;

static CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// A tab being dragged with the left button.
struct Drag {
  index: usize,
  start: (i32, i32),
  current: (i32, i32),
  is_moving: bool,
}

/// State of a bar, owned by its window on the event-loop thread.
struct BarState {
  frame: Option<TabFrame>,
  layout: TabLayout,
  hover: TabHit,
  is_tracking_leave: bool,
  pressed_close: Option<usize>,
  drag: Option<Drag>,
  /// Tooltip control showing the full title of a cut-off tab.
  tooltip: HWND,
  /// Tooltip tools registered, one per tab, with the tab index as ID.
  tool_count: usize,
  /// Per tab index, whether its title didn't fit, as of the last render.
  truncated: Vec<bool>,
  /// Text handed to the tooltip; must outlive the notification.
  tooltip_text: Vec<u16>,
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
        top: 0,
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
      tooltip: HWND(0),
      tool_count: 0,
      truncated: Vec::new(),
      tooltip_text: vec![0],
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

        // SAFETY: The window owns the state now and lives on this thread,
        // where it is only touched from here and its window procedure.
        unsafe {
          (*(state_ptr as *mut BarState)).tooltip = create_tooltip(hwnd);
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

  /// Puts the shown bar back behind its anchor, if other windows were
  /// raised in between (e.g. an app restacking its own windows).
  pub fn keep_behind_anchor(&self) {
    if self.last_frame.is_none() {
      return;
    }

    // SAFETY: Posting to a destroyed window just fails.
    unsafe {
      let _ = PostMessageW(
        HWND(self.hwnd),
        WM_RESTACK_TABS,
        WPARAM(0),
        LPARAM(0),
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

/// Creates the tooltip control of the bar `owner`. Its window is
/// destroyed along with the bar.
///
/// # Safety
///
/// Must be called on the bar's thread.
unsafe fn create_tooltip(owner: HWND) -> HWND {
  static COMMON_CONTROLS: OnceLock<()> = OnceLock::new();
  COMMON_CONTROLS.get_or_init(|| {
    let controls = INITCOMMONCONTROLSEX {
      dwSize: u32::try_from(std::mem::size_of::<INITCOMMONCONTROLSEX>())
        .unwrap_or_default(),
      dwICC: ICC_WIN95_CLASSES,
    };
    let _ = InitCommonControlsEx(&raw const controls);
  });

  let tooltip = CreateWindowExW(
    WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
    TOOLTIPS_CLASSW,
    PCWSTR::null(),
    WS_POPUP | WINDOW_STYLE(TTS_NOPREFIX | TTS_ALWAYSTIP),
    0,
    0,
    0,
    0,
    owner,
    None,
    None,
    None,
  );

  // Long titles wrap rather than spanning the screen.
  SendMessageW(tooltip, TTM_SETMAXTIPWIDTH, WPARAM(0), LPARAM(600));
  tooltip
}

/// Info of the tooltip tool for tab `index` of `bar`.
fn tool_info(bar: HWND, index: usize, rect: TabRect) -> TTTOOLINFOW {
  TTTOOLINFOW {
    cbSize: u32::try_from(std::mem::size_of::<TTTOOLINFOW>())
      .unwrap_or_default(),
    uFlags: TTF_SUBCLASS,
    hwnd: bar,
    uId: index,
    rect: RECT {
      left: rect.left,
      top: rect.top,
      right: rect.right,
      bottom: rect.bottom,
    },
    // `LPSTR_TEXTCALLBACKW`: the text is asked for when shown.
    lpszText: PWSTR(std::ptr::without_provenance_mut(usize::MAX)),
    ..Default::default()
  }
}

impl BarState {
  /// Registers one tooltip tool per tab, over the tab's current slot.
  ///
  /// # Safety
  ///
  /// `bar` must be the bar's window, called on its thread.
  unsafe fn sync_tooltip_tools(&mut self, bar: HWND) {
    if self.tooltip.0 == 0 {
      return;
    }

    for index in 0..self.tool_count {
      let info = tool_info(bar, index, TabRect::default());
      SendMessageW(
        self.tooltip,
        TTM_DELTOOLW,
        WPARAM(0),
        LPARAM(std::ptr::addr_of!(info) as isize),
      );
    }

    let order = self.display_order();
    for (position, index) in order.iter().enumerate() {
      let Some(slot) = self.layout.slots.get(position) else {
        continue;
      };

      let info = tool_info(bar, *index, slot.pill);
      SendMessageW(
        self.tooltip,
        TTM_ADDTOOLW,
        WPARAM(0),
        LPARAM(std::ptr::addr_of!(info) as isize),
      );
    }

    self.tool_count = order.len();
  }

  /// Points `info` at the full title of its tab if it was cut off, and
  /// at an empty text, which shows no tooltip, otherwise.
  fn fill_tooltip(&mut self, info: &mut NMTTDISPINFOW) {
    let index = info.hdr.idFrom;
    let title = self
      .frame
      .as_ref()
      .and_then(|frame| frame.tabs.get(index))
      .filter(|_| self.truncated.get(index).copied().unwrap_or(false))
      .map_or_else(String::new, |tab| tab.title.clone());

    self.tooltip_text = title.encode_utf16().chain([0]).collect();
    info.lpszText = PWSTR(self.tooltip_text.as_mut_ptr());
  }

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
      top: frame.rect.top - frame.outer_rect.top,
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

  /// Highlight of the active tab.
  fn active_pill(&self) -> Option<TabRect> {
    let frame = self.frame.as_ref()?;
    let order = self.display_order();
    let position = order.iter().position(|i| *i == frame.active_index)?;
    self.layout.slots.get(position).map(|slot| slot.pill)
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

  let (width, height) =
    (frame.outer_rect.width(), frame.outer_rect.height());
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
  state.truncated = paint_text_and_icons(mem_dc, hwnd, state, &frame);
  for (pixel, alpha) in pixels.iter_mut().zip(alpha) {
    *pixel = (*pixel & 0x00ff_ffff) | (alpha << 24);
  }

  paint_close_buttons(pixels, width, height, state, &frame);

  let blend = BLENDFUNCTION {
    BlendOp: u8::try_from(AC_SRC_OVER).unwrap_or_default(),
    BlendFlags: 0,
    SourceConstantAlpha: u8::MAX,
    AlphaFormat: u8::try_from(AC_SRC_ALPHA).unwrap_or_default(),
  };

  let position = POINT {
    x: frame.outer_rect.x(),
    y: frame.outer_rect.y(),
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

  canvas.fill_rect_with_corners(
    &Rect::from_ltrb(0, 0, width, height),
    style.strip_radii,
    style.background,
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

    let is_urgent = frame.tabs.get(index).is_some_and(|tab| tab.is_urgent);

    let color = if is_hovered {
      style.hover_background
    } else if is_urgent {
      style.urgent_background
    } else {
      style.inactive_background
    };

    canvas.fill_rounded_rect(&slot.pill.into(), pill_radius, color);
  }

  if let Some(pill) = state.active_pill() {
    canvas.fill_rounded_rect(
      &pill.into(),
      pill_radius,
      style.active_background,
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
) -> Vec<bool> {
  let mut truncated = vec![false; frame.tabs.len()];
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
        window_icons::icon_for(tab.hwnd, hwnd, WM_ICON_READY)
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
      // Icon-only tabs show their title as a tooltip.
      if let Some(is_truncated) = truncated.get_mut(index) {
        *is_truncated = true;
      }
      continue;
    }

    let title = if style.show_numbers {
      format!("{}. {}", index + 1, tab.title)
    } else {
      tab.title.clone()
    };

    // An empty slice's dangling pointer must never reach `DrawTextW`,
    // which reads through it on some systems (e.g. Wine).
    if title.is_empty() {
      continue;
    }

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

    let mut needed = rect;
    DrawTextW(
      dc,
      &mut text.clone(),
      &raw mut needed,
      DT_LEFT | DT_SINGLELINE | DT_NOPREFIX | DT_CALCRECT,
    );
    if let Some(is_truncated) = truncated.get_mut(index) {
      *is_truncated = needed.right > rect.right;
    }

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
  truncated
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
        &close.into(),
        close.width() / 4,
        frame.style.hover_background,
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
      color,
    );
    canvas.stroke_line(
      (right - inset, top + inset),
      (left + inset, bottom - inset),
      thickness,
      color,
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
        window_icons::invalidate(old.hwnd);
      }
    }
  }

  state.relayout();
  state.sync_tooltip_tools(hwnd);

  render(hwnd, state);

  let anchor_changed = previous.as_ref().map(|p| p.anchor)
    != state.frame.as_ref().map(|f| f.anchor);

  if let Some(anchor) =
    state.frame.as_ref().map(|frame| HWND(frame.anchor))
  {
    if restack || anchor_changed || !was_visible {
      restack_behind(hwnd, anchor);
    }
  }

  if !was_visible {
    ShowWindow(hwnd, SW_SHOWNOACTIVATE);
  }
}

/// Puts the bar directly behind `anchor` and the WM's overlays of it,
/// unless it is already there.
///
/// Going behind the overlays rather than between them and the window
/// keeps them settled, so they don't restack in turn.
///
/// # Safety
///
/// `hwnd` must be the bar's window.
unsafe fn restack_behind(hwnd: HWND, anchor: HWND) {
  window_class::match_z_band(hwnd, anchor);
  let target = window_class::insert_after_point(anchor);

  let mut prev = GetWindow(hwnd, GW_HWNDPREV);
  for _ in 0..MAX_OVERLAY_WALK {
    if prev == target {
      return;
    }
    if !OverlayKind::is_overlay(prev) {
      break;
    }
    prev = GetWindow(prev, GW_HWNDPREV);
  }

  let mut insert_after = target;
  for _ in 0..MAX_OVERLAY_WALK {
    let next = GetWindow(insert_after, GW_HWNDNEXT);
    if next == hwnd || !OverlayKind::is_overlay(next) {
      break;
    }
    insert_after = next;
  }

  let _ = SetWindowPos(
    hwnd,
    insert_after,
    0,
    0,
    0,
    0,
    SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOSENDCHANGING,
  );
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

  let (top, height) = state
    .frame
    .as_ref()
    .map_or((0, 0), |f| (f.rect.top - f.outer_rect.top, f.rect.height()));
  let y = y - top;
  let is_torn_off = y < -height || y > height * 2;

  if is_torn_off {
    (state.on_action)(TabAction::Float {
      index: drag.index,
      at_cursor: true,
    });
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
  let _ = AppendMenuW(menu, MF_STRING, MENU_FLOAT, w!("Float window"));
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
    MENU_FLOAT => (state.on_action)(TabAction::Float {
      index,
      at_cursor: false,
    }),
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
      // Posted, since a sent message would come back to this window as a
      // notification while its state is borrowed.
      let _ = PostMessageW(state.tooltip, TTM_POP, WPARAM(0), LPARAM(0));
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

/// Frees the bar's state as its window is destroyed.
///
/// # Safety
///
/// `state_ptr` must be the state allocated in `create`, freed only here.
unsafe fn on_destroy(hwnd: HWND, state_ptr: *mut BarState) {
  // Cleared first, so no stray message reaches freed state.
  SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);

  let state = Box::from_raw(state_ptr);
  if let Some(frame) = &state.frame {
    for tab in &frame.tabs {
      window_icons::invalidate(tab.hwnd);
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

  // The tooltip sends notifications from within the bar's own calls to
  // it, while the state is borrowed, so they are handled before the state
  // is borrowed here. Only `TTN_GETDISPINFOW` reads the state, and it only
  // comes from the tooltip's own timer, never from within such a call.
  if msg == WM_NOTIFY {
    // SAFETY: `WM_NOTIFY` comes with an `NMHDR`.
    let header = &*(lparam.0 as *const NMHDR);

    if header.code == TTN_GETDISPINFOW {
      // SAFETY: The state lives until `WM_DESTROY` and isn't borrowed
      // during this notification (see above). `TTN_GETDISPINFOW` comes
      // with an `NMTTDISPINFOW`.
      let state = &mut *state_ptr;
      if header.hwndFrom == state.tooltip {
        state.fill_tooltip(&mut *(lparam.0 as *mut NMTTDISPINFOW));
      }
    }

    return LRESULT(0);
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
      ShowWindow(hwnd, SW_HIDE);
      LRESULT(0)
    }
    WM_ICON_READY => {
      render(hwnd, state);
      LRESULT(0)
    }
    WM_RESTACK_TABS => {
      if let Some(anchor) = state.frame.as_ref().map(|f| HWND(f.anchor)) {
        restack_behind(hwnd, anchor);
      }
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
