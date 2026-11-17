Branch comparison, 2026-11-17T15:29:51: this = 51191dff, against origin/tryout/stand-ins (65a60de), origin/main (cfd3cef); builds this, stand-ins:nopeeks (GLAZEWM_ISOLATE=nopeeks), main. Target chrome, 3 round(s) of 10 bursts per scenario, build and setting, IPC load 0, CPU load 0,8 thread(s) of 16, with DWM timing, tile order Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot, launched none (0 window(s)).
tiles this cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-this-move-mid.png (mid at 64 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-this-move-rest.png
label=this cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.13
  command p90 ms                          18.84
  frames                                  63.00
  tick ms/frame                            0.39
  frame p90 ms                             0.52
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.74
  interval max ms (median burst)          10.74
  peek_frame ms/frame                      0.33
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
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
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)                7.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.80
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.60
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             1.80
  peek_cover ms (outside frames)           1.40
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            0.30
  bd_copy ms (outside frames)              0.10
  ovl_restack ms (outside frames)          0.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)        2.70
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              1.00
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.45
  input->first frame ms                   18.59
  dwm missed vblanks/burst                 6.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.53
| this cpu0 bd-config bo-config #1 | move | 18.13 | 18.84 | 63.00 | 0.39 | 0.52 | 0.00 | 5.71 | 5.74 | 10.74 | 10.74 | 0.33 | 0.16 | 0.05 | 0.01 | 0.03 | 0.61 | 0.26 | 0.49 | - | - | - | - | - | - | - | 4.70 | 7.70 | - | - | - | 17.80 | 2.10 | 1.10 | - | - | - | 2.60 | 0.30 | 0.50 | 1.80 | 1.40 | 0.00 | 1.20 | 0.30 | 0.10 | 0.60 | - | 0.20 | 2.70 | 0.20 | 1.00 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.45 | 18.59 | 6.00 | 11.43 | 8.53 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-this-resize-mid.png (mid at 69 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-this-resize-rest.png
label=this cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          52.08
  command p90 ms                          56.68
  frames                                  62.00
  tick ms/frame                            2.27
  frame p90 ms                             3.27
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         36.61
  interval max ms (median burst)          36.61
  peek_frame ms/frame                      1.86
  peek_draw ms/frame                       1.21
  peek_track ms/frame                      0.25
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.12
  peek_draw ms (worst frame)              14.94
  pd_alpha ms (worst frame)                0.98
  pd_draw ms (worst frame)                13.96
  pd_fill ms (worst frame)                10.58
  fill_source ms (worst frame)             2.09
  ring_raise ms (worst frame)              4.98
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.80
  cloak ms (outside frames)               19.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    51.80
  peek_new ms (outside frames)             3.40
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.80
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.10
  to_cloak ms (outside frames)             8.90
  peek_cover ms (outside frames)          10.90
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            6.50
  bd_copy ms (outside frames)              0.50
  ovl_restack ms (outside frames)          2.20
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.50
  peek_handover ms (outside frames)       14.90
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.27
  input->first frame ms                   67.00
  dwm missed vblanks/burst                28.00
  dwm display interval p90 ms             17.16
  dwm gpu busy p90 ms                      1.52
| this cpu0 bd-config bo-config #1 | resize | 52.08 | 56.68 | 62.00 | 2.27 | 3.27 | 3.00 | 5.71 | 5.79 | 36.61 | 36.61 | 1.86 | 1.21 | 0.25 | 0.12 | 0.12 | 14.94 | 0.98 | 13.96 | 10.58 | 2.09 | 4.98 | 0.00 | - | 0.03 | 2.00 | 8.80 | 19.30 | - | - | - | 51.80 | 3.40 | 2.00 | - | - | - | 12.80 | 0.90 | 2.10 | 8.90 | 10.90 | 0.00 | 1.30 | 6.50 | 0.50 | 2.20 | - | 1.50 | 14.90 | 0.10 | 0.70 | - | 6.00 | 0.00 | - | - | - | - | 3.27 | 67.00 | 28.00 | 17.16 | 1.52 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-this-switch-mid.png (mid at 65 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-this-switch-rest.png
label=this cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          30.43
  command p90 ms                          30.75
  frames                                  98.00
  tick ms/frame                            1.09
  frame p90 ms                             1.09
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         21.44
  interval max ms (median burst)          21.44
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.10
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.50
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)       10.30
  ws_park ms (outside frames)              2.10
  ws_show ms (outside frames)              7.50
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                19.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      2.54
| this cpu0 bd-config bo-config #1 | switch | 30.43 | 30.75 | 98.00 | 1.09 | 1.09 | 2.00 | 5.72 | 5.76 | 21.44 | 21.44 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 4.90 | - | - | - | - | 30.10 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | - | 0.10 | - | 0.00 | 0.50 | - | - | - | 10.30 | 2.10 | 7.50 | 0.40 | - | - | 19.00 | 11.43 | 2.54 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-this-minimize-mid.png (mid at 65 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-this-minimize-rest.png
label=this cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          43.76
  command p90 ms                          50.55
  frames                                  62.00
  tick ms/frame                            2.20
  frame p90 ms                             3.81
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         62.56
  interval max ms (median burst)          62.56
  peek_frame ms/frame                      1.81
  peek_draw ms/frame                       1.14
  peek_track ms/frame                      0.34
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.19
  peek_draw ms (worst frame)              10.17
  pd_alpha ms (worst frame)                0.98
  pd_draw ms (worst frame)                 9.27
  pd_fill ms (worst frame)                 5.52
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              4.62
  restack ms (worst frame)                 2.96
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           12.10
  cloak ms (outside frames)               19.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    43.50
  peek_new ms (outside frames)            10.40
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           10.50
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.30
  to_cloak ms (outside frames)             7.40
  peek_cover ms (outside frames)          16.20
  ring_prep ms (outside frames)            3.30
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            7.50
  bd_copy ms (outside frames)              0.80
  ovl_restack ms (outside frames)          0.40
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.70
  peek_handover ms (outside frames)       16.30
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              4.50
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.42
  input->first frame ms                   59.60
  dwm missed vblanks/burst                25.00
  dwm display interval p90 ms             28.57
  dwm gpu busy p90 ms                      7.87
| this cpu0 bd-config bo-config #1 | minimize | 43.76 | 50.55 | 62.00 | 2.20 | 3.81 | 3.00 | 5.72 | 5.78 | 62.56 | 62.56 | 1.81 | 1.14 | 0.34 | 0.14 | 0.19 | 10.17 | 0.98 | 9.27 | 5.52 | 0.00 | 4.62 | 2.96 | - | 0.03 | 2.00 | 12.10 | 19.60 | - | - | - | 43.50 | 10.40 | 1.80 | - | - | - | 10.50 | 1.00 | 2.30 | 7.40 | 16.20 | 3.30 | 1.30 | 7.50 | 0.80 | 0.40 | - | 1.70 | 16.30 | 0.30 | 4.50 | - | 6.00 | 0.00 | - | - | - | - | 9.42 | 59.60 | 25.00 | 28.57 | 7.87 |
tiles stand-ins:nopeeks cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-move-mid.png (mid at 74 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-move-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          14.82
  command p90 ms                          15.37
  frames                                  62.00
  tick ms/frame                            0.57
  frame p90 ms                             1.14
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          8.19
  interval max ms (median burst)           8.19
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.34
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)                9.00
  session_begin ms (outside frames)        0.60
  session_overlays ms (outside frames)     3.50
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.00
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.28
  input->first frame ms                   15.66
  dwm missed vblanks/burst                 9.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.64
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | move | 14.82 | 15.37 | 62.00 | 0.57 | 1.14 | 0.00 | 5.71 | 5.74 | 8.19 | 8.19 | - | - | - | - | - | - | - | - | - | - | - | - | 0.34 | 0.00 | - | 4.50 | 9.00 | 0.60 | 3.50 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | 0.28 | 15.66 | 9.00 | 11.43 | 8.64 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-resize-mid.png (mid at 70 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-resize-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          44.25
  command p90 ms                          45.67
  frames                                  62.00
  tick ms/frame                            1.81
  frame p90 ms                             4.35
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.81
  interval p99 ms                         20.19
  interval max ms (median burst)          20.19
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    1.02
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)               22.50
  session_begin ms (outside frames)        1.80
  session_overlays ms (outside frames)    17.30
  session_overlays ms/frame                0.06
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.20
  native_frame ms (outside frames)         0.30
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.27
  input->first frame ms                   46.89
  dwm missed vblanks/burst                11.00
  dwm display interval p90 ms              5.79
  dwm gpu busy p90 ms                      2.86
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | resize | 44.25 | 45.67 | 62.00 | 1.81 | 4.35 | 2.00 | 5.71 | 5.81 | 20.19 | 20.19 | - | - | - | - | - | - | - | - | - | - | - | - | 1.02 | 0.03 | - | 5.40 | 22.50 | 1.80 | 17.30 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.20 | 0.30 | - | - | - | - | - | - | - | - | - | - | 0.27 | 46.89 | 11.00 | 5.79 | 2.86 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-switch-mid.png (mid at 73 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-switch-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.66
  command p90 ms                          31.47
  frames                                  99.00
  tick ms/frame                            1.22
  frame p90 ms                             1.27
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         21.56
  interval max ms (median burst)          21.69
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            6.10
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                27.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      2.50
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | switch | 30.66 | 31.47 | 99.00 | 1.22 | 1.27 | 3.00 | 5.72 | 5.76 | 21.56 | 21.69 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 6.10 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.10 | - | - | - | - | - | - | - | - | - | - | - | - | 27.00 | 11.44 | 2.50 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-minimize-mid.png (mid at 73 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-minimize-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          23.40
  command p90 ms                          24.92
  frames                                 144.00
  tick ms/frame                            1.24
  frame p90 ms                             3.24
  frames >2x budget                        0.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         58.93
  interval max ms (median burst)          58.93
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.67
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.00
  cloak ms (outside frames)               25.80
  session_begin ms (outside frames)        5.90
  session_overlays ms (outside frames)    17.90
  session_overlays ms/frame                0.05
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.60
  native_frame ms (outside frames)         0.30
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     5.67
  input->first frame ms                   66.01
  dwm missed vblanks/burst                16.00
  dwm display interval p90 ms             11.46
  dwm gpu busy p90 ms                      6.64
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | minimize | 23.40 | 24.92 | 144.00 | 1.24 | 3.24 | 0.00 | 5.72 | 5.76 | 58.93 | 58.93 | - | - | - | - | - | - | - | - | - | - | - | - | 0.67 | 0.02 | 1.00 | 5.00 | 25.80 | 5.90 | 17.90 | 0.05 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.60 | 0.30 | - | - | - | - | - | - | - | - | - | - | 5.67 | 66.01 | 16.00 | 11.46 | 6.64 |
tiles main cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-main-default-move-mid.png (mid at 75 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-main-default-move-rest.png
label=main cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          12.30
  command p90 ms                          13.42
  frames                                  63.00
  tick ms/frame                            0.66
  frame p90 ms                             1.20
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.73
  interval p99 ms                          7.69
  interval max ms (median burst)           7.69
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.38
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)                7.70
  session_begin ms (outside frames)        0.90
  session_overlays ms (outside frames)     1.90
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.00
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.22
  input->first frame ms                   13.07
  dwm missed vblanks/burst                11.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      7.38
| main cpu0 bd-config bo-config #1 | move | 12.30 | 13.42 | 63.00 | 0.66 | 1.20 | 0.00 | 5.71 | 5.73 | 7.69 | 7.69 | - | - | - | - | - | - | - | - | - | - | - | - | 0.38 | 0.00 | 1.00 | 4.80 | 7.70 | 0.90 | 1.90 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | 0.22 | 13.07 | 11.00 | 11.44 | 7.38 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-main-default-resize-mid.png (mid at 65 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-main-default-resize-rest.png
label=main cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.26
  command p90 ms                          32.11
  frames                                  63.00
  tick ms/frame                            2.21
  frame p90 ms                             4.48
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         25.50
  interval max ms (median burst)          25.50
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    1.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)               17.70
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)     7.40
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         1.20
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.22
  input->first frame ms                   36.07
  dwm missed vblanks/burst                 6.00
  dwm display interval p90 ms              5.76
  dwm gpu busy p90 ms                      2.52
| main cpu0 bd-config bo-config #1 | resize | 30.26 | 32.11 | 63.00 | 2.21 | 4.48 | 1.00 | 5.71 | 5.76 | 25.50 | 25.50 | - | - | - | - | - | - | - | - | - | - | - | - | 1.00 | 0.03 | 2.00 | 4.80 | 17.70 | 2.20 | 7.40 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 1.20 | 0.20 | - | - | - | - | - | - | - | - | - | - | 0.22 | 36.07 | 6.00 | 5.76 | 2.52 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-main-default-switch-mid.png (mid at 76 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-main-default-switch-rest.png
label=main cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.51
  command p90 ms                          21.47
  frames                                  97.00
  tick ms/frame                            1.69
  frame p90 ms                             2.53
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         50.95
  interval max ms (median burst)          50.95
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.65
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            6.30
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                 9.00
  dwm display interval p90 ms              5.76
  dwm gpu busy p90 ms                      4.13
| main cpu0 bd-config bo-config #1 | switch | 20.51 | 21.47 | 97.00 | 1.69 | 2.53 | 1.00 | 5.71 | 5.74 | 50.95 | 50.95 | - | - | - | - | - | - | - | - | - | - | - | - | 0.65 | 0.01 | 3.00 | 6.30 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.10 | - | - | - | - | - | - | - | - | - | - | - | - | 9.00 | 5.76 | 4.13 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-main-default-minimize-mid.png (mid at 67 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu0-bdconfigboconfig-main-default-minimize-rest.png
label=main cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          18.86
  command p90 ms                          19.73
  frames                                 150.00
  tick ms/frame                            1.58
  frame p90 ms                             4.38
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         40.97
  interval max ms (median burst)          40.97
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.76
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            6.10
  cloak ms (outside frames)               22.90
  session_begin ms (outside frames)        3.80
  session_overlays ms (outside frames)     7.20
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.50
  native_frame ms (outside frames)         0.40
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.57
  input->first frame ms                   53.40
  dwm missed vblanks/burst                17.00
  dwm display interval p90 ms             11.45
  dwm gpu busy p90 ms                      5.08
