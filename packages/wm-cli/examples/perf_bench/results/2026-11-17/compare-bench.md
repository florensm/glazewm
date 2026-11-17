Branch comparison, 2026-11-17T09:59:59: this = 38350415, against origin/tryout/stand-ins (65a60de), origin/main (cfd3cef); builds this, stand-ins:nopeeks (GLAZEWM_ISOLATE=nopeeks), stand-ins, stand-ins:morepeeks (GLAZEWM_ISOLATE=morepeeks), main. Target chrome, 3 round(s) of 10 bursts per scenario, build and setting, IPC load 0, CPU load 0,8 thread(s) of 16, without DWM timing, tile order Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot, launched notepad,explorer,wt,chrome (4 window(s)).
tiles this cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-this-move-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-this-move-rest.png
label=this cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.78
  command p90 ms                          18.00
  frames                                  63.00
  tick ms/frame                            0.39
  frame p90 ms                             0.62
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          9.96
  interval max ms (median burst)           9.96
  peek_frame ms/frame                      0.37
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.66
  pd_alpha ms (worst frame)                0.26
  pd_draw ms (worst frame)                 0.61
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.30
  cloak ms (outside frames)                7.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    16.50
  peek_new ms (outside frames)             2.00
  peek_place ms (outside frames)           0.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.30
  peek_handover ms (outside frames)        2.90
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              0.90
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.86
  input->first frame ms                   17.42
| this cpu0 bd-config bo-config #1 | move | 16.78 | 18.00 | 63.00 | 0.39 | 0.62 | 0.00 | 5.71 | 5.74 | 9.96 | 9.96 | 0.37 | 0.17 | 0.05 | 0.01 | 0.03 | 0.66 | 0.26 | 0.61 | - | - | - | - | - | - | - | 5.30 | 7.80 | - | - | - | 16.50 | 2.00 | 0.90 | - | - | - | 1.30 | 2.90 | 0.20 | 0.90 | - | - | - | - | - | 2.86 | 17.42 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-this-resize-mid.png (mid at 75 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-this-resize-rest.png
label=this cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          47.76
  command p90 ms                          54.66
  frames                                  62.00
  tick ms/frame                            2.16
  frame p90 ms                             3.28
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         32.43
  interval max ms (median burst)          32.43
  peek_frame ms/frame                      1.78
  peek_draw ms/frame                       1.14
  peek_track ms/frame                      0.30
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.13
  peek_draw ms (worst frame)               9.28
  pd_alpha ms (worst frame)                1.01
  pd_draw ms (worst frame)                 8.03
  pd_fill ms (worst frame)                 4.33
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              3.53
  restack ms (worst frame)                 2.71
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.90
  cloak ms (outside frames)               17.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    47.40
  peek_new ms (outside frames)             3.40
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          10.70
  peek_handover ms (outside frames)       13.20
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              1.40
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.34
  input->first frame ms                   57.09
| this cpu0 bd-config bo-config #1 | resize | 47.76 | 54.66 | 62.00 | 2.16 | 3.28 | 2.00 | 5.72 | 5.76 | 32.43 | 32.43 | 1.78 | 1.14 | 0.30 | 0.14 | 0.13 | 9.28 | 1.01 | 8.03 | 4.33 | 0.00 | 3.53 | 2.71 | - | 0.04 | 2.00 | 8.90 | 17.00 | - | - | - | 47.40 | 3.40 | 1.90 | - | - | - | 10.70 | 13.20 | 0.40 | 1.40 | - | - | - | - | - | 3.34 | 57.09 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-this-switch-mid.png (mid at 76 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-this-switch-rest.png
label=this cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          25.66
  command p90 ms                          28.03
  frames                                  98.00
  tick ms/frame                            1.03
  frame p90 ms                             0.96
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.75
  interval p99 ms                         24.46
  interval max ms (median burst)          24.46
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
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.00
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    25.40
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.50
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)       10.00
  ws_park ms (outside frames)              2.10
  ws_show ms (outside frames)              7.40
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| this cpu0 bd-config bo-config #1 | switch | 25.66 | 28.03 | 98.00 | 1.03 | 0.96 | 2.00 | 5.72 | 5.75 | 24.46 | 24.46 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.01 | - | 5.00 | - | - | - | - | 25.40 | - | - | - | - | - | - | - | 0.00 | 0.50 | - | 10.00 | 2.10 | 7.40 | 0.40 | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-this-minimize-mid.png (mid at 70 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-this-minimize-rest.png
label=this cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          49.36
  command p90 ms                          57.16
  frames                                  61.00
  tick ms/frame                            2.41
  frame p90 ms                             3.47
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         60.50
  interval max ms (median burst)          60.50
  peek_frame ms/frame                      1.93
  peek_draw ms/frame                       1.21
  peek_track ms/frame                      0.35
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.19
  peek_draw ms (worst frame)              16.51
  pd_alpha ms (worst frame)                0.86
  pd_draw ms (worst frame)                15.63
  pd_fill ms (worst frame)                12.54
  fill_source ms (worst frame)             3.72
  ring_raise ms (worst frame)              4.56
  restack ms (worst frame)                 2.62
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           12.00
  cloak ms (outside frames)               18.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    49.00
  peek_new ms (outside frames)            12.90
  peek_place ms (outside frames)           1.70
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          22.50
  peek_handover ms (outside frames)       13.60
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              4.60
  restack ms (outside frames)              9.40
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.49
  input->first frame ms                   63.21
| this cpu0 bd-config bo-config #1 | minimize | 49.36 | 57.16 | 61.00 | 2.41 | 3.47 | 3.00 | 5.72 | 5.77 | 60.50 | 60.50 | 1.93 | 1.21 | 0.35 | 0.13 | 0.19 | 16.51 | 0.86 | 15.63 | 12.54 | 3.72 | 4.56 | 2.62 | - | 0.03 | 2.00 | 12.00 | 18.40 | - | - | - | 49.00 | 12.90 | 1.70 | - | - | - | 22.50 | 13.60 | 0.30 | 4.60 | 9.40 | - | - | - | - | 9.49 | 63.21 |
tiles stand-ins:nopeeks cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-move-mid.png (mid at 70 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-move-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          14.71
  command p90 ms                          17.71
  frames                                  62.00
  tick ms/frame                            0.48
  frame p90 ms                             0.94
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          7.50
  interval max ms (median burst)           7.50
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
  batch_commit ms/frame                    0.26
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.50
  cloak ms (outside frames)                7.50
  session_begin ms (outside frames)        1.00
  session_overlays ms (outside frames)     4.50
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.29
  input->first frame ms                   15.33
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | move | 14.71 | 17.71 | 62.00 | 0.48 | 0.94 | 0.00 | 5.71 | 5.74 | 7.50 | 7.50 | - | - | - | - | - | - | - | - | - | - | - | - | 0.26 | 0.00 | - | 3.50 | 7.50 | 1.00 | 4.50 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.29 | 15.33 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-resize-mid.png (mid at 73 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-resize-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          42.24
  command p90 ms                          43.57
  frames                                  62.00
  tick ms/frame                            1.89
  frame p90 ms                             4.00
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         22.57
  interval max ms (median burst)          22.57
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
  batch_commit ms/frame                    0.91
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.00
  cloak ms (outside frames)               21.20
  session_begin ms (outside frames)        2.10
  session_overlays ms (outside frames)    16.20
  session_overlays ms/frame                0.06
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.30
  input->first frame ms                   44.96
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | resize | 42.24 | 43.57 | 62.00 | 1.89 | 4.00 | 2.00 | 5.71 | 5.77 | 22.57 | 22.57 | - | - | - | - | - | - | - | - | - | - | - | - | 0.91 | 0.05 | - | 5.00 | 21.20 | 2.10 | 16.20 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.30 | 44.96 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-switch-mid.png (mid at 71 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-switch-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.72
  command p90 ms                          30.78
  frames                                  99.00
  tick ms/frame                            1.19
  frame p90 ms                             1.19
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         21.67
  interval max ms (median burst)          21.67
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
  ovl_region ms/frame                      0.02
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | switch | 29.72 | 30.78 | 99.00 | 1.19 | 1.19 | 2.00 | 5.71 | 5.76 | 21.67 | 21.67 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.02 | - | 5.60 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-minimize-mid.png (mid at 65 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-minimize-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          21.97
  command p90 ms                          25.10
  frames                                 145.00
  tick ms/frame                            1.07
  frame p90 ms                             2.36
  frames >2x budget                        0.00
  interval p50 ms                          5.72
  interval p90 ms                          5.75
  interval p99 ms                         51.57
  interval max ms (median burst)          51.57
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
  batch_commit ms/frame                    0.57
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            3.80
  cloak ms (outside frames)               25.80
  session_begin ms (outside frames)        5.30
  session_overlays ms (outside frames)    16.30
  session_overlays ms/frame                0.03
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     5.26
  input->first frame ms                   60.94
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | minimize | 21.97 | 25.10 | 145.00 | 1.07 | 2.36 | 0.00 | 5.72 | 5.75 | 51.57 | 51.57 | - | - | - | - | - | - | - | - | - | - | - | - | 0.57 | 0.02 | 1.00 | 3.80 | 25.80 | 5.30 | 16.30 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 5.26 | 60.94 |
tiles stand-ins cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-default-move-mid.png (mid at 64 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-default-move-rest.png
label=stand-ins cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.53
  command p90 ms                          21.21
  frames                                  63.00
  tick ms/frame                            0.88
  frame p90 ms                             0.93
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.75
  interval max ms (median burst)          10.75
  peek_frame ms/frame                      0.39
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
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.00
  cloak ms (outside frames)                6.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu0 bd-config bo-config #1 | move | 19.53 | 21.21 | 63.00 | 0.88 | 0.93 | 0.00 | 5.71 | 5.74 | 10.75 | 10.75 | 0.39 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.00 | 6.90 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-default-resize-mid.png (mid at 73 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-default-resize-rest.png
label=stand-ins cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          64.36
  command p90 ms                          66.32
  frames                                  61.00
  tick ms/frame                            3.89
  frame p90 ms                             3.54
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.83
  interval p99 ms                         75.58
  interval max ms (median burst)          75.58
  peek_frame ms/frame                      2.39
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.30
  cloak ms (outside frames)               17.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu0 bd-config bo-config #1 | resize | 64.36 | 66.32 | 61.00 | 3.89 | 3.54 | 4.00 | 5.72 | 5.83 | 75.58 | 75.58 | 2.39 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 4.30 | 17.40 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-default-switch-mid.png (mid at 73 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-default-switch-rest.png
label=stand-ins cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.00
  command p90 ms                          30.67
  frames                                  99.00
  tick ms/frame                            1.29
  frame p90 ms                             1.23
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         22.00
  interval max ms (median burst)          22.20
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu0 bd-config bo-config #1 | switch | 30.00 | 30.67 | 99.00 | 1.29 | 1.23 | 3.00 | 5.71 | 5.76 | 22.00 | 22.20 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 6.30 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-default-minimize-mid.png (mid at 70 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-default-minimize-rest.png
label=stand-ins cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          57.96
  command p90 ms                          64.05
  frames                                  60.00
  tick ms/frame                            3.16
  frame p90 ms                             4.27
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          7.22
  interval p99 ms                         72.61
  interval max ms (median burst)          72.61
  peek_frame ms/frame                      2.18
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            6.40
  cloak ms (outside frames)               13.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu0 bd-config bo-config #1 | minimize | 57.96 | 64.05 | 60.00 | 3.16 | 4.27 | 4.00 | 5.71 | 7.22 | 72.61 | 72.61 | 2.18 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 6.40 | 13.20 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles stand-ins:morepeeks cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-move-mid.png (mid at 65 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-move-rest.png
label=stand-ins:morepeeks cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.64
  command p90 ms                          20.67
  frames                                  63.00
  tick ms/frame                            0.92
  frame p90 ms                             0.96
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         11.12
  interval max ms (median burst)          11.12
  peek_frame ms/frame                      0.40
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
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)                7.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu0 bd-config bo-config #1 | move | 19.64 | 20.67 | 63.00 | 0.92 | 0.96 | 0.00 | 5.71 | 5.74 | 11.12 | 11.12 | 0.40 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.90 | 7.90 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-resize-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-resize-rest.png
label=stand-ins:morepeeks cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          65.03
  command p90 ms                          73.60
  frames                                  60.00
  tick ms/frame                            3.57
  frame p90 ms                             3.44
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.80
  interval p99 ms                         54.83
  interval max ms (median burst)          54.83
  peek_frame ms/frame                      2.36
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.80
  cloak ms (outside frames)               17.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu0 bd-config bo-config #1 | resize | 65.03 | 73.60 | 60.00 | 3.57 | 3.44 | 4.00 | 5.71 | 5.80 | 54.83 | 54.83 | 2.36 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 11.80 | 17.30 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-switch-mid.png (mid at 66 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-switch-rest.png
label=stand-ins:morepeeks cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.86
  command p90 ms                          30.71
  frames                                  99.00
  tick ms/frame                            1.29
  frame p90 ms                             1.16
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         23.48
  interval max ms (median burst)          23.48
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
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu0 bd-config bo-config #1 | switch | 29.86 | 30.71 | 99.00 | 1.29 | 1.16 | 2.00 | 5.72 | 5.76 | 23.48 | 23.48 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 6.20 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-minimize-mid.png (mid at 71 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-minimize-rest.png
label=stand-ins:morepeeks cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          59.12
  command p90 ms                          63.94
  frames                                  61.00
  tick ms/frame                            3.10
  frame p90 ms                             4.33
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.45
  interval p99 ms                         74.92
  interval max ms (median burst)          74.92
  peek_frame ms/frame                      2.15
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.90
  cloak ms (outside frames)               14.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu0 bd-config bo-config #1 | minimize | 59.12 | 63.94 | 61.00 | 3.10 | 4.33 | 4.00 | 5.72 | 7.45 | 74.92 | 74.92 | 2.15 | - | - | - | - | - | - | - | - | - | - | - | - | 0.02 | 2.00 | 9.90 | 14.40 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles main cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-main-default-move-mid.png (mid at 63 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-main-default-move-rest.png
label=main cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          13.92
  command p90 ms                          15.55
  frames                                  63.00
  tick ms/frame                            0.57
  frame p90 ms                             1.00
  frames >2x budget                        0.00
  interval p50 ms                          5.72
  interval p90 ms                          5.73
  interval p99 ms                          7.14
  interval max ms (median burst)           7.14
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
  batch_commit ms/frame                    0.30
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)                7.90
  session_begin ms (outside frames)        0.90
  session_overlays ms (outside frames)     3.30
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.23
  input->first frame ms                   14.83
| main cpu0 bd-config bo-config #1 | move | 13.92 | 15.55 | 63.00 | 0.57 | 1.00 | 0.00 | 5.72 | 5.73 | 7.14 | 7.14 | - | - | - | - | - | - | - | - | - | - | - | - | 0.30 | 0.00 | 1.00 | 4.70 | 7.90 | 0.90 | 3.30 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.23 | 14.83 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-main-default-resize-mid.png (mid at 65 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-main-default-resize-rest.png
label=main cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          28.39
  command p90 ms                          32.56
  frames                                  63.00
  tick ms/frame                            1.76
  frame p90 ms                             4.08
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         24.85
  interval max ms (median burst)          24.85
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
  batch_commit ms/frame                    0.91
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.60
  cloak ms (outside frames)               17.10
  session_begin ms (outside frames)        2.30
  session_overlays ms (outside frames)     6.70
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.22
  input->first frame ms                   33.45
| main cpu0 bd-config bo-config #1 | resize | 28.39 | 32.56 | 63.00 | 1.76 | 4.08 | 2.00 | 5.71 | 5.77 | 24.85 | 24.85 | - | - | - | - | - | - | - | - | - | - | - | - | 0.91 | 0.04 | 2.00 | 4.60 | 17.10 | 2.30 | 6.70 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.22 | 33.45 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-main-default-switch-mid.png (mid at 69 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-main-default-switch-rest.png
label=main cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.30
  command p90 ms                          21.06
  frames                                  97.00
  tick ms/frame                            1.64
  frame p90 ms                             2.41
  frames >2x budget                        1.00
  interval p50 ms                          5.72
  interval p90 ms                          5.73
  interval p99 ms                         49.46
  interval max ms (median burst)          49.46
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
  batch_commit ms/frame                    0.61
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.80
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| main cpu0 bd-config bo-config #1 | switch | 20.30 | 21.06 | 97.00 | 1.64 | 2.41 | 1.00 | 5.72 | 5.73 | 49.46 | 49.46 | - | - | - | - | - | - | - | - | - | - | - | - | 0.61 | 0.01 | 3.00 | 5.80 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-main-default-minimize-mid.png (mid at 69 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu0-bdconfigboconfig-main-default-minimize-rest.png
label=main cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.88
  command p90 ms                          19.20
  frames                                 150.00
  tick ms/frame                            1.46
  frame p90 ms                             3.83
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         36.88
  interval max ms (median burst)          36.88
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
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)               21.10
  session_begin ms (outside frames)        4.00
  session_overlays ms (outside frames)     6.90
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.13
  input->first frame ms                   53.99
| main cpu0 bd-config bo-config #1 | minimize | 16.88 | 19.20 | 150.00 | 1.46 | 3.83 | 3.00 | 5.71 | 5.74 | 36.88 | 36.88 | - | - | - | - | - | - | - | - | - | - | - | - | 0.75 | 0.03 | 4.00 | 4.70 | 21.10 | 4.00 | 6.90 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.13 | 53.99 |
tiles stand-ins:nopeeks cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.08
  command p90 ms                          17.63
  frames                                  63.00
  tick ms/frame                            0.47
  frame p90 ms                             0.93
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          7.40
  interval max ms (median burst)           7.40
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
  batch_commit ms/frame                    0.27
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.20
  cloak ms (outside frames)                8.30
  session_begin ms (outside frames)        0.90
  session_overlays ms (outside frames)     4.40
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.29
  input->first frame ms                   16.48
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | move | 16.08 | 17.63 | 63.00 | 0.47 | 0.93 | 0.00 | 5.71 | 5.74 | 7.40 | 7.40 | - | - | - | - | - | - | - | - | - | - | - | - | 0.27 | 0.00 | - | 4.20 | 8.30 | 0.90 | 4.40 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.29 | 16.48 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          40.18
  command p90 ms                          41.09
  frames                                  62.00
  tick ms/frame                            1.86
  frame p90 ms                             4.19
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         26.93
  interval max ms (median burst)          26.93
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
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.80
  cloak ms (outside frames)               20.10
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)    15.10
  session_overlays ms/frame                0.07
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.28
  input->first frame ms                   43.84
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | resize | 40.18 | 41.09 | 62.00 | 1.86 | 4.19 | 1.00 | 5.71 | 5.78 | 26.93 | 26.93 | - | - | - | - | - | - | - | - | - | - | - | - | 0.93 | 0.04 | - | 3.80 | 20.10 | 2.20 | 15.10 | 0.07 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.28 | 43.84 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.72
  command p90 ms                          31.44
  frames                                  99.00
  tick ms/frame                            1.21
  frame p90 ms                             1.23
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         21.89
  interval max ms (median burst)          21.89
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | switch | 29.72 | 31.44 | 99.00 | 1.21 | 1.23 | 3.00 | 5.71 | 5.76 | 21.89 | 21.89 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.03 | - | 6.30 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          23.40
  command p90 ms                          23.87
  frames                                 144.00
  tick ms/frame                            1.11
  frame p90 ms                             2.93
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         45.72
  interval max ms (median burst)          45.72
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
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            3.80
  cloak ms (outside frames)               22.10
  session_begin ms (outside frames)        2.10
  session_overlays ms (outside frames)    15.60
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     5.39
  input->first frame ms                   52.64
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | minimize | 23.40 | 23.87 | 144.00 | 1.11 | 2.93 | 0.00 | 5.71 | 5.76 | 45.72 | 45.72 | - | - | - | - | - | - | - | - | - | - | - | - | 0.60 | 0.02 | 1.00 | 3.80 | 22.10 | 2.10 | 15.60 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 5.39 | 52.64 |
tiles stand-ins cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.25
  command p90 ms                          20.88
  frames                                  63.00
  tick ms/frame                            0.90
  frame p90 ms                             0.92
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         10.84
  interval max ms (median burst)          10.84
  peek_frame ms/frame                      0.39
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
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.10
  cloak ms (outside frames)                6.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu0 bd-config bo-config #2 | move | 18.25 | 20.88 | 63.00 | 0.90 | 0.92 | 0.00 | 5.71 | 5.75 | 10.84 | 10.84 | 0.39 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.10 | 6.90 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          62.63
  command p90 ms                          74.46
  frames                                  61.00
  tick ms/frame                            3.74
  frame p90 ms                             3.74
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.92
  interval p99 ms                         73.11
  interval max ms (median burst)          73.11
  peek_frame ms/frame                      2.34
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.70
  cloak ms (outside frames)               18.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu0 bd-config bo-config #2 | resize | 62.63 | 74.46 | 61.00 | 3.74 | 3.74 | 4.00 | 5.72 | 5.92 | 73.11 | 73.11 | 2.34 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 10.70 | 18.90 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.27
  command p90 ms                          31.84
  frames                                  99.00
  tick ms/frame                            1.30
  frame p90 ms                             1.21
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         23.57
  interval max ms (median burst)          23.57
  peek_frame ms/frame                      0.00
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            6.60
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu0 bd-config bo-config #2 | switch | 30.27 | 31.84 | 99.00 | 1.30 | 1.21 | 2.00 | 5.71 | 5.77 | 23.57 | 23.57 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.02 | - | 6.60 | 0.00 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          59.81
  command p90 ms                          62.30
  frames                                  61.00
  tick ms/frame                            3.05
  frame p90 ms                             3.67
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.60
  interval p99 ms                         72.42
  interval max ms (median burst)          72.42
  peek_frame ms/frame                      2.10
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            7.30
  cloak ms (outside frames)               13.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu0 bd-config bo-config #2 | minimize | 59.81 | 62.30 | 61.00 | 3.05 | 3.67 | 4.00 | 5.72 | 6.60 | 72.42 | 72.42 | 2.10 | - | - | - | - | - | - | - | - | - | - | - | - | 0.02 | 2.00 | 7.30 | 13.80 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles stand-ins:morepeeks cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:morepeeks cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.97
  command p90 ms                          21.76
  frames                                  63.00
  tick ms/frame                            0.87
  frame p90 ms                             0.85
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         11.05
  interval max ms (median burst)          11.05
  peek_frame ms/frame                      0.39
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
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)                7.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu0 bd-config bo-config #2 | move | 19.97 | 21.76 | 63.00 | 0.87 | 0.85 | 0.00 | 5.71 | 5.75 | 11.05 | 11.05 | 0.39 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.40 | 7.80 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          65.06
  command p90 ms                          67.03
  frames                                  61.00
  tick ms/frame                            3.90
  frame p90 ms                             3.30
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.81
  interval p99 ms                         67.67
  interval max ms (median burst)          67.67
  peek_frame ms/frame                      2.27
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)               18.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu0 bd-config bo-config #2 | resize | 65.06 | 67.03 | 61.00 | 3.90 | 3.30 | 4.00 | 5.71 | 5.81 | 67.67 | 67.67 | 2.27 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 5.10 | 18.00 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:morepeeks cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.61
  command p90 ms                          31.61
  frames                                 100.00
  tick ms/frame                            1.32
  frame p90 ms                             1.21
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         24.01
  interval max ms (median burst)          24.01
  peek_frame ms/frame                      0.00
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
  dwm_flush ms (outside frames)            6.60
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu0 bd-config bo-config #2 | switch | 30.61 | 31.61 | 100.00 | 1.32 | 1.21 | 3.00 | 5.72 | 5.76 | 24.01 | 24.01 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 6.60 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          58.53
  command p90 ms                          66.89
  frames                                  60.00
  tick ms/frame                            3.14
  frame p90 ms                             3.67
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.12
  interval p99 ms                         73.43
  interval max ms (median burst)          73.43
  peek_frame ms/frame                      2.12
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.70
  cloak ms (outside frames)               17.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu0 bd-config bo-config #2 | minimize | 58.53 | 66.89 | 60.00 | 3.14 | 3.67 | 4.00 | 5.72 | 7.12 | 73.43 | 73.43 | 2.12 | - | - | - | - | - | - | - | - | - | - | - | - | 0.02 | 2.00 | 11.70 | 17.00 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles main cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          14.99
  command p90 ms                          16.23
  frames                                  63.00
  tick ms/frame                            0.57
  frame p90 ms                             0.99
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          7.05
  interval max ms (median burst)           7.05
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
  batch_commit ms/frame                    0.30
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)                9.10
  session_begin ms (outside frames)        0.90
  session_overlays ms (outside frames)     3.00
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.27
  input->first frame ms                   16.08
