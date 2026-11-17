Branch comparison, 2026-11-17T12:41:19: this = a3d1dff3, against origin/tryout/stand-ins (65a60de), origin/main (cfd3cef); builds this, stand-ins:nopeeks (GLAZEWM_ISOLATE=nopeeks), main. Target chrome, 3 round(s) of 10 bursts per scenario, build and setting, IPC load 0, CPU load 0,8 thread(s) of 16, with DWM timing, tile order Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot, launched none (0 window(s)).
tiles this cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-this-move-mid.png (mid at 64 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-this-move-rest.png
label=this cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.15
  command p90 ms                          18.34
  frames                                  63.00
  tick ms/frame                            0.39
  frame p90 ms                             0.55
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         10.81
  interval max ms (median burst)          10.81
  peek_frame ms/frame                      0.38
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.69
  pd_alpha ms (worst frame)                0.25
  pd_draw ms (worst frame)                 0.56
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.20
  cloak ms (outside frames)                7.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.90
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.10
  peek_handover ms (outside frames)        3.00
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.00
  restack ms (outside frames)              0.80
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.44
  input->first frame ms                   18.71
  dwm missed vblanks/burst                13.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.44
| this cpu0 bd-config bo-config #1 | move | 18.15 | 18.34 | 63.00 | 0.39 | 0.55 | 0.00 | 5.71 | 5.75 | 10.81 | 10.81 | 0.38 | 0.16 | 0.05 | 0.01 | 0.03 | 0.69 | 0.25 | 0.56 | - | - | - | - | - | - | - | 5.20 | 7.90 | - | - | - | 17.90 | 2.10 | 1.10 | - | - | - | 1.10 | 3.00 | 0.30 | 1.00 | 0.80 | - | - | - | - | 3.44 | 18.71 | 13.00 | 11.43 | 8.44 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-this-resize-mid.png (mid at 67 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-this-resize-rest.png
label=this cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          52.78
  command p90 ms                          56.07
  frames                                  62.00
  tick ms/frame                            2.28
  frame p90 ms                             3.27
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         36.52
  interval max ms (median burst)          36.52
  peek_frame ms/frame                      1.77
  peek_draw ms/frame                       1.22
  peek_track ms/frame                      0.28
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.13
  peek_draw ms (worst frame)              14.80
  pd_alpha ms (worst frame)                1.04
  pd_draw ms (worst frame)                13.68
  pd_fill ms (worst frame)                10.46
  fill_source ms (worst frame)             2.17
  ring_raise ms (worst frame)              4.76
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.50
  cloak ms (outside frames)               21.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    52.50
  peek_new ms (outside frames)             3.30
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          11.10
  peek_handover ms (outside frames)       14.10
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.27
  input->first frame ms                   67.99
  dwm missed vblanks/burst                22.00
  dwm display interval p90 ms             17.14
  dwm gpu busy p90 ms                      1.45
| this cpu0 bd-config bo-config #1 | resize | 52.78 | 56.07 | 62.00 | 2.28 | 3.27 | 3.00 | 5.71 | 5.78 | 36.52 | 36.52 | 1.77 | 1.22 | 0.28 | 0.13 | 0.13 | 14.80 | 1.04 | 13.68 | 10.46 | 2.17 | 4.76 | - | - | 0.03 | 2.00 | 9.50 | 21.20 | - | - | - | 52.50 | 3.30 | 2.00 | - | - | - | 11.10 | 14.10 | 0.10 | 0.70 | - | - | - | - | - | 3.27 | 67.99 | 22.00 | 17.14 | 1.45 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-this-switch-mid.png (mid at 72 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-this-switch-rest.png
label=this cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.46
  command p90 ms                          33.72
  frames                                  99.00
  tick ms/frame                            1.12
  frame p90 ms                             1.09
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         21.66
  interval max ms (median burst)          21.66
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
  dwm_flush ms (outside frames)            6.00
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    30.20
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
  ws_surrogates ms (outside frames)        9.90
  ws_park ms (outside frames)              2.00
  ws_show ms (outside frames)              8.60
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                18.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      5.31
| this cpu0 bd-config bo-config #1 | switch | 30.46 | 33.72 | 99.00 | 1.12 | 1.09 | 3.00 | 5.71 | 5.76 | 21.66 | 21.66 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 6.00 | - | - | - | - | 30.20 | - | - | - | - | - | - | - | 0.00 | 0.60 | - | 9.90 | 2.00 | 8.60 | 0.40 | - | - | 18.00 | 11.43 | 5.31 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-this-minimize-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-this-minimize-rest.png
label=this cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          49.59
  command p90 ms                          50.94
  frames                                  62.00
  tick ms/frame                            2.22
  frame p90 ms                             3.69
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         64.55
  interval max ms (median burst)          64.55
  peek_frame ms/frame                      1.79
  peek_draw ms/frame                       1.10
  peek_track ms/frame                      0.36
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.20
  peek_draw ms (worst frame)              11.36
  pd_alpha ms (worst frame)                0.90
  pd_draw ms (worst frame)                10.24
  pd_fill ms (worst frame)                 7.00
  fill_source ms (worst frame)             2.05
  ring_raise ms (worst frame)              3.30
  restack ms (worst frame)                 2.80
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           13.40
  cloak ms (outside frames)               19.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    49.30
  peek_new ms (outside frames)             9.40
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          17.80
  peek_handover ms (outside frames)       14.90
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              3.40
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.01
  input->first frame ms                   63.83
  dwm missed vblanks/burst                29.00
  dwm display interval p90 ms             28.57
  dwm gpu busy p90 ms                      7.83
| this cpu0 bd-config bo-config #1 | minimize | 49.59 | 50.94 | 62.00 | 2.22 | 3.69 | 3.00 | 5.72 | 5.78 | 64.55 | 64.55 | 1.79 | 1.10 | 0.36 | 0.14 | 0.20 | 11.36 | 0.90 | 10.24 | 7.00 | 2.05 | 3.30 | 2.80 | - | 0.02 | 2.00 | 13.40 | 19.70 | - | - | - | 49.30 | 9.40 | 1.80 | - | - | - | 17.80 | 14.90 | 0.40 | 3.40 | - | - | - | - | - | 8.01 | 63.83 | 29.00 | 28.57 | 7.83 |
tiles stand-ins:nopeeks cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-move-mid.png (mid at 74 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-move-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          14.50
  command p90 ms                          15.37
  frames                                  62.00
  tick ms/frame                            0.63
  frame p90 ms                             1.31
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          7.92
  interval max ms (median burst)           7.92
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
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)                9.00
  session_begin ms (outside frames)        0.60
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
  input->anim start ms                     0.27
  input->first frame ms                   15.16
  dwm missed vblanks/burst                10.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.64
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | move | 14.50 | 15.37 | 62.00 | 0.63 | 1.31 | 0.00 | 5.71 | 5.74 | 7.92 | 7.92 | - | - | - | - | - | - | - | - | - | - | - | - | 0.41 | 0.00 | - | 4.70 | 9.00 | 0.60 | 3.30 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.27 | 15.16 | 10.00 | 11.43 | 8.64 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-resize-mid.png (mid at 67 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-resize-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          42.31
  command p90 ms                          43.14
  frames                                  62.00
  tick ms/frame                            1.65
  frame p90 ms                             4.04
  frames >2x budget                        1.00
  interval p50 ms                          5.72
  interval p90 ms                          5.80
  interval p99 ms                         19.28
  interval max ms (median burst)          19.28
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
  batch_commit ms/frame                    0.97
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.30
  cloak ms (outside frames)               20.30
  session_begin ms (outside frames)        1.80
  session_overlays ms (outside frames)    16.00
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
  input->anim start ms                     0.28
  input->first frame ms                   45.30
  dwm missed vblanks/burst                16.00
  dwm display interval p90 ms             11.46
  dwm gpu busy p90 ms                      3.71
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | resize | 42.31 | 43.14 | 62.00 | 1.65 | 4.04 | 1.00 | 5.72 | 5.80 | 19.28 | 19.28 | - | - | - | - | - | - | - | - | - | - | - | - | 0.97 | 0.03 | - | 3.30 | 20.30 | 1.80 | 16.00 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.28 | 45.30 | 16.00 | 11.46 | 3.71 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-switch-mid.png (mid at 81 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-switch-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.98
  command p90 ms                          31.69
  frames                                  98.00
  tick ms/frame                            1.25
  frame p90 ms                             1.24
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         24.29
  interval max ms (median burst)          24.29
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
  dwm missed vblanks/burst                16.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      2.22
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | switch | 30.98 | 31.69 | 98.00 | 1.25 | 1.24 | 3.00 | 5.71 | 5.76 | 24.29 | 24.29 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | - | 6.40 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 16.00 | 11.43 | 2.22 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-minimize-mid.png (mid at 72 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-minimize-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          25.22
  command p90 ms                          28.24
  frames                                 143.00
  tick ms/frame                            1.17
  frame p90 ms                             3.38
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         55.43
  interval max ms (median burst)          55.43
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
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.00
  cloak ms (outside frames)               26.30
  session_begin ms (outside frames)        5.70
  session_overlays ms (outside frames)    17.20
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
  input->anim start ms                     5.38
  input->first frame ms                   64.47
  dwm missed vblanks/burst                17.00
  dwm display interval p90 ms             11.46
  dwm gpu busy p90 ms                      6.53
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | minimize | 25.22 | 28.24 | 143.00 | 1.17 | 3.38 | 0.00 | 5.71 | 5.76 | 55.43 | 55.43 | - | - | - | - | - | - | - | - | - | - | - | - | 0.64 | 0.02 | 1.00 | 4.00 | 26.30 | 5.70 | 17.20 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 5.38 | 64.47 | 17.00 | 11.46 | 6.53 |
tiles main cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-main-default-move-mid.png (mid at 71 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-main-default-move-rest.png
label=main cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          12.96
  command p90 ms                          13.38
  frames                                  63.00
  tick ms/frame                            0.71
  frame p90 ms                             1.36
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          7.74
  interval max ms (median burst)           7.74
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
  batch_commit ms/frame                    0.43
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)                7.80
  session_begin ms (outside frames)        0.90
  session_overlays ms (outside frames)     1.90
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
  input->first frame ms                   13.59
  dwm missed vblanks/burst                10.00
  dwm display interval p90 ms             17.12
  dwm gpu busy p90 ms                      7.27
| main cpu0 bd-config bo-config #1 | move | 12.96 | 13.38 | 63.00 | 0.71 | 1.36 | 0.00 | 5.71 | 5.74 | 7.74 | 7.74 | - | - | - | - | - | - | - | - | - | - | - | - | 0.43 | 0.00 | 1.00 | 4.90 | 7.80 | 0.90 | 1.90 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.23 | 13.59 | 10.00 | 17.12 | 7.27 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-main-default-resize-mid.png (mid at 61 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-main-default-resize-rest.png
label=main cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.12
  command p90 ms                          31.94
  frames                                  63.00
  tick ms/frame                            2.30
  frame p90 ms                             4.42
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         30.67
  interval max ms (median burst)          30.67
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
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)               18.00
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
  input->first frame ms                   34.53
  dwm missed vblanks/burst                 7.00
  dwm display interval p90 ms              5.74
  dwm gpu busy p90 ms                      1.38
| main cpu0 bd-config bo-config #1 | resize | 29.12 | 31.94 | 63.00 | 2.30 | 4.42 | 2.00 | 5.71 | 5.78 | 30.67 | 30.67 | - | - | - | - | - | - | - | - | - | - | - | - | 1.01 | 0.05 | 2.00 | 4.70 | 18.00 | 2.20 | 6.80 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.22 | 34.53 | 7.00 | 5.74 | 1.38 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-main-default-switch-mid.png (mid at 66 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-main-default-switch-rest.png
label=main cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.24
  command p90 ms                          21.35
  frames                                  97.00
  tick ms/frame                            1.73
  frame p90 ms                             2.64
  frames >2x budget                        1.00
  interval p50 ms                          5.72
  interval p90 ms                          5.74
  interval p99 ms                         55.41
  interval max ms (median burst)          55.41
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
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                 9.00
  dwm display interval p90 ms              5.77
  dwm gpu busy p90 ms                      3.80
| main cpu0 bd-config bo-config #1 | switch | 20.24 | 21.35 | 97.00 | 1.73 | 2.64 | 1.00 | 5.72 | 5.74 | 55.41 | 55.41 | - | - | - | - | - | - | - | - | - | - | - | - | 0.65 | 0.01 | 3.00 | 5.70 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 9.00 | 5.77 | 3.80 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-main-default-minimize-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu0-bdconfigboconfig-main-default-minimize-rest.png
label=main cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          19.71
  command p90 ms                          20.85
  frames                                 150.00
  tick ms/frame                            1.60
  frame p90 ms                             3.52
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
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
  batch_commit ms/frame                    0.82
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            5.50
  cloak ms (outside frames)               19.90
  session_begin ms (outside frames)        4.30
  session_overlays ms (outside frames)     7.10
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
  input->anim start ms                     6.43
  input->first frame ms                   52.00
  dwm missed vblanks/burst                14.00
  dwm display interval p90 ms             11.46
  dwm gpu busy p90 ms                      6.58
| main cpu0 bd-config bo-config #1 | minimize | 19.71 | 20.85 | 150.00 | 1.60 | 3.52 | 3.00 | 5.71 | 5.76 | 36.88 | 36.88 | - | - | - | - | - | - | - | - | - | - | - | - | 0.82 | 0.03 | 4.00 | 5.50 | 19.90 | 4.30 | 7.10 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.43 | 52.00 | 14.00 | 11.46 | 6.58 |
tiles stand-ins:nopeeks cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          15.53
  command p90 ms                          16.39
  frames                                  62.00
  tick ms/frame                            0.67
  frame p90 ms                             1.45
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                          8.28
  interval max ms (median burst)           8.28
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
  batch_commit ms/frame                    0.43
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)                9.20
  session_begin ms (outside frames)        0.70
  session_overlays ms (outside frames)     3.40
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
  input->anim start ms                     0.28
  input->first frame ms                   16.47
  dwm missed vblanks/burst                13.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      8.66
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | move | 15.53 | 16.39 | 62.00 | 0.67 | 1.45 | 0.00 | 5.71 | 5.75 | 8.28 | 8.28 | - | - | - | - | - | - | - | - | - | - | - | - | 0.43 | 0.00 | - | 4.80 | 9.20 | 0.70 | 3.40 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.28 | 16.47 | 13.00 | 11.44 | 8.66 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          39.16
  command p90 ms                          40.36
  frames                                  62.00
  tick ms/frame                            1.80
  frame p90 ms                             4.31
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         23.89
  interval max ms (median burst)          23.89
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
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.20
  cloak ms (outside frames)               19.80
  session_begin ms (outside frames)        1.90
  session_overlays ms (outside frames)    14.60
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
  input->anim start ms                     0.28
  input->first frame ms                   44.01
  dwm missed vblanks/burst                11.00
  dwm display interval p90 ms             11.46
  dwm gpu busy p90 ms                      4.79
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | resize | 39.16 | 40.36 | 62.00 | 1.80 | 4.31 | 1.00 | 5.71 | 5.78 | 23.89 | 23.89 | - | - | - | - | - | - | - | - | - | - | - | - | 1.05 | 0.04 | - | 3.20 | 19.80 | 1.90 | 14.60 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.28 | 44.01 | 11.00 | 11.46 | 4.79 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          31.48
  command p90 ms                          32.92
  frames                                  98.00
  tick ms/frame                            1.29
  frame p90 ms                             1.24
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         24.40
  interval max ms (median burst)          24.40
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
  dwm_flush ms (outside frames)            6.70
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
  dwm missed vblanks/burst                20.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      4.92
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | switch | 31.48 | 32.92 | 98.00 | 1.29 | 1.24 | 3.00 | 5.72 | 5.76 | 24.40 | 24.40 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.03 | - | 6.70 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 20.00 | 11.43 | 4.92 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          24.89
  command p90 ms                          29.75
  frames                                 143.00
  tick ms/frame                            1.22
  frame p90 ms                             3.25
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         48.86
  interval max ms (median burst)          48.86
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
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.00
  cloak ms (outside frames)               23.40
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)    17.20
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
  input->anim start ms                     5.80
  input->first frame ms                   57.00
  dwm missed vblanks/burst                14.00
  dwm display interval p90 ms             11.45
  dwm gpu busy p90 ms                      7.25
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | minimize | 24.89 | 29.75 | 143.00 | 1.22 | 3.25 | 1.00 | 5.71 | 5.76 | 48.86 | 48.86 | - | - | - | - | - | - | - | - | - | - | - | - | 0.68 | 0.02 | 1.00 | 5.00 | 23.40 | 2.20 | 17.20 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 5.80 | 57.00 | 14.00 | 11.45 | 7.25 |
tiles main cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          14.36
  command p90 ms                          16.04
  frames                                  63.00
  tick ms/frame                            0.72
  frame p90 ms                             1.33
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          8.71
  interval max ms (median burst)           8.71
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
  batch_commit ms/frame                    0.43
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            6.60
  cloak ms (outside frames)                9.70
  session_begin ms (outside frames)        1.00
  session_overlays ms (outside frames)     2.00
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
  input->first frame ms                   15.23
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      7.25
| main cpu0 bd-config bo-config #2 | move | 14.36 | 16.04 | 63.00 | 0.72 | 1.33 | 0.00 | 5.71 | 5.74 | 8.71 | 8.71 | - | - | - | - | - | - | - | - | - | - | - | - | 0.43 | 0.00 | 1.00 | 6.60 | 9.70 | 1.00 | 2.00 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.23 | 15.23 | 5.00 | 11.43 | 7.25 |
label=main cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.34
  command p90 ms                          32.86
  frames                                  63.00
  tick ms/frame                            2.22
  frame p90 ms                             4.29
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         25.84
  interval max ms (median burst)          25.84
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
  dwm_flush ms (outside frames)            4.30
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
  input->first frame ms                   32.78
  dwm missed vblanks/burst                 8.00
  dwm display interval p90 ms              5.75
  dwm gpu busy p90 ms                      2.70
| main cpu0 bd-config bo-config #2 | resize | 29.34 | 32.86 | 63.00 | 2.22 | 4.29 | 2.00 | 5.72 | 5.78 | 25.84 | 25.84 | - | - | - | - | - | - | - | - | - | - | - | - | 1.00 | 0.03 | 2.00 | 4.30 | 17.70 | 2.20 | 7.40 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.22 | 32.78 | 8.00 | 5.75 | 2.70 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          19.98
  command p90 ms                          21.59
  frames                                  97.00
  tick ms/frame                            1.68
  frame p90 ms                             2.65
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         51.24
  interval max ms (median burst)          52.15
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
  dwm_flush ms (outside frames)            4.50
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
  dwm missed vblanks/burst                 8.00
  dwm display interval p90 ms              5.77
  dwm gpu busy p90 ms                      3.12
| main cpu0 bd-config bo-config #2 | switch | 19.98 | 21.59 | 97.00 | 1.68 | 2.65 | 1.00 | 5.71 | 5.74 | 51.24 | 52.15 | - | - | - | - | - | - | - | - | - | - | - | - | 0.65 | 0.01 | 3.00 | 4.50 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 8.00 | 5.77 | 3.12 |
label=main cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.40
  command p90 ms                          20.98
  frames                                 149.00
  tick ms/frame                            1.56
  frame p90 ms                             3.99
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         32.02
  interval max ms (median burst)          32.46
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
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)               18.30
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)     6.30
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
  input->anim start ms                     6.55
  input->first frame ms                   45.84
  dwm missed vblanks/burst                 3.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      4.46
