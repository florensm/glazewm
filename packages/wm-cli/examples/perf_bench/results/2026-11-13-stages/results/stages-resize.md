# Resize: second-round stages (stages.py)

## resize-load0

### resize `default`: 40 reports

| stage | before answer ms | calls ≥0.3 ms | first peek_frame ms | calls ≥0.3 ms | outside frames ms | calls |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `peek_draw` | 0.00 | 0.0 | 29.01 | 6.0 | 0.00 | 0.0 |
| `peek_track` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 0.0 |
| `peek_ring` | 9.26 | 8.0 | 0.00 | 0.0 | 10.55 | 40.0 |
| `peek_backdrop` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 40.0 |
| `peek_styles` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 6.0 |
| `peek_warm` | 0.00 | 0.0 | 0.00 | 0.0 | 0.10 | 6.0 |
| `sg_window` | 3.60 | 6.0 | 0.00 | 0.0 | 3.60 | 6.0 |
| `sg_thumb` | 0.00 | 0.0 | 0.00 | 0.0 | 0.55 | 6.0 |
| `ws_surrogates` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 0.0 |
| `ws_park` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 0.0 |
| `ws_show` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 0.0 |
| `ws_start` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 0.0 |
| `sync_focus` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 7.0 |
| `tab_bars` | 0.00 | 0.0 | 0.00 | 0.0 | 0.10 | 7.0 |
| `overview` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 7.0 |
| `guards` | 0.40 | 1.0 | 0.00 | 0.0 | 0.45 | 7.0 |

### resize: first `peek_frame` (slowest input's timeline), medians

| mode | n | command | to first frame | peek_frame | wait (wake→tick) | peek_draw | draws | peek_track | after last draw | peek_ring in frame | peek_backdrop in frame | peek_frame − draw − track |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `default` | 40 | 47.70 | 77.40 | 31.73 | 2.46 | 29.01 | 6.0 | 0.00 | 0.08 | 0.00 | 0.00 | 2.57 |
| `noasynchandback` | 40 | 44.95 | 74.89 | 31.74 | 2.83 | 29.02 | 6.0 | 0.00 | 0.08 | 0.00 | 0.00 | 2.96 |

### resize: `peek_new` outside frames, medians per report (ms, calls)

| mode | n | peek_new | peek_styles | peek_warm | sg_window | sg_thumb | rest | rest % |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `default` | 40 | 5.65 (6) | 0.00 (6) | 0.10 (6) | 3.60 (6) | 0.55 (6) | 1.30 | 23 |
| `noasynchandback` | 40 | 5.70 (6) | 0.00 (6) | 0.10 (6) | 3.60 (6) | 0.60 (6) | 1.30 | 23 |

### resize `default`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: peek_frame | 108.35 | 80.80 | 27.80 | 26 |
| frames: tick | 142.40 | 40.70 | 101.45 | 72 |
| frames: platform_sync | 40.60 | 39.90 | 0.50 | 1 |
| frames: redraw | 36.25 | 23.65 | 12.75 | 38 |
| frames: redraw_loop | 23.35 | 22.25 | 1.00 | 5 |
| frames: rd_apply | 22.25 | 20.70 | 1.30 | 6 |
| frames: sync_tail | 3.00 | 1.20 | 1.70 | 52 |
| frames: peek_track (ring+backdrop) | 8.25 | 7.90 | 0.50 | 6 |
| frames: tick (+peek_draw+peek_track) | 142.40 | 120.90 | 21.30 | 15 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 36.25 | 36.15 | 0.00 | 0 |
| frames: sync_tail (+backdrop/border_sync) | 3.00 | 3.00 | 0.00 | 0 |
| outside: platform_sync | 52.00 | 51.30 | 0.85 | 2 |
| outside: redraw | 50.00 | 29.10 | 20.95 | 43 |
| outside: redraw_prep | 0.20 | 0.00 | 0.20 | 100 |
| outside: redraw_loop | 15.55 | 15.10 | 0.50 | 3 |
| outside: rd_apply | 3.95 | 3.90 | 0.00 | 0 |
| outside: sync_tail | 1.30 | 0.55 | 0.70 | 51 |
| outside: peek_new | 5.65 | 4.35 | 1.30 | 23 |
| outside: redraw (+cloak) | 50.00 | 49.95 | 0.00 | 0 |
| outside: sync_tail (+backdrop/border_sync) | 1.30 | 1.30 | 0.00 | 0 |

