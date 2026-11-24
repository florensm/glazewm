//! Where the exposé spreads a workspace's windows: as one cluster, each
//! about where it sits on screen, in an even grid, or in GNOME's rows.

use crate::{
  overview::{layout::RectF, rows::rows},
  OverviewExposeArrange,
};

/// How the exposé arranges a workspace's windows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Arrangement {
  pub style: OverviewExposeArrange,
  /// Space between windows, in `area`'s pixels.
  pub gap: f32,
  /// Most of the area the cluster takes either way.
  pub largest: f32,
  /// How much the cluster grows small windows before placing them.
  pub small_window_boost: f32,
  /// Smallest window scale before rows take over; `0` never.
  pub readable_floor: f32,
  /// The monitor's working area, in the windows' coordinates.
  pub work: RectF,
}

/// Where each of `windows` (screen rects) shows over `area` as `how`
/// arranges them, in the order given.
pub(crate) fn arrange(
  windows: &[RectF],
  area: &RectF,
  how: &Arrangement,
) -> Vec<RectF> {
  let rows_of = || rows(windows, area, how.gap, how.work.h);
  let rects = match how.style {
    OverviewExposeArrange::Cluster => boosted(windows, area, how),
    OverviewExposeArrange::Grid => grid(windows, area, how.gap),
    OverviewExposeArrange::Rows => return rows_of(),
  };
  if how.readable_floor > 0.0
    && smallest_scale(windows, &rects) < how.readable_floor
  {
    rows_of()
  } else {
    rects
  }
}

/// A shelf along the bottom of a room.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Shelf {
  /// Where each of its windows shows, in the order given.
  pub rects: Vec<RectF>,
  /// The band it takes, its label included, as wide as its windows.
  pub band: RectF,
  /// The room left above it for the other windows.
  pub room: RectF,
}

/// Lays `windows` (screen rects) out along the bottom of `area` in a row,
/// `gap` apart and centered, under a label `label` tall: `height` tall,
/// shrunk for a long row to fit, each no larger than it is.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn shelf(
  windows: &[RectF],
  area: &RectF,
  gap: f32,
  height: f32,
  label: f32,
) -> Shelf {
  let aspects = windows
    .iter()
    .map(|window| window.w.max(1.0) / window.h.max(1.0))
    .collect::<Vec<_>>();
  let gaps = gap * (windows.len().max(1) - 1) as f32;
  let widths = aspects.iter().sum::<f32>();
  let height =
    height.min((area.w - gaps).max(0.0) / widths.max(f32::EPSILON));

  let sizes = windows
    .iter()
    .zip(&aspects)
    .map(|(window, aspect)| {
      let h = height.min(window.h.max(1.0));
      (h * aspect, h)
    })
    .collect::<Vec<_>>();
  let total = sizes.iter().map(|(w, _)| w).sum::<f32>() + gaps;
  let bottom = area.bottom();
  let mut x = area.x + (area.w - total) / 2.0;
  let rects = sizes
    .iter()
    .map(|&(w, h)| {
      let rect = RectF::new(x, bottom - h, w, h);
      x += w + gap;
      rect
    })
    .collect();

  let top = bottom - height - gap - label;
  Shelf {
    rects,
    band: RectF::new(
      area.x + (area.w - total) / 2.0,
      top,
      total,
      bottom - top,
    ),
    room: RectF::new(
      area.x,
      area.y,
      area.w,
      (top - gap - area.y).max(0.0),
    ),
  }
}

/// Smallest scale any of `windows` shows at in `rects`.
fn smallest_scale(windows: &[RectF], rects: &[RectF]) -> f32 {
  windows
    .iter()
    .zip(rects)
    .map(|(window, rect)| rect.w / window.w.max(1.0))
    .fold(f32::INFINITY, f32::min)
}

