//! The `PeekSurface`s tiles' peeks are hosted on, and how the tiles on one
//! stack.

use uuid::Uuid;
use wm_common::WindowState;
use wm_platform::{
  perf::{self, Stage},
  PeekSurface, Rect, RectDelta, TabBarPlace, HWND,
};

use super::{
  motions::PeekRaise,
  overlays::{ring_reach, visible_frame},
  ready::is_stale_surface,
  Hold, PeekView, Role,
};
use crate::{
  animation::state::WindowAnimationState,
  models::WindowContainer,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Where a tile's peek draws on the `PeekSurface` it is hosted on: its
/// visible frame and its ring around it. A tile crossing
/// monitors has one for both its peeks (see [`Trail`](super::Trail)).
///
/// Tenants of one surface stack in the order their thumbnails were
/// registered, and a tenant registers some late (its ring copy as its
/// overlays are tracked, its fill and companion on its first ticks), over
/// the others' windows. So one drawn under a lower-ranked one its path may
/// meet is stacked whole again (see `PeekView::restack_crossings`).
pub(super) struct Tenancy {
  pub(super) reach: Reach,
  /// Its lane, while it moves with no transit (see
  /// [`Entry::lane`](super::Entry::lane)).
  pub(super) lane: Lane,
  /// The order its tile was put on top in, as a peek of its own is raised
  /// in: by each command that moves it, not by one that retargets it.
  pub(super) rank: Rank,
  /// Whether its motion raised nothing, as a grab's doesn't (see
  /// [`hold_on_peek`](super::motions::hold_on_peek)): its surface is
  /// wherever it went up for it, so it goes above the window as that
  /// lands, as a column's does.
  pub(super) never_raised: bool,
  /// Whether its pass raises it in its band after the tiles (see
  /// [`Host::InBand`]): its surface may be under its window as that lands
  /// (raised past it, or into its band), so it goes above it then, as its
  /// own peek would.
  pub(super) in_band: bool,
}

impl Tenancy {
  /// Whether its surface goes above its window as that lands (see
  /// `WindowPeek::stay_over_window`).
  pub(super) fn lifts_at_landing(&self) -> bool {
    self.never_raised || self.in_band
  }
}

/// Which of its monitor's surfaces for tiles a hosted peek draws on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::animation) enum Layer {
  /// The tiles' and columns' (`PeekView::surfaces`), raised to the top of
  /// the other windows for a motion.
  Tiles,
  /// The always-on-top band's (`PeekView::band_surfaces`), for windows
  /// shown there: raised only to the top of that band.
  TopBand,
}

/// Where a hosted peek draws, and how it ranks among its surface's
/// tenants (see [`host_for`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::animation) enum Host {
  /// On the tiles' surface, ranked as its motion puts it on top.
  Tiles,
  /// Raised in its band once its pass's raises are done (`peek_raises` in
  /// `platform_sync`), on that band's surface: over every tile its pass
  /// raised, in the order such raises come.
  InBand(Layer),
}

impl Host {
  pub(super) fn layer(self) -> Layer {
    match self {
      Self::Tiles => Layer::Tiles,
      Self::InBand(layer) => layer,
    }
  }

  /// How a tenant whose motion would put it on top by `own` is ranked.
  pub(super) fn stacking(self, own: Stacking) -> Stacking {
    match self {
      Self::Tiles => own,
      Self::InBand(_) => Stacking::OverPass,
    }
  }
}

/// Where a tile stacks among the tenants of its surface, by when each was
/// last put on top.
///
/// A moving tile's peek is raised once its pass's windows are cloaked
/// (`raise_moving` in `platform_sync`'s raise loop), above any put on top
/// in that pass before. A tile opening, closing, minimizing or restoring
/// goes over every tile its pass and earlier ones moved, so neighbours
/// reflowing around it need no restack (owner's decision, 2026-11-17;
/// peeks of their own put it under them).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Rank {
  /// The raise pass it was put on top in, from 1.
  pass: u64,
  /// How it was put on top in its pass.
  pub(super) level: Level,
  /// The order ranks were given in.
  order: u64,
}

/// How a tile was put on top within its raise pass, lowest first (see
/// [`Rank`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Level {
  /// Before its pass's raises.
  InPass,
  /// By its pass's raises.
  Raised,
  /// Over its pass's raises, as a tile opening, closing, minimizing or
  /// restoring.
  Transit,
}

/// How a tile is put on top of its surface's tenants (see [`Rank`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stacking {
  /// By its pass's raises, as a moving tile is.
  Raised,
  /// In its pass, before its raises, as an unfocused column is.
  InPass,
  /// Over its pass's raises, as a tile opening, closing, minimizing or
  /// restoring is.
  Transit,
  /// After its pass's raises, as one raised in its band then is (see
  /// [`Host::InBand`]): as if put on top first in the next pass.
  OverPass,
}

/// The rank of a tile put on top by `stacking` in raise pass `pass`
/// (counted from 0), given `order`-th.
pub(super) fn rank(stacking: Stacking, pass: u64, order: u64) -> Rank {
  let (pass, level) = match stacking {
    Stacking::InPass => (pass + 1, Level::InPass),
    Stacking::Raised => (pass + 1, Level::Raised),
    Stacking::Transit => (pass + 1, Level::Transit),
    Stacking::OverPass => (pass + 2, Level::InPass),
  };
  Rank { pass, level, order }
}

/// What a tile's peek draws around its rect: its visible frame, inside the
/// invisible `shadow_borders`, its stack's tab bar beside it at `bar`, and
/// its ring, `ring` past both.
#[derive(Clone, Debug)]
pub(super) struct Reach {
  shadow_borders: RectDelta,
  bar: Option<TabBarPlace>,
  ring: i32,
}

impl Reach {
  /// The lane of a tile drawn from `from` to `to`.
  pub(super) fn lane(&self, from: &Rect, to: &Rect) -> Lane {
    let drawn = |rect: &Rect| {
      let frame = visible_frame(rect, &self.shadow_borders);
      self
        .bar
        .as_ref()
        .map_or(frame.clone(), |bar| frame.union(&bar.rect(rect)))
        .inset(-self.ring)
    };
    Lane {
      from: drawn(from),
      to: drawn(to),
    }
  }
}

/// The area a tile's peek draws over at the start and the end of its
/// motion, which it goes between with its eased progress.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Lane {
  from: Rect,
  to: Rect,
}

impl Lane {
  /// All it may reach on the way, unless its easing overshoots.
  fn hull(&self) -> Rect {
    self.from.union(&self.to)
  }

