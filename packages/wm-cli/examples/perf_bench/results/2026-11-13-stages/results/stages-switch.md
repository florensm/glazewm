# Switch: stages per burst (stages.py)

## switch-load0

### switch: outside-frames ms per burst (median over rounds)

| stage | `nopeeks` | `default` | `noslidepeeks` |
| --- | --- | --- | --- |
| platform_sync | 27.0 | 26.6 | 27.1 |
| redraw | 21.4 | 20.9 | 21.5 |
| redraw_prep | 17.4 | 17.0 | 17.6 |
| ws_surrogates | 3.8 | 3.9 | 3.8 |
| ws_park | 2.4 | 2.4 | 2.5 |
| ws_show | 10.8 | 10.3 | 10.8 |
| ws_start | 0.4 | 0.4 | 0.4 |
| redraw_loop | 3.9 | 3.9 | 3.9 |
| sync_tail | 5.6 | 5.7 | 5.6 |
| sync_focus | 0.0 | 0.0 | 0.0 |
| tab_bars | 0.0 | 0.0 | 0.0 |
| overview | 0.0 | 0.0 | 0.0 |
| guards | 0.1 | 0.1 | 0.1 |
| backdrop_sync | 0.0 | 0.0 | 0.0 |
| border_sync | 0.0 | 0.1 | 0.1 |
| dwm_flush | 6.9 | 6.6 | 7.0 |
| sg_window | 2.3 | 2.3 | 2.2 |
| sg_thumb | 0.6 | 0.6 | 0.6 |
| platform_sync − redraw | 5.6 | 5.7 | 5.6 |
| sync_tail − children − backdrop/border_sync | 5.5 | 5.5 | 5.4 |
| reports per run | 10 | 10 | 10 |

### switch `nopeeks`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: tick | 69.30 | 27.25 | 42.00 | 63 |
| frames: platform_sync | 14.30 | 14.15 | 0.20 | 1 |
| frames: redraw | 10.80 | 10.80 | 0.00 | 0 |
| frames: redraw_loop | 10.60 | 10.25 | 0.40 | 3 |
| frames: rd_apply | 10.25 | 9.55 | 0.65 | 6 |
| frames: sync_tail | 2.95 | 0.60 | 2.30 | 70 |
| frames: tick (+peek_draw+peek_track) | 69.30 | 27.25 | 42.00 | 63 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 10.80 | 10.80 | 0.00 | 0 |
| frames: sync_tail (+backdrop/border_sync) | 2.95 | 1.00 | 1.70 | 52 |
| outside: platform_sync | 26.35 | 26.35 | 0.00 | 0 |
| outside: redraw | 20.75 | 20.75 | 0.00 | 0 |
| outside: redraw_prep | 16.95 | 16.90 | 0.10 | 1 |
| outside: redraw_loop | 3.90 | 3.80 | 0.10 | 3 |
| outside: rd_apply | 3.80 | 3.25 | 0.55 | 14 |
| outside: sync_tail | 5.50 | 0.05 | 5.40 | 99 |
| outside: redraw (+cloak) | 20.75 | 20.75 | 0.00 | 0 |
| outside: sync_tail (+backdrop/border_sync) | 5.50 | 0.10 | 5.35 | 98 |

### switch `default`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: tick | 69.25 | 27.15 | 41.45 | 63 |
| frames: platform_sync | 14.20 | 14.00 | 0.20 | 1 |
| frames: redraw | 10.95 | 10.95 | 0.00 | 0 |
| frames: redraw_loop | 10.65 | 10.30 | 0.30 | 3 |
| frames: rd_apply | 10.30 | 9.60 | 0.70 | 6 |
| frames: sync_tail | 2.50 | 0.60 | 1.65 | 70 |
| frames: tick (+peek_draw+peek_track) | 69.25 | 27.15 | 41.45 | 63 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 10.95 | 10.95 | 0.00 | 0 |
| frames: sync_tail (+backdrop/border_sync) | 2.50 | 1.20 | 1.10 | 49 |
| outside: platform_sync | 26.15 | 26.15 | 0.00 | 0 |
| outside: redraw | 20.05 | 20.05 | 0.00 | 0 |
| outside: redraw_prep | 16.65 | 16.60 | 0.10 | 1 |
| outside: redraw_loop | 3.80 | 3.65 | 0.10 | 3 |
| outside: rd_apply | 3.65 | 3.15 | 0.55 | 15 |
| outside: sync_tail | 5.55 | 0.05 | 5.45 | 99 |
| outside: redraw (+cloak) | 20.05 | 20.05 | 0.00 | 0 |
| outside: sync_tail (+backdrop/border_sync) | 5.55 | 0.10 | 5.30 | 97 |

