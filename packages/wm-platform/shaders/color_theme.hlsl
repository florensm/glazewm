// Per-window color theme pass. A direct port of
// `ColorFilter::apply_neighborhood` in `src/color_theme.rs`, which is the
// unit-tested reference: keep the two in sync.

// Mirror the constants of the same names.
#define MAX_COLOR_OVERRIDES 16
#define MAX_RAMP_STOPS 8
#define MAX_PALETTE_COLORS 16
#define MAX_FILTER_SLOTS 4
#define SLOT_ORIGINAL 0xFFFFFFFF
#define MAX_REGIONS 64
#define ANALYSIS_SIZE 64

#define MAX_CHROMA 0.32
#define EDGE_START 0.02
#define EDGE_FULL 0.06
#define MIX_ERROR_START 0.03
#define MIX_ERROR_FULL 0.1
#define MIN_CHANNEL_SPAN 0.02
#define SUBPIXEL_STEP_START 0.55
#define SUBPIXEL_STEP_FULL 0.75
#define INVERTED_TEXT_GAMMA 1.4
#define NEIGHBORHOOD_SIZE 15
#define FLAT_RANGE 0.01
#define HUE_MIN_CHROMA 0.03
#define HUE_AGREEMENT_START 0.6
#define HUE_AGREEMENT_FULL 0.85
#define INVERSION_FULL 0.1
#define PALETTE_CHROMA_START 0.02
#define PALETTE_CHROMA_FULL 0.06
#define PALETTE_SHARPNESS 10.7
#define MIN_LEVELS_SPAN 0.05
#define GAMUT_STEPS 8
#define TINT_LIGHTNESS_START 0.8
#define TINT_LIGHTNESS_FULL 0.9
#define TINT_THRESHOLD_SCALE 2.5
#define TINT_THRESHOLD_MAX 0.4
#define PAPER_FRINGE_AGREEMENT_START 0.3
#define PAPER_FRINGE_AGREEMENT_FULL 0.5
#define PAPER_ENVELOPE_START 0.2
#define PAPER_ENVELOPE_FULL 0.35
#define SOLID_INK_START 0.75
#define SOLID_INK_FULL 0.9
#define INK_BALANCE_COLORED 0.38
#define INK_BALANCE_NEUTRAL 0.5
#define INK_OVERSHOOT_FULL 0.1
#define KNOWN_INK_MIN_CONTRAST 0.1
#define KNOWN_INK_PAGE_REACH 0.5
#define KNOWN_INK_FIT_START 0.03
#define KNOWN_INK_FIT_FULL 0.08
#define KNOWN_INK_EVIDENCE_START 0.3
#define KNOWN_INK_EVIDENCE_FULL 0.6

// Mirrors `FilterConstants`.
struct Filter {
  uint4 counts;
  float4 tone;
  float4 hue;
  float4 lightness;
  float4 warmth;
  float4 ramp[MAX_RAMP_STOPS];
  float4 palette[MAX_PALETTE_COLORS];
  float4 overrides[MAX_COLOR_OVERRIDES * 2];
  float4 override_inks[MAX_COLOR_OVERRIDES];
};

// Mirrors `FilterSlots`: the theme's own filter, then its element filters.
cbuffer Filters : register(b0) {
  Filter filters[MAX_FILTER_SLOTS];
};

// Mirrors `FrameConstants`.
cbuffer Frame : register(b1) {
  // Captured content size; the frame pool's texture can be larger.
  uint2 frame_size;
  // Non-zero to show the captured pixels unchanged.
  uint passthrough;
  uint region_count;
  // Source paper and ink lightness in `x` and `y`.
  float4 levels;
};

// Mirrors `RegionConstants`: UI element rects in frame pixels (right and
// bottom exclusive), and the filter slot of each in `x`, sorted so later
// (smaller) regions win.
cbuffer Regions : register(b2) {
  int4 region_rects[MAX_REGIONS];
  uint4 region_slots[MAX_REGIONS];
};

Texture2D<float4> source : register(t0);