| main cpu0 bd-config bo-config #2 | move | 14.99 | 16.23 | 63.00 | 0.57 | 0.99 | 0.00 | 5.71 | 5.74 | 7.05 | 7.05 | - | - | - | - | - | - | - | - | - | - | - | - | 0.30 | 0.00 | 1.00 | 5.70 | 9.10 | 0.90 | 3.00 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.27 | 16.08 |
label=main cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.71
  command p90 ms                          32.18
  frames                                  63.00
  tick ms/frame                            1.84
  frame p90 ms                             3.98
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         23.00
  interval max ms (median burst)          23.00
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
  batch_commit ms/frame                    0.89
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.50
  cloak ms (outside frames)               18.90
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)     6.80
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.22
  input->first frame ms                   35.90
| main cpu0 bd-config bo-config #2 | resize | 30.71 | 32.18 | 63.00 | 1.84 | 3.98 | 2.00 | 5.72 | 5.77 | 23.00 | 23.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.89 | 0.05 | 2.00 | 5.50 | 18.90 | 2.20 | 6.80 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.22 | 35.90 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.41
  command p90 ms                          20.99
  frames                                  97.00
  tick ms/frame                            1.65
  frame p90 ms                             2.34
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.73
  interval p99 ms                         50.78
  interval max ms (median burst)          51.25
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
  batch_commit ms/frame                    0.61
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            6.30
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main cpu0 bd-config bo-config #2 | switch | 20.41 | 20.99 | 97.00 | 1.65 | 2.34 | 1.00 | 5.71 | 5.73 | 50.78 | 51.25 | - | - | - | - | - | - | - | - | - | - | - | - | 0.61 | 0.01 | 3.00 | 6.30 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=main cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.27
  command p90 ms                          18.07
  frames                                 149.00
  tick ms/frame                            1.32
  frame p90 ms                             3.22
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.74
  interval p99 ms                         34.50
  interval max ms (median burst)          34.50
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
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)               17.20
  session_begin ms (outside frames)        2.30
  session_overlays ms (outside frames)     6.50
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.31
  input->first frame ms                   42.67
| main cpu0 bd-config bo-config #2 | minimize | 16.27 | 18.07 | 149.00 | 1.32 | 3.22 | 2.00 | 5.72 | 5.74 | 34.50 | 34.50 | - | - | - | - | - | - | - | - | - | - | - | - | 0.64 | 0.03 | 4.00 | 4.50 | 17.20 | 2.30 | 6.50 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.31 | 42.67 |
tiles this cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          15.76
  command p90 ms                          17.07
  frames                                  63.00
  tick ms/frame                            0.38
  frame p90 ms                             0.57
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.03
  interval max ms (median burst)          10.03
  peek_frame ms/frame                      0.35
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.81
  pd_alpha ms (worst frame)                0.44
  pd_draw ms (worst frame)                 0.50
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)                7.50
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    15.40
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           0.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.10
  peek_handover ms (outside frames)        2.60
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.00
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.89
  input->first frame ms                   16.88
| this cpu0 bd-config bo-config #2 | move | 15.76 | 17.07 | 63.00 | 0.38 | 0.57 | 0.00 | 5.71 | 5.74 | 10.03 | 10.03 | 0.35 | 0.17 | 0.05 | 0.01 | 0.03 | 0.81 | 0.44 | 0.50 | - | - | - | - | - | - | - | 4.80 | 7.50 | - | - | - | 15.40 | 2.10 | 0.90 | - | - | - | 1.10 | 2.60 | 0.30 | 1.00 | - | - | - | - | - | 2.89 | 16.88 |
label=this cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          48.06
  command p90 ms                          54.06
  frames                                  61.00
  tick ms/frame                            2.36
  frame p90 ms                             3.88
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.82
  interval p99 ms                         43.71
  interval max ms (median burst)          43.71
  peek_frame ms/frame                      2.00
  peek_draw ms/frame                       1.34
  peek_track ms/frame                      0.28
  peek_ring ms/frame                       0.15
  peek_backdrop ms/frame                   0.12
  peek_draw ms (worst frame)              20.30
  pd_alpha ms (worst frame)                1.02
  pd_draw ms (worst frame)                18.99
  pd_fill ms (worst frame)                15.39
  fill_source ms (worst frame)             4.56
  ring_raise ms (worst frame)              5.61
  restack ms (worst frame)                 2.84
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.10
  cloak ms (outside frames)               15.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    47.70
  peek_new ms (outside frames)             3.50
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          11.00
  peek_handover ms (outside frames)       12.00
  peek_raise ms (outside frames)           0.70
  surface ms (outside frames)              2.00
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.32
  input->first frame ms                   68.50
| this cpu0 bd-config bo-config #2 | resize | 48.06 | 54.06 | 61.00 | 2.36 | 3.88 | 3.00 | 5.72 | 5.82 | 43.71 | 43.71 | 2.00 | 1.34 | 0.28 | 0.15 | 0.12 | 20.30 | 1.02 | 18.99 | 15.39 | 4.56 | 5.61 | 2.84 | - | 0.04 | 2.00 | 9.10 | 15.80 | - | - | - | 47.70 | 3.50 | 1.90 | - | - | - | 11.00 | 12.00 | 0.70 | 2.00 | - | - | - | - | - | 3.32 | 68.50 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          25.57
  command p90 ms                          31.90
  frames                                  98.00
  tick ms/frame                            1.06
  frame p90 ms                             1.11
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         19.26
  interval max ms (median burst)          19.26
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.30
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    25.20
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.50
  restack ms (outside frames)              0.00
  ws_surrogates ms (outside frames)       10.50
  ws_park ms (outside frames)              1.90
  ws_show ms (outside frames)              7.70
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| this cpu0 bd-config bo-config #2 | switch | 25.57 | 31.90 | 98.00 | 1.06 | 1.11 | 2.00 | 5.72 | 5.76 | 19.26 | 19.26 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.02 | - | 5.30 | - | - | - | - | 25.20 | - | - | - | - | - | - | - | 0.00 | 0.50 | 0.00 | 10.50 | 1.90 | 7.70 | 0.40 | - | - |
label=this cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          53.31
  command p90 ms                          56.02
  frames                                  62.00
  tick ms/frame                            2.33
  frame p90 ms                             3.72
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         47.94
  interval max ms (median burst)          47.94
  peek_frame ms/frame                      1.81
  peek_draw ms/frame                       1.19
  peek_track ms/frame                      0.35
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.19
  peek_draw ms (worst frame)              14.49
  pd_alpha ms (worst frame)                0.82
  pd_draw ms (worst frame)                13.71
  pd_fill ms (worst frame)                10.57
  fill_source ms (worst frame)             2.69
  ring_raise ms (worst frame)              5.79
  restack ms (worst frame)                 2.62
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.40
  cloak ms (outside frames)               15.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    53.00
  peek_new ms (outside frames)             7.50
  peek_place ms (outside frames)           1.70
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          22.40
  peek_handover ms (outside frames)       13.80
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              2.40
  restack ms (outside frames)              8.30
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.68
  input->first frame ms                   69.48
