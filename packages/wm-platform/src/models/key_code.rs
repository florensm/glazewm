use windows::Win32::UI::Input::KeyboardAndMouse::{
  VIRTUAL_KEY, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9,
  VK_A, VK_ADD, VK_B, VK_BACK, VK_C, VK_CAPITAL, VK_CONVERT, VK_D,
  VK_DECIMAL, VK_DELETE, VK_DIVIDE, VK_DOWN, VK_E, VK_END, VK_ESCAPE,
  VK_F, VK_F1, VK_F10, VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16,
  VK_F17, VK_F18, VK_F19, VK_F2, VK_F20, VK_F21, VK_F22, VK_F23, VK_F24,
  VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_G, VK_H, VK_HOME,
  VK_I, VK_INSERT, VK_J, VK_K, VK_L, VK_LCONTROL, VK_LEFT, VK_LMENU,
  VK_LSHIFT, VK_LWIN, VK_M, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE,
  VK_MEDIA_PREV_TRACK, VK_MEDIA_STOP, VK_MULTIPLY, VK_N, VK_NEXT,
  VK_NONCONVERT, VK_NUMLOCK, VK_NUMPAD0, VK_NUMPAD1, VK_NUMPAD2,
  VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8,
  VK_NUMPAD9, VK_O, VK_OEM_1, VK_OEM_102, VK_OEM_2, VK_OEM_3, VK_OEM_4,
  VK_OEM_5, VK_OEM_6, VK_OEM_7, VK_OEM_8, VK_OEM_COMMA, VK_OEM_MINUS,
  VK_OEM_PERIOD, VK_OEM_PLUS, VK_P, VK_PRIOR, VK_Q, VK_R, VK_RCONTROL,
  VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_S, VK_SCROLL,
  VK_SNAPSHOT, VK_SPACE, VK_SUBTRACT, VK_T, VK_TAB, VK_U, VK_UP, VK_V,
  VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP, VK_W, VK_X, VK_Y, VK_Z,
};

use crate::{Key, KeyCode};

#[derive(Debug, thiserror::Error)]
pub enum KeyConversionError {
  #[error("Unknown key code: {0}")]
  UnknownKeyCode(KeyCode),
}

/// Generates `TryFrom` implementations for converting between `Key` and
/// `KeyCode`, given each key's `VK_*` constant. Keys without one (e.g. the
/// mouse wheel) take `{}` and fail to convert.
macro_rules! impl_key_code_conversion {
  ($($variant:ident => { $($vk:expr)? }),* $(,)?) => {
    impl TryFrom<KeyCode> for Key {
      type Error = KeyConversionError;

      fn try_from(key_code: KeyCode) -> Result<Self, Self::Error> {
        let vk = VIRTUAL_KEY(key_code.0);
        $($(if vk == $vk { return Ok(Key::$variant); })?)*
        Err(KeyConversionError::UnknownKeyCode(key_code))
      }
    }

    impl TryFrom<Key> for KeyCode {
      type Error = KeyConversionError;

      fn try_from(key: Key) -> Result<Self, Self::Error> {
        match key {
          $(Key::$variant => {
            $(return Ok(KeyCode($vk.0));)?
            #[allow(unreachable_code)]
            return Err(KeyConversionError::UnknownKeyCode(KeyCode(0)));
          }),*
        }
      }
    }
  };
}