| main cpu0 bd-config bo-config #2 | minimize | 16.40 | 20.98 | 149.00 | 1.56 | 3.99 | 3.00 | 5.72 | 5.76 | 32.02 | 32.46 | - | - | - | - | - | - | - | - | - | - | - | - | 0.76 | 0.03 | 4.00 | 5.70 | 18.30 | 2.20 | 6.30 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.55 | 45.84 | 3.00 | 11.44 | 4.46 |
tiles this cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.59
  command p90 ms                          18.83
  frames                                  63.00
  tick ms/frame                            0.39
  frame p90 ms                             0.51
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.48
  interval max ms (median burst)          10.48
  peek_frame ms/frame                      0.34
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.66
  pd_alpha ms (worst frame)                0.24
  pd_draw ms (worst frame)                 0.61
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
  command ms (WM side)                    17.30
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.40
  peek_handover ms (outside frames)        2.80
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.10
  restack ms (outside frames)              0.80
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.50
  input->first frame ms                   18.66
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.77
| this cpu0 bd-config bo-config #2 | move | 17.59 | 18.83 | 63.00 | 0.39 | 0.51 | 0.00 | 5.71 | 5.74 | 10.48 | 10.48 | 0.34 | 0.16 | 0.05 | 0.01 | 0.03 | 0.66 | 0.24 | 0.61 | - | - | - | - | - | - | - | 4.70 | 7.40 | - | - | - | 17.30 | 2.10 | 1.10 | - | - | - | 1.40 | 2.80 | 0.30 | 1.10 | 0.80 | - | - | - | - | 3.50 | 18.66 | 5.00 | 11.43 | 8.77 |
label=this cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=12 wall=12.7s
  command p50 ms                          49.83
  command p90 ms                          55.90
  frames                                  61.00
  tick ms/frame                            2.10
  frame p90 ms                             3.22
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         33.61
  interval max ms (median burst)          33.61
  peek_frame ms/frame                      1.85
  peek_draw ms/frame                       1.21
  peek_track ms/frame                      0.23
  peek_ring ms/frame                       0.11
  peek_backdrop ms/frame                   0.11
  peek_draw ms (worst frame)              13.32
  pd_alpha ms (worst frame)                1.00
  pd_draw ms (worst frame)                11.94
  pd_fill ms (worst frame)                 8.51
  fill_source ms (worst frame)             1.87
  ring_raise ms (worst frame)              5.40
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)               17.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    41.60
  peek_new ms (outside frames)             2.40
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          11.10
  peek_handover ms (outside frames)        6.90
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.21
  input->first frame ms                   57.15
  dwm missed vblanks/burst                19.00
  dwm display interval p90 ms             17.16
  dwm gpu busy p90 ms                      1.48
| this cpu0 bd-config bo-config #2 | resize | 49.83 | 55.90 | 61.00 | 2.10 | 3.22 | 3.00 | 5.71 | 5.79 | 33.61 | 33.61 | 1.85 | 1.21 | 0.23 | 0.11 | 0.11 | 13.32 | 1.00 | 11.94 | 8.51 | 1.87 | 5.40 | 0.00 | - | 0.03 | 1.00 | 5.70 | 17.30 | - | - | - | 41.60 | 2.40 | 1.90 | - | - | - | 11.10 | 6.90 | 0.00 | 0.70 | - | - | - | - | - | 2.21 | 57.15 | 19.00 | 17.16 | 1.48 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          30.45
  command p90 ms                          33.74
  frames                                  99.00
  tick ms/frame                            1.10
  frame p90 ms                             1.05
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         19.89
  interval max ms (median burst)          19.89
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
  command ms (WM side)                    30.20
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
  ws_surrogates ms (outside frames)        9.80
  ws_park ms (outside frames)              2.10
  ws_show ms (outside frames)              8.10
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                26.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      5.70
| this cpu0 bd-config bo-config #2 | switch | 30.45 | 33.74 | 99.00 | 1.10 | 1.05 | 3.00 | 5.71 | 5.76 | 19.89 | 19.89 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.40 | - | - | - | - | 30.20 | - | - | - | - | - | - | - | 0.00 | 0.60 | 0.00 | 9.80 | 2.10 | 8.10 | 0.40 | - | - | 26.00 | 11.44 | 5.70 |
label=this cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=11 wall=12.5s
  command p50 ms                          50.99
  command p90 ms                          56.69
  frames                                  62.00
  tick ms/frame                            1.75
  frame p90 ms                             3.55
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         37.80
  interval max ms (median burst)          37.80
  peek_frame ms/frame                      1.73
  peek_draw ms/frame                       1.07
  peek_track ms/frame                      0.26
  peek_ring ms/frame                       0.11
  peek_backdrop ms/frame                   0.14
  peek_draw ms (worst frame)              11.54
  pd_alpha ms (worst frame)                0.64
  pd_draw ms (worst frame)                10.89
  pd_fill ms (worst frame)                 7.11
  fill_source ms (worst frame)             1.98
  ring_raise ms (worst frame)              3.09
  restack ms (worst frame)                 1.18
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.50
  cloak ms (outside frames)               20.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    12.00
  peek_new ms (outside frames)             7.20
  peek_place ms (outside frames)           1.30
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          13.70
  peek_handover ms (outside frames)        4.00
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              1.80
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.09
  input->first frame ms                   12.14
  dwm missed vblanks/burst                19.00
  dwm display interval p90 ms             17.15
  dwm gpu busy p90 ms                      1.82
| this cpu0 bd-config bo-config #2 | minimize | 50.99 | 56.69 | 62.00 | 1.75 | 3.55 | 3.00 | 5.72 | 5.76 | 37.80 | 37.80 | 1.73 | 1.07 | 0.26 | 0.11 | 0.14 | 11.54 | 0.64 | 10.89 | 7.11 | 1.98 | 3.09 | 1.18 | - | 0.02 | 0.00 | 12.50 | 20.60 | - | - | - | 12.00 | 7.20 | 1.30 | - | - | - | 13.70 | 4.00 | 0.20 | 1.80 | - | - | - | - | - | 2.09 | 12.14 | 19.00 | 17.15 | 1.82 |
tiles main cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          13.68
  command p90 ms                          14.41
  frames                                  63.00
  tick ms/frame                            0.70
  frame p90 ms                             1.36
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.73
  interval p99 ms                          7.77
  interval max ms (median burst)           7.77
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
  batch_commit ms/frame                    0.42
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)                8.80
  session_begin ms (outside frames)        0.90
  session_overlays ms (outside frames)     2.00
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
  input->anim start ms                     0.24
  input->first frame ms                   14.35
  dwm missed vblanks/burst                 4.00
  dwm display interval p90 ms             11.42
  dwm gpu busy p90 ms                      7.35
