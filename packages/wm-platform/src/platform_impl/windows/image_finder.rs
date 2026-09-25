//! Finds the images in a themed window through UI Automation, so the color
//! theme can leave pictures in their own colors.
//!
//! Queries run on a dedicated thread: they are cross-process calls into
//! the app's UI thread, which can be slow or hang with the app. A query
//! only runs after the window's content changed (a captured frame
//! arrived), at most every [`QUERY_INTERVAL`], so an idle window costs
//! nothing and a busy one is never flooded.

use std::{
  mem::ManuallyDrop,
  sync::{Arc, Condvar, Mutex, PoisonError},
  time::{Duration, Instant},
};

use windows::{
  core::ComInterface,
  Win32::{
    Foundation::{HWND, RECT},
    Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
    System::{
      Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize,
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
      },
      Variant::{VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0, VT_I4},
    },
    UI::Accessibility::{
      AutomationElementMode_None, CUIAutomation, CUIAutomation8,
      IUIAutomation, IUIAutomation2, IUIAutomationCacheRequest,
      IUIAutomationCondition, IUIAutomationElement, TreeScope_Descendants,
      UIA_BoundingRectanglePropertyId, UIA_ControlTypePropertyId,
      UIA_ImageControlTypeId, UIA_IsOffscreenPropertyId,
    },
  },
};

use crate::Rect;

/// Shortest time between two queries of the same window. Each query runs
/// on the app's UI thread, so this bounds the load put on it while its
/// content keeps changing (e.g. scrolling).
const QUERY_INTERVAL: Duration = Duration::from_millis(200);

/// How long a single query may block on an unresponsive app.
const QUERY_TIMEOUT_MS: u32 = 1000;

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
  /// The window's content changed since the last query.
  dirty: bool,
  stopped: bool,
}

impl ImageFinder {
  /// Starts watching `source`. `on_query` receives the images found by
  /// every query, relative to the window's frame (its
  /// `DWMWA_EXTENDED_FRAME_BOUNDS`, which the capture starts at).
  pub(crate) fn start(
    source: HWND,
    on_query: impl FnMut(Vec<Rect>) + Send + 'static,
  ) -> crate::Result<Self> {
    let shared = Arc::new(Shared {
      state: Mutex::new(FinderState {
        dirty: true,
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
    state.dirty = true;
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
  mut on_query: impl FnMut(Vec<Rect>),
) -> crate::Result<()> {
  let _apartment = ComApartment::enter()?;
  let query = ImageQuery::new()?;
  let mut root = None;
  let mut last_query: Option<Instant> = None;

  loop {
    {
      let mut state =
        shared.state.lock().unwrap_or_else(PoisonError::into_inner);

      while !state.dirty && !state.stopped {
        state = shared
          .wake
          .wait(state)
          .unwrap_or_else(PoisonError::into_inner);
      }

      // Throttle: wait out the rest of the interval, still stoppable.
      if let Some(deadline) = last_query.map(|at| at + QUERY_INTERVAL) {
        while !state.stopped {
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
      }

      if state.stopped {
        return Ok(());
      }

      state.dirty = false;
    }

    last_query = Some(Instant::now());

    match query.find(source, &mut root) {
      Ok(rects) => on_query(rects),
      Err(err) => {
        // The window may be closing or busy; retry on the next change.
        tracing::debug!("Image query failed: {err}.");
        root = None;
      }
    }
  }
}

/// The UI Automation objects a query needs, created once per thread.
struct ImageQuery {
  automation: IUIAutomation,
  condition: IUIAutomationCondition,
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

    // SAFETY: `automation` is live; the variant holds a plain integer,
    // which needs no clearing.
    let (condition, cache) = unsafe {
      let condition = automation.CreatePropertyCondition(
        UIA_ControlTypePropertyId,
        int_variant(i32::try_from(UIA_ImageControlTypeId.0)?),
      )?;

      // Only the two properties are fetched, in the same round trip as
      // the search, with no live element references kept.
      let cache = automation.CreateCacheRequest()?;
      cache.AddProperty(UIA_BoundingRectanglePropertyId)?;
      cache.AddProperty(UIA_IsOffscreenPropertyId)?;
      cache.SetAutomationElementMode(AutomationElementMode_None)?;

      (condition, cache)
    };

    Ok(Self {
      automation,
      condition,
      cache,
    })
  }

  /// The on-screen images of `source`, relative to its frame, largest
  /// first and at most `MAX_IMAGE_RECTS`. `root` caches the window's
  /// element across queries.
  fn find(
    &self,
    source: HWND,
    root: &mut Option<IUIAutomationElement>,
  ) -> crate::Result<Vec<Rect>> {
    let frame = extended_frame_bounds(source)?;

    let element = if let Some(element) = root {
      element.clone()
    } else {
      // SAFETY: A stale `source` just fails.
      let element = unsafe { self.automation.ElementFromHandle(source)? };
      *root = Some(element.clone());
      element
    };

    // SAFETY: All objects are live, and the cached properties read below
    // are the ones `self.cache` requested.
    let found = unsafe {
      element.FindAllBuildCache(
        TreeScope_Descendants,
        &self.condition,
        &self.cache,
      )?
    };

    // SAFETY: `found` is a live element array.
    let count = unsafe { found.Length()? };
    let mut rects = Vec::new();

    for index in 0..count {
      // SAFETY: `index` is within the array's length, and both
      // properties were cached by the search.
      let (offscreen, bounds) = unsafe {
        let image = found.GetElement(index)?;
        (
          image.CachedIsOffscreen()?.as_bool(),
          image.CachedBoundingRectangle()?,
        )
      };

      if offscreen {
        continue;
      }

      // Relative to the frame, and clipped to it.
      let rect = Rect::from_ltrb(
        bounds.left.max(frame.left) - frame.left,
        bounds.top.max(frame.top) - frame.top,
        bounds.right.min(frame.right) - frame.left,
        bounds.bottom.min(frame.bottom) - frame.top,
      );

      if rect.width() > 0 && rect.height() > 0 {
        rects.push(rect);
      }
    }

    rects.sort_by_key(|rect| {
      std::cmp::Reverse(i64::from(rect.width()) * i64::from(rect.height()))
    });
    rects.truncate(super::color_capture::MAX_IMAGE_RECTS);

    Ok(rects)
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