### resize `nopeeks`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: tick | 93.30 | 87.95 | 5.75 | 6 |
| frames: platform_sync | 84.65 | 83.45 | 1.20 | 1 |
| frames: redraw | 74.25 | 19.65 | 54.15 | 73 |
| frames: redraw_loop | 13.90 | 10.50 | 3.20 | 24 |
| frames: rd_apply | 10.15 | 10.10 | 0.00 | 0 |
| frames: sync_tail | 8.95 | 1.20 | 7.60 | 86 |
| frames: tick (+peek_draw+peek_track) | 93.30 | 87.95 | 5.75 | 6 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 74.25 | 74.15 | 0.10 | 0 |
| frames: sync_tail (+backdrop/border_sync) | 8.95 | 8.90 | 0.10 | 1 |
| outside: platform_sync | 33.05 | 32.95 | 0.20 | 1 |
| outside: redraw | 32.45 | 16.25 | 14.20 | 48 |
| outside: redraw_prep | 0.10 | 0.00 | 0.10 | 100 |
| outside: redraw_loop | 9.85 | 9.55 | 0.30 | 3 |
| outside: rd_apply | 3.15 | 3.10 | 0.00 | 0 |
| outside: sync_tail | 0.60 | 0.20 | 0.40 | 75 |
| outside: redraw (+cloak) | 32.45 | 35.95 | -3.90 | -13 |
| outside: sync_tail (+backdrop/border_sync) | 0.60 | 0.60 | -0.00 | -0 |

### resize `nopeeks+nokeepoverlays`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: tick | 90.15 | 85.65 | 4.40 | 5 |
| frames: platform_sync | 81.95 | 80.90 | 1.20 | 1 |
| frames: redraw | 71.85 | 19.50 | 52.10 | 73 |
| frames: redraw_loop | 13.40 | 10.30 | 3.00 | 24 |
| frames: rd_apply | 9.95 | 9.90 | 0.00 | 0 |
| frames: sync_tail | 8.75 | 1.20 | 7.55 | 86 |
| frames: tick (+peek_draw+peek_track) | 90.15 | 85.65 | 4.40 | 5 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 71.85 | 71.70 | 0.10 | 0 |
| frames: sync_tail (+backdrop/border_sync) | 8.75 | 8.65 | 0.20 | 2 |
| outside: platform_sync | 36.45 | 36.10 | 0.10 | 0 |
| outside: redraw | 35.25 | 20.20 | 15.60 | 41 |
| outside: redraw_prep | 0.10 | 0.00 | 0.10 | 100 |
| outside: redraw_loop | 11.65 | 11.25 | 0.30 | 3 |
| outside: rd_apply | 3.10 | 3.10 | 0.00 | 0 |
| outside: sync_tail | 0.60 | 0.20 | 0.40 | 69 |
| outside: redraw (+cloak) | 35.25 | 41.40 | -5.60 | -17 |
| outside: sync_tail (+backdrop/border_sync) | 0.60 | 0.70 | -0.10 | -11 |

## resize-cpu8

### resize `default`: 40 reports

| stage | before answer ms | calls ≥0.3 ms | first peek_frame ms | calls ≥0.3 ms | outside frames ms | calls |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `peek_draw` | 0.00 | 0.0 | 29.92 | 6.0 | 0.00 | 0.0 |
| `peek_track` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 0.0 |
| `peek_ring` | 12.25 | 9.0 | 0.00 | 0.0 | 13.60 | 40.0 |
| `peek_backdrop` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 40.0 |
| `peek_styles` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 6.0 |
| `peek_warm` | 0.00 | 0.0 | 0.00 | 0.0 | 0.10 | 6.0 |
| `sg_window` | 4.56 | 6.0 | 0.00 | 0.0 | 4.60 | 6.0 |
| `sg_thumb` | 0.00 | 0.0 | 0.00 | 0.0 | 0.60 | 6.0 |
| `ws_surrogates` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 0.0 |
| `ws_park` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 0.0 |
| `ws_show` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 0.0 |
| `ws_start` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 0.0 |
| `sync_focus` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 7.0 |
| `tab_bars` | 0.00 | 0.0 | 0.00 | 0.0 | 0.10 | 7.0 |
| `overview` | 0.00 | 0.0 | 0.00 | 0.0 | 0.00 | 7.0 |
| `guards` | 0.52 | 1.0 | 0.00 | 0.0 | 0.60 | 7.0 |

