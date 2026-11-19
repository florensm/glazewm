//! Edge colors of windows, which their stand-ins fill with while the
//! windows are resized.

use std::time::Duration;

use wm_common::ResizeStyle;
use wm_platform::{sample_edge_color_async, Color, HWND};

use super::AnimationManager;
use crate::user_config::UserConfig;

/// How long a sampled surrogate backdrop color stays valid per window.
///
/// Sampling costs two GPU→CPU `BitBlt` readbacks; now always done on a
/// background thread via `sample_edge_color_async` rather than blocking
/// the WM's main thread, but a miss still means the *current* motion
/// plays with a transparent backdrop instead of a matched color. App
/// background colors change rarely (theme switches), so a stale hit is a
/// cosmetic-only risk -- kept generous so a window resized only
/// occasionally still benefits from the earlier background sample instead
/// of re-paying for one every time.
const EDGE_COLOR_CACHE_TTL: Duration = Duration::from_secs(300);

/// Cache-size threshold above which stale edge-color entries are pruned.
const EDGE_COLOR_CACHE_PRUNE_LEN: usize = 128;

impl AnimationManager {
  /// Queues a first edge-color sample for a window just shown that has
  /// none, so its first `fill` resize, or grab resize on peeks, has a
  /// fill.
  ///
  /// Taken at the next idle point like a resize's, by then after any open
  /// animation has landed; never at a resize's start (see
  /// `sample_pending_edge_colors`).
  pub fn queue_first_edge_sample(
    &mut self,
    hwnd: HWND,
    config: &UserConfig,
  ) {
    let resize = &config.value.animations.window_resize;
    let grab = &config.value.general.grab_and_move;
    let grab_fills = grab.enabled
      && grab.resize
      && grab.draw == wm_common::GrabDraw::Peeks;
    let fills = (resize.enabled && resize.style != ResizeStyle::Stretch)
      || grab_fills;
    if fills && self.cached_edge_color(hwnd.0).is_none() {
      self.pending_edge_samples.insert(hwnd.0);
    }
  }

  /// Samples the edge color of every window that animated, or was shown
  /// with none yet, since the last idle point.
  ///
  /// Deferred to here because the screen readback stalls DWM: run at a
  /// motion's start, it more than tripled keypress-to-motion latency.
  pub(super) fn sample_pending_edge_colors(&mut self) {
    for hwnd in self.pending_edge_samples.drain() {
      sample_edge_color_async(
        HWND(hwnd),
        self.edge_color_cache.clone(),
        EDGE_COLOR_CACHE_PRUNE_LEN,
        EDGE_COLOR_CACHE_TTL,
        self.animation_timer_running.clone(),
      );
    }
  }

  /// Returns the last sampled surrogate backdrop color for `hwnd`,
  /// however old it is.
  ///
  /// Age is not consulted: the color only ever stands in for window
  /// content in a strip the thumbnail has not reached yet, and an app's
  /// background changes about as often as its theme -- so the previous
  /// sample beats none, which leaves that strip showing the backdrop raw
  /// and undimmed. Callers re-sample after every motion (see
  /// [`sample_edge_color_async`]), so this entry is never more than one
  /// animation behind the window.
  ///
  /// `sample_edge_color_async`'s background thread populates this same map
  /// directly (see `edge_color_cache`'s doc comment), so a lock failure
  /// here is treated the same as a miss rather than propagated.
  pub(in crate::animation) fn cached_edge_color(
    &self,
    hwnd: isize,
  ) -> Option<Color> {
    let map = self.edge_color_cache.lock().ok()?;
    map.get(&hwnd).map(|(color, _)| *color)
  }
}

#[cfg(test)]
mod tests {
  use std::time::Instant;

  use tokio::sync::mpsc;
  use wm_common::ParsedConfig;

  use super::*;

  /// A window just shown is queued for a first fill color only while a
  /// `fill` resize could use one and it has none yet.
  #[test]
  fn a_shown_window_is_queued_for_a_fill_color_it_lacks() {
    let queues = |edit: fn(&mut ParsedConfig), cached: bool| {
      let (tx, _rx) = mpsc::unbounded_channel();
      let mut manager = AnimationManager::new(tx);
      if cached {
        if let Ok(mut map) = manager.edge_color_cache.lock() {
          let color = Color {
            r: 1,
            g: 2,
            b: 3,
            a: 255,
          };
          map.insert(7, (color, Instant::now()));
        }
      }
      let mut parsed = ParsedConfig::default();
      edit(&mut parsed);
      let config = UserConfig::from_parsed(parsed);

      manager.queue_first_edge_sample(HWND(7), &config);
      manager.pending_edge_samples.contains(&7)
    };

    assert!(queues(|_| {}, false));
    assert!(!queues(|_| {}, true));
    assert!(!queues(
      |parsed| parsed.animations.window_resize.enabled = false,
      false
    ));
    assert!(!queues(
      |parsed| parsed.animations.window_resize.style =
        ResizeStyle::Stretch,
      false
    ));
  }
}
