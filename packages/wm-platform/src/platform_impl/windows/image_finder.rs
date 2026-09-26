//! Finds the images in a themed window through UI Automation, so the color
//! theme can leave pictures in their own colors.
//!
//! Queries run on a dedicated thread: they are cross-process calls into
//! the app's UI thread, which can be slow or hang with the app. They are
//! kept rare: a query only follows a change of the window's content (a
//! captured frame arrived), once it has settled, and never sooner than
//! [`MIN_INTERVAL`] after the last one. An idle window costs nothing.

use std::{
  mem::ManuallyDrop,
  sync::{Arc, Condvar, Mutex, PoisonError},
  time::{Duration, Instant},
};

use windows::{
  core::ComInterface,
  Win32::{
    Foundation::{HWND, RECT, VARIANT_TRUE},
    Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
    System::{
      Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize,
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
      },
      Variant::{
        VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_BOOL, VT_I4,
      },
    },
    UI::Accessibility::{
      AutomationElementMode_None, CUIAutomation, CUIAutomation8,
      IUIAutomation, IUIAutomation2, IUIAutomationCacheRequest,
      IUIAutomationElement, TreeScope_Subtree,
      UIA_BoundingRectanglePropertyId, UIA_ControlTypePropertyId,
      UIA_ImageControlTypeId, UIA_IsOffscreenPropertyId,
      UIA_IsScrollPatternAvailablePropertyId,
    },
  },
};

use crate::Rect;

/// Shortest time between two queries of the same window.
const MIN_INTERVAL: Duration = Duration::from_secs(3);

/// How long the content must stay unchanged before a query, so scrolling
/// or typing is looked at once when it stops rather than throughout.
const SETTLE: Duration = Duration::from_millis(500);

/// Longest a change waits for the content to settle; bounds how stale
/// the images get in a window that never stops animating.
const MAX_WAIT: Duration = Duration::from_secs(10);

/// How long a single query may block on an unresponsive app.
const QUERY_TIMEOUT_MS: u32 = 1000;

/// An image a query found, relative to the window's frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FoundImage {
  /// The part of it in view.
  pub rect: Rect,

  /// The view it's shown in: the intersection of the scrollable areas
  /// around it, beyond which it's cut off when drawn.
  pub view: Rect,
}

/// Watches one window for its images, reporting their rects relative to
/// the window's captured frame to a callback on its own thread.
///
/// The thread stops on [`stop`](Self::stop) or drop, once any query in
/// flight returns; neither waits on it.
pub(crate) struct ImageFinder {
  shared: Arc<Shared>,
}

struct Shared {
  state: Mutex<FinderState>,
  wake: Condvar,
}

struct FinderState {
  /// When the content first changed since the last query, if it did.
  changed_since: Option<Instant>,

  /// When the content last changed.
  last_change: Instant,

  stopped: bool,
}

impl ImageFinder {
  /// Starts watching `source`. `on_query` receives the images found by
  /// every query, relative to the window's frame (its
  /// `DWMWA_EXTENDED_FRAME_BOUNDS`, which the capture starts at).
  pub(crate) fn start(
    source: HWND,
    on_query: impl FnMut(Vec<FoundImage>) + Send + 'static,
  ) -> crate::Result<Self> {
    let shared = Arc::new(Shared {
      state: Mutex::new(FinderState {
        changed_since: Some(Instant::now()),
        last_change: Instant::now(),
        stopped: false,
      }),
      wake: Condvar::new(),
    });

    let thread_shared = shared.clone();
    let source_raw = source.0;

    std::thread::Builder::new()
      .name("color-theme-images".to_string())
      .spawn(move || {
        if let Err(err) = run(HWND(source_raw), &thread_shared, on_query) {
          tracing::debug!("Image finder stopped: {err}.");
        }
      })?;

    Ok(Self { shared })
  }

  /// Marks the window's content as changed, so it is queried again.
  pub(crate) fn content_changed(&self) {
    let mut state = self
      .shared
      .state
      .lock()
      .unwrap_or_else(PoisonError::into_inner);
    let now = Instant::now();
    state.changed_since.get_or_insert(now);
    state.last_change = now;
    self.shared.wake.notify_one();
  }

  /// Stops the thread once any query in flight returns.
  pub(crate) fn stop(&self) {
    let mut state = self
      .shared
      .state
      .lock()
      .unwrap_or_else(PoisonError::into_inner);
    state.stopped = true;
    self.shared.wake.notify_one();
  }
}

impl Drop for ImageFinder {
  fn drop(&mut self) {
    self.stop();
  }
}

/// Uninitializes COM for the thread when dropped.
struct ComApartment;

impl ComApartment {
  fn enter() -> crate::Result<Self> {
    // SAFETY: Called once on a thread this module owns; UI Automation
    // clients are meant to run in the multithreaded apartment.
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED)? };
    Ok(Self)
  }
}

