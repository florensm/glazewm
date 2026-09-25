// Per-window color theme pass. A direct port of
// `ColorTheme::apply_neighborhood` in `src/color_theme.rs`, which is the
// unit-tested reference: keep the two in sync.

#define MAX_COLOR_OVERRIDES 16

// Mirror the constants of the same names.
#define MAX_CHROMA 0.32
#define EDGE_START 0.02
#define EDGE_FULL 0.06
#define MIX_ERROR_START 0.03
#define MIX_ERROR_FULL 0.1
#define MIN_CHANNEL_SPAN 0.02

// Mirrors `ThemeConstants`.
cbuffer Theme : register(b0) {
  float4 background;
  float4 foreground;
  float saturation_threshold;
  uint override_count;
  uint ramp_enabled;
  uint padding;
  float4 overrides[MAX_COLOR_OVERRIDES * 2];
};

// Mirrors `FrameConstants`.
cbuffer Frame : register(b1) {
  // Captured content size; the frame pool's texture can be larger.
  uint2 frame_size;
  uint2 frame_padding;
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

float3 srgb_to_oklab(float3 srgb) {
  float r = srgb_to_linear(srgb.r);
  float g = srgb_to_linear(srgb.g);
  float b = srgb_to_linear(srgb.b);

  float l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
  float m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
  float s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

  l = pow(max(l, 0.0), 1.0 / 3.0);
  m = pow(max(m, 0.0), 1.0 / 3.0);
  s = pow(max(s, 0.0), 1.0 / 3.0);

  return float3(
    0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s);
}

float3 oklab_to_srgb(float3 lab) {
  float l = lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z;
  float m = lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z;
  float s = lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z;

  l = l * l * l;
  m = m * m * m;
  s = s * s * s;

  return float3(
    linear_to_srgb(4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s),
    linear_to_srgb(-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s),
    linear_to_srgb(-0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s));
}

// Mirrors `ramp_weight`.
float ramp_weight(float saturation, float threshold) {
  return saturation >= threshold
    ? 0.0
    : 1.0 - smoothstep(threshold * 0.5, threshold, saturation);
}

// Mirrors `override_weight`.
float override_weight(float dist, float tolerance) {
  return dist >= tolerance ? 0.0 : 1.0 - smoothstep(0.0, tolerance, dist);
}

// Mirrors `ColorTheme::apply`.
float3 apply_theme(float3 srgb) {
  float3 lab = srgb_to_oklab(srgb);
  float3 result = lab;

  if (ramp_enabled != 0) {
    float saturation = min(length(lab.yz) / MAX_CHROMA, 1.0);
    float weight = ramp_weight(saturation, saturation_threshold);

    float t = saturate(lab.x);
    float3 ramped = lerp(foreground.xyz, background.xyz, t);
    ramped.yz += lab.yz;

    result = lerp(lab, ramped, weight);
  }

  float best_weight = 0.0;
  float3 best_to = float3(0.0, 0.0, 0.0);

  [loop]
  for (uint i = 0; i < override_count; i++) {
    float4 from = overrides[i * 2];
    float weight = override_weight(distance(lab, from.xyz), from.w);

    if (weight > best_weight) {
      best_weight = weight;
      best_to = overrides[i * 2 + 1].xyz;
    }
  }

  result = lerp(result, best_to, best_weight);
  return saturate(oklab_to_srgb(result));
}

// Mirrors `ColorTheme::apply_neighborhood`.
float3 apply_neighborhood(float3 pixels[9]) {
  float3 center = pixels[4];
  float3 themed_center = apply_theme(center);

  float3 dark = center;
  float3 light = center;
  float dark_lightness = srgb_to_oklab(center).x;
  float light_lightness = dark_lightness;

  [unroll]
  for (int i = 0; i < 9; i++) {
    float lightness = srgb_to_oklab(pixels[i]).x;

    if (lightness < dark_lightness) {
      dark = pixels[i];
      dark_lightness = lightness;
    }

    if (lightness > light_lightness) {
      light = pixels[i];
      light_lightness = lightness;
    }
  }

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

  float3 row_average = lerp(lerp(pixels[3], pixels[5], 0.5), center, 1.0 / 3.0);
  float3 row_lab = srgb_to_oklab(row_average);
  float neutral = ramp_weight(
    min(length(row_lab.yz) / MAX_CHROMA, 1.0),
    saturation_threshold);

  float3 own = valid > 0.0 ? coverage : mean_coverage;
  coverage = lerp(mean_coverage, own, neutral);

  float3 reconstructed = lerp(light, dark, coverage);
  float fit = 1.0 - smoothstep(
    MIX_ERROR_START,
    MIX_ERROR_FULL,
    distance(center, reconstructed));

  float3 remixed = lerp(apply_theme(light), apply_theme(dark), coverage);
  return lerp(themed_center, remixed, edge * fit);
}

// Straight-alpha color at `position`, clamped to the captured content.
// Fully transparent pixels (rounded window corners) stand in as `fallback`.
float3 load_straight(int2 position, float3 fallback) {
  int2 clamped = clamp(position, int2(0, 0), int2(frame_size) - 1);
  float4 color = source.Load(int3(clamped, 0));
  return color.a > 0.0 ? color.rgb / color.a : fallback;
}

float4 ps_main(float4 position : SV_Position) : SV_Target {
  // Captured frames are premultiplied; theme the straight color and
  // re-premultiply so rounded window corners stay transparent.
  int2 xy = int2(position.xy);
  float4 color = source.Load(int3(xy, 0));

  if (color.a <= 0.0) {
    return float4(0.0, 0.0, 0.0, 0.0);
  }

  float3 center = color.rgb / color.a;
  float3 pixels[9];

  [unroll]
  for (int y = -1; y <= 1; y++) {
    [unroll]
    for (int x = -1; x <= 1; x++) {
      pixels[(y + 1) * 3 + (x + 1)] = load_straight(xy + int2(x, y), center);
    }
  }

  float3 themed = apply_neighborhood(pixels);
  return float4(themed * color.a, color.a);
}