//! Keeping pictures (photos, illustrations, dark banners) in their own
//! colors under a theme, for themes with `keep_pictures`.
//!
//! Pictures are found from each frame's pixels alone:
//!
//! 1. The GPU counts, per [`BLOCK_SIZE`] block, the smooth steps between
//!    neighboring pixels along each axis, and the pixels of the window's
//!    background and dark-surface colors (`ps_blocks`).
//! 2. [`PictureBlocks::new`] turns the counts into picture blocks: blocks
//!    shaded smoothly along both axes (UI gradients shade along one),
//!    grown through blocks that aren't mostly background, plus big dark
//!    surfaces, with enclosed holes (e.g. text on a banner) filled.
//! 3. Near those blocks, the GPU finds each picture's exact pixels
//!    (`cs_keep`), then leaves them unchanged and themes the rest
//!    (`ps_pictures`).

use std::collections::HashMap;

/// Side of the square blocks pictures are found in. Mirrors the shader's
/// `BLOCK_SIZE`.
pub(crate) const BLOCK_SIZE: u32 = 8;

/// Side of the square tiles `cs_keep` runs on, in blocks. Mirrors the
/// shader's `TILE_SIZE`.
pub(crate) const TILE_BLOCKS: u32 = 2;

/// Most background and dark-surface colors passed to the shaders. Mirrors
/// the shader's `MAX_SURFACES`.
pub(crate) const MAX_SURFACES: usize = 8;

/// The block is surrounded by picture blocks: all of it is picture.
pub(crate) const BLOCK_INTERIOR: u8 = 1;
/// The block is or touches a picture block: pixels in it can seed one.
pub(crate) const BLOCK_NEAR: u8 = 2;
/// The block may hold a picture's edge; decided per pixel by `cs_keep`.
pub(crate) const BLOCK_BAND: u8 = 4;

/// Fewer samples than this (e.g. a tiny popup) aren't worth measuring.
const MIN_SAMPLES: usize = 64;

/// Share of the window light colors must cover for the most common of
/// them to be the page; otherwise a dark hero section, outnumbering the
/// page, would be taken for it.
const MIN_LIGHT_SHARE: f32 = 0.2;

/// Share of the window a color must cover to be a surface.
const MIN_SURFACE_SHARE: f32 = 0.01;

/// Luma difference from the page within which a big flat color is
/// background. Darker ones in a light window are content (a banner),
/// harmless to keep under a dark theme.
const MAX_BACKGROUND_LUMA_DIFFERENCE: f32 = 0.3;

/// Luma below which a big flat color on a light page is a dark surface.
const MAX_DARK_SURFACE_LUMA: f32 = 0.35;

/// Share of smooth steps, along a block's less smooth axis, above which it
/// seeds a picture.
const MIN_SMOOTH_SHARE: f32 = 0.25;

/// Background share below which a block is foreground, which pictures
/// grow through.
const MAX_FOREGROUND_BACKGROUND_SHARE: f32 = 0.2;

/// Dark-surface share from which a block is solid dark surface; a dark
/// surface is seeded where 3×3 such blocks meet, so bold text never
/// qualifies.
const SOLID_DARK_SHARE: f32 = 0.9;

/// Dark-surface share from which a block is part of a dark surface it
/// touches. Dark surfaces only grow through their own color, so text next
/// to them isn't taken.
const MIN_DARK_SHARE: f32 = 0.3;

/// Background share from which a block is plain page.
const PAGE_BLOCK_SHARE: f32 = 0.95;

/// Largest share of the grid a region cut off by the window's edge can
/// take to be a hole; see `Grid::fill_holes`.
const MAX_EDGE_HOLE_SHARE: f32 = 0.1;

/// How far, in blocks, kept pixels can be from a picture block: seeds lie
/// in blocks next to one, and `cs_keep` grows them by up to 16 px.
const BAND_BLOCKS: usize = 3;

/// A window's big flat colors, measured from a sample of its pixels.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PictureSurfaces {
  /// The page, and other big flat colors near its lightness, most common
  /// first.
  pub backgrounds: Vec<[u8; 3]>,

  /// Big flat dark colors on a light page (banners, hero sections), most
  /// common first. Kept like pictures, since theming would turn them
  /// light.
  pub dark: Vec<[u8; 3]>,
}

/// Constant buffer layout shared with `cbuffer Pictures` in the shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PictureConstants {
  /// Background count, dark-surface count, tile count, then padding.
  counts: [u32; 4],
  backgrounds: [[f32; 4]; MAX_SURFACES],
  dark_surfaces: [[f32; 4]; MAX_SURFACES],
}

