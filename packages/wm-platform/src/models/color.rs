use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Color {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,
}

impl Color {
  #[must_use]
  #[allow(clippy::missing_panics_doc)]
  pub fn to_bgr(&self) -> u32 {
    let bgr = format!("{:02x}{:02x}{:02x}", self.b, self.g, self.r);
    // SAFETY: An invalid hex value is unrepresentable.
    u32::from_str_radix(&bgr, 16).unwrap()
  }

  /// Packs this color into ABGR order (alpha in the high byte, then blue,
  /// green, red -- the order the raw Win32 `SetWindowCompositionAttribute`
  /// gradient-color API requires). Inverse of [`from_abgr`].
  ///
  /// [`from_abgr`]: Color::from_abgr
  #[must_use]
  pub fn to_abgr(&self) -> u32 {
    (u32::from(self.a) << 24)
      | (u32::from(self.b) << 16)
      | (u32::from(self.g) << 8)
      | u32::from(self.r)
  }

  /// Unpacks an ABGR-packed `u32` (see [`to_abgr`]) into a `Color`.
  ///
  /// [`to_abgr`]: Color::to_abgr
  #[must_use]
  pub fn from_abgr(abgr: u32) -> Self {
    #[allow(clippy::cast_possible_truncation)]
    Color {
      a: (abgr >> 24) as u8,
      b: (abgr >> 16) as u8,
      g: (abgr >> 8) as u8,
      r: abgr as u8,
    }
  }

  /// Linearly interpolates each color channel toward `other` by `t`.
  ///
  /// `t` is clamped to `0.0..=1.0`; `0.0` returns `self`, `1.0` returns
  /// `other`.
  #[must_use]
  pub fn lerp(&self, other: &Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lerp_channel = |from: u8, to: u8| -> u8 {
      (f32::from(from) + (f32::from(to) - f32::from(from)) * t).round() as u8
    };

    Color {
      r: lerp_channel(self.r, other.r),
      g: lerp_channel(self.g, other.g),
      b: lerp_channel(self.b, other.b),
      a: lerp_channel(self.a, other.a),
    }
  }
}

impl FromStr for Color {
  type Err = crate::ParseError;

  fn from_str(unparsed: &str) -> Result<Self, crate::ParseError> {
    let mut chars = unparsed.chars();

    if chars.next() != Some('#') {
      return Err(crate::ParseError::Color(unparsed.to_string()));
    }

    let parse_hex = |slice: &str| -> Result<u8, crate::ParseError> {
      u8::from_str_radix(slice, 16)
        .map_err(|_| crate::ParseError::Color(unparsed.to_string()))
    };

    let r = parse_hex(&unparsed[1..3])?;
    let g = parse_hex(&unparsed[3..5])?;
    let b = parse_hex(&unparsed[5..7])?;

    let a = match unparsed.len() {
      9 => parse_hex(&unparsed[7..9])?,
      7 => 255,
      _ => return Err(crate::ParseError::Color(unparsed.to_string())),
    };

    Ok(Self { r, g, b, a })
  }
}

/// Deserialize a `Color` from either a string or a struct.
impl<'de> Deserialize<'de> for Color {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ColorDe {
      Struct { r: u8, g: u8, b: u8, a: u8 },
      String(String),
    }

    match ColorDe::deserialize(deserializer)? {
      ColorDe::Struct { r, g, b, a } => Ok(Self { r, g, b, a }),
      ColorDe::String(str) => {
        Self::from_str(&str).map_err(serde::de::Error::custom)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Color;

  #[test]
  fn lerp_at_endpoints_returns_start_and_end() {
    let start = Color { r: 0, g: 10, b: 20, a: 255 };
    let end = Color { r: 100, g: 110, b: 120, a: 0 };

    assert_eq!(start.lerp(&end, 0.0), start);
    assert_eq!(start.lerp(&end, 1.0), end);
  }

  #[test]
  fn lerp_at_midpoint_averages_channels() {
    let start = Color { r: 0, g: 0, b: 0, a: 0 };
    let end = Color { r: 100, g: 200, b: 50, a: 255 };

    let mid = start.lerp(&end, 0.5);
    assert_eq!(mid, Color { r: 50, g: 100, b: 25, a: 128 });
  }

  #[test]
  fn lerp_clamps_out_of_range_t() {
    let start = Color { r: 0, g: 0, b: 0, a: 0 };
    let end = Color { r: 100, g: 100, b: 100, a: 100 };

    assert_eq!(start.lerp(&end, -0.5), start);
    assert_eq!(start.lerp(&end, 1.5), end);
  }
}
