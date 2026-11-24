//! Titles and app icons on the exposé's windows, beyond the picked
//! window's own tag: `expose.titles: always` and `expose.app_icons`. Each
//! picture is drawn once at a set size and scaled with its window, so
//! moving or zooming one redraws nothing.

use windows::Win32::Foundation::HWND;

use crate::{
  dwm_thumbnail::Thumbnail,
  overview::{
    chrome::{
      draw_icon_badge, draw_note, draw_tag, note_size, tag_size,
      TagPicture, ICON_BADGE,
    },
    layout::RectF,
    thumbnails::Picture,
  },
  OverviewStyle,
};

/// Most badges shown at once.
const MAX_BADGES: usize = 48;

/// Widest a window's title shows, in logical pixels.
const TITLE_WIDTH: f32 = 320.0;

/// Part of its window's shorter side an icon takes at most.
const ICON_SHARE: f32 = 0.4;

/// What a badge shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum BadgeContent {
  Title(TagPicture),
  /// A window's app icon; `u64` bumps as icons arrive.
  Icon(isize, u64),
  /// The letter that picks a window while `Alt` shows hints.
  Hint(char),
}

/// A badge on a scene window.
pub(super) struct SceneBadge {
  pub content: BadgeContent,
  /// Where the window shows.
  pub rect: RectF,
  pub opacity: f32,
}

struct Slot {
  picture: Picture,
  drawn: Option<BadgeContent>,
}

/// Pictures for the badges of the scenes' windows.
#[derive(Default)]
pub(super) struct SceneBadges {
  slots: Vec<Slot>,
}

impl SceneBadges {
  /// Registers on `host`, above everything registered so far, room for
  /// as many badges as `count`, keeping the pictures of those it had.
  pub fn register(&mut self, host: HWND, count: usize) {
    let count = count.min(MAX_BADGES);
    self.slots.truncate(count);
    while self.slots.len() < count {
      let Some(picture) = Picture::new() else {
        break;
      };
      self.slots.push(Slot {
        picture,
        drawn: None,
      });
    }
    for slot in &mut self.slots {
      slot.picture.register(host);
    }
  }

  /// Hands their thumbnails over to `retired`, still registered.
  pub fn retire(&mut self, retired: &mut Vec<Thumbnail>) {
    for slot in &mut self.slots {
      slot.picture.retire(retired);
    }
  }

  /// Shows `badges`, cut to `clip`, each in the picture that last drew
  /// it if there is one, and hides the pictures left over.
  pub fn place(
    &mut self,
    badges: &[SceneBadge],
    clip: &RectF,
    style: &OverviewStyle,
    scale_factor: f32,
    notify: (HWND, u32),
  ) {
    let px = |length: f32| length * scale_factor;
    let mut used = vec![false; self.slots.len()];

    for badge in badges {
      let index = self
        .slots
        .iter()
        .enumerate()
        .position(|(index, slot)| {
          !used[index] && slot.drawn.as_ref() == Some(&badge.content)
        })
        .or_else(|| {
          (0..self.slots.len()).find(|&index| {
            !used[index]
              && !badges.iter().any(|other| {
                self.slots[index].drawn.as_ref() == Some(&other.content)
              })
          })
        });
      let Some(index) = index else {
        continue;
      };
      used[index] = true;
      let slot = &mut self.slots[index];

      if slot.drawn.as_ref() != Some(&badge.content) {
        match &badge.content {
          BadgeContent::Title(tag) => {
            let size = tag_size(tag, style, scale_factor, px(TITLE_WIDTH));
            slot.picture.draw(size, |surface| {
              draw_tag(surface, tag, style, scale_factor, notify);
            });
          }
          BadgeContent::Hint(letter) => {
            let text = letter.to_ascii_uppercase().to_string();
            let size = note_size(&text, style, scale_factor);
            slot.picture.draw(size, |surface| {
              draw_note(surface, &text, style, scale_factor);
            });
          }
          BadgeContent::Icon(hwnd, _) => {
            #[allow(clippy::cast_possible_truncation)]
            let side = px(ICON_BADGE).round().max(1.0) as i32;
            slot.picture.draw((side, side), |surface| {
              draw_icon_badge(surface, *hwnd, style, notify);
            });
          }
        }
        slot.drawn = Some(badge.content.clone());
      }

      let dest =
        badge_rect(&badge.content, &badge.rect, slot.picture.size(), px);
      slot.picture.place(&dest, clip, badge.opacity);
    }

    for (slot, is_used) in self.slots.iter_mut().zip(used) {
      if !is_used {
        slot.picture.hide();
      }
    }
  }
}

/// Where a badge `size` big shows on a window at `rect`: a title over the
/// window's bottom edge (under it, if the window is too short), an icon
/// in its middle, shrunk to fit.
#[allow(clippy::cast_precision_loss)]
fn badge_rect(
  content: &BadgeContent,
  rect: &RectF,
  (width, height): (i32, i32),
  px: impl Fn(f32) -> f32,
) -> RectF {
  let (width, height) = (width as f32, height as f32);
  let (cx, cy) = rect.center();
  match content {
    BadgeContent::Title(_) => {
      let y = if rect.h > height * 2.5 {
        rect.bottom() - height - px(10.0)
      } else {
        rect.bottom() + px(6.0)
      };
      RectF::new(cx - width / 2.0, y, width, height)
    }
    BadgeContent::Icon(..) => {
      let side = width.min(rect.w.min(rect.h) * ICON_SHARE);
      RectF::new(cx - side / 2.0, cy - side / 2.0, side, side)
    }
    // Along its top, clear of the icon and the title.
    BadgeContent::Hint(_) => {
      RectF::new(cx - width / 2.0, rect.y + px(12.0), width, height)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{badge_rect, BadgeContent, ICON_SHARE};
  use crate::overview::{chrome::TagPicture, layout::RectF};

  fn title() -> BadgeContent {
    BadgeContent::Title(TagPicture {
      hwnd: 1,
      text: "Editor".to_string(),
      icons: 0,
    })
  }

  /// A title sits over a tall window's bottom edge, inside it, and under
  /// a short one; centered either way.
  #[test]
  fn titles_sit_on_or_under_their_window() {
    let tall = RectF::new(100.0, 100.0, 400.0, 300.0);
    let rect = badge_rect(&title(), &tall, (200, 30), |length| length);
    assert!(rect.bottom() <= tall.bottom() && rect.y > tall.y);
    assert!((rect.center().0 - tall.center().0).abs() < 0.01);

    let short = RectF::new(100.0, 100.0, 400.0, 60.0);
    let rect = badge_rect(&title(), &short, (200, 30), |length| length);
    assert!(rect.y > short.bottom());
  }

  /// An icon shows centered, at its size or shrunk to fit a small window.
  #[test]
  fn icons_center_and_shrink_to_fit() {
    let large = RectF::new(0.0, 0.0, 800.0, 600.0);
    let rect =
      badge_rect(&BadgeContent::Icon(1, 0), &large, (72, 72), |l| l);
    assert_eq!(rect.center(), large.center());
    assert!((rect.w - 72.0).abs() < 0.01);

    let small = RectF::new(0.0, 0.0, 120.0, 90.0);
    let rect =
      badge_rect(&BadgeContent::Icon(1, 0), &small, (72, 72), |l| l);
    assert!((rect.w - 90.0 * ICON_SHARE).abs() < 0.01);
  }
}
