use std::{cell::RefCell, sync::OnceLock};

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{
      COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM,
    },
    Graphics::{
      Dwm::{
        DwmGetWindowAttribute, DwmUnregisterThumbnail,
        DWMWA_EXTENDED_FRAME_BOUNDS,
      },
      Gdi::{
        CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC,
        DeleteObject, DrawTextW, GdiFlush, ScreenToClient, SelectObject,
        SetBkMode, SetTextColor, AC_SRC_ALPHA, AC_SRC_OVER,
        ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        BLENDFUNCTION, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
        DIB_RGB_COLORS, DT_CALCRECT, DT_END_ELLIPSIS, DT_LEFT,
        DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER, FW_NORMAL, HBITMAP, HDC,
        HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS, TRANSPARENT,
      },
    },
    UI::{
      Input::KeyboardAndMouse::{
        VIRTUAL_KEY, VK_DOWN, VK_ESCAPE, VK_LEFT, VK_RETURN, VK_RIGHT,
        VK_UP,
      },
      WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DrawIconEx,
        GetCursorPos, GetWindowLongPtrW, GetWindowRect, LoadCursorW,
        PostMessageW, RegisterClassW, SetForegroundWindow,
        SetWindowLongPtrW, SetWindowPos, ShowWindow, UpdateLayeredWindow,
        CREATESTRUCTW, DI_NORMAL, GWLP_USERDATA, HICON, HWND_TOPMOST,
        IDC_ARROW, SC_KEYMENU, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE,
        ULW_ALPHA, WA_INACTIVE, WM_ACTIVATE, WM_APP, WM_CLOSE, WM_CREATE,
        WM_DESTROY, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP,
        WM_MOUSEMOVE, WM_SYSCOMMAND, WNDCLASSW, WS_EX_LAYERED,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
      },
    },
  },
};

use crate::{
  companion::Companion,
  native_surrogate::register_thumbnail,
  overview_layout::{
    label_layout, OverviewAction, OverviewLayout, OverviewLayoutParams,
  },
  paint::{self, Canvas},
  platform_impl, window_icons, Color, Direction, Dispatcher, Rect,
};

/// Posted with a `Box<(u64, OverviewFrame)>` in `WPARAM` to open the
/// overview as that session.
const WM_OPEN_OVERVIEW: u32 = WM_APP + 1;

/// Posted with a `Box<OverviewFrame>` in `WPARAM` to change what an open
/// overview shows.
const WM_UPDATE_OVERVIEW: u32 = WM_APP + 2;

/// Posted to close the overview.
const WM_HIDE_OVERVIEW: u32 = WM_APP + 3;

/// Posted by the icon thread once an icon is cached.
const WM_ICON_READY: u32 = WM_APP + 4;

/// Shown for a window whose frame can't be read, e.g. one being
/// destroyed.
const FALLBACK_WINDOW_SIZE: (i32, i32) = (1600, 1000);

static CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// A window shown in the overview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewItem {
  /// Handle of the window, whose live thumbnail is shown.
  pub hwnd: isize,
  pub title: String,
}

/// Look of the overview, in physical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct OverviewStyle {
  /// Scrim covering the overview's area.
  pub background: Color,

  /// Highlight behind the selected preview.
  pub selection: Color,

  /// Ring around the focused window's thumbnail.
  pub focused_border: Color,

  pub text: Color,
  pub font_family: String,
  pub font_size: i32,

  /// Space around and between previews.
  pub gap: i32,

  /// Scale factor of the overview's monitor, for its fixed sizes.
  pub scale_factor: f32,
}

/// Everything shown by the overview, posted to its thread as a whole.
#[derive(Clone, Debug, PartialEq)]
pub struct OverviewFrame {
  /// Area covered, e.g. the working area of a monitor.
  pub rect: Rect,

  /// Windows shown, in display order.
  pub items: Vec<OverviewItem>,

  /// Index in `items` of the focused window, which is selected on open.
  pub focused_index: Option<usize>,

  pub style: OverviewStyle,
}