/// The cluster, its small windows grown first: each about its middle by
/// `1 + boost × (1 − min(1, area / half the work area))`, then shrunk
/// back within the place that gets it, to no larger than it is.
fn boosted(
  windows: &[RectF],
  area: &RectF,
  how: &Arrangement,
) -> Vec<RectF> {
  let boost = how.small_window_boost.clamp(0.0, 0.6);
  if boost <= 0.0 {
    return spread(windows, area, how.gap, how.largest);
  }

  let half_work = (how.work.w * how.work.h / 2.0).max(1.0);
  let grown = windows
    .iter()
    .map(|window| {
      let small = 1.0 - (window.w * window.h / half_work).min(1.0);
      let factor = 1.0 + boost * small;
      scaled_about_center(window, factor)
    })
    .collect::<Vec<_>>();
  spread(&grown, area, how.gap, how.largest)
    .iter()
    .zip(windows)
    .map(|(slot, window)| {
      let scale = (slot.w / window.w.max(1.0))
        .min(slot.h / window.h.max(1.0))
        .min(1.0);
      let (w, h) = (window.w * scale, window.h * scale);
      let (cx, cy) = slot.center();
      RectF::new(cx - w / 2.0, cy - h / 2.0, w, h)
    })
    .collect()
}

/// Largest scale the grid shows a window at, as the rows do.
const MOST_SCALE: f32 = crate::overview::rows::MOST_SCALE;

/// Height, in screen pixels, of the bands the grid reads windows in by.
const GRID_LINE: f32 = 200.0;