| this cpu0 bd-config bo-config #2 | minimize | 53.31 | 56.02 | 62.00 | 2.33 | 3.72 | 3.00 | 5.71 | 5.77 | 47.94 | 47.94 | 1.81 | 1.19 | 0.35 | 0.13 | 0.19 | 14.49 | 0.82 | 13.71 | 10.57 | 2.69 | 5.79 | 2.62 | - | 0.02 | 2.00 | 10.40 | 15.80 | - | - | - | 53.00 | 7.50 | 1.70 | - | - | - | 22.40 | 13.80 | 0.20 | 2.40 | 8.30 | - | - | - | - | 9.68 | 69.48 |
tiles stand-ins cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.50
  command p90 ms                          20.83
  frames                                  63.00
  tick ms/frame                            0.87
  frame p90 ms                             0.96
  frames >2x budget                        0.00
  interval p50 ms                          5.72
  interval p90 ms                          5.74
  interval p99 ms                         10.74
  interval max ms (median burst)          10.74
  peek_frame ms/frame                      0.39
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
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.00
  cloak ms (outside frames)                6.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu0 bd-config bo-config #3 | move | 19.50 | 20.83 | 63.00 | 0.87 | 0.96 | 0.00 | 5.72 | 5.74 | 10.74 | 10.74 | 0.39 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.00 | 6.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          61.21
  command p90 ms                          68.26
  frames                                  60.00
  tick ms/frame                            3.82
  frame p90 ms                             3.41
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          5.85
  interval p99 ms                         51.18
  interval max ms (median burst)          51.18
  peek_frame ms/frame                      2.24
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.90
  cloak ms (outside frames)               17.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu0 bd-config bo-config #3 | resize | 61.21 | 68.26 | 60.00 | 3.82 | 3.41 | 5.00 | 5.71 | 5.85 | 51.18 | 51.18 | 2.24 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 8.90 | 17.00 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.79
  command p90 ms                          31.26
  frames                                  99.00
  tick ms/frame                            1.32
  frame p90 ms                             1.30
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         24.58
  interval max ms (median burst)          24.58
  peek_frame ms/frame                      0.00
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
  dwm_flush ms (outside frames)            7.10
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu0 bd-config bo-config #3 | switch | 30.79 | 31.26 | 99.00 | 1.32 | 1.30 | 3.00 | 5.72 | 5.76 | 24.58 | 24.58 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 7.10 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          57.58
  command p90 ms                          66.70
  frames                                  60.00
  tick ms/frame                            3.14
  frame p90 ms                             3.81
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          7.94
  interval p99 ms                         71.76
  interval max ms (median burst)          71.76
  peek_frame ms/frame                      2.22
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.50
  cloak ms (outside frames)               15.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu0 bd-config bo-config #3 | minimize | 57.58 | 66.70 | 60.00 | 3.14 | 3.81 | 3.00 | 5.72 | 7.94 | 71.76 | 71.76 | 2.22 | - | - | - | - | - | - | - | - | - | - | - | - | 0.02 | 2.00 | 11.50 | 15.30 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles stand-ins:morepeeks cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:morepeeks cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.73
  command p90 ms                          21.76
  frames                                  63.00
  tick ms/frame                            0.91
  frame p90 ms                             1.02
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.90
  interval max ms (median burst)          10.90
  peek_frame ms/frame                      0.39
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
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)                7.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu0 bd-config bo-config #3 | move | 20.73 | 21.76 | 63.00 | 0.91 | 1.02 | 0.00 | 5.71 | 5.74 | 10.90 | 10.90 | 0.39 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.90 | 7.90 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          64.99
  command p90 ms                          74.04
  frames                                  60.00
  tick ms/frame                            3.79
  frame p90 ms                             3.65
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.89
  interval p99 ms                         71.48
  interval max ms (median burst)          71.48
  peek_frame ms/frame                      2.27
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            7.50
  cloak ms (outside frames)               17.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu0 bd-config bo-config #3 | resize | 64.99 | 74.04 | 60.00 | 3.79 | 3.65 | 4.00 | 5.71 | 5.89 | 71.48 | 71.48 | 2.27 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 7.50 | 17.80 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:morepeeks cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.99
  command p90 ms                          31.35
  frames                                 100.00
  tick ms/frame                            1.33
  frame p90 ms                             1.29
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.75
  interval p99 ms                         23.58
  interval max ms (median burst)          23.58
  peek_frame ms/frame                      0.00
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
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu0 bd-config bo-config #3 | switch | 29.99 | 31.35 | 100.00 | 1.33 | 1.29 | 2.00 | 5.72 | 5.75 | 23.58 | 23.58 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 6.20 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          58.28
  command p90 ms                          63.17
  frames                                  60.00
  tick ms/frame                            3.19
  frame p90 ms                             4.10
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.11
  interval p99 ms                         72.25
  interval max ms (median burst)          72.25
  peek_frame ms/frame                      2.14
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.40
  cloak ms (outside frames)               15.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu0 bd-config bo-config #3 | minimize | 58.28 | 63.17 | 60.00 | 3.19 | 4.10 | 4.00 | 5.72 | 7.11 | 72.25 | 72.25 | 2.14 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 9.40 | 15.80 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles main cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          13.72
  command p90 ms                          16.15
  frames                                  63.00
  tick ms/frame                            0.57
  frame p90 ms                             0.99
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.73
  interval p99 ms                          7.19
  interval max ms (median burst)           7.19
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
  batch_commit ms/frame                    0.29
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.00
  cloak ms (outside frames)                7.80
  session_begin ms (outside frames)        1.00
  session_overlays ms (outside frames)     3.10
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.23
  input->first frame ms                   14.63
| main cpu0 bd-config bo-config #3 | move | 13.72 | 16.15 | 63.00 | 0.57 | 0.99 | 0.00 | 5.71 | 5.73 | 7.19 | 7.19 | - | - | - | - | - | - | - | - | - | - | - | - | 0.29 | 0.00 | 1.00 | 5.00 | 7.80 | 1.00 | 3.10 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.23 | 14.63 |
label=main cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.88
  command p90 ms                          31.87
  frames                                  63.00
  tick ms/frame                            1.81
  frame p90 ms                             3.89
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         32.53
  interval max ms (median burst)          32.53
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
  batch_commit ms/frame                    0.95
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)               17.90
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)     6.90
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.22
  input->first frame ms                   35.25
| main cpu0 bd-config bo-config #3 | resize | 29.88 | 31.87 | 63.00 | 1.81 | 3.89 | 2.00 | 5.72 | 5.76 | 32.53 | 32.53 | - | - | - | - | - | - | - | - | - | - | - | - | 0.95 | 0.05 | 2.00 | 5.10 | 17.90 | 2.20 | 6.90 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.22 | 35.25 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.70
  command p90 ms                          21.01
  frames                                  97.00
  tick ms/frame                            1.65
  frame p90 ms                             2.47
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         51.08
  interval max ms (median burst)          51.08
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
  dwm_flush ms (outside frames)            5.80
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main cpu0 bd-config bo-config #3 | switch | 19.70 | 21.01 | 97.00 | 1.65 | 2.47 | 1.00 | 5.71 | 5.74 | 51.08 | 51.08 | - | - | - | - | - | - | - | - | - | - | - | - | 0.62 | 0.01 | 3.00 | 5.80 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=main cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.01
  command p90 ms                          17.36
  frames                                 151.00
  tick ms/frame                            1.37
  frame p90 ms                             3.95
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         33.73
  interval max ms (median burst)          34.16
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
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.20
  cloak ms (outside frames)               17.40
  session_begin ms (outside frames)        1.90
  session_overlays ms (outside frames)     6.80
  session_overlays ms/frame                0.01
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.17
  input->first frame ms                   43.25
| main cpu0 bd-config bo-config #3 | minimize | 16.01 | 17.36 | 151.00 | 1.37 | 3.95 | 2.00 | 5.71 | 5.75 | 33.73 | 34.16 | - | - | - | - | - | - | - | - | - | - | - | - | 0.66 | 0.02 | 4.00 | 4.20 | 17.40 | 1.90 | 6.80 | 0.01 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.17 | 43.25 |
tiles this cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.33
  command p90 ms                          19.43
  frames                                  63.00
  tick ms/frame                            0.38
  frame p90 ms                             0.53
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.73
  interval p99 ms                         10.36
  interval max ms (median burst)          10.36
  peek_frame ms/frame                      0.39
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.91
  pd_alpha ms (worst frame)                0.44
  pd_draw ms (worst frame)                 0.51
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.80
  cloak ms (outside frames)                8.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.00
  peek_new ms (outside frames)             2.00
  peek_place ms (outside frames)           1.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.10
  peek_handover ms (outside frames)        2.60
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              1.10
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.89
  input->first frame ms                   18.49
| this cpu0 bd-config bo-config #3 | move | 17.33 | 19.43 | 63.00 | 0.38 | 0.53 | 0.00 | 5.71 | 5.73 | 10.36 | 10.36 | 0.39 | 0.16 | 0.05 | 0.01 | 0.03 | 0.91 | 0.44 | 0.51 | - | - | - | - | - | - | - | 5.80 | 8.30 | - | - | - | 17.00 | 2.00 | 1.00 | - | - | - | 1.10 | 2.60 | 0.40 | 1.10 | - | - | - | - | - | 2.89 | 18.49 |
label=this cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          46.93
  command p90 ms                          52.62
  frames                                  62.00
  tick ms/frame                            2.28
  frame p90 ms                             3.31
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         37.06
  interval max ms (median burst)          37.06
  peek_frame ms/frame                      1.84
  peek_draw ms/frame                       1.20
  peek_track ms/frame                      0.29
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.12
  peek_draw ms (worst frame)              13.73
  pd_alpha ms (worst frame)                1.21
  pd_draw ms (worst frame)                12.43
  pd_fill ms (worst frame)                 8.85
  fill_source ms (worst frame)             1.83
  ring_raise ms (worst frame)              5.64
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.70
  cloak ms (outside frames)               17.50
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    46.60
  peek_new ms (outside frames)             3.70
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          11.20
  peek_handover ms (outside frames)       11.50
  peek_raise ms (outside frames)           0.70
  surface ms (outside frames)              1.70
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.30
  input->first frame ms                   63.95
| this cpu0 bd-config bo-config #3 | resize | 46.93 | 52.62 | 62.00 | 2.28 | 3.31 | 3.00 | 5.71 | 5.77 | 37.06 | 37.06 | 1.84 | 1.20 | 0.29 | 0.14 | 0.12 | 13.73 | 1.21 | 12.43 | 8.85 | 1.83 | 5.64 | 0.00 | - | 0.04 | 2.00 | 8.70 | 17.50 | - | - | - | 46.60 | 3.70 | 1.90 | - | - | - | 11.20 | 11.50 | 0.70 | 1.70 | - | - | - | - | - | 3.30 | 63.95 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          24.71
  command p90 ms                          26.34
  frames                                  99.00
  tick ms/frame                            1.05
  frame p90 ms                             1.06
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         19.68
  interval max ms (median burst)          19.68
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
  dwm_flush ms (outside frames)            3.80
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    24.40
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.50
  restack ms (outside frames)              0.00
  ws_surrogates ms (outside frames)       10.60
  ws_park ms (outside frames)              1.90
  ws_show ms (outside frames)              6.20
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| this cpu0 bd-config bo-config #3 | switch | 24.71 | 26.34 | 99.00 | 1.05 | 1.06 | 2.00 | 5.71 | 5.76 | 19.68 | 19.68 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 3.80 | - | - | - | - | 24.40 | - | - | - | - | - | - | - | 0.00 | 0.50 | 0.00 | 10.60 | 1.90 | 6.20 | 0.40 | - | - |
label=this cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          50.47
  command p90 ms                          57.72
  frames                                  61.00
  tick ms/frame                            2.30
  frame p90 ms                             3.49
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         58.89
  interval max ms (median burst)          58.89
  peek_frame ms/frame                      1.88
  peek_draw ms/frame                       1.21
  peek_track ms/frame                      0.37
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.20
  peek_draw ms (worst frame)              19.65
  pd_alpha ms (worst frame)                0.83
  pd_draw ms (worst frame)                18.86
  pd_fill ms (worst frame)                15.22
  fill_source ms (worst frame)             5.19
  ring_raise ms (worst frame)              5.71
  restack ms (worst frame)                 2.59
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.00
  cloak ms (outside frames)               16.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    50.20
  peek_new ms (outside frames)            13.90
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          23.80
  peek_handover ms (outside frames)       12.30
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              4.70
  restack ms (outside frames)              9.20
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.07
  input->first frame ms                   74.89
| this cpu0 bd-config bo-config #3 | minimize | 50.47 | 57.72 | 61.00 | 2.30 | 3.49 | 3.00 | 5.72 | 5.76 | 58.89 | 58.89 | 1.88 | 1.21 | 0.37 | 0.14 | 0.20 | 19.65 | 0.83 | 18.86 | 15.22 | 5.19 | 5.71 | 2.59 | - | 0.03 | 2.00 | 11.00 | 16.40 | - | - | - | 50.20 | 13.90 | 1.80 | - | - | - | 23.80 | 12.30 | 0.20 | 4.70 | 9.20 | - | - | - | - | 10.07 | 74.89 |
tiles stand-ins:nopeeks cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.14
  command p90 ms                          18.13
  frames                                  62.00
  tick ms/frame                            0.47
  frame p90 ms                             0.91
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          7.27
  interval max ms (median burst)           7.27
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
  batch_commit ms/frame                    0.27
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)                8.60
  session_begin ms (outside frames)        0.90
  session_overlays ms (outside frames)     4.40
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.28
  input->first frame ms                   16.70
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | move | 16.14 | 18.13 | 62.00 | 0.47 | 0.91 | 0.00 | 5.71 | 5.74 | 7.27 | 7.27 | - | - | - | - | - | - | - | - | - | - | - | - | 0.27 | 0.00 | - | 4.50 | 8.60 | 0.90 | 4.40 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.28 | 16.70 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          39.56
  command p90 ms                          43.25
  frames                                  62.00
  tick ms/frame                            1.79
  frame p90 ms                             4.31
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         23.13
  interval max ms (median burst)          23.13
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
  batch_commit ms/frame                    0.96
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.90
  cloak ms (outside frames)               19.60
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)    14.70
  session_overlays ms/frame                0.07
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.28
  input->first frame ms                   44.44
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | resize | 39.56 | 43.25 | 62.00 | 1.79 | 4.31 | 1.00 | 5.71 | 5.78 | 23.13 | 23.13 | - | - | - | - | - | - | - | - | - | - | - | - | 0.96 | 0.04 | - | 3.90 | 19.60 | 2.20 | 14.70 | 0.07 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.28 | 44.44 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          28.23
  command p90 ms                          29.35
  frames                                 100.00
  tick ms/frame                            1.20
  frame p90 ms                             1.17
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         20.91
  interval max ms (median burst)          21.81
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | switch | 28.23 | 29.35 | 100.00 | 1.20 | 1.17 | 3.00 | 5.71 | 5.76 | 20.91 | 21.81 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.03 | - | 5.70 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          23.21
  command p90 ms                          26.06
  frames                                 144.00
  tick ms/frame                            1.11
  frame p90 ms                             2.74
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         40.46
  interval max ms (median burst)          40.46
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
  batch_commit ms/frame                    0.59
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.30
  cloak ms (outside frames)               16.90
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)    15.70
  session_overlays ms/frame                0.03
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     5.28
  input->first frame ms                   47.10
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | minimize | 23.21 | 26.06 | 144.00 | 1.11 | 2.74 | 0.00 | 5.71 | 5.76 | 40.46 | 40.46 | - | - | - | - | - | - | - | - | - | - | - | - | 0.59 | 0.02 | 1.00 | 4.30 | 16.90 | 2.20 | 15.70 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 5.28 | 47.10 |
tiles stand-ins:morepeeks cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-move-mid.png (mid at 61 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-move-rest.png
label=stand-ins:morepeeks cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          22.52
  command p90 ms                          23.53
  frames                                  63.00
  tick ms/frame                            1.06
  frame p90 ms                             1.09
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.93
  interval max ms (median burst)          12.93
  peek_frame ms/frame                      0.39
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
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.30
  cloak ms (outside frames)                7.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu8 bd-config bo-config #1 | move | 22.52 | 23.53 | 63.00 | 1.06 | 1.09 | 0.00 | 5.71 | 5.76 | 12.93 | 12.93 | 0.39 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.30 | 7.30 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-resize-mid.png (mid at 75 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-resize-rest.png
label=stand-ins:morepeeks cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          72.07
  command p90 ms                          79.48
  frames                                  60.00
  tick ms/frame                            4.02
  frame p90 ms                             4.36
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.44
  interval p99 ms                         64.60
  interval max ms (median burst)          64.60
  peek_frame ms/frame                      2.63
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.60
  cloak ms (outside frames)               19.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu8 bd-config bo-config #1 | resize | 72.07 | 79.48 | 60.00 | 4.02 | 4.36 | 4.00 | 5.72 | 6.44 | 64.60 | 64.60 | 2.63 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 4.60 | 19.40 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-switch-mid.png (mid at 82 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-switch-rest.png
label=stand-ins:morepeeks cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          32.51
  command p90 ms                          35.56
  frames                                  99.00
  tick ms/frame                            1.44
  frame p90 ms                             1.41
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         25.14
  interval max ms (median burst)          25.96
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu8 bd-config bo-config #1 | switch | 32.51 | 35.56 | 99.00 | 1.44 | 1.41 | 3.00 | 5.71 | 5.76 | 25.14 | 25.96 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 5.60 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-minimize-mid.png (mid at 69 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-minimize-rest.png
label=stand-ins:morepeeks cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          65.70
  command p90 ms                          76.64
  frames                                  60.00
  tick ms/frame                            3.56
  frame p90 ms                             5.24
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          8.03
  interval p99 ms                         81.67
  interval max ms (median burst)          81.67
  peek_frame ms/frame                      2.43
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.30
  cloak ms (outside frames)               14.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu8 bd-config bo-config #1 | minimize | 65.70 | 76.64 | 60.00 | 3.56 | 5.24 | 4.00 | 5.71 | 8.03 | 81.67 | 81.67 | 2.43 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 9.30 | 14.80 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles main cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-main-default-move-mid.png (mid at 63 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-main-default-move-rest.png
label=main cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          14.98
  command p90 ms                          16.44
  frames                                  63.00
  tick ms/frame                            0.61
  frame p90 ms                             1.15
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                          8.41
  interval max ms (median burst)           8.41
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
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)                9.10
  session_begin ms (outside frames)        1.00
  session_overlays ms (outside frames)     2.80
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.29
  input->first frame ms                   16.15