/// Where the left button went down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Press {
  Preview(usize),
  Background,
}

impl From<Option<usize>> for Press {
  fn from(hit: Option<usize>) -> Self {
    hit.map_or(Self::Background, Self::Preview)
  }
}

/// Sizes derived from the style, in physical pixels.
struct Metrics {
  padding: i32,
  corner_radius: i32,
  ring_width: i32,
  ring_gap: i32,
  title_height: i32,
  icon_size: i32,
  icon_spacing: i32,
}

impl Metrics {
  fn new(style: &OverviewStyle) -> Self {
    #[allow(clippy::cast_possible_truncation)]
    let px = |length: f32| (length * style.scale_factor).round() as i32;
    let font_size = style.font_size.max(1);

    Self {
      padding: px(10.0),
      corner_radius: px(10.0),
      ring_width: px(3.0).max(1),
      ring_gap: px(2.0),
      title_height: font_size * 2,
      icon_size: font_size * 3 / 2,
      icon_spacing: px(6.0),
    }
  }
}

/// State of the overview, owned by its window on the event-loop thread.
struct OverviewState {
  /// What is shown, or `None` while hidden.
  frame: Option<OverviewFrame>,

  /// Session of the shown frame, which actions are tagged with.
  session: u64,

  layout: OverviewLayout,

  /// Index of the preview that `Enter` picks.
  selected: usize,

  press: Option<Press>,

  /// Last cursor position seen, in client coordinates. Windows sends a
  /// mouse move when a window appears under the cursor, which must not
  /// move the selection.
  cursor: (i32, i32),

  /// Set once a pick or cancel is sent, after which input is ignored
  /// until the WM closes the overview.
  is_done: bool,

  /// Visible frame of each item, in its window's own coordinates, as of
  /// the last layout.
  window_frames: Vec<RECT>,

  /// DWM thumbnail handles, one per item, 0 where registration failed.
  thumbnails: Vec<isize>,

  /// Bitmap the overview is drawn into, kept while shown.
  surface: Option<Surface>,

  on_action: Box<dyn Fn(u64, OverviewAction) + Send + 'static>,
}

/// The window overview: a full-screen layered popup on the event-loop
/// thread, which handles its input and draws it, with a live DWM
/// thumbnail per window composited over it.
///
/// The WM thread only ever posts to it, since a synchronous call could
/// deadlock with the event loop waiting on the WM.
pub struct NativeOverview {
  hwnd: isize,

  /// Frame last posted, to skip posting identical ones. `None` while
  /// hidden.
  last_frame: Option<OverviewFrame>,

  /// Incremented on each open, so actions from an earlier session can be
  /// told apart.
  session: u64,
}

// SAFETY: Only the raw handle is kept, used to post messages, which is
// allowed from any thread.
unsafe impl Send for NativeOverview {}

impl NativeOverview {
  /// Creates the hidden overview. `on_action` is called on the event-loop
  /// thread for each user action, with the session it happened in.
  pub fn create(
    dispatcher: &Dispatcher,
    on_action: Box<dyn Fn(u64, OverviewAction) + Send + 'static>,
  ) -> crate::Result<Self> {
    let state = Box::new(RefCell::new(OverviewState {
      frame: None,
      session: 0,
      layout: OverviewLayout::default(),
      selected: 0,
      press: None,
      cursor: (0, 0),
      is_done: false,
      window_frames: Vec::new(),
      thumbnails: Vec::new(),
      surface: None,
      on_action,
    }));

    // Passed as an integer so the closure is `Send`.
    let state_ptr = Box::into_raw(state) as usize;

    let hwnd =
      dispatcher.dispatch_sync(move || -> crate::Result<isize> {
        ensure_class_registered();

        // SAFETY: The class is registered above. `state_ptr` is owned by
        // the window from here on and freed in `WM_DESTROY`.
        let hwnd = unsafe {
          CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            w!("GlazeWM_Overview"),
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
          unsafe {
            drop(Box::from_raw(state_ptr as *mut RefCell<OverviewState>));
          }
          return Err(crate::Error::Platform(
            "Failed to create overview window.".to_string(),
          ));
        }

        Ok(hwnd.0)
      })??;

    Ok(Self {
      hwnd,
      last_frame: None,
      session: 0,
    })
  }