  /// The lane of a tile that has landed at its end.
  pub(super) fn landed(&self) -> Self {
    Self {
      from: self.to.clone(),
      to: self.to.clone(),
    }
  }
}

/// Whether a window put on a peek may be drawn on one of its monitor's
/// `PeekSurface`s, which [`host_for`] picks: a move or
/// resize by command ([`move_on_peek`](super::motions::move_on_peek)), a
/// crossing between floating and tiling, a fullscreen toggle
/// ([`fullscreen_on_peek`](super::motions::fullscreen_on_peek)), an open,
/// restore, close or minimize
/// ([`enter_on_peek`](super::motions::enter_on_peek),
/// [`restore_on_peek`](super::motions::restore_on_peek),
/// [`leave_on_peek`](super::motions::leave_on_peek)). A stack's window
/// too, its tab bar drawn by a copy over the surface's other tiles as it
/// moves (see `carry_tab_bar`).
/// Not a scrolling layout's column, which the scroll view draws. One drawn
/// on two monitors is, on each one's surface (see
/// [`Trail`](super::Trail)).
///
/// Nor one entering beside a tile a grab holds (`beside_held`, see
/// [`hosts_transit`]).
// Independent conditions, each keeping a window on a peek of its own.
fn hosts_on_surface(is_column: bool, beside_held: bool) -> bool {
  !is_column && !beside_held
}

/// Where a tile moving onto another monitor that [`host_for`] puts on
/// `host` is drawn: there, both its parts (see [`Trail`](super::Trail)),
/// unless a grab holds a tile on either monitor's tiles' surface
/// (`held_on_either`), which the tile's raise would top over the floating
/// windows held tiles stay under (see [`hosts_hold`]). Kept off surfaces,
/// it plays on peeks of its own.
pub(super) fn crossing_host(
  host: Option<Host>,
  held_on_either: bool,
) -> Option<Host> {
  host.filter(|host| host.layer() != Layer::Tiles || !held_on_either)
}

/// The surface a peek that [`hosts_on_surface`] allows is drawn on, as its
/// own peek would play: a tile's (`is_tile`, fullscreen toggles included)
/// among the tiles, unless its pass also `raise`s it in a band (see
/// [`PeekRaise`]); then on that band's surface, after the pass's tiles. A
/// window other than a tile that no band raise takes (an unfocused
/// floating one not shown on top, a leaving one) keeps a peek of its own:
/// that stays just below its window, under the floating windows above it,
/// where a surface raised for other tenants can't stay.
pub(super) fn host_for(
  is_tile: bool,
  raise: Option<PeekRaise>,
) -> Option<Host> {
  match raise {
    Some(PeekRaise::TopMost) => Some(Host::InBand(Layer::TopBand)),
    Some(PeekRaise::Top) => Some(Host::InBand(Layer::Tiles)),
    None => is_tile.then_some(Host::Tiles),
  }
}

/// Whether a tile a grab resize holds (see
/// [`hold_on_peek`](super::motions::hold_on_peek)) is drawn on
/// its monitor's `PeekSurface`: only if that surface is not
/// `topped`, as a held peek stays just under its window for the whole
/// grab, never over floating windows above it,
/// and a topped surface is above them until its last tenant has gone.
/// [`can_hold`](super::motions::can_hold) has left out what else keeps a
/// tile on a peek of its own.
pub(super) fn hosts_hold(topped: bool) -> bool {
  !topped
}

/// Whether a scrolling layout's column, which the scroll view may draw on
/// the `PeekSurface` of its working area, a grab holding it or not, a
/// stack's included (see
/// `carry_tab_bar`), is drawn
/// there in its role: one that `rests` (at rest, or moving to rest hidden)
/// only if no other tenant of that surface does (`rests_elsewhere`, one
/// kept for a workspace switch's surrogate included), as the surface has
/// one place at rest.
pub(super) fn keeps_rest_hosted(
  rests: bool,
  rests_elsewhere: bool,
) -> bool {
  !rests || !rests_elsewhere
}

/// Whether an entry in `role` rests, or is to once it lands.
pub(super) fn rests(role: Role) -> bool {
  matches!(role, Role::Peek | Role::Moving { ends_shown: false })
}

/// Of the tenants of a surface, by role and window, the window it goes
/// just above at rest (see `PeekSurface::rest_above`): the one tenant's,
/// once every tenant is at rest.
fn rest_anchor(
  tenants: impl IntoIterator<Item = (Role, HWND)>,
) -> Option<HWND> {
  let mut anchor = None;
  for (role, window) in tenants {
    if role != Role::Peek || anchor.is_some() {
      return None;
    }
    anchor = Some(window);
  }
  anchor
}

/// Where `window`, entering or leaving, and raised in a band by its pass
/// as `raise` says, is drawn (see [`hosts_on_surface`], [`host_for`]).
///
/// One `entering` the tiles' surface beside a tile a grab holds keeps a
/// peek of its own: that surface would be raised to the top, putting the
/// held tiles there over the floating windows they stay under (see
/// [`hosts_hold`]). One leaving is never raised.
pub(super) fn hosts_transit(
  window: &WindowContainer,
  entering: bool,
  raise: Option<PeekRaise>,
  peeks: &PeekView,
) -> Option<Host> {
  let host = host_for(window.state() == WindowState::Tiling, raise)?;
  let beside_held = entering
    && host.layer() == Layer::Tiles
    && has_held_neighbour(window, peeks);
  hosts_on_surface(
    crate::layout::scrolling::column_of(&window.as_container()).is_some(),
    beside_held,
  )
  .then_some(host)
}

/// Whether another tile of `window`'s workspace is on a peek a grab holds.
pub(super) fn has_held_neighbour(
  window: &WindowContainer,
  peeks: &PeekView,
) -> bool {
  window.workspace().is_some_and(|workspace| {
    workspace
      .descendants()
      .filter_map(|container| container.as_window_container().ok())
      .any(|other| other.id() != window.id() && peeks.is_held(&other.id()))
  })
}