// Full-screen triangle; no vertex buffer.
float4 vs_main(uint id : SV_VertexID) : SV_Position {
  float2 uv = float2((id << 1) & 2, id & 2);
  return float4(uv * float2(2.0, -2.0) + float2(-1.0, 1.0), 0.0, 1.0);
}

float srgb_to_linear(float c) {
  return c <= 0.04045 ? c / 12.92 : pow(max((c + 0.055) / 1.055, 0.0), 2.4);
}

float linear_to_srgb(float c) {
  return c <= 0.0031308 ? c * 12.92 : 1.055 * pow(max(c, 0.0), 1.0 / 2.4) - 0.055;
}

float3 linear_to_oklab(float3 rgb) {
  float l = 0.4122214708 * rgb.r + 0.5363325363 * rgb.g + 0.0514459929 * rgb.b;
  float m = 0.2119034982 * rgb.r + 0.6806995451 * rgb.g + 0.1073969566 * rgb.b;
  float s = 0.0883024619 * rgb.r + 0.2817188376 * rgb.g + 0.6299787005 * rgb.b;

  l = pow(max(l, 0.0), 1.0 / 3.0);
  m = pow(max(m, 0.0), 1.0 / 3.0);
  s = pow(max(s, 0.0), 1.0 / 3.0);

  return float3(
    0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s);
}

float3 oklab_to_linear(float3 lab) {
  float l = lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z;
  float m = lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z;
  float s = lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z;

  l = l * l * l;
  m = m * m * m;
  s = s * s * s;

  return float3(
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s);
}

float3 srgb_to_oklab(float3 srgb) {
  return linear_to_oklab(float3(
    srgb_to_linear(srgb.r),
    srgb_to_linear(srgb.g),
    srgb_to_linear(srgb.b)));
}

float3 oklab_to_srgb(float3 lab) {
  float3 rgb = oklab_to_linear(lab);
  return float3(
    linear_to_srgb(rgb.r),
    linear_to_srgb(rgb.g),
    linear_to_srgb(rgb.b));
}

// Mirrors `ramp_weight`.
float ramp_weight(float saturation, float threshold) {
  return saturation >= threshold
    ? 0.0
    : 1.0 - smoothstep(threshold * 0.5, threshold, saturation);
}

// Mirrors `tint_threshold`.
float tint_threshold(float threshold, float lightness) {
  float tint = max(
    threshold, min(threshold * TINT_THRESHOLD_SCALE, TINT_THRESHOLD_MAX));
  return lerp(
    threshold,
    tint,
    smoothstep(TINT_LIGHTNESS_START, TINT_LIGHTNESS_FULL, lightness));
}

// Mirrors `override_weight`.
float override_weight(float dist, float tolerance) {
  return dist >= tolerance ? 0.0 : 1.0 - smoothstep(0.0, tolerance, dist);
}

// Mirrors `in_gamut`.
bool in_gamut(float3 rgb) {
  return all(rgb >= -1e-4) && all(rgb <= 1.0 + 1e-4);
}

// Mirrors `gamut_clip`.
float3 gamut_clip(float3 lab) {
  lab.x = saturate(lab.x);

  if (in_gamut(oklab_to_linear(lab))) {
    return lab;
  }

  float low = 0.0;
  float high = 1.0;

  [loop]
  for (int i = 0; i < GAMUT_STEPS; i++) {
    float mid = (low + high) * 0.5;

    if (in_gamut(oklab_to_linear(float3(lab.x, lab.yz * mid)))) {
      low = mid;
    } else {
      high = mid;
    }
  }

  return float3(lab.x, lab.yz * low);
}

// Mirrors `SourceLevels::normalize`.
float normalize_lightness(float lightness) {
  float span = levels.x - levels.y;

  if (abs(span) < MIN_LEVELS_SPAN) {
    return saturate(lightness);
  }

  return saturate((lightness - levels.y) / span);
}