  /// Opens the overview with `frame`, taking keyboard focus, or changes
  /// what it shows if it is open already.
  ///
  /// Must be called on the WM thread, not the event loop thread.
  pub fn show(&mut self, frame: OverviewFrame) {
    if self.last_frame.as_ref() == Some(&frame) {
      return;
    }

    let is_opening = self.last_frame.is_none();
    self.last_frame = Some(frame.clone());

    let (msg, payload) = if is_opening {
      self.session += 1;

      // Lets the overview take the foreground once it is shown.
      platform_impl::send_foreground_input();

      let payload = Box::new((self.session, frame));
      (WM_OPEN_OVERVIEW, Box::into_raw(payload) as usize)
    } else {
      (WM_UPDATE_OVERVIEW, Box::into_raw(Box::new(frame)) as usize)
    };

    // SAFETY: Ownership of the payload passes to the window procedure,
    // which frees it. If the post fails the window is gone and it leaks.
    unsafe {
      let _ =
        PostMessageW(HWND(self.hwnd), msg, WPARAM(payload), LPARAM(0));
    }
  }

  /// Closes the overview.
  pub fn hide(&mut self) {
    if self.last_frame.take().is_none() {
      return;
    }

    // SAFETY: Posting to a destroyed window just fails.
    unsafe {
      let _ = PostMessageW(
        HWND(self.hwnd),
        WM_HIDE_OVERVIEW,
        WPARAM(0),
        LPARAM(0),
      );
    }
  }

  /// Session of the last open, which the actions of that open carry.
  #[must_use]
  pub fn session(&self) -> u64 {
    self.session
  }
}

impl Drop for NativeOverview {
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
      lpszClassName: w!("GlazeWM_Overview"),
      lpfnWndProc: Some(wnd_proc),
      // SAFETY: `IDC_ARROW` is a system cursor.
      hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.unwrap_or_default(),
      ..Default::default()
    };

    // SAFETY: `class` is fully initialized with a static class name.
    unsafe { RegisterClassW(&raw const class) };
  });
}

/// A 32-bit top-down DIB section selected into its own memory DC.
struct Surface {
  dc: HDC,
  bitmap: HBITMAP,
  old_bitmap: HGDIOBJ,
  bits: *mut u32,
  width: i32,
  height: i32,
}

impl Surface {
  fn new(width: i32, height: i32) -> Option<Self> {
    if width <= 0 || height <= 0 {
      return None;
    }

    let info = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: u32::try_from(std::mem::size_of::<BITMAPINFOHEADER>())
          .ok()?,
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

    // SAFETY: `info` describes a valid 32-bit DIB and outlives the calls.
    // The DC and bitmap are released in `Drop`, or below on failure.
    unsafe {
      let dc = CreateCompatibleDC(None);
      if dc.is_invalid() {
        return None;
      }

      let mut bits = std::ptr::null_mut();
      let Ok(bitmap) = CreateDIBSection(
        dc,
        &raw const info,
        DIB_RGB_COLORS,
        &raw mut bits,
        None,
        0,
      ) else {
        let _ = DeleteDC(dc);
        return None;
      };

      Some(Self {
        dc,
        bitmap,
        old_bitmap: SelectObject(dc, bitmap),
        bits: bits.cast(),
        width,
        height,
      })
    }
  }

  fn pixels(&mut self) -> &mut [u32] {
    let len = usize::try_from(self.width * self.height).unwrap_or(0);

    // SAFETY: The DIB section holds `width * height` 32-bit pixels, owned
    // by `bitmap` until `Drop`, and GDI has finished drawing into it.
    unsafe {
      let _ = GdiFlush();
      std::slice::from_raw_parts_mut(self.bits, len)
    }
  }

  fn canvas(&mut self) -> Canvas<'_> {
    let (width, height) = (self.width, self.height);
    Canvas {
      pixels: self.pixels(),
      width,
      height,
    }
  }
}