impl PictureConstants {
  pub(crate) fn set_tile_count(&mut self, count: u32) {
    self.counts[2] = count;
  }
}

impl PictureSurfaces {
  /// The constants for these surfaces, with no tiles.
  #[must_use]
  pub(crate) fn constants(&self) -> PictureConstants {
    fn pack(colors: &[[u8; 3]]) -> [[f32; 4]; MAX_SURFACES] {
      let mut packed = [[0.0; 4]; MAX_SURFACES];
      for (slot, color) in packed.iter_mut().zip(colors) {
        let [r, g, b] = srgb(*color);
        *slot = [r, g, b, 0.0];
      }
      packed
    }

    #[allow(clippy::cast_possible_truncation)]
    let count = |colors: &[[u8; 3]]| colors.len().min(MAX_SURFACES) as u32;

    PictureConstants {
      counts: [count(&self.backgrounds), count(&self.dark), 0, 0],
      backgrounds: pack(&self.backgrounds),
      dark_surfaces: pack(&self.dark),
    }
  }
}

/// Finds the page and the other big flat colors in straight-alpha sRGB
/// samples of a window.
#[must_use]
pub(crate) fn estimate_surfaces(
  samples: &[[f32; 3]],
) -> Option<PictureSurfaces> {
  if samples.len() < MIN_SAMPLES {
    return None;
  }

  let mut counts = HashMap::<[u8; 3], usize>::new();
  for sample in samples {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let color = sample.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
    *counts.entry(color).or_default() += 1;
  }

  // Most common first; ties by color, so the result doesn't depend on
  // hash order.
  let mut colors = counts.into_iter().collect::<Vec<_>>();
  colors.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

  #[allow(clippy::cast_precision_loss)]
  let share = |count: usize| count as f32 / samples.len() as f32;
  let is_light = |color: [u8; 3]| luma(color) > 0.5;

  let light_share = share(
    colors
      .iter()
      .filter(|(color, _)| is_light(*color))
      .map(|(_, count)| count)
      .sum(),
  );

  let paper = if light_share >= MIN_LIGHT_SHARE {
    colors.iter().find(|(color, _)| is_light(*color))
  } else {
    colors.first()
  }
  .map(|(color, _)| luma(*color))?;

  let surfaces = colors
    .iter()
    .filter(|(_, count)| share(*count) >= MIN_SURFACE_SHARE)
    .map(|(color, _)| *color);

  let backgrounds = surfaces
    .clone()
    .filter(|color| {
      (luma(*color) - paper).abs() < MAX_BACKGROUND_LUMA_DIFFERENCE
    })
    .take(MAX_SURFACES)
    .collect();

  let dark = if paper > 0.5 {
    surfaces
      .filter(|color| luma(*color) < MAX_DARK_SURFACE_LUMA)
      .take(MAX_SURFACES)
      .collect()
  } else {
    Vec::new()
  };

  Some(PictureSurfaces { backgrounds, dark })
}

fn srgb(color: [u8; 3]) -> [f32; 3] {
  color.map(|c| f32::from(c) / 255.0)
}

/// Rec. 709 luma of an sRGB color.
fn luma(color: [u8; 3]) -> f32 {
  let [r, g, b] = srgb(color);
  0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Number of blocks across and down a frame of `size`.
#[must_use]
pub(crate) fn block_grid(size: (u32, u32)) -> (u32, u32) {
  (size.0.div_ceil(BLOCK_SIZE), size.1.div_ceil(BLOCK_SIZE))
}

/// Where the pictures in a frame are, per block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PictureBlocks {
  /// `BLOCK_*` flags per block, row by row.
  pub flags: Vec<u8>,

  /// Tiles `cs_keep` runs on, as `x | y << 16` in tiles: those holding
  /// band blocks or their neighbors, whose pixels `ps_pictures` reads.
  pub tiles: Vec<u32>,
}