| main cpu0 bd-config bo-config #3 | move | 13.68 | 14.41 | 63.00 | 0.70 | 1.36 | 0.00 | 5.71 | 5.73 | 7.77 | 7.77 | - | - | - | - | - | - | - | - | - | - | - | - | 0.42 | 0.01 | 1.00 | 5.70 | 8.80 | 0.90 | 2.00 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.24 | 14.35 | 4.00 | 11.42 | 7.35 |
label=main cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          28.16
  command p90 ms                          29.91
  frames                                  63.00
  tick ms/frame                            2.18
  frame p90 ms                             4.37
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
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
  batch_commit ms/frame                    1.01
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.20
  cloak ms (outside frames)               17.70
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)     6.40
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
  input->first frame ms                   34.06
  dwm missed vblanks/burst                 7.00
  dwm display interval p90 ms              5.75
  dwm gpu busy p90 ms                      1.15
| main cpu0 bd-config bo-config #3 | resize | 28.16 | 29.91 | 63.00 | 2.18 | 4.37 | 2.00 | 5.71 | 5.78 | 24.85 | 24.85 | - | - | - | - | - | - | - | - | - | - | - | - | 1.01 | 0.03 | 2.00 | 4.20 | 17.70 | 2.20 | 6.40 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.22 | 34.06 | 7.00 | 5.75 | 1.15 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          20.96
  command p90 ms                          22.56
  frames                                  97.00
  tick ms/frame                            1.70
  frame p90 ms                             2.47
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         54.34
  interval max ms (median burst)          55.25
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
  dwm missed vblanks/burst                 7.00
  dwm display interval p90 ms              5.75
  dwm gpu busy p90 ms                      2.70
| main cpu0 bd-config bo-config #3 | switch | 20.96 | 22.56 | 97.00 | 1.70 | 2.47 | 1.00 | 5.71 | 5.75 | 54.34 | 55.25 | - | - | - | - | - | - | - | - | - | - | - | - | 0.65 | 0.02 | 3.00 | 5.60 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 7.00 | 5.75 | 2.70 |
label=main cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          17.94
  command p90 ms                          19.47
  frames                                 150.00
  tick ms/frame                            1.49
  frame p90 ms                             3.98
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         34.95
  interval max ms (median burst)          34.95
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
  batch_commit ms/frame                    0.70
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)               17.90
  session_begin ms (outside frames)        2.30
  session_overlays ms (outside frames)     7.60
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
  input->anim start ms                     6.28
  input->first frame ms                   44.11
  dwm missed vblanks/burst                14.00
  dwm display interval p90 ms             11.46
  dwm gpu busy p90 ms                      6.82
| main cpu0 bd-config bo-config #3 | minimize | 17.94 | 19.47 | 150.00 | 1.49 | 3.98 | 2.00 | 5.72 | 5.76 | 34.95 | 34.95 | - | - | - | - | - | - | - | - | - | - | - | - | 0.70 | 0.03 | 4.00 | 5.40 | 17.90 | 2.30 | 7.60 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.28 | 44.11 | 14.00 | 11.46 | 6.82 |
tiles this cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.09
  command p90 ms                          18.72
  frames                                  63.00
  tick ms/frame                            0.38
  frame p90 ms                             0.54
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.49
  interval max ms (median burst)          10.49
  peek_frame ms/frame                      0.34
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.59
  pd_alpha ms (worst frame)                0.23
  pd_draw ms (worst frame)                 0.53
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
  command ms (WM side)                    17.80
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.20
  peek_handover ms (outside frames)        2.80
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.00
  restack ms (outside frames)              0.80
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.51
  input->first frame ms                   18.49
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.47
| this cpu0 bd-config bo-config #3 | move | 18.09 | 18.72 | 63.00 | 0.38 | 0.54 | 0.00 | 5.71 | 5.74 | 10.49 | 10.49 | 0.34 | 0.16 | 0.05 | 0.01 | 0.03 | 0.59 | 0.23 | 0.53 | - | - | - | - | - | - | - | 5.40 | 8.20 | - | - | - | 17.80 | 2.10 | 1.10 | - | - | - | 1.20 | 2.80 | 0.30 | 1.00 | 0.80 | - | - | - | - | 3.51 | 18.49 | 5.00 | 11.43 | 8.47 |
label=this cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          47.47
  command p90 ms                          56.47
  frames                                  61.00
  tick ms/frame                            2.43
  frame p90 ms                             3.66
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         45.03
  interval max ms (median burst)          45.03
  peek_frame ms/frame                      1.94
  peek_draw ms/frame                       1.33
  peek_track ms/frame                      0.27
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.13
  peek_draw ms (worst frame)              17.11
  pd_alpha ms (worst frame)                1.16
  pd_draw ms (worst frame)                15.82
  pd_fill ms (worst frame)                12.13
  fill_source ms (worst frame)             3.86
  ring_raise ms (worst frame)              5.99
  restack ms (worst frame)                 3.05
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.40
  cloak ms (outside frames)               18.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    47.10
  peek_new ms (outside frames)             3.40
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          11.60
  peek_handover ms (outside frames)       12.50
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.80
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.17
  input->first frame ms                   64.43
  dwm missed vblanks/burst                26.00
  dwm display interval p90 ms             17.17
  dwm gpu busy p90 ms                      2.86
| this cpu0 bd-config bo-config #3 | resize | 47.47 | 56.47 | 61.00 | 2.43 | 3.66 | 3.00 | 5.72 | 5.78 | 45.03 | 45.03 | 1.94 | 1.33 | 0.27 | 0.12 | 0.13 | 17.11 | 1.16 | 15.82 | 12.13 | 3.86 | 5.99 | 3.05 | - | 0.03 | 2.00 | 10.40 | 18.30 | - | - | - | 47.10 | 3.40 | 2.10 | - | - | - | 11.60 | 12.50 | 0.10 | 0.80 | - | - | - | - | - | 3.17 | 64.43 | 26.00 | 17.17 | 2.86 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          31.08
  command p90 ms                          33.93
  frames                                  99.00
  tick ms/frame                            1.14
  frame p90 ms                             1.17
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         21.94
  interval max ms (median burst)          21.94
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
  command ms (WM side)                    30.80
  peek_new ms (outside frames)             0.00
  peek_place ms (outside frames)           0.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           0.00
  peek_handover ms (outside frames)        0.00
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.60
  restack ms (outside frames)              0.00
  ws_surrogates ms (outside frames)       10.10
  ws_park ms (outside frames)              2.20
  ws_show ms (outside frames)              7.90
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                14.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      2.14
| this cpu0 bd-config bo-config #3 | switch | 31.08 | 33.93 | 99.00 | 1.14 | 1.17 | 3.00 | 5.72 | 5.76 | 21.94 | 21.94 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.50 | 0.00 | - | - | - | 30.80 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | 0.60 | 0.00 | 10.10 | 2.20 | 7.90 | 0.40 | 0.00 | 0.00 | 14.00 | 11.44 | 2.14 |
label=this cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          48.07
  command p90 ms                          51.01
  frames                                  62.00
  tick ms/frame                            2.17
  frame p90 ms                             3.76
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         52.13
  interval max ms (median burst)          52.13
  peek_frame ms/frame                      1.86
  peek_draw ms/frame                       1.11
  peek_track ms/frame                      0.32
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.19
  peek_draw ms (worst frame)               9.71
  pd_alpha ms (worst frame)                0.76
  pd_draw ms (worst frame)                 8.71
  pd_fill ms (worst frame)                 5.47
  fill_source ms (worst frame)             0.00
  ring_raise ms (worst frame)              4.00
  restack ms (worst frame)                 2.86
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.90
  cloak ms (outside frames)               18.60
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    47.80
  peek_new ms (outside frames)             7.20
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          18.80
  peek_handover ms (outside frames)       12.60
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              3.10
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.65
  input->first frame ms                   60.88
  dwm missed vblanks/burst                28.00
  dwm display interval p90 ms             22.84
  dwm gpu busy p90 ms                      3.09
| this cpu0 bd-config bo-config #3 | minimize | 48.07 | 51.01 | 62.00 | 2.17 | 3.76 | 2.00 | 5.72 | 5.77 | 52.13 | 52.13 | 1.86 | 1.11 | 0.32 | 0.13 | 0.19 | 9.71 | 0.76 | 8.71 | 5.47 | 0.00 | 4.00 | 2.86 | - | 0.02 | 2.00 | 10.90 | 18.60 | - | - | - | 47.80 | 7.20 | 1.80 | - | - | - | 18.80 | 12.60 | 0.40 | 3.10 | - | - | - | - | - | 8.65 | 60.88 | 28.00 | 22.84 | 3.09 |
tiles stand-ins:nopeeks cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          15.92
  command p90 ms                          17.41
  frames                                  62.00
  tick ms/frame                            0.63
  frame p90 ms                             1.27
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          8.00
  interval max ms (median burst)           8.00
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
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.60
  cloak ms (outside frames)               10.00
  session_begin ms (outside frames)        0.70
  session_overlays ms (outside frames)     3.40
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
  input->first frame ms                   16.60
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.08
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | move | 15.92 | 17.41 | 62.00 | 0.63 | 1.27 | 0.00 | 5.71 | 5.74 | 8.00 | 8.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.39 | 0.00 | - | 5.60 | 10.00 | 0.70 | 3.40 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.29 | 16.60 | 5.00 | 11.43 | 8.08 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          39.40
  command p90 ms                          43.88
  frames                                  62.00
  tick ms/frame                            1.73
  frame p90 ms                             4.09
  frames >2x budget                        1.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         20.36
  interval max ms (median burst)          20.36
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
  batch_commit ms/frame                    0.97
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.20
  cloak ms (outside frames)               19.90
  session_begin ms (outside frames)        1.80
  session_overlays ms (outside frames)    15.60
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
  input->anim start ms                     0.28
  input->first frame ms                   44.45
  dwm missed vblanks/burst                 6.00
  dwm display interval p90 ms             11.42
  dwm gpu busy p90 ms                      5.13
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | resize | 39.40 | 43.88 | 62.00 | 1.73 | 4.09 | 1.00 | 5.72 | 5.76 | 20.36 | 20.36 | - | - | - | - | - | - | - | - | - | - | - | - | 0.97 | 0.03 | - | 4.20 | 19.90 | 1.80 | 15.60 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.28 | 44.45 | 6.00 | 11.42 | 5.13 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          31.08
  command p90 ms                          33.93
  frames                                  99.00
  tick ms/frame                            1.26
  frame p90 ms                             1.21
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         23.22
  interval max ms (median burst)          23.22
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
  dwm_flush ms (outside frames)            6.00
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
  dwm missed vblanks/burst                19.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      2.44
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | switch | 31.08 | 33.93 | 99.00 | 1.26 | 1.21 | 3.00 | 5.71 | 5.77 | 23.22 | 23.22 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.03 | - | 6.00 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 19.00 | 11.43 | 2.44 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          27.36
  command p90 ms                          29.20
  frames                                 144.00
  tick ms/frame                            1.17
  frame p90 ms                             3.17
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         49.03
  interval max ms (median burst)          49.03
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
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)               22.50
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)    18.50
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
  input->anim start ms                     5.73
  input->first frame ms                   57.00
  dwm missed vblanks/burst                15.00
  dwm display interval p90 ms             11.46
  dwm gpu busy p90 ms                      6.56
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | minimize | 27.36 | 29.20 | 144.00 | 1.17 | 3.17 | 0.00 | 5.71 | 5.77 | 49.03 | 49.03 | - | - | - | - | - | - | - | - | - | - | - | - | 0.64 | 0.02 | 1.00 | 4.40 | 22.50 | 2.20 | 18.50 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 5.73 | 57.00 | 15.00 | 11.46 | 6.56 |
tiles this cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-this-move-mid.png (mid at 66 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-this-move-rest.png
label=this cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.82
  command p90 ms                          20.76
  frames                                  63.00
  tick ms/frame                            0.42
  frame p90 ms                             0.53
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         12.49
  interval max ms (median burst)          12.49
  peek_frame ms/frame                      0.33
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.62
  pd_alpha ms (worst frame)                0.26
  pd_draw ms (worst frame)                 0.47
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)                7.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.60
  peek_new ms (outside frames)             2.30
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.30
  peek_handover ms (outside frames)        3.40
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.20
  restack ms (outside frames)              0.80
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.69
  input->first frame ms                   19.37
  dwm missed vblanks/burst                 6.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.67