impl Drop for Surface {
  fn drop(&mut self) {
    // SAFETY: Both handles were created in `new` and are released once.
    unsafe {
      SelectObject(self.dc, self.old_bitmap);
      let _ = DeleteObject(self.bitmap);
      let _ = DeleteDC(self.dc);
    }
  }
}

impl OverviewState {
  fn relayout(&mut self) {
    let Some(frame) = &self.frame else {
      return;
    };

    let metrics = Metrics::new(&frame.style);
    self.window_frames = frame
      .items
      .iter()
      .map(|item| {
        visible_frame(HWND(item.hwnd)).unwrap_or(RECT {
          right: FALLBACK_WINDOW_SIZE.0,
          bottom: FALLBACK_WINDOW_SIZE.1,
          ..Default::default()
        })
      })
      .collect();

    let window_sizes = self
      .window_frames
      .iter()
      .map(|rect| (rect.right - rect.left, rect.bottom - rect.top))
      .collect::<Vec<_>>();

    self.layout = OverviewLayout::new(&OverviewLayoutParams {
      width: frame.rect.width(),
      height: frame.rect.height(),
      gap: frame.style.gap,
      padding: metrics.padding,
      title_height: metrics.title_height,
      window_sizes: &window_sizes,
    });
  }

  /// Sends `action` to the WM, once per session for a pick or cancel.
  fn send(&mut self, action: OverviewAction) {
    if self.is_done {
      return;
    }

    self.is_done = !matches!(action, OverviewAction::Deactivated);
    (self.on_action)(self.session, action);
  }

  fn pick_selected(&mut self) {
    let hwnd = self
      .frame
      .as_ref()
      .and_then(|frame| frame.items.get(self.selected))
      .map(|item| item.hwnd);

    if let Some(hwnd) = hwnd {
      self.send(OverviewAction::Pick(hwnd));
    }
  }

  /// Replaces every thumbnail on the overview `hwnd` with one per item,
  /// at its current cell.
  fn sync_thumbnails(&mut self, hwnd: HWND) {
    self.unregister_thumbnails();

    let Some(frame) = &self.frame else {
      return;
    };

    self.thumbnails = frame
      .items
      .iter()
      .zip(&self.window_frames)
      .zip(&self.layout.cells)
      .map(|((item, window_frame), cell)| {
        let source = HWND(item.hwnd);

        register_thumbnail(
          hwnd,
          source,
          Companion::find(source),
          *window_frame,
          to_rect(&cell.thumbnail),
          u8::MAX,
        )
        .unwrap_or(0)
      })
      .collect();
  }

  fn unregister_thumbnails(&mut self) {
    for thumbnail in self.thumbnails.drain(..) {
      if thumbnail != 0 {
        // SAFETY: Registered by this overview and not yet unregistered.
        unsafe {
          let _ = DwmUnregisterThumbnail(thumbnail);
        }
      }
    }
  }
}

/// `hwnd`'s visible frame, without its invisible resize borders, in the
/// window's own coordinates.
fn visible_frame(hwnd: HWND) -> Option<RECT> {
  let mut bounds = RECT::default();
  let mut frame = RECT::default();

  // SAFETY: Both rects outlive the calls and match the queried sizes; a
  // stale handle just makes them fail.
  unsafe {
    GetWindowRect(hwnd, &raw mut bounds).ok()?;
    DwmGetWindowAttribute(
      hwnd,
      DWMWA_EXTENDED_FRAME_BOUNDS,
      std::ptr::from_mut(&mut frame).cast(),
      u32::try_from(std::mem::size_of::<RECT>()).ok()?,
    )
    .ok()?;
  }

  let visible = RECT {
    left: frame.left - bounds.left,
    top: frame.top - bounds.top,
    right: frame.right - bounds.left,
    bottom: frame.bottom - bounds.top,
  };

  (visible.right > visible.left && visible.bottom > visible.top)
    .then_some(visible)
}

