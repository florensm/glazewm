Branch comparison, 2026-11-19T22:12:03: this = 092d63ad, against origin/main (ffba51c), tryout/restructure (dee4693); builds this, main, restructure. Target chrome, 3 round(s) of 10 bursts per scenario, build and setting, IPC load 0, CPU load 0,8 thread(s) of 16, with DWM timing, tile order Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot, launched none (0 window(s)).
tiles this cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.36
  command p90 ms                          20.11
  frames                                  63.00
  tick ms/frame                            0.38
  frame p90 ms                             0.50
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.57
  interval max ms (median burst)          10.57
  peek_frame ms/frame                      0.38
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.04
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.65
  pd_alpha ms (worst frame)                0.24
  pd_draw ms (worst frame)                 0.60
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.60
  cloak ms (outside frames)                8.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.10
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.70
  to_draw ms (outside frames)              0.40
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             1.80
  peek_cover ms (outside frames)           1.10
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            0.30
  bd_copy ms (outside frames)              0.10
  ovl_restack ms (outside frames)          0.30
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)        3.00
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              1.10
  restack ms (outside frames)              0.90
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.56
  input->first frame ms                   18.76
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      7.99
| this cpu0 bd-config bo-config #1 | move | 18.36 | 20.11 | 63.00 | 0.38 | 0.50 | 0.00 | 5.71 | 5.74 | 10.57 | 10.57 | 0.38 | 0.16 | 0.04 | 0.01 | 0.03 | 0.65 | 0.24 | 0.60 | - | - | - | - | - | - | - | 5.60 | 8.70 | - | - | - | 18.10 | 2.10 | 1.20 | - | - | - | 2.70 | 0.40 | 0.50 | 1.80 | 1.10 | 0.00 | 1.30 | 0.30 | 0.10 | 0.30 | - | 0.20 | 3.00 | 0.20 | 1.10 | 0.90 | 2.00 | 0.00 | - | - | - | - | 3.56 | 18.76 | 5.00 | 11.43 | 7.99 |
label=this cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          56.74
  command p90 ms                          58.52
  frames                                  61.00
  tick ms/frame                            2.27
  frame p90 ms                             3.46
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         40.95
  interval max ms (median burst)          40.95
  peek_frame ms/frame                      1.98
  peek_draw ms/frame                       1.30
  peek_track ms/frame                      0.27
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.13
  peek_draw ms (worst frame)              17.78
  pd_alpha ms (worst frame)                1.05
  pd_draw ms (worst frame)                16.88
  pd_fill ms (worst frame)                13.48
  fill_source ms (worst frame)             4.55
  ring_raise ms (worst frame)              6.17
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.30
  cloak ms (outside frames)               24.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    56.40
  peek_new ms (outside frames)             3.50
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.10
  to_draw ms (outside frames)              1.10
  to_alpha ms (outside frames)             2.80
  to_cloak ms (outside frames)             8.60
  peek_cover ms (outside frames)          11.70
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.10
  ring_copy ms (outside frames)            7.20
  bd_copy ms (outside frames)              0.50
  ovl_restack ms (outside frames)          2.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         2.20
  peek_handover ms (outside frames)       14.20
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.80
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.07
  input->first frame ms                   76.20
  dwm missed vblanks/burst                18.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      1.37
| this cpu0 bd-config bo-config #1 | resize | 56.74 | 58.52 | 61.00 | 2.27 | 3.46 | 3.00 | 5.72 | 5.77 | 40.95 | 40.95 | 1.98 | 1.30 | 0.27 | 0.13 | 0.13 | 17.78 | 1.05 | 16.88 | 13.48 | 4.55 | 6.17 | 0.00 | - | 0.04 | 2.00 | 10.30 | 24.30 | - | - | - | 56.40 | 3.50 | 2.00 | - | - | - | 13.10 | 1.10 | 2.80 | 8.60 | 11.70 | 0.00 | 1.10 | 7.20 | 0.50 | 2.10 | - | 2.20 | 14.20 | 0.10 | 0.80 | - | 6.00 | 0.00 | - | - | - | - | 3.07 | 76.20 | 18.00 | 11.44 | 1.37 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          30.69
  command p90 ms                          32.88
  frames                                  99.00
  tick ms/frame                            1.14
  frame p90 ms                             1.19
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         21.76
  interval max ms (median burst)          21.76
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.50
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.40
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)               -
  to_draw ms (outside frames)                 -
  to_alpha ms (outside frames)                -
  to_cloak ms (outside frames)                -
  peek_cover ms (outside frames)              -
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)        9.90
  ws_park ms (outside frames)              2.20
  ws_show ms (outside frames)              8.10
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                12.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      2.34
| this cpu0 bd-config bo-config #1 | switch | 30.69 | 32.88 | 99.00 | 1.14 | 1.19 | 3.00 | 5.72 | 5.76 | 21.76 | 21.76 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.50 | - | - | - | - | 30.40 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | - | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 9.90 | 2.20 | 8.10 | 0.40 | - | - | 12.00 | 11.44 | 2.34 |
label=this cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          45.49
  command p90 ms                          49.75
  frames                                  62.00
  tick ms/frame                            2.24
  frame p90 ms                             3.94
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         54.87
  interval max ms (median burst)          54.87
  peek_frame ms/frame                      1.83
  peek_draw ms/frame                       1.14
  peek_track ms/frame                      0.35
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.20
  peek_draw ms (worst frame)              11.58
  pd_alpha ms (worst frame)                1.05
  pd_draw ms (worst frame)                10.72
  pd_fill ms (worst frame)                 6.94
  fill_source ms (worst frame)             2.24
  ring_raise ms (worst frame)              3.15
  restack ms (worst frame)                 2.75
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           12.90
  cloak ms (outside frames)               18.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    45.20
  peek_new ms (outside frames)             9.40
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            9.50
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.30
  to_cloak ms (outside frames)             7.10
  peek_cover ms (outside frames)          17.80
  ring_prep ms (outside frames)            2.50
  ring_park ms (outside frames)            1.10
  ring_copy ms (outside frames)            7.70
  bd_copy ms (outside frames)              0.90
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.70
  peek_handover ms (outside frames)       14.30
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              3.00
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    11.03
  input->first frame ms                   60.81
  dwm missed vblanks/burst                30.00
  dwm display interval p90 ms             22.87
  dwm gpu busy p90 ms                      7.73
| this cpu0 bd-config bo-config #1 | minimize | 45.49 | 49.75 | 62.00 | 2.24 | 3.94 | 3.00 | 5.72 | 5.77 | 54.87 | 54.87 | 1.83 | 1.14 | 0.35 | 0.13 | 0.20 | 11.58 | 1.05 | 10.72 | 6.94 | 2.24 | 3.15 | 2.75 | - | 0.02 | 2.00 | 12.90 | 18.00 | - | - | - | 45.20 | 9.40 | 1.80 | - | - | - | 9.50 | 0.90 | 2.30 | 7.10 | 17.80 | 2.50 | 1.10 | 7.70 | 0.90 | 0.00 | - | 1.70 | 14.30 | 0.30 | 3.00 | - | 6.00 | 0.00 | - | - | - | - | 11.03 | 60.81 | 30.00 | 22.87 | 7.73 |
tiles main cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.85
  command p90 ms                          18.62
  frames                                  63.00
  tick ms/frame                            0.39
  frame p90 ms                             0.52
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.61
  interval max ms (median burst)          10.61
  peek_frame ms/frame                      0.34
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.66
  pd_alpha ms (worst frame)                0.27
  pd_draw ms (worst frame)                 0.51
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)                7.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.60
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.60
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.40
  to_cloak ms (outside frames)             1.80
  peek_cover ms (outside frames)           1.30
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            0.20
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        2.50
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.10
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.46
  input->first frame ms                   18.87
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.42
  dwm gpu busy p90 ms                      7.91
| main cpu0 bd-config bo-config #1 | move | 17.85 | 18.62 | 63.00 | 0.39 | 0.52 | 0.00 | 5.71 | 5.74 | 10.61 | 10.61 | 0.34 | 0.16 | 0.05 | 0.01 | 0.03 | 0.66 | 0.27 | 0.51 | - | - | - | - | - | - | - | 4.90 | 7.60 | - | - | - | 17.60 | 2.10 | 1.10 | - | - | - | 2.60 | 0.30 | 0.40 | 1.80 | 1.30 | 0.00 | 1.30 | 0.20 | 0.20 | 0.60 | - | 0.10 | 2.50 | 0.30 | 1.10 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.46 | 18.87 | 5.00 | 11.42 | 7.91 |
label=main cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          40.36
  command p90 ms                          45.66
  frames                                  62.00
  tick ms/frame                            2.15
  frame p90 ms                             3.26
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  interval p99 ms                         40.40
  interval max ms (median burst)          40.40
  peek_frame ms/frame                      1.87
  peek_draw ms/frame                       1.23
  peek_track ms/frame                      0.27
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.11
  peek_draw ms (worst frame)              16.16
  pd_alpha ms (worst frame)                0.97
  pd_draw ms (worst frame)                15.33
  pd_fill ms (worst frame)                11.83
  fill_source ms (worst frame)             2.58
  ring_raise ms (worst frame)              6.10
  restack ms (worst frame)                 2.78
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)               -
  cloak ms (outside frames)               11.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    40.00
  peek_new ms (outside frames)             3.30
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           10.80
  to_draw ms (outside frames)              0.80
  to_alpha ms (outside frames)             2.40
  to_cloak ms (outside frames)             7.30
  peek_cover ms (outside frames)           8.10
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.00
  ring_copy ms (outside frames)            3.50
  bd_copy ms (outside frames)              0.60
  ovl_restack ms (outside frames)          1.90
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.90
  peek_handover ms (outside frames)       14.50
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.98
  input->first frame ms                   55.55
  dwm missed vblanks/burst                22.00
  dwm display interval p90 ms             17.16
  dwm gpu busy p90 ms                      1.87
| main cpu0 bd-config bo-config #1 | resize | 40.36 | 45.66 | 62.00 | 2.15 | 3.26 | 3.00 | 5.72 | 5.79 | 40.40 | 40.40 | 1.87 | 1.23 | 0.27 | 0.12 | 0.11 | 16.16 | 0.97 | 15.33 | 11.83 | 2.58 | 6.10 | 2.78 | - | 0.03 | 2.00 | - | 11.30 | - | - | - | 40.00 | 3.30 | 2.00 | - | - | - | 10.80 | 0.80 | 2.40 | 7.30 | 8.10 | 0.00 | 1.00 | 3.50 | 0.60 | 1.90 | - | 0.90 | 14.50 | 0.10 | 0.70 | - | 6.00 | 0.00 | - | - | - | - | 2.98 | 55.55 | 22.00 | 17.16 | 1.87 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          31.21
  command p90 ms                          33.27
  frames                                  99.00
  tick ms/frame                            1.14
  frame p90 ms                             1.16
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         21.95
  interval max ms (median burst)          21.95
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            6.20
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.90
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)               -
  to_draw ms (outside frames)                 -
  to_alpha ms (outside frames)                -
  to_cloak ms (outside frames)                -
  peek_cover ms (outside frames)              -
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)        9.80
  ws_park ms (outside frames)              2.20
  ws_show ms (outside frames)              8.80
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                16.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      2.48
| main cpu0 bd-config bo-config #1 | switch | 31.21 | 33.27 | 99.00 | 1.14 | 1.16 | 3.00 | 5.71 | 5.75 | 21.95 | 21.95 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 6.20 | - | - | - | - | 30.90 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | - | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 9.80 | 2.20 | 8.80 | 0.40 | - | - | 16.00 | 11.44 | 2.48 |
label=main cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          39.92
  command p90 ms                          43.06
  frames                                  62.00
  tick ms/frame                            2.18
  frame p90 ms                             3.46
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.81
  interval p99 ms                         39.69
  interval max ms (median burst)          39.69
  peek_frame ms/frame                      1.78
  peek_draw ms/frame                       1.06
  peek_track ms/frame                      0.33
  peek_ring ms/frame                       0.11
  peek_backdrop ms/frame                   0.16
  peek_draw ms (worst frame)               8.64
  pd_alpha ms (worst frame)                0.94
  pd_draw ms (worst frame)                 7.60
  pd_fill ms (worst frame)                 4.38
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              3.50
  restack ms (worst frame)                 2.71
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.90
  cloak ms (outside frames)               13.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    39.60
  peek_new ms (outside frames)             8.00
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.90
  to_draw ms (outside frames)              0.80
  to_alpha ms (outside frames)             2.10
  to_cloak ms (outside frames)             9.50
  peek_cover ms (outside frames)          15.40
  ring_prep ms (outside frames)            4.60
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            4.40
  bd_copy ms (outside frames)              0.90
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.80
  peek_handover ms (outside frames)       14.40
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              2.80
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.31
  input->first frame ms                   52.21
  dwm missed vblanks/burst                33.00
  dwm display interval p90 ms             22.91
  dwm gpu busy p90 ms                      9.56
| main cpu0 bd-config bo-config #1 | minimize | 39.92 | 43.06 | 62.00 | 2.18 | 3.46 | 2.00 | 5.71 | 5.81 | 39.69 | 39.69 | 1.78 | 1.06 | 0.33 | 0.11 | 0.16 | 8.64 | 0.94 | 7.60 | 4.38 | 0.00 | 3.50 | 2.71 | - | 0.02 | 2.00 | 5.90 | 13.00 | - | - | - | 39.60 | 8.00 | 1.90 | - | - | - | 12.90 | 0.80 | 2.10 | 9.50 | 15.40 | 4.60 | 1.20 | 4.40 | 0.90 | 0.10 | - | 0.80 | 14.40 | 0.20 | 2.80 | - | 6.00 | 0.00 | - | - | - | - | 10.31 | 52.21 | 33.00 | 22.91 | 9.56 |
tiles restructure cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=restructure cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.29
  command p90 ms                          18.97
  frames                                  63.00
  tick ms/frame                            0.39
  frame p90 ms                             0.50
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         10.76
  interval max ms (median burst)          10.76
  peek_frame ms/frame                      0.37
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.61
  pd_alpha ms (worst frame)                0.25
  pd_draw ms (worst frame)                 0.51
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.20
  cloak ms (outside frames)                8.10
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.00
  peek_new ms (outside frames)             2.00
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.70
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             1.90
  peek_cover ms (outside frames)           1.30
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.10
  ring_copy ms (outside frames)            0.10
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        2.50
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.10
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.42
  input->first frame ms                   18.35
  dwm missed vblanks/burst                 4.00
  dwm display interval p90 ms             11.42
  dwm gpu busy p90 ms                      7.96
| restructure cpu0 bd-config bo-config #1 | move | 17.29 | 18.97 | 63.00 | 0.39 | 0.50 | 0.00 | 5.71 | 5.75 | 10.76 | 10.76 | 0.37 | 0.16 | 0.05 | 0.01 | 0.03 | 0.61 | 0.25 | 0.51 | - | - | - | - | - | - | - | 5.20 | 8.10 | - | - | - | 17.00 | 2.00 | 1.10 | - | - | - | 2.70 | 0.30 | 0.50 | 1.90 | 1.30 | 0.00 | 1.10 | 0.10 | 0.20 | 0.60 | - | 0.10 | 2.50 | 0.30 | 1.10 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.42 | 18.35 | 4.00 | 11.42 | 7.96 |
label=restructure cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          40.80
  command p90 ms                          42.57
  frames                                  62.00
  tick ms/frame                            2.25
  frame p90 ms                             3.22
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         37.45
  interval max ms (median burst)          37.45
  peek_frame ms/frame                      1.88
  peek_draw ms/frame                       1.19
  peek_track ms/frame                      0.29
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.11
  peek_draw ms (worst frame)              12.07
  pd_alpha ms (worst frame)                0.85
  pd_draw ms (worst frame)                11.21
  pd_fill ms (worst frame)                 7.85
  fill_source ms (worst frame)             1.82
  ring_raise ms (worst frame)              4.72
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)               -
  cloak ms (outside frames)               13.50
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    40.50
  peek_new ms (outside frames)             3.40
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.60
  to_draw ms (outside frames)              0.80
  to_alpha ms (outside frames)             2.80
  to_cloak ms (outside frames)             8.30
  peek_cover ms (outside frames)           7.80
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.00
  ring_copy ms (outside frames)            3.50
  bd_copy ms (outside frames)              0.60
  ovl_restack ms (outside frames)          2.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.80
  peek_handover ms (outside frames)       13.10
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              0.80
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.87
  input->first frame ms                   54.16
  dwm missed vblanks/burst                22.00
  dwm display interval p90 ms             17.18
  dwm gpu busy p90 ms                      1.77
| restructure cpu0 bd-config bo-config #1 | resize | 40.80 | 42.57 | 62.00 | 2.25 | 3.22 | 3.00 | 5.71 | 5.78 | 37.45 | 37.45 | 1.88 | 1.19 | 0.29 | 0.14 | 0.11 | 12.07 | 0.85 | 11.21 | 7.85 | 1.82 | 4.72 | 0.00 | - | 0.04 | 2.00 | - | 13.50 | - | - | - | 40.50 | 3.40 | 2.00 | - | - | - | 12.60 | 0.80 | 2.80 | 8.30 | 7.80 | 0.00 | 1.00 | 3.50 | 0.60 | 2.00 | - | 0.80 | 13.10 | 0.20 | 0.80 | - | 6.00 | 0.00 | - | - | - | - | 2.87 | 54.16 | 22.00 | 17.18 | 1.77 |
Moved 3 window(s) to workspace '2' to switch to.
label=restructure cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          30.80
  command p90 ms                          32.27
  frames                                  99.00
  tick ms/frame                            1.14
  frame p90 ms                             1.17
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         21.27
  interval max ms (median burst)          22.54
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.00
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.50
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)               -
  to_draw ms (outside frames)                 -
  to_alpha ms (outside frames)                -
  to_cloak ms (outside frames)                -
  peek_cover ms (outside frames)              -
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.50
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)        9.80
  ws_park ms (outside frames)              2.30
  ws_show ms (outside frames)              7.60
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                14.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      1.94
| restructure cpu0 bd-config bo-config #1 | switch | 30.80 | 32.27 | 99.00 | 1.14 | 1.17 | 3.00 | 5.71 | 5.75 | 21.27 | 22.54 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.04 | - | 5.00 | - | - | - | - | 30.50 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | - | 0.00 | 0.50 | 0.00 | 0.00 | 0.00 | 9.80 | 2.30 | 7.60 | 0.40 | - | - | 14.00 | 11.44 | 1.94 |
label=restructure cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          38.51
  command p90 ms                          46.52
  frames                                  62.00
  tick ms/frame                            2.26
  frame p90 ms                             3.81
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  interval p99 ms                         39.11
  interval max ms (median burst)          39.11
  peek_frame ms/frame                      1.76
  peek_draw ms/frame                       1.14
  peek_track ms/frame                      0.34
  peek_ring ms/frame                       0.11
  peek_backdrop ms/frame                   0.18
  peek_draw ms (worst frame)               8.91
  pd_alpha ms (worst frame)                0.71
  pd_draw ms (worst frame)                 8.29
  pd_fill ms (worst frame)                 4.42
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              3.54
  restack ms (worst frame)                 2.90
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.00
  cloak ms (outside frames)               13.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    38.20
  peek_new ms (outside frames)             7.80
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.60
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             1.80
  to_cloak ms (outside frames)             9.90
  peek_cover ms (outside frames)          13.30
  ring_prep ms (outside frames)            3.10
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            4.10
  bd_copy ms (outside frames)              1.00
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.00
  peek_handover ms (outside frames)       13.90
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              2.70
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.44
  input->first frame ms                   53.59
  dwm missed vblanks/burst                31.00
  dwm display interval p90 ms             22.91
  dwm gpu busy p90 ms                      9.30
