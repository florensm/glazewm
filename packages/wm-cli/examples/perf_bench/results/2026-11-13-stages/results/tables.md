# Per-scenario tables (median over 4 rounds, load 0 / CPU 8)

`(main run)`: the `nopeeks`/`default` rounds of `bench-run.ps1`'s `main,nopeeks,default` run, where each mode-round runs `resize` then `switch` in one WM session. input→first frame is not recorded on `switch` (0.00).

### resize (ms, median over rounds, load 0 / CPU 8)

| mode | command p50 | input→first frame | frame p90 | interval p90 | interval p99 | interval max (median burst) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `nopeeks` | 30.52 / 42.85 | 34.98 / 51.93 | 3.79 / 5.82 | 5.78 / 6.42 | 14.51 / 27.35 | 14.51 / 27.35 |
| `nopeeks+nokeepoverlays` | 35.88 / 49.10 | 39.69 / 54.75 | 3.78 / 4.92 | 5.78 / 5.97 | 14.09 / 27.35 | 14.09 / 27.35 |
| `default` | 47.73 / 67.63 | 82.22 / 103.50 | 2.96 / 3.35 | 5.80 / 5.88 | 35.62 / 53.16 | 35.62 / 53.16 |
| `noasynchandback` | 45.49 / 66.91 | 77.58 / 96.44 | 2.79 / 3.39 | 5.80 / 5.90 | 47.69 / 59.59 | 47.69 / 59.59 |
| `main` (main run) | 33.25 / 33.17 | 36.78 / 39.06 | 4.33 / 4.62 | 5.79 / 5.77 | 37.25 / 30.07 | 37.25 / 30.07 |
| `nopeeks` (main run) | 35.49 / 33.89 | 39.75 / 41.84 | 4.70 / 4.26 | 5.80 / 5.78 | 18.52 / 17.08 | 18.52 / 17.08 |
| `default` (main run) | 54.98 / 53.53 | 87.85 / 85.42 | 3.25 / 3.51 | 5.84 / 5.83 | 44.17 / 36.82 | 44.17 / 36.82 |

### switch (ms, median over rounds, load 0 / CPU 8)

| mode | command p50 | input→first frame | frame p90 | interval p90 | interval p99 | interval max (median burst) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `nopeeks` | 27.56 / 34.79 | 0.00 / 0.00 | 1.25 / 1.57 | 5.75 / 5.77 | 17.63 / 22.57 | 17.63 / 23.66 |
| `default` | 27.66 / 34.42 | 0.00 / 0.00 | 1.23 / 1.46 | 5.75 / 5.77 | 16.23 / 23.29 | 16.23 / 23.29 |
| `noslidepeeks` | 27.87 / 35.11 | 0.00 / 0.00 | 1.31 / 1.40 | 5.76 / 5.77 | 16.42 / 22.87 | 16.42 / 23.40 |
| `main` (main run) | 42.45 / 37.50 | 0.00 / 0.00 | 0.94 / 0.76 | 5.76 / 5.75 | 79.09 / 71.70 | 79.09 / 71.70 |
| `nopeeks` (main run) | 31.50 / 26.52 | 0.00 / 0.00 | 1.25 / 1.21 | 5.76 / 5.75 | 20.62 / 18.06 | 20.62 / 20.36 |
| `default` (main run) | 32.67 / 27.65 | 0.00 / 0.00 | 1.23 / 1.32 | 5.78 / 5.75 | 20.91 / 20.76 | 20.91 / 20.82 |