| main cpu8 bd-config bo-config #1 | move | 14.98 | 16.44 | 63.00 | 0.61 | 1.15 | 0.00 | 5.71 | 5.77 | 8.41 | 8.41 | - | - | - | - | - | - | - | - | - | - | - | - | 0.34 | 0.00 | 1.00 | 5.40 | 9.10 | 1.00 | 2.80 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.29 | 16.15 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-main-default-resize-mid.png (mid at 66 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-main-default-resize-rest.png
label=main cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          32.94
  command p90 ms                          34.26
  frames                                  63.00
  tick ms/frame                            2.00
  frame p90 ms                             4.44
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         28.18
  interval max ms (median burst)          28.18
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
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)               19.20
  session_begin ms (outside frames)        2.60
  session_overlays ms (outside frames)     7.20
  session_overlays ms/frame                0.03
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.23
  input->first frame ms                   37.89
| main cpu8 bd-config bo-config #1 | resize | 32.94 | 34.26 | 63.00 | 2.00 | 4.44 | 3.00 | 5.71 | 5.76 | 28.18 | 28.18 | - | - | - | - | - | - | - | - | - | - | - | - | 1.03 | 0.05 | 2.00 | 4.50 | 19.20 | 2.60 | 7.20 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.23 | 37.89 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-main-default-switch-mid.png (mid at 73 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-main-default-switch-rest.png
label=main cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          21.24
  command p90 ms                          22.03
  frames                                  96.00
  tick ms/frame                            1.91
  frame p90 ms                             2.86
  frames >2x budget                        1.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         57.70
  interval max ms (median burst)          57.70
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
  batch_commit ms/frame                    0.72
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            4.70
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| main cpu8 bd-config bo-config #1 | switch | 21.24 | 22.03 | 96.00 | 1.91 | 2.86 | 1.00 | 5.72 | 5.76 | 57.70 | 57.70 | - | - | - | - | - | - | - | - | - | - | - | - | 0.72 | 0.02 | 3.00 | 4.70 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-main-default-minimize-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-main-default-minimize-rest.png
label=main cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          18.60
  command p90 ms                          19.95
  frames                                 150.00
  tick ms/frame                            1.67
  frame p90 ms                             4.30
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         46.37
  interval max ms (median burst)          46.37
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
  batch_commit ms/frame                    0.82
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)               23.30
  session_begin ms (outside frames)        4.90
  session_overlays ms (outside frames)     8.70
  session_overlays ms/frame                0.03
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     7.17
  input->first frame ms                   56.75
| main cpu8 bd-config bo-config #1 | minimize | 18.60 | 19.95 | 150.00 | 1.67 | 4.30 | 2.00 | 5.71 | 5.77 | 46.37 | 46.37 | - | - | - | - | - | - | - | - | - | - | - | - | 0.82 | 0.03 | 4.00 | 4.80 | 23.30 | 4.90 | 8.70 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 7.17 | 56.75 |
tiles this cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-this-move-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-this-move-rest.png
label=this cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.69
  command p90 ms                          18.61
  frames                                  63.00
  tick ms/frame                            0.43
  frame p90 ms                             0.56
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.16
  interval max ms (median burst)          12.16
  peek_frame ms/frame                      0.35
  peek_draw ms/frame                       0.18
  peek_track ms/frame                      0.06
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.90
  pd_alpha ms (worst frame)                0.51
  pd_draw ms (worst frame)                 0.49
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
  command ms (WM side)                    16.40
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.20
  peek_handover ms (outside frames)        3.10
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              1.20
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.39
  input->first frame ms                   17.97
| this cpu8 bd-config bo-config #1 | move | 16.69 | 18.61 | 63.00 | 0.43 | 0.56 | 0.00 | 5.71 | 5.76 | 12.16 | 12.16 | 0.35 | 0.18 | 0.06 | 0.01 | 0.04 | 0.90 | 0.51 | 0.49 | - | - | - | - | - | - | - | 4.40 | 7.10 | - | - | - | 16.40 | 2.10 | 1.00 | - | - | - | 1.20 | 3.10 | 0.40 | 1.20 | - | - | - | - | - | 3.39 | 17.97 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-this-resize-mid.png (mid at 66 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-this-resize-rest.png
label=this cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          51.06
  command p90 ms                          56.39
  frames                                  61.00
  tick ms/frame                            2.58
  frame p90 ms                             3.91
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         37.59
  interval max ms (median burst)          37.59
  peek_frame ms/frame                      2.08
  peek_draw ms/frame                       1.38
  peek_track ms/frame                      0.31
  peek_ring ms/frame                       0.16
  peek_backdrop ms/frame                   0.13
  peek_draw ms (worst frame)              17.56
  pd_alpha ms (worst frame)                1.19
  pd_draw ms (worst frame)                16.43
  pd_fill ms (worst frame)                12.57
  fill_source ms (worst frame)             3.73
  ring_raise ms (worst frame)              5.86
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.80
  cloak ms (outside frames)               17.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    50.60
  peek_new ms (outside frames)             4.50
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          11.70
  peek_handover ms (outside frames)       11.50
  peek_raise ms (outside frames)           0.70
  surface ms (outside frames)              2.20
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.73
  input->first frame ms                   70.43
| this cpu8 bd-config bo-config #1 | resize | 51.06 | 56.39 | 61.00 | 2.58 | 3.91 | 3.00 | 5.72 | 5.77 | 37.59 | 37.59 | 2.08 | 1.38 | 0.31 | 0.16 | 0.13 | 17.56 | 1.19 | 16.43 | 12.57 | 3.73 | 5.86 | 0.00 | - | 0.03 | 2.00 | 8.80 | 17.90 | - | - | - | 50.60 | 4.50 | 2.00 | - | - | - | 11.70 | 11.50 | 0.70 | 2.20 | - | - | - | - | - | 3.73 | 70.43 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-this-switch-mid.png (mid at 69 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-this-switch-rest.png
label=this cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          28.65
  command p90 ms                          29.74
  frames                                  99.00
  tick ms/frame                            1.14
  frame p90 ms                             1.22
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         22.66
  interval max ms (median burst)          22.66
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
  dwm_flush ms (outside frames)            5.50
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    28.40
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)       12.00
  ws_park ms (outside frames)              2.10
  ws_show ms (outside frames)              8.10
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| this cpu8 bd-config bo-config #1 | switch | 28.65 | 29.74 | 99.00 | 1.14 | 1.22 | 3.00 | 5.71 | 5.76 | 22.66 | 22.66 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | - | 5.50 | - | - | - | - | 28.40 | - | - | - | - | - | - | - | 0.00 | 0.60 | - | 12.00 | 2.10 | 8.10 | 0.40 | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-this-minimize-mid.png (mid at 64 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-this-minimize-rest.png
label=this cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          58.82
  command p90 ms                          63.84
  frames                                  61.00
  tick ms/frame                            2.48
  frame p90 ms                             4.15
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         67.84
  interval max ms (median burst)          67.84
  peek_frame ms/frame                      2.10
  peek_draw ms/frame                       1.35
  peek_track ms/frame                      0.38
  peek_ring ms/frame                       0.15
  peek_backdrop ms/frame                   0.21
  peek_draw ms (worst frame)              20.91
  pd_alpha ms (worst frame)                1.00
  pd_draw ms (worst frame)                19.72
  pd_fill ms (worst frame)                15.97
  fill_source ms (worst frame)             6.69
  ring_raise ms (worst frame)              5.81
  restack ms (worst frame)                 3.05
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           12.70
  cloak ms (outside frames)               18.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    58.50
  peek_new ms (outside frames)            14.10
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          24.60
  peek_handover ms (outside frames)       15.00
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              6.20
  restack ms (outside frames)              9.60
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    12.39
  input->first frame ms                   85.05