| restructure cpu0 bd-config bo-config #1 | minimize | 38.51 | 46.52 | 62.00 | 2.26 | 3.81 | 3.00 | 5.72 | 5.79 | 39.11 | 39.11 | 1.76 | 1.14 | 0.34 | 0.11 | 0.18 | 8.91 | 0.71 | 8.29 | 4.42 | 0.00 | 3.54 | 2.90 | - | 0.03 | 2.00 | 4.00 | 13.00 | - | - | - | 38.20 | 7.80 | 1.90 | - | - | - | 12.60 | 0.90 | 1.80 | 9.90 | 13.30 | 3.10 | 1.30 | 4.10 | 1.00 | 0.10 | - | 1.00 | 13.90 | 0.30 | 2.70 | - | 6.00 | 0.00 | - | - | - | - | 8.44 | 53.59 | 31.00 | 22.91 | 9.30 |
tiles main cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.58
  command p90 ms                          19.41
  frames                                  63.00
  tick ms/frame                            0.39
  frame p90 ms                             0.52
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.60
  interval max ms (median burst)          10.60
  peek_frame ms/frame                      0.33
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.02
  peek_draw ms (worst frame)               0.62
  pd_alpha ms (worst frame)                0.24
  pd_draw ms (worst frame)                 0.55
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)                8.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.30
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.60
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             1.80
  peek_cover ms (outside frames)           1.30
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            0.10
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        2.50
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              1.00
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.47
  input->first frame ms                   19.31
  dwm missed vblanks/burst                 4.00
  dwm display interval p90 ms             11.41
  dwm gpu busy p90 ms                      7.84
| main cpu0 bd-config bo-config #2 | move | 18.58 | 19.41 | 63.00 | 0.39 | 0.52 | 0.00 | 5.71 | 5.74 | 10.60 | 10.60 | 0.33 | 0.16 | 0.05 | 0.01 | 0.02 | 0.62 | 0.24 | 0.55 | - | - | - | - | - | - | - | 5.40 | 8.20 | - | - | - | 18.30 | 2.10 | 1.10 | - | - | - | 2.60 | 0.30 | 0.50 | 1.80 | 1.30 | 0.00 | 1.30 | 0.10 | 0.20 | 0.60 | - | 0.10 | 2.50 | 0.20 | 1.00 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.47 | 19.31 | 4.00 | 11.41 | 7.84 |
label=main cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          40.20
  command p90 ms                          41.97
  frames                                  62.00
  tick ms/frame                            2.23
  frame p90 ms                             3.48
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         37.00
  interval max ms (median burst)          37.00
  peek_frame ms/frame                      1.90
  peek_draw ms/frame                       1.25
  peek_track ms/frame                      0.26
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.11
  peek_draw ms (worst frame)              14.96
  pd_alpha ms (worst frame)                0.95
  pd_draw ms (worst frame)                13.78
  pd_fill ms (worst frame)                10.49
  fill_source ms (worst frame)             2.09
  ring_raise ms (worst frame)              5.03
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)               -
  cloak ms (outside frames)               13.10
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    39.90
  peek_new ms (outside frames)             3.30
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.60
  to_draw ms (outside frames)              0.80
  to_alpha ms (outside frames)             2.00
  to_cloak ms (outside frames)             8.70
  peek_cover ms (outside frames)           7.70
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.10
  ring_copy ms (outside frames)            3.50
  bd_copy ms (outside frames)              0.60
  ovl_restack ms (outside frames)          2.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.80
  peek_handover ms (outside frames)       13.30
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.12
  input->first frame ms                   55.47
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             17.13
  dwm gpu busy p90 ms                      1.36
| main cpu0 bd-config bo-config #2 | resize | 40.20 | 41.97 | 62.00 | 2.23 | 3.48 | 3.00 | 5.72 | 5.76 | 37.00 | 37.00 | 1.90 | 1.25 | 0.26 | 0.13 | 0.11 | 14.96 | 0.95 | 13.78 | 10.49 | 2.09 | 5.03 | 0.00 | - | 0.03 | 2.00 | - | 13.10 | - | - | - | 39.90 | 3.30 | 2.00 | - | - | - | 12.60 | 0.80 | 2.00 | 8.70 | 7.70 | 0.00 | 1.10 | 3.50 | 0.60 | 2.00 | - | 0.80 | 13.30 | 0.20 | 0.70 | - | 6.00 | 0.00 | - | - | - | - | 3.12 | 55.47 | 5.00 | 17.13 | 1.36 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          31.25
  command p90 ms                          33.07
  frames                                  99.00
  tick ms/frame                            1.17
  frame p90 ms                             1.16
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         21.83
  interval max ms (median burst)          21.83
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            6.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.90
  peek_new ms (outside frames)             0.00
  peek_place ms (outside frames)           0.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            0.00
  to_draw ms (outside frames)              0.00
  to_alpha ms (outside frames)             0.00
  to_cloak ms (outside frames)             0.00
  peek_cover ms (outside frames)           0.00
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        0.00
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)        9.90
  ws_park ms (outside frames)              2.10
  ws_show ms (outside frames)              8.70
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                22.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      5.82
| main cpu0 bd-config bo-config #2 | switch | 31.25 | 33.07 | 99.00 | 1.17 | 1.16 | 3.00 | 5.72 | 5.76 | 21.83 | 21.83 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 6.00 | 0.00 | - | - | - | 30.90 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | 0.00 | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 9.90 | 2.10 | 8.70 | 0.40 | 0.00 | 0.00 | 22.00 | 11.44 | 5.82 |
label=main cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          39.70
  command p90 ms                          49.97
  frames                                  60.00
  tick ms/frame                            2.27
  frame p90 ms                             3.85
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         42.09
  interval max ms (median burst)          42.09
  peek_frame ms/frame                      1.75
  peek_draw ms/frame                       1.12
  peek_track ms/frame                      0.33
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.17
  peek_draw ms (worst frame)              13.97
  pd_alpha ms (worst frame)                0.82
  pd_draw ms (worst frame)                13.00
  pd_fill ms (worst frame)                 7.88
  fill_source ms (worst frame)             2.27
  ring_raise ms (worst frame)              3.58
  restack ms (worst frame)                 2.85
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.80
  cloak ms (outside frames)               12.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    39.40
  peek_new ms (outside frames)             7.70
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.50
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.60
  to_cloak ms (outside frames)            10.00
  peek_cover ms (outside frames)          13.20
  ring_prep ms (outside frames)            2.40
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            4.50
  bd_copy ms (outside frames)              0.90
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.90
  peek_handover ms (outside frames)       15.80
  peek_raise ms (outside frames)           0.50
  surface ms (outside frames)              3.00
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.35
  input->first frame ms                   55.64
  dwm missed vblanks/burst                29.00
  dwm display interval p90 ms             22.88
  dwm gpu busy p90 ms                      9.12
| main cpu0 bd-config bo-config #2 | minimize | 39.70 | 49.97 | 60.00 | 2.27 | 3.85 | 3.00 | 5.72 | 5.77 | 42.09 | 42.09 | 1.75 | 1.12 | 0.33 | 0.12 | 0.17 | 13.97 | 0.82 | 13.00 | 7.88 | 2.27 | 3.58 | 2.85 | - | 0.03 | 2.00 | 5.80 | 12.80 | - | - | - | 39.40 | 7.70 | 1.90 | - | - | - | 12.50 | 0.90 | 2.60 | 10.00 | 13.20 | 2.40 | 1.30 | 4.50 | 0.90 | 0.10 | - | 0.90 | 15.80 | 0.50 | 3.00 | - | 6.00 | 0.00 | - | - | - | - | 10.35 | 55.64 | 29.00 | 22.88 | 9.12 |
tiles restructure cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=restructure cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.24
  command p90 ms                          20.29
  frames                                  63.00
  tick ms/frame                            0.38
  frame p90 ms                             0.52
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.57
  interval max ms (median burst)          10.57
  peek_frame ms/frame                      0.37
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.69
  pd_alpha ms (worst frame)                0.25
  pd_draw ms (worst frame)                 0.60
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.20
  cloak ms (outside frames)                8.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.80
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.90
  to_draw ms (outside frames)              0.40
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             1.90
  peek_cover ms (outside frames)           1.00
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.10
  ring_copy ms (outside frames)            0.20
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.30
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        2.70
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.10
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.52
  input->first frame ms                   18.99
  dwm missed vblanks/burst                 4.00
  dwm display interval p90 ms             11.42
  dwm gpu busy p90 ms                      8.02
| restructure cpu0 bd-config bo-config #2 | move | 18.24 | 20.29 | 63.00 | 0.38 | 0.52 | 0.00 | 5.71 | 5.74 | 10.57 | 10.57 | 0.37 | 0.16 | 0.05 | 0.01 | 0.03 | 0.69 | 0.25 | 0.60 | - | - | - | - | - | - | - | 5.20 | 8.20 | - | - | - | 17.80 | 2.10 | 1.10 | - | - | - | 2.90 | 0.40 | 0.50 | 1.90 | 1.00 | 0.00 | 1.10 | 0.20 | 0.20 | 0.30 | - | 0.10 | 2.70 | 0.30 | 1.10 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.52 | 18.99 | 4.00 | 11.42 | 8.02 |
label=restructure cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          40.63
  command p90 ms                          42.39
  frames                                  61.00
  tick ms/frame                            2.26
  frame p90 ms                             3.23
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         35.74
  interval max ms (median burst)          35.74
  peek_frame ms/frame                      1.85
  peek_draw ms/frame                       1.18
  peek_track ms/frame                      0.27
  peek_ring ms/frame                       0.11
  peek_backdrop ms/frame                   0.12
  peek_draw ms (worst frame)              12.44
  pd_alpha ms (worst frame)                1.13
  pd_draw ms (worst frame)                11.20
  pd_fill ms (worst frame)                 7.94
  fill_source ms (worst frame)             1.95
  ring_raise ms (worst frame)              4.76
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)               -
  cloak ms (outside frames)               12.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    40.30
  peek_new ms (outside frames)             3.40
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.30
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             1.80
  to_cloak ms (outside frames)             8.70
  peek_cover ms (outside frames)           7.60
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.00
  ring_copy ms (outside frames)            3.50
  bd_copy ms (outside frames)              0.60
  ovl_restack ms (outside frames)          1.90
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.80
  peek_handover ms (outside frames)       13.50
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.99
  input->first frame ms                   53.70
  dwm missed vblanks/burst                22.00
  dwm display interval p90 ms             17.15
  dwm gpu busy p90 ms                      1.38
| restructure cpu0 bd-config bo-config #2 | resize | 40.63 | 42.39 | 61.00 | 2.26 | 3.23 | 3.00 | 5.71 | 5.79 | 35.74 | 35.74 | 1.85 | 1.18 | 0.27 | 0.11 | 0.12 | 12.44 | 1.13 | 11.20 | 7.94 | 1.95 | 4.76 | 0.00 | - | 0.04 | 2.00 | - | 12.70 | - | - | - | 40.30 | 3.40 | 1.90 | - | - | - | 12.30 | 0.90 | 1.80 | 8.70 | 7.60 | 0.00 | 1.00 | 3.50 | 0.60 | 1.90 | - | 0.80 | 13.50 | 0.10 | 0.70 | - | 6.00 | 0.00 | - | - | - | - | 2.99 | 53.70 | 22.00 | 17.15 | 1.38 |
Moved 3 window(s) to workspace '2' to switch to.
label=restructure cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.44
  command p90 ms                          32.86
  frames                                  99.00
  tick ms/frame                            1.14
  frame p90 ms                             1.19
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         20.62
  interval max ms (median burst)          20.62
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.20
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.20
  peek_new ms (outside frames)             0.00
  peek_place ms (outside frames)           0.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            0.00
  to_draw ms (outside frames)              0.00
  to_alpha ms (outside frames)             0.00
  to_cloak ms (outside frames)             0.00
  peek_cover ms (outside frames)           0.00
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        0.00
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)        9.60
  ws_park ms (outside frames)              2.10
  ws_show ms (outside frames)              8.00
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                16.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      1.89
| restructure cpu0 bd-config bo-config #2 | switch | 30.44 | 32.86 | 99.00 | 1.14 | 1.19 | 3.00 | 5.72 | 5.76 | 20.62 | 20.62 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.20 | 0.00 | - | - | - | 30.20 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | 0.00 | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 9.60 | 2.10 | 8.00 | 0.40 | 0.00 | 0.00 | 16.00 | 11.43 | 1.89 |
label=restructure cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          40.34
  command p90 ms                          46.56
  frames                                  61.00
  tick ms/frame                            2.18
  frame p90 ms                             3.59
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.80
  interval p99 ms                         39.59
  interval max ms (median burst)          39.59
  peek_frame ms/frame                      1.76
  peek_draw ms/frame                       1.08
  peek_track ms/frame                      0.34
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.18
  peek_draw ms (worst frame)              12.00
  pd_alpha ms (worst frame)                0.84
  pd_draw ms (worst frame)                11.07
  pd_fill ms (worst frame)                 6.92
  fill_source ms (worst frame)             2.24
  ring_raise ms (worst frame)              3.23
  restack ms (worst frame)                 2.74
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.30
  cloak ms (outside frames)               13.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    40.10
  peek_new ms (outside frames)             7.40
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.70
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.00
  to_cloak ms (outside frames)            10.00
  peek_cover ms (outside frames)          12.90
  ring_prep ms (outside frames)            3.10
  ring_park ms (outside frames)            1.40
  ring_copy ms (outside frames)            4.30
  bd_copy ms (outside frames)              0.90
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.90
  peek_handover ms (outside frames)       14.10
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              2.80
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.59
  input->first frame ms                   54.97
  dwm missed vblanks/burst                29.00
  dwm display interval p90 ms             22.89
  dwm gpu busy p90 ms                      9.85
| restructure cpu0 bd-config bo-config #2 | minimize | 40.34 | 46.56 | 61.00 | 2.18 | 3.59 | 3.00 | 5.72 | 5.80 | 39.59 | 39.59 | 1.76 | 1.08 | 0.34 | 0.12 | 0.18 | 12.00 | 0.84 | 11.07 | 6.92 | 2.24 | 3.23 | 2.74 | - | 0.03 | 2.00 | 4.30 | 13.20 | - | - | - | 40.10 | 7.40 | 1.80 | - | - | - | 12.70 | 1.00 | 2.00 | 10.00 | 12.90 | 3.10 | 1.40 | 4.30 | 0.90 | 0.10 | - | 0.90 | 14.10 | 0.30 | 2.80 | - | 6.00 | 0.00 | - | - | - | - | 8.59 | 54.97 | 29.00 | 22.89 | 9.85 |
tiles this cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.15
  command p90 ms                          18.95
  frames                                  63.00
  tick ms/frame                            0.38
  frame p90 ms                             0.52
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.62
  interval max ms (median burst)          10.62
  peek_frame ms/frame                      0.38
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.63
  pd_alpha ms (worst frame)                0.22
  pd_draw ms (worst frame)                 0.52
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)                8.50
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.90
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.70
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             1.80
  peek_cover ms (outside frames)           1.10
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.40
  ring_copy ms (outside frames)            0.30
  bd_copy ms (outside frames)              0.10
  ovl_restack ms (outside frames)          0.30
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)        3.00
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              1.10
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.58
  input->first frame ms                   18.60
  dwm missed vblanks/burst                 4.00
  dwm display interval p90 ms             11.42
  dwm gpu busy p90 ms                      7.78
| this cpu0 bd-config bo-config #2 | move | 18.15 | 18.95 | 63.00 | 0.38 | 0.52 | 0.00 | 5.71 | 5.74 | 10.62 | 10.62 | 0.38 | 0.16 | 0.05 | 0.01 | 0.03 | 0.63 | 0.22 | 0.52 | - | - | - | - | - | - | - | 5.70 | 8.50 | - | - | - | 17.90 | 2.10 | 1.10 | - | - | - | 2.70 | 0.30 | 0.50 | 1.80 | 1.10 | 0.00 | 1.40 | 0.30 | 0.10 | 0.30 | - | 0.20 | 3.00 | 0.20 | 1.10 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.58 | 18.60 | 4.00 | 11.42 | 7.78 |
label=this cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          48.40
  command p90 ms                          54.02
  frames                                  61.00
  tick ms/frame                            2.19
  frame p90 ms                             3.17
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         34.50
  interval max ms (median burst)          34.50
  peek_frame ms/frame                      1.91
  peek_draw ms/frame                       1.20
  peek_track ms/frame                      0.27
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.13
  peek_draw ms (worst frame)              15.69
  pd_alpha ms (worst frame)                0.94
  pd_draw ms (worst frame)                14.80
  pd_fill ms (worst frame)                11.46
  fill_source ms (worst frame)             2.13
  ring_raise ms (worst frame)              5.92
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.50
  cloak ms (outside frames)               19.50
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    48.10
  peek_new ms (outside frames)             3.30
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.60
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.30
  to_cloak ms (outside frames)             8.60
  peek_cover ms (outside frames)          11.10
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.80
  ring_copy ms (outside frames)            6.70
  bd_copy ms (outside frames)              0.50
  ovl_restack ms (outside frames)          2.20
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.60
  peek_handover ms (outside frames)       12.10
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.12
  input->first frame ms                   67.37
  dwm missed vblanks/burst                15.00
  dwm display interval p90 ms             17.14
  dwm gpu busy p90 ms                      1.37
| this cpu0 bd-config bo-config #2 | resize | 48.40 | 54.02 | 61.00 | 2.19 | 3.17 | 3.00 | 5.72 | 5.76 | 34.50 | 34.50 | 1.91 | 1.20 | 0.27 | 0.13 | 0.13 | 15.69 | 0.94 | 14.80 | 11.46 | 2.13 | 5.92 | 0.00 | - | 0.03 | 2.00 | 10.50 | 19.50 | - | - | - | 48.10 | 3.30 | 2.00 | - | - | - | 12.60 | 1.00 | 2.30 | 8.60 | 11.10 | 0.00 | 0.80 | 6.70 | 0.50 | 2.20 | - | 1.60 | 12.10 | 0.00 | 0.70 | - | 6.00 | 0.00 | - | - | - | - | 3.12 | 67.37 | 15.00 | 17.14 | 1.37 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          30.46
  command p90 ms                          31.10
  frames                                  99.00
  tick ms/frame                            1.15
  frame p90 ms                             1.20
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         22.06
  interval max ms (median burst)          22.06
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.20
  peek_new ms (outside frames)             0.00
  peek_place ms (outside frames)           0.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            0.00
  to_draw ms (outside frames)              0.00
  to_alpha ms (outside frames)             0.00
  to_cloak ms (outside frames)             0.00
  peek_cover ms (outside frames)           0.00
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)        0.00
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)        9.70
  ws_park ms (outside frames)              2.10
  ws_show ms (outside frames)              7.00
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                20.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      2.60
| this cpu0 bd-config bo-config #2 | switch | 30.46 | 31.10 | 99.00 | 1.15 | 1.20 | 3.00 | 5.72 | 5.76 | 22.06 | 22.06 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.04 | - | 4.50 | 0.00 | - | - | - | 30.20 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.20 | 0.00 | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 9.70 | 2.10 | 7.00 | 0.40 | 0.00 | 0.00 | 20.00 | 11.43 | 2.60 |
label=this cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          43.17
  command p90 ms                          49.94
  frames                                  62.00
  tick ms/frame                            2.32
  frame p90 ms                             3.80
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.81
  interval p99 ms                         44.04
  interval max ms (median burst)          44.04
  peek_frame ms/frame                      1.91
  peek_draw ms/frame                       1.15
  peek_track ms/frame                      0.34
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.21
  peek_draw ms (worst frame)              12.17
  pd_alpha ms (worst frame)                0.84
  pd_draw ms (worst frame)                11.31
  pd_fill ms (worst frame)                 7.67
  fill_source ms (worst frame)             2.21
  ring_raise ms (worst frame)              3.46
  restack ms (worst frame)                 2.84
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.00
  cloak ms (outside frames)               15.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    42.90
  peek_new ms (outside frames)             8.40
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           10.30
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.20
  to_cloak ms (outside frames)             7.20
  peek_cover ms (outside frames)          14.50
  ring_prep ms (outside frames)            2.20
  ring_park ms (outside frames)            1.00
  ring_copy ms (outside frames)            7.30
  bd_copy ms (outside frames)              0.90
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.40
  peek_handover ms (outside frames)       14.50
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              3.10
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.67
  input->first frame ms                   58.67
  dwm missed vblanks/burst                32.00
  dwm display interval p90 ms             22.88
  dwm gpu busy p90 ms                      9.68