| this cpu8 bd-config bo-config #1 | move | 18.82 | 20.76 | 63.00 | 0.42 | 0.53 | 0.00 | 5.71 | 5.77 | 12.49 | 12.49 | 0.33 | 0.17 | 0.05 | 0.01 | 0.04 | 0.62 | 0.26 | 0.47 | - | - | - | - | - | - | - | 5.10 | 7.90 | - | - | - | 18.60 | 2.30 | 1.20 | - | - | - | 1.30 | 3.40 | 0.30 | 1.20 | 0.80 | - | - | - | - | 3.69 | 19.37 | 6.00 | 11.43 | 8.67 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-this-resize-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-this-resize-rest.png
label=this cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          55.88
  command p90 ms                          62.24
  frames                                  60.00
  tick ms/frame                            2.71
  frame p90 ms                             4.09
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         45.19
  interval max ms (median burst)          45.19
  peek_frame ms/frame                      2.17
  peek_draw ms/frame                       1.47
  peek_track ms/frame                      0.34
  peek_ring ms/frame                       0.16
  peek_backdrop ms/frame                   0.17
  peek_draw ms (worst frame)              22.74
  pd_alpha ms (worst frame)                1.20
  pd_draw ms (worst frame)                21.53
  pd_fill ms (worst frame)                17.55
  fill_source ms (worst frame)             5.96
  ring_raise ms (worst frame)              5.67
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.20
  cloak ms (outside frames)               21.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    55.50
  peek_new ms (outside frames)             4.40
  peek_place ms (outside frames)           2.30
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          12.80
  peek_handover ms (outside frames)       13.20
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              1.00
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.81
  input->first frame ms                   79.09
  dwm missed vblanks/burst                25.00
  dwm display interval p90 ms             17.16
  dwm gpu busy p90 ms                      1.57
| this cpu8 bd-config bo-config #1 | resize | 55.88 | 62.24 | 60.00 | 2.71 | 4.09 | 3.00 | 5.72 | 5.77 | 45.19 | 45.19 | 2.17 | 1.47 | 0.34 | 0.16 | 0.17 | 22.74 | 1.20 | 21.53 | 17.55 | 5.96 | 5.67 | 0.00 | - | 0.05 | 2.00 | 10.20 | 21.00 | - | - | - | 55.50 | 4.40 | 2.30 | - | - | - | 12.80 | 13.20 | 0.20 | 1.00 | - | - | - | - | - | 3.81 | 79.09 | 25.00 | 17.16 | 1.57 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-this-switch-mid.png (mid at 72 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-this-switch-rest.png
label=this cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.64
  command p90 ms                          36.38
  frames                                  99.00
  tick ms/frame                            1.25
  frame p90 ms                             1.30
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         23.19
  interval max ms (median burst)          23.19
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
  dwm_flush ms (outside frames)            5.50
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    34.30
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)       11.70
  ws_park ms (outside frames)              2.40
  ws_show ms (outside frames)              8.30
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                30.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      5.30
| this cpu8 bd-config bo-config #1 | switch | 34.64 | 36.38 | 99.00 | 1.25 | 1.30 | 3.00 | 5.71 | 5.76 | 23.19 | 23.19 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 5.50 | - | - | - | - | 34.30 | - | - | - | - | - | - | - | 0.00 | 0.70 | - | 11.70 | 2.40 | 8.30 | 0.40 | - | - | 30.00 | 11.44 | 5.30 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-this-minimize-mid.png (mid at 64 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-this-minimize-rest.png
label=this cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          53.14
  command p90 ms                          58.46
  frames                                  61.00
  tick ms/frame                            2.50
  frame p90 ms                             4.35
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         68.56
  interval max ms (median burst)          68.56
  peek_frame ms/frame                      2.17
  peek_draw ms/frame                       1.33
  peek_track ms/frame                      0.39
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.24
  peek_draw ms (worst frame)              18.57
  pd_alpha ms (worst frame)                1.18
  pd_draw ms (worst frame)                17.33
  pd_fill ms (worst frame)                13.36
  fill_source ms (worst frame)             5.66
  ring_raise ms (worst frame)              4.23
  restack ms (worst frame)                 2.78
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           13.10
  cloak ms (outside frames)               19.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    52.90
  peek_new ms (outside frames)            10.40
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          19.00
  peek_handover ms (outside frames)       15.70
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              4.70
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.23
  input->first frame ms                   75.09
  dwm missed vblanks/burst                35.00
  dwm display interval p90 ms             22.91
  dwm gpu busy p90 ms                      9.56
| this cpu8 bd-config bo-config #1 | minimize | 53.14 | 58.46 | 61.00 | 2.50 | 4.35 | 3.00 | 5.71 | 5.78 | 68.56 | 68.56 | 2.17 | 1.33 | 0.39 | 0.14 | 0.24 | 18.57 | 1.18 | 17.33 | 13.36 | 5.66 | 4.23 | 2.78 | - | 0.03 | 2.00 | 13.10 | 19.80 | - | - | - | 52.90 | 10.40 | 1.80 | - | - | - | 19.00 | 15.70 | 0.40 | 4.70 | - | - | - | - | - | 10.23 | 75.09 | 35.00 | 22.91 | 9.56 |
tiles stand-ins:nopeeks cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-move-mid.png (mid at 70 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-move-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          15.70
  command p90 ms                          17.81
  frames                                  62.00
  tick ms/frame                            0.70
  frame p90 ms                             1.51
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          9.07
  interval max ms (median burst)           9.07
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
  batch_commit ms/frame                    0.46
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.70
  cloak ms (outside frames)                8.90
  session_begin ms (outside frames)        0.70
  session_overlays ms (outside frames)     4.10
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
  input->first frame ms                   16.81
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.76
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | move | 15.70 | 17.81 | 62.00 | 0.70 | 1.51 | 0.00 | 5.71 | 5.74 | 9.07 | 9.07 | - | - | - | - | - | - | - | - | - | - | - | - | 0.46 | 0.01 | - | 3.70 | 8.90 | 0.70 | 4.10 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.31 | 16.81 | 5.00 | 11.43 | 8.76 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-resize-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-resize-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          44.16
  command p90 ms                          48.09
  frames                                  63.00
  tick ms/frame                            1.91
  frame p90 ms                             4.62
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         18.90
  interval max ms (median burst)          18.90
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
  batch_commit ms/frame                    1.11
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)               21.60
  session_begin ms (outside frames)        2.00
  session_overlays ms (outside frames)    17.70
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
  input->anim start ms                     0.32
  input->first frame ms                   49.33
  dwm missed vblanks/burst                 6.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      3.65
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | resize | 44.16 | 48.09 | 63.00 | 1.91 | 4.62 | 2.00 | 5.71 | 5.79 | 18.90 | 18.90 | - | - | - | - | - | - | - | - | - | - | - | - | 1.11 | 0.03 | - | 4.50 | 21.60 | 2.00 | 17.70 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.32 | 49.33 | 6.00 | 11.43 | 3.65 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-switch-mid.png (mid at 73 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-switch-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.07
  command p90 ms                          36.15
  frames                                  99.00
  tick ms/frame                            1.40
  frame p90 ms                             1.37
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         25.98
  interval max ms (median burst)          25.98
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
  dwm_flush ms (outside frames)            5.40
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
  dwm missed vblanks/burst                31.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      2.69
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | switch | 34.07 | 36.15 | 99.00 | 1.40 | 1.37 | 3.00 | 5.71 | 5.75 | 25.98 | 25.98 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | - | 5.40 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 31.00 | 11.44 | 2.69 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-minimize-mid.png (mid at 72 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-minimize-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          33.39
  command p90 ms                          37.88
  frames                                 143.00
  tick ms/frame                            1.43
  frame p90 ms                             3.67
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         61.49
  interval max ms (median burst)          61.49
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)               29.50
  session_begin ms (outside frames)        5.60
  session_overlays ms (outside frames)    19.90
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
  input->anim start ms                     6.05
  input->first frame ms                   68.99
  dwm missed vblanks/burst                17.00
  dwm display interval p90 ms             11.45
  dwm gpu busy p90 ms                      6.53
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | minimize | 33.39 | 37.88 | 143.00 | 1.43 | 3.67 | 1.00 | 5.71 | 5.77 | 61.49 | 61.49 | - | - | - | - | - | - | - | - | - | - | - | - | 0.77 | 0.02 | 1.00 | 4.70 | 29.50 | 5.60 | 19.90 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.05 | 68.99 | 17.00 | 11.45 | 6.53 |
tiles main cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-main-default-move-mid.png (mid at 74 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-main-default-move-rest.png
label=main cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          14.29
  command p90 ms                          15.03
  frames                                  63.00
  tick ms/frame                            0.80
  frame p90 ms                             1.60
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                          8.81
  interval max ms (median burst)           8.81
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
  batch_commit ms/frame                    0.52
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.80
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
  input->first frame ms                   15.34
  dwm missed vblanks/burst                 4.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      7.37
| main cpu8 bd-config bo-config #1 | move | 14.29 | 15.03 | 63.00 | 0.80 | 1.60 | 0.00 | 5.71 | 5.75 | 8.81 | 8.81 | - | - | - | - | - | - | - | - | - | - | - | - | 0.52 | 0.01 | 1.00 | 5.80 | 9.40 | 1.10 | 2.00 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.25 | 15.34 | 4.00 | 11.43 | 7.37 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-main-default-resize-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-main-default-resize-rest.png
label=main cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          31.92
  command p90 ms                          34.33
  frames                                  63.00
  tick ms/frame                            2.57
  frame p90 ms                             4.82
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         32.13
  interval max ms (median burst)          32.13
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
  batch_commit ms/frame                    1.13
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)               18.90
  session_begin ms (outside frames)        2.80
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
  input->anim start ms                     0.25
  input->first frame ms                   36.83
  dwm missed vblanks/burst                 6.00
  dwm display interval p90 ms              5.74
  dwm gpu busy p90 ms                      1.44
| main cpu8 bd-config bo-config #1 | resize | 31.92 | 34.33 | 63.00 | 2.57 | 4.82 | 2.00 | 5.71 | 5.76 | 32.13 | 32.13 | - | - | - | - | - | - | - | - | - | - | - | - | 1.13 | 0.04 | 2.00 | 4.50 | 18.90 | 2.80 | 7.40 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.25 | 36.83 | 6.00 | 5.74 | 1.44 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-main-default-switch-mid.png (mid at 71 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-main-default-switch-rest.png
label=main cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          21.85
  command p90 ms                          23.02
  frames                                  97.00
  tick ms/frame                            1.91
  frame p90 ms                             2.89
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         58.80
  interval max ms (median burst)          58.80
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
  dwm missed vblanks/burst                11.00
  dwm display interval p90 ms             11.42
  dwm gpu busy p90 ms                      3.76
| main cpu8 bd-config bo-config #1 | switch | 21.85 | 23.02 | 97.00 | 1.91 | 2.89 | 1.00 | 5.71 | 5.75 | 58.80 | 58.80 | - | - | - | - | - | - | - | - | - | - | - | - | 0.73 | 0.02 | 3.00 | 5.60 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 11.00 | 11.42 | 3.76 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-main-default-minimize-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-124119\r1-cpu8-bdconfigboconfig-main-default-minimize-rest.png
label=main cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          21.09
  command p90 ms                          23.33
  frames                                 150.00
  tick ms/frame                            1.68
  frame p90 ms                             4.37
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.81
  interval p99 ms                         45.00
  interval max ms (median burst)          45.00
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
  batch_commit ms/frame                    0.85
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.00
  cloak ms (outside frames)               23.70
  session_begin ms (outside frames)        5.00
  session_overlays ms (outside frames)     8.70
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
  input->anim start ms                     7.42
  input->first frame ms                   57.72
  dwm missed vblanks/burst                13.00
  dwm display interval p90 ms             17.13
  dwm gpu busy p90 ms                      6.60
| main cpu8 bd-config bo-config #1 | minimize | 21.09 | 23.33 | 150.00 | 1.68 | 4.37 | 3.00 | 5.71 | 5.81 | 45.00 | 45.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.85 | 0.03 | 4.00 | 4.00 | 23.70 | 5.00 | 8.70 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 7.42 | 57.72 | 13.00 | 17.13 | 6.60 |
tiles stand-ins:nopeeks cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.92
  command p90 ms                          17.95
  frames                                  62.00
  tick ms/frame                            0.71
  frame p90 ms                             1.50
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          9.19
  interval max ms (median burst)           9.19
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
  batch_commit ms/frame                    0.47
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)                9.70
  session_begin ms (outside frames)        0.70
  session_overlays ms (outside frames)     4.20
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
  input->first frame ms                   17.66
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.69
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | move | 16.92 | 17.95 | 62.00 | 0.71 | 1.50 | 0.00 | 5.71 | 5.74 | 9.19 | 9.19 | - | - | - | - | - | - | - | - | - | - | - | - | 0.47 | 0.01 | - | 4.40 | 9.70 | 0.70 | 4.20 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.32 | 17.66 | 5.00 | 11.43 | 8.69 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          45.83
  command p90 ms                          48.02
  frames                                  62.00
  tick ms/frame                            1.99
  frame p90 ms                             5.08
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.80
  interval p99 ms                         19.65
  interval max ms (median burst)          19.65
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
  batch_commit ms/frame                    1.11
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.30
  cloak ms (outside frames)               22.90
  session_begin ms (outside frames)        2.00
  session_overlays ms (outside frames)    17.10
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
  input->anim start ms                     0.33
  input->first frame ms                   50.62
  dwm missed vblanks/burst                12.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      6.24
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | resize | 45.83 | 48.02 | 62.00 | 1.99 | 5.08 | 2.00 | 5.71 | 5.80 | 19.65 | 19.65 | - | - | - | - | - | - | - | - | - | - | - | - | 1.11 | 0.04 | - | 5.30 | 22.90 | 2.00 | 17.10 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.33 | 50.62 | 12.00 | 11.44 | 6.24 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.22
  command p90 ms                          35.38
  frames                                  99.00
  tick ms/frame                            1.40
  frame p90 ms                             1.38
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         25.52
  interval max ms (median burst)          26.11
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
  dwm missed vblanks/burst                21.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      1.86
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | switch | 34.22 | 35.38 | 99.00 | 1.40 | 1.38 | 3.00 | 5.71 | 5.76 | 25.52 | 26.11 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.04 | - | 5.20 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 21.00 | 11.43 | 1.86 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          30.64
  command p90 ms                          34.71
  frames                                 143.00
  tick ms/frame                            1.38
  frame p90 ms                             3.56
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         54.32
  interval max ms (median burst)          54.32
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            3.90
  cloak ms (outside frames)               24.10
  session_begin ms (outside frames)        2.60
  session_overlays ms (outside frames)    19.80
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
  input->anim start ms                     6.55
  input->first frame ms                   62.78
  dwm missed vblanks/burst                16.00
  dwm display interval p90 ms             11.46
  dwm gpu busy p90 ms                      7.30
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | minimize | 30.64 | 34.71 | 143.00 | 1.38 | 3.56 | 2.00 | 5.71 | 5.77 | 54.32 | 54.32 | - | - | - | - | - | - | - | - | - | - | - | - | 0.78 | 0.02 | 1.00 | 3.90 | 24.10 | 2.60 | 19.80 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.55 | 62.78 | 16.00 | 11.46 | 7.30 |
tiles main cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          13.66
  command p90 ms                          14.84
  frames                                  63.00
  tick ms/frame                            0.78
  frame p90 ms                             1.58
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                          8.84
  interval max ms (median burst)           8.84
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
  batch_commit ms/frame                    0.49
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.00
  cloak ms (outside frames)                8.50
  session_begin ms (outside frames)        1.00
  session_overlays ms (outside frames)     1.90
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
  input->anim start ms                     0.24
  input->first frame ms                   14.33
  dwm missed vblanks/burst                 4.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      7.31
