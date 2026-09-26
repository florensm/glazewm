//! How a [`ColorTheme`] is laid out for the shader: its filters in
//! numbered slots, and UI element rects as regions pointing at them.

use crate::color_theme::{
  element_filters, ColorTheme, ElementTreatment, FilterConstants,
  UiElementKind, MAX_ELEMENT_FILTERS,
};

/// Filters one window renders with: the theme's own, then its element
/// filters.
pub(crate) const MAX_FILTER_SLOTS: usize = 1 + MAX_ELEMENT_FILTERS;

/// Element slot that shows the captured pixels unchanged.
pub(crate) const SLOT_ORIGINAL: u32 = u32::MAX;

/// Maximum number of UI element regions per window; sizes the shader's
/// constant buffer.
pub(crate) const MAX_REGIONS: usize = 64;

/// Constant buffer layout shared with `cbuffer Filters` in the shader.
pub(crate) type FilterSlots = [FilterConstants; MAX_FILTER_SLOTS];

/// A UI element's bounds within the captured frame, in physical pixels
/// (right and bottom exclusive).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ElementRect {
  pub kind: UiElementKind,
  pub ltrb: [i32; 4],
}

/// The filter in each slot: the theme's own first, then its element
/// filters. Unused slots are the identity.
pub(crate) fn filter_slots(theme: &ColorTheme) -> FilterSlots {
  let mut slots = [FilterConstants::IDENTITY; MAX_FILTER_SLOTS];
  let options = theme.options();
  slots[0] = *options.filter.constants;

  for (slot, filter) in slots[1..].iter_mut().zip(element_filters(options))
  {
    *slot = *filter.constants;
  }

  slots
}

/// Filter slot of each element kind the theme renders differently (see
/// [`filter_slots`]), or [`SLOT_ORIGINAL`].
pub(crate) fn element_slots(
  theme: &ColorTheme,
) -> Vec<(UiElementKind, u32)> {
  let options = theme.options();
  let filters = element_filters(options);

  options
    .elements
    .iter()
    .map(|(kind, treatment)| {
      let slot = match treatment {
        ElementTreatment::Original => SLOT_ORIGINAL,
        ElementTreatment::Filter(filter) => filters
          .iter()
          .position(|other| *other == filter)
          // Bounded by `MAX_ELEMENT_FILTERS`.
          .map_or(0, |index| u32::try_from(index + 1).unwrap_or(0)),
      };

      (*kind, slot)
    })
    .collect()
}

/// The regions `elements` render with, as `(ltrb, slot)`: clipped to the
/// `size` of the frame, largest first so nested elements win in the
/// shader, and capped at [`MAX_REGIONS`] (dropping the smallest).
pub(crate) fn element_regions(
  theme: &ColorTheme,
  elements: &[ElementRect],
  size: (u32, u32),
) -> Vec<([i32; 4], u32)> {
  let slots = element_slots(theme);
  let width = i32::try_from(size.0).unwrap_or(i32::MAX);
  let height = i32::try_from(size.1).unwrap_or(i32::MAX);

  let mut regions = elements
    .iter()
    .filter_map(|element| {
      let slot = slots
        .iter()
        .find(|(kind, _)| *kind == element.kind)
        .map(|(_, slot)| *slot)?;

      let [left, top, right, bottom] = element.ltrb;
      let clipped = [
        left.max(0),
        top.max(0),
        right.min(width),
        bottom.min(height),
      ];

      (clipped[0] < clipped[2] && clipped[1] < clipped[3])
        .then_some((clipped, slot))
    })
    .collect::<Vec<_>>();

  let area = |ltrb: &[i32; 4]| {
    i64::from(ltrb[2] - ltrb[0]) * i64::from(ltrb[3] - ltrb[1])
  };
  regions.sort_by_key(|(ltrb, _)| std::cmp::Reverse(area(ltrb)));
  regions.truncate(MAX_REGIONS);
  regions
}

#[cfg(test)]
mod tests {
  use std::{collections::BTreeMap, str::FromStr};

  use super::*;
  use crate::{
    color_theme::{
      ColorFilter, ColorFilterOptions, ColorThemeOptions, RampStop,
    },
    Color,
  };

  fn filter(background: &str) -> ColorFilter {
    let color =
      |hex: &str| Color::from_str(hex).expect("valid test color");

    ColorFilter::new(&ColorFilterOptions {
      ramp: vec![
        RampStop {
          from: color("#ffffff"),
          to: color(background),
        },
        RampStop {
          from: color("#000000"),
          to: color("#d4d4d4"),
        },
      ],
      ..ColorFilterOptions::default()
    })
    .expect("valid filter")
  }

  fn theme(
    elements: impl IntoIterator<Item = (UiElementKind, ElementTreatment)>,
  ) -> ColorTheme {
    ColorTheme::new(ColorThemeOptions {
      filter: filter("#1e1e1e"),
      elements: elements.into_iter().collect::<BTreeMap<_, _>>(),
      detect_colors: false,
      skip_if_dark: false,
    })
    .expect("valid theme")
  }

  #[test]
  fn element_filters_share_slots() {
    let theme = theme([
      (UiElementKind::Hyperlink, ElementTreatment::Original),
      (
        UiElementKind::Edit,
        ElementTreatment::Filter(filter("#202040")),
      ),
      (
        UiElementKind::Button,
        ElementTreatment::Filter(filter("#402020")),
      ),
      (
        UiElementKind::ComboBox,
        ElementTreatment::Filter(filter("#202040")),
      ),
      (
        UiElementKind::TabItem,
        ElementTreatment::Filter(filter("#1e1e1e")),
      ),
    ]);

    assert_eq!(
      element_slots(&theme),
      vec![
        (UiElementKind::Edit, 1),
        (UiElementKind::Button, 2),
        (UiElementKind::Hyperlink, SLOT_ORIGINAL),
        (UiElementKind::ComboBox, 1),
        (UiElementKind::TabItem, 0),
      ]
    );

    let slots = filter_slots(&theme);
    assert_eq!(slots[0], *filter("#1e1e1e").constants);
    assert_eq!(slots[1], *filter("#202040").constants);
    assert_eq!(slots[3], FilterConstants::IDENTITY);
  }

  #[test]
  fn element_regions_are_filtered_clipped_and_ordered() {
    let theme = theme([
      (UiElementKind::Button, ElementTreatment::Original),
      (
        UiElementKind::Edit,
        ElementTreatment::Filter(filter("#202040")),
      ),
    ]);

    let element = |kind, ltrb| ElementRect { kind, ltrb };
    let regions = element_regions(
      &theme,
      &[
        // Partly left of the frame.
        element(UiElementKind::Button, [-100, 50, 100, 250]),
        element(UiElementKind::Edit, [0, 0, 400, 30]),
        // Not configured.
        element(UiElementKind::ListItem, [0, 0, 50, 50]),
        // Entirely outside the frame.
        element(UiElementKind::Edit, [900, 900, 950, 950]),
      ],
      (800, 600),
    );

    assert_eq!(
      regions,
      vec![([0, 50, 100, 250], SLOT_ORIGINAL), ([0, 0, 400, 30], 1)]
    );
  }
}
