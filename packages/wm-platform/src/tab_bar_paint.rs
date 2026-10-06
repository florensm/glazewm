//! Draws a stack's tab bar into a bitmap: for the bar's own window, and
//! for the overview, which shows the same bar on its cards.

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::{COLORREF, HWND, RECT},
    Graphics::Gdi::{
      CreateFontW, DeleteObject, DrawTextW, SelectObject, SetBkMode,
      SetTextColor, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
      DEFAULT_CHARSET, DT_CALCRECT, DT_END_ELLIPSIS, DT_LEFT, DT_NOPREFIX,
      DT_SINGLELINE, DT_VCENTER, FW_NORMAL, FW_SEMIBOLD, HDC, HFONT,
      OUT_DEFAULT_PRECIS, TRANSPARENT,
    },
    UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL},
  },
};

use crate::{
  paint::Canvas,
  surface::Surface,
  tab_layout::{TabHit, TabLayout, TabRect},
  window_icons, Rect, TabBarStyle, TabFrame,
};

/// How a bar is shown besides its frame: its layout, and what the cursor
/// is doing on it.
pub(crate) struct TabBarView<'a> {
  pub layout: &'a TabLayout,

  /// Tab indices in the order they fill the layout's slots, e.g. with a
  /// dragged tab at the slot it would be dropped into.
  pub order: &'a [usize],

  pub hover: TabHit,

  /// Tab being dragged, drawn hovered.
  pub dragged: Option<usize>,

  /// Per tab index, whether its close button shows.
  pub close_visible: &'a [bool],
}

/// Draws `frame` as `view` shows it into `surface`, which must be the size
/// of its `outer_rect`. `notify` is posted when a tab's icon arrives.
///
/// Returns, per tab index, whether its title was cut off.
pub(crate) fn paint_tab_bar(
  surface: &mut Surface,
  frame: &TabFrame,
  view: &TabBarView,
  notify: (HWND, u32),
) -> Vec<bool> {
  let (width, height) = (surface.width, surface.height);
  paint_shapes(surface.pixels(), width, height, view, frame);

  // GDI zeroes the alpha of every pixel it draws, so it is restored after
  // drawing text and icons, which only ever land on opaque parts.
  let alpha = surface.pixels().iter().map(|p| p >> 24).collect::<Vec<_>>();

  // SAFETY: The surface's DC has its bitmap selected.
  let truncated =
    unsafe { paint_text_and_icons(surface.dc, view, frame, notify) };

  for (pixel, alpha) in surface.pixels().iter_mut().zip(alpha) {
    *pixel = (*pixel & 0x00ff_ffff) | (alpha << 24);
  }

  paint_close_buttons(surface.pixels(), width, height, view, frame);
  truncated
}

/// Highlight of the active tab.
fn active_pill(view: &TabBarView, frame: &TabFrame) -> Option<TabRect> {
  let position = view
    .order
    .iter()
    .position(|index| *index == frame.active_index)?;
  view.layout.slots.get(position).map(|slot| slot.pill)
}

/// Draws the strip and the tab highlights.
fn paint_shapes(
  pixels: &mut [u32],
  width: i32,
  height: i32,
  view: &TabBarView,
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

  for (position, index) in view.order.iter().copied().enumerate() {
    let Some(slot) = view.layout.slots.get(position) else {
      continue;
    };

    if index == frame.active_index {
      continue;
    }

    let is_hovered = matches!(
      view.hover,
      TabHit::Tab(hovered) | TabHit::Close(hovered) if hovered == index
    ) || view.dragged == Some(index);

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

  if let Some(pill) = active_pill(view, frame) {
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
  view: &TabBarView,
  frame: &TabFrame,
  notify: (HWND, u32),
) -> Vec<bool> {
  let mut truncated = vec![false; frame.tabs.len()];
  let style = &frame.style;
  let regular = create_font(style, false);
  let bold = create_font(style, true);
  let old_font = SelectObject(dc, regular);
  SetBkMode(dc, TRANSPARENT);

  for (position, index) in view.order.iter().copied().enumerate() {
    let (Some(slot), Some(tab)) =
      (view.layout.slots.get(position), frame.tabs.get(index))
    else {
      continue;
    };

    let is_active = index == frame.active_index;

    if let Some(icon_rect) = slot.icon {
      if let Some(icon) =
        window_icons::icon_for(tab.hwnd, notify.0, notify.1)
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
  view: &TabBarView,
  frame: &TabFrame,
) {
  let mut canvas = Canvas {
    pixels,
    width,
    height,
  };

  for (position, index) in view.order.iter().copied().enumerate() {
    let Some(close) =
      view.layout.slots.get(position).and_then(|slot| slot.close)
    else {
      continue;
    };

    if !view.close_visible.get(index).copied().unwrap_or(false) {
      continue;
    }

    if view.hover == TabHit::Close(index) {
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
