//! Draws a stack's tab bar into a bitmap: for the bar's own window, and
//! for the overview, which shows the same bar on its cards.

use windows::Win32::Foundation::HWND;

use crate::{
  paint::Canvas,
  surface::Surface,
  tab_layout::{TabHit, TabLayout, TabRect},
  text::{draw_text, text_width, Align, Fonts},
  window_icons::draw_window_icon,
  Rect, TabFrame,
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
  let truncated = paint_text_and_icons(surface, view, frame, notify);
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

/// Draws titles and icons.
fn paint_text_and_icons(
  surface: &mut Surface,
  view: &TabBarView,
  frame: &TabFrame,
  notify: (HWND, u32),
) -> Vec<bool> {
  let mut truncated = vec![false; frame.tabs.len()];
  let style = &frame.style;
  let mut fonts = Fonts::new(&style.font_family);

  #[allow(clippy::cast_precision_loss)]
  let font_size = style.font_size as f32;

  for (position, index) in view.order.iter().copied().enumerate() {
    let (Some(slot), Some(tab)) =
      (view.layout.slots.get(position), frame.tabs.get(index))
    else {
      continue;
    };

    let is_active = index == frame.active_index;

    if let Some(icon_rect) = slot.icon {
      draw_window_icon(surface, tab.hwnd, &icon_rect.into(), 1.0, notify);
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

    let font = fonts.get(font_size, is_active);
    if let Some(is_truncated) = truncated.get_mut(index) {
      *is_truncated = text_width(&font, &title) > slot.text.width();
    }

    let color = if is_active {
      style.text
    } else {
      style.inactive_text
    };
    draw_text(
      surface,
      &font,
      &title,
      &slot.text.into(),
      color,
      Align::Left,
    );
  }

  truncated
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

#[cfg(test)]
mod tests {
  use windows::Win32::Foundation::HWND;

  use super::{paint_tab_bar, TabBarView};
  use crate::{
    surface::Surface,
    tab_layout::{TabHit, TabLayout},
    Color, CornerRadii, Rect, TabBarStyle, TabCloseMode, TabFrame,
    TabInfo,
  };

  #[test]
  fn flags_titles_cut_off() {
    let color = Color {
      r: 255,
      g: 255,
      b: 255,
      a: 255,
    };
    let tab = |title: String| TabInfo {
      title,
      hwnd: 0,
      is_urgent: false,
    };
    let frame = TabFrame {
      rect: Rect::from_ltrb(0, 0, 300, 28),
      outer_rect: Rect::from_ltrb(0, 0, 300, 28),
      clip: None,
      tabs: vec![tab("Notes".to_string()), tab("Long title ".repeat(10))],
      active_index: 0,
      anchor: 0,
      style: TabBarStyle {
        background: color,
        active_background: color,
        hover_background: color,
        inactive_background: color,
        urgent_background: color,
        text: color,
        inactive_text: color,
        font_family: "Segoe UI".to_string(),
        font_size: 12,
        corner_radius: 0,
        strip_radii: CornerRadii::default(),
        min_tab_width: 0,
        max_tab_width: 0,
        show_icons: false,
        show_numbers: false,
        close_button: TabCloseMode::Never,
        scale_factor: 1.0,
      },
    };
    let layout = TabLayout::for_frame(&frame);
    let view = TabBarView {
      layout: &layout,
      order: &[0, 1],
      hover: TabHit::Empty,
      dragged: None,
      close_visible: &[false, false],
    };

    let mut surface = Surface::new(300, 28).expect("surface");
    let truncated =
      paint_tab_bar(&mut surface, &frame, &view, (HWND::default(), 0));
    assert_eq!(truncated, [false, true]);
  }
}
