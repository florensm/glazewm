use wm_platform::{Rect, HWND};

/// Where a window is drawn while it moves, for what is drawn along with
/// it, such as its stack's tab bar.
pub struct StandIn {
  /// The layout rect asked about, moved along with the window.
  pub rect: Rect,
  /// Area the window is cut off at.
  pub clip: Option<Rect>,
  /// The window drawing it in place of the real one (a peek or a
  /// surrogate), which what goes along with it sits behind.
  pub anchor: Option<HWND>,
}

/// `rect` moved edge by edge as far as a window drawn at `current` is from
/// its `target`, since a window can also resize as it moves.
pub fn moved_along(rect: &Rect, target: &Rect, current: &Rect) -> Rect {
  Rect::from_ltrb(
    rect.left + current.left - target.left,
    rect.top + current.top - target.top,
    rect.right + current.right - target.right,
    rect.bottom + current.bottom - target.bottom,
  )
}

#[cfg(test)]
mod tests {
  use wm_platform::Rect;

  use super::moved_along;

  #[test]
  fn rect_moves_and_resizes_with_its_window() {
    let stack = Rect::from_ltrb(100, 0, 500, 400);
    let window = Rect::from_ltrb(100, 28, 500, 400);

    let slid = Rect::from_ltrb(-200, 28, 200, 400);
    assert_eq!(
      moved_along(&stack, &window, &slid),
      Rect::from_ltrb(-200, 0, 200, 400)
    );

    let narrowed = Rect::from_ltrb(150, 28, 400, 400);
    assert_eq!(
      moved_along(&stack, &window, &narrowed),
      Rect::from_ltrb(150, 0, 400, 400)
    );
  }
}