// Mirrors `FilterConstants::ramp_at`.
float3 ramp_at(uint slot, float t) {
  float3 result = filters[slot].ramp[0].xyz;

  [loop]
  for (uint i = 1; i < filters[slot].counts.x; i++) {
    float4 low = filters[slot].ramp[i - 1];
    float4 high = filters[slot].ramp[i];

    if (t > low.w) {
      float progress = min((t - low.w) / (high.w - low.w), 1.0);
      result = lerp(low.xyz, high.xyz, progress);
    }
  }

  return result;
}

// Mirrors `FilterConstants::snap_to_palette`.
float3 snap_to_palette(uint slot, float3 lab) {
  float chroma = length(lab.yz);

  if (chroma < 1e-4) {
    return lab;
  }

  float2 direction = lab.yz / chroma;
  float weight_sum = 0.0;
  float2 direction_sum = float2(0.0, 0.0);
  float chroma_sum = 0.0;
  float lightness_sum = 0.0;

  [loop]
  for (uint i = 0; i < filters[slot].counts.y; i++) {
    float4 entry = filters[slot].palette[i];
    float2 entry_direction = entry.yz / entry.w;
    float weight = exp((dot(direction, entry_direction) - 1.0) * PALETTE_SHARPNESS);

    weight_sum += weight;
    direction_sum += entry_direction * weight;
    chroma_sum += entry.w * weight;
    lightness_sum += entry.x * weight;
  }

  float sum_length = length(direction_sum);
  float2 target_direction =
    sum_length > 1e-4 ? direction_sum / sum_length : direction;
  float target_chroma = chroma_sum / weight_sum;

  float amount = filters[slot].hue.z
    * smoothstep(PALETTE_CHROMA_START, PALETTE_CHROMA_FULL, chroma);

  return float3(
    lerp(lab.x, lightness_sum / weight_sum, amount * filters[slot].hue.w),
    lerp(lab.yz, target_direction * target_chroma, amount));
}

// Mirrors `FilterConstants::tone_map`.
float3 tone_map(uint slot, float3 lab, float t) {
  float3 result = lab;

  if (filters[slot].counts.x > 0) {
    float saturation = min(length(lab.yz) / MAX_CHROMA, 1.0);
    float threshold = tint_threshold(filters[slot].tone.x, lab.x);
    float weight = ramp_weight(saturation, threshold);

    float3 stop = ramp_at(slot, t);
    float3 ramped = float3(stop.x, stop.yz + lab.yz);
    float3 accent = float3(lerp(lab.x, stop.x, filters[slot].tone.y), lab.yz);
    result = lerp(accent, ramped, weight);
  }

  float saturation = min(length(result.yz) / MAX_CHROMA, 1.0);
  float scale = filters[slot].tone.z
    * (1.0 + filters[slot].tone.w * (1.0 - saturation) * (1.0 - saturation));
  float2 ab = result.yz * scale;
  float2 rotation = filters[slot].hue.xy;
  result.yz = float2(
    ab.x * rotation.x - ab.y * rotation.y,
    ab.x * rotation.y + ab.y * rotation.x);

  if (filters[slot].counts.y > 0) {
    result = snap_to_palette(slot, result);
  }

  float4 lightness = filters[slot].lightness;
  result.x = ((result.x - 0.5) * lightness.y + 0.5) * lightness.x;
  return result;
}

// Mirrors `FilterConstants::keep_contrast`.
float3 keep_contrast(uint slot, float3 lab, float source_contrast) {
  float target = min(filters[slot].lightness.z, source_contrast);
  float paper = filters[slot].lightness.w;
  float difference = lab.x - paper;

  if (target <= 0.0 || abs(difference) >= target) {
    return lab;
  }

  float side = abs(difference) > 1e-4
    ? sign(difference)
    : (paper < 0.5 ? 1.0 : -1.0);
  float moved = paper + side * target;

  if (moved < 0.0 || moved > 1.0) {
    side = -side;
  }

  return float3(saturate(paper + side * target), lab.yz);
}

