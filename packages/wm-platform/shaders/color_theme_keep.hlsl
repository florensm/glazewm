// Finding the exact pixels of pictures near picture blocks, for
// `keep_pictures`; a compute shader, so feature level 11.0 and up.

#include "color_theme_pictures.hlsl"

// Most thread groups per dispatch dimension; more tiles than this are
// dispatched in rows of it. Mirrors `MAX_DISPATCH_GROUPS`.
#define MAX_DISPATCH_GROUPS 65535

// Tiles to run on, as `x | y << 16` in tiles.
Buffer<uint> tiles : register(t3);
RWTexture2D<uint> pixel_flags_out : register(u0);

// `cs_keep` works on a tile plus the pixels whose seeds can reach it
// (`GROW`), plus those deciding whether they are solid (`SOLID_RADIUS`).
#define KEEP_CELLS (TILE_SIZE + 2 * GROW)
#define KEEP_APRON (GROW + SOLID_RADIUS)
#define KEEP_WINDOW (TILE_SIZE + 2 * KEEP_APRON)
#define KEEP_WORDS ((KEEP_CELLS * KEEP_CELLS + 31) / 32)
#define KEEP_THREADS (TILE_SIZE * TILE_SIZE)

#define WINDOW_OTHER 0
#define WINDOW_PAGE 1
#define WINDOW_OUTSIDE 2

groupshared uint window_kinds[KEEP_WINDOW * KEEP_WINDOW];
// Whether a cell can be grown into: in the frame and not page.
groupshared uint cell_passable[KEEP_CELLS * KEEP_CELLS];
// Kept cells, one bit each; two generations, so each step grows by
// exactly one pixel.
groupshared uint kept_bits[2 * KEEP_WORDS];

bool page_within(int2 window_xy, int radius) {
  for (int dy = -radius; dy <= radius; dy++) {
    for (int dx = -radius; dx <= radius; dx++) {
      int2 xy = window_xy + int2(dx, dy);
      if (window_kinds[xy.y * KEEP_WINDOW + xy.x] == WINDOW_PAGE) {
        return true;
      }
    }
  }
  return false;
}

bool is_kept(uint generation, uint cell) {
  return (kept_bits[generation * KEEP_WORDS + cell / 32] & (1u << (cell % 32))) != 0;
}

void set_kept(uint generation, uint cell) {
  InterlockedOr(kept_bits[generation * KEEP_WORDS + cell / 32], 1u << (cell % 32));
}

// Finds the picture pixels of one tile: seeds (pixels of interior blocks,
// and solid pixels near picture blocks), grown by up to `GROW` steps
// through pixels that aren't page, so text beside a picture, separated
// from it by page, isn't reached. Writes `PIXEL_*` flags.
//
// Exact within the tile: a path of at most `GROW` steps to a tile pixel
// stays within the cells, and a cell's error from missing neighbors
// beyond the cells moves inwards one step per generation.
[numthreads(TILE_SIZE, TILE_SIZE, 1)]
void cs_keep(uint3 group : SV_GroupID, uint3 local : SV_GroupThreadID, uint thread : SV_GroupIndex) {
  uint tile_index = group.y * MAX_DISPATCH_GROUPS + group.x;
  if (tile_index >= tile_count) {
    return;
  }

  uint tile = tiles[tile_index];
  int2 tile_origin = int2(tile & 0xffff, tile >> 16) * TILE_SIZE;
  int2 window_origin = tile_origin - KEEP_APRON;
  int2 size = int2(frame_size);
  uint i;

  for (i = thread; i < KEEP_WINDOW * KEEP_WINDOW; i += KEEP_THREADS) {
    int2 xy = window_origin + int2(i % KEEP_WINDOW, i / KEEP_WINDOW);
    uint kind = WINDOW_OUTSIDE;

    if (all(xy >= 0) && all(xy < size)) {
      kind = is_background(load_color(xy)) ? WINDOW_PAGE : WINDOW_OTHER;
    }

    window_kinds[i] = kind;
  }

  for (i = thread; i < 2 * KEEP_WORDS; i += KEEP_THREADS) {
    kept_bits[i] = 0;
  }

  GroupMemoryBarrierWithGroupSync();

  for (i = thread; i < KEEP_CELLS * KEEP_CELLS; i += KEEP_THREADS) {
    int2 window_xy = int2(i % KEEP_CELLS, i / KEEP_CELLS) + SOLID_RADIUS;
    uint kind = window_kinds[window_xy.y * KEEP_WINDOW + window_xy.x];
    cell_passable[i] = kind == WINDOW_OTHER ? 1 : 0;

    if (kind != WINDOW_OUTSIDE) {
      uint flags = block_flags.Load(int3(uint2(window_origin + window_xy) / BLOCK_SIZE, 0));
      bool is_seed = (flags & BLOCK_INTERIOR) != 0
        || ((flags & BLOCK_NEAR) != 0 && kind == WINDOW_OTHER && !page_within(window_xy, SOLID_RADIUS));

      if (is_seed) {
        set_kept(0, i);
      }
    }
  }

  GroupMemoryBarrierWithGroupSync();

  for (uint step = 0; step < GROW; step++) {
    uint from = step % 2;
    uint to = 1 - from;

    for (i = thread; i < KEEP_CELLS * KEEP_CELLS; i += KEEP_THREADS) {
      bool kept = is_kept(from, i);

      if (!kept && cell_passable[i] != 0) {
        int2 cell = int2(i % KEEP_CELLS, i / KEEP_CELLS);

        for (int dy = -1; dy <= 1; dy++) {
          for (int dx = -1; dx <= 1; dx++) {
            // Nested: `&&` doesn't short-circuit in HLSL, and the index
            // must stay inside the array.
            int2 neighbor = cell + int2(dx, dy);
            if (all(neighbor >= 0) && all(neighbor < KEEP_CELLS)) {
              if (is_kept(from, neighbor.y * KEEP_CELLS + neighbor.x)) {
                kept = true;
              }
            }
          }
        }
      }

      if (kept) {
        set_kept(to, i);
      }
    }

    GroupMemoryBarrierWithGroupSync();

    // Cleared to become the next generation.
    for (i = thread; i < KEEP_WORDS; i += KEEP_THREADS) {
      kept_bits[from * KEEP_WORDS + i] = 0;
    }

    GroupMemoryBarrierWithGroupSync();
  }

  int2 xy = tile_origin + int2(local.xy);
  if (any(xy >= size)) {
    return;
  }

  int2 window_xy = int2(local.xy) + KEEP_APRON;
  uint cell = (local.y + GROW) * KEEP_CELLS + local.x + GROW;
  uint flags = 0;

  if (window_kinds[window_xy.y * KEEP_WINDOW + window_xy.x] == WINDOW_PAGE) {
    flags |= PIXEL_PAGE;
  }

  if (is_kept(GROW % 2, cell)) {
    flags |= PIXEL_KEPT;

    if (!page_within(window_xy, RIM) || background_distance(load_color(xy)) > RIM_MAX_DISTANCE) {
      flags |= PIXEL_CORE;
    }
  }

  pixel_flags_out[xy] = flags;
}