| main cpu0 bd-config bo-config #1 | minimize | 18.86 | 19.73 | 150.00 | 1.58 | 4.38 | 2.00 | 5.72 | 5.76 | 40.97 | 40.97 | - | - | - | - | - | - | - | - | - | - | - | - | 0.76 | 0.03 | 4.00 | 6.10 | 22.90 | 3.80 | 7.20 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.50 | 0.40 | - | - | - | - | - | - | - | - | - | - | 6.57 | 53.40 | 17.00 | 11.45 | 5.08 |
tiles stand-ins:nopeeks cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          13.70
  command p90 ms                          14.46
  frames                                  62.00
  tick ms/frame                            0.56
  frame p90 ms                             1.16
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                          7.72
  interval max ms (median burst)           7.72
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.34
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.70
  cloak ms (outside frames)                7.80
  session_begin ms (outside frames)        0.60
  session_overlays ms (outside frames)     3.30
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.00
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.27
  input->first frame ms                   14.36
  dwm missed vblanks/burst                11.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      8.68
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | move | 13.70 | 14.46 | 62.00 | 0.56 | 1.16 | 0.00 | 5.71 | 5.76 | 7.72 | 7.72 | - | - | - | - | - | - | - | - | - | - | - | - | 0.34 | 0.01 | - | 3.70 | 7.80 | 0.60 | 3.30 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | 0.27 | 14.36 | 11.00 | 11.44 | 8.68 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          39.25
  command p90 ms                          43.63
  frames                                  62.00
  tick ms/frame                            1.72
  frame p90 ms                             4.54
  frames >2x budget                        1.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         19.22
  interval max ms (median burst)          19.22
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    1.03
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)               20.00
  session_begin ms (outside frames)        1.90
  session_overlays ms (outside frames)    14.80
  session_overlays ms/frame                0.07
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.10
  native_frame ms (outside frames)         0.30
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.28
  input->first frame ms                   43.88
  dwm missed vblanks/burst                16.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      1.01
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | resize | 39.25 | 43.63 | 62.00 | 1.72 | 4.54 | 1.00 | 5.72 | 5.76 | 19.22 | 19.22 | - | - | - | - | - | - | - | - | - | - | - | - | 1.03 | 0.03 | - | 4.50 | 20.00 | 1.90 | 14.80 | 0.07 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.10 | 0.30 | - | - | - | - | - | - | - | - | - | - | 0.28 | 43.88 | 16.00 | 11.43 | 1.01 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          30.88
  command p90 ms                          32.00
  frames                                  99.00
  tick ms/frame                            1.27
  frame p90 ms                             1.33
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         24.69
  interval max ms (median burst)          24.69
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            6.40
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                19.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      1.95
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | switch | 30.88 | 32.00 | 99.00 | 1.27 | 1.33 | 3.00 | 5.72 | 5.76 | 24.69 | 24.69 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.03 | - | 6.40 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.10 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 19.00 | 11.43 | 1.95 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          22.71
  command p90 ms                          27.08
  frames                                 144.00
  tick ms/frame                            1.21
  frame p90 ms                             2.89
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         47.64
  interval max ms (median burst)          47.64
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.67
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)               22.60
  session_begin ms (outside frames)        2.40
  session_overlays ms (outside frames)    17.20
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.60
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     5.66
  input->first frame ms                   54.83
  dwm missed vblanks/burst                16.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      5.09
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | minimize | 22.71 | 27.08 | 144.00 | 1.21 | 2.89 | 0.00 | 5.71 | 5.77 | 47.64 | 47.64 | - | - | - | - | - | - | - | - | - | - | - | - | 0.67 | 0.02 | 1.00 | 4.40 | 22.60 | 2.40 | 17.20 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.60 | 0.20 | - | - | - | - | - | - | - | - | - | - | 5.66 | 54.83 | 16.00 | 11.43 | 5.09 |
tiles main cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          11.53
  command p90 ms                          13.33
  frames                                  63.00
  tick ms/frame                            0.67
  frame p90 ms                             1.23
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.73
  interval p99 ms                          7.81
  interval max ms (median burst)           7.81
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.37
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            3.50
  cloak ms (outside frames)                6.30
  session_begin ms (outside frames)        0.90
  session_overlays ms (outside frames)     1.90
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.00
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.22
  input->first frame ms                   12.15
  dwm missed vblanks/burst                12.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      7.31
| main cpu0 bd-config bo-config #2 | move | 11.53 | 13.33 | 63.00 | 0.67 | 1.23 | 0.00 | 5.71 | 5.73 | 7.81 | 7.81 | - | - | - | - | - | - | - | - | - | - | - | - | 0.37 | 0.00 | 1.00 | 3.50 | 6.30 | 0.90 | 1.90 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | 0.22 | 12.15 | 12.00 | 11.44 | 7.31 |
label=main cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=12 wall=12.4s
  command p50 ms                          30.57
  command p90 ms                          33.54
  frames                                  63.00
  tick ms/frame                            2.21
  frame p90 ms                             4.37
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         30.36
  interval max ms (median burst)          30.36
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.99
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)               12.60
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)     5.80
  session_overlays ms/frame                0.03
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.40
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.21
  input->first frame ms                   24.47
  dwm missed vblanks/burst                19.00
  dwm display interval p90 ms              5.76
  dwm gpu busy p90 ms                      1.58
| main cpu0 bd-config bo-config #2 | resize | 30.57 | 33.54 | 63.00 | 2.21 | 4.37 | 1.00 | 5.71 | 5.78 | 30.36 | 30.36 | - | - | - | - | - | - | - | - | - | - | - | - | 0.99 | 0.03 | 2.00 | 5.40 | 12.60 | 2.20 | 5.80 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.40 | - | - | - | - | - | - | - | - | - | - | 0.21 | 24.47 | 19.00 | 5.76 | 1.58 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.57
  command p90 ms                          22.19
  frames                                  97.00
  tick ms/frame                            1.69
  frame p90 ms                             2.43
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         52.65
  interval max ms (median burst)          52.65
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.62
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.60
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                 9.00
  dwm display interval p90 ms              5.74
  dwm gpu busy p90 ms                      3.40
| main cpu0 bd-config bo-config #2 | switch | 20.57 | 22.19 | 97.00 | 1.69 | 2.43 | 1.00 | 5.71 | 5.74 | 52.65 | 52.65 | - | - | - | - | - | - | - | - | - | - | - | - | 0.62 | 0.01 | 3.00 | 5.60 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.10 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 9.00 | 5.74 | 3.40 |
label=main cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.98
  command p90 ms                          19.40
  frames                                 150.00
  tick ms/frame                            1.22
  frame p90 ms                             4.14
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         38.51
  interval max ms (median burst)          38.51
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.60
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            5.00
  cloak ms (outside frames)               15.00
  session_begin ms (outside frames)        2.00
  session_overlays ms (outside frames)     7.20
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.60
  native_frame ms (outside frames)         0.40
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.59
  input->first frame ms                   41.25
  dwm missed vblanks/burst                22.00
  dwm display interval p90 ms             17.13
  dwm gpu busy p90 ms                      7.41
| main cpu0 bd-config bo-config #2 | minimize | 16.98 | 19.40 | 150.00 | 1.22 | 4.14 | 3.00 | 5.72 | 5.76 | 38.51 | 38.51 | - | - | - | - | - | - | - | - | - | - | - | - | 0.60 | 0.02 | 4.00 | 5.00 | 15.00 | 2.00 | 7.20 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.60 | 0.40 | - | - | - | - | - | - | - | - | - | - | 6.59 | 41.25 | 22.00 | 17.13 | 7.41 |
tiles this cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.01
  command p90 ms                          18.26
  frames                                  63.00
  tick ms/frame                            0.39
  frame p90 ms                             0.56
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         10.75
  interval max ms (median burst)          10.75
  peek_frame ms/frame                      0.34
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.61
  pd_alpha ms (worst frame)                0.27
  pd_draw ms (worst frame)                 0.58
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
  command ms (WM side)                    16.80
  peek_new ms (outside frames)             2.00
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.50
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.40
  to_cloak ms (outside frames)             1.80
  peek_cover ms (outside frames)           1.40
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            0.30
  bd_copy ms (outside frames)              0.20
  ovl_restack ms (outside frames)          0.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)        2.60
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.00
  restack ms (outside frames)              0.90
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.35
  input->first frame ms                   17.88
  dwm missed vblanks/burst                12.00
  dwm display interval p90 ms             17.11
  dwm gpu busy p90 ms                      8.81
| this cpu0 bd-config bo-config #2 | move | 17.01 | 18.26 | 63.00 | 0.39 | 0.56 | 0.00 | 5.71 | 5.75 | 10.75 | 10.75 | 0.34 | 0.16 | 0.05 | 0.01 | 0.03 | 0.61 | 0.27 | 0.58 | - | - | - | - | - | - | - | 4.40 | 7.10 | - | - | - | 16.80 | 2.00 | 1.10 | - | - | - | 2.50 | 0.30 | 0.40 | 1.80 | 1.40 | 0.00 | 1.20 | 0.30 | 0.20 | 0.60 | - | 0.20 | 2.60 | 0.30 | 1.00 | 0.90 | 2.00 | 0.00 | - | - | - | - | 3.35 | 17.88 | 12.00 | 17.11 | 8.81 |
label=this cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          45.13
  command p90 ms                          54.72
  frames                                  60.00
  tick ms/frame                            2.38
  frame p90 ms                             3.60
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         51.21
  interval max ms (median burst)          51.21
  peek_frame ms/frame                      1.94
  peek_draw ms/frame                       1.29
  peek_track ms/frame                      0.30
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.14
  peek_draw ms (worst frame)              17.24
  pd_alpha ms (worst frame)                1.14
  pd_draw ms (worst frame)                15.97
  pd_fill ms (worst frame)                12.37
  fill_source ms (worst frame)             2.47
  ring_raise ms (worst frame)              5.40
  restack ms (worst frame)                 3.05
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.80
  cloak ms (outside frames)               16.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    44.80
  peek_new ms (outside frames)             3.40
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.40
  to_draw ms (outside frames)              0.80
  to_alpha ms (outside frames)             2.70
  to_cloak ms (outside frames)             8.40
  peek_cover ms (outside frames)          11.30
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.00
  ring_copy ms (outside frames)            6.70
  bd_copy ms (outside frames)              0.50
  ovl_restack ms (outside frames)          2.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.50
  peek_handover ms (outside frames)       11.30
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.80
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.04
  input->first frame ms                   63.20
  dwm missed vblanks/burst                25.00
  dwm display interval p90 ms             17.16
  dwm gpu busy p90 ms                      1.57
| this cpu0 bd-config bo-config #2 | resize | 45.13 | 54.72 | 60.00 | 2.38 | 3.60 | 3.00 | 5.72 | 5.78 | 51.21 | 51.21 | 1.94 | 1.29 | 0.30 | 0.14 | 0.14 | 17.24 | 1.14 | 15.97 | 12.37 | 2.47 | 5.40 | 3.05 | - | 0.04 | 2.00 | 5.80 | 16.40 | - | - | - | 44.80 | 3.40 | 2.00 | - | - | - | 12.40 | 0.80 | 2.70 | 8.40 | 11.30 | 0.00 | 1.00 | 6.70 | 0.50 | 2.10 | - | 1.50 | 11.30 | 0.10 | 0.80 | - | 6.00 | 0.00 | - | - | - | - | 3.04 | 63.20 | 25.00 | 17.16 | 1.57 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          30.85
  command p90 ms                          31.58
  frames                                  99.00
  tick ms/frame                            1.12
  frame p90 ms                             1.09
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         19.88
  interval max ms (median burst)          20.96
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
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.60
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
  ws_surrogates ms (outside frames)        9.80
  ws_park ms (outside frames)              2.00
  ws_show ms (outside frames)              8.10
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                27.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      5.44
| this cpu0 bd-config bo-config #2 | switch | 30.85 | 31.58 | 99.00 | 1.12 | 1.09 | 3.00 | 5.72 | 5.76 | 19.88 | 20.96 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.70 | 0.00 | - | - | - | 30.60 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | 0.00 | 0.00 | 0.60 | 0.00 | 0.00 | 0.00 | 9.80 | 2.00 | 8.10 | 0.40 | 0.00 | 0.00 | 27.00 | 11.44 | 5.44 |
label=this cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          43.11
  command p90 ms                          51.16
  frames                                  61.00
  tick ms/frame                            2.30
  frame p90 ms                             3.83
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.80
  interval p99 ms                         52.12
  interval max ms (median burst)          52.12
  peek_frame ms/frame                      1.96
  peek_draw ms/frame                       1.19
  peek_track ms/frame                      0.34
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.21
  peek_draw ms (worst frame)              16.18
  pd_alpha ms (worst frame)                0.85
  pd_draw ms (worst frame)                15.36
  pd_fill ms (worst frame)                12.10
  fill_source ms (worst frame)             3.88
  ring_raise ms (worst frame)              4.74
  restack ms (worst frame)                 2.72
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           12.10
  cloak ms (outside frames)               18.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    42.80
  peek_new ms (outside frames)             7.40
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            9.30
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.30
  to_cloak ms (outside frames)             6.60
  peek_cover ms (outside frames)          15.40
  ring_prep ms (outside frames)            2.90
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            7.30
  bd_copy ms (outside frames)              0.90
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.70
  peek_handover ms (outside frames)       14.90
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              2.70
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.14
  input->first frame ms                   60.13
  dwm missed vblanks/burst                19.00
  dwm display interval p90 ms             28.62
  dwm gpu busy p90 ms                      1.81
| this cpu0 bd-config bo-config #2 | minimize | 43.11 | 51.16 | 61.00 | 2.30 | 3.83 | 3.00 | 5.71 | 5.80 | 52.12 | 52.12 | 1.96 | 1.19 | 0.34 | 0.12 | 0.21 | 16.18 | 0.85 | 15.36 | 12.10 | 3.88 | 4.74 | 2.72 | - | 0.02 | 2.00 | 12.10 | 18.80 | - | - | - | 42.80 | 7.40 | 1.80 | - | - | - | 9.30 | 0.90 | 2.30 | 6.60 | 15.40 | 2.90 | 1.20 | 7.30 | 0.90 | 0.10 | - | 1.70 | 14.90 | 0.40 | 2.70 | - | 6.00 | 0.00 | - | - | - | - | 10.14 | 60.13 | 19.00 | 28.62 | 1.81 |
tiles main cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          13.59
  command p90 ms                          13.88
  frames                                  63.00
  tick ms/frame                            0.67
  frame p90 ms                             1.22
  frames >2x budget                        0.00
  interval p50 ms                          5.72
  interval p90 ms                          5.74
  interval p99 ms                          7.69
  interval max ms (median burst)           7.69
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.37
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.90
  cloak ms (outside frames)                8.90
  session_begin ms (outside frames)        0.90
  session_overlays ms (outside frames)     1.90
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.00
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.22
  input->first frame ms                   14.22
  dwm missed vblanks/burst                10.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      7.30
| main cpu0 bd-config bo-config #3 | move | 13.59 | 13.88 | 63.00 | 0.67 | 1.22 | 0.00 | 5.72 | 5.74 | 7.69 | 7.69 | - | - | - | - | - | - | - | - | - | - | - | - | 0.37 | 0.00 | 1.00 | 5.90 | 8.90 | 0.90 | 1.90 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | 0.22 | 14.22 | 10.00 | 11.43 | 7.30 |
label=main cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          31.23
  command p90 ms                          34.69
  frames                                  63.00
  tick ms/frame                            2.26
  frame p90 ms                             4.45
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         31.19
  interval max ms (median burst)          31.19
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.93
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)               18.50
  session_begin ms (outside frames)        2.30
  session_overlays ms (outside frames)     7.40
  session_overlays ms/frame                0.03
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.60
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.22
  input->first frame ms                   35.75
  dwm missed vblanks/burst                 9.00
  dwm display interval p90 ms              5.75
  dwm gpu busy p90 ms                      1.50
| main cpu0 bd-config bo-config #3 | resize | 31.23 | 34.69 | 63.00 | 2.26 | 4.45 | 2.00 | 5.71 | 5.78 | 31.19 | 31.19 | - | - | - | - | - | - | - | - | - | - | - | - | 0.93 | 0.05 | 2.00 | 5.40 | 18.50 | 2.30 | 7.40 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.60 | 0.20 | - | - | - | - | - | - | - | - | - | - | 0.22 | 35.75 | 9.00 | 5.75 | 1.50 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          20.95
  command p90 ms                          23.45
  frames                                  97.00
  tick ms/frame                            1.69
  frame p90 ms                             2.64
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         54.65
  interval max ms (median burst)          54.65
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.64
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.60
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                12.00
  dwm display interval p90 ms              5.76
  dwm gpu busy p90 ms                      4.10