fn to_rect(rect: &Rect) -> RECT {
  RECT {
    left: rect.left,
    top: rect.top,
    right: rect.right,
    bottom: rect.bottom,
  }
}

/// Draws the overview and shows the result at its frame's rect.
///
/// # Safety
///
/// `hwnd` must be the overview's window, called on its thread.
unsafe fn render(hwnd: HWND, state: &mut OverviewState) {
  let Some(frame) = state.frame.as_ref() else {
    return;
  };

  let (width, height) = (frame.rect.width(), frame.rect.height());
  let is_reusable = state.surface.as_ref().is_some_and(|surface| {
    (surface.width, surface.height) == (width, height)
  });

  if !is_reusable {
    state.surface = Surface::new(width, height);
  }

  let Some(surface) = state.surface.as_mut() else {
    tracing::warn!("Failed to allocate the overview's bitmap.");
    return;
  };

  paint_shapes(
    &mut surface.canvas(),
    frame,
    &state.layout,
    state.selected,
  );
  paint_labels(hwnd, surface, frame, &state.layout);

  let blend = BLENDFUNCTION {
    BlendOp: u8::try_from(AC_SRC_OVER).unwrap_or_default(),
    BlendFlags: 0,
    SourceConstantAlpha: u8::MAX,
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
    surface.dc,
    Some(&raw const source),
    COLORREF(0),
    Some(&raw const blend),
    ULW_ALPHA,
  ) {
    tracing::warn!("Failed to draw the overview: {err}");
  }
}

/// Draws the scrim, the selection highlight and the focused window's
/// ring.
fn paint_shapes(
  canvas: &mut Canvas<'_>,
  frame: &OverviewFrame,
  layout: &OverviewLayout,
  selected: usize,
) {
  let style = &frame.style;
  let metrics = Metrics::new(style);

  // Fully transparent pixels of a layered window let clicks through.
  canvas.fill(Color {
    a: style.background.a.max(1),
    ..style.background
  });

  if let Some(cell) = layout.cells.get(selected) {
    canvas.fill_rounded_rect(
      &cell.cell,
      metrics.corner_radius,
      style.selection,
    );
  }

  if let Some(cell) = frame
    .focused_index
    .and_then(|index| layout.cells.get(index))
  {
    let outset = metrics.ring_width + metrics.ring_gap;
    canvas.stroke_rounded_rect(
      &cell.thumbnail.inset(-outset),
      metrics.corner_radius,
      metrics.ring_width,
      style.focused_border,
    );
  }
}

/// Draws each preview's icon and title under its thumbnail.
///
/// GDI can't draw onto translucent pixels, so both are drawn opaquely
/// into a scratch bitmap first, then blended into `surface`.
///
/// # Safety
///
/// `hwnd` must be the overview's window, called on its thread.
unsafe fn paint_labels(
  hwnd: HWND,
  surface: &mut Surface,
  frame: &OverviewFrame,
  layout: &OverviewLayout,
) {
  let style = &frame.style;
  let metrics = Metrics::new(style);
  let font = create_font(style);

  for (item, cell) in frame.items.iter().zip(&layout.cells) {
    let icon = window_icons::icon_for(item.hwnd, hwnd, WM_ICON_READY);
    let mut text = item.title.encode_utf16().collect::<Vec<_>>();

    let label = label_layout(
      &cell.title,
      icon.map(|_| metrics.icon_size),
      metrics.icon_spacing,
      text_width(surface.dc, font, &mut text),
    );

    if let (Some(icon), Some(rect)) = (icon, &label.icon) {
      if let Some(pixels) = draw_icon(icon, rect.width()) {
        surface.canvas().draw_premultiplied(
          rect.left,
          rect.top,
          &pixels,
          rect.width(),
        );
      }
    }

    if let Some(mask) = draw_text_mask(font, &mut text, &label.text) {
      surface.canvas().blend_mask(
        label.text.left,
        label.text.top,
        &mask,
        label.text.width(),
        style.text,
      );
    }
  }

  let _ = DeleteObject(font);
}