| this cpu0 bd-config bo-config #2 | minimize | 43.17 | 49.94 | 62.00 | 2.32 | 3.80 | 3.00 | 5.72 | 5.81 | 44.04 | 44.04 | 1.91 | 1.15 | 0.34 | 0.13 | 0.21 | 12.17 | 0.84 | 11.31 | 7.67 | 2.21 | 3.46 | 2.84 | - | 0.03 | 2.00 | 9.00 | 15.80 | - | - | - | 42.90 | 8.40 | 1.80 | - | - | - | 10.30 | 0.90 | 2.20 | 7.20 | 14.50 | 2.20 | 1.00 | 7.30 | 0.90 | 0.10 | - | 1.40 | 14.50 | 0.30 | 3.10 | - | 6.00 | 0.00 | - | - | - | - | 8.67 | 58.67 | 32.00 | 22.88 | 9.68 |
tiles restructure cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=restructure cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.69
  command p90 ms                          18.24
  frames                                  63.00
  tick ms/frame                            0.38
  frame p90 ms                             0.54
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.54
  interval max ms (median burst)          10.54
  peek_frame ms/frame                      0.37
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.04
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.61
  pd_alpha ms (worst frame)                0.25
  pd_draw ms (worst frame)                 0.54
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.50
  cloak ms (outside frames)                8.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.30
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.60
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.40
  to_cloak ms (outside frames)             1.90
  peek_cover ms (outside frames)           1.00
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            0.20
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.30
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        2.60
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.00
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.54
  input->first frame ms                   17.94
  dwm missed vblanks/burst                 4.00
  dwm display interval p90 ms             11.42
  dwm gpu busy p90 ms                      7.58
| restructure cpu0 bd-config bo-config #3 | move | 17.69 | 18.24 | 63.00 | 0.38 | 0.54 | 0.00 | 5.71 | 5.74 | 10.54 | 10.54 | 0.37 | 0.16 | 0.04 | 0.01 | 0.03 | 0.61 | 0.25 | 0.54 | - | - | - | - | - | - | - | 5.50 | 8.20 | - | - | - | 17.30 | 2.10 | 1.10 | - | - | - | 2.60 | 0.30 | 0.40 | 1.90 | 1.00 | 0.00 | 1.20 | 0.20 | 0.20 | 0.30 | - | 0.10 | 2.60 | 0.30 | 1.00 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.54 | 17.94 | 4.00 | 11.42 | 7.58 |
label=restructure cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          40.65
  command p90 ms                          43.69
  frames                                  61.00
  tick ms/frame                            2.32
  frame p90 ms                             3.70
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         37.60
  interval max ms (median burst)          37.60
  peek_frame ms/frame                      2.01
  peek_draw ms/frame                       1.33
  peek_track ms/frame                      0.27
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.10
  peek_draw ms (worst frame)              16.72
  pd_alpha ms (worst frame)                1.08
  pd_draw ms (worst frame)                15.74
  pd_fill ms (worst frame)                12.17
  fill_source ms (worst frame)             3.60
  ring_raise ms (worst frame)              6.07
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)               -
  cloak ms (outside frames)               13.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    40.30
  peek_new ms (outside frames)             3.20
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.10
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.20
  to_cloak ms (outside frames)             8.60
  peek_cover ms (outside frames)           8.20
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.10
  ring_copy ms (outside frames)            3.80
  bd_copy ms (outside frames)              0.60
  ovl_restack ms (outside frames)          2.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.50
  peek_handover ms (outside frames)       12.90
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.90
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.92
  input->first frame ms                   57.33
  dwm missed vblanks/burst                22.00
  dwm display interval p90 ms             17.16
  dwm gpu busy p90 ms                      1.52
| restructure cpu0 bd-config bo-config #3 | resize | 40.65 | 43.69 | 61.00 | 2.32 | 3.70 | 3.00 | 5.71 | 5.78 | 37.60 | 37.60 | 2.01 | 1.33 | 0.27 | 0.12 | 0.10 | 16.72 | 1.08 | 15.74 | 12.17 | 3.60 | 6.07 | 0.00 | - | 0.04 | 2.00 | - | 13.30 | - | - | - | 40.30 | 3.20 | 2.00 | - | - | - | 13.10 | 1.00 | 2.20 | 8.60 | 8.20 | 0.00 | 1.10 | 3.80 | 0.60 | 2.00 | - | 1.50 | 12.90 | 0.10 | 0.90 | - | 6.00 | 0.00 | - | - | - | - | 2.92 | 57.33 | 22.00 | 17.16 | 1.52 |
Moved 3 window(s) to workspace '2' to switch to.
label=restructure cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          30.27
  command p90 ms                          33.15
  frames                                  99.00
  tick ms/frame                            1.15
  frame p90 ms                             1.28
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.75
  interval p99 ms                         21.40
  interval max ms (median burst)          21.40
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.60
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.00
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)               -
  to_draw ms (outside frames)                 -
  to_alpha ms (outside frames)                -
  to_cloak ms (outside frames)                -
  peek_cover ms (outside frames)              -
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)        9.80
  ws_park ms (outside frames)              2.10
  ws_show ms (outside frames)              7.20
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                14.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      5.59
| restructure cpu0 bd-config bo-config #3 | switch | 30.27 | 33.15 | 99.00 | 1.15 | 1.28 | 3.00 | 5.72 | 5.75 | 21.40 | 21.40 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.00 | - | - | 0.02 | - | 4.60 | - | - | - | - | 30.00 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.20 | - | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 9.80 | 2.10 | 7.20 | 0.40 | - | - | 14.00 | 11.44 | 5.59 |
label=restructure cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          38.12
  command p90 ms                          47.32
  frames                                  62.00
  tick ms/frame                            2.37
  frame p90 ms                             3.94
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         36.50
  interval max ms (median burst)          36.50
  peek_frame ms/frame                      1.87
  peek_draw ms/frame                       1.17
  peek_track ms/frame                      0.32
  peek_ring ms/frame                       0.11
  peek_backdrop ms/frame                   0.17
  peek_draw ms (worst frame)              15.17
  pd_alpha ms (worst frame)                0.71
  pd_draw ms (worst frame)                14.45
  pd_fill ms (worst frame)                11.05
  fill_source ms (worst frame)             3.20
  ring_raise ms (worst frame)              4.35
  restack ms (worst frame)                 2.79
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)               13.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    37.80
  peek_new ms (outside frames)             6.70
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.70
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.10
  to_cloak ms (outside frames)             9.70
  peek_cover ms (outside frames)          12.80
  ring_prep ms (outside frames)            4.10
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            4.00
  bd_copy ms (outside frames)              0.90
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.00
  peek_handover ms (outside frames)       10.90
  peek_raise ms (outside frames)           0.50
  surface ms (outside frames)              2.90
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.18
  input->first frame ms                   54.00
  dwm missed vblanks/burst                32.00
  dwm display interval p90 ms             22.88
  dwm gpu busy p90 ms                      9.42
| restructure cpu0 bd-config bo-config #3 | minimize | 38.12 | 47.32 | 62.00 | 2.37 | 3.94 | 3.00 | 5.71 | 5.79 | 36.50 | 36.50 | 1.87 | 1.17 | 0.32 | 0.11 | 0.17 | 15.17 | 0.71 | 14.45 | 11.05 | 3.20 | 4.35 | 2.79 | - | 0.03 | 2.00 | 4.90 | 13.00 | - | - | - | 37.80 | 6.70 | 1.80 | - | - | - | 12.70 | 0.90 | 2.10 | 9.70 | 12.80 | 4.10 | 1.30 | 4.00 | 0.90 | 0.10 | - | 1.00 | 10.90 | 0.50 | 2.90 | - | 6.00 | 0.00 | - | - | - | - | 9.18 | 54.00 | 32.00 | 22.88 | 9.42 |
tiles this cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.29
  command p90 ms                          19.24
  frames                                  63.00
  tick ms/frame                            0.38
  frame p90 ms                             0.55
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.52
  interval max ms (median burst)          10.52
  peek_frame ms/frame                      0.33
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.64
  pd_alpha ms (worst frame)                0.22
  pd_draw ms (worst frame)                 0.54
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)                8.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.00
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.60
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.40
  to_cloak ms (outside frames)             1.80
  peek_cover ms (outside frames)           1.20
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.00
  ring_copy ms (outside frames)            0.30
  bd_copy ms (outside frames)              0.10
  ovl_restack ms (outside frames)          0.30
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)        2.60
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              1.10
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.59
  input->first frame ms                   18.69
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.42
  dwm gpu busy p90 ms                      8.00
| this cpu0 bd-config bo-config #3 | move | 18.29 | 19.24 | 63.00 | 0.38 | 0.55 | 0.00 | 5.71 | 5.74 | 10.52 | 10.52 | 0.33 | 0.16 | 0.05 | 0.01 | 0.03 | 0.64 | 0.22 | 0.54 | - | - | - | - | - | - | - | 5.40 | 8.20 | - | - | - | 18.00 | 2.10 | 1.10 | - | - | - | 2.60 | 0.30 | 0.40 | 1.80 | 1.20 | 0.00 | 1.00 | 0.30 | 0.10 | 0.30 | - | 0.20 | 2.60 | 0.20 | 1.10 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.59 | 18.69 | 5.00 | 11.42 | 8.00 |
label=this cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          54.59
  command p90 ms                          56.29
  frames                                  61.00
  tick ms/frame                            2.40
  frame p90 ms                             3.25
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         35.09
  interval max ms (median burst)          35.09
  peek_frame ms/frame                      1.84
  peek_draw ms/frame                       1.25
  peek_track ms/frame                      0.29
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.13
  peek_draw ms (worst frame)              17.43
  pd_alpha ms (worst frame)                1.06
  pd_draw ms (worst frame)                16.23
  pd_fill ms (worst frame)                12.90
  fill_source ms (worst frame)             3.37
  ring_raise ms (worst frame)              5.80
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.00
  cloak ms (outside frames)               24.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    54.30
  peek_new ms (outside frames)             3.30
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.00
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.10
  to_cloak ms (outside frames)             9.00
  peek_cover ms (outside frames)          11.20
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.90
  ring_copy ms (outside frames)            6.70
  bd_copy ms (outside frames)              0.50
  ovl_restack ms (outside frames)          2.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.20
  peek_handover ms (outside frames)       12.60
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.10
  input->first frame ms                   66.92
  dwm missed vblanks/burst                12.00
  dwm display interval p90 ms             17.14
  dwm gpu busy p90 ms                      1.26
| this cpu0 bd-config bo-config #3 | resize | 54.59 | 56.29 | 61.00 | 2.40 | 3.25 | 3.00 | 5.71 | 5.77 | 35.09 | 35.09 | 1.84 | 1.25 | 0.29 | 0.14 | 0.13 | 17.43 | 1.06 | 16.23 | 12.90 | 3.37 | 5.80 | 0.00 | - | 0.04 | 2.00 | 11.00 | 24.00 | - | - | - | 54.30 | 3.30 | 2.00 | - | - | - | 13.00 | 0.90 | 2.10 | 9.00 | 11.20 | 0.00 | 0.90 | 6.70 | 0.50 | 2.10 | - | 1.20 | 12.60 | 0.10 | 0.70 | - | 6.00 | 0.00 | - | - | - | - | 3.10 | 66.92 | 12.00 | 17.14 | 1.26 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          30.64
  command p90 ms                          33.25
  frames                                  99.00
  tick ms/frame                            1.17
  frame p90 ms                             1.06
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         22.65
  interval max ms (median burst)          22.69
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.90
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.40
  peek_new ms (outside frames)             0.00
  peek_place ms (outside frames)           0.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            0.00
  to_draw ms (outside frames)              0.00
  to_alpha ms (outside frames)             0.00
  to_cloak ms (outside frames)             0.00
  peek_cover ms (outside frames)           0.00
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)        0.00
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.70
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)        9.70
  ws_park ms (outside frames)              2.10
  ws_show ms (outside frames)              7.90
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                13.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      2.14
| this cpu0 bd-config bo-config #3 | switch | 30.64 | 33.25 | 99.00 | 1.17 | 1.06 | 4.00 | 5.71 | 5.76 | 22.65 | 22.69 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.90 | 0.00 | - | - | - | 30.40 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.20 | 0.00 | 0.00 | 0.70 | 0.00 | 0.00 | 0.00 | 9.70 | 2.10 | 7.90 | 0.40 | 0.00 | 0.00 | 13.00 | 11.44 | 2.14 |
label=this cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          46.06
  command p90 ms                          48.98
  frames                                  62.00
  tick ms/frame                            2.24
  frame p90 ms                             3.48
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         46.04
  interval max ms (median burst)          46.04
  peek_frame ms/frame                      1.77
  peek_draw ms/frame                       1.12
  peek_track ms/frame                      0.33
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.20
  peek_draw ms (worst frame)              10.79
  pd_alpha ms (worst frame)                0.70
  pd_draw ms (worst frame)                 9.96
  pd_fill ms (worst frame)                 6.71
  fill_source ms (worst frame)             1.97
  ring_raise ms (worst frame)              3.20
  restack ms (worst frame)                 2.91
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.70
  cloak ms (outside frames)               16.50
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    45.80
  peek_new ms (outside frames)             8.50
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           10.20
  to_draw ms (outside frames)              0.80
  to_alpha ms (outside frames)             1.60
  to_cloak ms (outside frames)             7.60
  peek_cover ms (outside frames)          18.40
  ring_prep ms (outside frames)            2.80
  ring_park ms (outside frames)            1.10
  ring_copy ms (outside frames)            7.40
  bd_copy ms (outside frames)              0.90
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.70
  peek_handover ms (outside frames)       14.40
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              2.30
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.51
  input->first frame ms                   60.61
  dwm missed vblanks/burst                27.00
  dwm display interval p90 ms             22.87
  dwm gpu busy p90 ms                      9.46
| this cpu0 bd-config bo-config #3 | minimize | 46.06 | 48.98 | 62.00 | 2.24 | 3.48 | 3.00 | 5.72 | 5.78 | 46.04 | 46.04 | 1.77 | 1.12 | 0.33 | 0.12 | 0.20 | 10.79 | 0.70 | 9.96 | 6.71 | 1.97 | 3.20 | 2.91 | - | 0.03 | 2.00 | 9.70 | 16.50 | - | - | - | 45.80 | 8.50 | 1.80 | - | - | - | 10.20 | 0.80 | 1.60 | 7.60 | 18.40 | 2.80 | 1.10 | 7.40 | 0.90 | 0.00 | - | 1.70 | 14.40 | 0.30 | 2.30 | - | 6.00 | 0.00 | - | - | - | - | 8.51 | 60.61 | 27.00 | 22.87 | 9.46 |
tiles main cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.81
  command p90 ms                          18.88
  frames                                  63.00
  tick ms/frame                            0.39
  frame p90 ms                             0.54
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.55
  interval max ms (median burst)          10.55
  peek_frame ms/frame                      0.34
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.62
  pd_alpha ms (worst frame)                0.26
  pd_draw ms (worst frame)                 0.52
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)                7.10
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.50
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.70
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             1.80
  peek_cover ms (outside frames)           1.30
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            0.10
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        2.50
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.10
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.57
  input->first frame ms                   18.78
  dwm missed vblanks/burst                 4.00
  dwm display interval p90 ms             11.42
  dwm gpu busy p90 ms                      8.01
| main cpu0 bd-config bo-config #3 | move | 17.81 | 18.88 | 63.00 | 0.39 | 0.54 | 0.00 | 5.71 | 5.74 | 10.55 | 10.55 | 0.34 | 0.16 | 0.05 | 0.01 | 0.03 | 0.62 | 0.26 | 0.52 | - | - | - | - | - | - | - | 4.40 | 7.10 | - | - | - | 17.50 | 2.10 | 1.20 | - | - | - | 2.70 | 0.30 | 0.50 | 1.80 | 1.30 | 0.00 | 1.20 | 0.10 | 0.20 | 0.60 | - | 0.10 | 2.50 | 0.30 | 1.10 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.57 | 18.78 | 4.00 | 11.42 | 8.01 |
label=main cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          41.93
  command p90 ms                          48.48
  frames                                  61.00
  tick ms/frame                            2.31
  frame p90 ms                             3.25
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         45.97
  interval max ms (median burst)          45.97
  peek_frame ms/frame                      1.95
  peek_draw ms/frame                       1.24
  peek_track ms/frame                      0.29
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.12
  peek_draw ms (worst frame)              15.84
  pd_alpha ms (worst frame)                0.95
  pd_draw ms (worst frame)                15.03
  pd_fill ms (worst frame)                11.68
  fill_source ms (worst frame)             3.85
  ring_raise ms (worst frame)              5.35
  restack ms (worst frame)                 3.02
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)               -
  cloak ms (outside frames)               14.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    41.60
  peek_new ms (outside frames)             3.30
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           14.60
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             3.00
  to_cloak ms (outside frames)             9.70
  peek_cover ms (outside frames)           7.90
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            3.50
  bd_copy ms (outside frames)              0.60
  ovl_restack ms (outside frames)          2.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.30
  peek_handover ms (outside frames)       12.40
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.94
  input->first frame ms                   57.25
  dwm missed vblanks/burst                12.00
  dwm display interval p90 ms             17.15
  dwm gpu busy p90 ms                      1.36
| main cpu0 bd-config bo-config #3 | resize | 41.93 | 48.48 | 61.00 | 2.31 | 3.25 | 3.00 | 5.72 | 5.78 | 45.97 | 45.97 | 1.95 | 1.24 | 0.29 | 0.13 | 0.12 | 15.84 | 0.95 | 15.03 | 11.68 | 3.85 | 5.35 | 3.02 | - | 0.04 | 2.00 | - | 14.80 | - | - | - | 41.60 | 3.30 | 1.90 | - | - | - | 14.60 | 0.90 | 3.00 | 9.70 | 7.90 | 0.00 | 1.20 | 3.50 | 0.60 | 2.00 | - | 1.30 | 12.40 | 0.10 | 0.70 | - | 6.00 | 0.00 | - | - | - | - | 2.94 | 57.25 | 12.00 | 17.15 | 1.36 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          31.23
  command p90 ms                          33.43
  frames                                  99.00
  tick ms/frame                            1.16
  frame p90 ms                             1.12
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         21.60
  interval max ms (median burst)          21.60
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            6.00
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.90
  peek_new ms (outside frames)             0.00
  peek_place ms (outside frames)           0.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            0.00
  to_draw ms (outside frames)              0.00
  to_alpha ms (outside frames)             0.00
  to_cloak ms (outside frames)             0.00
  peek_cover ms (outside frames)           0.00
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        0.00
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.70
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)        9.70
  ws_park ms (outside frames)              2.10
  ws_show ms (outside frames)              8.50
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                22.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      5.52
| main cpu0 bd-config bo-config #3 | switch | 31.23 | 33.43 | 99.00 | 1.16 | 1.12 | 3.00 | 5.71 | 5.76 | 21.60 | 21.60 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 6.00 | 0.00 | - | - | - | 30.90 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | 0.00 | 0.00 | 0.70 | 0.00 | 0.00 | 0.00 | 9.70 | 2.10 | 8.50 | 0.40 | 0.00 | 0.00 | 22.00 | 11.44 | 5.52 |
label=main cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          39.29
  command p90 ms                          46.62
  frames                                  61.00
  tick ms/frame                            2.32
  frame p90 ms                             3.83
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  interval p99 ms                         40.31
  interval max ms (median burst)          40.31
  peek_frame ms/frame                      1.81
  peek_draw ms/frame                       1.19
  peek_track ms/frame                      0.32
  peek_ring ms/frame                       0.11
  peek_backdrop ms/frame                   0.17
  peek_draw ms (worst frame)              14.12
  pd_alpha ms (worst frame)                0.85
  pd_draw ms (worst frame)                13.44
  pd_fill ms (worst frame)                 9.61
  fill_source ms (worst frame)             3.95
  ring_raise ms (worst frame)              3.11
  restack ms (worst frame)                 3.08
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.30
  cloak ms (outside frames)               13.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    39.00
  peek_new ms (outside frames)             7.00
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.60
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.50
  to_cloak ms (outside frames)             9.50
  peek_cover ms (outside frames)          13.40
  ring_prep ms (outside frames)            3.30
  ring_park ms (outside frames)            1.10
  ring_copy ms (outside frames)            4.30
  bd_copy ms (outside frames)              0.90
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.20
  peek_handover ms (outside frames)       14.00
  peek_raise ms (outside frames)           0.50
  surface ms (outside frames)              3.10
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.56
  input->first frame ms                   57.28
  dwm missed vblanks/burst                30.00
  dwm display interval p90 ms             22.90
  dwm gpu busy p90 ms                      8.94