| main cpu0 bd-config bo-config #3 | switch | 20.95 | 23.45 | 97.00 | 1.69 | 2.64 | 1.00 | 5.71 | 5.74 | 54.65 | 54.65 | - | - | - | - | - | - | - | - | - | - | - | - | 0.64 | 0.02 | 3.00 | 5.60 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.10 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 12.00 | 5.76 | 4.10 |
label=main cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          18.39
  command p90 ms                          19.56
  frames                                 150.00
  tick ms/frame                            1.58
  frame p90 ms                             3.96
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         33.49
  interval max ms (median burst)          34.65
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.77
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)               18.80
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)     7.00
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.50
  native_frame ms (outside frames)         0.40
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.57
  input->first frame ms                   43.80
  dwm missed vblanks/burst                23.00
  dwm display interval p90 ms             17.14
  dwm gpu busy p90 ms                      7.39
| main cpu0 bd-config bo-config #3 | minimize | 18.39 | 19.56 | 150.00 | 1.58 | 3.96 | 3.00 | 5.72 | 5.76 | 33.49 | 34.65 | - | - | - | - | - | - | - | - | - | - | - | - | 0.77 | 0.03 | 4.00 | 4.90 | 18.80 | 2.20 | 7.00 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.50 | 0.40 | - | - | - | - | - | - | - | - | - | - | 6.57 | 43.80 | 23.00 | 17.14 | 7.39 |
tiles this cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.33
  command p90 ms                          18.27
  frames                                  63.00
  tick ms/frame                            0.38
  frame p90 ms                             0.56
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         10.51
  interval max ms (median burst)          10.51
  peek_frame ms/frame                      0.37
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.65
  pd_alpha ms (worst frame)                0.26
  pd_draw ms (worst frame)                 0.65
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)                7.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.10
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.50
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.40
  to_cloak ms (outside frames)             1.80
  peek_cover ms (outside frames)           1.40
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.40
  ring_copy ms (outside frames)            0.30
  bd_copy ms (outside frames)              0.10
  ovl_restack ms (outside frames)          0.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
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
  input->anim start ms                     3.42
  input->first frame ms                   17.81
  dwm missed vblanks/burst                13.00
  dwm display interval p90 ms             11.45
  dwm gpu busy p90 ms                      8.51
| this cpu0 bd-config bo-config #3 | move | 17.33 | 18.27 | 63.00 | 0.38 | 0.56 | 0.00 | 5.71 | 5.75 | 10.51 | 10.51 | 0.37 | 0.17 | 0.05 | 0.01 | 0.03 | 0.65 | 0.26 | 0.65 | - | - | - | - | - | - | - | 4.80 | 7.60 | - | - | - | 17.10 | 2.10 | 1.20 | - | - | - | 2.50 | 0.30 | 0.40 | 1.80 | 1.40 | 0.00 | 1.40 | 0.30 | 0.10 | 0.60 | - | 0.20 | 2.60 | 0.30 | 1.00 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.42 | 17.81 | 13.00 | 11.45 | 8.51 |
label=this cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          44.59
  command p90 ms                          54.48
  frames                                  62.00
  tick ms/frame                            2.40
  frame p90 ms                             3.44
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         39.84
  interval max ms (median burst)          39.84
  peek_frame ms/frame                      1.80
  peek_draw ms/frame                       1.21
  peek_track ms/frame                      0.31
  peek_ring ms/frame                       0.17
  peek_backdrop ms/frame                   0.14
  peek_draw ms (worst frame)              13.87
  pd_alpha ms (worst frame)                1.12
  pd_draw ms (worst frame)                12.50
  pd_fill ms (worst frame)                 8.88
  fill_source ms (worst frame)             2.08
  ring_raise ms (worst frame)              5.26
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.90
  cloak ms (outside frames)               16.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    44.30
  peek_new ms (outside frames)             3.30
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.60
  to_draw ms (outside frames)              0.80
  to_alpha ms (outside frames)             2.80
  to_cloak ms (outside frames)             8.50
  peek_cover ms (outside frames)          11.20
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.00
  ring_copy ms (outside frames)            6.80
  bd_copy ms (outside frames)              0.50
  ovl_restack ms (outside frames)          2.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.50
  peek_handover ms (outside frames)       11.40
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.80
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.03
  input->first frame ms                   61.39
  dwm missed vblanks/burst                29.00
  dwm display interval p90 ms             17.17
  dwm gpu busy p90 ms                      1.46
| this cpu0 bd-config bo-config #3 | resize | 44.59 | 54.48 | 62.00 | 2.40 | 3.44 | 3.00 | 5.71 | 5.78 | 39.84 | 39.84 | 1.80 | 1.21 | 0.31 | 0.17 | 0.14 | 13.87 | 1.12 | 12.50 | 8.88 | 2.08 | 5.26 | 0.00 | - | 0.04 | 2.00 | 9.90 | 16.70 | - | - | - | 44.30 | 3.30 | 2.00 | - | - | - | 12.60 | 0.80 | 2.80 | 8.50 | 11.20 | 0.00 | 1.00 | 6.80 | 0.50 | 2.10 | - | 1.50 | 11.40 | 0.10 | 0.80 | - | 6.00 | 0.00 | - | - | - | - | 3.03 | 61.39 | 29.00 | 17.17 | 1.46 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          30.83
  command p90 ms                          32.67
  frames                                  99.00
  tick ms/frame                            1.12
  frame p90 ms                             1.07
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         22.70
  interval max ms (median burst)          22.70
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
  dwm_flush ms (outside frames)            6.30
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.50
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
  ws_park ms (outside frames)              2.00
  ws_show ms (outside frames)              8.70
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                23.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      1.93
| this cpu0 bd-config bo-config #3 | switch | 30.83 | 32.67 | 99.00 | 1.12 | 1.07 | 3.00 | 5.71 | 5.76 | 22.70 | 22.70 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 6.30 | 0.00 | - | - | - | 30.50 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | 0.00 | 0.00 | 0.70 | 0.00 | 0.00 | 0.00 | 9.70 | 2.00 | 8.70 | 0.40 | 0.00 | 0.00 | 23.00 | 11.44 | 1.93 |
label=this cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          45.15
  command p90 ms                          51.94
  frames                                  62.00
  tick ms/frame                            2.26
  frame p90 ms                             3.56
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.82
  interval p99 ms                         48.71
  interval max ms (median burst)          48.71
  peek_frame ms/frame                      1.90
  peek_draw ms/frame                       1.12
  peek_track ms/frame                      0.39
  peek_ring ms/frame                       0.17
  peek_backdrop ms/frame                   0.21
  peek_draw ms (worst frame)              11.51
  pd_alpha ms (worst frame)                0.82
  pd_draw ms (worst frame)                10.69
  pd_fill ms (worst frame)                 7.59
  fill_source ms (worst frame)             1.97
  ring_raise ms (worst frame)              3.36
  restack ms (worst frame)                 2.70
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.00
  cloak ms (outside frames)               18.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    44.80
  peek_new ms (outside frames)             6.80
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           10.20
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.10
  to_cloak ms (outside frames)             7.60
  peek_cover ms (outside frames)          17.50
  ring_prep ms (outside frames)            3.40
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            7.30
  bd_copy ms (outside frames)              0.90
  ovl_restack ms (outside frames)          0.10
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.70
  peek_handover ms (outside frames)       12.20
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              2.30
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.79
  input->first frame ms                   64.19
  dwm missed vblanks/burst                19.00
  dwm display interval p90 ms             28.57
  dwm gpu busy p90 ms                      2.53
| this cpu0 bd-config bo-config #3 | minimize | 45.15 | 51.94 | 62.00 | 2.26 | 3.56 | 3.00 | 5.71 | 5.82 | 48.71 | 48.71 | 1.90 | 1.12 | 0.39 | 0.17 | 0.21 | 11.51 | 0.82 | 10.69 | 7.59 | 1.97 | 3.36 | 2.70 | - | 0.03 | 2.00 | 10.00 | 18.30 | - | - | - | 44.80 | 6.80 | 1.80 | - | - | - | 10.20 | 1.00 | 2.10 | 7.60 | 17.50 | 3.40 | 1.20 | 7.30 | 0.90 | 0.10 | - | 1.70 | 12.20 | 0.30 | 2.30 | - | 6.00 | 0.00 | - | - | - | - | 9.79 | 64.19 | 19.00 | 28.57 | 2.53 |
tiles stand-ins:nopeeks cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          14.51
  command p90 ms                          16.16
  frames                                  62.00
  tick ms/frame                            0.57
  frame p90 ms                             1.13
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                          7.79
  interval max ms (median burst)           7.79
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.34
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)                8.50
  session_begin ms (outside frames)        0.60
  session_overlays ms (outside frames)     3.30
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.00
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.28
  input->first frame ms                   15.04
  dwm missed vblanks/burst                12.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      8.75
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | move | 14.51 | 16.16 | 62.00 | 0.57 | 1.13 | 0.00 | 5.71 | 5.75 | 7.79 | 7.79 | - | - | - | - | - | - | - | - | - | - | - | - | 0.34 | 0.00 | - | 4.50 | 8.50 | 0.60 | 3.30 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | 0.28 | 15.04 | 12.00 | 11.44 | 8.75 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          41.94
  command p90 ms                          44.35
  frames                                  62.00
  tick ms/frame                            1.82
  frame p90 ms                             4.47
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  interval p99 ms                         20.65
  interval max ms (median burst)          20.65
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    1.01
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.90
  cloak ms (outside frames)               20.30
  session_begin ms (outside frames)        1.70
  session_overlays ms (outside frames)    16.20
  session_overlays ms/frame                0.07
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.50
  native_frame ms (outside frames)         0.30
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.27
  input->first frame ms                   44.23
  dwm missed vblanks/burst                14.00
  dwm display interval p90 ms             11.45
  dwm gpu busy p90 ms                      5.13
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | resize | 41.94 | 44.35 | 62.00 | 1.82 | 4.47 | 2.00 | 5.72 | 5.79 | 20.65 | 20.65 | - | - | - | - | - | - | - | - | - | - | - | - | 1.01 | 0.03 | - | 3.90 | 20.30 | 1.70 | 16.20 | 0.07 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.50 | 0.30 | - | - | - | - | - | - | - | - | - | - | 0.27 | 44.23 | 14.00 | 11.45 | 5.13 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          31.55
  command p90 ms                          33.08
  frames                                  99.00
  tick ms/frame                            1.27
  frame p90 ms                             1.15
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         26.49
  interval max ms (median burst)          26.49
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            7.00
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                16.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      1.97
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | switch | 31.55 | 33.08 | 99.00 | 1.27 | 1.15 | 3.00 | 5.71 | 5.75 | 26.49 | 26.49 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.03 | - | 7.00 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.10 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 16.00 | 11.43 | 1.97 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          23.23
  command p90 ms                          26.06
  frames                                 144.00
  tick ms/frame                            1.19
  frame p90 ms                             3.51
  frames >2x budget                        0.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         48.85
  interval max ms (median burst)          48.85
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.66
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)               23.00
  session_begin ms (outside frames)        2.30
  session_overlays ms (outside frames)    17.60
  session_overlays ms/frame                0.03
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.50
  native_frame ms (outside frames)         0.30
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     5.76
  input->first frame ms                   57.08
  dwm missed vblanks/burst                15.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      6.27
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | minimize | 23.23 | 26.06 | 144.00 | 1.19 | 3.51 | 0.00 | 5.72 | 5.76 | 48.85 | 48.85 | - | - | - | - | - | - | - | - | - | - | - | - | 0.66 | 0.02 | 1.00 | 5.40 | 23.00 | 2.30 | 17.60 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.50 | 0.30 | - | - | - | - | - | - | - | - | - | - | 5.76 | 57.08 | 15.00 | 11.44 | 6.27 |
tiles this cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-this-move-mid.png (mid at 63 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-this-move-rest.png
label=this cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=11 wall=12.3s
  command p50 ms                          18.89
  command p90 ms                          20.59
  frames                                  63.00
  tick ms/frame                            0.42
  frame p90 ms                             0.53
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         12.61
  interval max ms (median burst)          12.61
  peek_frame ms/frame                      0.31
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.63
  pd_alpha ms (worst frame)                0.28
  pd_draw ms (worst frame)                 0.50
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)                7.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.40
  peek_new ms (outside frames)             1.10
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.90
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             2.00
  peek_cover ms (outside frames)           1.60
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.50
  ring_copy ms (outside frames)            0.30
  bd_copy ms (outside frames)              0.10
  ovl_restack ms (outside frames)          0.70
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)        3.00
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.20
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.57
  input->first frame ms                   19.33
  dwm missed vblanks/burst                 6.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      8.26
| this cpu8 bd-config bo-config #1 | move | 18.89 | 20.59 | 63.00 | 0.42 | 0.53 | 0.00 | 5.71 | 5.77 | 12.61 | 12.61 | 0.31 | 0.16 | 0.05 | 0.01 | 0.03 | 0.63 | 0.28 | 0.50 | - | - | - | - | - | - | - | 4.90 | 7.30 | - | - | - | 18.40 | 1.10 | 1.20 | - | - | - | 2.90 | 0.30 | 0.50 | 2.00 | 1.60 | 0.00 | 1.50 | 0.30 | 0.10 | 0.70 | - | 0.20 | 3.00 | 0.30 | 1.20 | 0.80 | 2.00 | 0.00 | - | - | - | - | 2.57 | 19.33 | 6.00 | 11.44 | 8.26 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-this-resize-mid.png (mid at 66 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-this-resize-rest.png
label=this cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          57.07
  command p90 ms                          63.97
  frames                                  61.00
  tick ms/frame                            2.74
  frame p90 ms                             4.37
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.81
  interval p99 ms                         38.80
  interval max ms (median burst)          38.80
  peek_frame ms/frame                      2.28
  peek_draw ms/frame                       1.53
  peek_track ms/frame                      0.31
  peek_ring ms/frame                       0.15
  peek_backdrop ms/frame                   0.15
  peek_draw ms (worst frame)              22.15
  pd_alpha ms (worst frame)                1.17
  pd_draw ms (worst frame)                20.96
  pd_fill ms (worst frame)                16.56
  fill_source ms (worst frame)             6.76
  ring_raise ms (worst frame)              5.69
  restack ms (worst frame)                 3.38
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.30
  cloak ms (outside frames)               21.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    56.80
  peek_new ms (outside frames)             4.00
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           14.20
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.10
  to_cloak ms (outside frames)             9.80
  peek_cover ms (outside frames)          12.90
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.10
  ring_copy ms (outside frames)            7.30
  bd_copy ms (outside frames)              0.60
  ovl_restack ms (outside frames)          2.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.30
  peek_handover ms (outside frames)       15.00
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              1.00
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.63
  input->first frame ms                   79.93
  dwm missed vblanks/burst                27.00
  dwm display interval p90 ms             17.16
  dwm gpu busy p90 ms                      1.38
| this cpu8 bd-config bo-config #1 | resize | 57.07 | 63.97 | 61.00 | 2.74 | 4.37 | 3.00 | 5.71 | 5.81 | 38.80 | 38.80 | 2.28 | 1.53 | 0.31 | 0.15 | 0.15 | 22.15 | 1.17 | 20.96 | 16.56 | 6.76 | 5.69 | 3.38 | - | 0.04 | 2.00 | 10.30 | 21.90 | - | - | - | 56.80 | 4.00 | 2.10 | - | - | - | 14.20 | 1.00 | 2.10 | 9.80 | 12.90 | 0.00 | 1.10 | 7.30 | 0.60 | 2.60 | - | 1.30 | 15.00 | 0.20 | 1.00 | - | 6.00 | 0.00 | - | - | - | - | 3.63 | 79.93 | 27.00 | 17.16 | 1.38 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-this-switch-mid.png (mid at 71 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-this-switch-rest.png
label=this cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          35.01
  command p90 ms                          35.51
  frames                                  99.00
  tick ms/frame                            1.25
  frame p90 ms                             1.24
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         22.33
  interval max ms (median burst)          23.26
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            6.20
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    34.80
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)       11.30
  ws_park ms (outside frames)              2.20
  ws_show ms (outside frames)              8.90
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                37.00
  dwm display interval p90 ms             11.47
  dwm gpu busy p90 ms                      6.94
