use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
};

use tokio::sync::mpsc;

use crate::{
  platform_event::KeybindingEvent, platform_impl, Dispatcher, Key,
};

/// Modifier key groups, where each entry maps a generic key (e.g.
/// `Key::Shift`) to all its variants (e.g. `Key::LShift`, `Key::RShift`).
///
/// `Cmd` and `Win` are treated as aliases within the same group.
const MODIFIER_GROUPS: &[(Key, &[Key])] = &[
  (Key::Shift, &[Key::Shift, Key::LShift, Key::RShift]),
  (Key::Ctrl, &[Key::Ctrl, Key::LCtrl, Key::RCtrl]),
  (Key::Alt, &[Key::Alt, Key::LAlt, Key::RAlt]),
  (
    Key::Win,
    &[
      Key::Win,
      Key::LWin,
      Key::RWin,
      Key::Cmd,
      Key::LCmd,
      Key::RCmd,
    ],
  ),
];

/// Turn of a mouse wheel that makes one notch.
const WHEEL_NOTCH: i32 = 120;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Keybinding(Vec<Key>);

impl Keybinding {
  /// Creates a new keybinding from a vector of keys.
  ///
  /// # Errors
  ///
  /// Returns [`Error::InvalidKeybinding`] if the keybinding is empty, or
  /// [`ParseError::Keybinding`] if a mouse wheel key isn't its last key,
  /// as the wheel can't be held.
  pub fn new(keys: Vec<Key>) -> crate::Result<Self> {
    let Some((_, held_keys)) = keys.split_last() else {
      return Err(crate::Error::InvalidKeybinding);
    };

    if held_keys.iter().any(|key| key.is_wheel()) {
      return Err(crate::Error::Parse(crate::ParseError::Keybinding(
        "A mouse wheel key can only be the last key.".to_string(),
      )));
    }

    Ok(Self(keys))
  }

  /// Returns the keys in the keybinding.
  #[must_use]
  pub fn keys(&self) -> &[Key] {
    &self.0
  }

  /// Returns the trigger key in the keybinding.
  #[must_use]
  #[allow(clippy::missing_panics_doc)]
  pub fn trigger_key(&self) -> &Key {
    // SAFETY: Keys vector is verified to be non-empty in
    // `Keybinding::new`.
    self.0.last().unwrap()
  }
}

/// Keybindings by their trigger key.
///
/// The trigger key is the final key in a keybinding. For example, in the
/// keybinding `[Key::Cmd, Key::Shift, Key::A]`, `Key::A` is the trigger
/// key.
type KeybindingMap = Arc<Mutex<HashMap<Key, Vec<Keybinding>>>>;

/// A listener for system-wide keybindings.
///
/// # Platform-specific
///
/// - **macOS**: Keybindings triggered by the mouse wheel never trigger.
#[derive(Debug)]
pub struct KeybindingListener {
  /// A receiver channel for outgoing keybinding events.
  event_rx: mpsc::UnboundedReceiver<KeybindingEvent>,

  keybinding_map: KeybindingMap,

  /// Whether the listener is currently enabled.
  enabled: Arc<AtomicBool>,

  /// The underlying keyboard hook used to listen for key events.
  keyboard_hook: platform_impl::KeyboardHook,

  /// Sender of keybinding events, for a wheel hook created later.
  #[cfg(target_os = "windows")]
  event_tx: mpsc::UnboundedSender<KeybindingEvent>,

  /// Hook for keybindings triggered by the mouse wheel, only while there
  /// are any, so the mouse isn't hooked for nothing.
  #[cfg(target_os = "windows")]
  wheel_hook: Option<platform_impl::WheelHook>,
}

