use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LengthValue {
  pub amount: f32,
  pub unit: LengthUnit,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LengthUnit {
  Percentage,
  Pixel,
}

impl LengthValue {
  #[must_use]
  pub fn from_px(px: i32) -> Self {
    Self {
      #[allow(clippy::cast_precision_loss)]
      amount: px as f32,
      unit: LengthUnit::Pixel,
    }
  }

  #[must_use]
  pub fn to_px(&self, total_px: i32, scale_factor: Option<f32>) -> i32 {
    let scale_factor = scale_factor.unwrap_or(1.0);

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    match self.unit {
      LengthUnit::Percentage => (self.amount * total_px as f32) as i32,
      LengthUnit::Pixel => (self.amount * scale_factor) as i32,
    }
  }

  #[must_use]
  pub fn to_percentage(&self, total_px: i32) -> f32 {
    match self.unit {
      LengthUnit::Percentage => self.amount,
      #[allow(clippy::cast_precision_loss)]
      LengthUnit::Pixel => self.amount / total_px as f32,
    }
  }
}

impl FromStr for LengthValue {
  type Err = crate::ParseError;

  /// Parses a string containing a number followed by a unit (`px`, `%`).
  /// Allows for negative numbers.
  ///
  /// Example:
  /// ```
  /// # use wm_platform::{LengthValue, LengthUnit};
  /// # use std::str::FromStr;
  /// let check = LengthValue {
  ///   amount: 100.0,
  ///   unit: LengthUnit::Pixel,
  /// };
  /// let parsed = LengthValue::from_str("100px");
  /// assert_eq!(parsed.unwrap(), check);
  /// ```
  fn from_str(unparsed: &str) -> Result<Self, crate::ParseError> {
    let error = || crate::ParseError::Length(unparsed.to_string());
    let value = unparsed.trim();

    let (amount, unit) = match value.strip_suffix('%') {
      Some(amount) => (amount, LengthUnit::Percentage),
      None => {
        (value.strip_suffix("px").unwrap_or(value), LengthUnit::Pixel)
      }
    };

    let amount = amount.trim().parse::<f32>().map_err(|_| error())?;

    // `f32::from_str` also accepts `inf`/`NaN`, which are meaningless as a
    // length and would propagate silently through `to_px`.
    if !amount.is_finite() {
      return Err(error());
    }

    Ok(LengthValue {
      // Store percentage units as a fraction of 1.
      amount: if unit == LengthUnit::Percentage {
        amount / 100.0
      } else {
        amount
      },
      unit,
    })
  }
}

/// Deserialize a `LengthValue` from either a string or a struct.
impl<'de> Deserialize<'de> for LengthValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum LengthValueDe {
      Struct { amount: f32, unit: LengthUnit },
      String(String),
    }

    match LengthValueDe::deserialize(deserializer)? {
      LengthValueDe::Struct { amount, unit } => Ok(Self { amount, unit }),
      LengthValueDe::String(str) => {
        Self::from_str(&str).map_err(serde::de::Error::custom)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::{LengthUnit, LengthValue};

  #[test]
  fn parses_pixels_with_and_without_the_unit() {
    for input in ["100px", "100"] {
      let parsed = LengthValue::from_str(input).unwrap();
      assert_eq!(parsed.unit, LengthUnit::Pixel);
      assert!((parsed.amount - 100.0).abs() < f32::EPSILON);
    }
  }

  /// Percentages are stored as a fraction of 1, not as the written number.
  #[test]
  fn parses_percentages_as_a_fraction() {
    let parsed = LengthValue::from_str("95%").unwrap();
    assert_eq!(parsed.unit, LengthUnit::Percentage);
    assert!((parsed.amount - 0.95).abs() < f32::EPSILON);
  }

  #[test]
  fn parses_signed_values() {
    let negative = LengthValue::from_str("-2%").unwrap();
    assert!((negative.amount - -0.02).abs() < f32::EPSILON);

    let positive = LengthValue::from_str("+5px").unwrap();
    assert!((positive.amount - 5.0).abs() < f32::EPSILON);
  }

  /// Surrounding and pre-unit whitespace was tolerated by the previous
  /// regex-based parser, so config values keep parsing either way.
  #[test]
  fn tolerates_surrounding_whitespace() {
    for input in ["  100px  ", "100 px"] {
      let parsed = LengthValue::from_str(input).unwrap();
      assert_eq!(parsed.unit, LengthUnit::Pixel);
      assert!((parsed.amount - 100.0).abs() < f32::EPSILON);
    }
  }

  /// `f32::from_str` accepts these; a length must not.
  #[test]
  fn rejects_non_finite_and_unparseable_values() {
    for input in ["inf", "-inf", "NaN", "", "px", "%", "abc", "100pt"] {
      assert!(
        LengthValue::from_str(input).is_err(),
        "expected {input:?} to be rejected"
      );
    }
  }
}