| this cpu8 bd-config bo-config #1 | switch | 35.01 | 35.51 | 99.00 | 1.25 | 1.24 | 3.00 | 5.71 | 5.76 | 22.33 | 23.26 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 6.20 | - | - | - | - | 34.80 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | - | 0.10 | - | 0.00 | 0.70 | - | - | - | 11.30 | 2.20 | 8.90 | 0.40 | - | - | 37.00 | 11.47 | 6.94 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-this-minimize-mid.png (mid at 73 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-this-minimize-rest.png
label=this cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          46.46
  command p90 ms                          57.11
  frames                                  61.00
  tick ms/frame                            2.57
  frame p90 ms                             4.54
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.81
  interval p99 ms                         67.15
  interval max ms (median burst)          67.15
  peek_frame ms/frame                      2.17
  peek_draw ms/frame                       1.36
  peek_track ms/frame                      0.39
  peek_ring ms/frame                       0.15
  peek_backdrop ms/frame                   0.23
  peek_draw ms (worst frame)              17.03
  pd_alpha ms (worst frame)                1.08
  pd_draw ms (worst frame)                16.11
  pd_fill ms (worst frame)                12.21
  fill_source ms (worst frame)             4.75
  ring_raise ms (worst frame)              3.61
  restack ms (worst frame)                 2.70
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.30
  cloak ms (outside frames)               20.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    46.20
  peek_new ms (outside frames)             9.30
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           11.90
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.30
  to_cloak ms (outside frames)             8.40
  peek_cover ms (outside frames)          17.50
  ring_prep ms (outside frames)            4.30
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            7.90
  bd_copy ms (outside frames)              1.00
  ovl_restack ms (outside frames)          0.40
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.80
  peek_handover ms (outside frames)       17.30
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              3.80
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.74
  input->first frame ms                   66.29
  dwm missed vblanks/burst                30.00
  dwm display interval p90 ms             28.61
  dwm gpu busy p90 ms                      2.60
| this cpu8 bd-config bo-config #1 | minimize | 46.46 | 57.11 | 61.00 | 2.57 | 4.54 | 3.00 | 5.71 | 5.81 | 67.15 | 67.15 | 2.17 | 1.36 | 0.39 | 0.15 | 0.23 | 17.03 | 1.08 | 16.11 | 12.21 | 4.75 | 3.61 | 2.70 | - | 0.03 | 2.00 | 11.30 | 20.40 | - | - | - | 46.20 | 9.30 | 1.80 | - | - | - | 11.90 | 1.00 | 2.30 | 8.40 | 17.50 | 4.30 | 1.30 | 7.90 | 1.00 | 0.40 | - | 1.80 | 17.30 | 0.40 | 3.80 | - | 6.00 | 0.00 | - | - | - | - | 8.74 | 66.29 | 30.00 | 28.61 | 2.60 |
tiles stand-ins:nopeeks cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-move-mid.png (mid at 69 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-move-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          15.69
  command p90 ms                          18.53
  frames                                  62.00
  tick ms/frame                            0.62
  frame p90 ms                             1.25
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                          9.00
  interval max ms (median burst)           9.00
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.36
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)                9.50
  session_begin ms (outside frames)        0.70
  session_overlays ms (outside frames)     3.50
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.00
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.31
  input->first frame ms                   16.61
  dwm missed vblanks/burst                12.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      8.70
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | move | 15.69 | 18.53 | 62.00 | 0.62 | 1.25 | 0.00 | 5.71 | 5.76 | 9.00 | 9.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.36 | 0.01 | - | 4.80 | 9.50 | 0.70 | 3.50 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | 0.31 | 16.61 | 12.00 | 11.44 | 8.70 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-resize-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-resize-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          45.72
  command p90 ms                          49.83
  frames                                  62.00
  tick ms/frame                            1.99
  frame p90 ms                             4.68
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          6.07
  interval p99 ms                         20.15
  interval max ms (median burst)          20.15
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    1.15
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.00
  cloak ms (outside frames)               22.50
  session_begin ms (outside frames)        2.00
  session_overlays ms (outside frames)    18.20
  session_overlays ms/frame                0.05
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.10
  native_frame ms (outside frames)         0.30
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.27
  input->first frame ms                   50.90
  dwm missed vblanks/burst                17.00
  dwm display interval p90 ms              5.77
  dwm gpu busy p90 ms                      1.73
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | resize | 45.72 | 49.83 | 62.00 | 1.99 | 4.68 | 2.00 | 5.71 | 6.07 | 20.15 | 20.15 | - | - | - | - | - | - | - | - | - | - | - | - | 1.15 | 0.03 | - | 4.00 | 22.50 | 2.00 | 18.20 | 0.05 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.10 | 0.30 | - | - | - | - | - | - | - | - | - | - | 0.27 | 50.90 | 17.00 | 5.77 | 1.73 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-switch-mid.png (mid at 83 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-switch-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.64
  command p90 ms                          35.39
  frames                                  99.00
  tick ms/frame                            1.40
  frame p90 ms                             1.34
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         26.71
  interval max ms (median burst)          26.71
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            6.50
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.10
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                26.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      1.91
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | switch | 34.64 | 35.39 | 99.00 | 1.40 | 1.34 | 4.00 | 5.71 | 5.76 | 26.71 | 26.71 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | - | 6.50 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.10 | - | - | - | - | - | - | - | - | - | - | - | - | 26.00 | 11.44 | 1.91 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-minimize-mid.png (mid at 64 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-minimize-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          26.73
  command p90 ms                          29.79
  frames                                 143.00
  tick ms/frame                            1.43
  frame p90 ms                             3.59
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         60.92
  interval max ms (median burst)          60.92
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.81
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.20
  cloak ms (outside frames)               30.80
  session_begin ms (outside frames)        5.90
  session_overlays ms (outside frames)    20.00
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.80
  native_frame ms (outside frames)         0.30
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.07
  input->first frame ms                   72.19
  dwm missed vblanks/burst                19.00
  dwm display interval p90 ms             11.46
  dwm gpu busy p90 ms                      5.86
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | minimize | 26.73 | 29.79 | 143.00 | 1.43 | 3.59 | 0.00 | 5.71 | 5.78 | 60.92 | 60.92 | - | - | - | - | - | - | - | - | - | - | - | - | 0.81 | 0.02 | 1.00 | 4.20 | 30.80 | 5.90 | 20.00 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.80 | 0.30 | - | - | - | - | - | - | - | - | - | - | 6.07 | 72.19 | 19.00 | 11.46 | 5.86 |
tiles main cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-main-default-move-mid.png (mid at 74 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-main-default-move-rest.png
label=main cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          13.45
  command p90 ms                          14.08
  frames                                  63.00
  tick ms/frame                            0.68
  frame p90 ms                             1.32
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                          8.80
  interval max ms (median burst)           8.80
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.39
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.00
  cloak ms (outside frames)                8.60
  session_begin ms (outside frames)        1.10
  session_overlays ms (outside frames)     2.00
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.10
  native_frame ms (outside frames)         0.00
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.26
  input->first frame ms                   14.31
  dwm missed vblanks/burst                13.00
  dwm display interval p90 ms             17.14
  dwm gpu busy p90 ms                      7.36
| main cpu8 bd-config bo-config #1 | move | 13.45 | 14.08 | 63.00 | 0.68 | 1.32 | 0.00 | 5.71 | 5.77 | 8.80 | 8.80 | - | - | - | - | - | - | - | - | - | - | - | - | 0.39 | 0.00 | 1.00 | 5.00 | 8.60 | 1.10 | 2.00 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.10 | 0.00 | - | - | - | - | - | - | - | - | - | - | 0.26 | 14.31 | 13.00 | 17.14 | 7.36 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-main-default-resize-mid.png (mid at 66 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-main-default-resize-rest.png
label=main cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=11 wall=12.4s
  command p50 ms                          30.67
  command p90 ms                          36.14
  frames                                  62.00
  tick ms/frame                            2.47
  frame p90 ms                             4.69
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         26.99
  interval max ms (median burst)          26.99
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    1.12
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)               14.70
  session_begin ms (outside frames)        2.40
  session_overlays ms (outside frames)     6.80
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.10
  native_frame ms (outside frames)         0.30
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.24
  input->first frame ms                   27.01
  dwm missed vblanks/burst                11.00
  dwm display interval p90 ms              5.75
  dwm gpu busy p90 ms                      1.98
| main cpu8 bd-config bo-config #1 | resize | 30.67 | 36.14 | 62.00 | 2.47 | 4.69 | 2.00 | 5.72 | 5.78 | 26.99 | 26.99 | - | - | - | - | - | - | - | - | - | - | - | - | 1.12 | 0.04 | 2.00 | 4.40 | 14.70 | 2.40 | 6.80 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.10 | 0.30 | - | - | - | - | - | - | - | - | - | - | 0.24 | 27.01 | 11.00 | 5.75 | 1.98 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-main-default-switch-mid.png (mid at 64 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-main-default-switch-rest.png
label=main cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          21.54
  command p90 ms                          22.56
  frames                                  97.00
  tick ms/frame                            1.90
  frame p90 ms                             2.68
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         62.14
  interval max ms (median burst)          62.14
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.73
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                12.00
  dwm display interval p90 ms             11.41
  dwm gpu busy p90 ms                      3.62
| main cpu8 bd-config bo-config #1 | switch | 21.54 | 22.56 | 97.00 | 1.90 | 2.68 | 1.00 | 5.71 | 5.76 | 62.14 | 62.14 | - | - | - | - | - | - | - | - | - | - | - | - | 0.73 | 0.02 | 3.00 | 5.10 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.20 | - | - | - | - | - | - | - | - | - | - | - | - | 12.00 | 11.41 | 3.62 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-main-default-minimize-mid.png (mid at 61 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-152951\r1-cpu8-bdconfigboconfig-main-default-minimize-rest.png
label=main cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.12
  command p90 ms                          21.57
  frames                                 149.00
  tick ms/frame                            1.80
  frame p90 ms                             5.04
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.81
  interval p99 ms                         44.12
  interval max ms (median burst)          44.12
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.88
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            3.90
  cloak ms (outside frames)               22.60
  session_begin ms (outside frames)        4.50
  session_overlays ms (outside frames)    10.80
  session_overlays ms/frame                0.03
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         1.70
  native_frame ms (outside frames)         0.50
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     7.35
  input->first frame ms                   56.79
  dwm missed vblanks/burst                18.00
  dwm display interval p90 ms             17.12
  dwm gpu busy p90 ms                      7.00
| main cpu8 bd-config bo-config #1 | minimize | 20.12 | 21.57 | 149.00 | 1.80 | 5.04 | 2.00 | 5.71 | 5.81 | 44.12 | 44.12 | - | - | - | - | - | - | - | - | - | - | - | - | 0.88 | 0.03 | 4.00 | 3.90 | 22.60 | 4.50 | 10.80 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 1.70 | 0.50 | - | - | - | - | - | - | - | - | - | - | 7.35 | 56.79 | 18.00 | 17.12 | 7.00 |
tiles stand-ins:nopeeks cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.10
  command p90 ms                          17.67
  frames                                  62.00
  tick ms/frame                            0.63
  frame p90 ms                             1.29
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                          8.97
  interval max ms (median burst)           8.97
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.38
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)                9.50
  session_begin ms (outside frames)        0.70
  session_overlays ms (outside frames)     4.40
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.00
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.30
  input->first frame ms                   17.88
  dwm missed vblanks/burst                11.00
  dwm display interval p90 ms             11.48
  dwm gpu busy p90 ms                      6.88
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | move | 17.10 | 17.67 | 62.00 | 0.63 | 1.29 | 0.00 | 5.71 | 5.76 | 8.97 | 8.97 | - | - | - | - | - | - | - | - | - | - | - | - | 0.38 | 0.01 | - | 4.50 | 9.50 | 0.70 | 4.40 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | 0.30 | 17.88 | 11.00 | 11.48 | 6.88 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          45.49
  command p90 ms                          48.10
  frames                                  62.00
  tick ms/frame                            2.01
  frame p90 ms                             4.42
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          6.47
  interval p99 ms                         22.50
  interval max ms (median burst)          22.50
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    1.16
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.30
  cloak ms (outside frames)               21.70
  session_begin ms (outside frames)        2.00
  session_overlays ms (outside frames)    17.40
  session_overlays ms/frame                0.06
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.10
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.30
  input->first frame ms                   48.84
  dwm missed vblanks/burst                30.00
  dwm display interval p90 ms             11.45
  dwm gpu busy p90 ms                      1.03
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | resize | 45.49 | 48.10 | 62.00 | 2.01 | 4.42 | 2.00 | 5.71 | 6.47 | 22.50 | 22.50 | - | - | - | - | - | - | - | - | - | - | - | - | 1.16 | 0.04 | - | 5.30 | 21.70 | 2.00 | 17.40 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.10 | 0.20 | - | - | - | - | - | - | - | - | - | - | 0.30 | 48.84 | 30.00 | 11.45 | 1.03 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          33.53
  command p90 ms                          35.83
  frames                                  99.00
  tick ms/frame                            1.42
  frame p90 ms                             1.41
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         27.27
  interval max ms (median burst)          28.57
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.60
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                30.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      2.41
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | switch | 33.53 | 35.83 | 99.00 | 1.42 | 1.41 | 3.00 | 5.71 | 5.77 | 27.27 | 28.57 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.04 | - | 5.60 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.20 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 30.00 | 11.44 | 2.41 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          32.60
  command p90 ms                          33.51
  frames                                 143.00
  tick ms/frame                            1.41
  frame p90 ms                             3.07
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         51.39
  interval max ms (median burst)          51.39
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.76
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            6.60
  cloak ms (outside frames)               22.40
  session_begin ms (outside frames)        2.40
  session_overlays ms (outside frames)    20.10
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.90
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.80
  input->first frame ms                   60.13
  dwm missed vblanks/burst                26.00
  dwm display interval p90 ms             17.14
  dwm gpu busy p90 ms                      7.03
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | minimize | 32.60 | 33.51 | 143.00 | 1.41 | 3.07 | 1.00 | 5.71 | 5.78 | 51.39 | 51.39 | - | - | - | - | - | - | - | - | - | - | - | - | 0.76 | 0.02 | 1.00 | 6.60 | 22.40 | 2.40 | 20.10 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.90 | 0.20 | - | - | - | - | - | - | - | - | - | - | 6.80 | 60.13 | 26.00 | 17.14 | 7.03 |
tiles main cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          14.50
  command p90 ms                          15.10
  frames                                  63.00
  tick ms/frame                            0.69
  frame p90 ms                             1.30
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                          8.86
  interval max ms (median burst)           8.86
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.41
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)                9.40
  session_begin ms (outside frames)        1.10
  session_overlays ms (outside frames)     2.00
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.10
  native_frame ms (outside frames)         0.00
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.25
  input->first frame ms                   15.50
  dwm missed vblanks/burst                11.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      7.37