impl PictureBlocks {
  /// Finds the picture blocks from `ps_blocks`' counts per block (smooth
  /// horizontal steps, smooth vertical steps, background pixels,
  /// dark-surface pixels), row by row.
  #[must_use]
  pub(crate) fn new(counts: &[[u8; 4]], frame_size: (u32, u32)) -> Self {
    let (grid_width, grid_height) = block_grid(frame_size);
    let (width, height) = (grid_width as usize, grid_height as usize);
    let (frame_width, frame_height) =
      (frame_size.0 as usize, frame_size.1 as usize);
    let block = BLOCK_SIZE as usize;

    if counts.len() != width * height {
      return Self {
        flags: vec![0; width * height],
        tiles: Vec::new(),
      };
    }

    let mut seeds = vec![false; width * height];
    let mut foreground = vec![false; width * height];
    let mut solid_dark = vec![false; width * height];
    let mut dark_share = vec![0.0; width * height];
    let mut is_page = vec![false; width * height];

    for y in 0..height {
      for x in 0..width {
        let index = y * width + x;
        let [smooth_x, smooth_y, background, dark] =
          counts[index].map(f32::from);

        let columns = block.min(frame_width - x * block);
        let rows = block.min(frame_height - y * block);
        let is_last_column = x * block + columns == frame_width;
        let is_last_row = y * block + rows == frame_height;

        // Steps to the next pixel, which the frame's last column and row
        // don't have.
        #[allow(clippy::cast_precision_loss)]
        let steps_x =
          (rows * (columns - usize::from(is_last_column))) as f32;
        #[allow(clippy::cast_precision_loss)]
        let steps_y = (columns * (rows - usize::from(is_last_row))) as f32;
        #[allow(clippy::cast_precision_loss)]
        let pixels = (columns * rows) as f32;

        let share = |count: f32, total: f32| {
          if total > 0.0 {
            count / total
          } else {
            0.0
          }
        };
        let smoothness =
          share(smooth_x, steps_x).min(share(smooth_y, steps_y));

        foreground[index] =
          background / pixels < MAX_FOREGROUND_BACKGROUND_SHARE;
        is_page[index] = background / pixels >= PAGE_BLOCK_SHARE;
        seeds[index] = foreground[index] && smoothness > MIN_SMOOTH_SHARE;
        solid_dark[index] = dark / pixels >= SOLID_DARK_SHARE;
        dark_share[index] = dark / pixels;
      }
    }

    let grid = Grid { width, height };

    let pictures = grid.grow(seeds, |index| foreground[index]);
    let dark_seeds = (0..width * height)
      .map(|index| {
        grid
          .neighborhood(index, 1)
          .all(|neighbor| neighbor.is_some_and(|n| solid_dark[n]))
      })
      .collect();
    let dark =
      grid.grow(dark_seeds, |index| dark_share[index] >= MIN_DARK_SHARE);

    let mut mask = pictures
      .iter()
      .zip(&dark)
      .map(|(picture, dark)| *picture || *dark)
      .collect::<Vec<_>>();
    grid.fill_holes(&mut mask, &is_page);

    let within_band = grid.dilate(&mask, BAND_BLOCKS);

    let mut flags = vec![0u8; width * height];
    for (index, flag) in flags.iter_mut().enumerate() {
      let mut neighbors = grid.neighborhood(index, 1);

      // Outside the window counts as picture: a picture the window's edge
      // cuts off is still whole up to that edge.
      if neighbors.all(|neighbor| neighbor.is_none_or(|n| mask[n])) {
        *flag |= BLOCK_INTERIOR;
      } else if within_band[index] {
        *flag |= BLOCK_BAND;
      }

      if grid.neighborhood(index, 1).flatten().any(|n| mask[n]) {
        *flag |= BLOCK_NEAR;
      }
    }

    let tiles = grid.tiles(&flags);
    Self { flags, tiles }
  }
}

/// Block grid dimensions, for neighborhood walks.
struct Grid {
  width: usize,
  height: usize,
}

impl Grid {
  /// The `(2 * radius + 1)²` blocks around `index`, row by row; `None`
  /// outside the grid.
  fn neighborhood(
    &self,
    index: usize,
    radius: usize,
  ) -> impl Iterator<Item = Option<usize>> + '_ {
    let (x, y) = (index % self.width, index / self.width);
    let span = 2 * radius + 1;

