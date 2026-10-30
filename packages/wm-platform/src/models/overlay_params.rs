use crate::{BackdropOverlayParams, BorderOverlayParams};

/// The params of a window's backdrop and border overlays, resolved
/// together so the two travel as one value. Each is `None` when its
/// effect is off for the window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverlayParams {
  pub backdrop: Option<BackdropOverlayParams>,
  pub border: Option<BorderOverlayParams>,
}

impl OverlayParams {
  /// Both overlays' params with their opacity multiplied by `factor`.
  #[must_use]
  pub fn with_opacity_scaled(self, factor: f32) -> Self {
    Self {
      backdrop: self.backdrop.map(|params| BackdropOverlayParams {
        opacity: params.opacity * factor,
        ..params
      }),
      border: self.border.map(|params| BorderOverlayParams {
        opacity: params.opacity * factor,
        ..params
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Color;

  fn backdrop(opacity: f32) -> BackdropOverlayParams {
    BackdropOverlayParams {
      tint: Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
      },
      blur_amount: 30.0,
      corner_radius: 8.0,
      opacity,
      saturation: 1.0,
      exposure: 0.0,
      contrast: 0.0,
      highlights: 0.0,
      shadows: 0.0,
      vignette: 0.0,
      grain: 0.0,
      parallax: 0.0,
    }
  }

  fn border(opacity: f32) -> BorderOverlayParams {
    BorderOverlayParams {
      color: Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
      },
      width: 2.0,
      corner_radius: 8.0,
      opacity,
    }
  }

  #[test]
  fn with_opacity_scaled_scales_both_overlays() {
    let params = OverlayParams {
      backdrop: Some(backdrop(0.8)),
      border: Some(border(1.0)),
    };

    assert_eq!(
      params.with_opacity_scaled(0.5),
      OverlayParams {
        backdrop: Some(backdrop(0.4)),
        border: Some(border(0.5)),
      }
    );
  }

  #[test]
  fn with_opacity_scaled_keeps_disabled_overlays_off() {
    let params = OverlayParams {
      backdrop: None,
      border: Some(border(1.0)),
    };

    assert_eq!(params.with_opacity_scaled(0.25).backdrop, None);
  }
}