| main cpu8 bd-config bo-config #2 | move | 14.50 | 15.10 | 63.00 | 0.69 | 1.30 | 0.00 | 5.71 | 5.76 | 8.86 | 8.86 | - | - | - | - | - | - | - | - | - | - | - | - | 0.41 | 0.01 | 1.00 | 5.70 | 9.40 | 1.10 | 2.00 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.10 | 0.00 | - | - | - | - | - | - | - | - | - | - | 0.25 | 15.50 | 11.00 | 11.44 | 7.37 |
label=main cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          34.78
  command p90 ms                          36.13
  frames                                  62.00
  tick ms/frame                            2.55
  frame p90 ms                             4.68
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.81
  interval p99 ms                         30.60
  interval max ms (median burst)          30.60
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    1.14
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)               20.70
  session_begin ms (outside frames)        2.50
  session_overlays ms (outside frames)     8.30
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.10
  native_frame ms (outside frames)         0.40
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.26
  input->first frame ms                   37.83
  dwm missed vblanks/burst                15.00
  dwm display interval p90 ms             11.41
  dwm gpu busy p90 ms                      3.44
| main cpu8 bd-config bo-config #2 | resize | 34.78 | 36.13 | 62.00 | 2.55 | 4.68 | 2.00 | 5.72 | 5.81 | 30.60 | 30.60 | - | - | - | - | - | - | - | - | - | - | - | - | 1.14 | 0.05 | 2.00 | 5.10 | 20.70 | 2.50 | 8.30 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.10 | 0.40 | - | - | - | - | - | - | - | - | - | - | 0.26 | 37.83 | 15.00 | 11.41 | 3.44 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          22.49
  command p90 ms                          22.89
  frames                                  96.00
  tick ms/frame                            1.99
  frame p90 ms                             3.05
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         62.08
  interval max ms (median burst)          62.91
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.77
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            6.40
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                 7.00
  dwm display interval p90 ms              5.75
  dwm gpu busy p90 ms                      2.93
| main cpu8 bd-config bo-config #2 | switch | 22.49 | 22.89 | 96.00 | 1.99 | 3.05 | 1.00 | 5.71 | 5.76 | 62.08 | 62.91 | - | - | - | - | - | - | - | - | - | - | - | - | 0.77 | 0.03 | 3.00 | 6.40 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.20 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 7.00 | 5.75 | 2.93 |
label=main cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.33
  command p90 ms                          21.55
  frames                                 146.00
  tick ms/frame                            1.71
  frame p90 ms                             4.73
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.92
  interval p99 ms                         39.23
  interval max ms (median burst)          39.23
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.86
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)               20.40
  session_begin ms (outside frames)        2.50
  session_overlays ms (outside frames)     8.80
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.70
  native_frame ms (outside frames)         0.30
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     7.46
  input->first frame ms                   48.20
  dwm missed vblanks/burst                16.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      7.14
| main cpu8 bd-config bo-config #2 | minimize | 20.33 | 21.55 | 146.00 | 1.71 | 4.73 | 2.00 | 5.71 | 5.92 | 39.23 | 39.23 | - | - | - | - | - | - | - | - | - | - | - | - | 0.86 | 0.03 | 4.00 | 4.40 | 20.40 | 2.50 | 8.80 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.70 | 0.30 | - | - | - | - | - | - | - | - | - | - | 7.46 | 48.20 | 16.00 | 11.44 | 7.14 |
tiles this cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.52
  command p90 ms                          19.90
  frames                                  63.00
  tick ms/frame                            0.42
  frame p90 ms                             0.55
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.27
  interval max ms (median burst)          12.27
  peek_frame ms/frame                      0.35
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.66
  pd_alpha ms (worst frame)                0.30
  pd_draw ms (worst frame)                 0.50
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
  command ms (WM side)                    18.20
  peek_new ms (outside frames)             2.30
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.90
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             2.00
  peek_cover ms (outside frames)           1.30
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.40
  ring_copy ms (outside frames)            0.30
  bd_copy ms (outside frames)              0.10
  ovl_restack ms (outside frames)          0.40
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)        3.00
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
  input->first frame ms                   19.03
  dwm missed vblanks/burst                13.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      8.61
| this cpu8 bd-config bo-config #2 | move | 18.52 | 19.90 | 63.00 | 0.42 | 0.55 | 0.00 | 5.71 | 5.76 | 12.27 | 12.27 | 0.35 | 0.17 | 0.05 | 0.01 | 0.04 | 0.66 | 0.30 | 0.50 | - | - | - | - | - | - | - | 4.70 | 7.70 | - | - | - | 18.20 | 2.30 | 1.20 | - | - | - | 2.90 | 0.30 | 0.50 | 2.00 | 1.30 | 0.00 | 1.40 | 0.30 | 0.10 | 0.40 | - | 0.20 | 3.00 | 0.30 | 1.20 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.76 | 19.03 | 13.00 | 11.44 | 8.61 |
label=this cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          54.96
  command p90 ms                          59.24
  frames                                  61.00
  tick ms/frame                            2.72
  frame p90 ms                             4.24
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         37.55
  interval max ms (median burst)          37.55
  peek_frame ms/frame                      2.19
  peek_draw ms/frame                       1.48
  peek_track ms/frame                      0.35
  peek_ring ms/frame                       0.18
  peek_backdrop ms/frame                   0.16
  peek_draw ms (worst frame)              18.89
  pd_alpha ms (worst frame)                1.19
  pd_draw ms (worst frame)                17.63
  pd_fill ms (worst frame)                13.54
  fill_source ms (worst frame)             2.65
  ring_raise ms (worst frame)              6.05
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.30
  cloak ms (outside frames)               21.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    54.70
  peek_new ms (outside frames)             4.00
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.30
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.30
  to_cloak ms (outside frames)             9.60
  peek_cover ms (outside frames)          12.60
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            7.10
  bd_copy ms (outside frames)              0.50
  ovl_restack ms (outside frames)          2.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.60
  peek_handover ms (outside frames)       13.70
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              0.90
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.68
  input->first frame ms                   74.71
  dwm missed vblanks/burst                32.00
  dwm display interval p90 ms             22.86
  dwm gpu busy p90 ms                      1.88
| this cpu8 bd-config bo-config #2 | resize | 54.96 | 59.24 | 61.00 | 2.72 | 4.24 | 3.00 | 5.72 | 5.78 | 37.55 | 37.55 | 2.19 | 1.48 | 0.35 | 0.18 | 0.16 | 18.89 | 1.19 | 17.63 | 13.54 | 2.65 | 6.05 | 0.00 | - | 0.04 | 2.00 | 10.30 | 21.70 | - | - | - | 54.70 | 4.00 | 2.10 | - | - | - | 13.30 | 1.00 | 2.30 | 9.60 | 12.60 | 0.00 | 1.20 | 7.10 | 0.50 | 2.60 | - | 1.60 | 13.70 | 0.20 | 0.90 | - | 6.00 | 0.00 | - | - | - | - | 3.68 | 74.71 | 32.00 | 22.86 | 1.88 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          33.83
  command p90 ms                          35.12
  frames                                  99.00
  tick ms/frame                            1.28
  frame p90 ms                             1.32
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         25.46
  interval max ms (median burst)          25.46
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
  dwm_flush ms (outside frames)            6.50
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    33.60
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
  ws_surrogates ms (outside frames)       11.70
  ws_park ms (outside frames)              2.30
  ws_show ms (outside frames)              9.20
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                32.00
  dwm display interval p90 ms             11.45
  dwm gpu busy p90 ms                      6.20
| this cpu8 bd-config bo-config #2 | switch | 33.83 | 35.12 | 99.00 | 1.28 | 1.32 | 3.00 | 5.71 | 5.77 | 25.46 | 25.46 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 6.50 | 0.00 | - | - | - | 33.60 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | 0.00 | 0.00 | 0.70 | 0.00 | 0.00 | 0.00 | 11.70 | 2.30 | 9.20 | 0.40 | 0.00 | 0.00 | 32.00 | 11.45 | 6.20 |
label=this cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          52.29
  command p90 ms                          55.76
  frames                                  62.00
  tick ms/frame                            2.49
  frame p90 ms                             4.20
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         52.42
  interval max ms (median burst)          52.42
  peek_frame ms/frame                      2.00
  peek_draw ms/frame                       1.26
  peek_track ms/frame                      0.37
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.22
  peek_draw ms (worst frame)              13.53
  pd_alpha ms (worst frame)                1.06
  pd_draw ms (worst frame)                12.55
  pd_fill ms (worst frame)                 8.92
  fill_source ms (worst frame)             2.70
  ring_raise ms (worst frame)              3.72
  restack ms (worst frame)                 3.24
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           13.30
  cloak ms (outside frames)               20.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    52.00
  peek_new ms (outside frames)             7.90
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           11.80
  to_draw ms (outside frames)              1.10
  to_alpha ms (outside frames)             2.50
  to_cloak ms (outside frames)             8.40
  peek_cover ms (outside frames)          15.60
  ring_prep ms (outside frames)            2.90
  ring_park ms (outside frames)            1.30
  ring_copy ms (outside frames)            7.80
  bd_copy ms (outside frames)              1.00
  ovl_restack ms (outside frames)          0.00
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         2.00
  peek_handover ms (outside frames)       16.40
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              3.80
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.69
  input->first frame ms                   69.41
  dwm missed vblanks/burst                37.00
  dwm display interval p90 ms             28.57
  dwm gpu busy p90 ms                      2.55
| this cpu8 bd-config bo-config #2 | minimize | 52.29 | 55.76 | 62.00 | 2.49 | 4.20 | 3.00 | 5.71 | 5.79 | 52.42 | 52.42 | 2.00 | 1.26 | 0.37 | 0.13 | 0.22 | 13.53 | 1.06 | 12.55 | 8.92 | 2.70 | 3.72 | 3.24 | - | 0.03 | 2.00 | 13.30 | 20.70 | - | - | - | 52.00 | 7.90 | 1.80 | - | - | - | 11.80 | 1.10 | 2.50 | 8.40 | 15.60 | 2.90 | 1.30 | 7.80 | 1.00 | 0.00 | - | 2.00 | 16.40 | 0.30 | 3.80 | - | 6.00 | 0.00 | - | - | - | - | 9.69 | 69.41 | 37.00 | 28.57 | 2.55 |
tiles main cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          13.57
  command p90 ms                          14.56
  frames                                  63.00
  tick ms/frame                            0.69
  frame p90 ms                             1.30
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                          8.98
  interval max ms (median burst)           8.98
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.41
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)                8.60
  session_begin ms (outside frames)        1.10
  session_overlays ms (outside frames)     1.90
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.00
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.24
  input->first frame ms                   14.37
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      7.38
| main cpu8 bd-config bo-config #3 | move | 13.57 | 14.56 | 63.00 | 0.69 | 1.30 | 0.00 | 5.71 | 5.76 | 8.98 | 8.98 | - | - | - | - | - | - | - | - | - | - | - | - | 0.41 | 0.00 | 1.00 | 5.40 | 8.60 | 1.10 | 1.90 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | 0.24 | 14.37 | 5.00 | 11.43 | 7.38 |
label=main cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          35.25
  command p90 ms                          36.09
  frames                                  63.00
  tick ms/frame                            2.46
  frame p90 ms                             4.92
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         29.49
  interval max ms (median burst)          29.49
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    1.12
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.20
  cloak ms (outside frames)               21.10
  session_begin ms (outside frames)        2.70
  session_overlays ms (outside frames)     8.00
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.60
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.26
  input->first frame ms                   39.95
  dwm missed vblanks/burst                11.00
  dwm display interval p90 ms              5.75
  dwm gpu busy p90 ms                      1.13
| main cpu8 bd-config bo-config #3 | resize | 35.25 | 36.09 | 63.00 | 2.46 | 4.92 | 2.00 | 5.71 | 5.77 | 29.49 | 29.49 | - | - | - | - | - | - | - | - | - | - | - | - | 1.12 | 0.04 | 2.00 | 5.20 | 21.10 | 2.70 | 8.00 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.60 | 0.20 | - | - | - | - | - | - | - | - | - | - | 0.26 | 39.95 | 11.00 | 5.75 | 1.13 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          22.63
  command p90 ms                          23.40
  frames                                  97.00
  tick ms/frame                            1.93
  frame p90 ms                             3.06
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         61.18
  interval max ms (median burst)          61.18
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.75
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.90
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)        0.00
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                10.00
  dwm display interval p90 ms             11.38
  dwm gpu busy p90 ms                      3.66
| main cpu8 bd-config bo-config #3 | switch | 22.63 | 23.40 | 97.00 | 1.93 | 3.06 | 1.00 | 5.71 | 5.76 | 61.18 | 61.18 | - | - | - | - | - | - | - | - | - | - | - | - | 0.75 | 0.03 | 3.00 | 5.90 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.20 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 10.00 | 11.38 | 3.66 |
label=main cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          17.74
  command p90 ms                          21.64
  frames                                 150.00
  tick ms/frame                            1.64
  frame p90 ms                             4.74
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         42.66
  interval max ms (median burst)          42.66
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.78
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)               19.20
  session_begin ms (outside frames)        2.50
  session_overlays ms (outside frames)     9.80
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         1.20
  native_frame ms (outside frames)         0.30
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     7.39
  input->first frame ms                   50.48
  dwm missed vblanks/burst                21.00
  dwm display interval p90 ms             17.12
  dwm gpu busy p90 ms                      7.22
| main cpu8 bd-config bo-config #3 | minimize | 17.74 | 21.64 | 150.00 | 1.64 | 4.74 | 3.00 | 5.71 | 5.79 | 42.66 | 42.66 | - | - | - | - | - | - | - | - | - | - | - | - | 0.78 | 0.03 | 4.00 | 4.70 | 19.20 | 2.50 | 9.80 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 1.20 | 0.30 | - | - | - | - | - | - | - | - | - | - | 7.39 | 50.48 | 21.00 | 17.12 | 7.22 |
tiles this cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.27
  command p90 ms                          20.04
  frames                                  63.00
  tick ms/frame                            0.42
  frame p90 ms                             0.55
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.83
  interval max ms (median burst)          12.83
  peek_frame ms/frame                      0.34
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.65
  pd_alpha ms (worst frame)                0.27
  pd_draw ms (worst frame)                 0.50
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)                7.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.00
  peek_new ms (outside frames)             2.40
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)            2.90
  to_draw ms (outside frames)              0.30
  to_alpha ms (outside frames)             0.50
  to_cloak ms (outside frames)             2.10
  peek_cover ms (outside frames)           1.30
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.40
  ring_copy ms (outside frames)            0.30
  bd_copy ms (outside frames)              0.10
  ovl_restack ms (outside frames)          0.40
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)        3.40
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.20
  restack ms (outside frames)              0.80
  peek_ready hits                          2.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.96
  input->first frame ms                   18.84
  dwm missed vblanks/burst                13.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.25