impl Drop for ComApartment {
  fn drop(&mut self) {
    // SAFETY: Balances the successful `CoInitializeEx` in `enter`, after
    // every COM object of the thread was released.
    unsafe { CoUninitialize() };
  }
}

/// The finder thread: waits for content changes and queries on them.
fn run(
  source: HWND,
  shared: &Shared,
  mut on_query: impl FnMut(Vec<FoundImage>),
) -> crate::Result<()> {
  let _apartment = ComApartment::enter()?;
  let query = ImageQuery::new()?;
  let mut root = None;
  let mut last_query: Option<Instant> = None;

  loop {
    {
      let mut state =
        shared.state.lock().unwrap_or_else(PoisonError::into_inner);

      loop {
        if state.stopped {
          return Ok(());
        }

        let Some(changed_since) = state.changed_since else {
          state = shared
            .wake
            .wait(state)
            .unwrap_or_else(PoisonError::into_inner);
          continue;
        };

        // Re-evaluated on every change, which can move it.
        let deadline =
          next_query_at(changed_since, state.last_change, last_query);
        let now = Instant::now();

        if now >= deadline {
          break;
        }

        state = shared
          .wake
          .wait_timeout(state, deadline - now)
          .unwrap_or_else(PoisonError::into_inner)
          .0;
      }

      state.changed_since = None;
    }

    last_query = Some(Instant::now());

    match query.find(source, &mut root) {
      Ok(images) => on_query(images),
      Err(err) => {
        // The window may be closing or busy; retry on the next change.
        tracing::debug!("Image query failed: {err}.");
        root = None;
      }
    }
  }
}

/// When to query after the content changed since `changed_since` and
/// last changed at `last_change`: once it settled (or waited long
/// enough), and spaced out from the `last_query`.
fn next_query_at(
  changed_since: Instant,
  last_change: Instant,
  last_query: Option<Instant>,
) -> Instant {
  let settled = (last_change + SETTLE).min(changed_since + MAX_WAIT);
  last_query.map_or(settled, |at| settled.max(at + MIN_INTERVAL))
}

/// The UI Automation objects a query needs, created once per thread.
struct ImageQuery {
  automation: IUIAutomation,
  cache: IUIAutomationCacheRequest,
}

impl ImageQuery {
  fn new() -> crate::Result<Self> {
    // SAFETY: COM is initialized on this thread by `ComApartment`.
    let automation: IUIAutomation = unsafe {
      CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)
        .or_else(|_| {
          CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
        })?
    };

    // Windows 8+; without it, the system's default timeouts apply.
    if let Ok(automation) = automation.cast::<IUIAutomation2>() {
      // SAFETY: Plain setters on a live object.
      unsafe {
        let _ = automation.SetConnectionTimeout(QUERY_TIMEOUT_MS);
        let _ = automation.SetTransactionTimeout(QUERY_TIMEOUT_MS);
      }
    }

    // SAFETY: `automation` is live; the variants hold plain values, which
    // need no clearing.
    let cache = unsafe {
      // Images, and the scrollable areas that cut them off: UI Automation
      // reports an image's full bounds even where one of those hides it.
      let images = automation.CreatePropertyCondition(
        UIA_ControlTypePropertyId,
        int_variant(i32::try_from(UIA_ImageControlTypeId.0)?),
      )?;
      let scrollables = automation.CreatePropertyCondition(
        UIA_IsScrollPatternAvailablePropertyId,
        true_variant(),
      )?;

      // One round trip caches just those elements, as a tree of their own
      // (the rest is skipped over), with three properties each and no
      // live element references kept.
      let cache = automation.CreateCacheRequest()?;
      cache.SetTreeScope(TreeScope_Subtree)?;
      cache.SetTreeFilter(
        &automation.CreateOrCondition(&images, &scrollables)?,
      )?;
      cache.AddProperty(UIA_BoundingRectanglePropertyId)?;
      cache.AddProperty(UIA_IsOffscreenPropertyId)?;
      cache.AddProperty(UIA_ControlTypePropertyId)?;
      cache.SetAutomationElementMode(AutomationElementMode_None)?;
      cache
    };

    Ok(Self { automation, cache })
  }

  /// The images of `source` in view, relative to its frame, largest
  /// first and at most `MAX_IMAGE_RECTS`. `root` caches the window's
  /// element across queries.
  fn find(
    &self,
    source: HWND,
    root: &mut Option<IUIAutomationElement>,
  ) -> crate::Result<Vec<FoundImage>> {
    let frame = extended_frame_bounds(source)?;
    let frame =
      Rect::from_ltrb(frame.left, frame.top, frame.right, frame.bottom);

    let element = if let Some(element) = root {
      element.clone()
    } else {
      // SAFETY: A stale `source` just fails.
      let element = unsafe { self.automation.ElementFromHandle(source)? };
      *root = Some(element.clone());
      element
    };

    // SAFETY: Both objects are live.
    let tree = unsafe { element.BuildUpdatedCache(&self.cache)? };

    let mut images = Vec::new();
    collect_images(&tree, &frame, &mut images)?;

    // Relative to the frame.
    for image in &mut images {
      image.rect = image.rect.translate_to_coordinates(
        image.rect.left - frame.left,
        image.rect.top - frame.top,
      );
      image.view = image.view.translate_to_coordinates(
        image.view.left - frame.left,
        image.view.top - frame.top,
      );
    }

    images.sort_by_key(|image| {
      std::cmp::Reverse(
        i64::from(image.rect.width()) * i64::from(image.rect.height()),
      )
    });
    images.truncate(super::color_capture::MAX_IMAGE_RECTS);

    Ok(images)
  }
}