impl KeybindingListener {
  /// Creates an instance of `KeybindingListener`.
  pub fn new(
    keybindings: &[Keybinding],
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let keybinding_map =
      Arc::new(Mutex::new(Self::create_keybinding_map(keybindings)));

    let enabled = Arc::new(AtomicBool::new(true));

    let keyboard_hook = Self::create_keyboard_hook(
      keybinding_map.clone(),
      enabled.clone(),
      event_tx.clone(),
      dispatcher,
    )?;

    #[allow(unused_mut)]
    let mut listener = Self {
      event_rx,
      keybinding_map,
      enabled,
      keyboard_hook,
      #[cfg(target_os = "windows")]
      event_tx,
      #[cfg(target_os = "windows")]
      wheel_hook: None,
    };

    #[cfg(target_os = "windows")]
    listener.sync_wheel_hook(keybindings)?;

    Ok(listener)
  }

  /// Returns the next keybinding event from the listener.
  ///
  /// This will block until a keybinding event is available.
  pub async fn next_event(&mut self) -> Option<KeybindingEvent> {
    let event = self.event_rx.recv().await;

    #[cfg(target_os = "windows")]
    if event.is_some() {
      crate::perf::record_event_dequeued(
        crate::perf::EventKind::Keybinding,
      );
    }

    event
  }

  /// Returns the next keybinding event if one is already queued.
  ///
  /// Unlike [`next_event`], never waits. Used by the main loop to service
  /// keybindings that piled up while the previous animation frame was
  /// running, before it starts the next one.
  ///
  /// [`next_event`]: KeybindingListener::next_event
  pub fn try_next_event(&mut self) -> Option<KeybindingEvent> {
    let event = self.event_rx.try_recv().ok();

    #[cfg(target_os = "windows")]
    if event.is_some() {
      crate::perf::record_event_dequeued(
        crate::perf::EventKind::Keybinding,
      );
    }

    event
  }

  /// Updates the keybindings for the keybinding listener.
  ///
  /// # Panics
  ///
  /// If the internal mutex is poisoned.
  pub fn update(&mut self, keybindings: &[Keybinding]) {
    *self.keybinding_map.lock().unwrap() =
      Self::create_keybinding_map(keybindings);

    #[cfg(target_os = "windows")]
    if let Err(err) = self.sync_wheel_hook(keybindings) {
      tracing::warn!("Failed to hook the mouse wheel: {err}");
    }
  }

  /// Enables or disables the keybinding listener.
  pub fn enable(&mut self, enabled: bool) {
    self.enabled.store(enabled, Ordering::Relaxed);
  }

  /// Terminates the keybinding listener.
  pub fn terminate(&mut self) -> crate::Result<()> {
    #[cfg(target_os = "windows")]
    if let Some(mut wheel_hook) = self.wheel_hook.take() {
      wheel_hook.terminate()?;
    }

    self.keyboard_hook.terminate()
  }