/// Windows in even cells over `area`, `gap` apart, in reading order, each
/// fit to its cell; the column count giving the windows most scale.
#[allow(clippy::cast_precision_loss)]
fn grid(windows: &[RectF], area: &RectF, gap: f32) -> Vec<RectF> {
  let count = windows.len();
  if count == 0 {
    return Vec::new();
  }

  let line = |window: &RectF| (window.center().1 / GRID_LINE).round();
  let mut order = (0..count).collect::<Vec<_>>();
  order.sort_by(|&a, &b| {
    line(&windows[a])
      .total_cmp(&line(&windows[b]))
      .then(windows[a].x.total_cmp(&windows[b].x))
  });

  let fit = |window: &RectF, (cell_w, cell_h): (f32, f32)| {
    (cell_w / window.w.max(1.0))
      .min(cell_h / window.h.max(1.0))
      .min(MOST_SCALE)
  };
  let cell = |columns: usize| {
    let lines = count.div_ceil(columns);
    (
      (area.w - gap * (columns - 1) as f32) / columns as f32,
      (area.h - gap * (lines - 1) as f32) / lines as f32,
    )
  };
  let columns = (1..=count)
    .max_by(|&a, &b| {
      let score = |columns| {
        windows
          .iter()
          .map(|window| fit(window, cell(columns)))
          .sum::<f32>()
      };
      // The fewest columns of those scoring alike.
      score(a).total_cmp(&score(b)).then(b.cmp(&a))
    })
    .unwrap_or(1);

  let (cell_w, cell_h) = cell(columns);
  let lines = count.div_ceil(columns);
  let used_h = cell_h * lines as f32 + gap * (lines - 1) as f32;
  let mut rects = vec![RectF::default(); count];
  for (position, &index) in order.iter().enumerate() {
    let (line, column) = (position / columns, position % columns);
    let in_line = columns.min(count - line * columns);
    let line_w = cell_w * in_line as f32 + gap * (in_line - 1) as f32;
    let window = &windows[index];
    let scale = fit(window, (cell_w, cell_h));
    let (w, h) = (window.w * scale, window.h * scale);
    let x =
      area.x + (area.w - line_w) / 2.0 + column as f32 * (cell_w + gap);
    let y =
      area.y + (area.h - used_h) / 2.0 + line as f32 * (cell_h + gap);
    rects[index] =
      RectF::new(x + (cell_w - w) / 2.0, y + (cell_h - h) / 2.0, w, h);
  }
  rects
}

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
fn spread(
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

/// `rect` grown `factor` times about its middle.
fn scaled_about_center(rect: &RectF, factor: f32) -> RectF {
  let (cx, cy) = rect.center();
  let (w, h) = (rect.w * factor, rect.h * factor);
  RectF::new(cx - w / 2.0, cy - h / 2.0, w, h)
}

fn bounds(rects: &[RectF]) -> RectF {
  rects[1..]
    .iter()
    .fold(rects[0], |all, rect| all.union(rect))
}

#[cfg(test)]
mod tests {
  use super::{arrange, shelf, spread, Arrangement};
  use crate::{
    overview::{layout::RectF, rows::rows},
    OverviewExposeArrange,
  };

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

  const WORK: RectF = RectF::new(0.0, 0.0, 1920.0, 1080.0);

  fn how(style: OverviewExposeArrange) -> Arrangement {
    Arrangement {
      style,
      gap: 24.0,
      largest: LARGEST,
      small_window_boost: 0.0,
      readable_floor: 0.0,
      work: WORK,
    }
  }

  /// A stack's two tabs, a tiled half, a float over it, a small dialog.
  fn mixed() -> Vec<RectF> {
    let stack = RectF::new(0.0, 0.0, 960.0, 1080.0);
    vec![
      stack,
      stack,
      RectF::new(960.0, 0.0, 960.0, 1080.0),
      RectF::new(1200.0, 300.0, 500.0, 400.0),
      RectF::new(300.0, 200.0, 320.0, 180.0),
    ]
  }

  /// Nine small windows in a 3 by 3 block.
  fn many() -> Vec<RectF> {
    (0..9_u8)
      .map(|index| {
        let (column, line) = (f32::from(index % 3), f32::from(index / 3));
        RectF::new(column * 640.0, line * 360.0, 600.0, 330.0)
      })
      .collect()
  }

  fn arrangements() -> Vec<Arrangement> {
    vec![
      how(OverviewExposeArrange::Cluster),
      Arrangement {
        small_window_boost: 0.35,
        ..how(OverviewExposeArrange::Cluster)
      },
      how(OverviewExposeArrange::Grid),
      how(OverviewExposeArrange::Rows),
    ]
  }

  /// Every arrangement keeps the windows within the area, at most their
  /// own size, apart (a stack's tabs too), and about centered.
  #[test]
  fn every_arrangement_keeps_the_invariants() {
    let lone = vec![RectF::new(100.0, 100.0, 1920.0, 1080.0)];
    for how in arrangements() {
      for windows in [mixed(), many(), lone.clone()] {
        let rects = arrange(&windows, &AREA, &how);
        assert_eq!(rects.len(), windows.len());
        for (index, rect) in rects.iter().enumerate() {
          let case = format!("{:?} {index} {rect:?}", how.style);
          assert!(within(rect, &AREA), "{case}");
          assert!(rect.w <= windows[index].w + 0.01, "{case}");
          assert!(rect.w > 0.0 && rect.h > 0.0, "{case}");
          let aspect = windows[index].w / windows[index].h;
          assert!((rect.w / rect.h - aspect).abs() < 1e-2, "{case}");
          for other in &rects[index + 1..] {
            assert!(!overlaps(rect, other), "{case} {other:?}");
          }
        }
        let ((cx, cy), (ax, ay)) =
          (cluster(&rects).center(), AREA.center());
        assert!(
          (cx - ax).abs() < AREA.w * 0.05,
          "{:?} {rects:?}",
          how.style
        );
        assert!(
          (cy - ay).abs() < AREA.h * 0.05,
          "{:?} {rects:?}",
          how.style
        );
      }
    }
  }

  /// The grid reads windows left to right, top to bottom, in even cells.
  #[test]
  fn the_grid_keeps_reading_order_in_even_cells() {
    let windows = many();
    let rects =
      arrange(&windows, &AREA, &how(OverviewExposeArrange::Grid));
    for line in rects.chunks(3) {
      assert!(line[0].right() < line[1].x && line[1].right() < line[2].x);
      assert!((line[0].y - line[2].y).abs() < 0.5);
    }
    assert!(
      rects[0].bottom() < rects[3].y && rects[3].bottom() < rects[6].y
    );
    // Alike windows, alike sizes.
    assert!(rects.iter().all(|rect| (rect.w - rects[0].w).abs() < 0.5));
  }

  /// GNOME's rows grow a small window up to 1.5× against a full-height
  /// one, and show none past 95%.
  #[test]
  fn rows_bump_small_windows_and_cap_the_scale() {
    // Wide enough that neither reaches the cap.
    let windows = [
      RectF::new(0.0, 0.0, 3000.0, 1080.0),
      RectF::new(3100.0, 400.0, 400.0, 300.0),
    ];
    let rects = rows(&windows, &AREA, 24.0, 1080.0);
    let large = rects[0].w / windows[0].w;
    let small = rects[1].w / windows[1].w;
    assert!(small < 0.95 && small > large * 1.3, "{large} {small}");
    // After it, as on screen: beside it, or in the next row.
    assert!(
      rects[1].x > rects[0].right() || rects[1].y > rects[0].bottom()
    );

    let lone =
      rows(&[RectF::new(0.0, 0.0, 400.0, 300.0)], &AREA, 24.0, 1080.0);
    assert!((lone[0].w - 400.0 * 0.95).abs() < 0.01, "{lone:?}");
    assert!(rows(&[], &AREA, 24.0, 1080.0).is_empty());
  }

  /// Boosting grows small windows against large ones, keeping where they
  /// sit.
  #[test]
  fn boost_grows_small_windows_keeping_their_places() {
    let windows = [
      RectF::new(0.0, 0.0, 1400.0, 1080.0),
      RectF::new(1500.0, 300.0, 400.0, 300.0),
    ];
    let ratio = |boost: f32| {
      let how = Arrangement {
        small_window_boost: boost,
        ..how(OverviewExposeArrange::Cluster)
      };
      let rects = arrange(&windows, &AREA, &how);
      assert!(rects[1].x > rects[0].right(), "still to its right");
      (rects[1].w / windows[1].w) / (rects[0].w / windows[0].w)
    };
    assert!((ratio(0.0) - 1.0).abs() < 1e-3, "the cluster scales alike");
    assert!(ratio(0.5) > 1.2, "{}", ratio(0.5));
  }

  /// The shelf's windows sit in a centered row along the bottom, at its
  /// height or their own, under its label; the room above leaves it
  /// clear.
  #[test]
  fn the_shelf_lines_windows_up_along_the_bottom() {
    let windows = [
      RectF::new(0.0, 0.0, 1600.0, 900.0),
      RectF::new(0.0, 0.0, 800.0, 800.0),
      RectF::new(0.0, 0.0, 60.0, 40.0),
    ];
    let shelf = shelf(&windows, &AREA, 20.0, 96.0, 30.0);
    for (index, rect) in shelf.rects.iter().enumerate() {
      assert!(within(rect, &AREA), "{rect:?}");
      assert!((rect.bottom() - AREA.bottom()).abs() < 0.01);
      assert!(rect.h <= 96.0 && rect.h <= windows[index].h);
      assert!(rect.y > shelf.band.y + 30.0, "under the label");
      assert!(rect.y > shelf.room.bottom(), "clear of the room");
      for other in &shelf.rects[index + 1..] {
        assert!(other.x > rect.right());
      }
    }
    assert!((shelf.rects[0].h - 96.0).abs() < 0.01);
    assert!(
      (shelf.rects[2].h - 40.0).abs() < 0.01,
      "no larger than it is"
    );
    assert!((shelf.band.center().0 - AREA.center().0).abs() < 0.5);
    assert!(shelf.room.y.abs() < f32::EPSILON);

    // Too many for the width: they shrink to fit.
    let crowd = vec![RectF::new(0.0, 0.0, 1600.0, 900.0); 20];
    let shelf = super::shelf(&crowd, &AREA, 20.0, 96.0, 30.0);
    assert!(shelf.rects.iter().all(|rect| within(rect, &AREA)));
    assert!(shelf.rects[0].h < 96.0);
  }

  /// Under the readable floor, rows take over; above it, nothing changes.
  #[test]
  fn readable_floor_switches_to_rows() {
    let windows = many();
    let cluster =
      arrange(&windows, &AREA, &how(OverviewExposeArrange::Cluster));
    let scale = cluster[0].w / windows[0].w;

    let floored = Arrangement {
      readable_floor: scale + 0.05,
      ..how(OverviewExposeArrange::Cluster)
    };
    assert_eq!(
      arrange(&windows, &AREA, &floored),
      rows(&windows, &AREA, 24.0, 1080.0)
    );
    let unmet = Arrangement {
      readable_floor: scale - 0.05,
      ..floored
    };
    assert_eq!(arrange(&windows, &AREA, &unmet), cluster);
  }
}