/// Whether two tiles drawn along lanes `a` and `b` on one clock never
/// overlap: apart along one axis, the same way round, at both ends.
///
/// Every edge moves linearly with the motions' shared eased progress (see
/// `Rect::interpolate`), so a gap at both ends holds all the way, at least
/// one pixel of it where it changes, as each edge is rounded on its own.
/// An easing that `overshoots` past either end only keeps a gap that
/// doesn't change.
fn lanes_stay_apart(a: &Lane, b: &Lane, overshoots: bool) -> bool {
  let holds = |(at_from, at_to): (i32, i32)| {
    if at_from == at_to {
      at_from >= 0
    } else {
      !overshoots && at_from >= 1 && at_to >= 1
    }
  };

  [
    (b.from.left - a.from.right, b.to.left - a.to.right),
    (a.from.left - b.from.right, a.to.left - b.to.right),
    (b.from.top - a.from.bottom, b.to.top - a.to.bottom),
    (a.from.top - b.from.bottom, a.to.top - b.to.bottom),
  ]
  .into_iter()
  .any(holds)
}

/// Whether two tiles drawn along lanes `a` and `b` may overlap: on one
/// clock, unless [`lanes_stay_apart`]; otherwise wherever their hulls
/// meet, or anywhere if either easing `overshoots` past its ends.
pub(super) fn lanes_meet(
  a: &Lane,
  b: &Lane,
  same_clock: bool,
  overshoots: bool,
) -> bool {
  if same_clock {
    !lanes_stay_apart(a, b, overshoots)
  } else {
    overshoots || a.hull().intersection(&b.hull()).is_some()
  }
}

/// Whether a tile ranked `rank` was put on top in a raise pass before the
/// one now open, `raise_passes` having closed (see [`rank`]).
fn ranks_before_pass(rank: Rank, raise_passes: u64) -> bool {
  rank.pass <= raise_passes
}

/// Whether a tile in `role`, ranked `rank` if it is on a surface, is put
/// on top again before a tile is ranked in the raise pass now open,
/// `raise_passes` having closed: one closing or minimizing, ranked in an
/// earlier pass, stays over every tile moving while it leaves.
fn reranks_leaving(
  role: Role,
  rank: Option<Rank>,
  raise_passes: u64,
) -> bool {
  matches!(role, Role::Leaving(_))
    && rank.is_some_and(|rank| ranks_before_pass(rank, raise_passes))
}

/// Of the tiles on one surface in rank order, by the first and last
/// registration orders of their thumbnails there (see
/// `WindowPeek::stacking`), those to register anew, in that order, so each
/// draws wholly over every lower-ranked one that `meets` it (by their
/// indices, lower first): each with a thumbnail under one of such a
/// tile's, as that tile is then, re-registered ones included.
pub(in crate::animation) fn out_of_order(
  spans: &[Option<(u64, u64)>],
  meets: impl Fn(usize, usize) -> bool,
) -> Vec<usize> {
  let mut spans = spans.to_vec();
  let mut next = spans
    .iter()
    .flatten()
    .map(|(_, last)| last + 1)
    .max()
    .unwrap_or_default();
  let mut restacked = Vec::new();

  for upper in 0..spans.len() {
    let Some(Some((first, _))) = spans.get(upper).copied() else {
      continue;
    };
    let is_under =
      spans.iter().take(upper).enumerate().any(|(lower, span)| {
        span.is_some_and(|(_, last)| last > first) && meets(lower, upper)
      });
    if is_under {
      if let Some(span) = spans.get_mut(upper) {
        *span = Some((next, next));
      }
      next += 1;
      restacked.push(upper);
    }
  }
  restacked
}

/// The keys of surfaces' tenants, given with their surface's window and
/// rank, grouped by surface, each group in rank order (see
/// [`out_of_order`]).
fn by_surface<K: Copy>(mut tenants: Vec<(isize, Rank, K)>) -> Vec<Vec<K>> {
  tenants.sort_by_key(|(surface, rank, _)| (*surface, *rank));
  tenants
    .chunk_by(|a, b| a.0 == b.0)
    .map(|group| group.iter().map(|(_, _, key)| *key).collect())
    .collect()
}

/// The lowest of `items`, as `is_below(a, b)` says `a` is below `b`; of
/// those level, the first.
fn lowest<T: Copy>(
  items: impl IntoIterator<Item = T>,
  is_below: impl Fn(T, T) -> bool,
) -> Option<T> {
  items.into_iter().fold(None, |low, item| match low {
    Some(low) if !is_below(item, low) => Some(low),
    _ => Some(item),
  })
}

/// Keeps the surface of each tile closing or minimizing on one above the
/// real windows of the tiles shown on its monitor until its transit ends
/// (see [`keeps_above_neighbours`]; proposal 6), so its neighbours, landed
/// and handed back while it plays, don't cover it: the surface goes just
/// above each of those windows it is below, never down. A neighbour still
/// drawn by a peek is left to it. Run at the end of every sync and on
/// every restack another app makes.
///
/// Only tiling windows: floating ones above stay above. The neighbours'
/// rings and backdrops sit behind their windows, so under the surface too.
pub fn keep_leaving_above(state: &WmState) {
  let peeks = &state.animation_manager.peeks;
  let leaving = peeks
    .entries
    .values()
    .filter(|entry| {
      keeps_above_neighbours(
        entry.role,
        entry.tenancy.is_some(),
        entry
          .transit
          .as_ref()
          .is_some_and(|transit| !transit.is_complete()),
      )
    })
    .collect::<Vec<_>>();
  if leaving.is_empty() {
    return;
  }

  for window in state.windows() {
    if window.state() != WindowState::Tiling
      || peeks.draws_window(&window.id())
      || !window
        .workspace()
        .is_some_and(|workspace| workspace.is_displayed())
    {
      continue;
    }
    let Some(clip) =
      window.monitor().and_then(|monitor| monitor.to_rect().ok())
    else {
      continue;
    };
    for entry in leaving.iter().filter(|entry| entry.clip == clip) {
      entry.peek.keep_surface_above(window.native().hwnd());
    }
  }
}

/// Whether a tile in `role`, on a surface (`is_hosted`) or not, keeps its
/// surface above its neighbours' real windows (see
/// [`keep_leaving_above`]): one closing or minimizing there while its
/// transit runs.
fn keeps_above_neighbours(
  role: Role,
  is_hosted: bool,
  transit_runs: bool,
) -> bool {
  matches!(role, Role::Leaving(_)) && is_hosted && transit_runs
}

/// The clips a tile, column or top band surface may span, read only for a
/// surface to be made for `clip` if a tile is to `host` there (see
/// `PeekView::tenancy`).
pub(super) fn surface_monitor_clips(
  state: &WmState,
  host: Option<Host>,
  clip: &Rect,
) -> Vec<Rect> {
  let peeks = &state.animation_manager.peeks;
  if host
    .is_some_and(|host| peeks.surface_at(host.layer(), clip).is_none())
  {
    surface_clips(state)
  } else {
    Vec::new()
  }
}