| main cpu0 bd-config bo-config #3 | minimize | 39.29 | 46.62 | 61.00 | 2.32 | 3.83 | 3.00 | 5.72 | 5.79 | 40.31 | 40.31 | 1.81 | 1.19 | 0.32 | 0.11 | 0.17 | 14.12 | 0.85 | 13.44 | 9.61 | 3.95 | 3.11 | 3.08 | - | 0.03 | 2.00 | 4.30 | 13.00 | - | - | - | 39.00 | 7.00 | 1.90 | - | - | - | 12.60 | 0.90 | 2.50 | 9.50 | 13.40 | 3.30 | 1.10 | 4.30 | 0.90 | 0.10 | - | 1.20 | 14.00 | 0.50 | 3.10 | - | 6.00 | 0.00 | - | - | - | - | 8.56 | 57.28 | 30.00 | 22.90 | 8.94 |
tiles this cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.02
  command p90 ms                          19.89
  frames                                  63.00
  tick ms/frame                            0.43
  frame p90 ms                             0.53
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.67
  interval max ms (median burst)          12.67
  peek_frame ms/frame                      0.34
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.06
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.63
  pd_alpha ms (worst frame)                0.29
  pd_draw ms (worst frame)                 0.48
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)                7.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.70
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.90
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             2.00
  peek_cover ms (outside frames)           1.50
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.50
  ring_copy ms (outside frames)            0.30
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.80
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.30
  peek_handover ms (outside frames)        3.10
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              1.30
  restack ms (outside frames)              0.90
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.45
  input->first frame ms                   20.21
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.11
| this cpu8 bd-config bo-config #1 | move | 19.02 | 19.89 | 63.00 | 0.43 | 0.53 | 0.00 | 5.71 | 5.76 | 12.67 | 12.67 | 0.34 | 0.17 | 0.06 | 0.01 | 0.04 | 0.63 | 0.29 | 0.48 | - | - | - | - | - | - | - | 4.70 | 7.60 | - | - | - | 18.70 | 2.10 | 1.20 | - | - | - | 2.90 | 0.30 | 0.50 | 2.00 | 1.50 | 0.00 | 1.50 | 0.30 | 0.20 | 0.80 | - | 0.30 | 3.10 | 0.40 | 1.30 | 0.90 | 2.00 | 0.00 | - | - | - | - | 3.45 | 20.21 | 5.00 | 11.43 | 8.11 |
label=this cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          57.34
  command p90 ms                          63.87
  frames                                  60.00
  tick ms/frame                            2.78
  frame p90 ms                             4.26
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         41.12
  interval max ms (median burst)          41.12
  peek_frame ms/frame                      2.13
  peek_draw ms/frame                       1.54
  peek_track ms/frame                      0.32
  peek_ring ms/frame                       0.15
  peek_backdrop ms/frame                   0.15
  peek_draw ms (worst frame)              21.61
  pd_alpha ms (worst frame)                1.13
  pd_draw ms (worst frame)                20.40
  pd_fill ms (worst frame)                16.15
  fill_source ms (worst frame)             6.87
  ring_raise ms (worst frame)              5.58
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.50
  cloak ms (outside frames)               23.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    56.90
  peek_new ms (outside frames)             4.10
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.50
  to_draw ms (outside frames)              1.10
  to_alpha ms (outside frames)             1.90
  to_cloak ms (outside frames)             9.40
  peek_cover ms (outside frames)          12.70
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            7.20
  bd_copy ms (outside frames)              0.60
  ovl_restack ms (outside frames)          2.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.20
  peek_handover ms (outside frames)       14.60
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.90
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.59
  input->first frame ms                   80.82
  dwm missed vblanks/burst                26.00
  dwm display interval p90 ms             17.17
  dwm gpu busy p90 ms                      1.42
| this cpu8 bd-config bo-config #1 | resize | 57.34 | 63.87 | 60.00 | 2.78 | 4.26 | 3.00 | 5.71 | 5.77 | 41.12 | 41.12 | 2.13 | 1.54 | 0.32 | 0.15 | 0.15 | 21.61 | 1.13 | 20.40 | 16.15 | 6.87 | 5.58 | 0.00 | - | 0.04 | 2.00 | 9.50 | 23.00 | - | - | - | 56.90 | 4.10 | 2.10 | - | - | - | 13.50 | 1.10 | 1.90 | 9.40 | 12.70 | 0.00 | 1.30 | 7.20 | 0.60 | 2.60 | - | 1.20 | 14.60 | 0.10 | 0.90 | - | 6.00 | 0.00 | - | - | - | - | 3.59 | 80.82 | 26.00 | 17.17 | 1.42 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.87
  command p90 ms                          35.84
  frames                                  99.00
  tick ms/frame                            1.27
  frame p90 ms                             1.25
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         22.23
  interval max ms (median burst)          24.09
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            6.60
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    34.60
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)               -
  to_draw ms (outside frames)                 -
  to_alpha ms (outside frames)                -
  to_cloak ms (outside frames)                -
  peek_cover ms (outside frames)              -
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.70
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)       11.60
  ws_park ms (outside frames)              2.30
  ws_show ms (outside frames)              9.20
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                24.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      1.97
| this cpu8 bd-config bo-config #1 | switch | 34.87 | 35.84 | 99.00 | 1.27 | 1.25 | 3.00 | 5.71 | 5.76 | 22.23 | 24.09 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.04 | - | 6.60 | - | - | - | - | 34.60 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.20 | - | 0.00 | 0.70 | 0.00 | 0.00 | 0.00 | 11.60 | 2.30 | 9.20 | 0.40 | - | - | 24.00 | 11.44 | 1.97 |
label=this cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          47.71
  command p90 ms                          62.76
  frames                                  61.00
  tick ms/frame                            2.48
  frame p90 ms                             4.70
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         51.73
  interval max ms (median burst)          51.73
  peek_frame ms/frame                      2.07
  peek_draw ms/frame                       1.29
  peek_track ms/frame                      0.36
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.21
  peek_draw ms (worst frame)              17.95
  pd_alpha ms (worst frame)                0.95
  pd_draw ms (worst frame)                16.91
  pd_fill ms (worst frame)                12.55
  fill_source ms (worst frame)             5.58
  ring_raise ms (worst frame)              3.35
  restack ms (worst frame)                 2.72
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           13.70
  cloak ms (outside frames)               21.10
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    47.30
  peek_new ms (outside frames)            10.40
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.20
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.60
  to_cloak ms (outside frames)             8.50
  peek_cover ms (outside frames)          16.00
  ring_prep ms (outside frames)            2.70
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            7.90
  bd_copy ms (outside frames)              1.00
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.70
  peek_handover ms (outside frames)       12.30
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              3.00
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.63
  input->first frame ms                   68.49
  dwm missed vblanks/burst                31.00
  dwm display interval p90 ms             28.58
  dwm gpu busy p90 ms                      9.43
| this cpu8 bd-config bo-config #1 | minimize | 47.71 | 62.76 | 61.00 | 2.48 | 4.70 | 3.00 | 5.71 | 5.78 | 51.73 | 51.73 | 2.07 | 1.29 | 0.36 | 0.14 | 0.21 | 17.95 | 0.95 | 16.91 | 12.55 | 5.58 | 3.35 | 2.72 | - | 0.02 | 2.00 | 13.70 | 21.10 | - | - | - | 47.30 | 10.40 | 1.90 | - | - | - | 12.20 | 0.90 | 2.60 | 8.50 | 16.00 | 2.70 | 1.30 | 7.90 | 1.00 | 0.10 | - | 1.70 | 12.30 | 0.30 | 3.00 | - | 6.00 | 0.00 | - | - | - | - | 8.63 | 68.49 | 31.00 | 28.58 | 9.43 |
tiles main cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.28
  command p90 ms                          20.33
  frames                                  63.00
  tick ms/frame                            0.43
  frame p90 ms                             0.51
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.52
  interval max ms (median burst)          12.52
  peek_frame ms/frame                      0.31
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.69
  pd_alpha ms (worst frame)                0.31
  pd_draw ms (worst frame)                 0.50
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)                7.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    19.00
  peek_new ms (outside frames)             2.30
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.90
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             2.10
  peek_cover ms (outside frames)           1.10
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.50
  ring_copy ms (outside frames)            0.20
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.40
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        3.20
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.30
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.66
  input->first frame ms                   19.87
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      8.49
| main cpu8 bd-config bo-config #1 | move | 19.28 | 20.33 | 63.00 | 0.43 | 0.51 | 0.00 | 5.71 | 5.76 | 12.52 | 12.52 | 0.31 | 0.17 | 0.05 | 0.01 | 0.03 | 0.69 | 0.31 | 0.50 | - | - | - | - | - | - | - | 4.90 | 7.80 | - | - | - | 19.00 | 2.30 | 1.20 | - | - | - | 2.90 | 0.30 | 0.50 | 2.10 | 1.10 | 0.00 | 1.50 | 0.20 | 0.20 | 0.40 | - | 0.10 | 3.20 | 0.30 | 1.30 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.66 | 19.87 | 5.00 | 11.44 | 8.49 |
label=main cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          42.93
  command p90 ms                          46.81
  frames                                  61.00
  tick ms/frame                            2.74
  frame p90 ms                             4.04
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         42.66
  interval max ms (median burst)          42.66
  peek_frame ms/frame                      2.18
  peek_draw ms/frame                       1.49
  peek_track ms/frame                      0.31
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.13
  peek_draw ms (worst frame)              21.32
  pd_alpha ms (worst frame)                1.20
  pd_draw ms (worst frame)                20.11
  pd_fill ms (worst frame)                16.21
  fill_source ms (worst frame)             5.16
  ring_raise ms (worst frame)              5.48
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)               -
  cloak ms (outside frames)               14.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    42.60
  peek_new ms (outside frames)             3.80
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           14.20
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.20
  to_cloak ms (outside frames)             9.40
  peek_cover ms (outside frames)           8.70
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.10
  ring_copy ms (outside frames)            4.00
  bd_copy ms (outside frames)              0.60
  ovl_restack ms (outside frames)          2.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.40
  peek_handover ms (outside frames)       13.50
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              1.00
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.53
  input->first frame ms                   65.26
  dwm missed vblanks/burst                24.00
  dwm display interval p90 ms             17.17
  dwm gpu busy p90 ms                      1.54
| main cpu8 bd-config bo-config #1 | resize | 42.93 | 46.81 | 61.00 | 2.74 | 4.04 | 3.00 | 5.71 | 5.78 | 42.66 | 42.66 | 2.18 | 1.49 | 0.31 | 0.14 | 0.13 | 21.32 | 1.20 | 20.11 | 16.21 | 5.16 | 5.48 | 0.00 | - | 0.04 | 2.00 | - | 14.60 | - | - | - | 42.60 | 3.80 | 2.10 | - | - | - | 14.20 | 1.00 | 2.20 | 9.40 | 8.70 | 0.00 | 1.10 | 4.00 | 0.60 | 2.00 | - | 1.40 | 13.50 | 0.20 | 1.00 | - | 6.00 | 0.00 | - | - | - | - | 3.53 | 65.26 | 24.00 | 17.17 | 1.54 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          32.89
  command p90 ms                          36.24
  frames                                  99.00
  tick ms/frame                            1.27
  frame p90 ms                             1.25
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         24.32
  interval max ms (median burst)          24.32
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.80
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    32.60
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)               -
  to_draw ms (outside frames)                 -
  to_alpha ms (outside frames)                -
  to_cloak ms (outside frames)                -
  peek_cover ms (outside frames)              -
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.70
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)       11.20
  ws_park ms (outside frames)              2.40
  ws_show ms (outside frames)              8.40
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                25.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      2.75
| main cpu8 bd-config bo-config #1 | switch | 32.89 | 36.24 | 99.00 | 1.27 | 1.25 | 3.00 | 5.71 | 5.76 | 24.32 | 24.32 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.80 | - | - | - | - | 32.60 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.20 | - | 0.00 | 0.70 | 0.00 | 0.00 | 0.00 | 11.20 | 2.40 | 8.40 | 0.40 | - | - | 25.00 | 11.44 | 2.75 |
label=main cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          46.14
  command p90 ms                          48.89
  frames                                  61.00
  tick ms/frame                            2.39
  frame p90 ms                             4.21
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         45.01
  interval max ms (median burst)          45.01
  peek_frame ms/frame                      2.05
  peek_draw ms/frame                       1.25
  peek_track ms/frame                      0.33
  peek_ring ms/frame                       0.11
  peek_backdrop ms/frame                   0.17
  peek_draw ms (worst frame)              16.98
  pd_alpha ms (worst frame)                1.18
  pd_draw ms (worst frame)                15.79
  pd_fill ms (worst frame)                11.75
  fill_source ms (worst frame)             2.91
  ring_raise ms (worst frame)              5.70
  restack ms (worst frame)                 3.26
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.00
  cloak ms (outside frames)               14.10
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    45.80
  peek_new ms (outside frames)             9.10
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.70
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.60
  to_cloak ms (outside frames)            11.20
  peek_cover ms (outside frames)          14.90
  ring_prep ms (outside frames)            4.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            4.40
  bd_copy ms (outside frames)              1.00
  ovl_restack ms (outside frames)          0.20
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.30
  peek_handover ms (outside frames)       15.10
  peek_raise ms (outside frames)           0.50
  surface ms (outside frames)              3.40
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.35
  input->first frame ms                   66.35
  dwm missed vblanks/burst                37.00
  dwm display interval p90 ms             28.58
  dwm gpu busy p90 ms                      9.67
| main cpu8 bd-config bo-config #1 | minimize | 46.14 | 48.89 | 61.00 | 2.39 | 4.21 | 3.00 | 5.71 | 5.78 | 45.01 | 45.01 | 2.05 | 1.25 | 0.33 | 0.11 | 0.17 | 16.98 | 1.18 | 15.79 | 11.75 | 2.91 | 5.70 | 3.26 | - | 0.02 | 2.00 | 4.00 | 14.10 | - | - | - | 45.80 | 9.10 | 1.80 | - | - | - | 13.70 | 1.00 | 2.60 | 11.20 | 14.90 | 4.00 | 1.20 | 4.40 | 1.00 | 0.20 | - | 1.30 | 15.10 | 0.50 | 3.40 | - | 6.00 | 0.00 | - | - | - | - | 8.35 | 66.35 | 37.00 | 28.58 | 9.67 |
tiles restructure cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=restructure cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.50
  command p90 ms                          19.68
  frames                                  63.00
  tick ms/frame                            0.42
  frame p90 ms                             0.53
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.39
  interval max ms (median burst)          12.39
  peek_frame ms/frame                      0.29
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.62
  pd_alpha ms (worst frame)                0.28
  pd_draw ms (worst frame)                 0.49
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)                7.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.20
  peek_new ms (outside frames)             2.20
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.80
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             2.00
  peek_cover ms (outside frames)           1.20
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.50
  ring_copy ms (outside frames)            0.20
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.40
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        3.30
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.20
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.76
  input->first frame ms                   19.01
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.46
| restructure cpu8 bd-config bo-config #1 | move | 18.50 | 19.68 | 63.00 | 0.42 | 0.53 | 0.00 | 5.71 | 5.76 | 12.39 | 12.39 | 0.29 | 0.17 | 0.05 | 0.01 | 0.03 | 0.62 | 0.28 | 0.49 | - | - | - | - | - | - | - | 4.70 | 7.80 | - | - | - | 18.20 | 2.20 | 1.20 | - | - | - | 2.80 | 0.30 | 0.50 | 2.00 | 1.20 | 0.00 | 1.50 | 0.20 | 0.20 | 0.40 | - | 0.10 | 3.30 | 0.30 | 1.20 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.76 | 19.01 | 5.00 | 11.43 | 8.46 |
label=restructure cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          44.81
  command p90 ms                          46.93
  frames                                  62.00
  tick ms/frame                            2.66
  frame p90 ms                             4.00
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  interval p99 ms                         38.71
  interval max ms (median burst)          38.71
  peek_frame ms/frame                      2.06
  peek_draw ms/frame                       1.41
  peek_track ms/frame                      0.30
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.12
  peek_draw ms (worst frame)              14.71
  pd_alpha ms (worst frame)                1.21
  pd_draw ms (worst frame)                13.18
  pd_fill ms (worst frame)                 9.20
  fill_source ms (worst frame)             2.56
  ring_raise ms (worst frame)              5.65
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)               -
  cloak ms (outside frames)               13.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    44.50
  peek_new ms (outside frames)             3.80
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.20
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             3.00
  to_cloak ms (outside frames)             8.60
  peek_cover ms (outside frames)           9.10
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            4.00
  bd_copy ms (outside frames)              0.70
  ovl_restack ms (outside frames)          2.30
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.80
  peek_handover ms (outside frames)       15.10
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.90
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.42
  input->first frame ms                   60.17
  dwm missed vblanks/burst                 2.00
  dwm display interval p90 ms              5.75
  dwm gpu busy p90 ms                      1.32
| restructure cpu8 bd-config bo-config #1 | resize | 44.81 | 46.93 | 62.00 | 2.66 | 4.00 | 3.00 | 5.72 | 5.79 | 38.71 | 38.71 | 2.06 | 1.41 | 0.30 | 0.13 | 0.12 | 14.71 | 1.21 | 13.18 | 9.20 | 2.56 | 5.65 | 0.00 | - | 0.03 | 2.00 | - | 13.60 | - | - | - | 44.50 | 3.80 | 2.10 | - | - | - | 13.20 | 1.00 | 3.00 | 8.60 | 9.10 | 0.00 | 1.20 | 4.00 | 0.70 | 2.30 | - | 0.80 | 15.10 | 0.10 | 0.90 | - | 6.00 | 0.00 | - | - | - | - | 3.42 | 60.17 | 2.00 | 5.75 | 1.32 |
Moved 3 window(s) to workspace '2' to switch to.
label=restructure cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          35.02
  command p90 ms                          35.56
  frames                                  99.00
  tick ms/frame                            1.31
  frame p90 ms                             1.24
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         27.14
  interval max ms (median burst)          27.14
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            6.30
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    34.70
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)               -
  to_draw ms (outside frames)                 -
  to_alpha ms (outside frames)                -
  to_cloak ms (outside frames)                -
  peek_cover ms (outside frames)              -
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)       11.90
  ws_park ms (outside frames)              2.30
  ws_show ms (outside frames)              9.00
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                21.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      2.21
| restructure cpu8 bd-config bo-config #1 | switch | 35.02 | 35.56 | 99.00 | 1.31 | 1.24 | 3.00 | 5.71 | 5.76 | 27.14 | 27.14 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.04 | - | 6.30 | - | - | - | - | 34.70 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.20 | - | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 11.90 | 2.30 | 9.00 | 0.40 | - | - | 21.00 | 11.43 | 2.21 |
label=restructure cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          42.77
  command p90 ms                          46.16
  frames                                  62.00
  tick ms/frame                            2.52
  frame p90 ms                             4.68
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.82
  interval p99 ms                         42.96
  interval max ms (median burst)          42.96
  peek_frame ms/frame                      2.07
  peek_draw ms/frame                       1.25
  peek_track ms/frame                      0.36
  peek_ring ms/frame                       0.11
  peek_backdrop ms/frame                   0.19
  peek_draw ms (worst frame)              10.33
  pd_alpha ms (worst frame)                1.04
  pd_draw ms (worst frame)                 9.31
  pd_fill ms (worst frame)                 5.39
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              3.83
  restack ms (worst frame)                 3.10
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.60
  cloak ms (outside frames)               14.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    42.50
  peek_new ms (outside frames)             8.00
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           14.30
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.00
  to_cloak ms (outside frames)            11.40
  peek_cover ms (outside frames)          14.90
  ring_prep ms (outside frames)            3.20
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            4.60
  bd_copy ms (outside frames)              1.10
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.30
  peek_handover ms (outside frames)       14.90
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              3.40
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.16
  input->first frame ms                   58.05
  dwm missed vblanks/burst                34.00
  dwm display interval p90 ms             28.58
  dwm gpu busy p90 ms                      9.65
| restructure cpu8 bd-config bo-config #1 | minimize | 42.77 | 46.16 | 62.00 | 2.52 | 4.68 | 3.00 | 5.71 | 5.82 | 42.96 | 42.96 | 2.07 | 1.25 | 0.36 | 0.11 | 0.19 | 10.33 | 1.04 | 9.31 | 5.39 | 0.00 | 3.83 | 3.10 | - | 0.03 | 2.00 | 5.60 | 14.80 | - | - | - | 42.50 | 8.00 | 2.00 | - | - | - | 14.30 | 1.00 | 2.00 | 11.40 | 14.90 | 3.20 | 1.30 | 4.60 | 1.10 | 0.10 | - | 1.30 | 14.90 | 0.30 | 3.40 | - | 6.00 | 0.00 | - | - | - | - | 10.16 | 58.05 | 34.00 | 28.58 | 9.65 |
tiles main cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.71
  command p90 ms                          21.05
  frames                                  63.00
  tick ms/frame                            0.42
  frame p90 ms                             0.51
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         12.86
  interval max ms (median burst)          12.86
  peek_frame ms/frame                      0.34
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.67
  pd_alpha ms (worst frame)                0.30
  pd_draw ms (worst frame)                 0.51
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)                7.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.50
  peek_new ms (outside frames)             2.30
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.90
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             2.10
  peek_cover ms (outside frames)           1.20
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.60
  ring_copy ms (outside frames)            0.20
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.40
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        3.40
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.30
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.79
  input->first frame ms                   19.44
  dwm missed vblanks/burst                 4.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.06