    (0..span * span).map(move |offset| {
      let neighbor_x = (x + offset % span).checked_sub(radius)?;
      let neighbor_y = (y + offset / span).checked_sub(radius)?;
      (neighbor_x < self.width && neighbor_y < self.height)
        .then_some(neighbor_y * self.width + neighbor_x)
    })
  }

  /// Grows `seeds` through 8-connected blocks that `passable` allows.
  fn grow(
    &self,
    mut mask: Vec<bool>,
    passable: impl Fn(usize) -> bool,
  ) -> Vec<bool> {
    let mut stack =
      (0..mask.len()).filter(|i| mask[*i]).collect::<Vec<_>>();

    while let Some(index) = stack.pop() {
      for neighbor in self.neighborhood(index, 1).flatten() {
        if !mask[neighbor] && passable(neighbor) {
          mask[neighbor] = true;
          stack.push(neighbor);
        }
      }
    }

    mask
  }

  /// Adds the regions `mask` encloses (e.g. white text inside a banner):
  /// those the grid's border can't reach without crossing it, and small
  /// ones the border cuts off without a block of plain page, like a
  /// banner's text scrolled halfway out of the window.
  fn fill_holes(&self, mask: &mut [bool], is_page: &[bool]) {
    let mut seen = vec![false; mask.len()];

    for start in 0..mask.len() {
      if mask[start] || seen[start] {
        continue;
      }

      // The 4-connected region of non-mask blocks around `start`.
      let mut region = vec![start];
      seen[start] = true;
      let mut next = 0;

      while let Some(&index) = region.get(next) {
        next += 1;
        let (x, y) = (index % self.width, index / self.width);
        let neighbors = [
          (x > 0).then(|| index - 1),
          (x + 1 < self.width).then(|| index + 1),
          (y > 0).then(|| index - self.width),
          (y + 1 < self.height).then(|| index + self.width),
        ];

        for neighbor in neighbors.into_iter().flatten() {
          if !mask[neighbor] && !seen[neighbor] {
            seen[neighbor] = true;
            region.push(neighbor);
          }
        }
      }

      let touches_border = region.iter().any(|index| {
        let (x, y) = (index % self.width, index / self.width);
        x == 0 || y == 0 || x == self.width - 1 || y == self.height - 1
      });
      #[allow(clippy::cast_precision_loss)]
      let is_small =
        region.len() as f32 <= mask.len() as f32 * MAX_EDGE_HOLE_SHARE;

      if !touches_border
        || (is_small && !region.iter().any(|index| is_page[*index]))
      {
        for index in region {
          mask[index] = true;
        }
      }
    }
  }

  /// Tiles `cs_keep` must run on for `flags`: those holding band blocks,
  /// or their neighbors, since `ps_pictures` reads the pixel flags up to
  /// 3 px around a band pixel.
  fn tiles(&self, flags: &[u8]) -> Vec<u32> {
    let band = flags
      .iter()
      .map(|flag| flag & BLOCK_BAND != 0)
      .collect::<Vec<_>>();
    let needs_pixels = self.dilate(&band, 1);

    let tile = TILE_BLOCKS as usize;
    let mut tiles = Vec::new();
    for tile_y in 0..self.height.div_ceil(tile) {
      for tile_x in 0..self.width.div_ceil(tile) {
        let is_needed = (tile_y * tile
          ..((tile_y + 1) * tile).min(self.height))
          .any(|y| {
            (tile_x * tile..((tile_x + 1) * tile).min(self.width))
              .any(|x| needs_pixels[y * self.width + x])
          });

        if is_needed {
          #[allow(clippy::cast_possible_truncation)]
          tiles.push(tile_x as u32 | (tile_y as u32) << 16);
        }
      }
    }

    tiles
  }

  /// Blocks within `radius` (Chebyshev) of a block in `mask`.
  fn dilate(&self, mask: &[bool], radius: usize) -> Vec<bool> {
    let dilate_line = |get: &dyn Fn(usize) -> bool, length: usize| {
      (0..length)
        .map(|i| {
          (i.saturating_sub(radius)..(i + radius + 1).min(length)).any(get)
        })
        .collect::<Vec<_>>()
    };

    let mut rows = vec![false; mask.len()];
    for y in 0..self.height {
      let line = dilate_line(&|x| mask[y * self.width + x], self.width);
      rows[y * self.width..(y + 1) * self.width].copy_from_slice(&line);
    }

    let mut dilated = vec![false; mask.len()];
    for x in 0..self.width {
      let line = dilate_line(&|y| rows[y * self.width + x], self.height);
      for (y, value) in line.into_iter().enumerate() {
        dilated[y * self.width + x] = value;
      }
    }

    dilated
  }
}

/// CPU mirror of the picture shaders (`ps_blocks`, `cs_keep`,
/// `ps_pictures`), for tests and GPU parity checks.
#[cfg(test)]
pub(crate) mod reference {
  use std::collections::VecDeque;

  use super::{
    srgb, PictureBlocks, PictureSurfaces, BLOCK_BAND, BLOCK_INTERIOR,
    BLOCK_NEAR, BLOCK_SIZE,
  };
  use crate::color_theme::{ColorFilter, SourceLevels, NEIGHBORHOOD_SIZE};

  /// Mirror the shader's constants of the same names.
  const SMOOTH_STEP_MIN: f32 = 0.008;
  const SMOOTH_STEP_MAX: f32 = 0.12;
  const SURFACE_TOLERANCE: f32 = 0.02;
  const GROW: usize = 16;
  const SOLID_RADIUS: i32 = 3;
  const RIM: i32 = 2;
  const RIM_SEARCH: i32 = 3;
  const RIM_MAX_DISTANCE: f32 = 0.25;
  const MIN_RIM_SPAN: f32 = 0.02;