/// Creates the title font, grayscale anti-aliased since its coverage is
/// used as a mask.
///
/// # Safety
///
/// The returned font must be deleted by the caller.
unsafe fn create_font(style: &OverviewStyle) -> HFONT {
  let family = style
    .font_family
    .encode_utf16()
    .chain(std::iter::once(0))
    .collect::<Vec<_>>();

  CreateFontW(
    // Negative: character height rather than cell height.
    -style.font_size.max(1),
    0,
    0,
    0,
    i32::try_from(FW_NORMAL.0).unwrap_or(400),
    0,
    0,
    0,
    u32::from(DEFAULT_CHARSET.0),
    u32::from(OUT_DEFAULT_PRECIS.0),
    u32::from(CLIP_DEFAULT_PRECIS.0),
    u32::from(ANTIALIASED_QUALITY.0),
    0,
    PCWSTR(family.as_ptr()),
  )
}

/// Width of `text` in `font`, measured on `dc`.
///
/// # Safety
///
/// `dc` and `font` must be valid.
unsafe fn text_width(dc: HDC, font: HFONT, text: &mut [u16]) -> i32 {
  // An empty slice's dangling pointer must never reach `DrawTextW`, which
  // reads through it on some systems (e.g. Wine).
  if text.is_empty() {
    return 0;
  }

  let old_font = SelectObject(dc, font);
  let mut rect = RECT::default();
  DrawTextW(
    dc,
    text,
    &raw mut rect,
    DT_LEFT | DT_SINGLELINE | DT_NOPREFIX | DT_CALCRECT,
  );
  SelectObject(dc, old_font);

  rect.right - rect.left
}

/// Coverage mask of `text` drawn into a `rect`-sized area, cut off with
/// an ellipsis if it doesn't fit.
///
/// # Safety
///
/// `font` must be valid.
unsafe fn draw_text_mask(
  font: HFONT,
  text: &mut [u16],
  rect: &Rect,
) -> Option<Vec<u8>> {
  if text.is_empty() {
    return None;
  }

  let mut scratch = Surface::new(rect.width(), rect.height())?;
  let old_font = SelectObject(scratch.dc, font);
  SetBkMode(scratch.dc, TRANSPARENT);
  SetTextColor(scratch.dc, COLORREF(0x00ff_ffff));

  let mut bounds = RECT {
    right: rect.width(),
    bottom: rect.height(),
    ..Default::default()
  };
  DrawTextW(
    scratch.dc,
    text,
    &raw mut bounds,
    DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX,
  );
  SelectObject(scratch.dc, old_font);

  Some(paint::mask_from_white_on_black(scratch.pixels()))
}

/// Premultiplied pixels of `icon` drawn at `size` x `size`.
///
/// # Safety
///
/// `icon` must be valid.
unsafe fn draw_icon(icon: HICON, size: i32) -> Option<Vec<u32>> {
  let draw_on = |background: u32| -> Option<Vec<u32>> {
    let mut scratch = Surface::new(size, size)?;
    scratch.pixels().fill(background);
    DrawIconEx(scratch.dc, 0, 0, icon, size, size, 0, None, DI_NORMAL)
      .ok()?;
    Some(scratch.pixels().to_vec())
  };

  let on_black = draw_on(0)?;
  let on_white = draw_on(0x00ff_ffff)?;
  Some(paint::matte_from_backgrounds(&on_black, &on_white))
}

/// Signed client coordinates packed into a mouse message's `LPARAM`.
fn mouse_position(lparam: LPARAM) -> (i32, i32) {
  #[allow(clippy::cast_possible_truncation)]
  let (x, y) = (lparam.0 as i16, (lparam.0 >> 16) as i16);
  (i32::from(x), i32::from(y))
}

/// Cursor position in `hwnd`'s client coordinates.
///
/// # Safety
///
/// `hwnd` must be valid.
unsafe fn cursor_in_client(hwnd: HWND) -> (i32, i32) {
  let mut point = POINT::default();
  if GetCursorPos(&raw mut point).is_ok() {
    let _ = ScreenToClient(hwnd, &raw mut point);
  }
  (point.x, point.y)
}