| main cpu8 bd-config bo-config #2 | move | 13.66 | 14.84 | 63.00 | 0.78 | 1.58 | 0.00 | 5.71 | 5.75 | 8.84 | 8.84 | - | - | - | - | - | - | - | - | - | - | - | - | 0.49 | 0.00 | 1.00 | 5.00 | 8.50 | 1.00 | 1.90 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.24 | 14.33 | 4.00 | 11.43 | 7.31 |
label=main cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          32.45
  command p90 ms                          34.17
  frames                                  63.00
  tick ms/frame                            2.48
  frame p90 ms                             4.85
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         28.83
  interval max ms (median burst)          28.83
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
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.00
  cloak ms (outside frames)               19.00
  session_begin ms (outside frames)        2.80
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
  input->anim start ms                     0.25
  input->first frame ms                   36.80
  dwm missed vblanks/burst                 7.00
  dwm display interval p90 ms              5.74
  dwm gpu busy p90 ms                      1.56
| main cpu8 bd-config bo-config #2 | resize | 32.45 | 34.17 | 63.00 | 2.48 | 4.85 | 2.00 | 5.71 | 5.77 | 28.83 | 28.83 | - | - | - | - | - | - | - | - | - | - | - | - | 1.15 | 0.04 | 2.00 | 5.00 | 19.00 | 2.80 | 7.40 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.25 | 36.80 | 7.00 | 5.74 | 1.56 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          22.13
  command p90 ms                          22.63
  frames                                  97.00
  tick ms/frame                            1.93
  frame p90 ms                             3.24
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         59.20
  interval max ms (median burst)          59.21
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
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.10
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
  dwm missed vblanks/burst                 7.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      3.04
| main cpu8 bd-config bo-config #2 | switch | 22.13 | 22.63 | 97.00 | 1.93 | 3.24 | 1.00 | 5.71 | 5.75 | 59.20 | 59.21 | - | - | - | - | - | - | - | - | - | - | - | - | 0.76 | 0.02 | 3.00 | 5.10 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 7.00 | 11.43 | 3.04 |
label=main cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          22.46
  command p90 ms                          24.44
  frames                                 148.00
  tick ms/frame                            1.70
  frame p90 ms                             4.45
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         36.73
  interval max ms (median burst)          36.73
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
  batch_commit ms/frame                    0.80
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.60
  cloak ms (outside frames)               17.50
  session_begin ms (outside frames)        2.40
  session_overlays ms (outside frames)     8.00
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
  input->anim start ms                     7.38
  input->first frame ms                   46.68
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      2.64
| main cpu8 bd-config bo-config #2 | minimize | 22.46 | 24.44 | 148.00 | 1.70 | 4.45 | 3.00 | 5.71 | 5.77 | 36.73 | 36.73 | - | - | - | - | - | - | - | - | - | - | - | - | 0.80 | 0.03 | 4.00 | 4.60 | 17.50 | 2.40 | 8.00 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 7.38 | 46.68 | 5.00 | 11.43 | 2.64 |
tiles this cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.69
  command p90 ms                          20.45
  frames                                  63.00
  tick ms/frame                            0.42
  frame p90 ms                             0.55
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         11.82
  interval max ms (median burst)          11.82
  peek_frame ms/frame                      0.36
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.67
  pd_alpha ms (worst frame)                0.30
  pd_draw ms (worst frame)                 0.49
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)                7.50
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    18.40
  peek_new ms (outside frames)             2.20
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.60
  peek_handover ms (outside frames)        3.20
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.20
  restack ms (outside frames)              0.90
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.59
  input->first frame ms                   19.64
  dwm missed vblanks/burst                 6.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.69
| this cpu8 bd-config bo-config #2 | move | 18.69 | 20.45 | 63.00 | 0.42 | 0.55 | 0.00 | 5.71 | 5.76 | 11.82 | 11.82 | 0.36 | 0.17 | 0.05 | 0.01 | 0.03 | 0.67 | 0.30 | 0.49 | - | - | - | - | - | - | - | 4.40 | 7.50 | - | - | - | 18.40 | 2.20 | 1.20 | - | - | - | 1.60 | 3.20 | 0.30 | 1.20 | 0.90 | - | - | - | - | 3.59 | 19.64 | 6.00 | 11.43 | 8.69 |
label=this cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          53.74
  command p90 ms                          59.40
  frames                                  61.00
  tick ms/frame                            2.75
  frame p90 ms                             4.25
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         41.23
  interval max ms (median burst)          41.23
  peek_frame ms/frame                      2.20
  peek_draw ms/frame                       1.50
  peek_track ms/frame                      0.31
  peek_ring ms/frame                       0.15
  peek_backdrop ms/frame                   0.14
  peek_draw ms (worst frame)              18.77
  pd_alpha ms (worst frame)                1.23
  pd_draw ms (worst frame)                17.46
  pd_fill ms (worst frame)                13.34
  fill_source ms (worst frame)             5.04
  ring_raise ms (worst frame)              6.00
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.20
  cloak ms (outside frames)               24.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    53.30
  peek_new ms (outside frames)             4.10
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          12.60
  peek_handover ms (outside frames)       12.00
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.90
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.63
  input->first frame ms                   77.89
  dwm missed vblanks/burst                25.00
  dwm display interval p90 ms             17.27
  dwm gpu busy p90 ms                      1.56
| this cpu8 bd-config bo-config #2 | resize | 53.74 | 59.40 | 61.00 | 2.75 | 4.25 | 3.00 | 5.71 | 5.77 | 41.23 | 41.23 | 2.20 | 1.50 | 0.31 | 0.15 | 0.14 | 18.77 | 1.23 | 17.46 | 13.34 | 5.04 | 6.00 | 0.00 | - | 0.05 | 2.00 | 10.20 | 24.80 | - | - | - | 53.30 | 4.10 | 2.10 | - | - | - | 12.60 | 12.00 | 0.10 | 0.90 | - | - | - | - | - | 3.63 | 77.89 | 25.00 | 17.27 | 1.56 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.32
  command p90 ms                          36.81
  frames                                  99.00
  tick ms/frame                            1.28
  frame p90 ms                             1.40
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         22.83
  interval max ms (median burst)          23.96
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
  command ms (WM side)                    34.00
  peek_new ms (outside frames)             0.00
  peek_place ms (outside frames)           0.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           0.00
  peek_handover ms (outside frames)        0.00
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.70
  restack ms (outside frames)              0.00
  ws_surrogates ms (outside frames)       11.80
  ws_park ms (outside frames)              2.40
  ws_show ms (outside frames)              8.60
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
  dwm missed vblanks/burst                25.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      5.47
| this cpu8 bd-config bo-config #2 | switch | 34.32 | 36.81 | 99.00 | 1.28 | 1.40 | 3.00 | 5.71 | 5.76 | 22.83 | 23.96 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.90 | 0.00 | - | - | - | 34.00 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | 0.70 | 0.00 | 11.80 | 2.40 | 8.60 | 0.40 | 0.00 | 0.00 | 25.00 | 11.44 | 5.47 |
label=this cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          55.51
  command p90 ms                          61.95
  frames                                  62.00
  tick ms/frame                            2.52
  frame p90 ms                             4.47
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         61.17
  interval max ms (median burst)          61.17
  peek_frame ms/frame                      2.09
  peek_draw ms/frame                       1.28
  peek_track ms/frame                      0.40
  peek_ring ms/frame                       0.16
  peek_backdrop ms/frame                   0.23
  peek_draw ms (worst frame)              14.24
  pd_alpha ms (worst frame)                1.11
  pd_draw ms (worst frame)                13.28
  pd_fill ms (worst frame)                 8.64
  fill_source ms (worst frame)             2.60
  ring_raise ms (worst frame)              3.77
  restack ms (worst frame)                 2.82
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           13.50
  cloak ms (outside frames)               23.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    55.10
  peek_new ms (outside frames)             9.20
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          21.90
  peek_handover ms (outside frames)       16.40
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              2.40
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.28
  input->first frame ms                   72.03
  dwm missed vblanks/burst                28.00
  dwm display interval p90 ms             22.88
  dwm gpu busy p90 ms                      8.40
| this cpu8 bd-config bo-config #2 | minimize | 55.51 | 61.95 | 62.00 | 2.52 | 4.47 | 3.00 | 5.71 | 5.79 | 61.17 | 61.17 | 2.09 | 1.28 | 0.40 | 0.16 | 0.23 | 14.24 | 1.11 | 13.28 | 8.64 | 2.60 | 3.77 | 2.82 | - | 0.03 | 2.00 | 13.50 | 23.20 | - | - | - | 55.10 | 9.20 | 1.80 | - | - | - | 21.90 | 16.40 | 0.30 | 2.40 | - | - | - | - | - | 9.28 | 72.03 | 28.00 | 22.88 | 8.40 |
tiles main cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          13.91
  command p90 ms                          14.92
  frames                                  63.00
  tick ms/frame                            0.77
  frame p90 ms                             1.58
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          9.02
  interval max ms (median burst)           9.02
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
  batch_commit ms/frame                    0.49
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.20
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
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)              -
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     0.24
  input->first frame ms                   14.60
  dwm missed vblanks/burst                 4.00
  dwm display interval p90 ms             11.42
  dwm gpu busy p90 ms                      7.33
| main cpu8 bd-config bo-config #3 | move | 13.91 | 14.92 | 63.00 | 0.77 | 1.58 | 0.00 | 5.71 | 5.74 | 9.02 | 9.02 | - | - | - | - | - | - | - | - | - | - | - | - | 0.49 | 0.00 | 1.00 | 5.20 | 8.60 | 1.10 | 1.90 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.24 | 14.60 | 4.00 | 11.42 | 7.33 |
label=main cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          31.38
  command p90 ms                          34.19
  frames                                  63.00
  tick ms/frame                            2.37
  frame p90 ms                             4.45
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         30.43
  interval max ms (median burst)          30.43
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
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            3.90
  cloak ms (outside frames)               18.80
  session_begin ms (outside frames)        2.70
  session_overlays ms (outside frames)     7.30
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
  input->first frame ms                   37.69
  dwm missed vblanks/burst                 7.00
  dwm display interval p90 ms              5.74
  dwm gpu busy p90 ms                      1.64