// Mirrors `FilterConstants::warm`.
float3 warm(uint slot, float3 lab) {
  float4 gains = filters[slot].warmth;

  if (gains.w != 0.0) {
    lab = linear_to_oklab(oklab_to_linear(lab) * gains.xyz);
  }

  return gamut_clip(lab);
}

// Mirrors `ColorFilter::apply`.
float3 apply_theme(uint slot, float3 srgb) {
  float3 lab = srgb_to_oklab(srgb);
  float3 result = tone_map(slot, lab, normalize_lightness(lab.x));

  result = keep_contrast(slot, result, abs(lab.x - levels.x));
  result = warm(slot, result);

  float best_weight = 0.0;
  float3 best_to = float3(0.0, 0.0, 0.0);

  [loop]
  for (uint i = 0; i < filters[slot].counts.z; i++) {
    float4 from = filters[slot].overrides[i * 2];
    float weight = override_weight(distance(lab, from.xyz), from.w);

    if (weight > best_weight) {
      best_weight = weight;
      best_to = filters[slot].overrides[i * 2 + 1].xyz;
    }
  }

  result = lerp(result, best_to, best_weight);
  return saturate(oklab_to_srgb(result));
}

// Mirrors `hue_agreement`.
float hue_agreement(float3 pixels[NEIGHBORHOOD_SIZE], float3 paper, float extreme) {
  float paper_mean = dot(paper, 1.0 / 3.0);
  float span = extreme - paper_mean;

  float2 hue_sum = float2(0.0, 0.0);
  float chroma_sum = 0.0;

  [unroll]
  for (int i = 0; i < NEIGHBORHOOD_SIZE; i++) {
    float mean = dot(pixels[i], 1.0 / 3.0);
    float t = abs(span) > MIN_CHANNEL_SPAN
      ? saturate((mean - paper_mean) / span)
      : 0.0;

    float2 deviation = srgb_to_oklab(pixels[i]).yz
      - srgb_to_oklab(lerp(paper, extreme, t)).yz;
    float chroma = length(deviation);

    if (chroma > HUE_MIN_CHROMA) {
      hue_sum += deviation;
      chroma_sum += chroma;
    }
  }

  return chroma_sum > 0.0 ? length(hue_sum) / chroma_sum : 1.0;
}

// Mirrors `channel_extreme`.
float channel_extreme(float3 color, float extreme) {
  return extreme > 0.5
    ? max(max(color.r, color.g), color.b)
    : min(min(color.r, color.g), color.b);
}

// Mirrors `neutral`.
float3 neutral(float3 srgb) {
  return oklab_to_srgb(float3(srgb_to_oklab(srgb).x, 0.0, 0.0));
}

// Mirrors `estimate_ink`.
float3 estimate_ink(
  float3 pixels[NEIGHBORHOOD_SIZE],
  float3 endpoint,
  float3 paper,
  float extreme,
  float mixed_hues) {
  float3 span = paper - extreme;

  if (any(abs(span) <= MIN_CHANNEL_SPAN)) {
    return endpoint;
  }

  float3 coverage = saturate((paper - endpoint) / span);

  float3 deviation_sum = float3(0.0, 0.0, 0.0);

  [unroll]
  for (int i = 0; i < NEIGHBORHOOD_SIZE; i++) {
    deviation_sum += pixels[i] - paper;
  }

  float3 valid = (float3)(abs(deviation_sum) > MIN_CHANNEL_SPAN);
  float full_scale = 0.0;

  [unroll]
  for (int j = 0; j < NEIGHBORHOOD_SIZE; j++) {
    float3 ratio =
      valid * (pixels[j] - paper) / (valid > 0.0 ? deviation_sum : 1.0);
    full_scale = max(full_scale, max(ratio.r, max(ratio.g, ratio.b)));
  }

  float3 estimated = paper + deviation_sum * full_scale;

  float length_sq = dot(estimated - paper, estimated - paper);
  float reach = length_sq > 0.0
    ? dot(endpoint - paper, estimated - paper) / length_sq
    : 1.0;
  float3 colored = lerp(
    estimated,
    endpoint,
    smoothstep(SOLID_INK_START, SOLID_INK_FULL, reach));

  float channel_step = max(
    abs(coverage.r - coverage.g),
    abs(coverage.g - coverage.b));
  float fringe =
    1.0 - smoothstep(SUBPIXEL_STEP_START, SUBPIXEL_STEP_FULL, channel_step);
  float3 neutral_ink =
    lerp(endpoint, channel_extreme(endpoint, extreme), fringe);

  float3 ratio = deviation_sum / (extreme - paper);
  float ratio_max = max(max(ratio.r, ratio.g), ratio.b);
  float balance = ratio_max > 0.0
    ? min(min(ratio.r, ratio.g), ratio.b) / ratio_max
    : 1.0;
  float3 outside = max(-estimated, estimated - 1.0);
  float overshoot = max(max(max(outside.r, outside.g), outside.b), 0.0);
  float colored_ink =
    (1.0 - smoothstep(INK_BALANCE_COLORED, INK_BALANCE_NEUTRAL, balance))
    * (1.0 - smoothstep(0.0, INK_OVERSHOOT_FULL, overshoot));

  return saturate(lerp(colored, neutral_ink, mixed_hues * (1.0 - colored_ink)));
}

