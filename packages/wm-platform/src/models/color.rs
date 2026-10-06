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
  /// Packs this color into the `0x00BBGGRR` order of a Win32 `COLORREF`,
  /// dropping its alpha.
  #[must_use]
  pub fn to_bgr(&self) -> u32 {
    u32::from(self.r)
      | (u32::from(self.g) << 8)
      | (u32::from(self.b) << 16)
  }

  /// Packs this color into ABGR order (alpha in the high byte, then blue,
  /// green, red), as the backdrop tint is passed around. Inverse of
  /// [`from_abgr`].
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

  /// Interpolates `t` of the way to `to`, in premultiplied alpha so a
  /// fully transparent endpoint contributes no hue of its own.
  #[must_use]
  pub fn lerp(&self, to: &Color, t: f32) -> Color {
    if t <= 0.0 {
      return *self;
    }
    if t >= 1.0 {
      return *to;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let to_u8 = |value: f32| value.round().clamp(0.0, 255.0) as u8;

    let alpha = crate::lerp_f32(f32::from(self.a), f32::from(to.a), t);
    let channel = |from_channel: u8, to_channel: u8| {
      if alpha <= 0.0 {
        return 0;
      }

      let premultiplied = crate::lerp_f32(
        f32::from(from_channel) * f32::from(self.a),
        f32::from(to_channel) * f32::from(to.a),
        t,
      );

      to_u8(premultiplied / alpha)
    };

    Color {
      r: channel(self.r, to.r),
      g: channel(self.g, to.g),
      b: channel(self.b, to.b),
      a: to_u8(alpha),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn to_bgr_packs_like_a_colorref() {
    let color = Color {
      r: 0x12,
      g: 0x34,
      b: 0x56,
      a: 0x78,
    };
    assert_eq!(color.to_bgr(), 0x0056_3412);
  }

  #[test]
  fn lerp_blends_opaque_colors_per_channel() {
    let from = Color {
      r: 0,
      g: 100,
      b: 200,
      a: 255,
    };
    let to = Color {
      r: 200,
      g: 100,
      b: 0,
      a: 255,
    };

    assert_eq!(from.lerp(&to, 0.0), from);
    assert_eq!(from.lerp(&to, 1.0), to);
    assert_eq!(
      from.lerp(&to, 0.5),
      Color {
        r: 100,
        g: 100,
        b: 100,
        a: 255
      }
    );
  }

  #[test]
  fn lerp_ignores_hue_of_transparent_endpoint() {
    let clear_black = Color {
      r: 0,
      g: 0,
      b: 0,
      a: 0,
    };
    let red = Color {
      r: 255,
      g: 0,
      b: 0,
      a: 200,
    };

    // Straight-alpha lerp would darken toward black on the way in.
    assert_eq!(
      clear_black.lerp(&red, 0.5),
      Color {
        r: 255,
        g: 0,
        b: 0,
        a: 100
      }
    );
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