| this cpu8 bd-config bo-config #1 | minimize | 58.82 | 63.84 | 61.00 | 2.48 | 4.15 | 3.00 | 5.71 | 5.78 | 67.84 | 67.84 | 2.10 | 1.35 | 0.38 | 0.15 | 0.21 | 20.91 | 1.00 | 19.72 | 15.97 | 6.69 | 5.81 | 3.05 | - | 0.03 | 2.00 | 12.70 | 18.60 | - | - | - | 58.50 | 14.10 | 1.80 | - | - | - | 24.60 | 15.00 | 0.30 | 6.20 | 9.60 | - | - | - | - | 12.39 | 85.05 |
tiles stand-ins:nopeeks cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-move-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-move-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.56
  command p90 ms                          18.47
  frames                                  63.00
  tick ms/frame                            0.53
  frame p90 ms                             1.06
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                          8.29
  interval max ms (median burst)           8.29
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
  batch_commit ms/frame                    0.30
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.70
  cloak ms (outside frames)                8.60
  session_begin ms (outside frames)        1.10
  session_overlays ms (outside frames)     4.60
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.31
  input->first frame ms                   17.19
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | move | 16.56 | 18.47 | 63.00 | 0.53 | 1.06 | 0.00 | 5.71 | 5.76 | 8.29 | 8.29 | - | - | - | - | - | - | - | - | - | - | - | - | 0.30 | 0.00 | - | 3.70 | 8.60 | 1.10 | 4.60 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.31 | 17.19 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-resize-mid.png (mid at 65 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-resize-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          43.90
  command p90 ms                          45.94
  frames                                  62.00
  tick ms/frame                            1.82
  frame p90 ms                             4.58
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         16.66
  interval max ms (median burst)          16.66
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
  batch_commit ms/frame                    1.06
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.90
  cloak ms (outside frames)               22.20
  session_begin ms (outside frames)        2.50
  session_overlays ms (outside frames)    15.30
  session_overlays ms/frame                0.06
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.31
  input->first frame ms                   48.15
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | resize | 43.90 | 45.94 | 62.00 | 1.82 | 4.58 | 1.00 | 5.71 | 5.79 | 16.66 | 16.66 | - | - | - | - | - | - | - | - | - | - | - | - | 1.06 | 0.02 | - | 3.90 | 22.20 | 2.50 | 15.30 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.31 | 48.15 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-switch-mid.png (mid at 79 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-switch-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          31.01
  command p90 ms                          33.92
  frames                                  99.00
  tick ms/frame                            1.37
  frame p90 ms                             1.56
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         25.01
  interval max ms (median burst)          25.01
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
  dwm_flush ms (outside frames)            4.70
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | switch | 31.01 | 33.92 | 99.00 | 1.37 | 1.56 | 3.00 | 5.71 | 5.76 | 25.01 | 25.01 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 4.70 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-minimize-mid.png (mid at 74 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-minimize-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          30.03
  command p90 ms                          30.88
  frames                                 143.00
  tick ms/frame                            1.25
  frame p90 ms                             2.71
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         59.56
  interval max ms (median burst)          59.56
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)               30.60
  session_begin ms (outside frames)        5.50
  session_overlays ms (outside frames)    18.20
  session_overlays ms/frame                0.03
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.14
  input->first frame ms                   70.23
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | minimize | 30.03 | 30.88 | 143.00 | 1.25 | 2.71 | 1.00 | 5.71 | 5.77 | 59.56 | 59.56 | - | - | - | - | - | - | - | - | - | - | - | - | 0.65 | 0.02 | 1.00 | 4.70 | 30.60 | 5.50 | 18.20 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.14 | 70.23 |
tiles stand-ins cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-default-move-mid.png (mid at 65 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-default-move-rest.png
label=stand-ins cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          21.58
  command p90 ms                          22.16
  frames                                  63.00
  tick ms/frame                            0.98
  frame p90 ms                             1.08
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         12.80
  interval max ms (median burst)          12.80
  peek_frame ms/frame                      0.37
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
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)                8.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu8 bd-config bo-config #1 | move | 21.58 | 22.16 | 63.00 | 0.98 | 1.08 | 0.00 | 5.71 | 5.75 | 12.80 | 12.80 | 0.37 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.50 | 8.00 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-default-resize-mid.png (mid at 67 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-default-resize-rest.png
label=stand-ins cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          69.78
  command p90 ms                          79.41
  frames                                  59.00
  tick ms/frame                            4.11
  frame p90 ms                             4.97
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.70
  interval p99 ms                         63.30
  interval max ms (median burst)          63.30
  peek_frame ms/frame                      2.60
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.30
  cloak ms (outside frames)               19.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu8 bd-config bo-config #1 | resize | 69.78 | 79.41 | 59.00 | 4.11 | 4.97 | 4.00 | 5.72 | 6.70 | 63.30 | 63.30 | 2.60 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 10.30 | 19.60 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-default-switch-mid.png (mid at 83 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-default-switch-rest.png
label=stand-ins cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          32.37
  command p90 ms                          33.64
  frames                                  99.00
  tick ms/frame                            1.46
  frame p90 ms                             1.50
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         25.95
  interval max ms (median burst)          25.95
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
  dwm_flush ms (outside frames)            5.20
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu8 bd-config bo-config #1 | switch | 32.37 | 33.64 | 99.00 | 1.46 | 1.50 | 3.00 | 5.71 | 5.76 | 25.95 | 25.95 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 5.20 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-default-minimize-mid.png (mid at 70 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-095958\r1-cpu8-bdconfigboconfig-stand-ins-default-minimize-rest.png
label=stand-ins cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          70.96
  command p90 ms                          78.72
  frames                                  60.00
  tick ms/frame                            3.65
  frame p90 ms                             4.96
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          7.47
  interval p99 ms                         81.04
  interval max ms (median burst)          81.04
  peek_frame ms/frame                      2.44
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.20
  cloak ms (outside frames)               19.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu8 bd-config bo-config #1 | minimize | 70.96 | 78.72 | 60.00 | 3.65 | 4.96 | 4.00 | 5.71 | 7.47 | 81.04 | 81.04 | 2.44 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 11.20 | 19.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles main cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          14.00
  command p90 ms                          16.12
  frames                                  63.00
  tick ms/frame                            0.62
  frame p90 ms                             1.16
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                          8.07
  interval max ms (median burst)           8.07
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
  batch_commit ms/frame                    0.35
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)                8.20
  session_begin ms (outside frames)        1.00
  session_overlays ms (outside frames)     2.90
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.25
  input->first frame ms                   15.18
| main cpu8 bd-config bo-config #2 | move | 14.00 | 16.12 | 63.00 | 0.62 | 1.16 | 0.00 | 5.71 | 5.76 | 8.07 | 8.07 | - | - | - | - | - | - | - | - | - | - | - | - | 0.35 | 0.00 | 1.00 | 4.70 | 8.20 | 1.00 | 2.90 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.25 | 15.18 |
label=main cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          32.14
  command p90 ms                          33.32
  frames                                  63.00
  tick ms/frame                            1.94
  frame p90 ms                             4.36
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         26.39
  interval max ms (median burst)          26.39
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
  batch_commit ms/frame                    1.05
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            3.70
  cloak ms (outside frames)               19.00
  session_begin ms (outside frames)        2.70
  session_overlays ms (outside frames)     7.40
  session_overlays ms/frame                0.03
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.23
  input->first frame ms                   36.86
| main cpu8 bd-config bo-config #2 | resize | 32.14 | 33.32 | 63.00 | 1.94 | 4.36 | 2.00 | 5.71 | 5.76 | 26.39 | 26.39 | - | - | - | - | - | - | - | - | - | - | - | - | 1.05 | 0.05 | 2.00 | 3.70 | 19.00 | 2.70 | 7.40 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.23 | 36.86 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          21.54
  command p90 ms                          22.88
  frames                                  97.00
  tick ms/frame                            1.87
  frame p90 ms                             2.67
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         56.38
  interval max ms (median burst)          56.78
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
  batch_commit ms/frame                    0.68
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.80
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main cpu8 bd-config bo-config #2 | switch | 21.54 | 22.88 | 97.00 | 1.87 | 2.67 | 1.00 | 5.71 | 5.76 | 56.38 | 56.78 | - | - | - | - | - | - | - | - | - | - | - | - | 0.68 | 0.02 | 3.00 | 5.80 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=main cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          19.93
  command p90 ms                          21.91
  frames                                 150.00
  tick ms/frame                            1.63
  frame p90 ms                             4.57
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         35.54
  interval max ms (median burst)          35.54
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
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            5.20
  cloak ms (outside frames)               18.20
  session_begin ms (outside frames)        2.30
  session_overlays ms (outside frames)     8.20
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.96
  input->first frame ms                   46.38
| main cpu8 bd-config bo-config #2 | minimize | 19.93 | 21.91 | 150.00 | 1.63 | 4.57 | 1.00 | 5.71 | 5.76 | 35.54 | 35.54 | - | - | - | - | - | - | - | - | - | - | - | - | 0.74 | 0.03 | 4.00 | 5.20 | 18.20 | 2.30 | 8.20 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.96 | 46.38 |
tiles this cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.52
  command p90 ms                          18.40
  frames                                  63.00
  tick ms/frame                            0.43
  frame p90 ms                             0.58
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         11.80
  interval max ms (median burst)          11.80
  peek_frame ms/frame                      0.33
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.06
  peek_ring ms/frame                       0.02
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.87
  pd_alpha ms (worst frame)                0.50
  pd_draw ms (worst frame)                 0.49
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
  command ms (WM side)                    17.20
  peek_new ms (outside frames)             2.30
  peek_place ms (outside frames)           1.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.60
  peek_handover ms (outside frames)        3.10
  peek_raise ms (outside frames)           0.50
  surface ms (outside frames)              1.30
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.17
  input->first frame ms                   18.73
| this cpu8 bd-config bo-config #2 | move | 17.52 | 18.40 | 63.00 | 0.43 | 0.58 | 0.00 | 5.71 | 5.76 | 11.80 | 11.80 | 0.33 | 0.17 | 0.06 | 0.02 | 0.04 | 0.87 | 0.50 | 0.49 | - | - | - | - | - | - | - | 4.60 | 7.30 | - | - | - | 17.20 | 2.30 | 1.00 | - | - | - | 1.60 | 3.10 | 0.50 | 1.30 | - | - | - | - | - | 3.17 | 18.73 |
label=this cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          52.75
  command p90 ms                          57.40
  frames                                  61.00
  tick ms/frame                            2.60
  frame p90 ms                             4.31
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         42.99
  interval max ms (median burst)          42.99
  peek_frame ms/frame                      2.18
  peek_draw ms/frame                       1.45
  peek_track ms/frame                      0.32
  peek_ring ms/frame                       0.16
  peek_backdrop ms/frame                   0.14
  peek_draw ms (worst frame)              14.66
  pd_alpha ms (worst frame)                1.25
  pd_draw ms (worst frame)                13.44
  pd_fill ms (worst frame)                 9.44
  fill_source ms (worst frame)             2.26
  ring_raise ms (worst frame)              6.08
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.50
  cloak ms (outside frames)               18.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    52.40
  peek_new ms (outside frames)             4.40
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          12.00
  peek_handover ms (outside frames)       12.70
  peek_raise ms (outside frames)           0.70
  surface ms (outside frames)              2.00
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.97
  input->first frame ms                   71.83
| this cpu8 bd-config bo-config #2 | resize | 52.75 | 57.40 | 61.00 | 2.60 | 4.31 | 3.00 | 5.72 | 5.78 | 42.99 | 42.99 | 2.18 | 1.45 | 0.32 | 0.16 | 0.14 | 14.66 | 1.25 | 13.44 | 9.44 | 2.26 | 6.08 | 0.00 | - | 0.04 | 2.00 | 9.50 | 18.60 | - | - | - | 52.40 | 4.40 | 2.00 | - | - | - | 12.00 | 12.70 | 0.70 | 2.00 | - | - | - | - | - | 3.97 | 71.83 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.08
  command p90 ms                          30.81
  frames                                  99.00
  tick ms/frame                            1.17
  frame p90 ms                             1.22
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         23.18
  interval max ms (median burst)          23.18
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
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    28.80
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  ws_surrogates ms (outside frames)       12.10
  ws_park ms (outside frames)              2.10
  ws_show ms (outside frames)              7.80
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| this cpu8 bd-config bo-config #2 | switch | 29.08 | 30.81 | 99.00 | 1.17 | 1.22 | 3.00 | 5.71 | 5.75 | 23.18 | 23.18 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.04 | - | 5.40 | - | - | - | - | 28.80 | - | - | - | - | - | - | - | 0.00 | 0.60 | 0.00 | 12.10 | 2.10 | 7.80 | 0.40 | - | - |
label=this cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          54.86
  command p90 ms                          61.43
  frames                                  60.00
  tick ms/frame                            2.45
  frame p90 ms                             4.49
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.81
  interval p99 ms                         53.57
  interval max ms (median burst)          53.57
  peek_frame ms/frame                      2.17
  peek_draw ms/frame                       1.33
  peek_track ms/frame                      0.39
  peek_ring ms/frame                       0.15
  peek_backdrop ms/frame                   0.20
  peek_draw ms (worst frame)              20.73
  pd_alpha ms (worst frame)                1.05
  pd_draw ms (worst frame)                19.75
  pd_fill ms (worst frame)                16.26
  fill_source ms (worst frame)             5.32
  ring_raise ms (worst frame)              6.84
  restack ms (worst frame)                 2.99
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.00
  cloak ms (outside frames)               17.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    54.60
  peek_new ms (outside frames)             8.40
  peek_place ms (outside frames)           1.70
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          24.60
  peek_handover ms (outside frames)       14.30
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              2.80
  restack ms (outside frames)              9.50
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.17
  input->first frame ms                   81.39
| this cpu8 bd-config bo-config #2 | minimize | 54.86 | 61.43 | 60.00 | 2.45 | 4.49 | 3.00 | 5.71 | 5.81 | 53.57 | 53.57 | 2.17 | 1.33 | 0.39 | 0.15 | 0.20 | 20.73 | 1.05 | 19.75 | 16.26 | 5.32 | 6.84 | 2.99 | - | 0.03 | 2.00 | 10.00 | 17.20 | - | - | - | 54.60 | 8.40 | 1.70 | - | - | - | 24.60 | 14.30 | 0.30 | 2.80 | 9.50 | - | - | - | - | 10.17 | 81.39 |
tiles stand-ins:nopeeks cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.16
  command p90 ms                          17.82
  frames                                  63.00
  tick ms/frame                            0.55
  frame p90 ms                             1.04
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                          8.41
  interval max ms (median burst)           8.41
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
  batch_commit ms/frame                    0.30
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.40
  cloak ms (outside frames)                7.90
  session_begin ms (outside frames)        1.00
  session_overlays ms (outside frames)     5.00
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.32
  input->first frame ms                   16.80
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | move | 16.16 | 17.82 | 63.00 | 0.55 | 1.04 | 0.00 | 5.71 | 5.76 | 8.41 | 8.41 | - | - | - | - | - | - | - | - | - | - | - | - | 0.30 | 0.00 | - | 3.40 | 7.90 | 1.00 | 5.00 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.32 | 16.80 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          45.50
  command p90 ms                          47.38
  frames                                  62.00
  tick ms/frame                            1.86
  frame p90 ms                             5.00
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         17.01
  interval max ms (median burst)          17.01
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
  batch_commit ms/frame                    1.09
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.60
  cloak ms (outside frames)               23.30
  session_begin ms (outside frames)        2.60
  session_overlays ms (outside frames)    17.40
  session_overlays ms/frame                0.06
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.29
  input->first frame ms                   50.67
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | resize | 45.50 | 47.38 | 62.00 | 1.86 | 5.00 | 1.00 | 5.71 | 5.78 | 17.01 | 17.01 | - | - | - | - | - | - | - | - | - | - | - | - | 1.09 | 0.04 | - | 3.60 | 23.30 | 2.60 | 17.40 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.29 | 50.67 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          31.46
  command p90 ms                          33.84
  frames                                  99.00
  tick ms/frame                            1.39
  frame p90 ms                             1.44
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         24.07
  interval max ms (median burst)          24.07
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
  dwm_flush ms (outside frames)            5.30
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | switch | 31.46 | 33.84 | 99.00 | 1.39 | 1.44 | 3.00 | 5.71 | 5.76 | 24.07 | 24.07 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.04 | - | 5.30 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          25.28
  command p90 ms                          29.63
  frames                                 143.00
  tick ms/frame                            1.32
  frame p90 ms                             3.28
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         45.70
  interval max ms (median burst)          45.70
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
  dwm_flush ms (outside frames)            4.60
  cloak ms (outside frames)               18.10
  session_begin ms (outside frames)        2.50
  session_overlays ms (outside frames)    18.80
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.08
  input->first frame ms                   53.29
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | minimize | 25.28 | 29.63 | 143.00 | 1.32 | 3.28 | 0.00 | 5.71 | 5.77 | 45.70 | 45.70 | - | - | - | - | - | - | - | - | - | - | - | - | 0.67 | 0.02 | 1.00 | 4.60 | 18.10 | 2.50 | 18.80 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.08 | 53.29 |
tiles stand-ins cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.22
  command p90 ms                          22.56
  frames                                  63.00
  tick ms/frame                            0.97
  frame p90 ms                             1.00
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         13.10
  interval max ms (median burst)          13.10
  peek_frame ms/frame                      0.38
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
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.30
  cloak ms (outside frames)                5.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu8 bd-config bo-config #2 | move | 20.22 | 22.56 | 63.00 | 0.97 | 1.00 | 0.00 | 5.71 | 5.76 | 13.10 | 13.10 | 0.38 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 3.30 | 5.90 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          69.38
  command p90 ms                          72.98
  frames                                  60.00
  tick ms/frame                            4.32
  frame p90 ms                             3.88
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          6.42
  interval p99 ms                         60.47
  interval max ms (median burst)          60.47
  peek_frame ms/frame                      2.59
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            3.40
  cloak ms (outside frames)               16.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu8 bd-config bo-config #2 | resize | 69.38 | 72.98 | 60.00 | 4.32 | 3.88 | 4.00 | 5.71 | 6.42 | 60.47 | 60.47 | 2.59 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 3.40 | 16.90 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          33.36
  command p90 ms                          35.28
  frames                                  99.00
  tick ms/frame                            1.47
  frame p90 ms                             1.38
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         26.89
  interval max ms (median burst)          26.89
  peek_frame ms/frame                      0.00
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu8 bd-config bo-config #2 | switch | 33.36 | 35.28 | 99.00 | 1.47 | 1.38 | 3.00 | 5.71 | 5.76 | 26.89 | 26.89 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 7.00 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          66.62
  command p90 ms                          70.97
  frames                                  59.00
  tick ms/frame                            3.65
  frame p90 ms                             5.37
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.95
  interval p99 ms                         81.92
  interval max ms (median burst)          81.92
  peek_frame ms/frame                      2.52
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.90
  cloak ms (outside frames)               17.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu8 bd-config bo-config #2 | minimize | 66.62 | 70.97 | 59.00 | 3.65 | 5.37 | 4.00 | 5.72 | 7.95 | 81.92 | 81.92 | 2.52 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 9.90 | 17.60 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles stand-ins:morepeeks cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:morepeeks cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          22.09
  command p90 ms                          23.37
  frames                                  63.00
  tick ms/frame                            1.02
  frame p90 ms                             1.08
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.96
  interval max ms (median burst)          12.96
  peek_frame ms/frame                      0.40
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
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.80
  cloak ms (outside frames)                7.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu8 bd-config bo-config #2 | move | 22.09 | 23.37 | 63.00 | 1.02 | 1.08 | 0.00 | 5.71 | 5.76 | 12.96 | 12.96 | 0.40 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 3.80 | 7.40 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          72.26
  command p90 ms                          80.86
  frames                                  59.00
  tick ms/frame                            3.98
  frame p90 ms                             4.09
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.93
  interval p99 ms                         62.49
  interval max ms (median burst)          62.49
  peek_frame ms/frame                      2.67
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)               17.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu8 bd-config bo-config #2 | resize | 72.26 | 80.86 | 59.00 | 3.98 | 4.09 | 4.00 | 5.71 | 5.93 | 62.49 | 62.49 | 2.67 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 4.80 | 17.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:morepeeks cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          31.45
  command p90 ms                          34.90
  frames                                  99.00
  tick ms/frame                            1.48
  frame p90 ms                             1.66
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         26.59
  interval max ms (median burst)          26.59
  peek_frame ms/frame                      0.00
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
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu8 bd-config bo-config #2 | switch | 31.45 | 34.90 | 99.00 | 1.48 | 1.66 | 3.00 | 5.71 | 5.76 | 26.59 | 26.59 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 4.80 | 0.00 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          63.77
  command p90 ms                          71.38
  frames                                  60.00
  tick ms/frame                            3.52
  frame p90 ms                             4.82
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.59
  interval p99 ms                         82.24
  interval max ms (median burst)          82.24
  peek_frame ms/frame                      2.47
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.90
  cloak ms (outside frames)               16.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu8 bd-config bo-config #2 | minimize | 63.77 | 71.38 | 60.00 | 3.52 | 4.82 | 4.00 | 5.72 | 6.59 | 82.24 | 82.24 | 2.47 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 8.90 | 16.40 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles this cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.92
  command p90 ms                          19.02
  frames                                  63.00
  tick ms/frame                            0.43
  frame p90 ms                             0.56
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.36
  interval max ms (median burst)          12.36
  peek_frame ms/frame                      0.36
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.06
  peek_ring ms/frame                       0.02
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.90
  pd_alpha ms (worst frame)                0.52
  pd_draw ms (worst frame)                 0.50
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.60
  cloak ms (outside frames)                8.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.70
  peek_new ms (outside frames)             2.20
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.20
  peek_handover ms (outside frames)        3.30
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              1.30
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.37
  input->first frame ms                   19.38