| this cpu8 bd-config bo-config #3 | move | 18.27 | 20.04 | 63.00 | 0.42 | 0.55 | 1.00 | 5.71 | 5.76 | 12.83 | 12.83 | 0.34 | 0.17 | 0.05 | 0.01 | 0.04 | 0.65 | 0.27 | 0.50 | - | - | - | - | - | - | - | 4.50 | 7.20 | - | - | - | 18.00 | 2.40 | 1.20 | - | - | - | 2.90 | 0.30 | 0.50 | 2.10 | 1.30 | 0.00 | 1.40 | 0.30 | 0.10 | 0.40 | - | 0.20 | 3.40 | 0.30 | 1.20 | 0.80 | 2.00 | 0.00 | - | - | - | - | 3.96 | 18.84 | 13.00 | 11.43 | 8.25 |
label=this cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          49.86
  command p90 ms                          59.27
  frames                                  60.00
  tick ms/frame                            2.68
  frame p90 ms                             4.34
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         43.51
  interval max ms (median burst)          43.51
  peek_frame ms/frame                      2.14
  peek_draw ms/frame                       1.48
  peek_track ms/frame                      0.34
  peek_ring ms/frame                       0.16
  peek_backdrop ms/frame                   0.16
  peek_draw ms (worst frame)              19.67
  pd_alpha ms (worst frame)                1.11
  pd_draw ms (worst frame)                18.27
  pd_fill ms (worst frame)                13.67
  fill_source ms (worst frame)             2.87
  ring_raise ms (worst frame)              6.08
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.00
  cloak ms (outside frames)               18.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    49.60
  peek_new ms (outside frames)             4.20
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           13.30
  to_draw ms (outside frames)              1.00
  to_alpha ms (outside frames)             2.60
  to_cloak ms (outside frames)             8.80
  peek_cover ms (outside frames)          12.40
  ring_prep ms (outside frames)            0.00
  ring_park ms (outside frames)            1.20
  ring_copy ms (outside frames)            7.10
  bd_copy ms (outside frames)              0.50
  ovl_restack ms (outside frames)          2.60
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.80
  peek_handover ms (outside frames)       12.60
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.90
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.81
  input->first frame ms                   69.80
  dwm missed vblanks/burst                29.00
  dwm display interval p90 ms             22.86
  dwm gpu busy p90 ms                      1.56
| this cpu8 bd-config bo-config #3 | resize | 49.86 | 59.27 | 60.00 | 2.68 | 4.34 | 3.00 | 5.71 | 5.78 | 43.51 | 43.51 | 2.14 | 1.48 | 0.34 | 0.16 | 0.16 | 19.67 | 1.11 | 18.27 | 13.67 | 2.87 | 6.08 | 0.00 | - | 0.04 | 2.00 | 10.00 | 18.40 | - | - | - | 49.60 | 4.20 | 2.10 | - | - | - | 13.30 | 1.00 | 2.60 | 8.80 | 12.40 | 0.00 | 1.20 | 7.10 | 0.50 | 2.60 | - | 1.80 | 12.60 | 0.10 | 0.90 | - | 6.00 | 0.00 | - | - | - | - | 3.81 | 69.80 | 29.00 | 22.86 | 1.56 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.28
  command p90 ms                          36.57
  frames                                  99.00
  tick ms/frame                            1.29
  frame p90 ms                             1.32
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         23.44
  interval max ms (median burst)          23.44
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
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    34.00
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
  surface ms (outside frames)              0.70
  restack ms (outside frames)              0.00
  peek_ready hits                          0.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)       11.50
  ws_park ms (outside frames)              2.50
  ws_show ms (outside frames)              7.60
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                21.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      1.98
| this cpu8 bd-config bo-config #3 | switch | 34.28 | 36.57 | 99.00 | 1.29 | 1.32 | 3.00 | 5.71 | 5.76 | 23.44 | 23.44 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 4.40 | - | - | - | - | 34.00 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.10 | - | 0.00 | 0.70 | 0.00 | 0.00 | 0.00 | 11.50 | 2.50 | 7.60 | 0.40 | - | - | 21.00 | 11.44 | 1.98 |
label=this cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          48.14
  command p90 ms                          54.81
  frames                                  61.00
  tick ms/frame                            2.42
  frame p90 ms                             4.65
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         53.64
  interval max ms (median burst)          53.64
  peek_frame ms/frame                      2.03
  peek_draw ms/frame                       1.31
  peek_track ms/frame                      0.38
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.22
  peek_draw ms (worst frame)              14.61
  pd_alpha ms (worst frame)                1.20
  pd_draw ms (worst frame)                13.34
  pd_fill ms (worst frame)                 9.20
  fill_source ms (worst frame)             2.87
  ring_raise ms (worst frame)              3.84
  restack ms (worst frame)                 2.76
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.70
  cloak ms (outside frames)               19.10
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    47.80
  peek_new ms (outside frames)             8.70
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  take_over ms (outside frames)           12.30
  to_draw ms (outside frames)              0.90
  to_alpha ms (outside frames)             2.80
  to_cloak ms (outside frames)             8.20
  peek_cover ms (outside frames)          15.90
  ring_prep ms (outside frames)            2.70
  ring_park ms (outside frames)            1.40
  ring_copy ms (outside frames)            7.90
  bd_copy ms (outside frames)              1.00
  ovl_restack ms (outside frames)          0.20
  batch_commit ms (outside frames)            -
  native_frame ms (outside frames)         1.80
  peek_handover ms (outside frames)       16.30
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              3.00
  restack ms (outside frames)                 -
  peek_ready hits                          6.00
  peek_ready misses                        0.00
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.09
  input->first frame ms                   68.67
  dwm missed vblanks/burst                31.00
  dwm display interval p90 ms             34.28
  dwm gpu busy p90 ms                      2.87
| this cpu8 bd-config bo-config #3 | minimize | 48.14 | 54.81 | 61.00 | 2.42 | 4.65 | 3.00 | 5.71 | 5.78 | 53.64 | 53.64 | 2.03 | 1.31 | 0.38 | 0.14 | 0.22 | 14.61 | 1.20 | 13.34 | 9.20 | 2.87 | 3.84 | 2.76 | - | 0.03 | 2.00 | 10.70 | 19.10 | - | - | - | 47.80 | 8.70 | 1.90 | - | - | - | 12.30 | 0.90 | 2.80 | 8.20 | 15.90 | 2.70 | 1.40 | 7.90 | 1.00 | 0.20 | - | 1.80 | 16.30 | 0.40 | 3.00 | - | 6.00 | 0.00 | - | - | - | - | 8.09 | 68.67 | 31.00 | 34.28 | 2.87 |
tiles stand-ins:nopeeks cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          15.66
  command p90 ms                          17.24
  frames                                  62.00
  tick ms/frame                            0.63
  frame p90 ms                             1.25
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                          9.51
  interval max ms (median burst)           9.51
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.37
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.90
  cloak ms (outside frames)                8.80
  session_begin ms (outside frames)        0.70
  session_overlays ms (outside frames)     3.40
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.00
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.32
  input->first frame ms                   16.32
  dwm missed vblanks/burst                10.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.67
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | move | 15.66 | 17.24 | 62.00 | 0.63 | 1.25 | 0.00 | 5.71 | 5.76 | 9.51 | 9.51 | - | - | - | - | - | - | - | - | - | - | - | - | 0.37 | 0.01 | - | 3.90 | 8.80 | 0.70 | 3.40 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | 0.32 | 16.32 | 10.00 | 11.43 | 8.67 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          46.47
  command p90 ms                          50.51
  frames                                  62.00
  tick ms/frame                            2.02
  frame p90 ms                             5.33
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.87
  interval p99 ms                         22.22
  interval max ms (median burst)          22.22
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    1.14
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)               23.10
  session_begin ms (outside frames)        1.90
  session_overlays ms (outside frames)    19.20
  session_overlays ms/frame                0.06
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.20
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.32
  input->first frame ms                   51.86
  dwm missed vblanks/burst                12.00
  dwm display interval p90 ms             11.45
  dwm gpu busy p90 ms                      1.25
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | resize | 46.47 | 50.51 | 62.00 | 2.02 | 5.33 | 2.00 | 5.71 | 5.87 | 22.22 | 22.22 | - | - | - | - | - | - | - | - | - | - | - | - | 1.14 | 0.03 | - | 4.40 | 23.10 | 1.90 | 19.20 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.20 | 0.20 | - | - | - | - | - | - | - | - | - | - | 0.32 | 51.86 | 12.00 | 11.45 | 1.25 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          33.75
  command p90 ms                          36.51
  frames                                  99.00
  tick ms/frame                            1.43
  frame p90 ms                             1.46
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         27.81
  interval max ms (median burst)          28.10
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.00
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.00
  native_frame ms (outside frames)         0.20
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                25.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      2.05
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | switch | 33.75 | 36.51 | 99.00 | 1.43 | 1.46 | 3.00 | 5.71 | 5.77 | 27.81 | 28.10 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.04 | - | 5.70 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.20 | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 25.00 | 11.44 | 2.05 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          26.67
  command p90 ms                          31.09
  frames                                 142.00
  tick ms/frame                            1.42
  frame p90 ms                             3.62
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         51.44
  interval max ms (median burst)          51.44
  peek_frame ms/frame                         -
  peek_draw ms/frame                          -
  peek_track ms/frame                         -
  peek_ring ms/frame                          -
  peek_backdrop ms/frame                      -
  peek_draw ms (worst frame)                  -
  pd_alpha ms (worst frame)                   -
  pd_draw ms (worst frame)                    -
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                    0.74
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.00
  cloak ms (outside frames)               23.50
  session_begin ms (outside frames)        2.90
  session_overlays ms (outside frames)    18.80
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
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
  ring_prep ms (outside frames)               -
  ring_park ms (outside frames)               -
  ring_copy ms (outside frames)               -
  bd_copy ms (outside frames)                 -
  ovl_restack ms (outside frames)             -
  batch_commit ms (outside frames)         0.60
  native_frame ms (outside frames)         0.30
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  peek_ready hits                             -
  peek_ready misses                           -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.46
  input->first frame ms                   59.15
  dwm missed vblanks/burst                18.00
  dwm display interval p90 ms             11.46
  dwm gpu busy p90 ms                      6.74
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | minimize | 26.67 | 31.09 | 142.00 | 1.42 | 3.62 | 2.00 | 5.71 | 5.77 | 51.44 | 51.44 | - | - | - | - | - | - | - | - | - | - | - | - | 0.74 | 0.02 | 1.00 | 5.00 | 23.50 | 2.90 | 18.80 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.60 | 0.30 | - | - | - | - | - | - | - | - | - | - | 6.46 | 59.15 | 18.00 | 11.46 | 6.74 |

## move

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 17.88 | 15.04 | 13.07 |
| cpu0 bd-config bo-config | frame p90 ms | 0.56 | 1.14 | 1.22 |
| cpu0 bd-config bo-config | interval p90 ms | 5.75 | 5.75 | 5.73 |
| cpu0 bd-config bo-config | interval p99 ms | 10.74 | 7.79 | 7.69 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 2.10 | — | — |
| cpu0 bd-config bo-config | take_over ms (outside frames) | 2.50 | — | — |
| cpu0 bd-config bo-config | peek_cover ms (outside frames) | 1.40 | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | 0.60 | 0.90 |
| cpu0 bd-config bo-config | restack ms (outside frames) | 0.80 | — | — |
| cpu0 bd-config bo-config | peek_ready misses | 0.00 | — | — |
| cpu0 bd-config bo-config | dwm gpu busy p90 ms | 8.53 | 8.68 | 7.31 |
| cpu0 bd-config bo-config | dwm missed vblanks/burst | 12.00 | 11.00 | 11.00 |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 19.03 | 16.61 | 14.37 |
| cpu8 bd-config bo-config | frame p90 ms | 0.55 | 1.25 | 1.30 |
| cpu8 bd-config bo-config | interval p90 ms | 5.76 | 5.76 | 5.76 |
| cpu8 bd-config bo-config | interval p99 ms | 12.61 | 9.00 | 8.86 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 2.30 | — | — |
| cpu8 bd-config bo-config | take_over ms (outside frames) | 2.90 | — | — |
| cpu8 bd-config bo-config | peek_cover ms (outside frames) | 1.30 | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | 0.70 | 1.10 |
| cpu8 bd-config bo-config | restack ms (outside frames) | 0.80 | — | — |
| cpu8 bd-config bo-config | peek_ready misses | 0.00 | — | — |
| cpu8 bd-config bo-config | dwm gpu busy p90 ms | 8.26 | 8.67 | 7.37 |
| cpu8 bd-config bo-config | dwm missed vblanks/burst | 13.00 | 11.00 | 11.00 |

### move, cpu0 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 17.33 | 14.51 | 12.30 |
| command p90 ms | 18.27 | 15.37 | 13.42 |
| frames | 63.00 | 62.00 | 63.00 |
| tick ms/frame | 0.39 | 0.57 | 0.67 |
| frame p90 ms | 0.56 | 1.14 | 1.22 |
| frames >2x budget | 0.00 | 0.00 | 0.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.75 | 5.75 | 5.73 |
| interval p99 ms | 10.74 | 7.79 | 7.69 |
| interval max ms (median burst) | 10.74 | 7.79 | 7.69 |
| peek_frame ms/frame | 0.34 | — | — |
| peek_draw ms/frame | 0.16 | — | — |
| peek_track ms/frame | 0.05 | — | — |
| peek_ring ms/frame | 0.01 | — | — |
| peek_backdrop ms/frame | 0.03 | — | — |
| peek_draw ms (worst frame) | 0.61 | — | — |
| pd_alpha ms (worst frame) | 0.26 | — | — |
| pd_draw ms (worst frame) | 0.58 | — | — |
| pd_fill ms (worst frame) | — | — | — |
| fill_source ms (worst frame) | — | — | — |
| ring_raise ms (worst frame) | — | — | — |
| restack ms (worst frame) | — | — | — |
| batch_commit ms/frame | — | 0.34 | 0.37 |
| ovl_region ms/frame | — | 0.00 | 0.00 |
| dwm_flush calls (in frames) | — | — | 1.00 |
| dwm_flush ms (outside frames) | 4.70 | 4.50 | 4.80 |
| cloak ms (outside frames) | 7.60 | 8.50 | 7.70 |
| session_begin ms (outside frames) | — | 0.60 | 0.90 |
| session_overlays ms (outside frames) | — | 3.30 | 1.90 |
| session_overlays ms/frame | — | 0.02 | 0.02 |
| command ms (WM side) | 17.10 | — | — |
| peek_new ms (outside frames) | 2.10 | — | — |
| peek_place ms (outside frames) | 1.10 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 2.50 | — | — |
| to_draw ms (outside frames) | 0.30 | — | — |
| to_alpha ms (outside frames) | 0.40 | — | — |
| to_cloak ms (outside frames) | 1.80 | — | — |
| peek_cover ms (outside frames) | 1.40 | — | — |
| ring_prep ms (outside frames) | 0.00 | — | — |
| ring_park ms (outside frames) | 1.20 | — | — |
| ring_copy ms (outside frames) | 0.30 | — | — |
| bd_copy ms (outside frames) | 0.10 | — | — |
| ovl_restack ms (outside frames) | 0.60 | — | — |
| batch_commit ms (outside frames) | — | 0.00 | 0.00 |
| native_frame ms (outside frames) | 0.20 | 0.00 | 0.00 |
| peek_handover ms (outside frames) | 2.60 | — | — |
| peek_raise ms (outside frames) | 0.30 | — | — |
| surface ms (outside frames) | 1.00 | — | — |
| restack ms (outside frames) | 0.80 | — | — |
| peek_ready hits | 2.00 | — | — |
| peek_ready misses | 0.00 | — | — |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 3.42 | 0.28 | 0.22 |
| input->first frame ms | 17.88 | 15.04 | 13.07 |
| dwm missed vblanks/burst | 12.00 | 11.00 | 11.00 |
| dwm display interval p90 ms | 11.45 | 11.44 | 11.44 |
| dwm gpu busy p90 ms | 8.53 | 8.68 | 7.31 |