/// Every monitor's rect and working area: the clips tiles' and columns'
/// surfaces span. Both kinds, so a surface made for a tile never drops an
/// unused column surface as off the monitors, nor the other way round.
pub(crate) fn surface_clips(state: &WmState) -> Vec<Rect> {
  state
    .monitors()
    .iter()
    .flat_map(|monitor| {
      monitor
        .to_rect()
        .ok()
        .into_iter()
        .chain([monitor.native_properties().working_area])
    })
    .collect()
}

impl PeekView {
  /// Puts each surface whose tenants all rest just above its resting
  /// tenant's cloaked window (see [`rest_anchor`]), where that tenant's
  /// own peek would be, once nothing moving on it needs it higher. One
  /// kept for a workspace switch's surrogate is a tenant still.
  pub(in crate::animation) fn settle_resting_surfaces(&self) {
    for surface in &self.surfaces {
      if surface.is_empty() {
        continue;
      }
      let tenants = self
        .entries
        .values()
        .chain(self.switching_out.values())
        .filter(|entry| entry.is_on(surface.hwnd()))
        .map(|entry| (entry.role, entry.peek.source()));
      if let Some(window) = rest_anchor(tenants) {
        surface.rest_above(window);
      }
    }
  }

  /// The surface a new peek of `window`, a tile moving by `motion` cut at
  /// `clip`, is drawn on, by its index in `host`'s layer's surfaces, made
  /// if there is none, and where it draws there, ranked as `stacking` puts
  /// it on top, or as `host` does (see [`Rank`], [`Host::stacking`]).
  /// `None` only if no surface could be made. Making one drops the unused
  /// surfaces of that layer whose clip is none of `monitor_clips` (see
  /// [`is_stale_surface`]).
  ///
  /// It joins whatever its surface's tenants are doing: a surface raised
  /// already stays above them (see `SurfaceWindow::admit`), and one
  /// drawn under a lower-ranked tile it may meet is stacked over it (see
  /// [`restack_crossings`](Self::restack_crossings)).
  #[allow(clippy::too_many_arguments)]
  pub(super) fn tenancy(
    &mut self,
    window: &WindowContainer,
    motion: &WindowAnimationState,
    clip: &Rect,
    host: Host,
    stacking: Stacking,
    config: &UserConfig,
    monitor_clips: &[Rect],
  ) -> Option<(usize, Tenancy)> {
    let reach = Reach {
      shadow_borders: window.native_properties().shadow_borders.clone(),
      bar: crate::tab_bars::tab_bar_place(
        window,
        &motion.target_rect,
        false,
        config,
      ),
      ring: ring_reach(window, config),
    };
    let lane = reach.lane(&motion.start_rect, &motion.target_rect);
    let index = self.surface_index(host.layer(), clip, monitor_clips)?;

    self.keep_leaving_on_top();
    let rank = self.take_rank(host.stacking(stacking));
    Some((
      index,
      Tenancy {
        reach,
        lane,
        rank,
        never_raised: false,
        in_band: matches!(host, Host::InBand(_)),
      },
    ))
  }

  /// The index of `layer`'s surface spanning `clip`, made if there is
  /// none, which drops that layer's unused surfaces off `monitor_clips`
  /// (see [`is_stale_surface`]). `None` only if none could be made.
  pub(super) fn surface_index(
    &mut self,
    layer: Layer,
    clip: &Rect,
    monitor_clips: &[Rect],
  ) -> Option<usize> {
    let surfaces = self.layer_surfaces_mut(layer);
    if let Some(index) =
      surfaces.iter().position(|surface| surface.clip() == clip)
    {
      return Some(index);
    }

    match PeekSurface::new(clip) {
      Ok(surface) => {
        surfaces.retain(|surface| {
          !is_stale_surface(
            surface.is_unused(),
            surface.clip(),
            monitor_clips,
          )
        });
        surfaces.push(surface);
        Some(surfaces.len() - 1)
      }
      Err(err) => {
        tracing::warn!("Failed to create a peek surface: {err}");
        None
      }
    }
  }

  /// Whether a grab holds a tile drawn on the tiles' surface spanning
  /// `clip`, if any.
  pub(super) fn holds_on_surface(&self, clip: &Rect) -> bool {
    self.surface_at(Layer::Tiles, clip).is_some_and(|surface| {
      self.entries.values().any(|entry| {
        entry.hold == Hold::Held && entry.is_on(surface.hwnd())
      })
    })
  }

  /// The surfaces of `layer`.
  pub(super) fn layer_surfaces(&self, layer: Layer) -> &[PeekSurface] {
    match layer {
      Layer::Tiles => &self.surfaces,
      Layer::TopBand => &self.band_surfaces,
    }
  }

  fn layer_surfaces_mut(&mut self, layer: Layer) -> &mut Vec<PeekSurface> {
    match layer {
      Layer::Tiles => &mut self.surfaces,
      Layer::TopBand => &mut self.band_surfaces,
    }
  }

  /// The surface of `layer` spanning `clip`, if any.
  pub(super) fn surface_at(
    &self,
    layer: Layer,
    clip: &Rect,
  ) -> Option<&PeekSurface> {
    self
      .layer_surfaces(layer)
      .iter()
      .find(|surface| surface.clip() == clip)
  }

  /// The surface `tenancy` found for a peek drawn as `host` says, if any
  /// (see [`tenancy`](Self::tenancy)).
  pub(super) fn hosting(
    &self,
    host: Option<Host>,
    tenancy: Option<&(usize, Tenancy)>,
  ) -> Option<&PeekSurface> {
    let (index, _) = tenancy?;
    self.layer_surfaces(host?.layer()).get(*index)
  }

  /// The rank of a tile put on top by `stacking` now (see [`Rank`]).
  pub(super) fn take_rank(&mut self, stacking: Stacking) -> Rank {
    let order = self.next_rank;
    self.next_rank += 1;
    rank(stacking, self.raise_passes, order)
  }