| main cpu8 bd-config bo-config #2 | move | 18.71 | 21.05 | 63.00 | 0.42 | 0.51 | 1.00 | 5.71 | 5.75 | 12.86 | 12.86 | 0.34 | 0.17 | 0.05 | 0.01 | 0.03 | 0.67 | 0.30 | 0.51 | - | - | - | - | - | - | - | 4.70 | 7.40 | - | - | - | 18.50 | 2.30 | 1.20 | - | - | - | 2.90 | 0.30 | 0.50 | 2.10 | 1.20 | 0.00 | 1.60 | 0.20 | 0.20 | 0.40 | - | 0.10 | 3.40 | 0.30 | 1.30 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.79 | 19.44 | 4.00 | 11.43 | 8.06 |
label=main cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          44.75
  command p90 ms                          46.27
  frames                                  60.00
  tick ms/frame                            2.63
  frame p90 ms                             4.33
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         42.46
  interval max ms (median burst)          42.46
  peek_frame ms/frame                      2.21
  peek_draw ms/frame                       1.53
  peek_track ms/frame                      0.32
  peek_ring ms/frame                       0.15
  peek_backdrop ms/frame                   0.13
  peek_draw ms (worst frame)              21.46
  pd_alpha ms (worst frame)                1.16
  pd_draw ms (worst frame)                20.17
  pd_fill ms (worst frame)                16.16
  fill_source ms (worst frame)             5.04
  ring_raise ms (worst frame)              6.99
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)               -
  cloak ms (outside frames)               14.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    44.50
  peek_new ms (outside frames)             4.00
  peek_place ms (outside frames)           2.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           14.30
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.40
  to_cloak ms (outside frames)             9.90
  peek_cover ms (outside frames)           9.00
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.40
  ring_copy ms (outside frames)            3.70
  bd_copy ms (outside frames)              0.60
  ovl_restack ms (outside frames)          2.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.00
  peek_handover ms (outside frames)       14.10
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              1.00
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.09
  input->first frame ms                   67.87
  dwm missed vblanks/burst                25.00
  dwm display interval p90 ms             17.16
  dwm gpu busy p90 ms                      1.39
| main cpu8 bd-config bo-config #2 | resize | 44.75 | 46.27 | 60.00 | 2.63 | 4.33 | 3.00 | 5.72 | 5.77 | 42.46 | 42.46 | 2.21 | 1.53 | 0.32 | 0.15 | 0.13 | 21.46 | 1.16 | 20.17 | 16.16 | 5.04 | 6.99 | 0.00 | - | 0.04 | 2.00 | - | 14.80 | - | - | - | 44.50 | 4.00 | 2.20 | - | - | - | 14.30 | 0.90 | 2.40 | 9.90 | 9.00 | 0.00 | 1.40 | 3.70 | 0.60 | 2.60 | - | 1.00 | 14.10 | 0.20 | 1.00 | - | 6.00 | 0.00 | - | - | - | - | 3.09 | 67.87 | 25.00 | 17.16 | 1.39 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          32.61
  command p90 ms                          35.07
  frames                                  99.00
  tick ms/frame                            1.24
  frame p90 ms                             1.26
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         25.26
  interval max ms (median burst)          25.26
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    32.30
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)               -
  to_draw ms (outside frames)                 -
  to_alpha ms (outside frames)                -
  to_cloak ms (outside frames)                -
  peek_cover ms (outside frames)              -
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)       11.40
  ws_park ms (outside frames)              2.40
  ws_show ms (outside frames)              8.10
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                30.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      2.39
| main cpu8 bd-config bo-config #2 | switch | 32.61 | 35.07 | 99.00 | 1.24 | 1.26 | 3.00 | 5.71 | 5.77 | 25.26 | 25.26 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.40 | - | - | - | - | 32.30 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.20 | - | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 11.40 | 2.40 | 8.10 | 0.40 | - | - | 30.00 | 11.44 | 2.39 |
label=main cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          44.53
  command p90 ms                          58.12
  frames                                  61.00
  tick ms/frame                            2.49
  frame p90 ms                             4.30
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.80
  interval p99 ms                         48.49
  interval max ms (median burst)          48.49
  peek_frame ms/frame                      2.03
  peek_draw ms/frame                       1.25
  peek_track ms/frame                      0.35
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.18
  peek_draw ms (worst frame)              14.11
  pd_alpha ms (worst frame)                1.03
  pd_draw ms (worst frame)                13.30
  pd_fill ms (worst frame)                 8.64
  fill_source ms (worst frame)             2.43
  ring_raise ms (worst frame)              4.17
  restack ms (worst frame)                 3.14
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.00
  cloak ms (outside frames)               15.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    44.20
  peek_new ms (outside frames)             8.20
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           14.60
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.60
  to_cloak ms (outside frames)            11.30
  peek_cover ms (outside frames)          14.70
  ring_prep ms (outside frames)            4.40
  ring_park ms (outside frames)            1.40
  ring_copy ms (outside frames)            4.50
  bd_copy ms (outside frames)              1.00
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.40
  peek_handover ms (outside frames)       15.40
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              2.70
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     7.97
  input->first frame ms                   65.74
  dwm missed vblanks/burst                32.00
  dwm display interval p90 ms             28.59
  dwm gpu busy p90 ms                     10.32
| main cpu8 bd-config bo-config #2 | minimize | 44.53 | 58.12 | 61.00 | 2.49 | 4.30 | 4.00 | 5.71 | 5.80 | 48.49 | 48.49 | 2.03 | 1.25 | 0.35 | 0.12 | 0.18 | 14.11 | 1.03 | 13.30 | 8.64 | 2.43 | 4.17 | 3.14 | - | 0.03 | 2.00 | 4.00 | 15.00 | - | - | - | 44.20 | 8.20 | 1.90 | - | - | - | 14.60 | 1.00 | 2.60 | 11.30 | 14.70 | 4.40 | 1.40 | 4.50 | 1.00 | 0.10 | - | 1.40 | 15.40 | 0.40 | 2.70 | - | 6.00 | 0.00 | - | - | - | - | 7.97 | 65.74 | 32.00 | 28.59 | 10.32 |
tiles restructure cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=restructure cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.10
  command p90 ms                          20.26
  frames                                  63.00
  tick ms/frame                            0.41
  frame p90 ms                             0.51
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.44
  interval max ms (median burst)          12.44
  peek_frame ms/frame                      0.35
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.74
  pd_alpha ms (worst frame)                0.33
  pd_draw ms (worst frame)                 0.46
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)                8.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.90
  peek_new ms (outside frames)             2.20
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.90
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             2.10
  peek_cover ms (outside frames)           1.10
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.50
  ring_copy ms (outside frames)            0.20
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.40
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        3.50
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.20
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.59
  input->first frame ms                   19.99
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.10
| restructure cpu8 bd-config bo-config #2 | move | 19.10 | 20.26 | 63.00 | 0.41 | 0.51 | 0.00 | 5.71 | 5.76 | 12.44 | 12.44 | 0.35 | 0.17 | 0.05 | 0.01 | 0.03 | 0.74 | 0.33 | 0.46 | - | - | - | - | - | - | - | 5.10 | 8.30 | - | - | - | 18.90 | 2.20 | 1.20 | - | - | - | 2.90 | 0.30 | 0.50 | 2.10 | 1.10 | 0.00 | 1.50 | 0.20 | 0.20 | 0.40 | - | 0.10 | 3.50 | 0.30 | 1.20 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.59 | 19.99 | 5.00 | 11.43 | 8.10 |
label=restructure cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          44.17
  command p90 ms                          46.43
  frames                                  61.00
  tick ms/frame                            2.56
  frame p90 ms                             4.20
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         42.17
  interval max ms (median burst)          42.17
  peek_frame ms/frame                      2.11
  peek_draw ms/frame                       1.50
  peek_track ms/frame                      0.28
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.12
  peek_draw ms (worst frame)              18.09
  pd_alpha ms (worst frame)                1.17
  pd_draw ms (worst frame)                16.86
  pd_fill ms (worst frame)                12.64
  fill_source ms (worst frame)             2.54
  ring_raise ms (worst frame)              5.74
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)               -
  cloak ms (outside frames)               14.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    43.90
  peek_new ms (outside frames)             3.80
  peek_place ms (outside frames)           2.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.80
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.10
  to_cloak ms (outside frames)             9.50
  peek_cover ms (outside frames)           9.00
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            3.70
  bd_copy ms (outside frames)              0.60
  ovl_restack ms (outside frames)          2.50
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.80
  peek_handover ms (outside frames)       14.00
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.80
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.49
  input->first frame ms                   64.01
  dwm missed vblanks/burst                24.00
  dwm display interval p90 ms             17.16
  dwm gpu busy p90 ms                      1.44
| restructure cpu8 bd-config bo-config #2 | resize | 44.17 | 46.43 | 61.00 | 2.56 | 4.20 | 3.00 | 5.71 | 5.77 | 42.17 | 42.17 | 2.11 | 1.50 | 0.28 | 0.14 | 0.12 | 18.09 | 1.17 | 16.86 | 12.64 | 2.54 | 5.74 | 0.00 | - | 0.04 | 2.00 | - | 14.30 | - | - | - | 43.90 | 3.80 | 2.20 | - | - | - | 13.80 | 0.90 | 2.10 | 9.50 | 9.00 | 0.00 | 1.20 | 3.70 | 0.60 | 2.50 | - | 0.80 | 14.00 | 0.10 | 0.80 | - | 6.00 | 0.00 | - | - | - | - | 3.49 | 64.01 | 24.00 | 17.16 | 1.44 |
Moved 3 window(s) to workspace '2' to switch to.
label=restructure cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.18
  command p90 ms                          36.69
  frames                                  99.00
  tick ms/frame                            1.27
  frame p90 ms                             1.32
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         23.82
  interval max ms (median burst)          23.82
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.50
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    33.90
  peek_new ms (outside frames)             0.00
  peek_place ms (outside frames)           0.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            0.00
  to_draw ms (outside frames)              0.00
  to_alpha ms (outside frames)             0.00
  to_cloak ms (outside frames)             0.00
  peek_cover ms (outside frames)           0.00
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        0.00
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)       11.40
  ws_park ms (outside frames)              2.40
  ws_show ms (outside frames)              8.30
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                23.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      2.72
| restructure cpu8 bd-config bo-config #2 | switch | 34.18 | 36.69 | 99.00 | 1.27 | 1.32 | 3.00 | 5.71 | 5.76 | 23.82 | 23.82 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.50 | 0.00 | - | - | - | 33.90 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | 0.00 | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 11.40 | 2.40 | 8.30 | 0.40 | 0.00 | 0.00 | 23.00 | 11.44 | 2.72 |
label=restructure cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          43.46
  command p90 ms                          49.74
  frames                                  62.00
  tick ms/frame                            2.44
  frame p90 ms                             4.22
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.81
  interval p99 ms                         51.31
  interval max ms (median burst)          51.31
  peek_frame ms/frame                      1.88
  peek_draw ms/frame                       1.17
  peek_track ms/frame                      0.35
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.19
  peek_draw ms (worst frame)              11.19
  pd_alpha ms (worst frame)                1.09
  pd_draw ms (worst frame)                10.01
  pd_fill ms (worst frame)                 5.05
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              4.09
  restack ms (worst frame)                 2.96
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.10
  cloak ms (outside frames)               14.50
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    43.20
  peek_new ms (outside frames)             8.00
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.90
  to_draw ms (outside frames)              1.10
  to_alpha ms (outside frames)             2.80
  to_cloak ms (outside frames)            10.50
  peek_cover ms (outside frames)          14.60
  ring_prep ms (outside frames)            3.90
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            4.60
  bd_copy ms (outside frames)              1.00
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.40
  peek_handover ms (outside frames)       16.30
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              2.80
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.87
  input->first frame ms                   61.21
  dwm missed vblanks/burst                31.00
  dwm display interval p90 ms             28.58
  dwm gpu busy p90 ms                      8.80
| restructure cpu8 bd-config bo-config #2 | minimize | 43.46 | 49.74 | 62.00 | 2.44 | 4.22 | 3.00 | 5.71 | 5.81 | 51.31 | 51.31 | 1.88 | 1.17 | 0.35 | 0.12 | 0.19 | 11.19 | 1.09 | 10.01 | 5.05 | 0.00 | 4.09 | 2.96 | - | 0.03 | 2.00 | 4.10 | 14.50 | - | - | - | 43.20 | 8.00 | 1.80 | - | - | - | 13.90 | 1.10 | 2.80 | 10.50 | 14.60 | 3.90 | 1.30 | 4.60 | 1.00 | 0.10 | - | 1.40 | 16.30 | 0.40 | 2.80 | - | 6.00 | 0.00 | - | - | - | - | 8.87 | 61.21 | 31.00 | 28.58 | 8.80 |
tiles this cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.82
  command p90 ms                          20.79
  frames                                  63.00
  tick ms/frame                            0.42
  frame p90 ms                             0.53
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.46
  interval max ms (median burst)          12.46
  peek_frame ms/frame                      0.31
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.06
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.59
  pd_alpha ms (worst frame)                0.26
  pd_draw ms (worst frame)                 0.47
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.60
  cloak ms (outside frames)                7.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.60
  peek_new ms (outside frames)             2.40
  peek_place ms (outside frames)           1.30
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            3.00
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             2.10
  peek_cover ms (outside frames)           1.60
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.60
  ring_copy ms (outside frames)            0.30
  bd_copy ms (outside frames)              0.10
  ovl_restack ms (outside frames)          0.80
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)        3.30
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.30
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.92
  input->first frame ms                   19.57
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.14
| this cpu8 bd-config bo-config #2 | move | 18.82 | 20.79 | 63.00 | 0.42 | 0.53 | 0.00 | 5.71 | 5.76 | 12.46 | 12.46 | 0.31 | 0.17 | 0.06 | 0.01 | 0.04 | 0.59 | 0.26 | 0.47 | - | - | - | - | - | - | - | 4.60 | 7.30 | - | - | - | 18.60 | 2.40 | 1.30 | - | - | - | 3.00 | 0.30 | 0.50 | 2.10 | 1.60 | 0.00 | 1.60 | 0.30 | 0.10 | 0.80 | - | 0.20 | 3.30 | 0.30 | 1.30 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.92 | 19.57 | 5.00 | 11.43 | 8.14 |
label=this cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          58.04
  command p90 ms                          60.96
  frames                                  60.00
  tick ms/frame                            2.74
  frame p90 ms                             4.19
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         42.11
  interval max ms (median burst)          42.11
  peek_frame ms/frame                      2.21
  peek_draw ms/frame                       1.49
  peek_track ms/frame                      0.33
  peek_ring ms/frame                       0.18
  peek_backdrop ms/frame                   0.14
  peek_draw ms (worst frame)              19.39
  pd_alpha ms (worst frame)                1.19
  pd_draw ms (worst frame)                18.02
  pd_fill ms (worst frame)                13.78
  fill_source ms (worst frame)             4.38
  ring_raise ms (worst frame)              5.75
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.00
  cloak ms (outside frames)               24.50
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    57.70
  peek_new ms (outside frames)             4.20
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.70
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.90
  to_cloak ms (outside frames)             9.50
  peek_cover ms (outside frames)          12.40
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            7.00
  bd_copy ms (outside frames)              0.60
  ovl_restack ms (outside frames)          2.50
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         2.50
  peek_handover ms (outside frames)       14.50
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              1.10
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.85
  input->first frame ms                   78.85
  dwm missed vblanks/burst                21.00
  dwm display interval p90 ms             17.15
  dwm gpu busy p90 ms                      1.46
| this cpu8 bd-config bo-config #2 | resize | 58.04 | 60.96 | 60.00 | 2.74 | 4.19 | 3.00 | 5.72 | 5.76 | 42.11 | 42.11 | 2.21 | 1.49 | 0.33 | 0.18 | 0.14 | 19.39 | 1.19 | 18.02 | 13.78 | 4.38 | 5.75 | 0.00 | - | 0.04 | 2.00 | 10.00 | 24.50 | - | - | - | 57.70 | 4.20 | 2.10 | - | - | - | 13.70 | 1.00 | 2.90 | 9.50 | 12.40 | 0.00 | 1.20 | 7.00 | 0.60 | 2.50 | - | 2.50 | 14.50 | 0.10 | 1.10 | - | 6.00 | 0.00 | - | - | - | - | 3.85 | 78.85 | 21.00 | 17.15 | 1.46 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          35.15
  command p90 ms                          36.86
  frames                                  99.00
  tick ms/frame                            1.34
  frame p90 ms                             1.47
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         23.01
  interval max ms (median burst)          24.39
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            6.00
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    34.90
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)               -
  to_draw ms (outside frames)                 -
  to_alpha ms (outside frames)                -
  to_cloak ms (outside frames)                -
  peek_cover ms (outside frames)              -
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)       11.90
  ws_park ms (outside frames)              2.30
  ws_show ms (outside frames)              8.50
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                25.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      5.93
| this cpu8 bd-config bo-config #2 | switch | 35.15 | 36.86 | 99.00 | 1.34 | 1.47 | 3.00 | 5.71 | 5.77 | 23.01 | 24.39 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 6.00 | - | - | - | - | 34.90 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | - | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 11.90 | 2.30 | 8.50 | 0.40 | - | - | 25.00 | 11.44 | 5.93 |
label=this cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          47.45
  command p90 ms                          59.97
  frames                                  62.00
  tick ms/frame                            2.61
  frame p90 ms                             4.34
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         46.81
  interval max ms (median burst)          46.81
  peek_frame ms/frame                      2.13
  peek_draw ms/frame                       1.29
  peek_track ms/frame                      0.39
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.24
  peek_draw ms (worst frame)              13.71
  pd_alpha ms (worst frame)                0.91
  pd_draw ms (worst frame)                12.90
  pd_fill ms (worst frame)                 9.21
  fill_source ms (worst frame)             2.33
  ring_raise ms (worst frame)              4.41
  restack ms (worst frame)                 3.02
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           15.40
  cloak ms (outside frames)               21.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    47.20
  peek_new ms (outside frames)            10.90
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.20
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.60
  to_cloak ms (outside frames)             8.70
  peek_cover ms (outside frames)          17.30
  ring_prep ms (outside frames)            4.20
  ring_park ms (outside frames)            1.10
  ring_copy ms (outside frames)            8.40
  bd_copy ms (outside frames)              1.00
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         2.10
  peek_handover ms (outside frames)       14.90
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              3.20
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.83
  input->first frame ms                   64.45
  dwm missed vblanks/burst                34.00
  dwm display interval p90 ms             28.57
  dwm gpu busy p90 ms                      9.16