// Mirrors `edge_colors`.
void edge_colors(
  float3 pixels[NEIGHBORHOOD_SIZE],
  float paper_fringes,
  out float3 dark,
  out float3 light) {
  float3 center = pixels[NEIGHBORHOOD_SIZE / 2];
  dark = center;
  light = center;
  float dark_lightness = srgb_to_oklab(center).x;
  float light_lightness = dark_lightness;

  float lightness_sum = 0.0;

  [unroll]
  for (int i = 0; i < NEIGHBORHOOD_SIZE; i++) {
    float lightness = srgb_to_oklab(pixels[i]).x;
    lightness_sum += lightness;

    if (lightness < dark_lightness) {
      dark = pixels[i];
      dark_lightness = lightness;
    }

    if (lightness > light_lightness) {
      light = pixels[i];
      light_lightness = lightness;
    }
  }

  float mean_lightness = lightness_sum / NEIGHBORHOOD_SIZE;
  bool paper_is_light =
    (light_lightness - mean_lightness) < (mean_lightness - dark_lightness);
  float extreme = paper_is_light ? 0.0 : 1.0;
  float3 paper = paper_is_light ? light : dark;
  float3 envelope = paper;

  [unroll]
  for (int j = 0; j < NEIGHBORHOOD_SIZE; j++) {
    envelope = paper_is_light
      ? max(envelope, pixels[j])
      : min(envelope, pixels[j]);
  }

  float3 envelope_gap = abs(envelope - paper);
  paper = lerp(
    envelope,
    paper,
    smoothstep(
      PAPER_ENVELOPE_START,
      PAPER_ENVELOPE_FULL,
      max(max(envelope_gap.r, envelope_gap.g), envelope_gap.b)));

  paper = lerp(paper, channel_extreme(paper, 1.0 - extreme), paper_fringes);

  float mixed_hues = 1.0 - smoothstep(
    HUE_AGREEMENT_START,
    HUE_AGREEMENT_FULL,
    hue_agreement(pixels, paper, extreme));

  if (paper_is_light) {
    dark = estimate_ink(pixels, dark, paper, 0.0, mixed_hues);
    light = paper;
  } else {
    dark = paper;
    light = estimate_ink(pixels, light, paper, 1.0, mixed_hues);
  }
}

// Mirrors `known_ink_coverage`.
float known_ink_coverage(float3 pixel, float3 ink, float3 paper) {
  float3 span = paper - ink;
  float3 valid = (float3)(abs(span) > MIN_CHANNEL_SPAN);
  float count = dot(valid, 1.0);
  float3 coverage =
    saturate((paper - pixel) / (valid > 0.0 ? span : 1.0));

  return count > 0.0 ? dot(coverage * valid, 1.0) / count : 0.0;
}