## switch-cpu8

### switch: outside-frames ms per burst (median over rounds)

| stage | `nopeeks` | `default` | `noslidepeeks` |
| --- | --- | --- | --- |
| platform_sync | 35.5 | 36.5 | 35.2 |
| redraw | 27.7 | 28.3 | 27.6 |
| redraw_prep | 21.3 | 21.9 | 21.9 |
| ws_surrogates | 4.8 | 4.8 | 4.8 |
| ws_park | 3.0 | 2.9 | 3.0 |
| ws_show | 12.9 | 13.6 | 13.5 |
| ws_start | 0.5 | 0.5 | 0.5 |
| redraw_loop | 6.4 | 6.2 | 5.3 |
| sync_tail | 7.8 | 7.9 | 7.5 |
| sync_focus | 0.0 | 0.0 | 0.0 |
| tab_bars | 0.0 | 0.0 | 0.0 |
| overview | 0.0 | 0.0 | 0.0 |
| guards | 0.1 | 0.1 | 0.1 |
| backdrop_sync | 0.0 | 0.0 | 0.0 |
| border_sync | 2.6 | 2.8 | 2.6 |
| dwm_flush | 8.3 | 9.1 | 8.9 |
| sg_window | 2.9 | 2.9 | 2.9 |
| sg_thumb | 0.7 | 0.7 | 0.7 |
| platform_sync − redraw | 7.9 | 8.0 | 7.5 |
| sync_tail − children − backdrop/border_sync | 5.0 | 4.9 | 5.0 |
| reports per run | 10 | 10 | 10 |

### switch `nopeeks`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: tick | 112.75 | 53.45 | 55.35 | 54 |
| frames: platform_sync | 29.60 | 29.35 | 0.20 | 1 |
| frames: redraw | 21.05 | 21.00 | 0.00 | 0 |
| frames: redraw_loop | 20.80 | 20.30 | 0.50 | 3 |
| frames: rd_apply | 20.30 | 18.90 | 0.75 | 5 |
| frames: sync_tail | 6.70 | 1.20 | 5.35 | 76 |
| frames: tick (+peek_draw+peek_track) | 112.75 | 53.45 | 55.35 | 54 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 21.05 | 21.00 | 0.00 | 0 |
| frames: sync_tail (+backdrop/border_sync) | 6.70 | 2.00 | 4.05 | 59 |
| outside: platform_sync | 34.80 | 34.80 | 0.10 | 0 |
| outside: redraw | 26.40 | 26.40 | 0.00 | 0 |
| outside: redraw_prep | 20.85 | 20.70 | 0.10 | 1 |
| outside: redraw_loop | 5.20 | 4.55 | 0.45 | 23 |
| outside: rd_apply | 4.55 | 3.90 | 0.65 | 14 |
| outside: sync_tail | 6.80 | 0.15 | 6.65 | 98 |
| outside: redraw (+cloak) | 26.40 | 26.40 | 0.00 | 0 |
| outside: sync_tail (+backdrop/border_sync) | 6.80 | 1.30 | 4.30 | 75 |

### switch `default`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: tick | 113.35 | 55.25 | 56.60 | 52 |
| frames: platform_sync | 29.40 | 29.10 | 0.20 | 1 |
| frames: redraw | 19.65 | 19.65 | 0.00 | 0 |
| frames: redraw_loop | 19.45 | 18.90 | 0.50 | 3 |
| frames: rd_apply | 18.90 | 17.75 | 0.75 | 4 |
| frames: sync_tail | 7.30 | 1.20 | 5.10 | 78 |
| frames: tick (+peek_draw+peek_track) | 113.35 | 55.25 | 56.60 | 52 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 19.65 | 19.65 | 0.00 | 0 |
| frames: sync_tail (+backdrop/border_sync) | 7.30 | 2.00 | 3.75 | 57 |
| outside: platform_sync | 34.20 | 33.90 | 0.10 | 0 |
| outside: redraw | 25.70 | 25.70 | 0.00 | 0 |
| outside: redraw_prep | 21.00 | 20.85 | 0.20 | 1 |
| outside: redraw_loop | 4.95 | 4.60 | 0.35 | 10 |
| outside: rd_apply | 4.60 | 4.00 | 0.70 | 14 |
| outside: sync_tail | 7.55 | 0.15 | 7.40 | 98 |
| outside: redraw (+cloak) | 25.70 | 25.70 | 0.00 | 0 |
| outside: sync_tail (+backdrop/border_sync) | 7.55 | 1.40 | 4.45 | 75 |