### move, cpu8 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 18.52 | 15.69 | 13.57 |
| command p90 ms | 20.04 | 17.67 | 14.56 |
| frames | 63.00 | 62.00 | 63.00 |
| tick ms/frame | 0.42 | 0.63 | 0.69 |
| frame p90 ms | 0.55 | 1.25 | 1.30 |
| frames >2x budget | 0.00 | 0.00 | 0.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.76 | 5.76 | 5.76 |
| interval p99 ms | 12.61 | 9.00 | 8.86 |
| interval max ms (median burst) | 12.61 | 9.00 | 8.86 |
| peek_frame ms/frame | 0.34 | — | — |
| peek_draw ms/frame | 0.17 | — | — |
| peek_track ms/frame | 0.05 | — | — |
| peek_ring ms/frame | 0.01 | — | — |
| peek_backdrop ms/frame | 0.04 | — | — |
| peek_draw ms (worst frame) | 0.65 | — | — |
| pd_alpha ms (worst frame) | 0.28 | — | — |
| pd_draw ms (worst frame) | 0.50 | — | — |
| pd_fill ms (worst frame) | — | — | — |
| fill_source ms (worst frame) | — | — | — |
| ring_raise ms (worst frame) | — | — | — |
| restack ms (worst frame) | — | — | — |
| batch_commit ms/frame | — | 0.37 | 0.41 |
| ovl_region ms/frame | — | 0.01 | 0.00 |
| dwm_flush calls (in frames) | — | — | 1.00 |
| dwm_flush ms (outside frames) | 4.70 | 4.50 | 5.40 |
| cloak ms (outside frames) | 7.30 | 9.50 | 8.60 |
| session_begin ms (outside frames) | — | 0.70 | 1.10 |
| session_overlays ms (outside frames) | — | 3.50 | 2.00 |
| session_overlays ms/frame | — | 0.02 | 0.02 |
| command ms (WM side) | 18.20 | — | — |
| peek_new ms (outside frames) | 2.30 | — | — |
| peek_place ms (outside frames) | 1.20 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 2.90 | — | — |
| to_draw ms (outside frames) | 0.30 | — | — |
| to_alpha ms (outside frames) | 0.50 | — | — |
| to_cloak ms (outside frames) | 2.00 | — | — |
| peek_cover ms (outside frames) | 1.30 | — | — |
| ring_prep ms (outside frames) | 0.00 | — | — |
| ring_park ms (outside frames) | 1.40 | — | — |
| ring_copy ms (outside frames) | 0.30 | — | — |
| bd_copy ms (outside frames) | 0.10 | — | — |
| ovl_restack ms (outside frames) | 0.40 | — | — |
| batch_commit ms (outside frames) | — | 0.00 | 0.10 |
| native_frame ms (outside frames) | 0.20 | 0.00 | 0.00 |
| peek_handover ms (outside frames) | 3.00 | — | — |
| peek_raise ms (outside frames) | 0.30 | — | — |
| surface ms (outside frames) | 1.20 | — | — |
| restack ms (outside frames) | 0.80 | — | — |
| peek_ready hits | 2.00 | — | — |
| peek_ready misses | 0.00 | — | — |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 3.76 | 0.31 | 0.25 |
| input->first frame ms | 19.03 | 16.61 | 14.37 |
| dwm missed vblanks/burst | 13.00 | 11.00 | 11.00 |
| dwm display interval p90 ms | 11.44 | 11.44 | 11.44 |
| dwm gpu busy p90 ms | 8.26 | 8.67 | 7.37 |

## resize

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 63.20 | 44.23 | 35.75 |
| cpu0 bd-config bo-config | frame p90 ms | 3.44 | 4.47 | 4.45 |
| cpu0 bd-config bo-config | interval p90 ms | 5.78 | 5.79 | 5.78 |
| cpu0 bd-config bo-config | interval p99 ms | 39.84 | 20.19 | 30.36 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 3.40 | — | — |
| cpu0 bd-config bo-config | take_over ms (outside frames) | 12.60 | — | — |
| cpu0 bd-config bo-config | peek_cover ms (outside frames) | 11.20 | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | 1.80 | 2.20 |
| cpu0 bd-config bo-config | restack ms (outside frames) | — | — | — |
| cpu0 bd-config bo-config | peek_ready misses | 0.00 | — | — |
| cpu0 bd-config bo-config | dwm gpu busy p90 ms | 1.52 | 2.86 | 1.58 |
| cpu0 bd-config bo-config | dwm missed vblanks/burst | 28.00 | 14.00 | 9.00 |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 74.71 | 50.90 | 37.83 |
| cpu8 bd-config bo-config | frame p90 ms | 4.34 | 4.68 | 4.69 |
| cpu8 bd-config bo-config | interval p90 ms | 5.78 | 6.07 | 5.78 |
| cpu8 bd-config bo-config | interval p99 ms | 38.80 | 22.22 | 29.49 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 4.00 | — | — |
| cpu8 bd-config bo-config | take_over ms (outside frames) | 13.30 | — | — |
| cpu8 bd-config bo-config | peek_cover ms (outside frames) | 12.60 | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | 2.00 | 2.50 |
| cpu8 bd-config bo-config | restack ms (outside frames) | — | — | — |
| cpu8 bd-config bo-config | peek_ready misses | 0.00 | — | — |
| cpu8 bd-config bo-config | dwm gpu busy p90 ms | 1.56 | 1.25 | 1.98 |
| cpu8 bd-config bo-config | dwm missed vblanks/burst | 29.00 | 17.00 | 11.00 |

### resize, cpu0 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 45.13 | 41.94 | 30.57 |
| command p90 ms | 54.72 | 44.35 | 33.54 |
| frames | 62.00 | 62.00 | 63.00 |
| tick ms/frame | 2.38 | 1.81 | 2.21 |
| frame p90 ms | 3.44 | 4.47 | 4.45 |
| frames >2x budget | 3.00 | 2.00 | 1.00 |
| interval p50 ms | 5.71 | 5.72 | 5.71 |
| interval p90 ms | 5.78 | 5.79 | 5.78 |
| interval p99 ms | 39.84 | 20.19 | 30.36 |
| interval max ms (median burst) | 39.84 | 20.19 | 30.36 |
| peek_frame ms/frame | 1.86 | — | — |
| peek_draw ms/frame | 1.21 | — | — |
| peek_track ms/frame | 0.30 | — | — |
| peek_ring ms/frame | 0.14 | — | — |
| peek_backdrop ms/frame | 0.14 | — | — |
| peek_draw ms (worst frame) | 14.94 | — | — |
| pd_alpha ms (worst frame) | 1.12 | — | — |
| pd_draw ms (worst frame) | 13.96 | — | — |
| pd_fill ms (worst frame) | 10.58 | — | — |
| fill_source ms (worst frame) | 2.09 | — | — |
| ring_raise ms (worst frame) | 5.26 | — | — |
| restack ms (worst frame) | 0.00 | — | — |
| batch_commit ms/frame | — | 1.02 | 0.99 |
| ovl_region ms/frame | 0.04 | 0.03 | 0.03 |
| dwm_flush calls (in frames) | 2.00 | — | 2.00 |
| dwm_flush ms (outside frames) | 8.80 | 4.50 | 5.40 |
| cloak ms (outside frames) | 16.70 | 20.30 | 17.70 |
| session_begin ms (outside frames) | — | 1.80 | 2.20 |
| session_overlays ms (outside frames) | — | 16.20 | 7.40 |
| session_overlays ms/frame | — | 0.07 | 0.03 |
| command ms (WM side) | 44.80 | — | — |
| peek_new ms (outside frames) | 3.40 | — | — |
| peek_place ms (outside frames) | 2.00 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 12.60 | — | — |
| to_draw ms (outside frames) | 0.80 | — | — |
| to_alpha ms (outside frames) | 2.70 | — | — |
| to_cloak ms (outside frames) | 8.50 | — | — |
| peek_cover ms (outside frames) | 11.20 | — | — |
| ring_prep ms (outside frames) | 0.00 | — | — |
| ring_park ms (outside frames) | 1.00 | — | — |
| ring_copy ms (outside frames) | 6.70 | — | — |
| bd_copy ms (outside frames) | 0.50 | — | — |
| ovl_restack ms (outside frames) | 2.10 | — | — |
| batch_commit ms (outside frames) | — | 0.20 | 0.60 |
| native_frame ms (outside frames) | 1.50 | 0.30 | 0.20 |
| peek_handover ms (outside frames) | 11.40 | — | — |
| peek_raise ms (outside frames) | 0.10 | — | — |
| surface ms (outside frames) | 0.80 | — | — |
| restack ms (outside frames) | — | — | — |
| peek_ready hits | 6.00 | — | — |
| peek_ready misses | 0.00 | — | — |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 3.04 | 0.27 | 0.22 |
| input->first frame ms | 63.20 | 44.23 | 35.75 |
| dwm missed vblanks/burst | 28.00 | 14.00 | 9.00 |
| dwm display interval p90 ms | 17.16 | 11.43 | 5.76 |
| dwm gpu busy p90 ms | 1.52 | 2.86 | 1.58 |

### resize, cpu8 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 54.96 | 45.72 | 34.78 |
| command p90 ms | 59.27 | 49.83 | 36.13 |
| frames | 61.00 | 62.00 | 62.00 |
| tick ms/frame | 2.72 | 2.01 | 2.47 |
| frame p90 ms | 4.34 | 4.68 | 4.69 |
| frames >2x budget | 3.00 | 2.00 | 2.00 |
| interval p50 ms | 5.71 | 5.71 | 5.72 |
| interval p90 ms | 5.78 | 6.07 | 5.78 |
| interval p99 ms | 38.80 | 22.22 | 29.49 |
| interval max ms (median burst) | 38.80 | 22.22 | 29.49 |
| peek_frame ms/frame | 2.19 | — | — |
| peek_draw ms/frame | 1.48 | — | — |
| peek_track ms/frame | 0.34 | — | — |
| peek_ring ms/frame | 0.16 | — | — |
| peek_backdrop ms/frame | 0.16 | — | — |
| peek_draw ms (worst frame) | 19.67 | — | — |
| pd_alpha ms (worst frame) | 1.17 | — | — |
| pd_draw ms (worst frame) | 18.27 | — | — |
| pd_fill ms (worst frame) | 13.67 | — | — |
| fill_source ms (worst frame) | 2.87 | — | — |
| ring_raise ms (worst frame) | 6.05 | — | — |
| restack ms (worst frame) | 0.00 | — | — |
| batch_commit ms/frame | — | 1.15 | 1.12 |
| ovl_region ms/frame | 0.04 | 0.03 | 0.04 |
| dwm_flush calls (in frames) | 2.00 | — | 2.00 |
| dwm_flush ms (outside frames) | 10.30 | 4.40 | 5.10 |
| cloak ms (outside frames) | 21.70 | 22.50 | 20.70 |
| session_begin ms (outside frames) | — | 2.00 | 2.50 |
| session_overlays ms (outside frames) | — | 18.20 | 8.00 |
| session_overlays ms/frame | — | 0.06 | 0.04 |
| command ms (WM side) | 54.70 | — | — |
| peek_new ms (outside frames) | 4.00 | — | — |
| peek_place ms (outside frames) | 2.10 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 13.30 | — | — |
| to_draw ms (outside frames) | 1.00 | — | — |
| to_alpha ms (outside frames) | 2.30 | — | — |
| to_cloak ms (outside frames) | 9.60 | — | — |
| peek_cover ms (outside frames) | 12.60 | — | — |
| ring_prep ms (outside frames) | 0.00 | — | — |
| ring_park ms (outside frames) | 1.20 | — | — |
| ring_copy ms (outside frames) | 7.10 | — | — |
| bd_copy ms (outside frames) | 0.50 | — | — |
| ovl_restack ms (outside frames) | 2.60 | — | — |
| batch_commit ms (outside frames) | — | 0.10 | 0.10 |
| native_frame ms (outside frames) | 1.60 | 0.20 | 0.30 |
| peek_handover ms (outside frames) | 13.70 | — | — |
| peek_raise ms (outside frames) | 0.20 | — | — |
| surface ms (outside frames) | 0.90 | — | — |
| restack ms (outside frames) | — | — | — |
| peek_ready hits | 6.00 | — | — |
| peek_ready misses | 0.00 | — | — |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 3.68 | 0.30 | 0.26 |
| input->first frame ms | 74.71 | 50.90 | 37.83 |
| dwm missed vblanks/burst | 29.00 | 17.00 | 11.00 |
| dwm display interval p90 ms | 22.86 | 11.45 | 5.75 |
| dwm gpu busy p90 ms | 1.56 | 1.25 | 1.98 |

## switch

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 0.00 | 0.00 | 0.00 |
| cpu0 bd-config bo-config | frame p90 ms | 1.09 | 1.27 | 2.53 |
| cpu0 bd-config bo-config | interval p90 ms | 5.76 | 5.76 | 5.74 |
| cpu0 bd-config bo-config | interval p99 ms | 21.44 | 24.69 | 52.65 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 0.00 | — | — |
| cpu0 bd-config bo-config | take_over ms (outside frames) | 0.00 | — | — |
| cpu0 bd-config bo-config | peek_cover ms (outside frames) | 0.00 | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | — | 0.00 |
| cpu0 bd-config bo-config | restack ms (outside frames) | 0.00 | — | — |
| cpu0 bd-config bo-config | peek_ready misses | 0.00 | — | — |
| cpu0 bd-config bo-config | dwm gpu busy p90 ms | 2.54 | 1.97 | 4.10 |
| cpu0 bd-config bo-config | dwm missed vblanks/burst | 23.00 | 19.00 | 9.00 |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 0.00 | 0.00 | 0.00 |
| cpu8 bd-config bo-config | frame p90 ms | 1.32 | 1.41 | 3.05 |
| cpu8 bd-config bo-config | interval p90 ms | 5.76 | 5.77 | 5.76 |
| cpu8 bd-config bo-config | interval p99 ms | 23.44 | 27.27 | 62.08 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 0.00 | — | — |
| cpu8 bd-config bo-config | take_over ms (outside frames) | 0.00 | — | — |
| cpu8 bd-config bo-config | peek_cover ms (outside frames) | 0.00 | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | — | 0.00 |
| cpu8 bd-config bo-config | restack ms (outside frames) | 0.00 | — | — |
| cpu8 bd-config bo-config | peek_ready misses | 0.00 | — | — |
| cpu8 bd-config bo-config | dwm gpu busy p90 ms | 6.20 | 2.05 | 3.62 |
| cpu8 bd-config bo-config | dwm missed vblanks/burst | 32.00 | 26.00 | 10.00 |

