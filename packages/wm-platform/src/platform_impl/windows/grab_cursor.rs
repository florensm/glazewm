//! The cursor shown during a grab: the move cursor for a move, and the
//! resize cursor of the grabbed edges for a resize.
//!
//! The app under the cursor sets the cursor on every mouse move, and only
//! a window under the cursor can set it instead. So, as `AltSnap` does, a
//! small invisible window of the grab hook's thread is kept centred on the
//! cursor for the gesture, and sets the cursor on `WM_SETCURSOR`. It has
//! no redirection bitmap and draws nothing, so it is never seen, yet it is
//! hit-tested like any window, unlike a click-through or layered one.

use std::{cell::Cell, sync::OnceLock};

use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
      DefWindowProcW, LoadCursorW, SetCursor, SetWindowPos, HCURSOR,
      HWND_TOPMOST, IDC_SIZEALL, IDC_SIZENESW, IDC_SIZENS, IDC_SIZENWSE,
      IDC_SIZEWE, SET_WINDOW_POS_FLAGS, SWP_HIDEWINDOW, SWP_NOACTIVATE,
      SWP_NOMOVE, SWP_NOREDRAW, SWP_NOSENDCHANGING, SWP_NOSIZE,
      SWP_NOZORDER, SWP_SHOWWINDOW, WM_LBUTTONDOWN, WM_MBUTTONDOWN,
      WM_RBUTTONDOWN, WM_SETCURSOR, WM_XBUTTONDOWN, WS_EX_NOACTIVATE,
      WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    },
  },
};

use crate::{
  window_class::{
    create_popup, ensure_class_registered, is_close_request,
  },
  Gesture, Point, Rect, ResizeEdges,
};

const CLASS_NAME: PCWSTR = w!("GlazeWM_GrabCursor");

/// Width and height of the window, in physical pixels. It is moved onto
/// each cursor position before the move is delivered, so any size works;
/// a margin keeps the cursor on it should the hook miss a move.
const SIZE: i32 = 256;

static CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

thread_local! {
  /// Cursor the window sets, on the thread owning it.
  static SHAPE: Cell<HCURSOR> = const { Cell::new(HCURSOR(0)) };

  /// Whether the window is shown for a gesture.
  static IS_SHOWN: Cell<bool> = const { Cell::new(false) };
}

/// The cursor a gesture shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
  Move,
  WestEast,
  NorthSouth,
  NorthWestSouthEast,
  NorthEastSouthWest,
}

impl Shape {
  fn of(gesture: Gesture) -> Self {
    let Gesture::Resize(ResizeEdges {
      left,
      right,
      top,
      bottom,
    }) = gesture
    else {
      return Self::Move;
    };

    match (left || right, top || bottom) {
      (true, true) if left == top => Self::NorthWestSouthEast,
      (true, true) => Self::NorthEastSouthWest,
      (true, false) => Self::WestEast,
      (false, true) => Self::NorthSouth,
      (false, false) => Self::Move,
    }
  }

  fn resource(self) -> PCWSTR {
    match self {
      Self::Move => IDC_SIZEALL,
      Self::WestEast => IDC_SIZEWE,
      Self::NorthSouth => IDC_SIZENS,
      Self::NorthWestSouthEast => IDC_SIZENWSE,
      Self::NorthEastSouthWest => IDC_SIZENESW,
    }
  }
}

/// The window setting the cursor during a grab. Used only on the thread
/// that created it, whose messages must be dispatched to it; the thread's
/// exit destroys it.
pub(super) struct GrabCursor {
  hwnd: HWND,
}

impl GrabCursor {
  pub(super) fn new() -> crate::Result<Self> {
    ensure_class_registered(&CLASS_REGISTERED, CLASS_NAME, wnd_proc);

    // A tool window that never activates is never managed by the WM.
    let hwnd = create_popup(
      CLASS_NAME,
      WS_EX_TOPMOST
        | WS_EX_TOOLWINDOW
        | WS_EX_NOACTIVATE
        | WS_EX_NOREDIRECTIONBITMAP,
      &Rect::from_xy(0, 0, SIZE, SIZE),
      "grab cursor",
    )?;

    Ok(Self { hwnd })
  }

