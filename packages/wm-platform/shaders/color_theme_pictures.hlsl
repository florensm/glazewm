// Keeping pictures in their own colors under a theme (`keep_pictures`).
// See `color_pictures.rs` for the steps; `color_pictures::reference` is
// the CPU mirror of this file.

#include "color_theme.hlsl"

// Mirror `color_pictures.rs`.
#define BLOCK_SIZE 8
#define TILE_SIZE 16
#define MAX_SURFACES 8

#define BLOCK_INTERIOR 1
#define BLOCK_NEAR 2
#define BLOCK_BAND 4

#define PIXEL_KEPT 1
#define PIXEL_CORE 2
#define PIXEL_PAGE 4

// Steps between neighboring pixels in this range (sRGB distance) are
// smooth shading. The lower end is above one step of 8-bit dithering,
// which browsers add to CSS gradients.
#define SMOOTH_STEP_MIN 0.008
#define SMOOTH_STEP_MAX 0.05
// Distance within which a pixel is of a background or dark surface.
#define SURFACE_TOLERANCE 0.02
// How far (in steps through pixels that aren't page) a picture reaches
// past its seeds.
#define GROW 16
// Pixels with no page within this distance are solid picture.
#define SOLID_RADIUS 3
// Width of the anti-aliased edge between a picture and the page.
#define RIM 2
// How far a rim pixel looks for the page and the picture it mixes.
#define RIM_SEARCH 3
// Rim pixels further than this from every background are picture, not a
// mix with the page.
#define RIM_MAX_DISTANCE 0.25
// A page and picture closer than this don't tell coverage apart.
#define MIN_RIM_SPAN 0.02

// Mirrors `PictureConstants`.
cbuffer Pictures : register(b2) {
  uint background_count;
  uint dark_count;
  // Tiles `cs_keep` runs on this frame.
  uint tile_count;
  uint pictures_padding;
  float4 backgrounds[MAX_SURFACES];
  float4 dark_surfaces[MAX_SURFACES];
};

// `BLOCK_*` flags per block.
Texture2D<uint> block_flags : register(t1);
// `PIXEL_*` flags, valid in the tiles `cs_keep` ran on this frame.
Texture2D<uint> pixel_flags : register(t2);

float background_distance(float3 color) {
  float best = 1e9;
  for (uint i = 0; i < background_count; i++) {
    best = min(best, distance(color, backgrounds[i].rgb));
  }
  return best;
}

bool is_background(float3 color) {
  return background_distance(color) < SURFACE_TOLERANCE;
}

bool is_dark_surface(float3 color) {
  for (uint i = 0; i < dark_count; i++) {
    if (distance(color, dark_surfaces[i].rgb) < SURFACE_TOLERANCE) {
      return true;
    }
  }
  return false;
}

bool is_smooth_step(float3 a, float3 b) {
  float step = distance(a, b);
  return step > SMOOTH_STEP_MIN && step < SMOOTH_STEP_MAX;
}

// Straight color at `xy`, inside the captured content.
float3 load_color(int2 xy) {
  return load_straight(xy, float3(0.0, 0.0, 0.0));
}

// Per block: smooth horizontal steps, smooth vertical steps, background
// pixels, dark-surface pixels.
uint4 block_counts(int2 block) {
  int2 size = int2(frame_size);
  int2 origin = block * BLOCK_SIZE;
  int2 end = min(origin + BLOCK_SIZE, size);
  uint4 counts = uint4(0, 0, 0, 0);

  for (int y = origin.y; y < end.y; y++) {
    for (int x = origin.x; x < end.x; x++) {
      float3 color = load_color(int2(x, y));

      if (x + 1 < size.x && is_smooth_step(color, load_color(int2(x + 1, y)))) {
        counts.x++;
      }
      if (y + 1 < size.y && is_smooth_step(color, load_color(int2(x, y + 1)))) {
        counts.y++;
      }
      if (is_background(color)) {
        counts.z++;
      }
      if (is_dark_surface(color)) {
        counts.w++;
      }
    }
  }

  return counts;
}

// Drawn into a target of one texel per block.
uint4 ps_blocks(float4 position : SV_Position) : SV_Target {
  return block_counts(int2(position.xy));
}

// A kept pixel at a picture's edge is a mix of page and picture: it's
// re-mixed from the themed page and the picture's own color next to it,
// at the same coverage. Returns `w` 0 without both nearby.
float4 remix_rim(int2 xy, float3 color) {
  int2 size = int2(frame_size);
  bool has_page = false;
  bool has_picture = false;
  int2 page_xy = int2(0, 0);
  int2 picture_xy = int2(0, 0);
  int best = 0x7fffffff;

  for (int dy = -RIM_SEARCH; dy <= RIM_SEARCH; dy++) {
    for (int dx = -RIM_SEARCH; dx <= RIM_SEARCH; dx++) {
      int2 neighbor = xy + int2(dx, dy);
      if (any(neighbor < 0) || any(neighbor >= size)) {
        continue;
      }

      uint flags = pixel_flags.Load(int3(neighbor, 0));
      if ((flags & PIXEL_PAGE) != 0) {
        if (!has_page) {
          has_page = true;
          page_xy = neighbor;
        }
      } else if ((flags & PIXEL_CORE) != 0 && dx * dx + dy * dy < best) {
        best = dx * dx + dy * dy;
        picture_xy = neighbor;
        has_picture = true;
      }
    }
  }

  if (!has_page || !has_picture) {
    return float4(0.0, 0.0, 0.0, 0.0);
  }

  float3 page = load_color(page_xy);
  float3 picture = load_color(picture_xy);
  float span = distance(page, picture);
  float coverage = span > MIN_RIM_SPAN ? saturate(distance(color, page) / span) : 1.0;
  float3 themed_page = themed_at(page_xy, page);

  return float4(lerp(themed_page, picture, coverage), 1.0);
}

// `ps_main`, leaving pictures unchanged.
float4 ps_pictures(float4 position : SV_Position) : SV_Target {
  int2 xy = int2(position.xy);
  float4 color = source.Load(int3(xy, 0));

  if (color.a <= 0.0) {
    return float4(0.0, 0.0, 0.0, 0.0);
  }

  if (passthrough != 0) {
    return color;
  }

  uint block = block_flags.Load(int3(uint2(xy) / BLOCK_SIZE, 0));
  if ((block & BLOCK_INTERIOR) != 0) {
    return color;
  }

  float3 center = color.rgb / color.a;

  if ((block & BLOCK_BAND) != 0) {
    uint flags = pixel_flags.Load(int3(xy, 0));

    if ((flags & PIXEL_CORE) != 0) {
      return color;
    }

    if ((flags & PIXEL_KEPT) != 0) {
      float4 remixed = remix_rim(xy, center);
      if (remixed.w > 0.0) {
        return float4(remixed.rgb * color.a, color.a);
      }
    }
  }

  float3 themed = themed_at(xy, center);
  return float4(themed * color.a, color.a);
}