  /// Registers anew, in rank order, each tile on a surface drawn under a
  /// lower-ranked one its path may meet (see [`out_of_order`]), so each
  /// draws whole over those ranked below it, as peeks of their own raised
  /// in that order would. Each part goes up before the one it replaces
  /// goes (see `WindowPeek::register_anew`). Called wherever a tile may
  /// have registered something or been set on a new path.
  ///
  /// Per surface: a tile crossing monitors is restacked on each of its
  /// two by the same rank and lane (see [`Trail`](super::Trail)).
  pub(in crate::animation) fn restack_crossings(&mut self) {
    let tenants = self
      .entries
      .iter()
      .flat_map(|(id, entry)| {
        let rank = entry.tenancy.as_ref().map(|tenancy| tenancy.rank);
        rank.into_iter().flat_map(move |rank| {
          entry
            .hosted_parts()
            .map(move |(part, surface)| (surface.0, rank, (*id, part)))
        })
      })
      .collect::<Vec<_>>();
    if tenants.len() < 2 {
      return;
    }

    for group in by_surface(tenants) {
      let members = group
        .iter()
        .filter_map(|(id, part)| Some((*id, *part, self.entries.get(id)?)))
        .collect::<Vec<_>>();
      let spans = members
        .iter()
        .map(|(_, part, entry)| entry.part(*part)?.stacking())
        .collect::<Vec<_>>();
      let meets = |lower: usize, upper: usize| match (
        members.get(lower),
        members.get(upper),
      ) {
        (Some((_, _, lower)), Some((_, _, upper))) => lower.meets(upper),
        _ => false,
      };
      let restacked = out_of_order(&spans, meets)
        .into_iter()
        .filter_map(|index| {
          let (id, part, _) = members.get(index)?;
          // Named in the debug line, to tell which tile it went over.
          let below = (0..index)
            .filter(|lower| meets(*lower, index))
            .filter_map(|lower| members.get(lower).map(|(id, ..)| *id))
            .collect::<Vec<_>>();
          Some((*id, *part, below))
        })
        .collect::<Vec<_>>();
      if restacked.is_empty() {
        continue;
      }

      let _scope = perf::scope(Stage::Restack);
      for (id, part, below) in restacked {
        tracing::debug!(
          "Stacking {id}'s peek over a tile it may cross, of those ranked \
           below it: {below:?}."
        );
        if let Some(peek) = self
          .entries
          .get_mut(&id)
          .and_then(|entry| entry.part_mut(part))
        {
          if !peek.register_anew() {
            tracing::warn!("Failed to stack {id}'s peek over another.");
          }
        }
      }
    }
  }

  /// The tiles joining surfaces whose windows, still shown, are above
  /// their surface, with its window: every tile on one not raised since
  /// it was last empty, which went below the lowest of their windows, and
  /// on a raised one, those yet to be composited whose window it is below
  /// (see `SurfaceWindow::admit`). Both layers' surfaces.
  ///
  /// A tile joining two (see [`Trail`](super::Trail)) is given the lower
  /// of them, so its overlays go under both.
  pub(super) fn joining_tenants(&self) -> Vec<(Uuid, HWND)> {
    let surface_of = |hwnd: HWND| {
      self
        .surfaces
        .iter()
        .chain(&self.band_surfaces)
        .find(|surface| surface.hwnd() == hwnd)
    };

    self
      .entries
      .iter()
      .filter_map(|(id, entry)| {
        let joined = entry.hosted_parts().filter_map(|(_, hwnd)| {
          surface_of(hwnd).filter(|surface| {
            !surface.is_raised()
              || (!entry.composited
                && matches!(entry.role, Role::Moving { .. })
                && surface.is_below(entry.peek.source()))
          })
        });
        let anchor = lowest(joined, |a, b| a.is_below(b.hwnd()))?;
        Some((*id, anchor.hwnd()))
      })
      .collect()
  }

  /// Whether the `PeekSurface` spanning `clip`, if any, is at the top of
  /// its band (see `PeekSurface::is_topped`).
  pub(super) fn is_surface_topped(&self, clip: &Rect) -> bool {
    self
      .surfaces
      .iter()
      .find(|surface| surface.clip() == clip)
      .is_some_and(PeekSurface::is_topped)
  }

  /// Whether a tiles' and columns' `PeekSurface` spans `clip`.
  pub fn has_surface(&self, clip: &Rect) -> bool {
    self.surface_at(Layer::Tiles, clip).is_some()
  }

  /// Drops the ready peeks cut at none of `clips`, every monitor's rect
  /// and working area, then the tiles', columns' and top band's surfaces
  /// left unused off them (see [`is_stale_surface`]), as the displays
  /// change.
  ///
  /// One in use stays listed, so what draws on it is still raised,
  /// restacked and settled with it, and no second surface is made at its
  /// clip; it goes at a later purge, or as a surface is made, once unused.
  pub fn retain_surfaces(&mut self, clips: &[Rect]) {
    self
      .ready
      .retain(|_, ready| clips.contains(&ready.fit.clip));
    for surfaces in [&mut self.surfaces, &mut self.band_surfaces] {
      surfaces.retain(|surface| {
        !is_stale_surface(surface.is_unused(), surface.clip(), clips)
      });
    }
  }

  /// Drops every ready peek, then every surface left unused, tiles',
  /// columns', the top band's and switches', as the config reloads or the
  /// WM pauses or exits. One in use stays, as for
  /// [`retain_surfaces`](Self::retain_surfaces): a switch's tenants live
  /// in the switch, which plays on to its end.
  pub fn drop_unused_surfaces(&mut self) {
    self.ready.clear();
    for surfaces in [&mut self.surfaces, &mut self.band_surfaces] {
      surfaces.retain(|surface| {
        !is_stale_surface(surface.is_unused(), surface.clip(), &[])
      });
    }
    self.switch_surfaces.retain(|surface| {
      !is_stale_surface(surface.is_unused(), surface.clip(), &[])
    });
  }

  /// The surface a workspace switch on the monitor whose working area is
  /// `area` draws its windows on (see `WorkspaceSurrogate::hosted`), made
  /// if there is none, which drops the unused ones spanning none of
  /// `areas`, every monitor's working area. `None` if none could be made.
  ///
  /// Apart from the tiles' surfaces, for two reasons. Spanning a whole
  /// monitor, as a tile's does, it would read to Explorer as a fullscreen
  /// app for the whole switch, hiding app bars set to make way for one.
  /// And it stays below the switch's windows to the end of its teardown,
  /// its backdrop copies under their uncloaked windows, while a tile
  /// moving meanwhile raises its surface over every window.
  pub fn switch_surface(
    &mut self,
    area: &Rect,
    areas: impl FnOnce() -> Vec<Rect>,
  ) -> Option<&PeekSurface> {
    if let Some(index) = self
      .switch_surfaces
      .iter()
      .position(|surface| surface.clip() == area)
    {
      return self.switch_surfaces.get(index);
    }

    let surface = PeekSurface::new(area)
      .inspect_err(|err| {
        tracing::warn!("Failed to create a switch surface: {err}");
      })
      .ok()?;
    let areas = areas();
    self.switch_surfaces.retain(|old| {
      !is_stale_surface(old.is_unused(), old.clip(), &areas)
    });
    self.switch_surfaces.push(surface);
    self.switch_surfaces.last()
  }