/// Adds the images under `node` of a cached tree of images and scrollable
/// areas to `images`, each cut down to `view` and the scrollable areas
/// between it and `node`.
fn collect_images(
  node: &IUIAutomationElement,
  view: &Rect,
  images: &mut Vec<FoundImage>,
) -> crate::Result<()> {
  // SAFETY: `node` is a live element from a cache request that included
  // its children and these three properties.
  let Ok(children) = (unsafe { node.GetCachedChildren() }) else {
    return Ok(());
  };

  // SAFETY: `children` is a live element array.
  for index in 0..unsafe { children.Length()? } {
    // SAFETY: `index` is within the array's length, and the properties
    // were cached.
    let (child, control_type, offscreen, bounds) = unsafe {
      let child = children.GetElement(index)?;
      let control_type = child.CachedControlType()?;
      let offscreen = child.CachedIsOffscreen()?.as_bool();
      let bounds = child.CachedBoundingRectangle()?;
      (child, control_type, offscreen, bounds)
    };

    let bounds = Rect::from_ltrb(
      bounds.left,
      bounds.top,
      bounds.right,
      bounds.bottom,
    );
    let Some(visible) = intersection(&bounds, view) else {
      continue;
    };

    if control_type == UIA_ImageControlTypeId {
      if !offscreen {
        images.push(FoundImage {
          rect: visible,
          view: view.clone(),
        });
      }
    } else {
      // A scrollable area: what's under it is cut off at its bounds.
      collect_images(&child, &visible, images)?;
    }
  }

  Ok(())
}

/// The overlap of `a` and `b`, if any.
fn intersection(a: &Rect, b: &Rect) -> Option<Rect> {
  let rect = Rect::from_ltrb(
    a.left.max(b.left),
    a.top.max(b.top),
    a.right.min(b.right),
    a.bottom.min(b.bottom),
  );

  (rect.width() > 0 && rect.height() > 0).then_some(rect)
}

/// A `VT_BOOL` true variant.
fn true_variant() -> VARIANT {
  VARIANT {
    Anonymous: VARIANT_0 {
      Anonymous: ManuallyDrop::new(VARIANT_0_0 {
        vt: VT_BOOL,
        wReserved1: 0,
        wReserved2: 0,
        wReserved3: 0,
        Anonymous: VARIANT_0_0_0 {
          boolVal: VARIANT_TRUE,
        },
      }),
    },
  }
}

/// A `VT_I4` variant, as UI Automation takes control type ids.
fn int_variant(value: i32) -> VARIANT {
  VARIANT {
    Anonymous: VARIANT_0 {
      Anonymous: ManuallyDrop::new(VARIANT_0_0 {
        vt: VT_I4,
        wReserved1: 0,
        wReserved2: 0,
        wReserved3: 0,
        Anonymous: VARIANT_0_0_0 { lVal: value },
      }),
    },
  }
}

/// `source`'s frame without its invisible resize borders, where its
/// capture starts; UI Automation reports rects in the same physical
/// screen pixels.
fn extended_frame_bounds(source: HWND) -> crate::Result<RECT> {
  let mut rect = RECT::default();

  // SAFETY: `rect` is exactly the size passed and outlives the call.
  unsafe {
    DwmGetWindowAttribute(
      source,
      DWMWA_EXTENDED_FRAME_BOUNDS,
      std::ptr::from_mut(&mut rect).cast(),
      u32::try_from(std::mem::size_of::<RECT>())?,
    )?;
  }

  Ok(rect)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn queries_once_the_content_settles() {
    let start = Instant::now();

    // Scrolling for a second: looked at half a second after it stops.
    let last_change = start + Duration::from_secs(1);
    assert_eq!(
      next_query_at(start, last_change, None),
      last_change + SETTLE
    );
  }

  #[test]
  fn never_queries_sooner_than_the_interval() {
    let start = Instant::now();

    assert_eq!(
      next_query_at(start, start, Some(start)),
      start + MIN_INTERVAL,
    );
  }

  #[test]
  fn a_window_that_never_settles_still_gets_queried() {
    let start = Instant::now();
    let last_change = start + Duration::from_secs(60);

    assert_eq!(next_query_at(start, last_change, None), start + MAX_WAIT,);
  }
}
