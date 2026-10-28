# Workspace-switch animation: scene vs surrogate benchmark

Measured 2026-10-28 with this `perf_bench` example (`--scenario switch --dwm`),
switching between two workspaces of 3 Explorer windows, bursts of 10.

## Test machine (matters a lot)

- Single 3440×1440 @ **174 Hz** → frame budget **5.75 ms**.
- GPU **RTX 3070**. It idles at **210 MHz** and brief (<10 ms) switch work does
  not sustain load long enough to boost, so unlocked numbers are dominated by
  clock-scaling noise. All results below are with the **graphics clock locked**
  (`nvidia-smi --lock-gpu-clocks`) for comparability. (Only the graphics clock
  was locked; a little residual memory-clock drift remains between cells.)
- `1500 MHz` ≈ "GPU boosted"; `450 MHz` ≈ the clock the GPU actually runs at
  mid-switch / a weak low-end GPU.
- DWM frame timing comes from the PresentMon service API, which needs the
  process to be elevated.

## Scene build (`tryout/scene-spike`), per GPU clock

| clock | busy p50 | busy p90 | display interval p90 | missed vblanks/burst | on-time |
|------:|---------:|---------:|---------------------:|---------------------:|:-------:|
| 1500 MHz | 2.2 ms | 2.3 ms | 5.7 ms (1 vblank) | 4–5 | ~94% |
| 450 MHz  | 11.3 ms | 12.3 ms | 17.2 ms (3 vblanks) | ~52 | ~33% |

The scene is smooth when the GPU is boosted, but **GPU-bound at the low clock**
— which is the regime of a quick switch or a weak GPU.

## Scene vs surrogate at 450 MHz (the decisive comparison)

| build | config | busy p50 | busy p90 | missed vblanks | frames/burst | cadence |
|-------|--------|---------:|---------:|---------------:|-------------:|:-------:|
| scene   | base                | 11.3 ms | 12.3 ms | ~52 | ~28 | 3-vblank |
| scene   | backdrop+transparency | 12.5 ms | 13.7 ms | ~53 | ~28 | 3-vblank |
| scene   | rounded corners     | 11.3 ms | 12.2 ms | ~53 | ~27 | 3-vblank |
| layouts | base                | **3.0 ms** | **4.0 ms** | **~11** | ~97 | 1–2 vblank |
| layouts | backdrop+transparency | **2.6 ms** | **3.9 ms** | **~10** | ~97 | 1–2 vblank |
| layouts | rounded corners     | **3.0 ms** | **4.0 ms** | **~13** | ~94 | 2-vblank |

The surrogate build is **~4× cheaper on GPU-busy and drops ~5× fewer frames**,
and backdrop+transparency barely affects it (it is the scene's worst case).
Surrogate trades GPU for CPU (per-frame WM tick ~0.75 ms vs the scene's
~0.25 ms) — the right trade when the GPU is the bottleneck.

## Where the scene's heavy frames are (`--frames`, scene @ 450 MHz)

Per-frame GPU-busy through the animation, split into build / motion / lift:

| phase | busy p50 | busy max |
|-------|---------:|---------:|
| build (first frames) | ~11–12 ms | ~16 ms |
| motion (middle)      | ~10–11 ms | ~12 ms |
| lift (end)           | ~10 ms    | ~13 ms |

The build/first frame is the single heaviest, but **motion frames are nearly as
heavy** and the lift is only slightly cheaper — the whole full-screen composite
is the cost, not a one-off warm-up spike. (Surrogate, for contrast: ~2.7–3 ms
everywhere, lift ~1 ms.)

## Takeaway

The scene stays visually solid (no copy/real flash, no border/backdrop flicker)
but composites a full-screen scene every frame (desktop copy + every window's
thumbnail + a warm-up Layer), so it is GPU-bound on a weak or unboosted GPU. To
make it as cheap as the surrogate build while keeping its solidity, it needs to
**composite only the moving windows and drop the full-screen desktop cover**.
Trimming the warm-up frame or capping overlapping groups helps at the margins
but will not close the gap, since the motion frames are GPU-bound too.