// Mirrors `ColorFilter::known_ink_remix`; the weight is in `w`.
float4 known_ink_remix(uint slot, float3 pixels[NEIGHBORHOOD_SIZE]) {
  float3 center = pixels[NEIGHBORHOOD_SIZE / 2];
  float4 best = float4(0.0, 0.0, 0.0, 0.0);

  [loop]
  for (uint i = 0; i < filters[slot].counts.z; i++) {
    float3 ink = filters[slot].override_inks[i].xyz;

    [unroll]
    for (int polarity = 0; polarity < 2; polarity++) {
      bool page_is_light = polarity == 0;
      float3 paper = ink;

      [unroll]
      for (int j = 0; j < NEIGHBORHOOD_SIZE; j++) {
        paper = page_is_light ? max(paper, pixels[j]) : min(paper, pixels[j]);
      }

      bool3 reaches = page_is_light
        ? paper >= lerp(ink, 1.0, KNOWN_INK_PAGE_REACH)
        : paper <= lerp(ink, 0.0, KNOWN_INK_PAGE_REACH);

      if (!all(reaches) || distance(paper, ink) < KNOWN_INK_MIN_CONTRAST) {
        continue;
      }

      float violation = 0.0;
      float most_ink = 0.0;

      [unroll]
      for (int k = 0; k < NEIGHBORHOOD_SIZE; k++) {
        float3 direction = sign(ink - paper);
        float3 past = direction != 0.0
          ? (pixels[k] - ink) * direction
          : abs(pixels[k] - ink);
        violation = max(violation, max(max(past.r, past.g), past.b));
        most_ink = max(most_ink, known_ink_coverage(pixels[k], ink, paper));
      }

      float weight =
        (1.0 - smoothstep(KNOWN_INK_FIT_START, KNOWN_INK_FIT_FULL, violation))
        * smoothstep(KNOWN_INK_EVIDENCE_START, KNOWN_INK_EVIDENCE_FULL, most_ink);

      if (weight > best.w) {
        float3 themed_ink = apply_theme(slot, ink);
        float3 themed_paper = apply_theme(slot, paper);
        float coverage = known_ink_coverage(center, ink, paper);

        float inverted = smoothstep(
          0.0,
          INVERSION_FULL,
          srgb_to_oklab(themed_ink).x - srgb_to_oklab(themed_paper).x);
        coverage = lerp(
          coverage,
          pow(saturate(coverage), 1.0 / INVERTED_TEXT_GAMMA),
          inverted);

        best = float4(lerp(themed_paper, themed_ink, coverage), weight);
      }
    }
  }

  return best;
}