### resize: first `peek_frame` (slowest input's timeline), medians

| mode | n | command | to first frame | peek_frame | wait (wake→tick) | peek_draw | draws | peek_track | after last draw | peek_ring in frame | peek_backdrop in frame | peek_frame − draw − track |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `default` | 40 | 66.75 | 100.08 | 33.05 | 2.78 | 29.92 | 6.0 | 0.00 | 0.11 | 0.00 | 0.00 | 2.93 |
| `noasynchandback` | 40 | 65.29 | 95.97 | 32.43 | 2.24 | 30.36 | 6.0 | 0.00 | 0.11 | 0.00 | 0.00 | 2.39 |

### resize: `peek_new` outside frames, medians per report (ms, calls)

| mode | n | peek_new | peek_styles | peek_warm | sg_window | sg_thumb | rest | rest % |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `default` | 40 | 7.15 (6) | 0.00 (6) | 0.10 (6) | 4.60 (6) | 0.60 (6) | 1.70 | 24 |
| `noasynchandback` | 40 | 7.00 (6) | 0.00 (6) | 0.10 (6) | 4.60 (6) | 0.60 (6) | 1.65 | 23 |

### resize `default`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: peek_frame | 125.55 | 93.25 | 32.05 | 25 |
| frames: tick | 173.55 | 54.85 | 117.20 | 68 |
| frames: platform_sync | 54.80 | 54.35 | 0.90 | 2 |
| frames: redraw | 50.80 | 31.05 | 20.35 | 37 |
| frames: redraw_loop | 30.70 | 29.15 | 1.20 | 4 |
| frames: rd_apply | 29.15 | 27.20 | 2.00 | 6 |
| frames: sync_tail | 1.90 | 0.90 | 1.00 | 50 |
| frames: peek_track (ring+backdrop) | 12.95 | 12.75 | 0.50 | 3 |
| frames: tick (+peek_draw+peek_track) | 173.55 | 149.30 | 23.10 | 14 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 50.80 | 50.80 | 0.00 | 0 |
| frames: sync_tail (+backdrop/border_sync) | 1.90 | 1.95 | 0.00 | 0 |
| outside: platform_sync | 76.20 | 74.55 | 1.40 | 2 |
| outside: redraw | 71.70 | 38.75 | 32.40 | 46 |
| outside: redraw_prep | 0.20 | 0.00 | 0.20 | 100 |
| outside: redraw_loop | 19.60 | 19.05 | 0.60 | 3 |
| outside: rd_apply | 5.90 | 5.85 | 0.00 | 0 |
| outside: sync_tail | 2.40 | 0.70 | 1.45 | 61 |
| outside: peek_new | 7.15 | 5.45 | 1.70 | 24 |
| outside: redraw (+cloak) | 71.70 | 71.55 | 0.00 | 0 |
| outside: sync_tail (+backdrop/border_sync) | 2.40 | 2.40 | 0.00 | 0 |

