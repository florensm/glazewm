//! Where the exposé spreads a workspace's windows: as one cluster,
//! each about where it sits on screen.

use crate::overview::layout::RectF;

/// Rounds of pushing apart and rescaling, for the gap to settle at the
/// scale the cluster ends up at.
const ROUNDS: usize = 6;

/// Spreads windows whose screen rects are `windows` over `area` as one
/// cluster, centered: all scaled alike, each where it sits relative to
/// the others, pushed apart only where they'd be under `gap` apart.
///
/// The cluster takes at most `largest` of `area` either way, and shows no
/// window larger than it is. Returns a rect per window, in the order
/// given.
pub(crate) fn spread(
  windows: &[RectF],
  area: &RectF,
  gap: f32,
  largest: f32,
) -> Vec<RectF> {
  if windows.is_empty() {
    return Vec::new();
  }

  let mut rects = windows
    .iter()
    .map(|window| {
      RectF::new(window.x, window.y, window.w.max(1.0), window.h.max(1.0))
    })
    .collect::<Vec<_>>();

  // The gap is in `area`'s pixels, so it's kept in screen pixels at the
  // scale the cluster ends up at, which pushing apart lowers.
  let mut scale = fit_scale(&rects, area, largest);
  for _ in 0..ROUNDS {
    separate(&mut rects, gap / scale.max(f32::EPSILON));
    let next = fit_scale(&rects, area, largest);
    let settled = (next - scale).abs() <= scale * 0.01;
    scale = next;
    if settled {
      break;
    }
  }

  let (bx, by) = bounds(&rects).center();
  let (ax, ay) = area.center();
  rects
    .iter()
    .map(|rect| {
      let (cx, cy) = rect.center();
      let (w, h) = (rect.w * scale, rect.h * scale);
      RectF::new(
        ax + (cx - bx) * scale - w / 2.0,
        ay + (cy - by) * scale - h / 2.0,
        w,
        h,
      )
    })
    .collect()
}

/// Pushes overlapping `rects` apart, each pair along the axis they
/// overlap least on, until all are at least `pad` apart.
fn separate(rects: &mut [RectF], pad: f32) {
  // shortcut: pairwise pushes, fine for the dozens of windows on a
  // workspace; a sweep would be needed for hundreds.
  for _ in 0..500 {
    let mut moved = false;
    for a in 0..rects.len() {
      for b in a + 1..rects.len() {
        let ((ax, ay), (bx, by)) = (rects[a].center(), rects[b].center());
        let over_x =
          f32::midpoint(rects[a].w, rects[b].w) + pad - (bx - ax).abs();
        let over_y =
          f32::midpoint(rects[a].h, rects[b].h) + pad - (by - ay).abs();
        if over_x <= 0.0 || over_y <= 0.0 {
          continue;
        }
        moved = true;
        // On one spot (a stack's tabs), the later one goes right or down.
        let away = |from: f32, to: f32| if to < from { -1.0 } else { 1.0 };
        if over_x <= over_y {
          let push = over_x / 2.0 * away(ax, bx);
          rects[a] = rects[a].translate(-push, 0.0);
          rects[b] = rects[b].translate(push, 0.0);
        } else {
          let push = over_y / 2.0 * away(ay, by);
          rects[a] = rects[a].translate(0.0, -push);
          rects[b] = rects[b].translate(0.0, push);
        }
      }
    }
    if !moved {
      break;
    }
  }
}

/// The scale at which all of `rects` fit `largest` of `area`, at most
/// their own size.
fn fit_scale(rects: &[RectF], area: &RectF, largest: f32) -> f32 {
  let bounds = bounds(rects);
  (largest * area.w / bounds.w)
    .min(largest * area.h / bounds.h)
    .clamp(0.0, 1.0)
}

fn bounds(rects: &[RectF]) -> RectF {
  rects[1..]
    .iter()
    .fold(rects[0], |all, rect| all.union(rect))
}

#[cfg(test)]
mod tests {
  use super::spread;
  use crate::overview::layout::RectF;

  const AREA: RectF = RectF::new(0.0, 0.0, 1600.0, 900.0);
  const LARGEST: f32 = 0.85;

  fn overlaps(a: &RectF, b: &RectF) -> bool {
    a.x < b.right()
      && b.x < a.right()
      && a.y < b.bottom()
      && b.y < a.bottom()
  }

  fn within(rect: &RectF, area: &RectF) -> bool {
    let slack = 0.01;
    rect.x >= area.x - slack
      && rect.y >= area.y - slack
      && rect.right() <= area.right() + slack
      && rect.bottom() <= area.bottom() + slack
  }

  fn cluster(rects: &[RectF]) -> RectF {
    rects[1..]
      .iter()
      .fold(rects[0], |all, rect| all.union(rect))
  }

