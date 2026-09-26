//! Where a themed window's UI elements (images, text boxes, buttons, ...)
//! are, read from its UI Automation tree so a theme can render them
//! differently.
//!
//! Every query runs on one background thread: UIA calls go across
//! processes and are answered by the app's UI thread, so they can take
//! long or hang with the app, and must never block the WM or the frame
//! path. Queries only read (no patterns are invoked, nothing gets focus),
//! run only after the window's content changed, and are throttled by how
//! long the last one took, which bounds the load on the app.

use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex, OnceLock, PoisonError,
  },
  time::{Duration, Instant},
};

use windows::Win32::{
  Foundation::{HWND, RECT},
  Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
  System::{
    Com::{
      CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER,
      COINIT_MULTITHREADED,
    },
    Variant::{VARIANT, VT_I4},
  },
  UI::Accessibility::{
    AutomationElementMode_None, CUIAutomation8, IUIAutomation,
    IUIAutomation2, IUIAutomationCacheRequest, IUIAutomationCondition,
    TreeScope_Descendants, UIA_BoundingRectanglePropertyId,
    UIA_ButtonControlTypeId, UIA_CheckBoxControlTypeId,
    UIA_ComboBoxControlTypeId, UIA_ControlTypePropertyId,
    UIA_DataItemControlTypeId, UIA_DocumentControlTypeId,
    UIA_EditControlTypeId, UIA_HeaderControlTypeId,
    UIA_HeaderItemControlTypeId, UIA_HyperlinkControlTypeId,
    UIA_ImageControlTypeId, UIA_IsOffscreenPropertyId,
    UIA_ListItemControlTypeId, UIA_MenuItemControlTypeId,
    UIA_RadioButtonControlTypeId, UIA_SplitButtonControlTypeId,
    UIA_StatusBarControlTypeId, UIA_TabItemControlTypeId,
    UIA_TitleBarControlTypeId, UIA_ToolBarControlTypeId,
    UIA_TreeItemControlTypeId, UIA_CONTROLTYPE_ID,
  },
};

use crate::color_theme::{ElementRect, UiElementKind};

/// Shortest time between two queries of the same window.
const MIN_QUERY_INTERVAL: Duration = Duration::from_millis(400);

/// A query waits at least this many times its own duration before the
/// next one of the same window, so a slow tree costs the app at most ~5%
/// of its UI thread.
const QUERY_BACKOFF: u32 = 20;

/// How long UIA waits for an unresponsive app before giving up on a
/// query.
const UIA_TIMEOUT_MS: u32 = 1000;

/// Accessibility queries of one window, stopped when dropped.
pub(crate) struct ElementWatch {
  watch: Arc<Watch>,
}

/// Tells the worker a watched window's content changed, so its elements
/// may have moved.
#[derive(Clone)]
pub(crate) struct ContentChanged {
  watch: Arc<Watch>,
}

struct Watch {
  /// Raw `HWND` of the watched window.
  source: isize,

  kinds: Mutex<Vec<UiElementKind>>,

  /// Set when the content changed since the last query.
  dirty: AtomicBool,
  closed: AtomicBool,

  on_update: Box<dyn Fn(Vec<ElementRect>) + Send + Sync>,
}

impl ElementWatch {
  /// Starts querying `source` for elements of `kinds`, reporting each
  /// result to `on_update` on the worker thread.
  pub(crate) fn start(
    source: HWND,
    kinds: Vec<UiElementKind>,
    on_update: impl Fn(Vec<ElementRect>) + Send + Sync + 'static,
  ) -> Self {
    let watch = Arc::new(Watch {
      source: source.0,
      kinds: Mutex::new(kinds),
      dirty: AtomicBool::new(true),
      closed: AtomicBool::new(false),
      on_update: Box::new(on_update),
    });

    worker().add(watch.clone());
    Self { watch }
  }

  /// Switches the element kinds queried, re-querying right away.
  pub(crate) fn set_kinds(&self, kinds: Vec<UiElementKind>) {
    let mut current = self
      .watch
      .kinds
      .lock()
      .unwrap_or_else(PoisonError::into_inner);

    if *current != kinds {
      *current = kinds;
      drop(current);
      self.content_changed().notify();
    }
  }

  pub(crate) fn content_changed(&self) -> ContentChanged {
    ContentChanged {
      watch: self.watch.clone(),
    }
  }
}

impl Drop for ElementWatch {
  fn drop(&mut self) {
    self.watch.closed.store(true, Ordering::Relaxed);
    worker().wake.notify_one();
  }
}

impl ContentChanged {
  pub(crate) fn notify(&self) {
    if !self.watch.dirty.swap(true, Ordering::Relaxed) {
      worker().wake.notify_one();
    }
  }
}

struct Worker {
  watches: Mutex<Vec<Arc<Watch>>>,
  wake: Condvar,

  /// Cleared if UI Automation can't be used, so watches aren't kept.
  available: AtomicBool,
}