| this cpu8 bd-config bo-config #2 | minimize | 47.45 | 59.97 | 62.00 | 2.61 | 4.34 | 3.00 | 5.71 | 5.78 | 46.81 | 46.81 | 2.13 | 1.29 | 0.39 | 0.14 | 0.24 | 13.71 | 0.91 | 12.90 | 9.21 | 2.33 | 4.41 | 3.02 | - | 0.03 | 2.00 | 15.40 | 21.80 | - | - | - | 47.20 | 10.90 | 1.90 | - | - | - | 12.20 | 1.00 | 2.60 | 8.70 | 17.30 | 4.20 | 1.10 | 8.40 | 1.00 | 0.10 | - | 2.10 | 14.90 | 0.40 | 3.20 | - | 6.00 | 0.00 | - | - | - | - | 8.83 | 64.45 | 34.00 | 28.57 | 9.16 |
tiles restructure cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=restructure cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.62
  command p90 ms                          20.46
  frames                                  63.00
  tick ms/frame                            0.43
  frame p90 ms                             0.52
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         12.63
  interval max ms (median burst)          12.63
  peek_frame ms/frame                      0.31
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.06
  peek_ring ms/frame                       0.02
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.61
  pd_alpha ms (worst frame)                0.26
  pd_draw ms (worst frame)                 0.49
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)                7.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.30
  peek_new ms (outside frames)             2.40
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.90
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             2.10
  peek_cover ms (outside frames)           1.50
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.60
  ring_copy ms (outside frames)            0.10
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.80
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        3.00
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.30
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.80
  input->first frame ms                   19.53
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.14
| restructure cpu8 bd-config bo-config #3 | move | 18.62 | 20.46 | 63.00 | 0.43 | 0.52 | 0.00 | 5.71 | 5.75 | 12.63 | 12.63 | 0.31 | 0.17 | 0.06 | 0.02 | 0.03 | 0.61 | 0.26 | 0.49 | - | - | - | - | - | - | - | 4.50 | 7.30 | - | - | - | 18.30 | 2.40 | 1.20 | - | - | - | 2.90 | 0.30 | 0.50 | 2.10 | 1.50 | 0.00 | 1.60 | 0.10 | 0.20 | 0.80 | - | 0.10 | 3.00 | 0.30 | 1.30 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.80 | 19.53 | 5.00 | 11.43 | 8.14 |
label=restructure cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          44.73
  command p90 ms                          46.33
  frames                                  61.00
  tick ms/frame                            2.56
  frame p90 ms                             4.33
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.81
  interval p99 ms                         37.25
  interval max ms (median burst)          37.25
  peek_frame ms/frame                      2.17
  peek_draw ms/frame                       1.46
  peek_track ms/frame                      0.27
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.12
  peek_draw ms (worst frame)              15.52
  pd_alpha ms (worst frame)                1.20
  pd_draw ms (worst frame)                14.31
  pd_fill ms (worst frame)                10.18
  fill_source ms (worst frame)             2.66
  ring_raise ms (worst frame)              6.11
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)               -
  cloak ms (outside frames)               14.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    44.40
  peek_new ms (outside frames)             4.10
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.90
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.10
  to_cloak ms (outside frames)             9.50
  peek_cover ms (outside frames)           9.70
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            4.10
  bd_copy ms (outside frames)              0.70
  ovl_restack ms (outside frames)          2.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.80
  peek_handover ms (outside frames)       13.90
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.90
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.62
  input->first frame ms                   60.82
  dwm missed vblanks/burst                 1.00
  dwm display interval p90 ms              5.73
  dwm gpu busy p90 ms                      1.39
| restructure cpu8 bd-config bo-config #3 | resize | 44.73 | 46.33 | 61.00 | 2.56 | 4.33 | 3.00 | 5.71 | 5.81 | 37.25 | 37.25 | 2.17 | 1.46 | 0.27 | 0.12 | 0.12 | 15.52 | 1.20 | 14.31 | 10.18 | 2.66 | 6.11 | 0.00 | - | 0.04 | 2.00 | - | 14.20 | - | - | - | 44.40 | 4.10 | 2.10 | - | - | - | 13.90 | 1.00 | 2.10 | 9.50 | 9.70 | 0.00 | 1.20 | 4.10 | 0.70 | 2.60 | - | 0.80 | 13.90 | 0.10 | 0.90 | - | 6.00 | 0.00 | - | - | - | - | 3.62 | 60.82 | 1.00 | 5.73 | 1.39 |
Moved 3 window(s) to workspace '2' to switch to.
label=restructure cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          32.86
  command p90 ms                          36.52
  frames                                  99.00
  tick ms/frame                            1.30
  frame p90 ms                             1.36
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         24.45
  interval max ms (median burst)          24.45
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.50
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    32.60
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)               -
  to_draw ms (outside frames)                 -
  to_alpha ms (outside frames)                -
  to_cloak ms (outside frames)                -
  peek_cover ms (outside frames)              -
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)       11.20
  ws_park ms (outside frames)              2.40
  ws_show ms (outside frames)              8.00
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                23.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      5.40
| restructure cpu8 bd-config bo-config #3 | switch | 32.86 | 36.52 | 99.00 | 1.30 | 1.36 | 3.00 | 5.71 | 5.77 | 24.45 | 24.45 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.04 | - | 5.50 | - | - | - | - | 32.60 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | - | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 11.20 | 2.40 | 8.00 | 0.40 | - | - | 23.00 | 11.44 | 5.40 |
label=restructure cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          40.87
  command p90 ms                          50.05
  frames                                  61.00
  tick ms/frame                            2.69
  frame p90 ms                             4.30
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.80
  interval p99 ms                         40.72
  interval max ms (median burst)          40.72
  peek_frame ms/frame                      2.19
  peek_draw ms/frame                       1.25
  peek_track ms/frame                      0.42
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.21
  peek_draw ms (worst frame)              13.42
  pd_alpha ms (worst frame)                0.95
  pd_draw ms (worst frame)                12.47
  pd_fill ms (worst frame)                 8.77
  fill_source ms (worst frame)             2.57
  ring_raise ms (worst frame)              3.94
  restack ms (worst frame)                 2.75
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            3.60
  cloak ms (outside frames)               14.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    40.60
  peek_new ms (outside frames)             8.10
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           14.60
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.80
  to_cloak ms (outside frames)            10.80
  peek_cover ms (outside frames)          14.10
  ring_prep ms (outside frames)            3.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            4.50
  bd_copy ms (outside frames)              1.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.30
  peek_handover ms (outside frames)       13.10
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              3.20
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     7.81
  input->first frame ms                   60.94
  dwm missed vblanks/burst                31.00
  dwm display interval p90 ms             28.57
  dwm gpu busy p90 ms                      8.98
| restructure cpu8 bd-config bo-config #3 | minimize | 40.87 | 50.05 | 61.00 | 2.69 | 4.30 | 3.00 | 5.71 | 5.80 | 40.72 | 40.72 | 2.19 | 1.25 | 0.42 | 0.14 | 0.21 | 13.42 | 0.95 | 12.47 | 8.77 | 2.57 | 3.94 | 2.75 | - | 0.03 | 2.00 | 3.60 | 14.60 | - | - | - | 40.60 | 8.10 | 1.90 | - | - | - | 14.60 | 1.00 | 2.80 | 10.80 | 14.10 | 3.00 | 1.20 | 4.50 | 1.00 | 0.00 | - | 1.30 | 13.10 | 0.40 | 3.20 | - | 6.00 | 0.00 | - | - | - | - | 7.81 | 60.94 | 31.00 | 28.57 | 8.98 |
tiles this cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.69
  command p90 ms                          20.91
  frames                                  63.00
  tick ms/frame                            0.42
  frame p90 ms                             0.54
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.18
  interval max ms (median burst)          12.18
  peek_frame ms/frame                      0.34
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.67
  pd_alpha ms (worst frame)                0.30
  pd_draw ms (worst frame)                 0.47
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.60
  cloak ms (outside frames)                8.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    19.40
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            3.00
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             2.10
  peek_cover ms (outside frames)           1.20
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.50
  ring_copy ms (outside frames)            0.30
  bd_copy ms (outside frames)              0.10
  ovl_restack ms (outside frames)          0.40
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)        3.50
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.20
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.53
  input->first frame ms                   20.32
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      7.71
| this cpu8 bd-config bo-config #3 | move | 19.69 | 20.91 | 63.00 | 0.42 | 0.54 | 0.00 | 5.71 | 5.76 | 12.18 | 12.18 | 0.34 | 0.17 | 0.05 | 0.01 | 0.04 | 0.67 | 0.30 | 0.47 | - | - | - | - | - | - | - | 5.60 | 8.90 | - | - | - | 19.40 | 2.10 | 1.20 | - | - | - | 3.00 | 0.30 | 0.50 | 2.10 | 1.20 | 0.00 | 1.50 | 0.30 | 0.10 | 0.40 | - | 0.20 | 3.50 | 0.30 | 1.20 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.53 | 20.32 | 5.00 | 11.43 | 7.71 |
label=this cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          55.13
  command p90 ms                          66.26
  frames                                  61.00
  tick ms/frame                            2.57
  frame p90 ms                             3.99
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         43.87
  interval max ms (median burst)          43.87
  peek_frame ms/frame                      2.07
  peek_draw ms/frame                       1.44
  peek_track ms/frame                      0.30
  peek_ring ms/frame                       0.16
  peek_backdrop ms/frame                   0.14
  peek_draw ms (worst frame)              17.62
  pd_alpha ms (worst frame)                1.15
  pd_draw ms (worst frame)                16.45
  pd_fill ms (worst frame)                12.36
  fill_source ms (worst frame)             2.40
  ring_raise ms (worst frame)              5.76
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.00
  cloak ms (outside frames)               20.10
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    54.80
  peek_new ms (outside frames)             4.00
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.70
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.50
  to_cloak ms (outside frames)             8.50
  peek_cover ms (outside frames)          12.50
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            7.10
  bd_copy ms (outside frames)              0.50
  ovl_restack ms (outside frames)          2.70
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         2.00
  peek_handover ms (outside frames)       16.40
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              1.00
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.60
  input->first frame ms                   78.66
  dwm missed vblanks/burst                22.00
  dwm display interval p90 ms             17.17
  dwm gpu busy p90 ms                      1.32
| this cpu8 bd-config bo-config #3 | resize | 55.13 | 66.26 | 61.00 | 2.57 | 3.99 | 3.00 | 5.71 | 5.79 | 43.87 | 43.87 | 2.07 | 1.44 | 0.30 | 0.16 | 0.14 | 17.62 | 1.15 | 16.45 | 12.36 | 2.40 | 5.76 | 0.00 | - | 0.04 | 2.00 | 10.00 | 20.10 | - | - | - | 54.80 | 4.00 | 2.10 | - | - | - | 12.70 | 1.00 | 2.50 | 8.50 | 12.50 | 0.00 | 1.20 | 7.10 | 0.50 | 2.70 | - | 2.00 | 16.40 | 0.10 | 1.00 | - | 6.00 | 0.00 | - | - | - | - | 3.60 | 78.66 | 22.00 | 17.17 | 1.32 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          32.91
  command p90 ms                          35.37
  frames                                  99.00
  tick ms/frame                            1.25
  frame p90 ms                             1.19
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         22.72
  interval max ms (median burst)          22.72
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    32.60
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)               -
  to_draw ms (outside frames)                 -
  to_alpha ms (outside frames)                -
  to_cloak ms (outside frames)                -
  peek_cover ms (outside frames)              -
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)       11.30
  ws_park ms (outside frames)              2.80
  ws_show ms (outside frames)              7.70
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                24.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      5.90
| this cpu8 bd-config bo-config #3 | switch | 32.91 | 35.37 | 99.00 | 1.25 | 1.19 | 3.00 | 5.71 | 5.76 | 22.72 | 22.72 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.10 | - | - | - | - | 32.60 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.20 | - | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 11.30 | 2.80 | 7.70 | 0.40 | - | - | 24.00 | 11.44 | 5.90 |
label=this cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          46.95
  command p90 ms                          54.19
  frames                                  61.00
  tick ms/frame                            2.39
  frame p90 ms                             4.02
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         60.77
  interval max ms (median burst)          60.77
  peek_frame ms/frame                      1.99
  peek_draw ms/frame                       1.23
  peek_track ms/frame                      0.37
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.22
  peek_draw ms (worst frame)              14.97
  pd_alpha ms (worst frame)                1.27
  pd_draw ms (worst frame)                13.62
  pd_fill ms (worst frame)                 9.58
  fill_source ms (worst frame)             3.14
  ring_raise ms (worst frame)              3.59
  restack ms (worst frame)                 3.47
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           13.00
  cloak ms (outside frames)               21.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    46.60
  peek_new ms (outside frames)             8.70
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.50
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.10
  to_cloak ms (outside frames)             8.90
  peek_cover ms (outside frames)          19.30
  ring_prep ms (outside frames)            4.50
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            8.20
  bd_copy ms (outside frames)              1.00
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         2.10
  peek_handover ms (outside frames)       16.10
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              3.40
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.05
  input->first frame ms                   66.69
  dwm missed vblanks/burst                33.00
  dwm display interval p90 ms             28.58
  dwm gpu busy p90 ms                      9.72
| this cpu8 bd-config bo-config #3 | minimize | 46.95 | 54.19 | 61.00 | 2.39 | 4.02 | 3.00 | 5.71 | 5.78 | 60.77 | 60.77 | 1.99 | 1.23 | 0.37 | 0.14 | 0.22 | 14.97 | 1.27 | 13.62 | 9.58 | 3.14 | 3.59 | 3.47 | - | 0.03 | 2.00 | 13.00 | 21.60 | - | - | - | 46.60 | 8.70 | 1.90 | - | - | - | 12.50 | 1.00 | 2.10 | 8.90 | 19.30 | 4.50 | 1.30 | 8.20 | 1.00 | 0.10 | - | 2.10 | 16.10 | 0.40 | 3.40 | - | 6.00 | 0.00 | - | - | - | - | 9.05 | 66.69 | 33.00 | 28.58 | 9.72 |
tiles main cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.20
  command p90 ms                          19.39
  frames                                  63.00
  tick ms/frame                            0.43
  frame p90 ms                             0.51
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         12.57
  interval max ms (median burst)          12.57
  peek_frame ms/frame                      0.30
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.06
  peek_ring ms/frame                       0.02
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.71
  pd_alpha ms (worst frame)                0.30
  pd_draw ms (worst frame)                 0.48
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)                7.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.90
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.80
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             2.00
  peek_cover ms (outside frames)           1.40
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.60
  ring_copy ms (outside frames)            0.20
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.70
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)        2.90
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.20
  restack ms (outside frames)              0.90
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.51
  input->first frame ms                   19.39
  dwm missed vblanks/burst                 4.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.18
| main cpu8 bd-config bo-config #3 | move | 18.20 | 19.39 | 63.00 | 0.43 | 0.51 | 0.00 | 5.71 | 5.75 | 12.57 | 12.57 | 0.30 | 0.17 | 0.06 | 0.02 | 0.03 | 0.71 | 0.30 | 0.48 | - | - | - | - | - | - | - | 4.70 | 7.70 | - | - | - | 17.90 | 2.10 | 1.20 | - | - | - | 2.80 | 0.30 | 0.50 | 2.00 | 1.40 | 0.00 | 1.60 | 0.20 | 0.20 | 0.70 | - | 0.10 | 2.90 | 0.30 | 1.20 | 0.90 | 2.00 | 0.00 | - | - | - | - | 3.51 | 19.39 | 4.00 | 11.43 | 8.18 |
label=main cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          46.17
  command p90 ms                          49.31
  frames                                  61.00
  tick ms/frame                            2.62
  frame p90 ms                             4.26
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  interval p99 ms                         48.40
  interval max ms (median burst)          48.40
  peek_frame ms/frame                      2.18
  peek_draw ms/frame                       1.52
  peek_track ms/frame                      0.28
  peek_ring ms/frame                       0.15
  peek_backdrop ms/frame                   0.12
  peek_draw ms (worst frame)              20.27
  pd_alpha ms (worst frame)                1.22
  pd_draw ms (worst frame)                18.03
  pd_fill ms (worst frame)                13.72
  fill_source ms (worst frame)             4.32
  ring_raise ms (worst frame)              5.97
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)               -
  cloak ms (outside frames)               14.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    45.70
  peek_new ms (outside frames)             3.90
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.80
  to_draw ms (outside frames)              1.10
  to_alpha ms (outside frames)             3.10
  to_cloak ms (outside frames)             9.20
  peek_cover ms (outside frames)           9.30
  ring_prep ms (outside frames)            0.10
  ring_park ms (outside frames)            1.50
  ring_copy ms (outside frames)            4.00
  bd_copy ms (outside frames)              0.70
  ovl_restack ms (outside frames)          2.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.50
  peek_handover ms (outside frames)       14.80
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              0.90
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.54
  input->first frame ms                   65.24
  dwm missed vblanks/burst                13.00
  dwm display interval p90 ms             17.17
  dwm gpu busy p90 ms                      1.43
| main cpu8 bd-config bo-config #3 | resize | 46.17 | 49.31 | 61.00 | 2.62 | 4.26 | 3.00 | 5.72 | 5.79 | 48.40 | 48.40 | 2.18 | 1.52 | 0.28 | 0.15 | 0.12 | 20.27 | 1.22 | 18.03 | 13.72 | 4.32 | 5.97 | 0.00 | - | 0.04 | 2.00 | - | 14.30 | - | - | - | 45.70 | 3.90 | 2.10 | - | - | - | 13.80 | 1.10 | 3.10 | 9.20 | 9.30 | 0.10 | 1.50 | 4.00 | 0.70 | 2.60 | - | 1.50 | 14.80 | 0.20 | 0.90 | - | 6.00 | 0.00 | - | - | - | - | 3.54 | 65.24 | 13.00 | 17.17 | 1.43 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.37
  command p90 ms                          35.37
  frames                                  99.00
  tick ms/frame                            1.27
  frame p90 ms                             1.35
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         23.68
  interval max ms (median burst)          23.68
  peek_frame ms/frame                      0.00
  peek_draw ms/frame                       0.00
  peek_track ms/frame                      0.00
  peek_ring ms/frame                       0.00
  peek_backdrop ms/frame                   0.00
  peek_draw ms (worst frame)               0.00
  pd_alpha ms (worst frame)                0.00
  pd_draw ms (worst frame)                 0.00
  pd_fill ms (worst frame)                 0.00
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              0.00
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.60
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    34.10
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)               -
  to_draw ms (outside frames)                 -
  to_alpha ms (outside frames)                -
  to_cloak ms (outside frames)                -
  peek_cover ms (outside frames)              -
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            0.00
  ring_copy ms (outside frames)            0.00
  bd_copy ms (outside frames)              0.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)       11.60
  ws_park ms (outside frames)              2.40
  ws_show ms (outside frames)              8.10
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                27.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      5.61
| main cpu8 bd-config bo-config #3 | switch | 34.37 | 35.37 | 99.00 | 1.27 | 1.35 | 3.00 | 5.71 | 5.77 | 23.68 | 23.68 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.60 | - | - | - | - | 34.10 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.20 | - | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 11.60 | 2.40 | 8.10 | 0.40 | - | - | 27.00 | 11.44 | 5.61 |
label=main cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          43.90
  command p90 ms                          44.95
  frames                                  61.00
  tick ms/frame                            2.20
  frame p90 ms                             4.37
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         42.39
  interval max ms (median burst)          42.39
  peek_frame ms/frame                      1.96
  peek_draw ms/frame                       1.24
  peek_track ms/frame                      0.33
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.15
  peek_draw ms (worst frame)              13.71
  pd_alpha ms (worst frame)                0.93
  pd_draw ms (worst frame)                12.54
  pd_fill ms (worst frame)                 8.29
  fill_source ms (worst frame)             2.19
  ring_raise ms (worst frame)              4.07
  restack ms (worst frame)                 2.80
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)               14.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    43.60
  peek_new ms (outside frames)             8.90
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.40
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.40
  to_cloak ms (outside frames)            10.20
  peek_cover ms (outside frames)          15.40
  ring_prep ms (outside frames)            4.10
  ring_park ms (outside frames)            1.40
  ring_copy ms (outside frames)            4.60
  bd_copy ms (outside frames)              1.00
  ovl_restack ms (outside frames)          0.20
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.30
  peek_handover ms (outside frames)       14.80
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              3.30
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.40
  input->first frame ms                   61.73
  dwm missed vblanks/burst                32.00
  dwm display interval p90 ms             28.60
  dwm gpu busy p90 ms                      9.57
| main cpu8 bd-config bo-config #3 | minimize | 43.90 | 44.95 | 61.00 | 2.20 | 4.37 | 3.00 | 5.71 | 5.79 | 42.39 | 42.39 | 1.96 | 1.24 | 0.33 | 0.12 | 0.15 | 13.71 | 0.93 | 12.54 | 8.29 | 2.19 | 4.07 | 2.80 | - | 0.03 | 2.00 | 4.80 | 14.00 | - | - | - | 43.60 | 8.90 | 1.90 | - | - | - | 13.40 | 0.90 | 2.40 | 10.20 | 15.40 | 4.10 | 1.40 | 4.60 | 1.00 | 0.20 | - | 1.30 | 14.80 | 0.30 | 3.30 | - | 6.00 | 0.00 | - | - | - | - | 9.40 | 61.73 | 32.00 | 28.60 | 9.57 |