  /// Creates and starts the keyboard hook with the given callback.
  fn create_keyboard_hook(
    keybinding_map: KeybindingMap,
    enabled: Arc<AtomicBool>,
    event_tx: mpsc::UnboundedSender<KeybindingEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<platform_impl::KeyboardHook> {
    platform_impl::KeyboardHook::new(
      move |event: platform_impl::KeyEvent| -> bool {
        if !enabled.load(Ordering::Relaxed) || !event.is_keypress {
          return false;
        }

        let Ok(keybinding_map) = keybinding_map.lock() else {
          tracing::error!("Failed to acquire lock on keybinding map.");
          return false;
        };

        // Find keybinding candidates whose trigger key is the pressed key.
        let Some(candidates) = keybinding_map.get(&event.key) else {
          return false;
        };

        let Some(keybinding) =
          matching_keybinding(candidates, event.key, |key| {
            event.is_key_down(key)
          })
        else {
          return false;
        };

        #[cfg(target_os = "windows")]
        crate::perf::mark_event_queued(crate::perf::EventKind::Keybinding);

        let _ = event_tx.send(KeybindingEvent::now(keybinding.clone()));

        true
      },
      dispatcher,
    )
  }

  /// Hooks the mouse wheel while some keybinding is triggered by it, and
  /// unhooks it otherwise.
  #[cfg(target_os = "windows")]
  fn sync_wheel_hook(
    &mut self,
    keybindings: &[Keybinding],
  ) -> crate::Result<()> {
    let has_wheel_keybindings = keybindings
      .iter()
      .any(|keybinding| keybinding.trigger_key().is_wheel());

    match (&self.wheel_hook, has_wheel_keybindings) {
      (None, true) => {
        self.wheel_hook = Some(Self::create_wheel_hook(
          self.keybinding_map.clone(),
          self.enabled.clone(),
          self.event_tx.clone(),
        )?);
      }
      (Some(_), false) => {
        if let Some(mut wheel_hook) = self.wheel_hook.take() {
          wheel_hook.terminate()?;
        }
      }
      _ => {}
    }

    Ok(())
  }

  /// Creates and starts the wheel hook, which triggers a keybinding once
  /// per notch the wheel turns while its other keys are held.
  ///
  /// Turns towards a keybinding never reach the window under the cursor,
  /// not even those short of a notch.
  #[cfg(target_os = "windows")]
  fn create_wheel_hook(
    keybinding_map: KeybindingMap,
    enabled: Arc<AtomicBool>,
    event_tx: mpsc::UnboundedSender<KeybindingEvent>,
  ) -> crate::Result<platform_impl::WheelHook> {
    // Turns of the vertical and the horizontal wheel short of a notch.
    let mut trackers = [WheelTracker::default(); 2];

    platform_impl::WheelHook::new(
      move |event: platform_impl::WheelEvent| -> bool {
        if !enabled.load(Ordering::Relaxed) || event.delta == 0 {
          return false;
        }

        let Ok(keybinding_map) = keybinding_map.lock() else {
          tracing::error!("Failed to acquire lock on keybinding map.");
          return false;
        };

        let trigger = match (event.is_horizontal, event.delta > 0) {
          (false, true) => Key::WheelUp,
          (false, false) => Key::WheelDown,
          (true, true) => Key::WheelRight,
          (true, false) => Key::WheelLeft,
        };

        let tracker = &mut trackers[usize::from(event.is_horizontal)];

        let Some(keybinding) =
          keybinding_map.get(&trigger).and_then(|candidates| {
            matching_keybinding(
              candidates,
              trigger,
              platform_impl::WheelEvent::is_key_down,
            )
          })
        else {
          tracker.reset();
          return false;
        };

        for _ in 0..tracker.accumulate(event.delta).unsigned_abs() {
          crate::perf::mark_event_queued(
            crate::perf::EventKind::Keybinding,
          );
          let _ = event_tx.send(KeybindingEvent::now(keybinding.clone()));
        }

        true
      },
    )
  }

  /// Builds the keybinding map from configs.
  fn create_keybinding_map(
    keybindings: &[Keybinding],
  ) -> HashMap<Key, Vec<Keybinding>> {
    let mut keybinding_map = HashMap::new();

    for keybinding in keybindings {
      keybinding_map
        .entry(*keybinding.trigger_key())
        .or_insert_with(Vec::new)
        .push(keybinding.clone());
    }

    keybinding_map
  }
}

impl Drop for KeybindingListener {
  fn drop(&mut self) {
    let _ = self.terminate();
  }
}

/// The longest of `candidates` whose keys other than `trigger` are all
/// held down, per `is_down`, unless a modifier outside it is held too.
fn matching_keybinding(
  candidates: &[Keybinding],
  trigger: Key,
  is_down: impl Fn(Key) -> bool,
) -> Option<&Keybinding> {
  let mut cached_key_states = HashMap::new();

  // Find the longest keybinding whose keys are all pressed.
  let longest_keybinding = candidates
    .iter()
    .filter(|keybinding| {
      keybinding.keys().iter().all(|&key| {
        key == trigger
          || *cached_key_states.entry(key).or_insert_with(|| is_down(key))
      })
    })
    .max_by_key(|keybinding| keybinding.keys().len())?;

  // Reject if any modifier keys not in the keybinding are held.
  let has_extra_modifiers = MODIFIER_GROUPS
    .iter()
    // Filter out modifier groups that have keys in the keybinding.
    .filter(|(_, group_keys)| {
      !group_keys
        .iter()
        .any(|key| longest_keybinding.keys().contains(key))
    })
    // Use the group's "generic" key (e.g. `Key::Shift`) to check if the
    // modifier is held. This avoids lookups for `Key::LShift` and
    // `Key::RShift`.
    .any(|(generic_key, _)| {
      cached_key_states
        .get(generic_key)
        .copied()
        .unwrap_or_else(|| is_down(*generic_key))
    });

  (!has_extra_modifiers).then_some(longest_keybinding)
}

/// Turns of a mouse wheel short of a notch, as fine-grained wheels and
/// touchpads report them.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
struct WheelTracker {
  total: i32,
  last: i16,
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
impl WheelTracker {
  /// Adds a turn of `delta`, in 120ths of a notch, and takes the whole
  /// notches it now adds up to: positive up (or right), negative down.
  ///
  /// Turning back the other way starts over.
  fn accumulate(&mut self, delta: i16) -> i32 {
    if (self.last > 0 && delta < 0) || (self.last < 0 && delta > 0) {
      self.total = 0;
    }

    self.last = delta;
    self.total += i32::from(delta);

    let notches = self.total / WHEEL_NOTCH;
    self.total %= WHEEL_NOTCH;
    notches
  }