fn worker() -> &'static Worker {
  static WORKER: OnceLock<Worker> = OnceLock::new();

  WORKER.get_or_init(|| {
    let spawned = std::thread::Builder::new()
      .name("color-theme-ui-elements".to_string())
      .spawn(|| worker().run());

    if let Err(err) = spawned {
      tracing::warn!("Failed to start UI element queries: {err}.");
    }

    Worker {
      watches: Mutex::new(Vec::new()),
      wake: Condvar::new(),
      available: AtomicBool::new(true),
    }
  })
}

impl Worker {
  fn add(&self, watch: Arc<Watch>) {
    if !self.available.load(Ordering::Relaxed) {
      return;
    }

    self
      .watches
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .push(watch);
    self.wake.notify_one();
  }

  fn run(&self) {
    let client = match Client::create() {
      Ok(client) => client,
      Err(err) => {
        tracing::warn!(
          "UI Automation unavailable for color themes: {err}."
        );
        self.available.store(false, Ordering::Relaxed);
        self
          .watches
          .lock()
          .unwrap_or_else(PoisonError::into_inner)
          .clear();
        return;
      }
    };

    // When each watch (by pointer) may next be queried.
    let mut next_query = HashMap::<usize, Instant>::new();

    loop {
      let Some(watch) = self.next_due(&mut next_query) else {
        continue;
      };

      watch.dirty.store(false, Ordering::Relaxed);
      let kinds = watch
        .kinds
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();

      let started = Instant::now();
      let elements = client.query(HWND(watch.source), &kinds);
      let took = started.elapsed();

      next_query.insert(
        Arc::as_ptr(&watch) as usize,
        Instant::now() + MIN_QUERY_INTERVAL.max(took * QUERY_BACKOFF),
      );

      match elements {
        Ok(elements) if !watch.closed.load(Ordering::Relaxed) => {
          (watch.on_update)(elements);
        }
        Ok(_) => {}
        // Mostly a window closing mid-query; it is retried on the next
        // content change otherwise.
        Err(err) => tracing::debug!("UI element query failed: {err}."),
      }
    }
  }

  /// Waits until a dirty watch may be queried, and returns it. Returns
  /// `None` on a wake-up without one (the caller loops).
  fn next_due(
    &self,
    next_query: &mut HashMap<usize, Instant>,
  ) -> Option<Arc<Watch>> {
    let mut watches =
      self.watches.lock().unwrap_or_else(PoisonError::into_inner);

    watches.retain(|watch| !watch.closed.load(Ordering::Relaxed));
    next_query.retain(|key, _| {
      watches
        .iter()
        .any(|watch| Arc::as_ptr(watch) as usize == *key)
    });

    let now = Instant::now();
    let mut earliest: Option<Instant> = None;

    for watch in watches.iter() {
      if !watch.dirty.load(Ordering::Relaxed) {
        continue;
      }

      let due = next_query
        .get(&(Arc::as_ptr(watch) as usize))
        .copied()
        .unwrap_or(now);

      if due <= now {
        return Some(watch.clone());
      }

      earliest = Some(earliest.map_or(due, |earliest| earliest.min(due)));
    }

    drop(match earliest {
      Some(due) => self
        .wake
        .wait_timeout(watches, due - now)
        .map_or_else(|err| err.into_inner().0, |(guard, _)| guard),
      None => self
        .wake
        .wait(watches)
        .unwrap_or_else(PoisonError::into_inner),
    });

    None
  }
}

/// The worker thread's UI Automation client.
struct Client {
  automation: IUIAutomation,
  cache_request: IUIAutomationCacheRequest,
}