| this cpu8 bd-config bo-config #3 | move | 17.92 | 19.02 | 63.00 | 0.43 | 0.56 | 0.00 | 5.71 | 5.76 | 12.36 | 12.36 | 0.36 | 0.17 | 0.06 | 0.02 | 0.04 | 0.90 | 0.52 | 0.50 | - | - | - | - | - | - | - | 5.60 | 8.30 | - | - | - | 17.70 | 2.20 | 1.10 | - | - | - | 1.20 | 3.30 | 0.40 | 1.30 | - | - | - | - | - | 3.37 | 19.38 |
label=this cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=12 wall=12.7s
  command p50 ms                          52.61
  command p90 ms                          62.98
  frames                                  62.00
  tick ms/frame                            2.54
  frame p90 ms                             4.17
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.87
  interval p99 ms                         38.43
  interval max ms (median burst)          38.43
  peek_frame ms/frame                      2.12
  peek_draw ms/frame                       1.39
  peek_track ms/frame                      0.29
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.13
  peek_draw ms (worst frame)              17.04
  pd_alpha ms (worst frame)                1.18
  pd_draw ms (worst frame)                15.67
  pd_fill ms (worst frame)                11.65
  fill_source ms (worst frame)             2.33
  ring_raise ms (worst frame)              5.47
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            6.80
  cloak ms (outside frames)               19.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    50.50
  peek_new ms (outside frames)             3.70
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          11.80
  peek_handover ms (outside frames)        6.50
  peek_raise ms (outside frames)           0.60
  surface ms (outside frames)              1.80
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.52
  input->first frame ms                   71.44
| this cpu8 bd-config bo-config #3 | resize | 52.61 | 62.98 | 62.00 | 2.54 | 4.17 | 3.00 | 5.71 | 5.87 | 38.43 | 38.43 | 2.12 | 1.39 | 0.29 | 0.14 | 0.13 | 17.04 | 1.18 | 15.67 | 11.65 | 2.33 | 5.47 | 0.00 | - | 0.04 | 1.00 | 6.80 | 19.20 | - | - | - | 50.50 | 3.70 | 2.00 | - | - | - | 11.80 | 6.50 | 0.60 | 1.80 | - | - | - | - | - | 2.52 | 71.44 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          28.76
  command p90 ms                          30.56
  frames                                  98.00
  tick ms/frame                            1.17
  frame p90 ms                             1.23
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         21.43
  interval max ms (median burst)          21.43
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
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            6.30
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    28.40
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  ws_surrogates ms (outside frames)       11.80
  ws_park ms (outside frames)              2.10
  ws_show ms (outside frames)              8.80
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| this cpu8 bd-config bo-config #3 | switch | 28.76 | 30.56 | 98.00 | 1.17 | 1.23 | 3.00 | 5.71 | 5.76 | 21.43 | 21.43 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | 0.00 | - | - | 0.03 | - | 6.30 | - | - | - | - | 28.40 | - | - | - | - | - | - | - | 0.00 | 0.60 | 0.00 | 11.80 | 2.10 | 8.80 | 0.40 | - | - |
label=this cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          60.14
  command p90 ms                          62.89
  frames                                  62.00
  tick ms/frame                            2.46
  frame p90 ms                             4.03
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         52.76
  interval max ms (median burst)          52.76
  peek_frame ms/frame                      2.01
  peek_draw ms/frame                       1.30
  peek_track ms/frame                      0.38
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.21
  peek_draw ms (worst frame)              16.10
  pd_alpha ms (worst frame)                0.97
  pd_draw ms (worst frame)                14.93
  pd_fill ms (worst frame)                10.99
  fill_source ms (worst frame)             2.22
  ring_raise ms (worst frame)              5.64
  restack ms (worst frame)                 3.14
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.10
  cloak ms (outside frames)               17.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    59.90
  peek_new ms (outside frames)             8.30
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          25.90
  peek_handover ms (outside frames)       13.40
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              3.00
  restack ms (outside frames)             10.10
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.68
  input->first frame ms                   79.49
| this cpu8 bd-config bo-config #3 | minimize | 60.14 | 62.89 | 62.00 | 2.46 | 4.03 | 3.00 | 5.71 | 5.77 | 52.76 | 52.76 | 2.01 | 1.30 | 0.38 | 0.14 | 0.21 | 16.10 | 0.97 | 14.93 | 10.99 | 2.22 | 5.64 | 3.14 | - | 0.03 | 2.00 | 10.10 | 17.40 | - | - | - | 59.90 | 8.30 | 1.80 | - | - | - | 25.90 | 13.40 | 0.30 | 3.00 | 10.10 | - | - | - | - | 10.68 | 79.49 |
tiles stand-ins:nopeeks cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.06
  command p90 ms                          18.97
  frames                                  63.00
  tick ms/frame                            0.52
  frame p90 ms                             1.04
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                          8.25
  interval max ms (median burst)           8.25
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
  batch_commit ms/frame                    0.29
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)                9.70
  session_begin ms (outside frames)        1.10
  session_overlays ms (outside frames)     4.60
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.31
  input->first frame ms                   18.76
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | move | 18.06 | 18.97 | 63.00 | 0.52 | 1.04 | 0.00 | 5.71 | 5.76 | 8.25 | 8.25 | - | - | - | - | - | - | - | - | - | - | - | - | 0.29 | 0.00 | - | 4.90 | 9.70 | 1.10 | 4.60 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.31 | 18.76 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          44.16
  command p90 ms                          48.08
  frames                                  62.00
  tick ms/frame                            1.87
  frame p90 ms                             4.87
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.81
  interval p99 ms                         20.43
  interval max ms (median burst)          20.43
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
  batch_commit ms/frame                    1.09
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.30
  cloak ms (outside frames)               21.40
  session_begin ms (outside frames)        2.70
  session_overlays ms (outside frames)    17.30
  session_overlays ms/frame                0.05
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.34
  input->first frame ms                   48.88
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | resize | 44.16 | 48.08 | 62.00 | 1.87 | 4.87 | 2.00 | 5.72 | 5.81 | 20.43 | 20.43 | - | - | - | - | - | - | - | - | - | - | - | - | 1.09 | 0.03 | - | 4.30 | 21.40 | 2.70 | 17.30 | 0.05 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.34 | 48.88 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          31.50
  command p90 ms                          33.39
  frames                                  99.00
  tick ms/frame                            1.37
  frame p90 ms                             1.47
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         24.39
  interval max ms (median burst)          24.39
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
  dwm_flush ms (outside frames)            5.20
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | switch | 31.50 | 33.39 | 99.00 | 1.37 | 1.47 | 3.00 | 5.71 | 5.76 | 24.39 | 24.39 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.03 | - | 5.20 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          26.69
  command p90 ms                          29.89
  frames                                 143.00
  tick ms/frame                            1.32
  frame p90 ms                             3.41
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         48.02
  interval max ms (median burst)          48.02
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.60
  cloak ms (outside frames)               21.80
  session_begin ms (outside frames)        2.50
  session_overlays ms (outside frames)    16.70
  session_overlays ms/frame                0.04
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     5.86
  input->first frame ms                   58.33
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | minimize | 26.69 | 29.89 | 143.00 | 1.32 | 3.41 | 0.00 | 5.71 | 5.77 | 48.02 | 48.02 | - | - | - | - | - | - | - | - | - | - | - | - | 0.75 | 0.02 | 1.00 | 4.60 | 21.80 | 2.50 | 16.70 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 5.86 | 58.33 |
tiles stand-ins cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.96
  command p90 ms                          21.99
  frames                                  64.00
  tick ms/frame                            1.03
  frame p90 ms                             1.10
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         13.13
  interval max ms (median burst)          13.13
  peek_frame ms/frame                      0.39
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
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)                7.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu8 bd-config bo-config #3 | move | 20.96 | 21.99 | 64.00 | 1.03 | 1.10 | 0.00 | 5.71 | 5.75 | 13.13 | 13.13 | 0.39 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.50 | 7.80 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          69.11
  command p90 ms                          78.53
  frames                                  59.00
  tick ms/frame                            4.13
  frame p90 ms                             4.69
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.94
  interval p99 ms                         49.16
  interval max ms (median burst)          49.16
  peek_frame ms/frame                      2.70
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)               18.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu8 bd-config bo-config #3 | resize | 69.11 | 78.53 | 59.00 | 4.13 | 4.69 | 5.00 | 5.71 | 6.94 | 49.16 | 49.16 | 2.70 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 5.10 | 18.00 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          30.98
  command p90 ms                          32.71
  frames                                  99.00
  tick ms/frame                            1.49
  frame p90 ms                             1.51
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         28.53
  interval max ms (median burst)          28.53
  peek_frame ms/frame                      0.00
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
  dwm_flush ms (outside frames)            4.30
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu8 bd-config bo-config #3 | switch | 30.98 | 32.71 | 99.00 | 1.49 | 1.51 | 3.00 | 5.71 | 5.76 | 28.53 | 28.53 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 4.30 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          65.43
  command p90 ms                          69.60
  frames                                  60.00
  tick ms/frame                            3.61
  frame p90 ms                             4.61
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          7.37
  interval p99 ms                         80.65
  interval max ms (median burst)          80.65
  peek_frame ms/frame                      2.51
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.00
  cloak ms (outside frames)               16.10
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins cpu8 bd-config bo-config #3 | minimize | 65.43 | 69.60 | 60.00 | 3.61 | 4.61 | 4.00 | 5.71 | 7.37 | 80.65 | 80.65 | 2.51 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 10.00 | 16.10 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles stand-ins:morepeeks cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:morepeeks cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.87
  command p90 ms                          22.26
  frames                                  64.00
  tick ms/frame                            0.99
  frame p90 ms                             1.03
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         13.09
  interval max ms (median burst)          13.09
  peek_frame ms/frame                      0.39
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
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.10
  cloak ms (outside frames)                6.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu8 bd-config bo-config #3 | move | 19.87 | 22.26 | 64.00 | 0.99 | 1.03 | 1.00 | 5.71 | 5.75 | 13.09 | 13.09 | 0.39 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 3.10 | 6.30 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          75.80
  command p90 ms                          78.86
  frames                                  59.00
  tick ms/frame                            4.11
  frame p90 ms                             4.44
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.61
  interval p99 ms                         73.19
  interval max ms (median burst)          73.19
  peek_frame ms/frame                      2.51
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.00
  cloak ms (outside frames)               20.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu8 bd-config bo-config #3 | resize | 75.80 | 78.86 | 59.00 | 4.11 | 4.44 | 4.00 | 5.72 | 6.61 | 73.19 | 73.19 | 2.51 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 10.00 | 20.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:morepeeks cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          35.32
  command p90 ms                          36.66
  frames                                  99.00
  tick ms/frame                            1.52
  frame p90 ms                             1.49
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         29.62
  interval max ms (median burst)          29.62
  peek_frame ms/frame                      0.00
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
  dwm_flush ms (outside frames)            5.80
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu8 bd-config bo-config #3 | switch | 35.32 | 36.66 | 99.00 | 1.52 | 1.49 | 3.00 | 5.72 | 5.76 | 29.62 | 29.62 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 5.80 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          67.12
  command p90 ms                          70.58
  frames                                  61.00
  tick ms/frame                            3.62
  frame p90 ms                             4.73
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          7.97
  interval p99 ms                         82.62
  interval max ms (median burst)          82.62
  peek_frame ms/frame                      2.51
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            7.70
  cloak ms (outside frames)               15.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                        -
  input->first frame ms                       -
| stand-ins:morepeeks cpu8 bd-config bo-config #3 | minimize | 67.12 | 70.58 | 61.00 | 3.62 | 4.73 | 5.00 | 5.71 | 7.97 | 82.62 | 82.62 | 2.51 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 7.70 | 15.20 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles main cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          14.86
  command p90 ms                          16.26
  frames                                  63.00
  tick ms/frame                            0.60
  frame p90 ms                             1.10
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                          7.85
  interval max ms (median burst)           7.85
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
  batch_commit ms/frame                    0.33
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)                8.40
  session_begin ms (outside frames)        1.00
  session_overlays ms (outside frames)     2.90
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.25
  input->first frame ms                   15.50
| main cpu8 bd-config bo-config #3 | move | 14.86 | 16.26 | 63.00 | 0.60 | 1.10 | 0.00 | 5.71 | 5.76 | 7.85 | 7.85 | - | - | - | - | - | - | - | - | - | - | - | - | 0.33 | 0.00 | 1.00 | 4.80 | 8.40 | 1.00 | 2.90 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.25 | 15.50 |
label=main cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          31.36
  command p90 ms                          34.26
  frames                                  63.00
  tick ms/frame                            1.94
  frame p90 ms                             4.27
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         26.88
  interval max ms (median burst)          26.88
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
  batch_commit ms/frame                    1.05
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)               18.10
  session_begin ms (outside frames)        2.70
  session_overlays ms (outside frames)     7.10
  session_overlays ms/frame                0.03
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.22
  input->first frame ms                   35.58
| main cpu8 bd-config bo-config #3 | resize | 31.36 | 34.26 | 63.00 | 1.94 | 4.27 | 2.00 | 5.71 | 5.77 | 26.88 | 26.88 | - | - | - | - | - | - | - | - | - | - | - | - | 1.05 | 0.05 | 2.00 | 4.70 | 18.10 | 2.70 | 7.10 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.22 | 35.58 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          21.17
  command p90 ms                          22.32
  frames                                  97.00
  tick ms/frame                            1.86
  frame p90 ms                             2.76
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         55.76
  interval max ms (median burst)          56.18
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
  batch_commit ms/frame                    0.71
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.50
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| main cpu8 bd-config bo-config #3 | switch | 21.17 | 22.32 | 97.00 | 1.86 | 2.76 | 1.00 | 5.71 | 5.76 | 55.76 | 56.18 | - | - | - | - | - | - | - | - | - | - | - | - | 0.71 | 0.02 | 3.00 | 5.50 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=main cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.79
  command p90 ms                          20.36
  frames                                 150.00
  tick ms/frame                            1.62
  frame p90 ms                             4.29
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         34.11
  interval max ms (median burst)          35.11
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
  dwm_flush ms (outside frames)            2.80
  cloak ms (outside frames)               15.20
  session_begin ms (outside frames)        2.30
  session_overlays ms (outside frames)     9.40
  session_overlays ms/frame                0.02
  command ms (WM side)                        -
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     6.81
  input->first frame ms                   44.02
| main cpu8 bd-config bo-config #3 | minimize | 16.79 | 20.36 | 150.00 | 1.62 | 4.29 | 3.00 | 5.71 | 5.77 | 34.11 | 35.11 | - | - | - | - | - | - | - | - | - | - | - | - | 0.77 | 0.03 | 4.00 | 2.80 | 15.20 | 2.30 | 9.40 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.81 | 44.02 |