## move

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | main | restructure |
| --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 18.69 | 18.87 | 18.35 |
| cpu0 bd-config bo-config | frame p90 ms | 0.52 | 0.52 | 0.52 |
| cpu0 bd-config bo-config | interval p90 ms | 5.74 | 5.74 | 5.74 |
| cpu0 bd-config bo-config | interval p99 ms | 10.57 | 10.60 | 10.57 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 2.10 | 2.10 | 2.10 |
| cpu0 bd-config bo-config | take_over ms (outside frames) | 2.70 | 2.60 | 2.70 |
| cpu0 bd-config bo-config | peek_cover ms (outside frames) | 1.10 | 1.30 | 1.00 |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | — | — |
| cpu0 bd-config bo-config | restack ms (outside frames) | 0.80 | 0.80 | 0.80 |
| cpu0 bd-config bo-config | peek_ready misses | 0.00 | 0.00 | 0.00 |
| cpu0 bd-config bo-config | dwm gpu busy p90 ms | 7.99 | 7.91 | 7.96 |
| cpu0 bd-config bo-config | dwm missed vblanks/burst | 5.00 | 4.00 | 4.00 |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 20.21 | 19.44 | 19.53 |
| cpu8 bd-config bo-config | frame p90 ms | 0.53 | 0.51 | 0.52 |
| cpu8 bd-config bo-config | interval p90 ms | 5.76 | 5.75 | 5.76 |
| cpu8 bd-config bo-config | interval p99 ms | 12.46 | 12.57 | 12.44 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 2.10 | 2.30 | 2.20 |
| cpu8 bd-config bo-config | take_over ms (outside frames) | 3.00 | 2.90 | 2.90 |
| cpu8 bd-config bo-config | peek_cover ms (outside frames) | 1.50 | 1.20 | 1.20 |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | — | — |
| cpu8 bd-config bo-config | restack ms (outside frames) | 0.80 | 0.80 | 0.80 |
| cpu8 bd-config bo-config | peek_ready misses | 0.00 | 0.00 | 0.00 |
| cpu8 bd-config bo-config | dwm gpu busy p90 ms | 8.11 | 8.18 | 8.14 |
| cpu8 bd-config bo-config | dwm missed vblanks/burst | 5.00 | 4.00 | 5.00 |

### move, cpu0 bd-config bo-config

| metric | this | main | restructure |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 18.29 | 17.85 | 17.69 |
| command p90 ms | 19.24 | 18.88 | 18.97 |
| frames | 63.00 | 63.00 | 63.00 |
| tick ms/frame | 0.38 | 0.39 | 0.38 |
| frame p90 ms | 0.52 | 0.52 | 0.52 |
| frames >2x budget | 0.00 | 0.00 | 0.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.74 | 5.74 | 5.74 |
| interval p99 ms | 10.57 | 10.60 | 10.57 |
| interval max ms (median burst) | 10.57 | 10.60 | 10.57 |
| peek_frame ms/frame | 0.38 | 0.34 | 0.37 |
| peek_draw ms/frame | 0.16 | 0.16 | 0.16 |
| peek_track ms/frame | 0.05 | 0.05 | 0.05 |
| peek_ring ms/frame | 0.01 | 0.01 | 0.01 |
| peek_backdrop ms/frame | 0.03 | 0.03 | 0.03 |
| peek_draw ms (worst frame) | 0.64 | 0.62 | 0.61 |
| pd_alpha ms (worst frame) | 0.22 | 0.26 | 0.25 |
| pd_draw ms (worst frame) | 0.54 | 0.52 | 0.54 |
| pd_fill ms (worst frame) | — | — | — |
| fill_source ms (worst frame) | — | — | — |
| ring_raise ms (worst frame) | — | — | — |
| restack ms (worst frame) | — | — | — |
| batch_commit ms/frame | — | — | — |
| ovl_region ms/frame | — | — | — |
| dwm_flush calls (in frames) | — | — | — |
| dwm_flush ms (outside frames) | 5.60 | 4.90 | 5.20 |
| cloak ms (outside frames) | 8.50 | 7.60 | 8.20 |
| session_begin ms (outside frames) | — | — | — |
| session_overlays ms (outside frames) | — | — | — |
| session_overlays ms/frame | — | — | — |
| command ms (WM side) | 18.00 | 17.60 | 17.30 |
| peek_new ms (outside frames) | 2.10 | 2.10 | 2.10 |
| peek_place ms (outside frames) | 1.10 | 1.10 | 1.10 |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 2.70 | 2.60 | 2.70 |
| to_draw ms (outside frames) | 0.30 | 0.30 | 0.30 |
| to_alpha ms (outside frames) | 0.50 | 0.50 | 0.50 |
| to_cloak ms (outside frames) | 1.80 | 1.80 | 1.90 |
| peek_cover ms (outside frames) | 1.10 | 1.30 | 1.00 |
| ring_prep ms (outside frames) | 0.00 | 0.00 | 0.00 |
| ring_park ms (outside frames) | 1.30 | 1.30 | 1.10 |
| ring_copy ms (outside frames) | 0.30 | 0.10 | 0.20 |
| bd_copy ms (outside frames) | 0.10 | 0.20 | 0.20 |
| ovl_restack ms (outside frames) | 0.30 | 0.60 | 0.30 |
| batch_commit ms (outside frames) | — | — | — |
| native_frame ms (outside frames) | 0.20 | 0.10 | 0.10 |
| peek_handover ms (outside frames) | 3.00 | 2.50 | 2.60 |
| peek_raise ms (outside frames) | 0.20 | 0.30 | 0.30 |
| surface ms (outside frames) | 1.10 | 1.10 | 1.10 |
| restack ms (outside frames) | 0.80 | 0.80 | 0.80 |
| peek_ready hits | 2.00 | 2.00 | 2.00 |
| peek_ready misses | 0.00 | 0.00 | 0.00 |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 3.58 | 3.47 | 3.52 |
| input->first frame ms | 18.69 | 18.87 | 18.35 |
| dwm missed vblanks/burst | 5.00 | 4.00 | 4.00 |
| dwm display interval p90 ms | 11.42 | 11.42 | 11.42 |
| dwm gpu busy p90 ms | 7.99 | 7.91 | 7.96 |

### move, cpu8 bd-config bo-config

| metric | this | main | restructure |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 19.02 | 18.71 | 18.62 |
| command p90 ms | 20.79 | 20.33 | 20.26 |
| frames | 63.00 | 63.00 | 63.00 |
| tick ms/frame | 0.42 | 0.43 | 0.42 |
| frame p90 ms | 0.53 | 0.51 | 0.52 |
| frames >2x budget | 0.00 | 0.00 | 0.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.76 | 5.75 | 5.76 |
| interval p99 ms | 12.46 | 12.57 | 12.44 |
| interval max ms (median burst) | 12.46 | 12.57 | 12.44 |
| peek_frame ms/frame | 0.34 | 0.31 | 0.31 |
| peek_draw ms/frame | 0.17 | 0.17 | 0.17 |
| peek_track ms/frame | 0.06 | 0.05 | 0.05 |
| peek_ring ms/frame | 0.01 | 0.01 | 0.01 |
| peek_backdrop ms/frame | 0.04 | 0.03 | 0.03 |
| peek_draw ms (worst frame) | 0.63 | 0.69 | 0.62 |
| pd_alpha ms (worst frame) | 0.29 | 0.30 | 0.28 |
| pd_draw ms (worst frame) | 0.47 | 0.50 | 0.49 |
| pd_fill ms (worst frame) | — | — | — |
| fill_source ms (worst frame) | — | — | — |
| ring_raise ms (worst frame) | — | — | — |
| restack ms (worst frame) | — | — | — |
| batch_commit ms/frame | — | — | — |
| ovl_region ms/frame | — | — | — |
| dwm_flush calls (in frames) | — | — | — |
| dwm_flush ms (outside frames) | 4.70 | 4.70 | 4.70 |
| cloak ms (outside frames) | 7.60 | 7.70 | 7.80 |
| session_begin ms (outside frames) | — | — | — |
| session_overlays ms (outside frames) | — | — | — |
| session_overlays ms/frame | — | — | — |
| command ms (WM side) | 18.70 | 18.50 | 18.30 |
| peek_new ms (outside frames) | 2.10 | 2.30 | 2.20 |
| peek_place ms (outside frames) | 1.20 | 1.20 | 1.20 |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 3.00 | 2.90 | 2.90 |
| to_draw ms (outside frames) | 0.30 | 0.30 | 0.30 |
| to_alpha ms (outside frames) | 0.50 | 0.50 | 0.50 |
| to_cloak ms (outside frames) | 2.10 | 2.10 | 2.10 |
| peek_cover ms (outside frames) | 1.50 | 1.20 | 1.20 |
| ring_prep ms (outside frames) | 0.00 | 0.00 | 0.00 |
| ring_park ms (outside frames) | 1.50 | 1.60 | 1.50 |
| ring_copy ms (outside frames) | 0.30 | 0.20 | 0.20 |
| bd_copy ms (outside frames) | 0.10 | 0.20 | 0.20 |
| ovl_restack ms (outside frames) | 0.80 | 0.40 | 0.40 |
| batch_commit ms (outside frames) | — | — | — |
| native_frame ms (outside frames) | 0.20 | 0.10 | 0.10 |
| peek_handover ms (outside frames) | 3.30 | 3.20 | 3.30 |
| peek_raise ms (outside frames) | 0.30 | 0.30 | 0.30 |
| surface ms (outside frames) | 1.30 | 1.30 | 1.20 |
| restack ms (outside frames) | 0.80 | 0.80 | 0.80 |
| peek_ready hits | 2.00 | 2.00 | 2.00 |
| peek_ready misses | 0.00 | 0.00 | 0.00 |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 3.53 | 3.66 | 3.76 |
| input->first frame ms | 20.21 | 19.44 | 19.53 |
| dwm missed vblanks/burst | 5.00 | 4.00 | 5.00 |
| dwm display interval p90 ms | 11.43 | 11.43 | 11.43 |
| dwm gpu busy p90 ms | 8.11 | 8.18 | 8.14 |

## resize

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | main | restructure |
| --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 67.37 | 55.55 | 54.16 |
| cpu0 bd-config bo-config | frame p90 ms | 3.25 | 3.26 | 3.23 |
| cpu0 bd-config bo-config | interval p90 ms | 5.77 | 5.78 | 5.78 |
| cpu0 bd-config bo-config | interval p99 ms | 35.09 | 40.40 | 37.45 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 3.30 | 3.30 | 3.40 |
| cpu0 bd-config bo-config | take_over ms (outside frames) | 13.00 | 12.60 | 12.60 |
| cpu0 bd-config bo-config | peek_cover ms (outside frames) | 11.20 | 7.90 | 7.80 |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | — | — |
| cpu0 bd-config bo-config | restack ms (outside frames) | — | — | — |
| cpu0 bd-config bo-config | peek_ready misses | 0.00 | 0.00 | 0.00 |
| cpu0 bd-config bo-config | dwm gpu busy p90 ms | 1.37 | 1.36 | 1.52 |
| cpu0 bd-config bo-config | dwm missed vblanks/burst | 15.00 | 12.00 | 22.00 |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 78.85 | 65.26 | 60.82 |
| cpu8 bd-config bo-config | frame p90 ms | 4.19 | 4.26 | 4.20 |
| cpu8 bd-config bo-config | interval p90 ms | 5.77 | 5.78 | 5.79 |
| cpu8 bd-config bo-config | interval p99 ms | 42.11 | 42.66 | 38.71 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 4.10 | 3.90 | 3.80 |
| cpu8 bd-config bo-config | take_over ms (outside frames) | 13.50 | 14.20 | 13.80 |
| cpu8 bd-config bo-config | peek_cover ms (outside frames) | 12.50 | 9.00 | 9.10 |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | — | — |
| cpu8 bd-config bo-config | restack ms (outside frames) | — | — | — |
| cpu8 bd-config bo-config | peek_ready misses | 0.00 | 0.00 | 0.00 |
| cpu8 bd-config bo-config | dwm gpu busy p90 ms | 1.42 | 1.43 | 1.39 |
| cpu8 bd-config bo-config | dwm missed vblanks/burst | 22.00 | 24.00 | 2.00 |

### resize, cpu0 bd-config bo-config

| metric | this | main | restructure |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 54.59 | 40.36 | 40.65 |
| command p90 ms | 56.29 | 45.66 | 42.57 |
| frames | 61.00 | 62.00 | 61.00 |
| tick ms/frame | 2.27 | 2.23 | 2.26 |
| frame p90 ms | 3.25 | 3.26 | 3.23 |
| frames >2x budget | 3.00 | 3.00 | 3.00 |
| interval p50 ms | 5.72 | 5.72 | 5.71 |
| interval p90 ms | 5.77 | 5.78 | 5.78 |
| interval p99 ms | 35.09 | 40.40 | 37.45 |
| interval max ms (median burst) | 35.09 | 40.40 | 37.45 |
| peek_frame ms/frame | 1.91 | 1.90 | 1.88 |
| peek_draw ms/frame | 1.25 | 1.24 | 1.19 |
| peek_track ms/frame | 0.27 | 0.27 | 0.27 |
| peek_ring ms/frame | 0.13 | 0.13 | 0.12 |
| peek_backdrop ms/frame | 0.13 | 0.11 | 0.11 |
| peek_draw ms (worst frame) | 17.43 | 15.84 | 12.44 |
| pd_alpha ms (worst frame) | 1.05 | 0.95 | 1.08 |
| pd_draw ms (worst frame) | 16.23 | 15.03 | 11.21 |
| pd_fill ms (worst frame) | 12.90 | 11.68 | 7.94 |
| fill_source ms (worst frame) | 3.37 | 2.58 | 1.95 |
| ring_raise ms (worst frame) | 5.92 | 5.35 | 4.76 |
| restack ms (worst frame) | 0.00 | 2.78 | 0.00 |
| batch_commit ms/frame | — | — | — |
| ovl_region ms/frame | 0.04 | 0.03 | 0.04 |
| dwm_flush calls (in frames) | 2.00 | 2.00 | 2.00 |
| dwm_flush ms (outside frames) | 10.50 | — | — |
| cloak ms (outside frames) | 24.00 | 13.10 | 13.30 |
| session_begin ms (outside frames) | — | — | — |
| session_overlays ms (outside frames) | — | — | — |
| session_overlays ms/frame | — | — | — |
| command ms (WM side) | 54.30 | 40.00 | 40.30 |
| peek_new ms (outside frames) | 3.30 | 3.30 | 3.40 |
| peek_place ms (outside frames) | 2.00 | 2.00 | 2.00 |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 13.00 | 12.60 | 12.60 |
| to_draw ms (outside frames) | 1.00 | 0.80 | 0.90 |
| to_alpha ms (outside frames) | 2.30 | 2.40 | 2.20 |
| to_cloak ms (outside frames) | 8.60 | 8.70 | 8.60 |
| peek_cover ms (outside frames) | 11.20 | 7.90 | 7.80 |
| ring_prep ms (outside frames) | 0.00 | 0.00 | 0.00 |
| ring_park ms (outside frames) | 0.90 | 1.10 | 1.00 |
| ring_copy ms (outside frames) | 6.70 | 3.50 | 3.50 |
| bd_copy ms (outside frames) | 0.50 | 0.60 | 0.60 |
| ovl_restack ms (outside frames) | 2.10 | 2.00 | 2.00 |
| batch_commit ms (outside frames) | — | — | — |
| native_frame ms (outside frames) | 1.60 | 0.90 | 0.80 |
| peek_handover ms (outside frames) | 12.60 | 13.30 | 13.10 |
| peek_raise ms (outside frames) | 0.10 | 0.10 | 0.10 |
| surface ms (outside frames) | 0.70 | 0.70 | 0.80 |
| restack ms (outside frames) | — | — | — |
| peek_ready hits | 6.00 | 6.00 | 6.00 |
| peek_ready misses | 0.00 | 0.00 | 0.00 |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 3.10 | 2.98 | 2.92 |
| input->first frame ms | 67.37 | 55.55 | 54.16 |
| dwm missed vblanks/burst | 15.00 | 12.00 | 22.00 |
| dwm display interval p90 ms | 17.14 | 17.15 | 17.16 |
| dwm gpu busy p90 ms | 1.37 | 1.36 | 1.52 |

### resize, cpu8 bd-config bo-config

| metric | this | main | restructure |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 57.34 | 44.75 | 44.73 |
| command p90 ms | 63.87 | 46.81 | 46.43 |
| frames | 60.00 | 61.00 | 61.00 |
| tick ms/frame | 2.74 | 2.63 | 2.56 |
| frame p90 ms | 4.19 | 4.26 | 4.20 |
| frames >2x budget | 3.00 | 3.00 | 3.00 |
| interval p50 ms | 5.71 | 5.72 | 5.71 |
| interval p90 ms | 5.77 | 5.78 | 5.79 |
| interval p99 ms | 42.11 | 42.66 | 38.71 |
| interval max ms (median burst) | 42.11 | 42.66 | 38.71 |
| peek_frame ms/frame | 2.13 | 2.18 | 2.11 |
| peek_draw ms/frame | 1.49 | 1.52 | 1.46 |
| peek_track ms/frame | 0.32 | 0.31 | 0.28 |
| peek_ring ms/frame | 0.16 | 0.15 | 0.13 |
| peek_backdrop ms/frame | 0.14 | 0.13 | 0.12 |
| peek_draw ms (worst frame) | 19.39 | 21.32 | 15.52 |
| pd_alpha ms (worst frame) | 1.15 | 1.20 | 1.20 |
| pd_draw ms (worst frame) | 18.02 | 20.11 | 14.31 |
| pd_fill ms (worst frame) | 13.78 | 16.16 | 10.18 |
| fill_source ms (worst frame) | 4.38 | 5.04 | 2.56 |
| ring_raise ms (worst frame) | 5.75 | 5.97 | 5.74 |
| restack ms (worst frame) | 0.00 | 0.00 | 0.00 |
| batch_commit ms/frame | — | — | — |
| ovl_region ms/frame | 0.04 | 0.04 | 0.04 |
| dwm_flush calls (in frames) | 2.00 | 2.00 | 2.00 |
| dwm_flush ms (outside frames) | 10.00 | — | — |
| cloak ms (outside frames) | 23.00 | 14.60 | 14.20 |
| session_begin ms (outside frames) | — | — | — |
| session_overlays ms (outside frames) | — | — | — |
| session_overlays ms/frame | — | — | — |
| command ms (WM side) | 56.90 | 44.50 | 44.40 |
| peek_new ms (outside frames) | 4.10 | 3.90 | 3.80 |
| peek_place ms (outside frames) | 2.10 | 2.10 | 2.10 |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 13.50 | 14.20 | 13.80 |
| to_draw ms (outside frames) | 1.00 | 1.00 | 1.00 |
| to_alpha ms (outside frames) | 2.50 | 2.40 | 2.10 |
| to_cloak ms (outside frames) | 9.40 | 9.40 | 9.50 |
| peek_cover ms (outside frames) | 12.50 | 9.00 | 9.10 |
| ring_prep ms (outside frames) | 0.00 | 0.00 | 0.00 |
| ring_park ms (outside frames) | 1.20 | 1.40 | 1.20 |
| ring_copy ms (outside frames) | 7.10 | 4.00 | 4.00 |
| bd_copy ms (outside frames) | 0.60 | 0.60 | 0.70 |
| ovl_restack ms (outside frames) | 2.60 | 2.60 | 2.50 |
| batch_commit ms (outside frames) | — | — | — |
| native_frame ms (outside frames) | 2.00 | 1.40 | 0.80 |
| peek_handover ms (outside frames) | 14.60 | 14.10 | 14.00 |
| peek_raise ms (outside frames) | 0.10 | 0.20 | 0.10 |
| surface ms (outside frames) | 1.00 | 1.00 | 0.90 |
| restack ms (outside frames) | — | — | — |
| peek_ready hits | 6.00 | 6.00 | 6.00 |
| peek_ready misses | 0.00 | 0.00 | 0.00 |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 3.60 | 3.53 | 3.49 |
| input->first frame ms | 78.85 | 65.26 | 60.82 |
| dwm missed vblanks/burst | 22.00 | 24.00 | 2.00 |
| dwm display interval p90 ms | 17.17 | 17.17 | 5.75 |
| dwm gpu busy p90 ms | 1.42 | 1.43 | 1.39 |