  /// Drops the switch surfaces left unused off `areas`, every monitor's
  /// working area, as [`retain_surfaces`](Self::retain_surfaces) does
  /// tiles'. One a switch still draws on stays until that has torn down.
  pub fn retain_switch_surfaces(&mut self, areas: &[Rect]) {
    self.switch_surfaces.retain(|surface| {
      !is_stale_surface(surface.is_unused(), surface.clip(), areas)
    });
  }

  /// Opens or closes a pass of `raise_moving` calls, within which a
  /// surface already raised over everything since is not raised again
  /// (see `PeekSurface::set_raise_pass`). Tiles put on top after one
  /// closes rank over those it raised (see [`Rank::pass`]).
  pub fn set_raise_pass(&mut self, is_open: bool) {
    for surface in &self.surfaces {
      surface.set_raise_pass(is_open);
    }
    if !is_open {
      self.raise_passes += 1;
    }
  }

  /// Notes that every surface but the one `raised` is on may be under
  /// something just raised (see `PeekSurface::note_covered`).
  fn note_raised(&self, raised: Option<HWND>) {
    for surface in &self.surfaces {
      if Some(surface.hwnd()) != raised {
        surface.note_covered();
      }
    }
  }

  /// Ranks the peeks of tiles closing or minimizing on a surface, put on
  /// top in an earlier raise pass, over the pass now open, before a tile
  /// is ranked in it (see [`reranks_leaving`]): whichever sync moves the
  /// tiles reflowing for them, those never rank over them. Each is
  /// registered anew, once, if under them, by the caller's
  /// [`restack_crossings`](Self::restack_crossings).
  pub(super) fn keep_leaving_on_top(&mut self) {
    let leaving = self
      .entries
      .iter()
      .filter(|(_, entry)| {
        reranks_leaving(
          entry.role,
          entry.tenancy.as_ref().map(|tenancy| tenancy.rank),
          self.raise_passes,
        )
      })
      .map(|(id, _)| *id)
      .collect::<Vec<_>>();

    for id in leaving {
      let rank = self.take_rank(Stacking::Transit);
      if let Some(tenancy) = self
        .entries
        .get_mut(&id)
        .and_then(|entry| entry.tenancy.as_mut())
      {
        tenancy.rank = rank;
      }
      tracing::debug!(
        "Leaving {id} ranked over the moves of the pass now open."
      );
    }
  }

  /// Puts `window_id`'s moving peek on top, so no window it passes hides
  /// it. A peek goes up just below its real window: a restored one was at
  /// the bottom of the z-order, under the windows it comes back over, and
  /// a moved one may be under windows or stand-ins it passes. Not one
  /// leaving, which stays
  /// below its neighbours (task 4). A hosted one's surface, raised in the
  /// open pass with nothing raised since, stays (see
  /// [`set_raise_pass`](Self::set_raise_pass)).
  pub fn raise_moving(&self, window_id: &Uuid) {
    if let Some(entry) = self
      .entries
      .get(window_id)
      .filter(|entry| matches!(entry.role, Role::Moving { .. }))
    {
      // A crossing's second surface, raised after the first, may be over
      // it from then on: each raise notes the others covered.
      for peek in std::iter::once(&entry.peek)
        .chain(entry.trail.as_ref().map(|trail| &trail.peek))
      {
        peek.raise_to_top();
        self.note_raised(peek.is_hosted().then(|| peek.hwnd()));
      }
    }
  }

  /// Puts `window_id`'s moving peek at the top of the band `raise` names
  /// (see [`floating_peek_raise`](super::motions::floating_peek_raise)). A
  /// hosted one's surface is that band's (see [`host_for`]), raised with
  /// every peek on it, and so is its trail's, hosted, on the monitor it
  /// comes from. A trail of its own is left where it went up.
  pub fn raise_in_band(&self, window_id: &Uuid, raise: PeekRaise) {
    if let Some(entry) = self
      .entries
      .get(window_id)
      .filter(|entry| matches!(entry.role, Role::Moving { .. }))
    {
      let topmost = raise == PeekRaise::TopMost;
      entry.peek.raise_to_top_of_band(topmost);
      if let Some(trail) = entry.hosted_trail() {
        trail.raise_to_top_of_band(topmost);
      }
      self.note_raised(None);
    }
  }
}

#[cfg(test)]
mod tests {

  use wm_platform::{LengthValue, Rect, RectDelta, HWND};

  use super::{
    by_surface, crossing_host, host_for, hosts_hold, hosts_on_surface,
    keeps_above_neighbours, keeps_rest_hosted, lanes_meet,
    lanes_stay_apart, lowest, out_of_order, rank, ranks_before_pass,
    reranks_leaving, rest_anchor, rests, Host, Lane, Layer, Level, Rank,
    Stacking,
  };
  use crate::animation::{
    peek_view::{motions::PeekRaise, Part, Role},
    AwayTransition,
  };

  #[test]
  fn only_plain_tiles_go_on_a_surface() {
    assert!(hosts_on_surface(false, false));
    assert!(!hosts_on_surface(true, false));
    assert!(!hosts_on_surface(false, true));
  }

  /// Both parts go on surfaces, unless a grab holds a tile on either
  /// monitor's tiles' surface, which a raise there would top.
  #[test]
  fn a_crossing_tile_is_hosted_unless_a_grab_holds_tiles_there() {
    let band = Some(Host::InBand(Layer::TopBand));
    for host in [Some(Host::Tiles), Some(Host::InBand(Layer::Tiles)), band]
    {
      assert_eq!(crossing_host(host, false), host);
    }
    assert_eq!(crossing_host(Some(Host::Tiles), true), None);
    assert_eq!(
      crossing_host(Some(Host::InBand(Layer::Tiles)), true),
      None
    );
    assert_eq!(crossing_host(band, true), band);
    assert_eq!(crossing_host(None, false), None);
  }