## move

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 17.42 | 16.48 | — | — | 14.83 |
| cpu0 bd-config bo-config | frame p90 ms | 0.57 | 0.93 | 0.93 | 0.96 | 0.99 |
| cpu0 bd-config bo-config | interval p90 ms | 5.74 | 5.74 | 5.74 | 5.74 | 5.73 |
| cpu0 bd-config bo-config | interval p99 ms | 10.03 | 7.40 | 10.75 | 11.05 | 7.14 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 2.00 | — | — | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | 0.90 | — | — | 0.90 |
| cpu0 bd-config bo-config | restack ms (outside frames) | — | — | — | — | — |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 18.73 | 17.19 | — | — | 15.50 |
| cpu8 bd-config bo-config | frame p90 ms | 0.56 | 1.04 | 1.08 | 1.08 | 1.15 |
| cpu8 bd-config bo-config | interval p90 ms | 5.76 | 5.76 | 5.75 | 5.76 | 5.76 |
| cpu8 bd-config bo-config | interval p99 ms | 12.16 | 8.29 | 13.10 | 12.96 | 8.07 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 2.20 | — | — | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | 1.10 | — | — | 1.00 |
| cpu8 bd-config bo-config | restack ms (outside frames) | — | — | — | — | — |

### move, cpu0 bd-config bo-config

| metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 16.78 | 16.08 | 19.50 | 19.97 | 13.92 |
| command p90 ms | 18.00 | 17.71 | 20.88 | 21.76 | 16.15 |
| frames | 63.00 | 62.00 | 63.00 | 63.00 | 63.00 |
| tick ms/frame | 0.38 | 0.47 | 0.88 | 0.91 | 0.57 |
| frame p90 ms | 0.57 | 0.93 | 0.93 | 0.96 | 0.99 |
| frames >2x budget | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.74 | 5.74 | 5.74 | 5.74 | 5.73 |
| interval p99 ms | 10.03 | 7.40 | 10.75 | 11.05 | 7.14 |
| interval max ms (median burst) | 10.03 | 7.40 | 10.75 | 11.05 | 7.14 |
| peek_frame ms/frame | 0.37 | — | 0.39 | 0.39 | — |
| peek_draw ms/frame | 0.17 | — | — | — | — |
| peek_track ms/frame | 0.05 | — | — | — | — |
| peek_ring ms/frame | 0.01 | — | — | — | — |
| peek_backdrop ms/frame | 0.03 | — | — | — | — |
| peek_draw ms (worst frame) | 0.81 | — | — | — | — |
| pd_alpha ms (worst frame) | 0.44 | — | — | — | — |
| pd_draw ms (worst frame) | 0.51 | — | — | — | — |
| pd_fill ms (worst frame) | — | — | — | — | — |
| fill_source ms (worst frame) | — | — | — | — | — |
| ring_raise ms (worst frame) | — | — | — | — | — |
| restack ms (worst frame) | — | — | — | — | — |
| batch_commit ms/frame | — | 0.27 | — | — | 0.30 |
| ovl_region ms/frame | — | 0.00 | — | — | 0.00 |
| dwm_flush calls (in frames) | — | — | — | — | 1.00 |
| dwm_flush ms (outside frames) | 5.30 | 4.20 | 4.00 | 4.90 | 5.00 |
| cloak ms (outside frames) | 7.80 | 8.30 | 6.90 | 7.90 | 7.90 |
| session_begin ms (outside frames) | — | 0.90 | — | — | 0.90 |
| session_overlays ms (outside frames) | — | 4.40 | 0.00 | 0.00 | 3.10 |
| session_overlays ms/frame | — | 0.02 | 0.00 | 0.00 | 0.02 |
| command ms (WM side) | 16.50 | — | — | — | — |
| peek_new ms (outside frames) | 2.00 | — | — | — | — |
| peek_place ms (outside frames) | 0.90 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — |
| peek_cover ms (outside frames) | 1.10 | — | — | — | — |
| peek_handover ms (outside frames) | 2.60 | — | — | — | — |
| peek_raise ms (outside frames) | 0.30 | — | — | — | — |
| surface ms (outside frames) | 1.00 | — | — | — | — |
| restack ms (outside frames) | — | — | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — | — | — |
| ws_park ms (outside frames) | — | — | — | — | — |
| ws_show ms (outside frames) | — | — | — | — | — |
| ws_start ms (outside frames) | — | — | — | — | — |
| input->anim start ms | 2.89 | 0.29 | — | — | 0.23 |
| input->first frame ms | 17.42 | 16.48 | — | — | 14.83 |

### move, cpu8 bd-config bo-config

| metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 17.52 | 16.56 | 20.96 | 22.09 | 14.86 |
| command p90 ms | 18.61 | 18.47 | 22.16 | 23.37 | 16.26 |
| frames | 63.00 | 63.00 | 63.00 | 63.00 | 63.00 |
| tick ms/frame | 0.43 | 0.53 | 0.98 | 1.02 | 0.61 |
| frame p90 ms | 0.56 | 1.04 | 1.08 | 1.08 | 1.15 |
| frames >2x budget | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.76 | 5.76 | 5.75 | 5.76 | 5.76 |
| interval p99 ms | 12.16 | 8.29 | 13.10 | 12.96 | 8.07 |
| interval max ms (median burst) | 12.16 | 8.29 | 13.10 | 12.96 | 8.07 |
| peek_frame ms/frame | 0.35 | — | 0.38 | 0.39 | — |
| peek_draw ms/frame | 0.17 | — | — | — | — |
| peek_track ms/frame | 0.06 | — | — | — | — |
| peek_ring ms/frame | 0.02 | — | — | — | — |
| peek_backdrop ms/frame | 0.04 | — | — | — | — |
| peek_draw ms (worst frame) | 0.90 | — | — | — | — |
| pd_alpha ms (worst frame) | 0.51 | — | — | — | — |
| pd_draw ms (worst frame) | 0.49 | — | — | — | — |
| pd_fill ms (worst frame) | — | — | — | — | — |
| fill_source ms (worst frame) | — | — | — | — | — |
| ring_raise ms (worst frame) | — | — | — | — | — |
| restack ms (worst frame) | — | — | — | — | — |
| batch_commit ms/frame | — | 0.30 | — | — | 0.34 |
| ovl_region ms/frame | — | 0.00 | — | — | 0.00 |
| dwm_flush calls (in frames) | — | — | — | — | 1.00 |
| dwm_flush ms (outside frames) | 4.60 | 3.70 | 4.50 | 3.80 | 4.80 |
| cloak ms (outside frames) | 7.30 | 8.60 | 7.80 | 7.30 | 8.40 |
| session_begin ms (outside frames) | — | 1.10 | — | — | 1.00 |
| session_overlays ms (outside frames) | — | 4.60 | 0.00 | 0.00 | 2.90 |
| session_overlays ms/frame | — | 0.02 | 0.00 | 0.00 | 0.02 |
| command ms (WM side) | 17.20 | — | — | — | — |
| peek_new ms (outside frames) | 2.20 | — | — | — | — |
| peek_place ms (outside frames) | 1.00 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — |
| peek_cover ms (outside frames) | 1.20 | — | — | — | — |
| peek_handover ms (outside frames) | 3.10 | — | — | — | — |
| peek_raise ms (outside frames) | 0.40 | — | — | — | — |
| surface ms (outside frames) | 1.30 | — | — | — | — |
| restack ms (outside frames) | — | — | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — | — | — |
| ws_park ms (outside frames) | — | — | — | — | — |
| ws_show ms (outside frames) | — | — | — | — | — |
| ws_start ms (outside frames) | — | — | — | — | — |
| input->anim start ms | 3.37 | 0.31 | — | — | 0.25 |
| input->first frame ms | 18.73 | 17.19 | — | — | 15.50 |

## resize

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 63.95 | 44.44 | — | — | 35.25 |
| cpu0 bd-config bo-config | frame p90 ms | 3.31 | 4.19 | 3.54 | 3.44 | 3.98 |
| cpu0 bd-config bo-config | interval p90 ms | 5.77 | 5.78 | 5.85 | 5.81 | 5.77 |
| cpu0 bd-config bo-config | interval p99 ms | 37.06 | 23.13 | 73.11 | 67.67 | 24.85 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 3.50 | — | — | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | 2.20 | — | — | 2.20 |
| cpu0 bd-config bo-config | restack ms (outside frames) | — | — | — | — | — |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 71.44 | 48.88 | — | — | 36.86 |
| cpu8 bd-config bo-config | frame p90 ms | 4.17 | 4.87 | 4.69 | 4.36 | 4.36 |
| cpu8 bd-config bo-config | interval p90 ms | 5.78 | 5.79 | 6.70 | 6.44 | 5.76 |
| cpu8 bd-config bo-config | interval p99 ms | 38.43 | 17.01 | 60.47 | 64.60 | 26.88 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 4.40 | — | — | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | 2.60 | — | — | 2.70 |
| cpu8 bd-config bo-config | restack ms (outside frames) | — | — | — | — | — |

### resize, cpu0 bd-config bo-config

| metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 47.76 | 40.18 | 62.63 | 65.03 | 29.88 |
| command p90 ms | 54.06 | 43.25 | 68.26 | 73.60 | 32.18 |
| frames | 62.00 | 62.00 | 61.00 | 60.00 | 63.00 |
| tick ms/frame | 2.28 | 1.86 | 3.82 | 3.79 | 1.81 |
| frame p90 ms | 3.31 | 4.19 | 3.54 | 3.44 | 3.98 |
| frames >2x budget | 3.00 | 1.00 | 4.00 | 4.00 | 2.00 |
| interval p50 ms | 5.72 | 5.71 | 5.72 | 5.71 | 5.72 |
| interval p90 ms | 5.77 | 5.78 | 5.85 | 5.81 | 5.77 |
| interval p99 ms | 37.06 | 23.13 | 73.11 | 67.67 | 24.85 |
| interval max ms (median burst) | 37.06 | 23.13 | 73.11 | 67.67 | 24.85 |
| peek_frame ms/frame | 1.84 | — | 2.34 | 2.27 | — |
| peek_draw ms/frame | 1.20 | — | — | — | — |
| peek_track ms/frame | 0.29 | — | — | — | — |
| peek_ring ms/frame | 0.14 | — | — | — | — |
| peek_backdrop ms/frame | 0.12 | — | — | — | — |
| peek_draw ms (worst frame) | 13.73 | — | — | — | — |
| pd_alpha ms (worst frame) | 1.02 | — | — | — | — |
| pd_draw ms (worst frame) | 12.43 | — | — | — | — |
| pd_fill ms (worst frame) | 8.85 | — | — | — | — |
| fill_source ms (worst frame) | 1.83 | — | — | — | — |
| ring_raise ms (worst frame) | 5.61 | — | — | — | — |
| restack ms (worst frame) | 2.71 | — | — | — | — |
| batch_commit ms/frame | — | 0.93 | — | — | 0.91 |
| ovl_region ms/frame | 0.04 | 0.04 | 0.04 | 0.04 | 0.05 |
| dwm_flush calls (in frames) | 2.00 | — | 2.00 | 2.00 | 2.00 |
| dwm_flush ms (outside frames) | 8.90 | 3.90 | 8.90 | 7.50 | 5.10 |
| cloak ms (outside frames) | 17.00 | 20.10 | 17.40 | 17.80 | 17.90 |
| session_begin ms (outside frames) | — | 2.20 | — | — | 2.20 |
| session_overlays ms (outside frames) | — | 15.10 | 0.00 | 0.00 | 6.80 |
| session_overlays ms/frame | — | 0.07 | 0.00 | 0.00 | 0.04 |
| command ms (WM side) | 47.40 | — | — | — | — |
| peek_new ms (outside frames) | 3.50 | — | — | — | — |
| peek_place ms (outside frames) | 1.90 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — |
| peek_cover ms (outside frames) | 11.00 | — | — | — | — |
| peek_handover ms (outside frames) | 12.00 | — | — | — | — |
| peek_raise ms (outside frames) | 0.70 | — | — | — | — |
| surface ms (outside frames) | 1.70 | — | — | — | — |
| restack ms (outside frames) | — | — | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — | — | — |
| ws_park ms (outside frames) | — | — | — | — | — |
| ws_show ms (outside frames) | — | — | — | — | — |
| ws_start ms (outside frames) | — | — | — | — | — |
| input->anim start ms | 3.32 | 0.28 | — | — | 0.22 |
| input->first frame ms | 63.95 | 44.44 | — | — | 35.25 |

### resize, cpu8 bd-config bo-config

| metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 52.61 | 44.16 | 69.38 | 72.26 | 32.14 |
| command p90 ms | 57.40 | 47.38 | 78.53 | 79.48 | 34.26 |
| frames | 61.00 | 62.00 | 59.00 | 59.00 | 63.00 |
| tick ms/frame | 2.58 | 1.86 | 4.13 | 4.02 | 1.94 |
| frame p90 ms | 4.17 | 4.87 | 4.69 | 4.36 | 4.36 |
| frames >2x budget | 3.00 | 1.00 | 4.00 | 4.00 | 2.00 |
| interval p50 ms | 5.72 | 5.71 | 5.71 | 5.72 | 5.71 |
| interval p90 ms | 5.78 | 5.79 | 6.70 | 6.44 | 5.76 |
| interval p99 ms | 38.43 | 17.01 | 60.47 | 64.60 | 26.88 |
| interval max ms (median burst) | 38.43 | 17.01 | 60.47 | 64.60 | 26.88 |
| peek_frame ms/frame | 2.12 | — | 2.60 | 2.63 | — |
| peek_draw ms/frame | 1.39 | — | — | — | — |
| peek_track ms/frame | 0.31 | — | — | — | — |
| peek_ring ms/frame | 0.16 | — | — | — | — |
| peek_backdrop ms/frame | 0.13 | — | — | — | — |
| peek_draw ms (worst frame) | 17.04 | — | — | — | — |
| pd_alpha ms (worst frame) | 1.19 | — | — | — | — |
| pd_draw ms (worst frame) | 15.67 | — | — | — | — |
| pd_fill ms (worst frame) | 11.65 | — | — | — | — |
| fill_source ms (worst frame) | 2.33 | — | — | — | — |
| ring_raise ms (worst frame) | 5.86 | — | — | — | — |
| restack ms (worst frame) | 0.00 | — | — | — | — |
| batch_commit ms/frame | — | 1.09 | — | — | 1.05 |
| ovl_region ms/frame | 0.04 | 0.03 | 0.04 | 0.04 | 0.05 |
| dwm_flush calls (in frames) | 2.00 | — | 2.00 | 2.00 | 2.00 |
| dwm_flush ms (outside frames) | 8.80 | 3.90 | 5.10 | 4.80 | 4.50 |
| cloak ms (outside frames) | 18.60 | 22.20 | 18.00 | 19.40 | 19.00 |
| session_begin ms (outside frames) | — | 2.60 | — | — | 2.70 |
| session_overlays ms (outside frames) | — | 17.30 | 0.00 | 0.00 | 7.20 |
| session_overlays ms/frame | — | 0.06 | 0.00 | 0.00 | 0.03 |
| command ms (WM side) | 50.60 | — | — | — | — |
| peek_new ms (outside frames) | 4.40 | — | — | — | — |
| peek_place ms (outside frames) | 2.00 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — |
| peek_cover ms (outside frames) | 11.80 | — | — | — | — |
| peek_handover ms (outside frames) | 11.50 | — | — | — | — |
| peek_raise ms (outside frames) | 0.70 | — | — | — | — |
| surface ms (outside frames) | 2.00 | — | — | — | — |
| restack ms (outside frames) | — | — | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — | — | — |
| ws_park ms (outside frames) | — | — | — | — | — |
| ws_show ms (outside frames) | — | — | — | — | — |
| ws_start ms (outside frames) | — | — | — | — | — |
| input->anim start ms | 3.73 | 0.31 | — | — | 0.23 |
| input->first frame ms | 71.44 | 48.88 | — | — | 36.86 |