impl_key_code_conversion! {
  // Letter keys
  A => { VK_A },
  B => { VK_B },
  C => { VK_C },
  D => { VK_D },
  E => { VK_E },
  F => { VK_F },
  G => { VK_G },
  H => { VK_H },
  I => { VK_I },
  J => { VK_J },
  K => { VK_K },
  L => { VK_L },
  M => { VK_M },
  N => { VK_N },
  O => { VK_O },
  P => { VK_P },
  Q => { VK_Q },
  R => { VK_R },
  S => { VK_S },
  T => { VK_T },
  U => { VK_U },
  V => { VK_V },
  W => { VK_W },
  X => { VK_X },
  Y => { VK_Y },
  Z => { VK_Z },
  // Number keys
  D0 => { VK_0 },
  D1 => { VK_1 },
  D2 => { VK_2 },
  D3 => { VK_3 },
  D4 => { VK_4 },
  D5 => { VK_5 },
  D6 => { VK_6 },
  D7 => { VK_7 },
  D8 => { VK_8 },
  D9 => { VK_9 },
  // Function keys
  F1 => { VK_F1 },
  F2 => { VK_F2 },
  F3 => { VK_F3 },
  F4 => { VK_F4 },
  F5 => { VK_F5 },
  F6 => { VK_F6 },
  F7 => { VK_F7 },
  F8 => { VK_F8 },
  F9 => { VK_F9 },
  F10 => { VK_F10 },
  F11 => { VK_F11 },
  F12 => { VK_F12 },
  F13 => { VK_F13 },
  F14 => { VK_F14 },
  F15 => { VK_F15 },
  F16 => { VK_F16 },
  F17 => { VK_F17 },
  F18 => { VK_F18 },
  F19 => { VK_F19 },
  F20 => { VK_F20 },
  F21 => { VK_F21 },
  F22 => { VK_F22 },
  F23 => { VK_F23 },
  F24 => { VK_F24 },
  // Modifier keys
  LShift => { VK_LSHIFT },
  RShift => { VK_RSHIFT },
  LCtrl => { VK_LCONTROL },
  RCtrl => { VK_RCONTROL },
  LAlt => { VK_LMENU },
  RAlt => { VK_RMENU },
  Shift => { VK_LSHIFT },
  Ctrl => { VK_LCONTROL },
  Alt => { VK_LMENU },
  Win => { VK_LWIN },
  LWin => { VK_LWIN },
  RWin => { VK_RWIN },
  // Special keys
  Space => { VK_SPACE },
  Tab => { VK_TAB },
  Enter => { VK_RETURN },
  Delete => { VK_DELETE },
  Escape => { VK_ESCAPE },
  Backspace => { VK_BACK },
  // Arrow keys
  Left => { VK_LEFT },
  Right => { VK_RIGHT },
  Up => { VK_UP },
  Down => { VK_DOWN },
  // Navigation keys
  Home => { VK_HOME },
  End => { VK_END },
  PageUp => { VK_PRIOR },
  PageDown => { VK_NEXT },
  Insert => { VK_INSERT },
  // OEM keys
  OemSemicolon => { VK_OEM_1 },
  OemQuestion => { VK_OEM_2 },
  OemTilde => { VK_OEM_3 },
  OemOpenBrackets => { VK_OEM_4 },
  OemPipe => { VK_OEM_5 },
  OemCloseBrackets => { VK_OEM_6 },
  OemQuotes => { VK_OEM_7 },
  Oem8 => { VK_OEM_8 },
  Oem102 => { VK_OEM_102 },
  OemPlus => { VK_OEM_PLUS },
  OemComma => { VK_OEM_COMMA },
  OemMinus => { VK_OEM_MINUS },
  OemPeriod => { VK_OEM_PERIOD },
  // Numpad
  Numpad0 => { VK_NUMPAD0 },
  Numpad1 => { VK_NUMPAD1 },
  Numpad2 => { VK_NUMPAD2 },
  Numpad3 => { VK_NUMPAD3 },
  Numpad4 => { VK_NUMPAD4 },
  Numpad5 => { VK_NUMPAD5 },
  Numpad6 => { VK_NUMPAD6 },
  Numpad7 => { VK_NUMPAD7 },
  Numpad8 => { VK_NUMPAD8 },
  Numpad9 => { VK_NUMPAD9 },
  NumpadAdd => { VK_ADD },
  NumpadSubtract => { VK_SUBTRACT },
  NumpadMultiply => { VK_MULTIPLY },
  NumpadDivide => { VK_DIVIDE },
  NumpadDecimal => { VK_DECIMAL },
  // Lock keys
  NumLock => { VK_NUMLOCK },
  ScrollLock => { VK_SCROLL },
  CapsLock => { VK_CAPITAL },
  // Media keys
  VolumeUp => { VK_VOLUME_UP },
  VolumeDown => { VK_VOLUME_DOWN },
  VolumeMute => { VK_VOLUME_MUTE },
  MediaNextTrack => { VK_MEDIA_NEXT_TRACK },
  MediaPrevTrack => { VK_MEDIA_PREV_TRACK },
  MediaStop => { VK_MEDIA_STOP },
  MediaPlayPause => { VK_MEDIA_PLAY_PAUSE },
  PrintScreen => { VK_SNAPSHOT },
  // Language-specific keys
  Muhenkan => { VK_NONCONVERT },
  Henkan => { VK_CONVERT },
  // The mouse wheel has no key codes.
  WheelUp => {},
  WheelDown => {},
  WheelLeft => {},
  WheelRight => {},
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_key_conversion_roundtrip() {
    let test_keys = [
      Key::A,
      Key::S,
      Key::D,
      Key::F,
      Key::LAlt,
      Key::RCtrl,
      Key::LShift,
      Key::Space,
      Key::Tab,
      Key::Enter,
      Key::F1,
      Key::F12,
      Key::Left,
      Key::Right,
    ];

    for key in test_keys {
      let code: KeyCode = key.try_into().unwrap();
      let key2: Key = code.try_into().unwrap();
      assert_eq!(key, key2, "Roundtrip failed for key: {key:?}");
    }
  }

  #[test]
  fn test_key_without_code() {
    assert!(KeyCode::try_from(Key::Win).is_ok());
    assert!(KeyCode::try_from(Key::WheelUp).is_err());
  }
}