  /// A tile on two surfaces is restacked on each, by its one rank.
  #[test]
  fn tenants_are_grouped_per_surface_in_rank_order() {
    let rank = |pass, order| Rank {
      pass,
      level: Level::Raised,
      order,
    };
    let groups = by_surface(vec![
      (2, rank(1, 3), ('a', Part::Trail)),
      (1, rank(1, 3), ('a', Part::Own)),
      (1, rank(1, 1), ('b', Part::Own)),
      (2, rank(0, 7), ('c', Part::Own)),
    ]);
    assert_eq!(
      groups,
      vec![
        vec![('b', Part::Own), ('a', Part::Own)],
        vec![('c', Part::Own), ('a', Part::Trail)],
      ]
    );
  }

  /// A crossing tile's overlays go under the lower of its two surfaces.
  #[test]
  fn the_lowest_surface_anchors_the_overlays() {
    // Higher numbers are higher in the z-order.
    let is_below = |a: u8, b: u8| a < b;
    assert_eq!(lowest([3, 1], is_below), Some(1));
    assert_eq!(lowest([1, 3], is_below), Some(1));
    assert_eq!(lowest([2], is_below), Some(2));
    assert_eq!(lowest(std::iter::empty::<u8>(), is_below), None);
  }

  /// As their own peeks play: in the band their pass raises them in, or
  /// with the tiles; one left where it went up keeps a peek of its own.
  #[test]
  fn band_raises_pick_the_surface() {
    for is_tile in [true, false] {
      assert_eq!(
        host_for(is_tile, Some(PeekRaise::TopMost)),
        Some(Host::InBand(Layer::TopBand))
      );
      assert_eq!(
        host_for(is_tile, Some(PeekRaise::Top)),
        Some(Host::InBand(Layer::Tiles))
      );
    }
    assert_eq!(host_for(true, None), Some(Host::Tiles));
    assert_eq!(host_for(false, None), None);
  }

  /// Raised in its band after its pass's raises: over them, under the next
  /// pass's, in the order such raises come.
  #[test]
  fn band_raises_rank_after_their_pass() {
    let over = rank(Stacking::OverPass, 3, 10);
    for under in [
      rank(Stacking::Raised, 3, 11),
      rank(Stacking::InPass, 3, 12),
      rank(Stacking::Transit, 3, 13),
    ] {
      assert!(under < over);
    }
    assert!(over < rank(Stacking::InPass, 4, 11));
    assert!(over < rank(Stacking::Raised, 4, 11));
    assert!(over < rank(Stacking::OverPass, 3, 11));
    assert_eq!(
      Host::InBand(Layer::TopBand).stacking(Stacking::Raised),
      Stacking::OverPass
    );
    assert_eq!(Host::Tiles.stacking(Stacking::Transit), Stacking::Transit);
  }

  /// A move raised after its pass's columns, a later pass over an earlier
  /// one; an opening or leaving tile over every move of its pass and
  /// before, several in the order they started.
  #[test]
  fn tiles_rank_by_when_their_peeks_go_on_top() {
    let raised = rank(Stacking::Raised, 3, 10);
    let column = rank(Stacking::InPass, 3, 11);
    assert!(column < raised);
    assert!(rank(Stacking::Raised, 2, 12) < column);
    assert!(raised < rank(Stacking::InPass, 4, 9));
    assert!(raised < rank(Stacking::Raised, 3, 12));

    let transit = rank(Stacking::Transit, 3, 8);
    for under in [raised, column, rank(Stacking::Raised, 3, 30)] {
      assert!(under < transit);
    }
    assert!(transit < rank(Stacking::Transit, 3, 9));
    assert!(transit < rank(Stacking::Raised, 4, 0));
    assert!(transit < rank(Stacking::OverPass, 3, 0));
  }

  fn lane(from: (i32, i32, i32, i32), to: (i32, i32, i32, i32)) -> Lane {
    Lane {
      from: Rect::from_ltrb(from.0, from.1, from.2, from.3),
      to: Rect::from_ltrb(to.0, to.1, to.2, to.3),
    }
  }

  /// Neighbours resized across their shared edge keep their gap all the
  /// way, an overshooting easing's too.
  #[test]
  fn resized_neighbours_stay_apart() {
    let left = lane((0, 0, 500, 900), (0, 0, 700, 900));
    let right = lane((510, 0, 1000, 900), (710, 0, 1000, 900));

    assert!(lanes_stay_apart(&left, &right, false));
    assert!(lanes_stay_apart(&right, &left, true));
  }

  /// Tiles that swap places cross on the way.
  #[test]
  fn swapped_tiles_do_not_stay_apart() {
    let left = lane((0, 0, 500, 900), (510, 0, 1000, 900));
    let right = lane((510, 0, 1000, 900), (0, 0, 500, 900));

    assert!(!lanes_stay_apart(&left, &right, false));
  }

  /// A stacked tile's lane takes in its tab bar.
  #[test]
  fn a_stacked_tiles_lane_takes_in_its_bar() {
    use wm_platform::TabBarPlace;

    use super::Reach;

    let window = Rect::from_ltrb(0, 28, 400, 400);
    let bar =
      TabBarPlace::between(&Rect::from_ltrb(0, 0, 400, 36), &window, true);
    let reach = |bar: Option<TabBarPlace>| Reach {
      shadow_borders: RectDelta::new(
        LengthValue::from_px(0),
        LengthValue::from_px(0),
        LengthValue::from_px(0),
        LengthValue::from_px(0),
      ),
      bar,
      ring: 2,
    };

    let moved = Rect::from_ltrb(500, 28, 900, 400);
    assert_eq!(
      reach(Some(bar)).lane(&window, &moved),
      Lane {
        from: Rect::from_ltrb(-2, -2, 402, 402),
        to: Rect::from_ltrb(498, -2, 902, 402),
      }
    );
    assert_eq!(reach(None).lane(&window, &moved).from.top, 26);
  }

  /// A held tile goes on its surface unless that is at the top of its
  /// band, over the floating windows a held peek stays under.
  #[test]
  fn a_held_tile_goes_on_a_surface_not_raised_to_the_top() {
    assert!(hosts_hold(false));
    assert!(!hosts_hold(true));
  }

  /// A surface has one place at rest: a second
  /// resting column keeps a peek of its own; a moving one ending shown
  /// always goes.
  #[test]
  fn one_resting_column_per_surface() {
    assert!(keeps_rest_hosted(true, false));
    assert!(!keeps_rest_hosted(true, true));
    assert!(keeps_rest_hosted(false, false));
    assert!(keeps_rest_hosted(false, true));
    assert!(rests(Role::Peek));
    assert!(rests(Role::Moving { ends_shown: false }));
    assert!(!rests(Role::Moving { ends_shown: true }));
    assert!(!rests(Role::HandingBack));
  }