## switch

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | — | 0.00 | — | — | 0.00 |
| cpu0 bd-config bo-config | frame p90 ms | 1.06 | 1.19 | 1.23 | 1.21 | 2.41 |
| cpu0 bd-config bo-config | interval p90 ms | 5.76 | 5.76 | 5.76 | 5.76 | 5.73 |
| cpu0 bd-config bo-config | interval p99 ms | 19.68 | 21.67 | 23.57 | 23.58 | 50.78 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | — | — | — | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | — | — | — | 0.00 |
| cpu0 bd-config bo-config | restack ms (outside frames) | 0.00 | — | — | — | — |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | — | 0.00 | — | — | 0.00 |
| cpu8 bd-config bo-config | frame p90 ms | 1.22 | 1.47 | 1.50 | 1.49 | 2.76 |
| cpu8 bd-config bo-config | interval p90 ms | 5.76 | 5.76 | 5.76 | 5.76 | 5.76 |
| cpu8 bd-config bo-config | interval p99 ms | 22.66 | 24.39 | 26.89 | 26.59 | 56.38 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | — | — | — | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | 0.00 | — | — | 0.00 |
| cpu8 bd-config bo-config | restack ms (outside frames) | 0.00 | — | — | — | — |

### switch, cpu0 bd-config bo-config

| metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 25.57 | 29.72 | 30.27 | 29.99 | 20.30 |
| command p90 ms | 28.03 | 30.78 | 31.26 | 31.35 | 21.01 |
| frames | 98.00 | 99.00 | 99.00 | 100.00 | 97.00 |
| tick ms/frame | 1.05 | 1.20 | 1.30 | 1.32 | 1.65 |
| frame p90 ms | 1.06 | 1.19 | 1.23 | 1.21 | 2.41 |
| frames >2x budget | 2.00 | 3.00 | 3.00 | 2.00 | 1.00 |
| interval p50 ms | 5.72 | 5.71 | 5.71 | 5.72 | 5.71 |
| interval p90 ms | 5.76 | 5.76 | 5.76 | 5.76 | 5.73 |
| interval p99 ms | 19.68 | 21.67 | 23.57 | 23.58 | 50.78 |
| interval max ms (median burst) | 19.68 | 21.81 | 23.57 | 23.58 | 51.08 |
| peek_frame ms/frame | 0.00 | — | 0.00 | 0.00 | — |
| peek_draw ms/frame | 0.00 | — | — | — | — |
| peek_track ms/frame | 0.00 | — | — | — | — |
| peek_ring ms/frame | 0.00 | — | — | — | — |
| peek_backdrop ms/frame | 0.00 | — | — | — | — |
| peek_draw ms (worst frame) | 0.00 | — | — | — | — |
| pd_alpha ms (worst frame) | 0.00 | — | — | — | — |
| pd_draw ms (worst frame) | 0.00 | — | — | — | — |
| pd_fill ms (worst frame) | 0.00 | — | — | — | — |
| fill_source ms (worst frame) | 0.00 | — | — | — | — |
| ring_raise ms (worst frame) | 0.00 | — | — | — | — |
| restack ms (worst frame) | — | — | — | — | — |
| batch_commit ms/frame | — | 0.00 | — | — | 0.61 |
| ovl_region ms/frame | 0.02 | 0.03 | 0.03 | 0.03 | 0.01 |
| dwm_flush calls (in frames) | — | — | — | — | 3.00 |
| dwm_flush ms (outside frames) | 5.00 | 5.70 | 6.60 | 6.20 | 5.80 |
| cloak ms (outside frames) | — | — | 0.00 | — | 0.00 |
| session_begin ms (outside frames) | — | — | — | — | 0.00 |
| session_overlays ms (outside frames) | — | 0.00 | 0.00 | 0.00 | 0.00 |
| session_overlays ms/frame | — | 0.00 | 0.00 | 0.00 | 0.00 |
| command ms (WM side) | 25.20 | — | — | — | — |
| peek_new ms (outside frames) | — | — | — | — | — |
| peek_place ms (outside frames) | — | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — |
| peek_cover ms (outside frames) | — | — | — | — | — |
| peek_handover ms (outside frames) | — | — | — | — | — |
| peek_raise ms (outside frames) | 0.00 | — | — | — | — |
| surface ms (outside frames) | 0.50 | — | — | — | — |
| restack ms (outside frames) | 0.00 | — | — | — | — |
| ws_surrogates ms (outside frames) | 10.50 | — | — | — | — |
| ws_park ms (outside frames) | 1.90 | — | — | — | — |
| ws_show ms (outside frames) | 7.40 | — | — | — | — |
| ws_start ms (outside frames) | 0.40 | — | — | — | — |
| input->anim start ms | — | 0.00 | — | — | 0.00 |
| input->first frame ms | — | 0.00 | — | — | 0.00 |

### switch, cpu8 bd-config bo-config

| metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 28.76 | 31.46 | 32.37 | 32.51 | 21.24 |
| command p90 ms | 30.56 | 33.84 | 33.64 | 35.56 | 22.32 |
| frames | 99.00 | 99.00 | 99.00 | 99.00 | 97.00 |
| tick ms/frame | 1.17 | 1.37 | 1.47 | 1.48 | 1.87 |
| frame p90 ms | 1.22 | 1.47 | 1.50 | 1.49 | 2.76 |
| frames >2x budget | 3.00 | 3.00 | 3.00 | 3.00 | 1.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.76 | 5.76 | 5.76 | 5.76 | 5.76 |
| interval p99 ms | 22.66 | 24.39 | 26.89 | 26.59 | 56.38 |
| interval max ms (median burst) | 22.66 | 24.39 | 26.89 | 26.59 | 56.78 |
| peek_frame ms/frame | 0.00 | — | 0.00 | 0.00 | — |
| peek_draw ms/frame | 0.00 | — | — | — | — |
| peek_track ms/frame | 0.00 | — | — | — | — |
| peek_ring ms/frame | 0.00 | — | — | — | — |
| peek_backdrop ms/frame | 0.00 | — | — | — | — |
| peek_draw ms (worst frame) | 0.00 | — | — | — | — |
| pd_alpha ms (worst frame) | 0.00 | — | — | — | — |
| pd_draw ms (worst frame) | 0.00 | — | — | — | — |
| pd_fill ms (worst frame) | 0.00 | — | — | — | — |
| fill_source ms (worst frame) | 0.00 | — | — | — | — |
| ring_raise ms (worst frame) | 0.00 | — | — | — | — |
| restack ms (worst frame) | — | — | — | — | — |
| batch_commit ms/frame | — | 0.00 | — | — | 0.71 |
| ovl_region ms/frame | 0.04 | 0.03 | 0.03 | 0.03 | 0.02 |
| dwm_flush calls (in frames) | — | — | — | — | 3.00 |
| dwm_flush ms (outside frames) | 5.50 | 5.20 | 5.20 | 5.60 | 5.50 |
| cloak ms (outside frames) | — | 0.00 | — | 0.00 | 0.00 |
| session_begin ms (outside frames) | — | 0.00 | — | — | 0.00 |
| session_overlays ms (outside frames) | — | 0.00 | 0.00 | 0.00 | 0.00 |
| session_overlays ms/frame | — | 0.00 | 0.00 | 0.00 | 0.00 |
| command ms (WM side) | 28.40 | — | — | — | — |
| peek_new ms (outside frames) | — | — | — | — | — |
| peek_place ms (outside frames) | — | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — |
| peek_cover ms (outside frames) | — | — | — | — | — |
| peek_handover ms (outside frames) | — | — | — | — | — |
| peek_raise ms (outside frames) | 0.00 | — | — | — | — |
| surface ms (outside frames) | 0.60 | — | — | — | — |
| restack ms (outside frames) | 0.00 | — | — | — | — |
| ws_surrogates ms (outside frames) | 12.00 | — | — | — | — |
| ws_park ms (outside frames) | 2.10 | — | — | — | — |
| ws_show ms (outside frames) | 8.10 | — | — | — | — |
| ws_start ms (outside frames) | 0.40 | — | — | — | — |
| input->anim start ms | — | 0.00 | — | — | 0.00 |
| input->first frame ms | — | 0.00 | — | — | 0.00 |

## minimize

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 69.48 | 52.64 | — | — | 43.25 |
| cpu0 bd-config bo-config | frame p90 ms | 3.49 | 2.74 | 3.81 | 4.10 | 3.83 |
| cpu0 bd-config bo-config | interval p90 ms | 5.77 | 5.76 | 7.22 | 7.12 | 5.74 |
| cpu0 bd-config bo-config | interval p99 ms | 58.89 | 45.72 | 72.42 | 73.43 | 34.50 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 12.90 | — | — | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | 2.20 | — | — | 2.30 |
| cpu0 bd-config bo-config | restack ms (outside frames) | 9.20 | — | — | — | — |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 81.39 | 58.33 | — | — | 46.38 |
| cpu8 bd-config bo-config | frame p90 ms | 4.15 | 3.28 | 4.96 | 4.82 | 4.30 |
| cpu8 bd-config bo-config | interval p90 ms | 5.78 | 5.77 | 7.47 | 7.97 | 5.77 |
| cpu8 bd-config bo-config | interval p99 ms | 53.57 | 48.02 | 81.04 | 82.24 | 35.54 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 8.40 | — | — | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | 2.50 | — | — | 2.30 |
| cpu8 bd-config bo-config | restack ms (outside frames) | 9.60 | — | — | — | — |

### minimize, cpu0 bd-config bo-config

| metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 50.47 | 23.21 | 57.96 | 58.53 | 16.27 |
| command p90 ms | 57.16 | 25.10 | 64.05 | 63.94 | 18.07 |
| frames | 61.00 | 144.00 | 60.00 | 60.00 | 150.00 |
| tick ms/frame | 2.33 | 1.11 | 3.14 | 3.14 | 1.37 |
| frame p90 ms | 3.49 | 2.74 | 3.81 | 4.10 | 3.83 |
| frames >2x budget | 3.00 | 0.00 | 4.00 | 4.00 | 2.00 |
| interval p50 ms | 5.72 | 5.71 | 5.72 | 5.72 | 5.71 |
| interval p90 ms | 5.77 | 5.76 | 7.22 | 7.12 | 5.74 |
| interval p99 ms | 58.89 | 45.72 | 72.42 | 73.43 | 34.50 |
| interval max ms (median burst) | 58.89 | 45.72 | 72.42 | 73.43 | 34.50 |
| peek_frame ms/frame | 1.88 | — | 2.18 | 2.14 | — |
| peek_draw ms/frame | 1.21 | — | — | — | — |
| peek_track ms/frame | 0.35 | — | — | — | — |
| peek_ring ms/frame | 0.13 | — | — | — | — |
| peek_backdrop ms/frame | 0.19 | — | — | — | — |
| peek_draw ms (worst frame) | 16.51 | — | — | — | — |
| pd_alpha ms (worst frame) | 0.83 | — | — | — | — |
| pd_draw ms (worst frame) | 15.63 | — | — | — | — |
| pd_fill ms (worst frame) | 12.54 | — | — | — | — |
| fill_source ms (worst frame) | 3.72 | — | — | — | — |
| ring_raise ms (worst frame) | 5.71 | — | — | — | — |
| restack ms (worst frame) | 2.62 | — | — | — | — |
| batch_commit ms/frame | — | 0.59 | — | — | 0.66 |
| ovl_region ms/frame | 0.03 | 0.02 | 0.02 | 0.02 | 0.03 |
| dwm_flush calls (in frames) | 2.00 | 1.00 | 2.00 | 2.00 | 4.00 |
| dwm_flush ms (outside frames) | 11.00 | 3.80 | 7.30 | 9.90 | 4.50 |
| cloak ms (outside frames) | 16.40 | 22.10 | 13.80 | 15.80 | 17.40 |
| session_begin ms (outside frames) | — | 2.20 | — | — | 2.30 |
| session_overlays ms (outside frames) | — | 15.70 | 0.00 | 0.00 | 6.80 |
| session_overlays ms/frame | — | 0.03 | 0.00 | 0.00 | 0.02 |
| command ms (WM side) | 50.20 | — | — | — | — |
| peek_new ms (outside frames) | 12.90 | — | — | — | — |
| peek_place ms (outside frames) | 1.70 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — |
| peek_cover ms (outside frames) | 22.50 | — | — | — | — |
| peek_handover ms (outside frames) | 13.60 | — | — | — | — |
| peek_raise ms (outside frames) | 0.20 | — | — | — | — |
| surface ms (outside frames) | 4.60 | — | — | — | — |
| restack ms (outside frames) | 9.20 | — | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — | — | — |
| ws_park ms (outside frames) | — | — | — | — | — |
| ws_show ms (outside frames) | — | — | — | — | — |
| ws_start ms (outside frames) | — | — | — | — | — |
| input->anim start ms | 9.68 | 5.28 | — | — | 6.17 |
| input->first frame ms | 69.48 | 52.64 | — | — | 43.25 |

### minimize, cpu8 bd-config bo-config

| metric | this | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 58.82 | 26.69 | 66.62 | 65.70 | 18.60 |
| command p90 ms | 62.89 | 29.89 | 70.97 | 71.38 | 20.36 |
| frames | 61.00 | 143.00 | 60.00 | 60.00 | 150.00 |
| tick ms/frame | 2.46 | 1.32 | 3.65 | 3.56 | 1.63 |
| frame p90 ms | 4.15 | 3.28 | 4.96 | 4.82 | 4.30 |
| frames >2x budget | 3.00 | 0.00 | 4.00 | 4.00 | 2.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.78 | 5.77 | 7.47 | 7.97 | 5.77 |
| interval p99 ms | 53.57 | 48.02 | 81.04 | 82.24 | 35.54 |
| interval max ms (median burst) | 53.57 | 48.02 | 81.04 | 82.24 | 35.54 |
| peek_frame ms/frame | 2.10 | — | 2.51 | 2.47 | — |
| peek_draw ms/frame | 1.33 | — | — | — | — |
| peek_track ms/frame | 0.38 | — | — | — | — |
| peek_ring ms/frame | 0.15 | — | — | — | — |
| peek_backdrop ms/frame | 0.21 | — | — | — | — |
| peek_draw ms (worst frame) | 20.73 | — | — | — | — |
| pd_alpha ms (worst frame) | 1.00 | — | — | — | — |
| pd_draw ms (worst frame) | 19.72 | — | — | — | — |
| pd_fill ms (worst frame) | 15.97 | — | — | — | — |
| fill_source ms (worst frame) | 5.32 | — | — | — | — |
| ring_raise ms (worst frame) | 5.81 | — | — | — | — |
| restack ms (worst frame) | 3.05 | — | — | — | — |
| batch_commit ms/frame | — | 0.67 | — | — | 0.77 |
| ovl_region ms/frame | 0.03 | 0.02 | 0.03 | 0.03 | 0.03 |
| dwm_flush calls (in frames) | 2.00 | 1.00 | 2.00 | 2.00 | 4.00 |
| dwm_flush ms (outside frames) | 10.10 | 4.60 | 10.00 | 8.90 | 4.80 |
| cloak ms (outside frames) | 17.40 | 21.80 | 17.60 | 15.20 | 18.20 |
| session_begin ms (outside frames) | — | 2.50 | — | — | 2.30 |
| session_overlays ms (outside frames) | — | 18.20 | 0.00 | 0.00 | 8.70 |
| session_overlays ms/frame | — | 0.04 | 0.00 | 0.00 | 0.02 |
| command ms (WM side) | 58.50 | — | — | — | — |
| peek_new ms (outside frames) | 8.40 | — | — | — | — |
| peek_place ms (outside frames) | 1.80 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — |
| peek_cover ms (outside frames) | 24.60 | — | — | — | — |
| peek_handover ms (outside frames) | 14.30 | — | — | — | — |
| peek_raise ms (outside frames) | 0.30 | — | — | — | — |
| surface ms (outside frames) | 3.00 | — | — | — | — |
| restack ms (outside frames) | 9.60 | — | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — | — | — |
| ws_park ms (outside frames) | — | — | — | — | — |
| ws_show ms (outside frames) | — | — | — | — | — |
| ws_start ms (outside frames) | — | — | — | — | — |
| input->anim start ms | 10.68 | 6.08 | — | — | 6.96 |
| input->first frame ms | 81.39 | 58.33 | — | — | 46.38 |