  /// Shows `gesture`'s cursor, from `point` on.
  pub(super) fn show(&self, gesture: Gesture, point: &Point) {
    // SAFETY: `resource` is a system cursor, shared and never destroyed.
    let Ok(cursor) =
      (unsafe { LoadCursorW(None, Shape::of(gesture).resource()) })
    else {
      return;
    };

    SHAPE.set(cursor);
    IS_SHOWN.set(true);
    place(self.hwnd, point, HWND_TOPMOST, SWP_SHOWWINDOW);
  }

  /// Keeps the window under the cursor, about to move to `point`.
  pub(super) fn follow(&self, point: &Point) {
    if IS_SHOWN.get() {
      place(self.hwnd, point, HWND(0), SWP_NOZORDER);
    }
  }

  /// Hands the cursor back to the window under it.
  pub(super) fn hide(&self) {
    hide(self.hwnd);
  }
}

/// Centres `hwnd` on `point`.
fn place(
  hwnd: HWND,
  point: &Point,
  insert_after: HWND,
  flags: SET_WINDOW_POS_FLAGS,
) {
  // SAFETY: `hwnd` is a window of this thread; a destroyed one just fails.
  let _ = unsafe {
    SetWindowPos(
      hwnd,
      insert_after,
      point.x - SIZE / 2,
      point.y - SIZE / 2,
      SIZE,
      SIZE,
      flags | SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOREDRAW,
    )
  };
}

fn hide(hwnd: HWND) {
  if !IS_SHOWN.replace(false) {
    return;
  }

  // SAFETY: `hwnd` is a window of this thread; a destroyed one just fails.
  let _ = unsafe {
    SetWindowPos(
      hwnd,
      HWND(0),
      0,
      0,
      0,
      0,
      SWP_HIDEWINDOW
        | SWP_NOMOVE
        | SWP_NOSIZE
        | SWP_NOZORDER
        | SWP_NOACTIVATE
        | SWP_NOSENDCHANGING,
    )
  };
}

/// Sets the gesture's cursor while the cursor is over the window.
unsafe extern "system" fn wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match msg {
    WM_SETCURSOR if SHAPE.get().0 != 0 => {
      // SAFETY: The shape is a system cursor.
      unsafe { SetCursor(SHAPE.get()) };
      LRESULT(1)
    }
    // The hook takes the grab's own button, so a press here is another
    // one; hidden, should the gesture's end have been missed, it can't
    // keep taking presses around where it was left.
    WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN => {
      hide(hwnd);
      LRESULT(0)
    }
    _ if is_close_request(msg, wparam) => LRESULT(0),
    // SAFETY: Forwards this window's own message.
    _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
  }
}

#[cfg(test)]
mod tests {
  use super::Shape;
  use crate::{Gesture, ResizeEdges};

  /// The shape of a resize grabbing `edges`, a mix of `l`, `r`, `t` and
  /// `b`.
  fn resize(edges: &str) -> Shape {
    Shape::of(Gesture::Resize(ResizeEdges {
      left: edges.contains('l'),
      right: edges.contains('r'),
      top: edges.contains('t'),
      bottom: edges.contains('b'),
    }))
  }

  #[test]
  fn a_move_shows_the_move_cursor() {
    assert_eq!(Shape::of(Gesture::Move), Shape::Move);
  }

  #[test]
  fn a_resize_shows_the_cursor_of_its_edges() {
    assert_eq!(resize("l"), Shape::WestEast);
    assert_eq!(resize("r"), Shape::WestEast);
    assert_eq!(resize("t"), Shape::NorthSouth);
    assert_eq!(resize("b"), Shape::NorthSouth);
    assert_eq!(resize("lt"), Shape::NorthWestSouthEast);
    assert_eq!(resize("rb"), Shape::NorthWestSouthEast);
    assert_eq!(resize("rt"), Shape::NorthEastSouthWest);
    assert_eq!(resize("lb"), Shape::NorthEastSouthWest);
  }
}