  /// A surface goes to its resting tenant's window only once every tenant
  /// rests, and it has one.
  #[test]
  fn a_surface_rests_by_its_one_resting_tenant() {
    let window = HWND(7);
    assert_eq!(rest_anchor([(Role::Peek, window)]), Some(window));
    assert_eq!(rest_anchor([]), None);
    assert_eq!(
      rest_anchor([
        (Role::Peek, window),
        (Role::Moving { ends_shown: true }, HWND(8)),
      ]),
      None
    );
    assert_eq!(
      rest_anchor([(Role::Releasing { settled: true }, HWND(8))]),
      None
    );
    assert_eq!(
      rest_anchor([(Role::Peek, window), (Role::Peek, HWND(8))]),
      None
    );
  }

  /// Only a tile leaving on a surface keeps it above its neighbours, and
  /// only while its transit runs.
  #[test]
  fn a_leaving_tile_keeps_its_surface_above_while_it_plays() {
    let close = Role::Leaving(AwayTransition::Close);
    let minimize = Role::Leaving(AwayTransition::Minimize);

    assert!(keeps_above_neighbours(close, true, true));
    assert!(keeps_above_neighbours(minimize, true, true));
    assert!(!keeps_above_neighbours(close, false, true));
    assert!(!keeps_above_neighbours(close, true, false));
    assert!(!keeps_above_neighbours(
      Role::Moving { ends_shown: true },
      true,
      true
    ));
  }

  /// A close or minimize ranked in an earlier raise pass is ranked again
  /// before a tile is ranked in the one now open; a moving tile, one
  /// ranked in it, and one on a peek of its own are not.
  #[test]
  fn a_leaving_tile_ranked_in_an_earlier_pass_is_ranked_again() {
    let earlier = Some(rank(Stacking::Transit, 3, 0));
    let current = Some(rank(Stacking::Transit, 4, 1));
    let minimize = Role::Leaving(AwayTransition::Minimize);
    let close = Role::Leaving(AwayTransition::Close);

    assert!(reranks_leaving(minimize, earlier, 4));
    assert!(reranks_leaving(close, earlier, 4));
    assert!(!reranks_leaving(minimize, current, 4));
    assert!(!reranks_leaving(minimize, None, 4));
    assert!(!reranks_leaving(
      Role::Moving { ends_shown: true },
      earlier,
      4
    ));
  }

  /// A minimize ranked in an earlier raise pass is re-ranked for the moves
  /// of the one now open; one ranked in it is not.
  #[test]
  fn a_minimize_ranked_in_an_earlier_pass_is_ranked_again() {
    let earlier = rank(Stacking::Transit, 3, 0);
    let current = rank(Stacking::Transit, 4, 1);

    assert!(ranks_before_pass(earlier, 4));
    assert!(!ranks_before_pass(current, 4));
    // Ranked again, it goes over the open pass's moves.
    assert!(rank(Stacking::Raised, 4, 2) < current);
  }

  /// On one clock, only lanes that don't stay apart meet; otherwise any
  /// whose hulls overlap, and any at all past an overshooting easing.
  #[test]
  fn lanes_meet_where_their_tiles_may_overlap() {
    let left = lane((0, 0, 500, 900), (0, 0, 700, 900));
    let right = lane((510, 0, 1000, 900), (710, 0, 1000, 900));
    let below = lane((0, 1000, 500, 1400), (0, 1000, 500, 1400));

    assert!(!lanes_meet(&left, &right, true, false));
    assert!(lanes_meet(&left, &right, false, false));
    assert!(!lanes_meet(&left, &below, false, false));
    assert!(lanes_meet(&left, &below, false, true));
    assert!(!lanes_meet(&left.landed(), &right.landed(), false, false));
  }

  /// A tile under part of a lower-ranked one it meets goes over it, and
  /// then over it every higher one it meets; ones in order, or apart,
  /// stay.
  #[test]
  fn tiles_under_lower_ranked_ones_they_meet_are_restacked() {
    let all = |_: usize, _: usize| true;
    let none = |_: usize, _: usize| false;

    // Registered in rank order: nothing to do.
    let in_order = [Some((0, 3)), Some((4, 6)), Some((7, 9))];
    assert!(out_of_order(&in_order, all).is_empty());

    // The lowest's ring registered over the others' windows.
    let late_ring = [Some((0, 12)), Some((4, 6)), Some((7, 9))];
    assert_eq!(out_of_order(&late_ring, all), vec![1, 2]);
    assert!(out_of_order(&late_ring, none).is_empty());
    // Only those meeting the lowest go.
    assert_eq!(
      out_of_order(&late_ring, |lower, _| lower == 0),
      vec![1, 2]
    );
    assert_eq!(
      out_of_order(&late_ring, |lower, upper| (lower, upper) == (0, 2)),
      vec![2]
    );

    // A reused one ranked on top with registrations kept from before.
    let reused = [Some((5, 9)), Some((0, 3))];
    assert_eq!(out_of_order(&reused, all), vec![1]);

    // One restacked goes over the ones above that meet it.
    let chain = [Some((0, 5)), Some((3, 4)), Some((6, 8))];
    assert_eq!(
      out_of_order(&chain, |lower, upper| upper == lower + 1),
      vec![1, 2]
    );
    assert!(out_of_order(&[None, Some((0, 1))], all).is_empty());
  }

  /// A gap that changes holds for a progress in 0..1 only, and by a pixel
  /// at least, as each edge is rounded on its own.
  #[test]
  fn a_changing_gap_holds_only_without_overshoot() {
    let still = lane((0, 0, 500, 400), (0, 0, 500, 400));
    let leaving = lane((0, 420, 500, 800), (0, 600, 500, 800));
    let touching = lane((0, 400, 500, 800), (0, 600, 500, 800));

    assert!(lanes_stay_apart(&still, &leaving, false));
    assert!(!lanes_stay_apart(&still, &leaving, true));
    assert!(!lanes_stay_apart(&still, &touching, false));
  }

  /// Edges that only touch, at both ends alike, are apart.
  #[test]
  fn touching_tiles_stay_apart() {
    let top = lane((0, 0, 500, 400), (100, 0, 600, 400));
    let bottom = lane((0, 400, 500, 800), (100, 400, 600, 800));

    assert!(lanes_stay_apart(&top, &bottom, true));
  }
}