  pub const PIXEL_KEPT: u8 = 1;
  pub const PIXEL_CORE: u8 = 2;
  pub const PIXEL_PAGE: u8 = 4;

  /// An opaque frame of straight sRGB pixels, row by row.
  pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[f32; 3]>,
  }

  impl Image {
    #[must_use]
    pub fn at(&self, x: usize, y: usize) -> [f32; 3] {
      self.pixels[y * self.width + x]
    }

    fn size(&self) -> (u32, u32) {
      #[allow(clippy::cast_possible_truncation)]
      (self.width as u32, self.height as u32)
    }

    fn offset(
      &self,
      x: usize,
      y: usize,
      dx: i32,
      dy: i32,
    ) -> Option<(usize, usize)> {
      let x = x.checked_add_signed(dx as isize)?;
      let y = y.checked_add_signed(dy as isize)?;
      (x < self.width && y < self.height).then_some((x, y))
    }
  }

  fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2))
      .sqrt()
  }

  fn nearest(color: [f32; 3], surfaces: &[[u8; 3]]) -> f32 {
    surfaces
      .iter()
      .map(|surface| distance(color, srgb(*surface)))
      .fold(f32::MAX, f32::min)
  }

  fn is_background(color: [f32; 3], surfaces: &PictureSurfaces) -> bool {
    nearest(color, &surfaces.backgrounds) < SURFACE_TOLERANCE
  }

  /// `ps_blocks`.
  #[must_use]
  pub fn block_counts(
    image: &Image,
    surfaces: &PictureSurfaces,
  ) -> Vec<[u8; 4]> {
    let block = BLOCK_SIZE as usize;
    let (width, height) =
      (image.width.div_ceil(block), image.height.div_ceil(block));
    let mut counts = vec![[0u8; 4]; width * height];

    for y in 0..image.height {
      for x in 0..image.width {
        let color = image.at(x, y);
        let count = &mut counts[(y / block) * width + x / block];
        let is_smooth = |other: [f32; 3]| {
          let step = distance(color, other);
          step > SMOOTH_STEP_MIN && step < SMOOTH_STEP_MAX
        };

        if x + 1 < image.width && is_smooth(image.at(x + 1, y)) {
          count[0] += 1;
        }
        if y + 1 < image.height && is_smooth(image.at(x, y + 1)) {
          count[1] += 1;
        }
        if is_background(color, surfaces) {
          count[2] += 1;
        }
        if nearest(color, &surfaces.dark) < SURFACE_TOLERANCE {
          count[3] += 1;
        }
      }
    }

    counts
  }

  /// `cs_keep`, for every pixel rather than per tile.
  #[must_use]
  pub fn pixel_flags(
    image: &Image,
    surfaces: &PictureSurfaces,
    blocks: &PictureBlocks,
  ) -> Vec<u8> {
    let (width, height) = (image.width, image.height);
    let block = BLOCK_SIZE as usize;
    let grid_width = width.div_ceil(block);
    let is_page = image
      .pixels
      .iter()
      .map(|color| is_background(*color, surfaces))
      .collect::<Vec<_>>();

    let page_within = |x: usize, y: usize, radius: i32| {
      (-radius..=radius).any(|dy| {
        (-radius..=radius).any(|dx| {
          image
            .offset(x, y, dx, dy)
            .is_some_and(|(nx, ny)| is_page[ny * width + nx])
        })
      })
    };

    // Kept: within `GROW` steps of a seed, through pixels that aren't
    // page.
    let mut steps = vec![usize::MAX; width * height];
    let mut queue = VecDeque::new();
    for y in 0..height {
      for x in 0..width {
        let flags = blocks.flags[(y / block) * grid_width + x / block];
        let is_seed = flags & BLOCK_INTERIOR != 0
          || (flags & BLOCK_NEAR != 0
            && !is_page[y * width + x]
            && !page_within(x, y, SOLID_RADIUS));

        if is_seed {
          steps[y * width + x] = 0;
          queue.push_back((x, y));
        }
      }
    }

    while let Some((x, y)) = queue.pop_front() {
      let step = steps[y * width + x];
      if step == GROW {
        continue;
      }

      for dy in -1..=1 {
        for dx in -1..=1 {
          if let Some((nx, ny)) = image.offset(x, y, dx, dy) {
            let index = ny * width + nx;
            if steps[index] == usize::MAX && !is_page[index] {
              steps[index] = step + 1;
              queue.push_back((nx, ny));
            }
          }
        }
      }
    }

    (0..width * height)
      .map(|index| {
        let (x, y) = (index % width, index / width);
        let mut flags = 0;

        if is_page[index] {
          flags |= PIXEL_PAGE;
        }

        if steps[index] != usize::MAX {
          flags |= PIXEL_KEPT;

          if !page_within(x, y, RIM)
            || nearest(image.pixels[index], &surfaces.backgrounds)
              > RIM_MAX_DISTANCE
          {
            flags |= PIXEL_CORE;
          }
        }

        flags
      })
      .collect()
  }

  /// `ps_pictures`.
  #[must_use]
  pub fn compose(
    image: &Image,
    blocks: &PictureBlocks,
    pixel_flags: &[u8],
    filter: &ColorFilter,
    levels: SourceLevels,
  ) -> Vec<[f32; 3]> {
    let block = BLOCK_SIZE as usize;
    let grid_width = image.width.div_ceil(block);

    let themed_at = |x: usize, y: usize| {
      let mut pixels = [[0.0; 3]; NEIGHBORHOOD_SIZE];
      for dy in -1isize..=1 {
        for dx in -2isize..=2 {
          let nx = x.saturating_add_signed(dx).min(image.width - 1);
          let ny = y.saturating_add_signed(dy).min(image.height - 1);
          #[allow(clippy::cast_sign_loss)]
          let slot = ((dy + 1) * 5 + dx + 2) as usize;
          pixels[slot] = image.at(nx, ny);
        }
      }
      filter.apply_neighborhood(&pixels, levels)
    };

    (0..image.width * image.height)
      .map(|index| {
        let (x, y) = (index % image.width, index / image.width);
        let color = image.pixels[index];
        let block_flags =
          blocks.flags[(y / block) * grid_width + x / block];

        if block_flags & BLOCK_INTERIOR != 0 {
          return color;
        }

        if block_flags & BLOCK_BAND != 0 {
          let flags = pixel_flags[index];

          if flags & PIXEL_CORE != 0 {
            return color;
          }

          if flags & PIXEL_KEPT != 0 {
            if let Some(remixed) =
              remix_rim(image, pixel_flags, x, y, &themed_at)
            {
              return remixed;
            }
          }
        }

        themed_at(x, y)
      })
      .collect()
  }

  /// A kept pixel at a picture's edge, a mix of page and picture: re-mixed
  /// from the themed page and the picture's own color next to it, at the
  /// same coverage. `None` without both nearby.
  fn remix_rim(
    image: &Image,
    pixel_flags: &[u8],
    x: usize,
    y: usize,
    themed_at: &dyn Fn(usize, usize) -> [f32; 3],
  ) -> Option<[f32; 3]> {
    let mut page = None;
    let mut picture = None;
    let mut best = i32::MAX;

    for dy in -RIM_SEARCH..=RIM_SEARCH {
      for dx in -RIM_SEARCH..=RIM_SEARCH {
        let Some((nx, ny)) = image.offset(x, y, dx, dy) else {
          continue;
        };
        let flags = pixel_flags[ny * image.width + nx];

        if flags & PIXEL_PAGE != 0 {
          page.get_or_insert((nx, ny));
        } else if flags & PIXEL_CORE != 0 && dx * dx + dy * dy < best {
          best = dx * dx + dy * dy;
          picture = Some((nx, ny));
        }
      }
    }

    let (page_x, page_y) = page?;
    let picture = image.at(picture?.0, picture?.1);
    let page = image.at(page_x, page_y);
    let color = image.at(x, y);

    let span = distance(page, picture);
    let coverage = if span > MIN_RIM_SPAN {
      (distance(color, page) / span).clamp(0.0, 1.0)
    } else {
      1.0
    };
    let themed_page = themed_at(page_x, page_y);

    Some([0, 1, 2].map(|ch| {
      themed_page[ch] + (picture[ch] - themed_page[ch]) * coverage
    }))
  }

  /// The whole pipeline: `image` themed by `filter`, pictures kept.
  #[must_use]
  pub fn keep_pictures(
    image: &Image,
    surfaces: &PictureSurfaces,
    filter: &ColorFilter,
    levels: SourceLevels,
  ) -> Vec<[f32; 3]> {
    let blocks =
      PictureBlocks::new(&block_counts(image, surfaces), image.size());
    let flags = pixel_flags(image, surfaces, &blocks);
    compose(image, &blocks, &flags, filter, levels)
  }
}