  fn assert_centered(rects: &[RectF], area: &RectF) {
    let ((cx, cy), (ax, ay)) = (cluster(rects).center(), area.center());
    assert!((cx - ax).abs() < 0.5 && (cy - ay).abs() < 0.5, "{rects:?}");
  }

  #[test]
  fn no_windows_spread_to_nothing() {
    assert!(spread(&[], &AREA, 16.0, LARGEST).is_empty());
  }

  /// A lone window shows centered, at its shape, no larger than it is
  /// nor than `LARGEST` of the area.
  #[test]
  fn a_lone_window_is_centered_and_capped() {
    let small = RectF::new(100.0, 100.0, 400.0, 300.0);
    let rects = spread(&[small], &AREA, 20.0, LARGEST);
    assert_eq!(rects, vec![RectF::new(600.0, 300.0, 400.0, 300.0)]);

    let large = RectF::new(0.0, 0.0, 1920.0, 1080.0);
    let rect = spread(&[large], &AREA, 20.0, LARGEST)[0];
    assert!((rect.h - AREA.h * LARGEST).abs() < 0.5, "{rect:?}");
    assert!((rect.w / rect.h - 1920.0 / 1080.0).abs() < 1e-3);
    assert_centered(&[rect], &AREA);
  }

  /// The cluster takes the part of the area it's given, either way.
  #[test]
  fn the_cluster_takes_its_largest_part() {
    let large = RectF::new(0.0, 0.0, 1920.0, 1080.0);
    for largest in [0.6, 0.94, 1.0] {
      let rect = spread(&[large], &AREA, 20.0, largest)[0];
      assert!((rect.h - AREA.h * largest).abs() < 0.5, "{rect:?}");
      assert!(within(&rect, &AREA), "{rect:?}");
      assert_centered(&[rect], &AREA);
    }
  }

  /// Two windows far apart on screen stay a cluster: centered, as far
  /// apart as on screen at the cluster's scale, not flung to the edges.
  #[test]
  fn windows_apart_keep_their_spacing_centered() {
    let windows = [
      RectF::new(0.0, 300.0, 400.0, 300.0),
      RectF::new(800.0, 300.0, 400.0, 300.0),
    ];
    let rects = spread(&windows, &AREA, 20.0, LARGEST);

    // Under the area, so at their own size and their own distance.
    assert!((rects[0].w - 400.0).abs() < 1e-3);
    assert!((rects[1].x - rects[0].x - 800.0).abs() < 1e-3);
    assert_centered(&rects, &AREA);
  }

  /// Tiled windows touching on screen come the gap apart, keeping their
  /// layout, within the area.
  #[test]
  fn tiled_windows_come_the_gap_apart() {
    let windows = [
      RectF::new(0.0, 0.0, 960.0, 1080.0),
      RectF::new(960.0, 0.0, 960.0, 540.0),
      RectF::new(960.0, 540.0, 960.0, 540.0),
    ];
    let gap = 24.0;
    let rects = spread(&windows, &AREA, gap, LARGEST);

    for (index, rect) in rects.iter().enumerate() {
      assert!(within(rect, &AREA), "{rect:?}");
      for other in &rects[index + 1..] {
        // About the gap, within the slack of the scale settling.
        assert!(!overlaps(&rect.inset(-gap * 0.45), other), "{rects:?}");
      }
    }
    assert!(rects[1].x > rects[0].right());
    assert!(rects[2].y > rects[1].bottom());
    assert_centered(&rects, &AREA);
  }

  /// A stack's tabs on one spot, a minimized window where it's restored
  /// to and a floating window over a tiled one come apart, all at one
  /// scale, and the cluster stays centered.
  #[test]
  fn tabs_and_placeholders_come_apart_centered() {
    let stack = RectF::new(0.0, 0.0, 960.0, 1080.0);
    let windows = [
      stack,
      stack,
      RectF::new(960.0, 0.0, 960.0, 1080.0),
      RectF::new(1200.0, 300.0, 500.0, 400.0),
      RectF::new(300.0, 200.0, 800.0, 600.0),
    ];
    let gap = 24.0;
    let rects = spread(&windows, &AREA, gap, LARGEST);

    let scale = rects[0].w / windows[0].w;
    for (index, rect) in rects.iter().enumerate() {
      assert!(within(rect, &AREA), "{rect:?}");
      assert!((rect.w / windows[index].w - scale).abs() < 1e-3);
      for other in &rects[index + 1..] {
        assert!(!overlaps(&rect.inset(-gap / 4.0), other), "{rects:?}");
      }
    }
    // The tiled right half stays right of the stack.
    assert!(rects[2].center().0 > rects[0].center().0);
    assert_centered(&rects, &AREA);
  }
}