  fn reset(&mut self) {
    *self = Self::default();
  }
}

#[cfg(test)]
mod tests {
  use super::{matching_keybinding, Keybinding, WheelTracker};
  use crate::Key;

  fn keybinding(keys: &[Key]) -> Keybinding {
    Keybinding::new(keys.to_vec()).unwrap()
  }

  #[test]
  fn the_mouse_wheel_is_only_a_last_key() {
    assert!(Keybinding::new(vec![Key::Alt, Key::WheelUp]).is_ok());
    assert!(Keybinding::new(vec![Key::WheelUp, Key::A]).is_err());
    assert!(Keybinding::new(Vec::new()).is_err());
  }

  #[test]
  fn wheel_notches_add_up_and_reset_on_reversal() {
    let mut tracker = WheelTracker::default();

    assert_eq!(tracker.accumulate(120), 1);
    assert_eq!(tracker.accumulate(-240), -2);

    // Fine-grained turns add up to a notch.
    assert_eq!(tracker.accumulate(40), 0);
    assert_eq!(tracker.accumulate(40), 0);
    assert_eq!(tracker.accumulate(40), 1);

    // Turning back drops what was left over.
    assert_eq!(tracker.accumulate(100), 0);
    assert_eq!(tracker.accumulate(-60), 0);
    assert_eq!(tracker.accumulate(-60), -1);
  }

  #[test]
  fn matches_the_longest_keybinding_without_extra_modifiers() {
    let candidates = [
      keybinding(&[Key::Alt, Key::WheelDown]),
      keybinding(&[Key::Alt, Key::Shift, Key::WheelDown]),
    ];

    let with_shift = |key| matches!(key, Key::Alt | Key::Shift);
    assert_eq!(
      matching_keybinding(&candidates, Key::WheelDown, with_shift),
      Some(&candidates[1])
    );

    let alt_only = |key| key == Key::Alt;
    assert_eq!(
      matching_keybinding(&candidates, Key::WheelDown, alt_only),
      Some(&candidates[0])
    );

    let with_ctrl = |key| matches!(key, Key::Alt | Key::Ctrl);
    assert_eq!(
      matching_keybinding(&candidates, Key::WheelDown, with_ctrl),
      None
    );
  }
}