// Kept pixels are compared exactly: they must be the captured ones.
#[allow(clippy::float_cmp)]
#[cfg(test)]
mod tests {
  use super::{
    block_grid, estimate_surfaces,
    reference::{block_counts, keep_pictures, Image},
    srgb, PictureBlocks, PictureSurfaces, BLOCK_BAND, BLOCK_INTERIOR,
    TILE_BLOCKS,
  };
  use crate::{
    color_theme::{
      ColorFilter, ColorFilterOptions, RampStop, SourceLevels,
    },
    Color,
  };

  const WHITE: [u8; 3] = [255, 255, 255];
  const NAVY: [u8; 3] = [5, 40, 90];

  fn sample(color: [u8; 3]) -> [f32; 3] {
    srgb(color)
  }

  #[test]
  fn finds_page_backgrounds_and_dark_surfaces() {
    let mut samples = vec![sample(WHITE); 600];
    samples.extend(vec![sample([240, 240, 240]); 100]);
    samples.extend(vec![sample(NAVY); 250]);
    // Text, and a color too rare to be a surface.
    samples.extend(vec![sample([0, 0, 0]); 45]);
    samples.extend(vec![sample([200, 0, 0]); 5]);

    let surfaces = estimate_surfaces(&samples).expect("enough samples");

    assert_eq!(surfaces.backgrounds, vec![WHITE, [240, 240, 240]]);
    assert_eq!(surfaces.dark, vec![NAVY, [0, 0, 0]]);
  }