| main cpu8 bd-config bo-config #3 | resize | 31.38 | 34.19 | 63.00 | 2.37 | 4.45 | 2.00 | 5.71 | 5.75 | 30.43 | 30.43 | - | - | - | - | - | - | - | - | - | - | - | - | 1.06 | 0.04 | 2.00 | 3.90 | 18.80 | 2.70 | 7.30 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.22 | 37.69 | 7.00 | 5.74 | 1.64 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          22.50
  command p90 ms                          23.78
  frames                                  97.00
  tick ms/frame                            1.97
  frame p90 ms                             3.22
  frames >2x budget                        1.00
  interval p50 ms                          5.72
  interval p90 ms                          5.75
  interval p99 ms                         59.38
  interval max ms (median burst)          59.38
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
  dwm missed vblanks/burst                11.00
  dwm display interval p90 ms             11.36
  dwm gpu busy p90 ms                      4.10
| main cpu8 bd-config bo-config #3 | switch | 22.50 | 23.78 | 97.00 | 1.97 | 3.22 | 1.00 | 5.72 | 5.75 | 59.38 | 59.38 | - | - | - | - | - | - | - | - | - | - | - | - | 0.75 | 0.02 | 3.00 | 5.60 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 11.00 | 11.36 | 4.10 |
label=main cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          20.71
  command p90 ms                          25.12
  frames                                 148.00
  tick ms/frame                            1.66
  frame p90 ms                             4.38
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         33.86
  interval max ms (median burst)          43.08
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
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.00
  cloak ms (outside frames)               16.60
  session_begin ms (outside frames)        2.60
  session_overlays ms (outside frames)     8.40
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
  input->anim start ms                     7.24
  input->first frame ms                   45.84
  dwm missed vblanks/burst                18.00
  dwm display interval p90 ms             17.13
  dwm gpu busy p90 ms                      7.41
| main cpu8 bd-config bo-config #3 | minimize | 20.71 | 25.12 | 148.00 | 1.66 | 4.38 | 1.00 | 5.71 | 5.77 | 33.86 | 43.08 | - | - | - | - | - | - | - | - | - | - | - | - | 0.81 | 0.03 | 4.00 | 4.00 | 16.60 | 2.60 | 8.40 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 7.24 | 45.84 | 18.00 | 17.13 | 7.41 |
tiles this cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.61
  command p90 ms                          19.79
  frames                                  63.00
  tick ms/frame                            0.41
  frame p90 ms                             0.52
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         12.29
  interval max ms (median burst)          12.29
  peek_frame ms/frame                      0.30
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.61
  pd_alpha ms (worst frame)                0.27
  pd_draw ms (worst frame)                 0.47
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
  command ms (WM side)                    18.30
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.70
  peek_handover ms (outside frames)        3.30
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.20
  restack ms (outside frames)              0.90
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.49
  input->first frame ms                   19.58
  dwm missed vblanks/burst                 6.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.74
| this cpu8 bd-config bo-config #3 | move | 18.61 | 19.79 | 63.00 | 0.41 | 0.52 | 0.00 | 5.71 | 5.75 | 12.29 | 12.29 | 0.30 | 0.17 | 0.05 | 0.01 | 0.04 | 0.61 | 0.27 | 0.47 | - | - | - | - | - | - | - | 4.70 | 7.80 | - | - | - | 18.30 | 2.10 | 1.20 | - | - | - | 1.70 | 3.30 | 0.30 | 1.20 | 0.90 | - | - | - | - | 3.49 | 19.58 | 6.00 | 11.43 | 8.74 |
label=this cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          51.30
  command p90 ms                          59.43
  frames                                  60.00
  tick ms/frame                            2.55
  frame p90 ms                             3.98
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         44.44
  interval max ms (median burst)          44.44
  peek_frame ms/frame                      2.16
  peek_draw ms/frame                       1.51
  peek_track ms/frame                      0.34
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.16
  peek_draw ms (worst frame)              20.68
  pd_alpha ms (worst frame)                1.22
  pd_draw ms (worst frame)                19.20
  pd_fill ms (worst frame)                14.69
  fill_source ms (worst frame)             3.60
  ring_raise ms (worst frame)              6.57
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.30
  cloak ms (outside frames)               20.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    51.00
  peek_new ms (outside frames)             3.70
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          12.50
  peek_handover ms (outside frames)       13.00
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.90
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.49
  input->first frame ms                   77.96
  dwm missed vblanks/burst                27.00
  dwm display interval p90 ms             17.16
  dwm gpu busy p90 ms                      1.42
| this cpu8 bd-config bo-config #3 | resize | 51.30 | 59.43 | 60.00 | 2.55 | 3.98 | 3.00 | 5.72 | 5.78 | 44.44 | 44.44 | 2.16 | 1.51 | 0.34 | 0.14 | 0.16 | 20.68 | 1.22 | 19.20 | 14.69 | 3.60 | 6.57 | 0.00 | - | 0.04 | 2.00 | 10.30 | 20.70 | - | - | - | 51.00 | 3.70 | 2.10 | - | - | - | 12.50 | 13.00 | 0.10 | 0.90 | - | - | - | - | - | 3.49 | 77.96 | 27.00 | 17.16 | 1.42 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          35.46
  command p90 ms                          37.24
  frames                                  99.00
  tick ms/frame                            1.21
  frame p90 ms                             1.30
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         23.37
  interval max ms (median burst)          23.37
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
  command ms (WM side)                    35.10
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.80
  restack ms (outside frames)              0.00
  ws_surrogates ms (outside frames)       11.90
  ws_park ms (outside frames)              2.40
  ws_show ms (outside frames)              8.90
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
  dwm missed vblanks/burst                21.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      2.27
| this cpu8 bd-config bo-config #3 | switch | 35.46 | 37.24 | 99.00 | 1.21 | 1.30 | 3.00 | 5.71 | 5.77 | 23.37 | 23.37 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 6.00 | - | - | - | - | 35.10 | - | - | - | - | - | - | - | 0.00 | 0.80 | 0.00 | 11.90 | 2.40 | 8.90 | 0.40 | - | - | 21.00 | 11.44 | 2.27 |
label=this cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          51.14
  command p90 ms                          56.64
  frames                                  61.00
  tick ms/frame                            2.70
  frame p90 ms                             4.57
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.82
  interval p99 ms                         59.99
  interval max ms (median burst)          59.99
  peek_frame ms/frame                      2.16
  peek_draw ms/frame                       1.37
  peek_track ms/frame                      0.36
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.21
  peek_draw ms (worst frame)              14.66
  pd_alpha ms (worst frame)                1.13
  pd_draw ms (worst frame)                13.78
  pd_fill ms (worst frame)                 9.35
  fill_source ms (worst frame)             2.41
  ring_raise ms (worst frame)              4.45
  restack ms (worst frame)                 3.05
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           13.70
  cloak ms (outside frames)               21.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    50.90
  peek_new ms (outside frames)             7.40
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          20.00
  peek_handover ms (outside frames)       15.30
  peek_raise ms (outside frames)           0.50
  surface ms (outside frames)              2.50
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.19
  input->first frame ms                   68.41
  dwm missed vblanks/burst                36.00
  dwm display interval p90 ms             28.58
  dwm gpu busy p90 ms                      8.66
| this cpu8 bd-config bo-config #3 | minimize | 51.14 | 56.64 | 61.00 | 2.70 | 4.57 | 3.00 | 5.71 | 5.82 | 59.99 | 59.99 | 2.16 | 1.37 | 0.36 | 0.14 | 0.21 | 14.66 | 1.13 | 13.78 | 9.35 | 2.41 | 4.45 | 3.05 | - | 0.03 | 2.00 | 13.70 | 21.90 | - | - | - | 50.90 | 7.40 | 1.80 | - | - | - | 20.00 | 15.30 | 0.50 | 2.50 | - | - | - | - | - | 9.19 | 68.41 | 36.00 | 28.58 | 8.66 |
tiles stand-ins:nopeeks cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          15.43
  command p90 ms                          17.27
  frames                                  62.00
  tick ms/frame                            0.71
  frame p90 ms                             1.49
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          9.23
  interval max ms (median burst)           9.23
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
  batch_commit ms/frame                    0.46
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.50
  cloak ms (outside frames)                8.40
  session_begin ms (outside frames)        0.70
  session_overlays ms (outside frames)     3.80
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
  input->anim start ms                     0.33
  input->first frame ms                   16.47
  dwm missed vblanks/burst                 5.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      8.76
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | move | 15.43 | 17.27 | 62.00 | 0.71 | 1.49 | 0.00 | 5.71 | 5.74 | 9.23 | 9.23 | - | - | - | - | - | - | - | - | - | - | - | - | 0.46 | 0.01 | - | 3.50 | 8.40 | 0.70 | 3.80 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.33 | 16.47 | 5.00 | 11.43 | 8.76 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          46.98
  command p90 ms                          49.56
  frames                                  62.00
  tick ms/frame                            1.90
  frame p90 ms                             4.50
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         21.74
  interval max ms (median burst)          21.74
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
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)               23.90
  session_begin ms (outside frames)        2.10
  session_overlays ms (outside frames)    17.60
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
  input->anim start ms                     0.33
  input->first frame ms                   51.86
  dwm missed vblanks/burst                 7.00
  dwm display interval p90 ms             11.43
  dwm gpu busy p90 ms                      4.32
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | resize | 46.98 | 49.56 | 62.00 | 1.90 | 4.50 | 2.00 | 5.71 | 5.77 | 21.74 | 21.74 | - | - | - | - | - | - | - | - | - | - | - | - | 1.12 | 0.04 | - | 4.50 | 23.90 | 2.10 | 17.60 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.33 | 51.86 | 7.00 | 11.43 | 4.32 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.92
  command p90 ms                          43.24
  frames                                  99.00
  tick ms/frame                            1.41
  frame p90 ms                             1.49
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         26.95
  interval max ms (median burst)          27.17
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
  dwm missed vblanks/burst                30.00
  dwm display interval p90 ms             11.44
  dwm gpu busy p90 ms                      5.50
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | switch | 34.92 | 43.24 | 99.00 | 1.41 | 1.49 | 3.00 | 5.71 | 5.76 | 26.95 | 27.17 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.04 | - | 6.10 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 | 30.00 | 11.44 | 5.50 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          29.66
  command p90 ms                          37.10
  frames                                 142.00
  tick ms/frame                            1.40
  frame p90 ms                             3.51
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         46.69
  interval max ms (median burst)          46.69
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
  batch_commit ms/frame                    0.79
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)               22.50
  session_begin ms (outside frames)        2.60
  session_overlays ms (outside frames)    19.40
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
  input->anim start ms                     6.41
  input->first frame ms                   60.30
  dwm missed vblanks/burst                17.00
  dwm display interval p90 ms             11.46
  dwm gpu busy p90 ms                      7.08
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | minimize | 29.66 | 37.10 | 142.00 | 1.40 | 3.51 | 2.00 | 5.71 | 5.77 | 46.69 | 46.69 | - | - | - | - | - | - | - | - | - | - | - | - | 0.79 | 0.02 | 1.00 | 4.90 | 22.50 | 2.60 | 19.40 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.41 | 60.30 | 17.00 | 11.46 | 7.08 |

## move

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 18.66 | 16.47 | 14.35 |
| cpu0 bd-config bo-config | frame p90 ms | 0.54 | 1.31 | 1.36 |
| cpu0 bd-config bo-config | interval p90 ms | 5.74 | 5.74 | 5.74 |
| cpu0 bd-config bo-config | interval p99 ms | 10.49 | 8.00 | 7.77 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 2.10 | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | 0.70 | 0.90 |
| cpu0 bd-config bo-config | restack ms (outside frames) | 0.80 | — | — |
| cpu0 bd-config bo-config | dwm gpu busy p90 ms | 8.47 | 8.64 | 7.27 |
| cpu0 bd-config bo-config | dwm missed vblanks/burst | 5.00 | 10.00 | 5.00 |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 19.58 | 16.81 | 14.60 |
| cpu8 bd-config bo-config | frame p90 ms | 0.53 | 1.50 | 1.58 |
| cpu8 bd-config bo-config | interval p90 ms | 5.76 | 5.74 | 5.75 |
| cpu8 bd-config bo-config | interval p99 ms | 12.29 | 9.19 | 8.84 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 2.20 | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | 0.70 | 1.10 |
| cpu8 bd-config bo-config | restack ms (outside frames) | 0.90 | — | — |
| cpu8 bd-config bo-config | dwm gpu busy p90 ms | 8.69 | 8.76 | 7.33 |
| cpu8 bd-config bo-config | dwm missed vblanks/burst | 6.00 | 5.00 | 4.00 |