## switch

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | main | restructure |
| --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 0.00 | 0.00 | 0.00 |
| cpu0 bd-config bo-config | frame p90 ms | 1.19 | 1.16 | 1.19 |
| cpu0 bd-config bo-config | interval p90 ms | 5.76 | 5.76 | 5.75 |
| cpu0 bd-config bo-config | interval p99 ms | 22.06 | 21.83 | 21.27 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 0.00 | 0.00 | 0.00 |
| cpu0 bd-config bo-config | take_over ms (outside frames) | 0.00 | 0.00 | 0.00 |
| cpu0 bd-config bo-config | peek_cover ms (outside frames) | 0.00 | 0.00 | 0.00 |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | — | — |
| cpu0 bd-config bo-config | restack ms (outside frames) | 0.00 | 0.00 | 0.00 |
| cpu0 bd-config bo-config | peek_ready misses | 0.00 | 0.00 | 0.00 |
| cpu0 bd-config bo-config | dwm gpu busy p90 ms | 2.34 | 5.52 | 1.94 |
| cpu0 bd-config bo-config | dwm missed vblanks/burst | 13.00 | 22.00 | 14.00 |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | — | — | 0.00 |
| cpu8 bd-config bo-config | frame p90 ms | 1.25 | 1.26 | 1.32 |
| cpu8 bd-config bo-config | interval p90 ms | 5.76 | 5.77 | 5.76 |
| cpu8 bd-config bo-config | interval p99 ms | 22.72 | 24.32 | 24.45 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | — | — | 0.00 |
| cpu8 bd-config bo-config | take_over ms (outside frames) | — | — | 0.00 |
| cpu8 bd-config bo-config | peek_cover ms (outside frames) | — | — | 0.00 |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | — | — |
| cpu8 bd-config bo-config | restack ms (outside frames) | 0.00 | 0.00 | 0.00 |
| cpu8 bd-config bo-config | peek_ready misses | 0.00 | 0.00 | 0.00 |
| cpu8 bd-config bo-config | dwm gpu busy p90 ms | 5.90 | 2.75 | 2.72 |
| cpu8 bd-config bo-config | dwm missed vblanks/burst | 24.00 | 27.00 | 23.00 |

### switch, cpu0 bd-config bo-config

| metric | this | main | restructure |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 30.64 | 31.23 | 30.44 |
| command p90 ms | 32.88 | 33.27 | 32.86 |
| frames | 99.00 | 99.00 | 99.00 |
| tick ms/frame | 1.15 | 1.16 | 1.14 |
| frame p90 ms | 1.19 | 1.16 | 1.19 |
| frames >2x budget | 3.00 | 3.00 | 3.00 |
| interval p50 ms | 5.72 | 5.71 | 5.72 |
| interval p90 ms | 5.76 | 5.76 | 5.75 |
| interval p99 ms | 22.06 | 21.83 | 21.27 |
| interval max ms (median burst) | 22.06 | 21.83 | 21.40 |
| peek_frame ms/frame | 0.00 | 0.00 | 0.00 |
| peek_draw ms/frame | 0.00 | 0.00 | 0.00 |
| peek_track ms/frame | 0.00 | 0.00 | 0.00 |
| peek_ring ms/frame | 0.00 | 0.00 | 0.00 |
| peek_backdrop ms/frame | 0.00 | 0.00 | 0.00 |
| peek_draw ms (worst frame) | 0.00 | 0.00 | 0.00 |
| pd_alpha ms (worst frame) | 0.00 | 0.00 | 0.00 |
| pd_draw ms (worst frame) | 0.00 | 0.00 | 0.00 |
| pd_fill ms (worst frame) | 0.00 | 0.00 | 0.00 |
| fill_source ms (worst frame) | 0.00 | 0.00 | 0.00 |
| ring_raise ms (worst frame) | 0.00 | 0.00 | 0.00 |
| restack ms (worst frame) | — | — | — |
| batch_commit ms/frame | — | — | — |
| ovl_region ms/frame | 0.03 | 0.03 | 0.03 |
| dwm_flush calls (in frames) | — | — | — |
| dwm_flush ms (outside frames) | 5.50 | 6.00 | 5.00 |
| cloak ms (outside frames) | 0.00 | 0.00 | 0.00 |
| session_begin ms (outside frames) | — | — | — |
| session_overlays ms (outside frames) | — | — | — |
| session_overlays ms/frame | — | — | — |
| command ms (WM side) | 30.40 | 30.90 | 30.20 |
| peek_new ms (outside frames) | 0.00 | 0.00 | 0.00 |
| peek_place ms (outside frames) | 0.00 | 0.00 | 0.00 |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 0.00 | 0.00 | 0.00 |
| to_draw ms (outside frames) | 0.00 | 0.00 | 0.00 |
| to_alpha ms (outside frames) | 0.00 | 0.00 | 0.00 |
| to_cloak ms (outside frames) | 0.00 | 0.00 | 0.00 |
| peek_cover ms (outside frames) | 0.00 | 0.00 | 0.00 |
| ring_prep ms (outside frames) | 0.00 | 0.00 | 0.00 |
| ring_park ms (outside frames) | 0.00 | 0.00 | 0.00 |
| ring_copy ms (outside frames) | 0.00 | 0.00 | 0.00 |
| bd_copy ms (outside frames) | 0.00 | 0.00 | 0.00 |
| ovl_restack ms (outside frames) | 0.00 | 0.00 | 0.00 |
| batch_commit ms (outside frames) | — | — | — |
| native_frame ms (outside frames) | 0.20 | 0.10 | 0.10 |
| peek_handover ms (outside frames) | 0.00 | 0.00 | 0.00 |
| peek_raise ms (outside frames) | 0.00 | 0.00 | 0.00 |
| surface ms (outside frames) | 0.60 | 0.60 | 0.60 |
| restack ms (outside frames) | 0.00 | 0.00 | 0.00 |
| peek_ready hits | 0.00 | 0.00 | 0.00 |
| peek_ready misses | 0.00 | 0.00 | 0.00 |
| ws_surrogates ms (outside frames) | 9.70 | 9.80 | 9.80 |
| ws_park ms (outside frames) | 2.10 | 2.10 | 2.10 |
| ws_show ms (outside frames) | 7.90 | 8.70 | 7.60 |
| ws_start ms (outside frames) | 0.40 | 0.40 | 0.40 |
| input->anim start ms | 0.00 | 0.00 | 0.00 |
| input->first frame ms | 0.00 | 0.00 | 0.00 |
| dwm missed vblanks/burst | 13.00 | 22.00 | 14.00 |
| dwm display interval p90 ms | 11.44 | 11.44 | 11.44 |
| dwm gpu busy p90 ms | 2.34 | 5.52 | 1.94 |

### switch, cpu8 bd-config bo-config

| metric | this | main | restructure |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 34.87 | 32.89 | 34.18 |
| command p90 ms | 35.84 | 35.37 | 36.52 |
| frames | 99.00 | 99.00 | 99.00 |
| tick ms/frame | 1.27 | 1.27 | 1.30 |
| frame p90 ms | 1.25 | 1.26 | 1.32 |
| frames >2x budget | 3.00 | 3.00 | 3.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.76 | 5.77 | 5.76 |
| interval p99 ms | 22.72 | 24.32 | 24.45 |
| interval max ms (median burst) | 24.09 | 24.32 | 24.45 |
| peek_frame ms/frame | 0.00 | 0.00 | 0.00 |
| peek_draw ms/frame | 0.00 | 0.00 | 0.00 |
| peek_track ms/frame | 0.00 | 0.00 | 0.00 |
| peek_ring ms/frame | 0.00 | 0.00 | 0.00 |
| peek_backdrop ms/frame | 0.00 | 0.00 | 0.00 |
| peek_draw ms (worst frame) | 0.00 | 0.00 | 0.00 |
| pd_alpha ms (worst frame) | 0.00 | 0.00 | 0.00 |
| pd_draw ms (worst frame) | 0.00 | 0.00 | 0.00 |
| pd_fill ms (worst frame) | 0.00 | 0.00 | 0.00 |
| fill_source ms (worst frame) | 0.00 | 0.00 | 0.00 |
| ring_raise ms (worst frame) | 0.00 | 0.00 | 0.00 |
| restack ms (worst frame) | — | — | — |
| batch_commit ms/frame | — | — | — |
| ovl_region ms/frame | 0.03 | 0.03 | 0.04 |
| dwm_flush calls (in frames) | — | — | — |
| dwm_flush ms (outside frames) | 6.00 | 5.60 | 5.50 |
| cloak ms (outside frames) | — | — | 0.00 |
| session_begin ms (outside frames) | — | — | — |
| session_overlays ms (outside frames) | — | — | — |
| session_overlays ms/frame | — | — | — |
| command ms (WM side) | 34.60 | 32.60 | 33.90 |
| peek_new ms (outside frames) | — | — | 0.00 |
| peek_place ms (outside frames) | — | — | 0.00 |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | — | — | 0.00 |
| to_draw ms (outside frames) | — | — | 0.00 |
| to_alpha ms (outside frames) | — | — | 0.00 |
| to_cloak ms (outside frames) | — | — | 0.00 |
| peek_cover ms (outside frames) | — | — | 0.00 |
| ring_prep ms (outside frames) | 0.00 | 0.00 | 0.00 |
| ring_park ms (outside frames) | 0.00 | 0.00 | 0.00 |
| ring_copy ms (outside frames) | 0.00 | 0.00 | 0.00 |
| bd_copy ms (outside frames) | 0.00 | 0.00 | 0.00 |
| ovl_restack ms (outside frames) | 0.00 | 0.00 | 0.00 |
| batch_commit ms (outside frames) | — | — | — |
| native_frame ms (outside frames) | 0.20 | 0.20 | 0.10 |
| peek_handover ms (outside frames) | — | — | 0.00 |
| peek_raise ms (outside frames) | 0.00 | 0.00 | 0.00 |
| surface ms (outside frames) | 0.60 | 0.60 | 0.60 |
| restack ms (outside frames) | 0.00 | 0.00 | 0.00 |
| peek_ready hits | 0.00 | 0.00 | 0.00 |
| peek_ready misses | 0.00 | 0.00 | 0.00 |
| ws_surrogates ms (outside frames) | 11.60 | 11.40 | 11.40 |
| ws_park ms (outside frames) | 2.30 | 2.40 | 2.40 |
| ws_show ms (outside frames) | 8.50 | 8.10 | 8.30 |
| ws_start ms (outside frames) | 0.40 | 0.40 | 0.40 |
| input->anim start ms | — | — | 0.00 |
| input->first frame ms | — | — | 0.00 |
| dwm missed vblanks/burst | 24.00 | 27.00 | 23.00 |
| dwm display interval p90 ms | 11.44 | 11.44 | 11.44 |
| dwm gpu busy p90 ms | 5.90 | 2.75 | 2.72 |

## minimize

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | main | restructure |
| --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 60.61 | 55.64 | 54.00 |
| cpu0 bd-config bo-config | frame p90 ms | 3.80 | 3.83 | 3.81 |
| cpu0 bd-config bo-config | interval p90 ms | 5.78 | 5.79 | 5.79 |
| cpu0 bd-config bo-config | interval p99 ms | 46.04 | 40.31 | 39.11 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 8.50 | 7.70 | 7.40 |
| cpu0 bd-config bo-config | take_over ms (outside frames) | 10.20 | 12.60 | 12.70 |
| cpu0 bd-config bo-config | peek_cover ms (outside frames) | 17.80 | 13.40 | 12.90 |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | — | — |
| cpu0 bd-config bo-config | restack ms (outside frames) | — | — | — |
| cpu0 bd-config bo-config | peek_ready misses | 0.00 | 0.00 | 0.00 |
| cpu0 bd-config bo-config | dwm gpu busy p90 ms | 9.46 | 9.12 | 9.42 |
| cpu0 bd-config bo-config | dwm missed vblanks/burst | 30.00 | 30.00 | 31.00 |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 66.69 | 65.74 | 60.94 |
| cpu8 bd-config bo-config | frame p90 ms | 4.34 | 4.30 | 4.30 |
| cpu8 bd-config bo-config | interval p90 ms | 5.78 | 5.79 | 5.81 |
| cpu8 bd-config bo-config | interval p99 ms | 51.73 | 45.01 | 42.96 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 10.40 | 8.90 | 8.00 |
| cpu8 bd-config bo-config | take_over ms (outside frames) | 12.20 | 13.70 | 14.30 |
| cpu8 bd-config bo-config | peek_cover ms (outside frames) | 17.30 | 14.90 | 14.60 |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | — | — |
| cpu8 bd-config bo-config | restack ms (outside frames) | — | — | — |
| cpu8 bd-config bo-config | peek_ready misses | 0.00 | 0.00 | 0.00 |
| cpu8 bd-config bo-config | dwm gpu busy p90 ms | 9.43 | 9.67 | 8.98 |
| cpu8 bd-config bo-config | dwm missed vblanks/burst | 33.00 | 32.00 | 31.00 |

### minimize, cpu0 bd-config bo-config

| metric | this | main | restructure |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 45.49 | 39.70 | 38.51 |
| command p90 ms | 49.75 | 46.62 | 46.56 |
| frames | 62.00 | 61.00 | 62.00 |
| tick ms/frame | 2.24 | 2.27 | 2.26 |
| frame p90 ms | 3.80 | 3.83 | 3.81 |
| frames >2x budget | 3.00 | 3.00 | 3.00 |
| interval p50 ms | 5.72 | 5.72 | 5.72 |
| interval p90 ms | 5.78 | 5.79 | 5.79 |
| interval p99 ms | 46.04 | 40.31 | 39.11 |
| interval max ms (median burst) | 46.04 | 40.31 | 39.11 |
| peek_frame ms/frame | 1.83 | 1.78 | 1.76 |
| peek_draw ms/frame | 1.14 | 1.12 | 1.14 |
| peek_track ms/frame | 0.34 | 0.33 | 0.34 |
| peek_ring ms/frame | 0.13 | 0.11 | 0.11 |
| peek_backdrop ms/frame | 0.20 | 0.17 | 0.18 |
| peek_draw ms (worst frame) | 11.58 | 13.97 | 12.00 |
| pd_alpha ms (worst frame) | 0.84 | 0.85 | 0.71 |
| pd_draw ms (worst frame) | 10.72 | 13.00 | 11.07 |
| pd_fill ms (worst frame) | 6.94 | 7.88 | 6.92 |
| fill_source ms (worst frame) | 2.21 | 2.27 | 2.24 |
| ring_raise ms (worst frame) | 3.20 | 3.50 | 3.54 |
| restack ms (worst frame) | 2.84 | 2.85 | 2.79 |
| batch_commit ms/frame | — | — | — |
| ovl_region ms/frame | 0.03 | 0.03 | 0.03 |
| dwm_flush calls (in frames) | 2.00 | 2.00 | 2.00 |
| dwm_flush ms (outside frames) | 9.70 | 5.80 | 4.30 |
| cloak ms (outside frames) | 16.50 | 13.00 | 13.00 |
| session_begin ms (outside frames) | — | — | — |
| session_overlays ms (outside frames) | — | — | — |
| session_overlays ms/frame | — | — | — |
| command ms (WM side) | 45.20 | 39.40 | 38.20 |
| peek_new ms (outside frames) | 8.50 | 7.70 | 7.40 |
| peek_place ms (outside frames) | 1.80 | 1.90 | 1.80 |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 10.20 | 12.60 | 12.70 |
| to_draw ms (outside frames) | 0.90 | 0.90 | 0.90 |
| to_alpha ms (outside frames) | 2.20 | 2.50 | 2.00 |
| to_cloak ms (outside frames) | 7.20 | 9.50 | 9.90 |
| peek_cover ms (outside frames) | 17.80 | 13.40 | 12.90 |
| ring_prep ms (outside frames) | 2.50 | 3.30 | 3.10 |
| ring_park ms (outside frames) | 1.10 | 1.20 | 1.30 |
| ring_copy ms (outside frames) | 7.40 | 4.40 | 4.10 |
| bd_copy ms (outside frames) | 0.90 | 0.90 | 0.90 |
| ovl_restack ms (outside frames) | 0.00 | 0.10 | 0.10 |
| batch_commit ms (outside frames) | — | — | — |
| native_frame ms (outside frames) | 1.70 | 0.90 | 1.00 |
| peek_handover ms (outside frames) | 14.40 | 14.40 | 13.90 |
| peek_raise ms (outside frames) | 0.30 | 0.50 | 0.30 |
| surface ms (outside frames) | 3.00 | 3.00 | 2.80 |
| restack ms (outside frames) | — | — | — |
| peek_ready hits | 6.00 | 6.00 | 6.00 |
| peek_ready misses | 0.00 | 0.00 | 0.00 |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 8.67 | 10.31 | 8.59 |
| input->first frame ms | 60.61 | 55.64 | 54.00 |
| dwm missed vblanks/burst | 30.00 | 30.00 | 31.00 |
| dwm display interval p90 ms | 22.87 | 22.90 | 22.89 |
| dwm gpu busy p90 ms | 9.46 | 9.12 | 9.42 |

### minimize, cpu8 bd-config bo-config

| metric | this | main | restructure |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 47.45 | 44.53 | 42.77 |
| command p90 ms | 59.97 | 48.89 | 49.74 |
| frames | 61.00 | 61.00 | 62.00 |
| tick ms/frame | 2.48 | 2.39 | 2.52 |
| frame p90 ms | 4.34 | 4.30 | 4.30 |
| frames >2x budget | 3.00 | 3.00 | 3.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.78 | 5.79 | 5.81 |
| interval p99 ms | 51.73 | 45.01 | 42.96 |
| interval max ms (median burst) | 51.73 | 45.01 | 42.96 |
| peek_frame ms/frame | 2.07 | 2.03 | 2.07 |
| peek_draw ms/frame | 1.29 | 1.25 | 1.25 |
| peek_track ms/frame | 0.37 | 0.33 | 0.36 |
| peek_ring ms/frame | 0.14 | 0.12 | 0.12 |
| peek_backdrop ms/frame | 0.22 | 0.17 | 0.19 |
| peek_draw ms (worst frame) | 14.97 | 14.11 | 11.19 |
| pd_alpha ms (worst frame) | 0.95 | 1.03 | 1.04 |
| pd_draw ms (worst frame) | 13.62 | 13.30 | 10.01 |
| pd_fill ms (worst frame) | 9.58 | 8.64 | 5.39 |
| fill_source ms (worst frame) | 3.14 | 2.43 | 0.00 |
| ring_raise ms (worst frame) | 3.59 | 4.17 | 3.94 |
| restack ms (worst frame) | 3.02 | 3.14 | 2.96 |
| batch_commit ms/frame | — | — | — |
| ovl_region ms/frame | 0.03 | 0.03 | 0.03 |
| dwm_flush calls (in frames) | 2.00 | 2.00 | 2.00 |
| dwm_flush ms (outside frames) | 13.70 | 4.00 | 4.10 |
| cloak ms (outside frames) | 21.60 | 14.10 | 14.60 |
| session_begin ms (outside frames) | — | — | — |
| session_overlays ms (outside frames) | — | — | — |
| session_overlays ms/frame | — | — | — |
| command ms (WM side) | 47.20 | 44.20 | 42.50 |
| peek_new ms (outside frames) | 10.40 | 8.90 | 8.00 |
| peek_place ms (outside frames) | 1.90 | 1.90 | 1.90 |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 12.20 | 13.70 | 14.30 |
| to_draw ms (outside frames) | 1.00 | 1.00 | 1.00 |
| to_alpha ms (outside frames) | 2.60 | 2.60 | 2.80 |
| to_cloak ms (outside frames) | 8.70 | 11.20 | 10.80 |
| peek_cover ms (outside frames) | 17.30 | 14.90 | 14.60 |
| ring_prep ms (outside frames) | 4.20 | 4.10 | 3.20 |
| ring_park ms (outside frames) | 1.30 | 1.40 | 1.30 |
| ring_copy ms (outside frames) | 8.20 | 4.50 | 4.60 |
| bd_copy ms (outside frames) | 1.00 | 1.00 | 1.00 |
| ovl_restack ms (outside frames) | 0.10 | 0.20 | 0.10 |
| batch_commit ms (outside frames) | — | — | — |
| native_frame ms (outside frames) | 2.10 | 1.30 | 1.30 |
| peek_handover ms (outside frames) | 14.90 | 15.10 | 14.90 |
| peek_raise ms (outside frames) | 0.40 | 0.40 | 0.40 |
| surface ms (outside frames) | 3.20 | 3.30 | 3.20 |
| restack ms (outside frames) | — | — | — |
| peek_ready hits | 6.00 | 6.00 | 6.00 |
| peek_ready misses | 0.00 | 0.00 | 0.00 |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 8.83 | 8.35 | 8.87 |
| input->first frame ms | 66.69 | 65.74 | 60.94 |
| dwm missed vblanks/burst | 33.00 | 32.00 | 31.00 |
| dwm display interval p90 ms | 28.58 | 28.59 | 28.58 |
| dwm gpu busy p90 ms | 9.43 | 9.67 | 8.98 |