/// Shows `frame` as session `session`, and takes the foreground.
///
/// # Safety
///
/// `hwnd` must be the overview's window, called on its thread.
unsafe fn open(
  hwnd: HWND,
  state: &mut OverviewState,
  session: u64,
  frame: OverviewFrame,
) {
  close(hwnd, state);

  let rect = frame.rect.clone();
  state.selected = frame
    .focused_index
    .unwrap_or(0)
    .min(frame.items.len().saturating_sub(1));
  state.frame = Some(frame);
  state.session = session;
  state.press = None;
  state.is_done = false;
  state.relayout();

  // Drawn and given its thumbnails while still hidden, so it all appears
  // in one frame.
  render(hwnd, state);
  state.sync_thumbnails(hwnd);

  if let Err(err) = SetWindowPos(
    hwnd,
    HWND_TOPMOST,
    rect.x(),
    rect.y(),
    rect.width(),
    rect.height(),
    SWP_NOACTIVATE | SWP_SHOWWINDOW,
  ) {
    tracing::warn!("Failed to show the overview: {err}");
  }

  state.cursor = cursor_in_client(hwnd);

  if !SetForegroundWindow(hwnd).as_bool() {
    tracing::warn!("Overview could not take the foreground.");
  }
}

/// Shows `frame` in place of the current one, keeping the selected window
/// selected. Ignored once closed, e.g. by losing the foreground.
///
/// # Safety
///
/// `hwnd` must be the overview's window, called on its thread.
unsafe fn update(
  hwnd: HWND,
  state: &mut OverviewState,
  frame: OverviewFrame,
) {
  if state.frame.is_none() {
    return;
  }

  let Some(previous) = state.frame.replace(frame) else {
    return;
  };

  let selected_hwnd = previous.items.get(state.selected).map(|i| i.hwnd);
  let Some(current) = &state.frame else {
    return;
  };

  for item in &previous.items {
    if !current.items.contains(item) {
      window_icons::invalidate(item.hwnd);
    }
  }

  state.selected = current
    .items
    .iter()
    .position(|item| Some(item.hwnd) == selected_hwnd)
    .unwrap_or(state.selected)
    .min(current.items.len().saturating_sub(1));
  state.press = None;

  state.relayout();
  render(hwnd, state);
  state.sync_thumbnails(hwnd);
}

/// Hides the overview and frees what it held while shown.
///
/// # Safety
///
/// `hwnd` must be the overview's window, called on its thread.
unsafe fn close(hwnd: HWND, state: &mut OverviewState) {
  // Taken first: hiding the window deactivates it, which must not count as
  // losing the foreground.
  let Some(frame) = state.frame.take() else {
    return;
  };

  ShowWindow(hwnd, SW_HIDE);
  state.unregister_thumbnails();
  state.surface = None;
  state.layout = OverviewLayout::default();

  for item in &frame.items {
    window_icons::invalidate(item.hwnd);
  }
}

/// Moves the selection to the preview under the cursor.
///
/// # Safety
///
/// `hwnd` must be the overview's window, called on its thread.
unsafe fn on_mouse_move(
  hwnd: HWND,
  state: &mut OverviewState,
  position: (i32, i32),
) {
  if position == state.cursor {
    return;
  }
  state.cursor = position;

  if let Some(index) = state.layout.hit_test(position.0, position.1) {
    if index != state.selected {
      state.selected = index;
      render(hwnd, state);
    }
  }
}

/// Picks the preview, or cancels on the background, released over where
/// the left button went down.
fn on_left_button_up(state: &mut OverviewState, (x, y): (i32, i32)) {
  let press = state.press.take();

  if press != Some(Press::from(state.layout.hit_test(x, y))) {
    return;
  }

  match press {
    Some(Press::Preview(index)) => {
      state.selected = index;
      state.pick_selected();
    }
    Some(Press::Background) => state.send(OverviewAction::Cancel),
    None => {}
  }
}