### move, cpu0 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 18.09 | 15.53 | 13.68 |
| command p90 ms | 18.72 | 16.39 | 14.41 |
| frames | 63.00 | 62.00 | 63.00 |
| tick ms/frame | 0.39 | 0.63 | 0.71 |
| frame p90 ms | 0.54 | 1.31 | 1.36 |
| frames >2x budget | 0.00 | 0.00 | 0.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.74 | 5.74 | 5.74 |
| interval p99 ms | 10.49 | 8.00 | 7.77 |
| interval max ms (median burst) | 10.49 | 8.00 | 7.77 |
| peek_frame ms/frame | 0.34 | — | — |
| peek_draw ms/frame | 0.16 | — | — |
| peek_track ms/frame | 0.05 | — | — |
| peek_ring ms/frame | 0.01 | — | — |
| peek_backdrop ms/frame | 0.03 | — | — |
| peek_draw ms (worst frame) | 0.66 | — | — |
| pd_alpha ms (worst frame) | 0.24 | — | — |
| pd_draw ms (worst frame) | 0.56 | — | — |
| pd_fill ms (worst frame) | — | — | — |
| fill_source ms (worst frame) | — | — | — |
| ring_raise ms (worst frame) | — | — | — |
| restack ms (worst frame) | — | — | — |
| batch_commit ms/frame | — | 0.41 | 0.43 |
| ovl_region ms/frame | — | 0.00 | 0.00 |
| dwm_flush calls (in frames) | — | — | 1.00 |
| dwm_flush ms (outside frames) | 5.20 | 4.80 | 5.70 |
| cloak ms (outside frames) | 7.90 | 9.20 | 8.80 |
| session_begin ms (outside frames) | — | 0.70 | 0.90 |
| session_overlays ms (outside frames) | — | 3.40 | 2.00 |
| session_overlays ms/frame | — | 0.02 | 0.02 |
| command ms (WM side) | 17.80 | — | — |
| peek_new ms (outside frames) | 2.10 | — | — |
| peek_place ms (outside frames) | 1.10 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| peek_cover ms (outside frames) | 1.20 | — | — |
| peek_handover ms (outside frames) | 2.80 | — | — |
| peek_raise ms (outside frames) | 0.30 | — | — |
| surface ms (outside frames) | 1.00 | — | — |
| restack ms (outside frames) | 0.80 | — | — |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 3.50 | 0.28 | 0.23 |
| input->first frame ms | 18.66 | 16.47 | 14.35 |
| dwm missed vblanks/burst | 5.00 | 10.00 | 5.00 |
| dwm display interval p90 ms | 11.43 | 11.43 | 11.43 |
| dwm gpu busy p90 ms | 8.47 | 8.64 | 7.27 |

### move, cpu8 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 18.69 | 15.70 | 13.91 |
| command p90 ms | 20.45 | 17.81 | 14.92 |
| frames | 63.00 | 62.00 | 63.00 |
| tick ms/frame | 0.42 | 0.71 | 0.78 |
| frame p90 ms | 0.53 | 1.50 | 1.58 |
| frames >2x budget | 0.00 | 0.00 | 0.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.76 | 5.74 | 5.75 |
| interval p99 ms | 12.29 | 9.19 | 8.84 |
| interval max ms (median burst) | 12.29 | 9.19 | 8.84 |
| peek_frame ms/frame | 0.33 | — | — |
| peek_draw ms/frame | 0.17 | — | — |
| peek_track ms/frame | 0.05 | — | — |
| peek_ring ms/frame | 0.01 | — | — |
| peek_backdrop ms/frame | 0.04 | — | — |
| peek_draw ms (worst frame) | 0.62 | — | — |
| pd_alpha ms (worst frame) | 0.27 | — | — |
| pd_draw ms (worst frame) | 0.47 | — | — |
| pd_fill ms (worst frame) | — | — | — |
| fill_source ms (worst frame) | — | — | — |
| ring_raise ms (worst frame) | — | — | — |
| restack ms (worst frame) | — | — | — |
| batch_commit ms/frame | — | 0.46 | 0.49 |
| ovl_region ms/frame | — | 0.01 | 0.00 |
| dwm_flush calls (in frames) | — | — | 1.00 |
| dwm_flush ms (outside frames) | 4.70 | 3.70 | 5.20 |
| cloak ms (outside frames) | 7.80 | 8.90 | 8.60 |
| session_begin ms (outside frames) | — | 0.70 | 1.10 |
| session_overlays ms (outside frames) | — | 4.10 | 1.90 |
| session_overlays ms/frame | — | 0.02 | 0.02 |
| command ms (WM side) | 18.40 | — | — |
| peek_new ms (outside frames) | 2.20 | — | — |
| peek_place ms (outside frames) | 1.20 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| peek_cover ms (outside frames) | 1.60 | — | — |
| peek_handover ms (outside frames) | 3.30 | — | — |
| peek_raise ms (outside frames) | 0.30 | — | — |
| surface ms (outside frames) | 1.20 | — | — |
| restack ms (outside frames) | 0.90 | — | — |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 3.59 | 0.32 | 0.24 |
| input->first frame ms | 19.58 | 16.81 | 14.60 |
| dwm missed vblanks/burst | 6.00 | 5.00 | 4.00 |
| dwm display interval p90 ms | 11.43 | 11.43 | 11.43 |
| dwm gpu busy p90 ms | 8.69 | 8.76 | 7.33 |

## resize

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 64.43 | 44.45 | 34.06 |
| cpu0 bd-config bo-config | frame p90 ms | 3.27 | 4.09 | 4.37 |
| cpu0 bd-config bo-config | interval p90 ms | 5.78 | 5.78 | 5.78 |
| cpu0 bd-config bo-config | interval p99 ms | 36.52 | 20.36 | 25.84 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 3.30 | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | 1.80 | 2.20 |
| cpu0 bd-config bo-config | restack ms (outside frames) | — | — | — |
| cpu0 bd-config bo-config | dwm gpu busy p90 ms | 1.48 | 4.79 | 1.38 |
| cpu0 bd-config bo-config | dwm missed vblanks/burst | 22.00 | 11.00 | 7.00 |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 77.96 | 50.62 | 36.83 |
| cpu8 bd-config bo-config | frame p90 ms | 4.09 | 4.62 | 4.82 |
| cpu8 bd-config bo-config | interval p90 ms | 5.77 | 5.79 | 5.76 |
| cpu8 bd-config bo-config | interval p99 ms | 44.44 | 19.65 | 30.43 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 4.10 | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | 2.00 | 2.80 |
| cpu8 bd-config bo-config | restack ms (outside frames) | — | — | — |
| cpu8 bd-config bo-config | dwm gpu busy p90 ms | 1.56 | 4.32 | 1.56 |
| cpu8 bd-config bo-config | dwm missed vblanks/burst | 25.00 | 7.00 | 7.00 |

### resize, cpu0 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 49.83 | 39.40 | 29.12 |
| command p90 ms | 56.07 | 43.14 | 31.94 |
| frames | 61.00 | 62.00 | 63.00 |
| tick ms/frame | 2.28 | 1.73 | 2.22 |
| frame p90 ms | 3.27 | 4.09 | 4.37 |
| frames >2x budget | 3.00 | 1.00 | 2.00 |
| interval p50 ms | 5.71 | 5.72 | 5.71 |
| interval p90 ms | 5.78 | 5.78 | 5.78 |
| interval p99 ms | 36.52 | 20.36 | 25.84 |
| interval max ms (median burst) | 36.52 | 20.36 | 25.84 |
| peek_frame ms/frame | 1.85 | — | — |
| peek_draw ms/frame | 1.22 | — | — |
| peek_track ms/frame | 0.27 | — | — |
| peek_ring ms/frame | 0.12 | — | — |
| peek_backdrop ms/frame | 0.13 | — | — |
| peek_draw ms (worst frame) | 14.80 | — | — |
| pd_alpha ms (worst frame) | 1.04 | — | — |
| pd_draw ms (worst frame) | 13.68 | — | — |
| pd_fill ms (worst frame) | 10.46 | — | — |
| fill_source ms (worst frame) | 2.17 | — | — |
| ring_raise ms (worst frame) | 5.40 | — | — |
| restack ms (worst frame) | 0.00 | — | — |
| batch_commit ms/frame | — | 0.97 | 1.01 |
| ovl_region ms/frame | 0.03 | 0.03 | 0.03 |
| dwm_flush calls (in frames) | 2.00 | — | 2.00 |
| dwm_flush ms (outside frames) | 9.50 | 3.30 | 4.30 |
| cloak ms (outside frames) | 18.30 | 19.90 | 17.70 |
| session_begin ms (outside frames) | — | 1.80 | 2.20 |
| session_overlays ms (outside frames) | — | 15.60 | 6.80 |
| session_overlays ms/frame | — | 0.06 | 0.04 |
| command ms (WM side) | 47.10 | — | — |
| peek_new ms (outside frames) | 3.30 | — | — |
| peek_place ms (outside frames) | 2.00 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| peek_cover ms (outside frames) | 11.10 | — | — |
| peek_handover ms (outside frames) | 12.50 | — | — |
| peek_raise ms (outside frames) | 0.10 | — | — |
| surface ms (outside frames) | 0.70 | — | — |
| restack ms (outside frames) | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 3.17 | 0.28 | 0.22 |
| input->first frame ms | 64.43 | 44.45 | 34.06 |
| dwm missed vblanks/burst | 22.00 | 11.00 | 7.00 |
| dwm display interval p90 ms | 17.16 | 11.46 | 5.75 |
| dwm gpu busy p90 ms | 1.48 | 4.79 | 1.38 |

### resize, cpu8 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 53.74 | 45.83 | 31.92 |
| command p90 ms | 59.43 | 48.09 | 34.19 |
| frames | 60.00 | 62.00 | 63.00 |
| tick ms/frame | 2.71 | 1.91 | 2.48 |
| frame p90 ms | 4.09 | 4.62 | 4.82 |
| frames >2x budget | 3.00 | 2.00 | 2.00 |
| interval p50 ms | 5.72 | 5.71 | 5.71 |
| interval p90 ms | 5.77 | 5.79 | 5.76 |
| interval p99 ms | 44.44 | 19.65 | 30.43 |
| interval max ms (median burst) | 44.44 | 19.65 | 30.43 |
| peek_frame ms/frame | 2.17 | — | — |
| peek_draw ms/frame | 1.50 | — | — |
| peek_track ms/frame | 0.34 | — | — |
| peek_ring ms/frame | 0.15 | — | — |
| peek_backdrop ms/frame | 0.16 | — | — |
| peek_draw ms (worst frame) | 20.68 | — | — |
| pd_alpha ms (worst frame) | 1.22 | — | — |
| pd_draw ms (worst frame) | 19.20 | — | — |
| pd_fill ms (worst frame) | 14.69 | — | — |
| fill_source ms (worst frame) | 5.04 | — | — |
| ring_raise ms (worst frame) | 6.00 | — | — |
| restack ms (worst frame) | 0.00 | — | — |
| batch_commit ms/frame | — | 1.11 | 1.13 |
| ovl_region ms/frame | 0.05 | 0.04 | 0.04 |
| dwm_flush calls (in frames) | 2.00 | — | 2.00 |
| dwm_flush ms (outside frames) | 10.20 | 4.50 | 4.50 |
| cloak ms (outside frames) | 21.00 | 22.90 | 18.90 |
| session_begin ms (outside frames) | — | 2.00 | 2.80 |
| session_overlays ms (outside frames) | — | 17.60 | 7.40 |
| session_overlays ms/frame | — | 0.06 | 0.03 |
| command ms (WM side) | 53.30 | — | — |
| peek_new ms (outside frames) | 4.10 | — | — |
| peek_place ms (outside frames) | 2.10 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| peek_cover ms (outside frames) | 12.60 | — | — |
| peek_handover ms (outside frames) | 13.00 | — | — |
| peek_raise ms (outside frames) | 0.10 | — | — |
| surface ms (outside frames) | 0.90 | — | — |
| restack ms (outside frames) | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 3.63 | 0.33 | 0.25 |
| input->first frame ms | 77.96 | 50.62 | 36.83 |
| dwm missed vblanks/burst | 25.00 | 7.00 | 7.00 |
| dwm display interval p90 ms | 17.16 | 11.43 | 5.74 |
| dwm gpu busy p90 ms | 1.56 | 4.32 | 1.56 |