## main-load0

### switch: outside-frames ms per burst (median over rounds)

| stage | `main` | `nopeeks` | `default` |
| --- | --- | --- | --- |
| platform_sync | 37.0 | 26.9 | 32.8 |
| redraw | 34.7 | 20.3 | 25.6 |
| redraw_prep | 14.5 | 16.3 | 20.3 |
| ws_surrogates | 0.0 | 3.3 | 4.1 |
| ws_park | 0.0 | 2.3 | 2.6 |
| ws_show | 0.0 | 10.2 | 13.0 |
| ws_start | 0.0 | 0.4 | 0.4 |
| redraw_loop | 19.9 | 4.1 | 5.1 |
| sync_tail | 0.0 | 6.6 | 7.2 |
| sync_focus | 0.0 | 0.0 | 0.0 |
| tab_bars | 0.0 | 0.0 | 0.0 |
| overview | 0.0 | 0.0 | 0.0 |
| guards | 0.0 | 0.1 | 0.1 |
| backdrop_sync | 0.0 | 0.0 | 0.0 |
| border_sync | 1.9 | 2.1 | 2.4 |
| dwm_flush | 6.9 | 7.1 | 8.9 |
| sg_window | 0.0 | 1.9 | 2.4 |
| sg_thumb | 0.0 | 0.6 | 0.7 |
| platform_sync − redraw | 2.3 | 6.6 | 7.2 |
| sync_tail − children − backdrop/border_sync | -1.9 | 4.4 | 4.7 |
| reports per run | 10 | 9 | 10 |

### switch `main`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: tick | 126.10 | 77.95 | 49.80 | 40 |
| frames: platform_sync | 24.40 | 21.25 | 2.95 | 11 |
| frames: redraw | 21.25 | 21.25 | 0.00 | 0 |
| frames: redraw_loop | 21.05 | 18.90 | 3.30 | 17 |
| frames: rd_apply | 18.90 | 18.85 | 0.10 | 0 |
| frames: tick (+peek_draw+peek_track) | 126.10 | 77.95 | 49.80 | 40 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 21.25 | 41.50 | -19.55 | -84 |
| outside: platform_sync | 33.25 | 29.85 | 2.95 | 10 |
| outside: redraw | 29.85 | 29.75 | 0.10 | 1 |
| outside: redraw_prep | 13.10 | 0.00 | 13.10 | 100 |
| outside: redraw_loop | 18.25 | 3.15 | 7.50 | 60 |
| outside: rd_apply | 3.15 | 3.15 | 0.00 | 0 |
| outside: redraw (+cloak) | 29.85 | 29.75 | 0.10 | 1 |

### switch `nopeeks`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: tick | 107.70 | 57.85 | 49.60 | 48 |
| frames: platform_sync | 26.00 | 25.85 | 0.20 | 1 |
| frames: redraw | 17.25 | 17.25 | 0.00 | 0 |
| frames: redraw_loop | 17.05 | 16.60 | 0.50 | 3 |
| frames: rd_apply | 16.60 | 15.55 | 1.05 | 6 |
| frames: sync_tail | 7.75 | 1.20 | 6.65 | 84 |
| frames: tick (+peek_draw+peek_track) | 107.70 | 57.85 | 49.60 | 48 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 17.25 | 17.25 | 0.00 | 0 |
| frames: sync_tail (+backdrop/border_sync) | 7.75 | 1.80 | 6.10 | 77 |
| outside: platform_sync | 29.40 | 29.30 | 0.10 | 0 |
| outside: redraw | 19.05 | 19.00 | 0.00 | 0 |
| outside: redraw_prep | 17.30 | 17.15 | 0.10 | 1 |
| outside: redraw_loop | 2.10 | 1.35 | 0.40 | 26 |
| outside: rd_apply | 1.35 | 1.20 | 0.20 | 15 |
| outside: sync_tail | 8.30 | 0.20 | 8.10 | 98 |
| outside: redraw (+cloak) | 19.05 | 19.00 | 0.00 | 0 |
| outside: sync_tail (+backdrop/border_sync) | 8.30 | 3.45 | 4.75 | 68 |