// Mirrors `ColorFilter::apply_neighborhood`.
float3 apply_neighborhood(uint slot, float3 pixels[NEIGHBORHOOD_SIZE]) {
  float3 center = pixels[NEIGHBORHOOD_SIZE / 2];
  float3 themed_center = apply_theme(slot, center);

  float range = 0.0;
  [unroll]
  for (int f = 0; f < NEIGHBORHOOD_SIZE; f++) {
    float3 difference = abs(pixels[f] - center);
    range = max(range, max(difference.r, max(difference.g, difference.b)));
  }

  if (range < FLAT_RANGE) {
    return themed_center;
  }

  float neutral_agreement = hue_agreement(pixels, float3(1.0, 1.0, 1.0), 0.0);
  float fringes = 1.0 - smoothstep(
    HUE_AGREEMENT_START,
    HUE_AGREEMENT_FULL,
    neutral_agreement);

  float3 dark;
  float3 light;
  edge_colors(
    pixels,
    1.0 - smoothstep(
      PAPER_FRINGE_AGREEMENT_START,
      PAPER_FRINGE_AGREEMENT_FULL,
      neutral_agreement),
    dark,
    light);

  float edge = smoothstep(
    EDGE_START,
    EDGE_FULL,
    distance(srgb_to_oklab(dark), srgb_to_oklab(light)));

  if (edge <= 0.0) {
    return themed_center;
  }

  float3 span = dark - light;
  float3 valid = step(MIN_CHANNEL_SPAN, abs(span));
  float3 coverage =
    valid * saturate((center - light) / (valid > 0.0 ? span : 1.0));

  float valid_count = dot(valid, 1.0);
  float mean_coverage =
    valid_count > 0.0 ? dot(coverage, 1.0) / valid_count : 0.0;

  coverage = valid > 0.0 ? coverage : mean_coverage;

  float channel_step = max(
    abs(coverage.r - coverage.g),
    abs(coverage.g - coverage.b));
  float subpixel =
    1.0 - smoothstep(SUBPIXEL_STEP_START, SUBPIXEL_STEP_FULL, channel_step);

  float3 reconstructed =
    lerp(light, dark, lerp(mean_coverage, coverage, subpixel));
  float fit = 1.0 - smoothstep(
    MIX_ERROR_START,
    MIX_ERROR_FULL,
    distance(center, reconstructed));

  float3 themed_dark = apply_theme(slot, dark);
  float3 themed_light = apply_theme(slot, light);

  float inverted = smoothstep(
    0.0,
    INVERSION_FULL,
    srgb_to_oklab(themed_dark).x - srgb_to_oklab(themed_light).x);
  float remix_coverage = lerp(
    mean_coverage,
    pow(mean_coverage, 1.0 / INVERTED_TEXT_GAMMA),
    inverted);

  float3 remixed = lerp(themed_light, themed_dark, remix_coverage);
  float3 unexplained =
    lerp(themed_center, apply_theme(slot, neutral(center)), edge * fringes);
  float3 estimated = lerp(unexplained, remixed, edge * fit);

  float4 known = known_ink_remix(slot, pixels);
  return lerp(estimated, known.rgb, known.w);
}

// Straight-alpha color at `position`, clamped to the captured content.
// Fully transparent pixels (rounded window corners) stand in as `fallback`.
float3 load_straight(int2 position, float3 fallback) {
  int2 clamped = clamp(position, int2(0, 0), int2(frame_size) - 1);
  float4 color = source.Load(int3(clamped, 0));
  return color.a > 0.0 ? color.rgb / color.a : fallback;
}

// Filter slot of the UI element at `position`, or 0 outside all of them.
uint slot_at(int2 position) {
  uint slot = 0;

  [loop]
  for (uint i = 0; i < region_count; i++) {
    int4 rect = region_rects[i];

    if (all(position >= rect.xy) && all(position < rect.zw)) {
      slot = region_slots[i].x;
    }
  }

  return slot;
}

float4 ps_main(float4 position : SV_Position) : SV_Target {
  // Captured frames are premultiplied; theme the straight color and
  // re-premultiply so rounded window corners stay transparent.
  int2 xy = int2(position.xy);
  float4 color = source.Load(int3(xy, 0));

  if (color.a <= 0.0) {
    return float4(0.0, 0.0, 0.0, 0.0);
  }

  uint slot = passthrough != 0 ? SLOT_ORIGINAL : slot_at(xy);

  if (slot == SLOT_ORIGINAL) {
    return color;
  }

  float3 center = color.rgb / color.a;
  float3 pixels[NEIGHBORHOOD_SIZE];

  [unroll]
  for (int y = -1; y <= 1; y++) {
    [unroll]
    for (int x = -2; x <= 2; x++) {
      pixels[(y + 1) * 5 + (x + 2)] = load_straight(xy + int2(x, y), center);
    }
  }

  float3 themed = apply_neighborhood(slot, pixels);
  return float4(themed * color.a, color.a);
}

// Point-samples the captured content into an `ANALYSIS_SIZE` square, as
// straight-alpha color, for measuring the window's own colors.
float4 ps_sample(float4 position : SV_Position) : SV_Target {
  int2 xy = int2(position.xy * float2(frame_size) / ANALYSIS_SIZE);
  float4 color = source.Load(int3(min(xy, int2(frame_size) - 1), 0));
  return color.a > 0.0 ? float4(color.rgb / color.a, color.a) : color;
}