## switch

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 0.00 | 0.00 | 0.00 |
| cpu0 bd-config bo-config | frame p90 ms | 1.09 | 1.24 | 2.64 |
| cpu0 bd-config bo-config | interval p90 ms | 5.76 | 5.76 | 5.74 |
| cpu0 bd-config bo-config | interval p99 ms | 21.66 | 24.29 | 54.34 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 0.00 | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | — | 0.00 |
| cpu0 bd-config bo-config | restack ms (outside frames) | 0.00 | — | — |
| cpu0 bd-config bo-config | dwm gpu busy p90 ms | 5.31 | 2.44 | 3.12 |
| cpu0 bd-config bo-config | dwm missed vblanks/burst | 18.00 | 19.00 | 8.00 |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 0.00 | 0.00 | 0.00 |
| cpu8 bd-config bo-config | frame p90 ms | 1.30 | 1.38 | 3.22 |
| cpu8 bd-config bo-config | interval p90 ms | 5.76 | 5.76 | 5.75 |
| cpu8 bd-config bo-config | interval p99 ms | 23.19 | 25.98 | 59.20 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 0.00 | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | — | 0.00 |
| cpu8 bd-config bo-config | restack ms (outside frames) | 0.00 | — | — |
| cpu8 bd-config bo-config | dwm gpu busy p90 ms | 5.30 | 2.69 | 3.76 |
| cpu8 bd-config bo-config | dwm missed vblanks/burst | 25.00 | 30.00 | 11.00 |

### switch, cpu0 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 30.46 | 31.08 | 20.24 |
| command p90 ms | 33.74 | 32.92 | 21.59 |
| frames | 99.00 | 98.00 | 97.00 |
| tick ms/frame | 1.12 | 1.26 | 1.70 |
| frame p90 ms | 1.09 | 1.24 | 2.64 |
| frames >2x budget | 3.00 | 3.00 | 1.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.76 | 5.76 | 5.74 |
| interval p99 ms | 21.66 | 24.29 | 54.34 |
| interval max ms (median burst) | 21.66 | 24.29 | 55.25 |
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
| batch_commit ms/frame | — | 0.00 | 0.65 |
| ovl_region ms/frame | 0.03 | 0.03 | 0.01 |
| dwm_flush calls (in frames) | — | — | 3.00 |
| dwm_flush ms (outside frames) | 5.50 | 6.40 | 5.60 |
| cloak ms (outside frames) | 0.00 | — | 0.00 |
| session_begin ms (outside frames) | — | — | 0.00 |
| session_overlays ms (outside frames) | — | 0.00 | 0.00 |
| session_overlays ms/frame | — | 0.00 | 0.00 |
| command ms (WM side) | 30.20 | — | — |
| peek_new ms (outside frames) | 0.00 | — | — |
| peek_place ms (outside frames) | 0.00 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| peek_cover ms (outside frames) | 0.00 | — | — |
| peek_handover ms (outside frames) | 0.00 | — | — |
| peek_raise ms (outside frames) | 0.00 | — | — |
| surface ms (outside frames) | 0.60 | — | — |
| restack ms (outside frames) | 0.00 | — | — |
| ws_surrogates ms (outside frames) | 9.90 | — | — |
| ws_park ms (outside frames) | 2.10 | — | — |
| ws_show ms (outside frames) | 8.10 | — | — |
| ws_start ms (outside frames) | 0.40 | — | — |
| input->anim start ms | 0.00 | 0.00 | 0.00 |
| input->first frame ms | 0.00 | 0.00 | 0.00 |
| dwm missed vblanks/burst | 18.00 | 19.00 | 8.00 |
| dwm display interval p90 ms | 11.44 | 11.43 | 5.77 |
| dwm gpu busy p90 ms | 5.31 | 2.44 | 3.12 |

### switch, cpu8 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 34.64 | 34.22 | 22.13 |
| command p90 ms | 36.81 | 36.15 | 23.02 |
| frames | 99.00 | 99.00 | 97.00 |
| tick ms/frame | 1.25 | 1.40 | 1.93 |
| frame p90 ms | 1.30 | 1.38 | 3.22 |
| frames >2x budget | 3.00 | 3.00 | 1.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.76 | 5.76 | 5.75 |
| interval p99 ms | 23.19 | 25.98 | 59.20 |
| interval max ms (median burst) | 23.37 | 26.11 | 59.21 |
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
| ovl_region ms/frame | 0.03 | 0.04 | 0.02 |
| dwm_flush calls (in frames) | — | — | 3.00 |
| dwm_flush ms (outside frames) | 5.90 | 5.40 | 5.60 |
| cloak ms (outside frames) | 0.00 | — | 0.00 |
| session_begin ms (outside frames) | — | — | 0.00 |
| session_overlays ms (outside frames) | — | 0.00 | 0.00 |
| session_overlays ms/frame | — | 0.00 | 0.00 |
| command ms (WM side) | 34.30 | — | — |
| peek_new ms (outside frames) | 0.00 | — | — |
| peek_place ms (outside frames) | 0.00 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| peek_cover ms (outside frames) | 0.00 | — | — |
| peek_handover ms (outside frames) | 0.00 | — | — |
| peek_raise ms (outside frames) | 0.00 | — | — |
| surface ms (outside frames) | 0.70 | — | — |
| restack ms (outside frames) | 0.00 | — | — |
| ws_surrogates ms (outside frames) | 11.80 | — | — |
| ws_park ms (outside frames) | 2.40 | — | — |
| ws_show ms (outside frames) | 8.60 | — | — |
| ws_start ms (outside frames) | 0.40 | — | — |
| input->anim start ms | 0.00 | 0.00 | 0.00 |
| input->first frame ms | 0.00 | 0.00 | 0.00 |
| dwm missed vblanks/burst | 25.00 | 30.00 | 11.00 |
| dwm display interval p90 ms | 11.44 | 11.44 | 11.42 |
| dwm gpu busy p90 ms | 5.30 | 2.69 | 3.76 |

## minimize

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 60.88 | 57.00 | 45.84 |
| cpu0 bd-config bo-config | frame p90 ms | 3.69 | 3.25 | 3.98 |
| cpu0 bd-config bo-config | interval p90 ms | 5.77 | 5.76 | 5.76 |
| cpu0 bd-config bo-config | interval p99 ms | 52.13 | 49.03 | 34.95 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 7.20 | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | 2.20 | 2.30 |
| cpu0 bd-config bo-config | restack ms (outside frames) | — | — | — |
| cpu0 bd-config bo-config | dwm gpu busy p90 ms | 3.09 | 6.56 | 6.58 |
| cpu0 bd-config bo-config | dwm missed vblanks/burst | 28.00 | 15.00 | 14.00 |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 72.03 | 62.78 | 46.68 |
| cpu8 bd-config bo-config | frame p90 ms | 4.47 | 3.56 | 4.38 |
| cpu8 bd-config bo-config | interval p90 ms | 5.79 | 5.77 | 5.77 |
| cpu8 bd-config bo-config | interval p99 ms | 61.17 | 54.32 | 36.73 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 9.20 | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | 2.60 | 2.60 |
| cpu8 bd-config bo-config | restack ms (outside frames) | — | — | — |
| cpu8 bd-config bo-config | dwm gpu busy p90 ms | 8.66 | 7.08 | 6.60 |
| cpu8 bd-config bo-config | dwm missed vblanks/burst | 35.00 | 17.00 | 13.00 |

### minimize, cpu0 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 49.59 | 25.22 | 17.94 |
| command p90 ms | 51.01 | 29.20 | 20.85 |
| frames | 62.00 | 143.00 | 150.00 |
| tick ms/frame | 2.17 | 1.17 | 1.56 |
| frame p90 ms | 3.69 | 3.25 | 3.98 |
| frames >2x budget | 3.00 | 0.00 | 3.00 |
| interval p50 ms | 5.72 | 5.71 | 5.72 |
| interval p90 ms | 5.77 | 5.76 | 5.76 |
| interval p99 ms | 52.13 | 49.03 | 34.95 |
| interval max ms (median burst) | 52.13 | 49.03 | 34.95 |
| peek_frame ms/frame | 1.79 | — | — |
| peek_draw ms/frame | 1.10 | — | — |
| peek_track ms/frame | 0.32 | — | — |
| peek_ring ms/frame | 0.13 | — | — |
| peek_backdrop ms/frame | 0.19 | — | — |
| peek_draw ms (worst frame) | 11.36 | — | — |
| pd_alpha ms (worst frame) | 0.76 | — | — |
| pd_draw ms (worst frame) | 10.24 | — | — |
| pd_fill ms (worst frame) | 7.00 | — | — |
| fill_source ms (worst frame) | 1.98 | — | — |
| ring_raise ms (worst frame) | 3.30 | — | — |
| restack ms (worst frame) | 2.80 | — | — |
| batch_commit ms/frame | — | 0.64 | 0.76 |
| ovl_region ms/frame | 0.02 | 0.02 | 0.03 |
| dwm_flush calls (in frames) | 2.00 | 1.00 | 4.00 |
| dwm_flush ms (outside frames) | 12.50 | 4.40 | 5.50 |
| cloak ms (outside frames) | 19.70 | 23.40 | 18.30 |
| session_begin ms (outside frames) | — | 2.20 | 2.30 |
| session_overlays ms (outside frames) | — | 17.20 | 7.10 |
| session_overlays ms/frame | — | 0.04 | 0.02 |
| command ms (WM side) | 47.80 | — | — |
| peek_new ms (outside frames) | 7.20 | — | — |
| peek_place ms (outside frames) | 1.80 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| peek_cover ms (outside frames) | 17.80 | — | — |
| peek_handover ms (outside frames) | 12.60 | — | — |
| peek_raise ms (outside frames) | 0.40 | — | — |
| surface ms (outside frames) | 3.10 | — | — |
| restack ms (outside frames) | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 8.01 | 5.73 | 6.43 |
| input->first frame ms | 60.88 | 57.00 | 45.84 |
| dwm missed vblanks/burst | 28.00 | 15.00 | 14.00 |
| dwm display interval p90 ms | 22.84 | 11.46 | 11.46 |
| dwm gpu busy p90 ms | 3.09 | 6.56 | 6.58 |

### minimize, cpu8 bd-config bo-config

| metric | this | stand-ins:nopeeks | main |
| --- | --- | --- | --- |
| runs | 3 | 3 | 3 |
| command p50 ms | 53.14 | 30.64 | 21.09 |
| command p90 ms | 58.46 | 37.10 | 24.44 |
| frames | 61.00 | 143.00 | 148.00 |
| tick ms/frame | 2.52 | 1.40 | 1.68 |
| frame p90 ms | 4.47 | 3.56 | 4.38 |
| frames >2x budget | 3.00 | 2.00 | 3.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.79 | 5.77 | 5.77 |
| interval p99 ms | 61.17 | 54.32 | 36.73 |
| interval max ms (median burst) | 61.17 | 54.32 | 43.08 |
| peek_frame ms/frame | 2.16 | — | — |
| peek_draw ms/frame | 1.33 | — | — |
| peek_track ms/frame | 0.39 | — | — |
| peek_ring ms/frame | 0.14 | — | — |
| peek_backdrop ms/frame | 0.23 | — | — |
| peek_draw ms (worst frame) | 14.66 | — | — |
| pd_alpha ms (worst frame) | 1.13 | — | — |
| pd_draw ms (worst frame) | 13.78 | — | — |
| pd_fill ms (worst frame) | 9.35 | — | — |
| fill_source ms (worst frame) | 2.60 | — | — |
| ring_raise ms (worst frame) | 4.23 | — | — |
| restack ms (worst frame) | 2.82 | — | — |
| batch_commit ms/frame | — | 0.78 | 0.81 |
| ovl_region ms/frame | 0.03 | 0.02 | 0.03 |
| dwm_flush calls (in frames) | 2.00 | 1.00 | 4.00 |
| dwm_flush ms (outside frames) | 13.50 | 4.70 | 4.00 |
| cloak ms (outside frames) | 21.90 | 24.10 | 17.50 |
| session_begin ms (outside frames) | — | 2.60 | 2.60 |
| session_overlays ms (outside frames) | — | 19.80 | 8.40 |
| session_overlays ms/frame | — | 0.04 | 0.02 |
| command ms (WM side) | 52.90 | — | — |
| peek_new ms (outside frames) | 9.20 | — | — |
| peek_place ms (outside frames) | 1.80 | — | — |
| pd_fill ms (outside frames) | — | — | — |
| fill_source ms (outside frames) | — | — | — |
| ring_raise ms (outside frames) | — | — | — |
| peek_cover ms (outside frames) | 20.00 | — | — |
| peek_handover ms (outside frames) | 15.70 | — | — |
| peek_raise ms (outside frames) | 0.40 | — | — |
| surface ms (outside frames) | 2.50 | — | — |
| restack ms (outside frames) | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — |
| ws_park ms (outside frames) | — | — | — |
| ws_show ms (outside frames) | — | — | — |
| ws_start ms (outside frames) | — | — | — |
| input->anim start ms | 9.28 | 6.41 | 7.38 |
| input->first frame ms | 72.03 | 62.78 | 46.68 |
| dwm missed vblanks/burst | 35.00 | 17.00 | 13.00 |
| dwm display interval p90 ms | 22.91 | 11.46 | 17.13 |
| dwm gpu busy p90 ms | 8.66 | 7.08 | 6.60 |