impl Client {
  fn create() -> crate::Result<Self> {
    // SAFETY: Called once, on the worker thread this client lives on; MTA
    // is what UIA recommends for clients off a UI thread.
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED)? };

    // SAFETY: Plain COM object creation.
    let automation: IUIAutomation = unsafe {
      CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)?
    };

    // Bounds how long a hung app can stall the worker.
    if let Ok(automation) =
      windows::core::ComInterface::cast::<IUIAutomation2>(&automation)
    {
      // SAFETY: Plain property setters.
      unsafe {
        let _ = automation.SetConnectionTimeout(UIA_TIMEOUT_MS);
        let _ = automation.SetTransactionTimeout(UIA_TIMEOUT_MS);
      }
    }

    // Only the cached properties are needed, fetched in one round trip.
    // SAFETY: Plain COM calls on a live object.
    let cache_request = unsafe {
      let request = automation.CreateCacheRequest()?;
      request.AddProperty(UIA_ControlTypePropertyId)?;
      request.AddProperty(UIA_BoundingRectanglePropertyId)?;
      request.AddProperty(UIA_IsOffscreenPropertyId)?;
      request.SetAutomationElementMode(AutomationElementMode_None)?;
      request
    };

    Ok(Self {
      automation,
      cache_request,
    })
  }

  /// On-screen elements of `kinds` in `source`, relative to its frame
  /// (`DWMWA_EXTENDED_FRAME_BOUNDS`), which is what WGC captures.
  fn query(
    &self,
    source: HWND,
    kinds: &[UiElementKind],
  ) -> crate::Result<Vec<ElementRect>> {
    let Some(condition) = self.condition(kinds)? else {
      return Ok(Vec::new());
    };

    // SAFETY: Plain COM calls; a stale `source` just fails.
    let found = unsafe {
      self
        .automation
        .ElementFromHandle(source)?
        .FindAllBuildCache(
          TreeScope_Descendants,
          &condition,
          &self.cache_request,
        )?
    };

    let origin = frame_origin(source)?;
    // SAFETY: Plain COM call on a live array.
    let count = unsafe { found.Length()? };
    let mut elements = Vec::new();

    for index in 0..count {
      // SAFETY: `index` is within the array's length, and the cached
      // properties were all requested above.
      let (control_type, rect, offscreen) = unsafe {
        let element = found.GetElement(index)?;
        (
          element.CachedControlType()?,
          element.CachedBoundingRectangle()?,
          element.CachedIsOffscreen()?,
        )
      };

      let Some(kind) = kind_of(control_type) else {
        continue;
      };

      if offscreen.as_bool()
        || rect.right <= rect.left
        || rect.bottom <= rect.top
      {
        continue;
      }

      elements.push(ElementRect {
        kind,
        ltrb: [
          rect.left - origin.0,
          rect.top - origin.1,
          rect.right - origin.0,
          rect.bottom - origin.1,
        ],
      });
    }

    Ok(elements)
  }

  /// A condition matching any control type of `kinds`.
  fn condition(
    &self,
    kinds: &[UiElementKind],
  ) -> crate::Result<Option<IUIAutomationCondition>> {
    let mut condition: Option<IUIAutomationCondition> = None;

    for control_type in kinds.iter().flat_map(|kind| control_types(*kind))
    {
      let mut value = VARIANT::default();

      // SAFETY: Writes a `VT_I4` variant, whose value is `lVal`.
      let matches = unsafe {
        let inner = &mut *value.Anonymous.Anonymous;
        inner.vt = VT_I4;
        inner.Anonymous.lVal = i32::try_from(control_type.0)?;
        self
          .automation
          .CreatePropertyCondition(UIA_ControlTypePropertyId, value)?
      };

      condition = Some(match condition {
        // SAFETY: Plain COM call combining two live conditions.
        Some(existing) => unsafe {
          self.automation.CreateOrCondition(&existing, &matches)?
        },
        None => matches,
      });
    }

    Ok(condition)
  }
}

/// Top-left corner of `source`'s frame in screen coordinates.
fn frame_origin(source: HWND) -> crate::Result<(i32, i32)> {
  let mut rect = RECT::default();

  // SAFETY: `rect` is a `RECT`, exactly the size passed, and outlives the
  // call; a stale `source` just fails.
  unsafe {
    DwmGetWindowAttribute(
      source,
      DWMWA_EXTENDED_FRAME_BOUNDS,
      std::ptr::from_mut(&mut rect).cast(),
      u32::try_from(std::mem::size_of::<RECT>())?,
    )?;
  }

  Ok((rect.left, rect.top))
}

/// UIA control types reported as `kind`.
fn control_types(kind: UiElementKind) -> &'static [UIA_CONTROLTYPE_ID] {
  match kind {
    UiElementKind::Image => &[UIA_ImageControlTypeId],
    UiElementKind::Edit => &[UIA_EditControlTypeId],
    UiElementKind::Document => &[UIA_DocumentControlTypeId],
    UiElementKind::Button => {
      &[UIA_ButtonControlTypeId, UIA_SplitButtonControlTypeId]
    }
    UiElementKind::Hyperlink => &[UIA_HyperlinkControlTypeId],
    UiElementKind::CheckBox => &[UIA_CheckBoxControlTypeId],
    UiElementKind::RadioButton => &[UIA_RadioButtonControlTypeId],
    UiElementKind::ComboBox => &[UIA_ComboBoxControlTypeId],
    UiElementKind::ListItem => &[UIA_ListItemControlTypeId],
    UiElementKind::TreeItem => &[UIA_TreeItemControlTypeId],
    UiElementKind::TabItem => &[UIA_TabItemControlTypeId],
    UiElementKind::MenuItem => &[UIA_MenuItemControlTypeId],
    UiElementKind::DataItem => &[UIA_DataItemControlTypeId],
    UiElementKind::Header => {
      &[UIA_HeaderControlTypeId, UIA_HeaderItemControlTypeId]
    }
    UiElementKind::ToolBar => &[UIA_ToolBarControlTypeId],
    UiElementKind::StatusBar => &[UIA_StatusBarControlTypeId],
    UiElementKind::TitleBar => &[UIA_TitleBarControlTypeId],
  }
}

fn kind_of(control_type: UIA_CONTROLTYPE_ID) -> Option<UiElementKind> {
  UiElementKind::ALL
    .into_iter()
    .find(|kind| control_types(*kind).contains(&control_type))
}