  #[test]
  fn page_is_the_most_common_light_color_under_a_dark_hero() {
    let mut samples = vec![sample([0, 0, 0]); 600];
    samples.extend(vec![sample([245, 245, 247]); 300]);
    samples.extend(vec![sample(WHITE); 100]);

    let surfaces = estimate_surfaces(&samples).expect("enough samples");

    assert_eq!(surfaces.backgrounds[0], [245, 245, 247]);
    assert_eq!(surfaces.dark, vec![[0, 0, 0]]);
  }

  #[test]
  fn dark_windows_have_no_dark_surfaces() {
    let mut samples = vec![sample([30, 30, 30]); 900];
    samples.extend(vec![sample([10, 10, 10]); 100]);

    let surfaces = estimate_surfaces(&samples).expect("enough samples");

    assert_eq!(surfaces.backgrounds, vec![[30, 30, 30], [10, 10, 10]]);
    assert_eq!(surfaces.dark, Vec::<[u8; 3]>::new());
    assert!(estimate_surfaces(&samples[..10]).is_none());
  }

  fn white_page(width: usize, height: usize) -> Image {
    Image {
      width,
      height,
      pixels: vec![sample(WHITE); width * height],
    }
  }

  fn fill(
    image: &mut Image,
    rect: (usize, usize, usize, usize),
    color: impl Fn(usize, usize) -> [u8; 3],
  ) {
    for y in rect.1..rect.1 + rect.3 {
      for x in rect.0..rect.0 + rect.2 {
        image.pixels[y * image.width + x] =
          sample(color(x - rect.0, y - rect.1));
      }
    }
  }

  /// Shaded along both axes, in steps of 3/255 per channel.
  #[allow(clippy::cast_possible_truncation)]
  fn photo(x: usize, y: usize) -> [u8; 3] {
    [(40 + x * 3) as u8, (30 + y * 3) as u8, 90]
  }

  /// Shaded top to bottom only, like a toolbar.
  #[allow(clippy::cast_possible_truncation)]
  fn toolbar(_: usize, y: usize) -> [u8; 3] {
    let value = (250 - y * 3) as u8;
    [value, value, 255]
  }

  fn surfaces() -> PictureSurfaces {
    PictureSurfaces {
      backgrounds: vec![WHITE],
      dark: vec![NAVY],
    }
  }

  fn filter() -> ColorFilter {
    let stop = |from: [u8; 3], to: [u8; 3]| RampStop {
      from: Color {
        r: from[0],
        g: from[1],
        b: from[2],
        a: 255,
      },
      to: Color {
        r: to[0],
        g: to[1],
        b: to[2],
        a: 255,
      },
    };

    ColorFilter::new(&ColorFilterOptions {
      ramp: vec![
        stop(WHITE, [30, 30, 46]),
        stop([0, 0, 0], [205, 214, 244]),
      ],
      ..ColorFilterOptions::default()
    })
    .expect("valid filter")
  }

  #[test]
  fn keeps_a_photo_and_themes_text_beside_it() {
    let mut image = white_page(160, 96);
    fill(&mut image, (16, 16, 56, 56), photo);
    // A 1 px "glyph" 8 px right of the photo.
    fill(&mut image, (80, 30, 1, 12), |_, _| [0, 0, 0]);

    let out = keep_pictures(
      &image,
      &surfaces(),
      &filter(),
      SourceLevels::default(),
    );
    let at = |x: usize, y: usize| out[y * image.width + x];

    for y in 16..72 {
      for x in 16..72 {
        assert_eq!(at(x, y), image.at(x, y), "photo pixel ({x}, {y})");
      }
    }

    assert_ne!(at(80, 35), image.at(80, 35), "text is themed");
    assert_ne!(at(120, 80), image.at(120, 80), "page is themed");
    assert_ne!(
      at(15, 40),
      image.at(15, 40),
      "page next to the photo is themed"
    );
  }