### resize `nopeeks`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: tick | 130.10 | 121.80 | 7.65 | 6 |
| frames: platform_sync | 116.85 | 115.55 | 1.40 | 1 |
| frames: redraw | 102.25 | 32.55 | 71.65 | 68 |
| frames: redraw_loop | 24.55 | 21.05 | 3.70 | 15 |
| frames: rd_apply | 19.85 | 19.85 | 0.00 | 0 |
| frames: sync_tail | 13.45 | 1.50 | 11.85 | 89 |
| frames: tick (+peek_draw+peek_track) | 130.10 | 121.80 | 7.65 | 6 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 102.25 | 102.20 | 0.15 | 0 |
| frames: sync_tail (+backdrop/border_sync) | 13.45 | 13.25 | 0.20 | 1 |
| outside: platform_sync | 49.75 | 49.50 | 0.25 | 1 |
| outside: redraw | 47.40 | 25.65 | 19.40 | 43 |
| outside: redraw_prep | 0.20 | 0.00 | 0.20 | 100 |
| outside: redraw_loop | 15.00 | 14.60 | 0.40 | 3 |
| outside: rd_apply | 5.00 | 4.95 | 0.00 | 0 |
| outside: sync_tail | 1.20 | 0.20 | 1.00 | 83 |
| outside: redraw (+cloak) | 47.40 | 51.85 | -5.30 | -12 |
| outside: sync_tail (+backdrop/border_sync) | 1.20 | 1.25 | 0.00 | 0 |

### resize `nopeeks+nokeepoverlays`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: tick | 126.00 | 119.55 | 7.05 | 5 |
| frames: platform_sync | 115.70 | 114.65 | 1.30 | 1 |
| frames: redraw | 102.55 | 30.20 | 71.25 | 71 |
| frames: redraw_loop | 22.45 | 18.95 | 3.70 | 17 |
| frames: rd_apply | 18.60 | 18.55 | 0.00 | 0 |
| frames: sync_tail | 12.10 | 1.30 | 10.70 | 90 |
| frames: tick (+peek_draw+peek_track) | 126.00 | 119.55 | 7.05 | 5 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 102.55 | 102.40 | 0.15 | 0 |
| frames: sync_tail (+backdrop/border_sync) | 12.10 | 11.95 | 0.20 | 1 |
| outside: platform_sync | 55.20 | 55.00 | 0.20 | 0 |
| outside: redraw | 53.75 | 34.50 | 19.05 | 36 |
| outside: redraw_prep | 0.20 | 0.00 | 0.20 | 100 |
| outside: redraw_loop | 17.00 | 16.50 | 0.40 | 2 |
| outside: rd_apply | 4.40 | 4.35 | 0.00 | 0 |
| outside: sync_tail | 1.10 | 0.20 | 0.90 | 82 |
| outside: redraw (+cloak) | 53.75 | 61.45 | -7.90 | -15 |
| outside: sync_tail (+backdrop/border_sync) | 1.10 | 1.10 | 0.00 | 0 |

## main-load0 (`default` of the main run)

### resize: first `peek_frame` (slowest input's timeline), medians

| mode | n | command | to first frame | peek_frame | wait (wake→tick) | peek_draw | draws | peek_track | after last draw | peek_ring in frame | peek_backdrop in frame | peek_frame − draw − track |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `default` | 40 | 56.62 | 85.25 | 30.89 | 3.25 | 27.00 | 6.0 | 0.00 | 0.09 | 0.00 | 0.00 | 3.39 |

### resize: `peek_new` outside frames, medians per report (ms, calls)

| mode | n | peek_new | peek_styles | peek_warm | sg_window | sg_thumb | rest | rest % |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `default` | 40 | 6.45 (6) | 0.00 (6) | 0.10 (6) | 4.20 (6) | 0.60 (6) | 1.45 | 23 |

## main-cpu8 (`default` of the main run)

### resize: first `peek_frame` (slowest input's timeline), medians

| mode | n | command | to first frame | peek_frame | wait (wake→tick) | peek_draw | draws | peek_track | after last draw | peek_ring in frame | peek_backdrop in frame | peek_frame − draw − track |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `default` | 40 | 52.38 | 83.59 | 32.67 | 3.32 | 29.93 | 6.0 | 0.00 | 0.10 | 0.00 | 0.00 | 3.44 |

### resize: `peek_new` outside frames, medians per report (ms, calls)

| mode | n | peek_new | peek_styles | peek_warm | sg_window | sg_thumb | rest | rest % |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `default` | 40 | 6.60 (6) | 0.00 (6) | 0.10 (6) | 4.30 (6) | 0.60 (6) | 1.50 | 23 |