## main-cpu8

### switch: outside-frames ms per burst (median over rounds)

| stage | `main` | `nopeeks` | `default` |
| --- | --- | --- | --- |
| platform_sync | 31.9 | 27.1 | 28.4 |
| redraw | 29.4 | 20.7 | 20.8 |
| redraw_prep | 13.1 | 16.5 | 16.4 |
| ws_surrogates | 0.0 | 4.2 | 4.4 |
| ws_park | 0.0 | 2.7 | 2.7 |
| ws_show | 0.0 | 9.1 | 8.7 |
| ws_start | 0.0 | 0.5 | 0.4 |
| redraw_loop | 16.6 | 4.2 | 4.3 |
| sync_tail | 0.0 | 6.3 | 6.6 |
| sync_focus | 0.0 | 0.0 | 0.0 |
| tab_bars | 0.0 | 0.0 | 0.0 |
| overview | 0.0 | 0.0 | 0.0 |
| guards | 0.0 | 0.1 | 0.1 |
| backdrop_sync | 0.0 | 0.0 | 0.0 |
| border_sync | 2.0 | 1.9 | 2.2 |
| dwm_flush | 5.0 | 5.0 | 4.7 |
| sg_window | 0.0 | 2.6 | 2.6 |
| sg_thumb | 0.0 | 0.6 | 0.7 |
| platform_sync − redraw | 2.5 | 6.4 | 6.7 |
| sync_tail − children − backdrop/border_sync | -2.0 | 4.4 | 4.4 |
| reports per run | 10 | 10 | 10 |

### switch `main`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: tick | 117.40 | 70.65 | 47.45 | 40 |
| frames: platform_sync | 23.05 | 20.90 | 2.45 | 11 |
| frames: redraw | 20.90 | 20.90 | 0.00 | 0 |
| frames: redraw_loop | 20.75 | 18.55 | 2.90 | 16 |
| frames: rd_apply | 18.55 | 18.55 | 0.00 | 0 |
| frames: tick (+peek_draw+peek_track) | 117.40 | 70.65 | 47.45 | 40 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 20.90 | 37.75 | -16.50 | -78 |
| outside: platform_sync | 37.15 | 35.00 | 3.00 | 9 |
| outside: redraw | 35.00 | 34.95 | 0.10 | 1 |
| outside: redraw_prep | 14.05 | 0.00 | 14.05 | 100 |
| outside: redraw_loop | 20.30 | 3.65 | 6.50 | 61 |
| outside: rd_apply | 3.65 | 3.60 | 0.00 | 0 |
| outside: redraw (+cloak) | 35.00 | 34.95 | 0.10 | 1 |

### switch `nopeeks`: parent vs children, medians per report (ms)

| stage | total | children | gap | gap % |
| --- | ---: | ---: | ---: | ---: |
| frames: tick | 107.10 | 56.35 | 48.25 | 48 |
| frames: platform_sync | 26.30 | 25.95 | 0.20 | 1 |
| frames: redraw | 17.45 | 17.40 | 0.00 | 0 |
| frames: redraw_loop | 17.20 | 16.70 | 0.40 | 3 |
| frames: rd_apply | 16.70 | 15.45 | 1.00 | 5 |
| frames: sync_tail | 7.45 | 1.10 | 6.40 | 83 |
| frames: tick (+peek_draw+peek_track) | 107.10 | 56.35 | 48.25 | 48 |
| frames: redraw (+all in-frame dwm_flush, batch_commit) | 17.45 | 17.40 | 0.00 | 0 |
| frames: sync_tail (+backdrop/border_sync) | 7.45 | 1.50 | 5.50 | 69 |
| outside: platform_sync | 27.65 | 27.60 | 0.10 | 0 |
| outside: redraw | 19.55 | 19.55 | 0.00 | 0 |
| outside: redraw_prep | 16.80 | 16.65 | 0.10 | 1 |
| outside: redraw_loop | 2.30 | 1.35 | 0.50 | 29 |
| outside: rd_apply | 1.35 | 1.25 | 0.20 | 15 |
| outside: sync_tail | 7.20 | 0.20 | 7.00 | 98 |
| outside: redraw (+cloak) | 19.55 | 19.55 | 0.00 | 0 |
| outside: sync_tail (+backdrop/border_sync) | 7.20 | 2.15 | 4.05 | 76 |