  #[test]
  fn keeps_small_detailed_pictures_at_any_grid_alignment() {
    // A 28 px avatar with fine detail (steps of 20/255), like a shrunk
    // photo; list rows 36 px apart put it at both half-block offsets.
    #[allow(clippy::cast_possible_truncation)]
    let detailed =
      |x: usize, y: usize| [(120 + 20 * ((x + y) % 2)) as u8, 80, 60];

    for top in [16, 20] {
      let mut image = white_page(96, 64);
      fill(&mut image, (16, top, 28, 28), detailed);

      let out = keep_pictures(
        &image,
        &surfaces(),
        &filter(),
        SourceLevels::default(),
      );

      assert_eq!(
        out[(top + 14) * image.width + 30],
        image.at(30, top + 14),
        "avatar at y {top} is kept"
      );
    }
  }

  #[test]
  fn themes_one_directional_ui_gradients() {
    let mut image = white_page(160, 64);
    fill(&mut image, (0, 8, 160, 24), toolbar);

    let out = keep_pictures(
      &image,
      &surfaces(),
      &filter(),
      SourceLevels::default(),
    );

    assert!(
      (0..image.pixels.len()).all(|i| out[i] != image.pixels[i]),
      "every pixel is themed"
    );
  }

  #[test]
  fn keeps_dark_banners_with_their_text() {
    let mut image = white_page(160, 96);
    fill(&mut image, (0, 16, 160, 48), |_, _| NAVY);
    // White text on the banner, and black text on the page below it.
    fill(&mut image, (40, 36, 30, 2), |_, _| WHITE);
    fill(&mut image, (40, 80, 30, 2), |_, _| [0, 0, 0]);

    let out = keep_pictures(
      &image,
      &surfaces(),
      &filter(),
      SourceLevels::default(),
    );
    let at = |x: usize, y: usize| out[y * image.width + x];

    assert_eq!(at(50, 20), sample(NAVY), "banner is kept");
    assert_eq!(at(50, 36), sample(WHITE), "its text is kept");
    assert_ne!(at(50, 80), sample([0, 0, 0]), "page text is themed");
  }

  #[test]
  fn keeps_banner_text_cut_off_by_the_window_edge() {
    let mut image = white_page(160, 96);
    fill(&mut image, (0, 48, 160, 48), |_, _| NAVY);
    // White text on the banner, running out of the window's bottom edge.
    for x in (40..120).step_by(4) {
      fill(&mut image, (x, 84, 2, 12), |_, _| WHITE);
    }

    let out = keep_pictures(
      &image,
      &surfaces(),
      &filter(),
      SourceLevels::default(),
    );
    let at = |x: usize, y: usize| out[y * image.width + x];

    assert_eq!(at(40, 95), sample(WHITE), "text at the edge is kept");
    assert_eq!(at(42, 95), sample(NAVY), "the banner around it too");
    assert_ne!(at(40, 20), sample(WHITE), "the page is themed");
  }

  #[test]
  fn tiles_cover_every_band_block_and_its_neighbors() {
    let mut image = white_page(200, 120);
    fill(&mut image, (50, 30, 64, 48), photo);

    let surfaces = surfaces();
    let blocks =
      PictureBlocks::new(&block_counts(&image, &surfaces), (200, 120));
    let (width, height) = block_grid((200, 120));
    let tile = TILE_BLOCKS as usize;
    let covered = |x: usize, y: usize| {
      #[allow(clippy::cast_possible_truncation)]
      let id = (x / tile) as u32 | ((y / tile) as u32) << 16;
      blocks.tiles.contains(&id)
    };

    assert!(blocks.flags.iter().any(|f| f & BLOCK_INTERIOR != 0));
    for y in 0..height as usize {
      for x in 0..width as usize {
        if blocks.flags[y * width as usize + x] & BLOCK_BAND != 0 {
          for (nx, ny) in [
            (x.saturating_sub(1), y.saturating_sub(1)),
            (x, y),
            (
              (x + 1).min(width as usize - 1),
              (y + 1).min(height as usize - 1),
            ),
          ] {
            assert!(
              covered(nx, ny),
              "block ({nx}, {ny}) near band block ({x}, {y})"
            );
          }
        }
      }
    }
  }

  #[test]
  fn mismatched_counts_keep_nothing() {
    let blocks = PictureBlocks::new(&[[0; 4]; 3], (64, 64));

    assert!(blocks.flags.iter().all(|f| *f == 0));
    assert_eq!(blocks.tiles, Vec::<u32>::new());
  }
}