/// Moves the selection with the arrow keys, picks it with `Enter`, and
/// cancels with `Escape`.
///
/// # Safety
///
/// `hwnd` must be the overview's window, called on its thread.
unsafe fn on_key_down(
  hwnd: HWND,
  state: &mut OverviewState,
  key: VIRTUAL_KEY,
) {
  let direction = match key {
    VK_LEFT => Direction::Left,
    VK_RIGHT => Direction::Right,
    VK_UP => Direction::Up,
    VK_DOWN => Direction::Down,
    VK_RETURN => return state.pick_selected(),
    VK_ESCAPE => return state.send(OverviewAction::Cancel),
    _ => return,
  };

  let selected = state.layout.move_selection(state.selected, &direction);
  if selected != state.selected {
    state.selected = selected;
    render(hwnd, state);
  }
}

/// Window procedure of the overview.
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

  let state_ptr =
    GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut RefCell<OverviewState>;
  if state_ptr.is_null() {
    return DefWindowProcW(hwnd, msg, wparam, lparam);
  }

  if msg == WM_DESTROY {
    // Cleared first, so no stray message reaches freed state.
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);

    // SAFETY: Allocated in `create` and freed only here.
    let state = Box::from_raw(state_ptr);
    if let Ok(mut state) = state.try_borrow_mut() {
      state.unregister_thumbnails();
    }
    return LRESULT(0);
  }

  // SAFETY: The state lives until `WM_DESTROY` and is only touched on this
  // thread.
  let cell = &*state_ptr;

  // Showing, hiding, moving and activating the window send it messages
  // while its state is borrowed; those get the default handling.
  let Ok(mut state) = cell.try_borrow_mut() else {
    return DefWindowProcW(hwnd, msg, wparam, lparam);
  };
  let state = &mut *state;
  let is_open = state.frame.is_some() && !state.is_done;

  match msg {
    WM_OPEN_OVERVIEW => {
      // SAFETY: Posted by `show` with an owned, leaked payload.
      let payload = Box::from_raw(wparam.0 as *mut (u64, OverviewFrame));
      let (session, frame) = *payload;
      open(hwnd, state, session, frame);
      LRESULT(0)
    }
    WM_UPDATE_OVERVIEW => {
      // SAFETY: Posted by `show` with an owned, leaked frame.
      let frame = Box::from_raw(wparam.0 as *mut OverviewFrame);
      update(hwnd, state, *frame);
      LRESULT(0)
    }
    WM_HIDE_OVERVIEW => {
      close(hwnd, state);
      LRESULT(0)
    }
    WM_ICON_READY => {
      render(hwnd, state);
      LRESULT(0)
    }
    WM_ACTIVATE => {
      // The low word of `wparam` is the activation state.
      if (wparam.0 & 0xffff) == WA_INACTIVE as usize
        && state.frame.is_some()
      {
        close(hwnd, state);
        state.send(OverviewAction::Deactivated);
      }
      LRESULT(0)
    }
    // Releasing the Alt key of the binding that opened the overview would
    // otherwise enter menu mode, swallowing the next key press.
    WM_SYSCOMMAND if (wparam.0 & 0xfff0) == SC_KEYMENU as usize => {
      LRESULT(0)
    }
    WM_MOUSEMOVE if is_open => {
      on_mouse_move(hwnd, state, mouse_position(lparam));
      LRESULT(0)
    }
    WM_LBUTTONDOWN if is_open => {
      let (x, y) = mouse_position(lparam);
      state.press = Some(Press::from(state.layout.hit_test(x, y)));
      LRESULT(0)
    }
    WM_LBUTTONUP if is_open => {
      on_left_button_up(state, mouse_position(lparam));
      LRESULT(0)
    }
    WM_KEYDOWN if is_open => {
      #[allow(clippy::cast_possible_truncation)]
      on_key_down(hwnd, state, VIRTUAL_KEY(wparam.0 as u16));
      LRESULT(0)
    }
    WM_CLOSE => {
      let _ = DestroyWindow(hwnd);
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, msg, wparam, lparam),
  }
}