### switch, cpu0 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 30.83 | 30.88 | 20.57 |
| command p90 ms | 31.58 | 32.00 | 22.19 |
| frames | 99.00 | 99.00 | 97.00 |
| tick ms/frame | 1.12 | 1.27 | 1.69 |
| frame p90 ms | 1.09 | 1.27 | 2.53 |
| frames >2x budget | 3.00 | 3.00 | 1.00 |
| interval p50 ms | 5.72 | 5.72 | 5.71 |
| interval p90 ms | 5.76 | 5.76 | 5.74 |
| interval p99 ms | 21.44 | 24.69 | 52.65 |
| interval max ms (median burst) | 21.44 | 24.69 | 52.65 |
| peek_frame ms/frame | 0.00 | — | — |
| peek_draw ms/frame | 0.00 | — | — |
| peek_track ms/frame | 0.00 | — | — |
| peek_ring ms/frame | 0.00 | — | — |
| peek_backdrop ms/frame | 0.00 | — | — |
| peek_draw ms (worst frame) | 0.00 | — | — |
| pd_alpha ms (worst frame) | 0.00 | — | — |
| pd_draw ms (worst frame) | 0.00 | — | — |
| pd_fill ms (worst frame) | 0.00 | — | — |
| fill_source ms (worst frame) | 0.00 | — | — |
| ring_raise ms (worst frame) | 0.00 | — | — |
| restack ms (worst frame) | — | — | — |
| batch_commit ms/frame | — | 0.00 | 0.64 |
| ovl_region ms/frame | 0.03 | 0.03 | 0.01 |
| dwm_flush calls (in frames) | — | — | 3.00 |
| dwm_flush ms (outside frames) | 5.70 | 6.40 | 5.60 |
| cloak ms (outside frames) | 0.00 | — | 0.00 |
| session_begin ms (outside frames) | — | — | 0.00 |
| session_overlays ms (outside frames) | — | 0.00 | 0.00 |
| session_overlays ms/frame | — | 0.00 | 0.00 |
| command ms (WM side) | 30.50 | — | — |
| peek_new ms (outside frames) | 0.00 | — | — |
| peek_place ms (outside frames) | 0.00 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 0.00 | — | — |
| to_draw ms (outside frames) | 0.00 | — | — |
| to_alpha ms (outside frames) | 0.00 | — | — |
| to_cloak ms (outside frames) | 0.00 | — | — |
| peek_cover ms (outside frames) | 0.00 | — | — |
| ring_prep ms (outside frames) | 0.00 | — | — |
| ring_park ms (outside frames) | 0.00 | — | — |
| ring_copy ms (outside frames) | 0.00 | — | — |
| bd_copy ms (outside frames) | 0.00 | — | — |
| ovl_restack ms (outside frames) | 0.00 | — | — |
| batch_commit ms (outside frames) | — | 0.00 | 0.00 |
| native_frame ms (outside frames) | 0.10 | 0.10 | 0.10 |
| peek_handover ms (outside frames) | 0.00 | — | — |
| peek_raise ms (outside frames) | 0.00 | — | — |
| surface ms (outside frames) | 0.60 | — | — |
| restack ms (outside frames) | 0.00 | — | — |
| peek_ready hits | 0.00 | — | — |
| peek_ready misses | 0.00 | — | — |
| ws_surrogates ms (outside frames) | 9.80 | — | — |
| ws_park ms (outside frames) | 2.00 | — | — |
| ws_show ms (outside frames) | 8.10 | — | — |
| ws_start ms (outside frames) | 0.40 | — | — |
| input->anim start ms | 0.00 | 0.00 | 0.00 |
| input->first frame ms | 0.00 | 0.00 | 0.00 |
| dwm missed vblanks/burst | 23.00 | 19.00 | 9.00 |
| dwm display interval p90 ms | 11.44 | 11.43 | 5.76 |
| dwm gpu busy p90 ms | 2.54 | 1.97 | 4.10 |

### switch, cpu8 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 34.28 | 33.75 | 22.49 |
| command p90 ms | 35.51 | 35.83 | 22.89 |
| frames | 99.00 | 99.00 | 97.00 |
| tick ms/frame | 1.28 | 1.42 | 1.93 |
| frame p90 ms | 1.32 | 1.41 | 3.05 |
| frames >2x budget | 3.00 | 3.00 | 1.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.76 | 5.77 | 5.76 |
| interval p99 ms | 23.44 | 27.27 | 62.08 |
| interval max ms (median burst) | 23.44 | 28.10 | 62.14 |
| peek_frame ms/frame | 0.00 | — | — |
| peek_draw ms/frame | 0.00 | — | — |
| peek_track ms/frame | 0.00 | — | — |
| peek_ring ms/frame | 0.00 | — | — |
| peek_backdrop ms/frame | 0.00 | — | — |
| peek_draw ms (worst frame) | 0.00 | — | — |
| pd_alpha ms (worst frame) | 0.00 | — | — |
| pd_draw ms (worst frame) | 0.00 | — | — |
| pd_fill ms (worst frame) | 0.00 | — | — |
| fill_source ms (worst frame) | 0.00 | — | — |
| ring_raise ms (worst frame) | 0.00 | — | — |
| restack ms (worst frame) | — | — | — |
| batch_commit ms/frame | — | 0.00 | 0.75 |
| ovl_region ms/frame | 0.03 | 0.04 | 0.03 |
| dwm_flush calls (in frames) | — | — | 3.00 |
| dwm_flush ms (outside frames) | 6.20 | 5.70 | 5.90 |
| cloak ms (outside frames) | 0.00 | — | 0.00 |
| session_begin ms (outside frames) | — | — | 0.00 |
| session_overlays ms (outside frames) | — | 0.00 | 0.00 |
| session_overlays ms/frame | — | 0.00 | 0.00 |
| command ms (WM side) | 34.00 | — | — |
| peek_new ms (outside frames) | 0.00 | — | — |
| peek_place ms (outside frames) | 0.00 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 0.00 | — | — |
| to_draw ms (outside frames) | 0.00 | — | — |
| to_alpha ms (outside frames) | 0.00 | — | — |
| to_cloak ms (outside frames) | 0.00 | — | — |
| peek_cover ms (outside frames) | 0.00 | — | — |
| ring_prep ms (outside frames) | 0.00 | — | — |
| ring_park ms (outside frames) | 0.00 | — | — |
| ring_copy ms (outside frames) | 0.00 | — | — |
| bd_copy ms (outside frames) | 0.00 | — | — |
| ovl_restack ms (outside frames) | 0.00 | — | — |
| batch_commit ms (outside frames) | — | 0.00 | 0.00 |
| native_frame ms (outside frames) | 0.10 | 0.20 | 0.20 |
| peek_handover ms (outside frames) | 0.00 | — | — |
| peek_raise ms (outside frames) | 0.00 | — | — |
| surface ms (outside frames) | 0.70 | — | — |
| restack ms (outside frames) | 0.00 | — | — |
| peek_ready hits | 0.00 | — | — |
| peek_ready misses | 0.00 | — | — |
| ws_surrogates ms (outside frames) | 11.50 | — | — |
| ws_park ms (outside frames) | 2.30 | — | — |
| ws_show ms (outside frames) | 8.90 | — | — |
| ws_start ms (outside frames) | 0.40 | — | — |
| input->anim start ms | 0.00 | 0.00 | 0.00 |
| input->first frame ms | 0.00 | 0.00 | 0.00 |
| dwm missed vblanks/burst | 32.00 | 26.00 | 10.00 |
| dwm display interval p90 ms | 11.45 | 11.44 | 11.38 |
| dwm gpu busy p90 ms | 6.20 | 2.05 | 3.62 |

## minimize

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 60.13 | 57.08 | 43.80 |
| cpu0 bd-config bo-config | frame p90 ms | 3.81 | 3.24 | 4.14 |
| cpu0 bd-config bo-config | interval p90 ms | 5.80 | 5.76 | 5.76 |
| cpu0 bd-config bo-config | interval p99 ms | 52.12 | 48.85 | 38.51 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 7.40 | — | — |
| cpu0 bd-config bo-config | take_over ms (outside frames) | 10.20 | — | — |
| cpu0 bd-config bo-config | peek_cover ms (outside frames) | 16.20 | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | 2.40 | 2.20 |
| cpu0 bd-config bo-config | restack ms (outside frames) | — | — | — |
| cpu0 bd-config bo-config | peek_ready misses | 0.00 | — | — |
| cpu0 bd-config bo-config | dwm gpu busy p90 ms | 2.53 | 6.27 | 7.39 |
| cpu0 bd-config bo-config | dwm missed vblanks/burst | 19.00 | 16.00 | 22.00 |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 68.67 | 60.13 | 50.48 |
| cpu8 bd-config bo-config | frame p90 ms | 4.54 | 3.59 | 4.74 |
| cpu8 bd-config bo-config | interval p90 ms | 5.79 | 5.78 | 5.81 |
| cpu8 bd-config bo-config | interval p99 ms | 53.64 | 51.44 | 42.66 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 8.70 | — | — |
| cpu8 bd-config bo-config | take_over ms (outside frames) | 11.90 | — | — |
| cpu8 bd-config bo-config | peek_cover ms (outside frames) | 15.90 | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | 2.90 | 2.50 |
| cpu8 bd-config bo-config | restack ms (outside frames) | — | — | — |
| cpu8 bd-config bo-config | peek_ready misses | 0.00 | — | — |
| cpu8 bd-config bo-config | dwm gpu busy p90 ms | 2.60 | 6.74 | 7.14 |
| cpu8 bd-config bo-config | dwm missed vblanks/burst | 31.00 | 19.00 | 18.00 |

### minimize, cpu0 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 43.76 | 23.23 | 18.39 |
| command p90 ms | 51.16 | 26.06 | 19.56 |
| frames | 62.00 | 144.00 | 150.00 |
| tick ms/frame | 2.26 | 1.21 | 1.58 |
| frame p90 ms | 3.81 | 3.24 | 4.14 |
| frames >2x budget | 3.00 | 0.00 | 3.00 |
| interval p50 ms | 5.71 | 5.72 | 5.72 |
| interval p90 ms | 5.80 | 5.76 | 5.76 |
| interval p99 ms | 52.12 | 48.85 | 38.51 |
| interval max ms (median burst) | 52.12 | 48.85 | 38.51 |
| peek_frame ms/frame | 1.90 | — | — |
| peek_draw ms/frame | 1.14 | — | — |
| peek_track ms/frame | 0.34 | — | — |
| peek_ring ms/frame | 0.14 | — | — |
| peek_backdrop ms/frame | 0.21 | — | — |
| peek_draw ms (worst frame) | 11.51 | — | — |
| pd_alpha ms (worst frame) | 0.85 | — | — |
| pd_draw ms (worst frame) | 10.69 | — | — |
| pd_fill ms (worst frame) | 7.59 | — | — |
| fill_source ms (worst frame) | 1.97 | — | — |
| ring_raise ms (worst frame) | 4.62 | — | — |
| restack ms (worst frame) | 2.72 | — | — |
| batch_commit ms/frame | — | 0.67 | 0.76 |
| ovl_region ms/frame | 0.03 | 0.02 | 0.03 |
| dwm_flush calls (in frames) | 2.00 | 1.00 | 4.00 |
| dwm_flush ms (outside frames) | 12.10 | 5.00 | 5.00 |
| cloak ms (outside frames) | 18.80 | 23.00 | 18.80 |
| session_begin ms (outside frames) | — | 2.40 | 2.20 |
| session_overlays ms (outside frames) | — | 17.60 | 7.20 |
| session_overlays ms/frame | — | 0.04 | 0.02 |
| command ms (WM side) | 43.50 | — | — |
| peek_new ms (outside frames) | 7.40 | — | — |
| peek_place ms (outside frames) | 1.80 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 10.20 | — | — |
| to_draw ms (outside frames) | 1.00 | — | — |
| to_alpha ms (outside frames) | 2.30 | — | — |
| to_cloak ms (outside frames) | 7.40 | — | — |
| peek_cover ms (outside frames) | 16.20 | — | — |
| ring_prep ms (outside frames) | 3.30 | — | — |
| ring_park ms (outside frames) | 1.20 | — | — |
| ring_copy ms (outside frames) | 7.30 | — | — |
| bd_copy ms (outside frames) | 0.90 | — | — |
| ovl_restack ms (outside frames) | 0.10 | — | — |
| batch_commit ms (outside frames) | — | 0.60 | 0.50 |
| native_frame ms (outside frames) | 1.70 | 0.30 | 0.40 |
| peek_handover ms (outside frames) | 14.90 | — | — |
| peek_raise ms (outside frames) | 0.30 | — | — |
| surface ms (outside frames) | 2.70 | — | — |
| restack ms (outside frames) | — | — | — |
| peek_ready hits | 6.00 | — | — |
| peek_ready misses | 0.00 | — | — |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 9.79 | 5.67 | 6.57 |
| input->first frame ms | 60.13 | 57.08 | 43.80 |
| dwm missed vblanks/burst | 19.00 | 16.00 | 22.00 |
| dwm display interval p90 ms | 28.57 | 11.44 | 17.13 |
| dwm gpu busy p90 ms | 2.53 | 6.27 | 7.39 |

### minimize, cpu8 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 48.14 | 26.73 | 20.12 |
| command p90 ms | 55.76 | 31.09 | 21.57 |
| frames | 61.00 | 143.00 | 149.00 |
| tick ms/frame | 2.49 | 1.42 | 1.71 |
| frame p90 ms | 4.54 | 3.59 | 4.74 |
| frames >2x budget | 3.00 | 1.00 | 2.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.79 | 5.78 | 5.81 |
| interval p99 ms | 53.64 | 51.44 | 42.66 |
| interval max ms (median burst) | 53.64 | 51.44 | 42.66 |
| peek_frame ms/frame | 2.03 | — | — |
| peek_draw ms/frame | 1.31 | — | — |
| peek_track ms/frame | 0.38 | — | — |
| peek_ring ms/frame | 0.14 | — | — |
| peek_backdrop ms/frame | 0.22 | — | — |
| peek_draw ms (worst frame) | 14.61 | — | — |
| pd_alpha ms (worst frame) | 1.08 | — | — |
| pd_draw ms (worst frame) | 13.34 | — | — |
| pd_fill ms (worst frame) | 9.20 | — | — |
| fill_source ms (worst frame) | 2.87 | — | — |
| ring_raise ms (worst frame) | 3.72 | — | — |
| restack ms (worst frame) | 2.76 | — | — |
| batch_commit ms/frame | — | 0.76 | 0.86 |
| ovl_region ms/frame | 0.03 | 0.02 | 0.03 |
| dwm_flush calls (in frames) | 2.00 | 1.00 | 4.00 |
| dwm_flush ms (outside frames) | 11.30 | 5.00 | 4.40 |
| cloak ms (outside frames) | 20.40 | 23.50 | 20.40 |
| session_begin ms (outside frames) | — | 2.90 | 2.50 |
| session_overlays ms (outside frames) | — | 20.00 | 9.80 |
| session_overlays ms/frame | — | 0.04 | 0.02 |
| command ms (WM side) | 47.80 | — | — |
| peek_new ms (outside frames) | 8.70 | — | — |
| peek_place ms (outside frames) | 1.80 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| take_over ms (outside frames) | 11.90 | — | — |
| to_draw ms (outside frames) | 1.00 | — | — |
| to_alpha ms (outside frames) | 2.50 | — | — |
| to_cloak ms (outside frames) | 8.40 | — | — |
| peek_cover ms (outside frames) | 15.90 | — | — |
| ring_prep ms (outside frames) | 2.90 | — | — |
| ring_park ms (outside frames) | 1.30 | — | — |
| ring_copy ms (outside frames) | 7.90 | — | — |
| bd_copy ms (outside frames) | 1.00 | — | — |
| ovl_restack ms (outside frames) | 0.20 | — | — |
| batch_commit ms (outside frames) | — | 0.80 | 1.20 |
| native_frame ms (outside frames) | 1.80 | 0.30 | 0.30 |
| peek_handover ms (outside frames) | 16.40 | — | — |
| peek_raise ms (outside frames) | 0.40 | — | — |
| surface ms (outside frames) | 3.80 | — | — |
| restack ms (outside frames) | — | — | — |
| peek_ready hits | 6.00 | — | — |
| peek_ready misses | 0.00 | — | — |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 8.74 | 6.46 | 7.39 |
| input->first frame ms | 68.67 | 60.13 | 50.48 |
| dwm missed vblanks/burst | 31.00 | 19.00 | 18.00 |
| dwm display interval p90 ms | 28.61 | 11.46 | 17.12 |
| dwm gpu busy p90 ms | 2.60 | 6.74 | 7.14 |
