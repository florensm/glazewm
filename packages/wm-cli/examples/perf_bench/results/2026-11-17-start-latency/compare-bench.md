Branch comparison, 2026-11-17T11:02:57: this = c6068c12, against origin/tryout/start-latency (ffb2d88), origin/tryout/stand-ins (65a60de), origin/main (cfd3cef); builds this, start-latency, stand-ins:nopeeks (GLAZEWM_ISOLATE=nopeeks), stand-ins, stand-ins:morepeeks (GLAZEWM_ISOLATE=morepeeks), main. Target chrome, 3 round(s) of 10 bursts per scenario, build and setting, IPC load 0, CPU load 0,8 thread(s) of 16, without DWM timing, tile order Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot, launched none (0 window(s)).
tiles this cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-this-move-mid.png (mid at 61 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-this-move-rest.png
label=this cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.22
  command p90 ms                          18.46
  frames                                  63.00
  tick ms/frame                            0.37
  frame p90 ms                             0.48
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.29
  interval max ms (median burst)          10.29
  peek_frame ms/frame                      0.35
  peek_draw ms/frame                       0.15
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.73
  pd_alpha ms (worst frame)                0.41
  pd_draw ms (worst frame)                 0.48
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)                7.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    15.90
  peek_new ms (outside frames)             2.00
  peek_place ms (outside frames)           0.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.00
  peek_handover ms (outside frames)        2.60
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.10
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.89
  input->first frame ms                   17.21
| this cpu0 bd-config bo-config #1 | move | 16.22 | 18.46 | 63.00 | 0.37 | 0.48 | 0.00 | 5.71 | 5.74 | 10.29 | 10.29 | 0.35 | 0.15 | 0.05 | 0.01 | 0.03 | 0.73 | 0.41 | 0.48 | - | - | - | - | - | - | - | 4.70 | 7.20 | - | - | - | 15.90 | 2.00 | 0.90 | - | - | - | 1.00 | 2.60 | 0.30 | 1.10 | - | - | - | - | - | 2.89 | 17.21 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-this-resize-mid.png (mid at 71 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-this-resize-rest.png
label=this cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          46.64
  command p90 ms                          55.90
  frames                                  61.00
  tick ms/frame                            2.39
  frame p90 ms                             3.53
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.79
  interval p99 ms                         40.26
  interval max ms (median burst)          40.26
  peek_frame ms/frame                      2.01
  peek_draw ms/frame                       1.25
  peek_track ms/frame                      0.29
  peek_ring ms/frame                       0.17
  peek_backdrop ms/frame                   0.13
  peek_draw ms (worst frame)              17.51
  pd_alpha ms (worst frame)                1.14
  pd_draw ms (worst frame)                16.66
  pd_fill ms (worst frame)                13.21
  fill_source ms (worst frame)             4.14
  ring_raise ms (worst frame)              4.97
  restack ms (worst frame)                 2.88
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            7.30
  cloak ms (outside frames)               16.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    46.30
  peek_new ms (outside frames)             3.30
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          11.60
  peek_handover ms (outside frames)       10.70
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.27
  input->first frame ms                   71.12
| this cpu0 bd-config bo-config #1 | resize | 46.64 | 55.90 | 61.00 | 2.39 | 3.53 | 3.00 | 5.72 | 5.79 | 40.26 | 40.26 | 2.01 | 1.25 | 0.29 | 0.17 | 0.13 | 17.51 | 1.14 | 16.66 | 13.21 | 4.14 | 4.97 | 2.88 | - | 0.05 | 2.00 | 7.30 | 16.20 | - | - | - | 46.30 | 3.30 | 1.90 | - | - | - | 11.60 | 10.70 | 0.10 | 0.70 | - | - | - | - | - | 3.27 | 71.12 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-this-switch-mid.png (mid at 71 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-this-switch-rest.png
label=this cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          26.40
  command p90 ms                          28.59
  frames                                  99.00
  tick ms/frame                            1.12
  frame p90 ms                             1.08
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         18.95
  interval max ms (median burst)          22.42
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
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    26.10
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
  ws_surrogates ms (outside frames)       10.30
  ws_park ms (outside frames)              2.00
  ws_show ms (outside frames)              8.00
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| this cpu0 bd-config bo-config #1 | switch | 26.40 | 28.59 | 99.00 | 1.12 | 1.08 | 3.00 | 5.71 | 5.76 | 18.95 | 22.42 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | - | 5.40 | - | - | - | - | 26.10 | - | - | - | - | - | - | - | 0.00 | 0.50 | - | 10.30 | 2.00 | 8.00 | 0.40 | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-this-minimize-mid.png (mid at 63 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-this-minimize-rest.png
label=this cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          50.38
  command p90 ms                          58.00
  frames                                  61.00
  tick ms/frame                            2.33
  frame p90 ms                             3.49
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         64.93
  interval max ms (median burst)          64.93
  peek_frame ms/frame                      1.88
  peek_draw ms/frame                       1.20
  peek_track ms/frame                      0.35
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.21
  peek_draw ms (worst frame)              17.31
  pd_alpha ms (worst frame)                0.81
  pd_draw ms (worst frame)                16.50
  pd_fill ms (worst frame)                13.15
  fill_source ms (worst frame)             3.60
  ring_raise ms (worst frame)              4.34
  restack ms (worst frame)                 2.63
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.10
  cloak ms (outside frames)               19.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    50.10
  peek_new ms (outside frames)            12.10
  peek_place ms (outside frames)           1.70
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          23.00
  peek_handover ms (outside frames)       15.40
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              4.70
  restack ms (outside frames)              9.20
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.95
  input->first frame ms                   70.99
| this cpu0 bd-config bo-config #1 | minimize | 50.38 | 58.00 | 61.00 | 2.33 | 3.49 | 3.00 | 5.72 | 5.78 | 64.93 | 64.93 | 1.88 | 1.20 | 0.35 | 0.12 | 0.21 | 17.31 | 0.81 | 16.50 | 13.15 | 3.60 | 4.34 | 2.63 | - | 0.02 | 2.00 | 11.10 | 19.00 | - | - | - | 50.10 | 12.10 | 1.70 | - | - | - | 23.00 | 15.40 | 0.30 | 4.70 | 9.20 | - | - | - | - | 9.95 | 70.99 |
tiles start-latency cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-start-latency-default-move-mid.png (mid at 69 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-start-latency-default-move-rest.png
label=start-latency cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.59
  command p90 ms                          20.51
  frames                                  63.00
  tick ms/frame                            0.84
  frame p90 ms                             0.87
  frames >2x budget                        0.00
  interval p50 ms                          5.72
  interval p90 ms                          5.75
  interval p99 ms                         10.95
  interval max ms (median burst)          10.95
  peek_frame ms/frame                      0.39
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.02
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.78
  pd_alpha ms (worst frame)                0.41
  pd_draw ms (worst frame)                 0.46
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.60
  cloak ms (outside frames)                7.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    19.30
  peek_new ms (outside frames)             5.40
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.10
  peek_handover ms (outside frames)        2.50
  peek_raise ms (outside frames)           0.50
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     4.28
  input->first frame ms                   21.55
| start-latency cpu0 bd-config bo-config #1 | move | 19.59 | 20.51 | 63.00 | 0.84 | 0.87 | 0.00 | 5.72 | 5.75 | 10.95 | 10.95 | 0.39 | 0.16 | 0.05 | 0.02 | 0.03 | 0.78 | 0.41 | 0.46 | - | - | - | - | - | - | - | 4.60 | 7.80 | - | 0.00 | 0.00 | 19.30 | 5.40 | 1.20 | - | - | - | 1.10 | 2.50 | 0.50 | - | - | - | - | - | - | 4.28 | 21.55 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-start-latency-default-resize-mid.png (mid at 79 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-start-latency-default-resize-rest.png
label=start-latency cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          63.56
  command p90 ms                          69.01
  frames                                  61.00
  tick ms/frame                            3.33
  frame p90 ms                             3.27
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         51.32
  interval max ms (median burst)          51.32
  peek_frame ms/frame                      2.25
  peek_draw ms/frame                       1.27
  peek_track ms/frame                      0.31
  peek_ring ms/frame                       0.17
  peek_backdrop ms/frame                   0.15
  peek_draw ms (worst frame)              23.73
  pd_alpha ms (worst frame)                0.87
  pd_draw ms (worst frame)                22.91
  pd_fill ms (worst frame)                19.74
  fill_source ms (worst frame)             5.66
  ring_raise ms (worst frame)              8.47
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            8.70
  cloak ms (outside frames)               18.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    63.30
  peek_new ms (outside frames)            16.80
  peek_place ms (outside frames)           4.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           9.90
  peek_handover ms (outside frames)       13.60
  peek_raise ms (outside frames)           2.30
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     4.42
  input->first frame ms                   87.22
| start-latency cpu0 bd-config bo-config #1 | resize | 63.56 | 69.01 | 61.00 | 3.33 | 3.27 | 4.00 | 5.72 | 5.77 | 51.32 | 51.32 | 2.25 | 1.27 | 0.31 | 0.17 | 0.15 | 23.73 | 0.87 | 22.91 | 19.74 | 5.66 | 8.47 | - | - | 0.05 | 1.00 | 8.70 | 18.40 | - | 0.00 | 0.00 | 63.30 | 16.80 | 4.00 | - | - | - | 9.90 | 13.60 | 2.30 | - | - | - | - | - | - | 4.42 | 87.22 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-start-latency-default-switch-mid.png (mid at 77 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-start-latency-default-switch-rest.png
label=start-latency cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.23
  command p90 ms                          31.46
  frames                                  99.00
  tick ms/frame                            1.40
  frame p90 ms                             1.22
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         23.75
  interval max ms (median burst)          25.56
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
  dwm_flush ms (outside frames)            6.30
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    29.90
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)       12.80
  ws_park ms (outside frames)              1.90
  ws_show ms (outside frames)              9.70
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| start-latency cpu0 bd-config bo-config #1 | switch | 30.23 | 31.46 | 99.00 | 1.40 | 1.22 | 3.00 | 5.71 | 5.75 | 23.75 | 25.56 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.02 | - | 6.30 | - | - | 0.00 | 0.00 | 29.90 | - | - | - | - | - | - | - | 0.00 | - | - | 12.80 | 1.90 | 9.70 | 0.40 | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-start-latency-default-minimize-mid.png (mid at 68 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-start-latency-default-minimize-rest.png
label=start-latency cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          58.72
  command p90 ms                          64.18
  frames                                  60.00
  tick ms/frame                            3.03
  frame p90 ms                             4.21
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.68
  interval p99 ms                         80.90
  interval max ms (median burst)          80.90
  peek_frame ms/frame                      2.19
  peek_draw ms/frame                       1.29
  peek_track ms/frame                      0.33
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.19
  peek_draw ms (worst frame)              26.52
  pd_alpha ms (worst frame)                0.73
  pd_draw ms (worst frame)                25.91
  pd_fill ms (worst frame)                20.97
  fill_source ms (worst frame)             6.80
  ring_raise ms (worst frame)              7.91
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.50
  cloak ms (outside frames)               16.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    58.40
  peek_new ms (outside frames)            27.50
  peek_place ms (outside frames)           6.70
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          20.50
  peek_handover ms (outside frames)       14.70
  peek_raise ms (outside frames)           1.50
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.41
  input->first frame ms                   90.10
| start-latency cpu0 bd-config bo-config #1 | minimize | 58.72 | 64.18 | 60.00 | 3.03 | 4.21 | 4.00 | 5.72 | 7.68 | 80.90 | 80.90 | 2.19 | 1.29 | 0.33 | 0.14 | 0.19 | 26.52 | 0.73 | 25.91 | 20.97 | 6.80 | 7.91 | - | - | 0.02 | 2.00 | 9.50 | 16.40 | - | 0.00 | 0.00 | 58.40 | 27.50 | 6.70 | - | - | - | 20.50 | 14.70 | 1.50 | - | - | - | - | - | - | 8.41 | 90.10 |
tiles stand-ins:nopeeks cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-move-mid.png (mid at 75 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-move-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.91
  command p90 ms                          19.90
  frames                                  62.00
  tick ms/frame                            0.53
  frame p90 ms                             1.09
  frames >2x budget                        0.00
  interval p50 ms                          5.72
  interval p90 ms                          5.74
  interval p99 ms                          7.47
  interval max ms (median burst)           7.47
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
  batch_commit ms/frame                    0.31
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)                9.60
  session_begin ms (outside frames)        0.70
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
  input->anim start ms                     0.28
  input->first frame ms                   17.58
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | move | 16.91 | 19.90 | 62.00 | 0.53 | 1.09 | 0.00 | 5.72 | 5.74 | 7.47 | 7.47 | - | - | - | - | - | - | - | - | - | - | - | - | 0.31 | 0.00 | - | 5.40 | 9.60 | 0.70 | 4.50 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.28 | 17.58 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-resize-mid.png (mid at 64 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-resize-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          38.94
  command p90 ms                          43.21
  frames                                  63.00
  tick ms/frame                            1.92
  frame p90 ms                             4.39
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         30.90
  interval max ms (median burst)          30.90
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
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)               20.10
  session_begin ms (outside frames)        1.80
  session_overlays ms (outside frames)    14.90
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
  input->first frame ms                   42.90
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | resize | 38.94 | 43.21 | 63.00 | 1.92 | 4.39 | 2.00 | 5.71 | 5.78 | 30.90 | 30.90 | - | - | - | - | - | - | - | - | - | - | - | - | 0.95 | 0.06 | - | 5.10 | 20.10 | 1.80 | 14.90 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.28 | 42.90 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-switch-mid.png (mid at 77 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-switch-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.21
  command p90 ms                          31.55
  frames                                  99.00
  tick ms/frame                            1.23
  frame p90 ms                             1.16
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         22.92
  interval max ms (median burst)          22.92
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
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | switch | 30.21 | 31.55 | 99.00 | 1.23 | 1.16 | 3.00 | 5.71 | 5.76 | 22.92 | 22.92 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 6.00 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-minimize-mid.png (mid at 72 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-nopeeks-minimize-rest.png
label=stand-ins:nopeeks cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          21.60
  command p90 ms                          24.65
  frames                                 144.00
  tick ms/frame                            1.16
  frame p90 ms                             3.16
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         54.17
  interval max ms (median burst)          54.17
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
  batch_commit ms/frame                    0.63
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            3.90
  cloak ms (outside frames)               25.20
  session_begin ms (outside frames)        5.20
  session_overlays ms (outside frames)    17.80
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
  input->anim start ms                     5.84
  input->first frame ms                   61.61
| stand-ins:nopeeks cpu0 bd-config bo-config #1 | minimize | 21.60 | 24.65 | 144.00 | 1.16 | 3.16 | 0.00 | 5.71 | 5.77 | 54.17 | 54.17 | - | - | - | - | - | - | - | - | - | - | - | - | 0.63 | 0.02 | 1.00 | 3.90 | 25.20 | 5.20 | 17.80 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 5.84 | 61.61 |
tiles stand-ins cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-default-move-mid.png (mid at 71 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-default-move-rest.png
label=stand-ins cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.76
  command p90 ms                          20.56
  frames                                  63.00
  tick ms/frame                            0.84
  frame p90 ms                             0.89
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         10.87
  interval max ms (median burst)          10.87
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
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)                7.70
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
| stand-ins cpu0 bd-config bo-config #1 | move | 19.76 | 20.56 | 63.00 | 0.84 | 0.89 | 0.00 | 5.71 | 5.76 | 10.87 | 10.87 | 0.38 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.80 | 7.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-default-resize-mid.png (mid at 69 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-default-resize-rest.png
label=stand-ins cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          64.77
  command p90 ms                          73.83
  frames                                  60.00
  tick ms/frame                            3.44
  frame p90 ms                             3.80
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          6.11
  interval p99 ms                         49.26
  interval max ms (median burst)          49.26
  peek_frame ms/frame                      2.38
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
  dwm_flush ms (outside frames)            7.80
  cloak ms (outside frames)               16.70
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
| stand-ins cpu0 bd-config bo-config #1 | resize | 64.77 | 73.83 | 60.00 | 3.44 | 3.80 | 5.00 | 5.72 | 6.11 | 49.26 | 49.26 | 2.38 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 7.80 | 16.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-default-switch-mid.png (mid at 75 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-default-switch-rest.png
label=stand-ins cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.22
  command p90 ms                          31.09
  frames                                  99.00
  tick ms/frame                            1.40
  frame p90 ms                             1.17
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         23.62
  interval max ms (median burst)          25.31
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
| stand-ins cpu0 bd-config bo-config #1 | switch | 30.22 | 31.09 | 99.00 | 1.40 | 1.17 | 3.00 | 5.71 | 5.76 | 23.62 | 25.31 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 6.20 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-default-minimize-mid.png (mid at 66 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-default-minimize-rest.png
label=stand-ins cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          61.68
  command p90 ms                          64.06
  frames                                  60.00
  tick ms/frame                            3.20
  frame p90 ms                             3.82
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          7.47
  interval p99 ms                         73.01
  interval max ms (median burst)          73.01
  peek_frame ms/frame                      2.21
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
  dwm_flush ms (outside frames)            7.50
  cloak ms (outside frames)               14.00
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
| stand-ins cpu0 bd-config bo-config #1 | minimize | 61.68 | 64.06 | 60.00 | 3.20 | 3.82 | 3.00 | 5.72 | 7.47 | 73.01 | 73.01 | 2.21 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 7.50 | 14.00 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles stand-ins:morepeeks cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-move-mid.png (mid at 66 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-move-rest.png
label=stand-ins:morepeeks cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.52
  command p90 ms                          20.86
  frames                                  63.00
  tick ms/frame                            0.83
  frame p90 ms                             0.92
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         10.91
  interval max ms (median burst)          10.91
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
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)                7.70
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
| stand-ins:morepeeks cpu0 bd-config bo-config #1 | move | 19.52 | 20.86 | 63.00 | 0.83 | 0.92 | 0.00 | 5.71 | 5.75 | 10.91 | 10.91 | 0.38 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.50 | 7.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-resize-mid.png (mid at 77 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-resize-rest.png
label=stand-ins:morepeeks cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          62.68
  command p90 ms                          65.30
  frames                                  61.00
  tick ms/frame                            3.41
  frame p90 ms                             3.17
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         45.67
  interval max ms (median burst)          45.67
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
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)               16.50
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
| stand-ins:morepeeks cpu0 bd-config bo-config #1 | resize | 62.68 | 65.30 | 61.00 | 3.41 | 3.17 | 4.00 | 5.72 | 5.77 | 45.67 | 45.67 | 2.15 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 4.70 | 16.50 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-switch-mid.png (mid at 62 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-switch-rest.png
label=stand-ins:morepeeks cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.65
  command p90 ms                          29.78
  frames                                  99.00
  tick ms/frame                            1.39
  frame p90 ms                             1.11
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         21.37
  interval max ms (median burst)          24.68
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
| stand-ins:morepeeks cpu0 bd-config bo-config #1 | switch | 29.65 | 29.78 | 99.00 | 1.39 | 1.11 | 3.00 | 5.71 | 5.75 | 21.37 | 24.68 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 5.10 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-minimize-mid.png (mid at 68 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-stand-ins-morepeeks-minimize-rest.png
label=stand-ins:morepeeks cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          57.43
  command p90 ms                          62.25
  frames                                  60.00
  tick ms/frame                            3.25
  frame p90 ms                             4.36
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.04
  interval p99 ms                         72.42
  interval max ms (median burst)          72.42
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
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           12.30
  cloak ms (outside frames)               14.70
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
| stand-ins:morepeeks cpu0 bd-config bo-config #1 | minimize | 57.43 | 62.25 | 60.00 | 3.25 | 4.36 | 4.00 | 5.72 | 7.04 | 72.42 | 72.42 | 2.24 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 12.30 | 14.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles main cpu0 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-main-default-move-mid.png (mid at 75 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-main-default-move-rest.png
label=main cpu0 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          14.02
  command p90 ms                          14.68
  frames                                  63.00
  tick ms/frame                            0.62
  frame p90 ms                             1.12
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          7.59
  interval max ms (median burst)           7.59
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
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)                8.10
  session_begin ms (outside frames)        0.90
  session_overlays ms (outside frames)     3.20
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
  input->first frame ms                   15.06
| main cpu0 bd-config bo-config #1 | move | 14.02 | 14.68 | 63.00 | 0.62 | 1.12 | 0.00 | 5.71 | 5.74 | 7.59 | 7.59 | - | - | - | - | - | - | - | - | - | - | - | - | 0.35 | 0.00 | 1.00 | 4.90 | 8.10 | 0.90 | 3.20 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.23 | 15.06 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-main-default-resize-mid.png (mid at 70 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-main-default-resize-rest.png
label=main cpu0 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.55
  command p90 ms                          32.76
  frames                                  63.00
  tick ms/frame                            2.28
  frame p90 ms                             3.99
  frames >2x budget                        2.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         35.55
  interval max ms (median burst)          35.55
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
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)               18.10
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
  input->first frame ms                   35.17
| main cpu0 bd-config bo-config #1 | resize | 30.55 | 32.76 | 63.00 | 2.28 | 3.99 | 2.00 | 5.72 | 5.76 | 35.55 | 35.55 | - | - | - | - | - | - | - | - | - | - | - | - | 0.96 | 0.05 | 2.00 | 5.10 | 18.10 | 2.20 | 6.90 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.22 | 35.17 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-main-default-switch-mid.png (mid at 69 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-main-default-switch-rest.png
label=main cpu0 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.66
  command p90 ms                          20.49
  frames                                  97.00
  tick ms/frame                            1.62
  frame p90 ms                             2.44
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         53.11
  interval max ms (median burst)          53.11
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.30
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
| main cpu0 bd-config bo-config #1 | switch | 19.66 | 20.49 | 97.00 | 1.62 | 2.44 | 1.00 | 5.71 | 5.75 | 53.11 | 53.11 | - | - | - | - | - | - | - | - | - | - | - | - | 0.61 | 0.02 | 3.00 | 5.30 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-main-default-minimize-mid.png (mid at 76 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu0-bdconfigboconfig-main-default-minimize-rest.png
label=main cpu0 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.96
  command p90 ms                          19.00
  frames                                 150.00
  tick ms/frame                            1.54
  frame p90 ms                             4.20
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         37.66
  interval max ms (median burst)          37.66
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
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)               21.30
  session_begin ms (outside frames)        4.20
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
  input->anim start ms                     6.04
  input->first frame ms                   55.13
| main cpu0 bd-config bo-config #1 | minimize | 16.96 | 19.00 | 150.00 | 1.54 | 4.20 | 2.00 | 5.71 | 5.76 | 37.66 | 37.66 | - | - | - | - | - | - | - | - | - | - | - | - | 0.73 | 0.03 | 4.00 | 4.70 | 21.30 | 4.20 | 6.50 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.04 | 55.13 |
tiles start-latency cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=start-latency cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.74
  command p90 ms                          20.90
  frames                                  63.00
  tick ms/frame                            0.85
  frame p90 ms                             0.95
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.98
  interval max ms (median burst)          10.98
  peek_frame ms/frame                      0.39
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.04
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.75
  pd_alpha ms (worst frame)                0.40
  pd_draw ms (worst frame)                 0.48
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)                7.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    19.60
  peek_new ms (outside frames)             5.50
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.10
  peek_handover ms (outside frames)        2.60
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     4.29
  input->first frame ms                   21.67
| start-latency cpu0 bd-config bo-config #2 | move | 19.74 | 20.90 | 63.00 | 0.85 | 0.95 | 0.00 | 5.71 | 5.74 | 10.98 | 10.98 | 0.39 | 0.16 | 0.04 | 0.01 | 0.03 | 0.75 | 0.40 | 0.48 | - | - | - | - | - | - | - | 4.40 | 7.00 | - | 0.00 | 0.00 | 19.60 | 5.50 | 1.20 | - | - | - | 1.10 | 2.60 | 0.40 | - | - | - | - | - | - | 4.29 | 21.67 |
label=start-latency cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          64.61
  command p90 ms                          66.12
  frames                                  59.00
  tick ms/frame                            3.39
  frame p90 ms                             4.32
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          6.15
  interval p99 ms                         50.49
  interval max ms (median burst)          50.49
  peek_frame ms/frame                      2.45
  peek_draw ms/frame                       1.40
  peek_track ms/frame                      0.28
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.13
  peek_draw ms (worst frame)              27.62
  pd_alpha ms (worst frame)                0.96
  pd_draw ms (worst frame)                26.65
  pd_fill ms (worst frame)                23.04
  fill_source ms (worst frame)             7.65
  ring_raise ms (worst frame)              8.96
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            4.30
  cloak ms (outside frames)               19.10
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    64.20
  peek_new ms (outside frames)            16.20
  peek_place ms (outside frames)           4.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          10.40
  peek_handover ms (outside frames)       10.30
  peek_raise ms (outside frames)           2.20
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     4.10
  input->first frame ms                   93.27
| start-latency cpu0 bd-config bo-config #2 | resize | 64.61 | 66.12 | 59.00 | 3.39 | 4.32 | 4.00 | 5.71 | 6.15 | 50.49 | 50.49 | 2.45 | 1.40 | 0.28 | 0.13 | 0.13 | 27.62 | 0.96 | 26.65 | 23.04 | 7.65 | 8.96 | - | - | 0.04 | 1.00 | 4.30 | 19.10 | - | 0.00 | 0.00 | 64.20 | 16.20 | 4.10 | - | - | - | 10.40 | 10.30 | 2.20 | - | - | - | - | - | - | 4.10 | 93.27 |
Moved 3 window(s) to workspace '2' to switch to.
label=start-latency cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.24
  command p90 ms                          33.39
  frames                                 100.00
  tick ms/frame                            1.42
  frame p90 ms                             1.17
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         25.04
  interval max ms (median burst)          25.04
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
  dwm_flush ms (outside frames)            5.30
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    29.90
  peek_new ms (outside frames)             0.00
  peek_place ms (outside frames)           0.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           0.00
  peek_handover ms (outside frames)        0.00
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)       12.90
  ws_park ms (outside frames)              1.90
  ws_show ms (outside frames)              9.20
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| start-latency cpu0 bd-config bo-config #2 | switch | 30.24 | 33.39 | 100.00 | 1.42 | 1.17 | 3.00 | 5.71 | 5.76 | 25.04 | 25.04 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.30 | 0.00 | - | 0.00 | 0.00 | 29.90 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | - | - | 12.90 | 1.90 | 9.20 | 0.40 | 0.00 | 0.00 |
label=start-latency cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          60.89
  command p90 ms                          68.31
  frames                                  60.00
  tick ms/frame                            3.02
  frame p90 ms                             3.70
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          8.55
  interval p99 ms                         74.22
  interval max ms (median burst)          74.22
  peek_frame ms/frame                      2.21
  peek_draw ms/frame                       1.26
  peek_track ms/frame                      0.34
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.21
  peek_draw ms (worst frame)              27.03
  pd_alpha ms (worst frame)                0.70
  pd_draw ms (worst frame)                26.13
  pd_fill ms (worst frame)                22.02
  fill_source ms (worst frame)             9.04
  ring_raise ms (worst frame)              6.91
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.40
  cloak ms (outside frames)               18.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    60.60
  peek_new ms (outside frames)            25.50
  peek_place ms (outside frames)           5.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          18.40
  peek_handover ms (outside frames)       15.10
  peek_raise ms (outside frames)           1.50
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.32
  input->first frame ms                   92.09
| start-latency cpu0 bd-config bo-config #2 | minimize | 60.89 | 68.31 | 60.00 | 3.02 | 3.70 | 4.00 | 5.71 | 8.55 | 74.22 | 74.22 | 2.21 | 1.26 | 0.34 | 0.13 | 0.21 | 27.03 | 0.70 | 26.13 | 22.02 | 9.04 | 6.91 | - | - | 0.03 | 2.00 | 9.40 | 18.70 | - | 0.00 | 0.00 | 60.60 | 25.50 | 5.90 | - | - | - | 18.40 | 15.10 | 1.50 | - | - | - | - | - | - | 9.32 | 92.09 |
tiles stand-ins:nopeeks cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          14.94
  command p90 ms                          17.18
  frames                                  62.00
  tick ms/frame                            0.55
  frame p90 ms                             1.13
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                          7.68
  interval max ms (median burst)           7.68
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
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.60
  cloak ms (outside frames)                7.60
  session_begin ms (outside frames)        0.60
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
  input->anim start ms                     0.31
  input->first frame ms                   15.62
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | move | 14.94 | 17.18 | 62.00 | 0.55 | 1.13 | 0.00 | 5.71 | 5.75 | 7.68 | 7.68 | - | - | - | - | - | - | - | - | - | - | - | - | 0.33 | 0.00 | - | 3.60 | 7.60 | 0.60 | 4.50 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.31 | 15.62 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          39.15
  command p90 ms                          42.44
  frames                                  62.00
  tick ms/frame                            1.86
  frame p90 ms                             4.28
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         25.61
  interval max ms (median burst)          25.61
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
  batch_commit ms/frame                    0.98
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)               20.50
  session_begin ms (outside frames)        1.60
  session_overlays ms (outside frames)    15.00
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
  input->first frame ms                   42.53
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | resize | 39.15 | 42.44 | 62.00 | 1.86 | 4.28 | 2.00 | 5.71 | 5.77 | 25.61 | 25.61 | - | - | - | - | - | - | - | - | - | - | - | - | 0.98 | 0.04 | - | 4.70 | 20.50 | 1.60 | 15.00 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.29 | 42.53 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.06
  command p90 ms                          30.54
  frames                                  99.00
  tick ms/frame                            1.21
  frame p90 ms                             1.19
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         23.49
  interval max ms (median burst)          23.49
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
  dwm_flush ms (outside frames)            4.60
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
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | switch | 29.06 | 30.54 | 99.00 | 1.21 | 1.19 | 3.00 | 5.72 | 5.76 | 23.49 | 23.49 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.03 | - | 4.60 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          23.92
  command p90 ms                          27.04
  frames                                 145.00
  tick ms/frame                            1.17
  frame p90 ms                             3.09
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         44.57
  interval max ms (median burst)          44.57
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
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)               19.80
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
  input->anim start ms                     5.51
  input->first frame ms                   52.42
| stand-ins:nopeeks cpu0 bd-config bo-config #2 | minimize | 23.92 | 27.04 | 145.00 | 1.17 | 3.09 | 0.00 | 5.71 | 5.76 | 44.57 | 44.57 | - | - | - | - | - | - | - | - | - | - | - | - | 0.65 | 0.02 | 1.00 | 4.80 | 19.80 | 2.20 | 17.20 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 5.51 | 52.42 |
tiles stand-ins cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.91
  command p90 ms                          20.57
  frames                                  63.00
  tick ms/frame                            0.85
  frame p90 ms                             0.91
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         10.79
  interval max ms (median burst)          10.79
  peek_frame ms/frame                      0.41
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
  dwm_flush ms (outside frames)            5.20
  cloak ms (outside frames)                8.20
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
| stand-ins cpu0 bd-config bo-config #2 | move | 18.91 | 20.57 | 63.00 | 0.85 | 0.91 | 0.00 | 5.71 | 5.75 | 10.79 | 10.79 | 0.41 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 5.20 | 8.20 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          65.55
  command p90 ms                          69.78
  frames                                  60.00
  tick ms/frame                            3.53
  frame p90 ms                             3.73
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.90
  interval p99 ms                         68.14
  interval max ms (median burst)          68.14
  peek_frame ms/frame                      2.30
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
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            7.10
  cloak ms (outside frames)               18.70
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
| stand-ins cpu0 bd-config bo-config #2 | resize | 65.55 | 69.78 | 60.00 | 3.53 | 3.73 | 4.00 | 5.72 | 5.90 | 68.14 | 68.14 | 2.30 | - | - | - | - | - | - | - | - | - | - | - | - | 0.05 | 2.00 | 7.10 | 18.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.96
  command p90 ms                          31.40
  frames                                 100.00
  tick ms/frame                            1.41
  frame p90 ms                             1.12
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         24.96
  interval max ms (median burst)          24.96
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
| stand-ins cpu0 bd-config bo-config #2 | switch | 30.96 | 31.40 | 100.00 | 1.41 | 1.12 | 3.00 | 5.71 | 5.74 | 24.96 | 24.96 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 6.50 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          61.41
  command p90 ms                          65.65
  frames                                  61.00
  tick ms/frame                            3.18
  frame p90 ms                             4.21
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          6.94
  interval p99 ms                         77.51
  interval max ms (median burst)          77.51
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.50
  cloak ms (outside frames)               14.70
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
| stand-ins cpu0 bd-config bo-config #2 | minimize | 61.41 | 65.65 | 61.00 | 3.18 | 4.21 | 4.00 | 5.71 | 6.94 | 77.51 | 77.51 | 2.24 | - | - | - | - | - | - | - | - | - | - | - | - | 0.02 | 2.00 | 8.50 | 14.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles stand-ins:morepeeks cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:morepeeks cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.61
  command p90 ms                          20.72
  frames                                  63.00
  tick ms/frame                            0.89
  frame p90 ms                             0.96
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         11.00
  interval max ms (median burst)          11.00
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
  dwm_flush ms (outside frames)            4.60
  cloak ms (outside frames)                7.50
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
| stand-ins:morepeeks cpu0 bd-config bo-config #2 | move | 19.61 | 20.72 | 63.00 | 0.89 | 0.96 | 0.00 | 5.71 | 5.76 | 11.00 | 11.00 | 0.39 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.60 | 7.50 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          63.92
  command p90 ms                          72.89
  frames                                  60.00
  tick ms/frame                            3.52
  frame p90 ms                             3.44
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.86
  interval p99 ms                         51.04
  interval max ms (median burst)          51.04
  peek_frame ms/frame                      2.29
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
  dwm_flush ms (outside frames)           11.40
  cloak ms (outside frames)               19.10
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
| stand-ins:morepeeks cpu0 bd-config bo-config #2 | resize | 63.92 | 72.89 | 60.00 | 3.52 | 3.44 | 4.00 | 5.72 | 5.86 | 51.04 | 51.04 | 2.29 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 11.40 | 19.10 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:morepeeks cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.63
  command p90 ms                          35.62
  frames                                  99.00
  tick ms/frame                            1.43
  frame p90 ms                             1.25
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         26.24
  interval max ms (median burst)          26.24
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
| stand-ins:morepeeks cpu0 bd-config bo-config #2 | switch | 30.63 | 35.62 | 99.00 | 1.43 | 1.25 | 3.00 | 5.72 | 5.76 | 26.24 | 26.24 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 5.80 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          59.32
  command p90 ms                          64.86
  frames                                  61.00
  tick ms/frame                            3.03
  frame p90 ms                             3.62
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.08
  interval p99 ms                         73.67
  interval max ms (median burst)          73.67
  peek_frame ms/frame                      2.06
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
  dwm_flush ms (outside frames)           11.70
  cloak ms (outside frames)               18.20
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
| stand-ins:morepeeks cpu0 bd-config bo-config #2 | minimize | 59.32 | 64.86 | 61.00 | 3.03 | 3.62 | 4.00 | 5.72 | 7.08 | 73.67 | 73.67 | 2.06 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 11.70 | 18.20 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles main cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          14.35
  command p90 ms                          15.67
  frames                                  63.00
  tick ms/frame                            0.63
  frame p90 ms                             1.16
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          7.47
  interval max ms (median burst)           7.47
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
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.60
  cloak ms (outside frames)                8.30
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
  input->anim start ms                     0.23
  input->first frame ms                   15.08
| main cpu0 bd-config bo-config #2 | move | 14.35 | 15.67 | 63.00 | 0.63 | 1.16 | 0.00 | 5.71 | 5.74 | 7.47 | 7.47 | - | - | - | - | - | - | - | - | - | - | - | - | 0.36 | 0.00 | 1.00 | 5.60 | 8.30 | 0.90 | 3.00 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.23 | 15.08 |
label=main cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.57
  command p90 ms                          31.32
  frames                                  63.00
  tick ms/frame                            2.20
  frame p90 ms                             3.73
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         33.68
  interval max ms (median burst)          33.68
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
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.10
  cloak ms (outside frames)               17.80
  session_begin ms (outside frames)        2.10
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
  input->first frame ms                   34.33
| main cpu0 bd-config bo-config #2 | resize | 29.57 | 31.32 | 63.00 | 2.20 | 3.73 | 2.00 | 5.71 | 5.76 | 33.68 | 33.68 | - | - | - | - | - | - | - | - | - | - | - | - | 0.91 | 0.05 | 2.00 | 4.10 | 17.80 | 2.10 | 6.90 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.22 | 34.33 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.52
  command p90 ms                          21.43
  frames                                  97.00
  tick ms/frame                            1.62
  frame p90 ms                             2.36
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         51.92
  interval max ms (median burst)          52.95
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
  ovl_region ms/frame                      0.02
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
| main cpu0 bd-config bo-config #2 | switch | 20.52 | 21.43 | 97.00 | 1.62 | 2.36 | 1.00 | 5.71 | 5.75 | 51.92 | 52.95 | - | - | - | - | - | - | - | - | - | - | - | - | 0.62 | 0.02 | 3.00 | 5.90 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=main cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          16.80
  command p90 ms                          18.23
  frames                                 149.00
  tick ms/frame                            1.43
  frame p90 ms                             3.57
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         38.48
  interval max ms (median burst)          38.48
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
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)               16.90
  session_begin ms (outside frames)        2.10
  session_overlays ms (outside frames)     7.30
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
  input->anim start ms                     6.47
  input->first frame ms                   49.42
| main cpu0 bd-config bo-config #2 | minimize | 16.80 | 18.23 | 149.00 | 1.43 | 3.57 | 3.00 | 5.71 | 5.75 | 38.48 | 38.48 | - | - | - | - | - | - | - | - | - | - | - | - | 0.68 | 0.03 | 4.00 | 4.90 | 16.90 | 2.10 | 7.30 | 0.01 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.47 | 49.42 |
tiles this cpu0 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu0 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          15.70
  command p90 ms                          17.79
  frames                                  63.00
  tick ms/frame                            0.38
  frame p90 ms                             0.52
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         10.08
  interval max ms (median burst)          10.08
  peek_frame ms/frame                      0.37
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.75
  pd_alpha ms (worst frame)                0.41
  pd_draw ms (worst frame)                 0.46
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)                7.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    15.40
  peek_new ms (outside frames)             2.10
  peek_place ms (outside frames)           0.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.20
  peek_handover ms (outside frames)        2.70
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.00
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.87
  input->first frame ms                   17.85
| this cpu0 bd-config bo-config #2 | move | 15.70 | 17.79 | 63.00 | 0.38 | 0.52 | 0.00 | 5.71 | 5.75 | 10.08 | 10.08 | 0.37 | 0.16 | 0.05 | 0.01 | 0.03 | 0.75 | 0.41 | 0.46 | - | - | - | - | - | - | - | 5.10 | 7.80 | - | - | - | 15.40 | 2.10 | 0.90 | - | - | - | 1.20 | 2.70 | 0.30 | 1.00 | - | - | - | - | - | 2.87 | 17.85 |
label=this cpu0 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          45.75
  command p90 ms                          53.32
  frames                                  62.00
  tick ms/frame                            2.42
  frame p90 ms                             3.20
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         36.71
  interval max ms (median burst)          36.71
  peek_frame ms/frame                      1.84
  peek_draw ms/frame                       1.22
  peek_track ms/frame                      0.30
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.12
  peek_draw ms (worst frame)              15.80
  pd_alpha ms (worst frame)                1.02
  pd_draw ms (worst frame)                14.71
  pd_fill ms (worst frame)                10.95
  fill_source ms (worst frame)             3.63
  ring_raise ms (worst frame)              5.22
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)               15.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    45.40
  peek_new ms (outside frames)             3.20
  peek_place ms (outside frames)           1.90
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          10.80
  peek_handover ms (outside frames)       11.70
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.80
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.21
  input->first frame ms                   64.18
| this cpu0 bd-config bo-config #2 | resize | 45.75 | 53.32 | 62.00 | 2.42 | 3.20 | 3.00 | 5.72 | 5.77 | 36.71 | 36.71 | 1.84 | 1.22 | 0.30 | 0.14 | 0.12 | 15.80 | 1.02 | 14.71 | 10.95 | 3.63 | 5.22 | 0.00 | - | 0.04 | 2.00 | 4.90 | 15.90 | - | - | - | 45.40 | 3.20 | 1.90 | - | - | - | 10.80 | 11.70 | 0.10 | 0.80 | - | - | - | - | - | 3.21 | 64.18 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu0 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          25.67
  command p90 ms                          27.16
  frames                                  98.00
  tick ms/frame                            1.11
  frame p90 ms                             1.02
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         20.68
  interval max ms (median burst)          20.68
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
  dwm_flush ms (outside frames)            4.60
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
  restack ms (outside frames)              0.00
  ws_surrogates ms (outside frames)       10.20
  ws_park ms (outside frames)              2.00
  ws_show ms (outside frames)              7.10
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| this cpu0 bd-config bo-config #2 | switch | 25.67 | 27.16 | 98.00 | 1.11 | 1.02 | 3.00 | 5.72 | 5.76 | 20.68 | 20.68 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.02 | - | 4.60 | - | - | - | - | 25.40 | - | - | - | - | - | - | - | 0.00 | 0.50 | 0.00 | 10.20 | 2.00 | 7.10 | 0.40 | - | - |
label=this cpu0 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          51.43
  command p90 ms                          55.01
  frames                                  61.00
  tick ms/frame                            2.35
  frame p90 ms                             3.76
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         50.44
  interval max ms (median burst)          50.44
  peek_frame ms/frame                      1.86
  peek_draw ms/frame                       1.17
  peek_track ms/frame                      0.31
  peek_ring ms/frame                       0.12
  peek_backdrop ms/frame                   0.17
  peek_draw ms (worst frame)              15.61
  pd_alpha ms (worst frame)                0.87
  pd_draw ms (worst frame)                14.94
  pd_fill ms (worst frame)                11.33
  fill_source ms (worst frame)             3.54
  ring_raise ms (worst frame)              4.47
  restack ms (worst frame)                 2.81
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.30
  cloak ms (outside frames)               15.50
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    51.20
  peek_new ms (outside frames)             6.80
  peek_place ms (outside frames)           1.70
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          23.80
  peek_handover ms (outside frames)       15.40
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              1.90
  restack ms (outside frames)              8.60
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    12.38
  input->first frame ms                   68.02
| this cpu0 bd-config bo-config #2 | minimize | 51.43 | 55.01 | 61.00 | 2.35 | 3.76 | 3.00 | 5.72 | 5.78 | 50.44 | 50.44 | 1.86 | 1.17 | 0.31 | 0.12 | 0.17 | 15.61 | 0.87 | 14.94 | 11.33 | 3.54 | 4.47 | 2.81 | - | 0.02 | 2.00 | 10.30 | 15.50 | - | - | - | 51.20 | 6.80 | 1.70 | - | - | - | 23.80 | 15.40 | 0.20 | 1.90 | 8.60 | - | - | - | - | 12.38 | 68.02 |
tiles stand-ins:nopeeks cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.17
  command p90 ms                          16.88
  frames                                  62.00
  tick ms/frame                            0.54
  frame p90 ms                             1.09
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          7.67
  interval max ms (median burst)           7.67
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
  batch_commit ms/frame                    0.32
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.70
  cloak ms (outside frames)                9.10
  session_begin ms (outside frames)        0.70
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
  input->anim start ms                     0.30
  input->first frame ms                   16.72
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | move | 16.17 | 16.88 | 62.00 | 0.54 | 1.09 | 0.00 | 5.71 | 5.74 | 7.67 | 7.67 | - | - | - | - | - | - | - | - | - | - | - | - | 0.32 | 0.00 | - | 4.70 | 9.10 | 0.70 | 4.50 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.30 | 16.72 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          39.84
  command p90 ms                          43.54
  frames                                  62.00
  tick ms/frame                            1.76
  frame p90 ms                             4.48
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         22.21
  interval max ms (median burst)          22.21
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
  batch_commit ms/frame                    0.98
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)               20.80
  session_begin ms (outside frames)        1.70
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
  input->anim start ms                     0.28
  input->first frame ms                   44.11
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | resize | 39.84 | 43.54 | 62.00 | 1.76 | 4.48 | 1.00 | 5.71 | 5.77 | 22.21 | 22.21 | - | - | - | - | - | - | - | - | - | - | - | - | 0.98 | 0.03 | - | 4.80 | 20.80 | 1.70 | 15.30 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.28 | 44.11 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.39
  command p90 ms                          32.63
  frames                                  99.00
  tick ms/frame                            1.23
  frame p90 ms                             1.20
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         22.58
  interval max ms (median burst)          22.58
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
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | switch | 30.39 | 32.63 | 99.00 | 1.23 | 1.20 | 3.00 | 5.72 | 5.76 | 22.58 | 22.58 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.04 | - | 5.90 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=stand-ins:nopeeks cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          23.18
  command p90 ms                          25.21
  frames                                 142.00
  tick ms/frame                            1.15
  frame p90 ms                             3.23
  frames >2x budget                        0.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         48.04
  interval max ms (median burst)          48.04
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
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            3.60
  cloak ms (outside frames)               22.00
  session_begin ms (outside frames)        2.10
  session_overlays ms (outside frames)    16.80
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
  input->anim start ms                     5.57
  input->first frame ms                   55.39
| stand-ins:nopeeks cpu0 bd-config bo-config #3 | minimize | 23.18 | 25.21 | 142.00 | 1.15 | 3.23 | 0.00 | 5.72 | 5.76 | 48.04 | 48.04 | - | - | - | - | - | - | - | - | - | - | - | - | 0.62 | 0.02 | 1.00 | 3.60 | 22.00 | 2.10 | 16.80 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 5.57 | 55.39 |
tiles stand-ins cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.23
  command p90 ms                          20.80
  frames                                  63.00
  tick ms/frame                            0.82
  frame p90 ms                             0.88
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         10.82
  interval max ms (median burst)          10.82
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
  dwm_flush ms (outside frames)            4.40
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
| stand-ins cpu0 bd-config bo-config #3 | move | 19.23 | 20.80 | 63.00 | 0.82 | 0.88 | 0.00 | 5.71 | 5.75 | 10.82 | 10.82 | 0.38 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.40 | 7.40 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          64.73
  command p90 ms                          68.48
  frames                                  60.00
  tick ms/frame                            3.65
  frame p90 ms                             4.05
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.87
  interval p99 ms                         52.27
  interval max ms (median burst)          52.27
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
  dwm_flush ms (outside frames)           10.30
  cloak ms (outside frames)               19.30
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
| stand-ins cpu0 bd-config bo-config #3 | resize | 64.73 | 68.48 | 60.00 | 3.65 | 4.05 | 4.00 | 5.72 | 5.87 | 52.27 | 52.27 | 2.36 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 10.30 | 19.30 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.33
  command p90 ms                          31.52
  frames                                  99.00
  tick ms/frame                            1.44
  frame p90 ms                             1.21
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.75
  interval p99 ms                         24.34
  interval max ms (median burst)          26.48
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
  dwm_flush ms (outside frames)            6.20
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
| stand-ins cpu0 bd-config bo-config #3 | switch | 29.33 | 31.52 | 99.00 | 1.44 | 1.21 | 3.00 | 5.72 | 5.75 | 24.34 | 26.48 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.02 | - | 6.20 | 0.00 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          62.42
  command p90 ms                          68.79
  frames                                  60.00
  tick ms/frame                            3.27
  frame p90 ms                             3.67
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.00
  interval p99 ms                         79.16
  interval max ms (median burst)          79.16
  peek_frame ms/frame                      2.25
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
  dwm_flush ms (outside frames)           11.10
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
| stand-ins cpu0 bd-config bo-config #3 | minimize | 62.42 | 68.79 | 60.00 | 3.27 | 3.67 | 4.00 | 5.72 | 7.00 | 79.16 | 79.16 | 2.25 | - | - | - | - | - | - | - | - | - | - | - | - | 0.02 | 2.00 | 11.10 | 17.40 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles stand-ins:morepeeks cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:morepeeks cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.78
  command p90 ms                          19.88
  frames                                  63.00
  tick ms/frame                            0.86
  frame p90 ms                             0.89
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         10.51
  interval max ms (median burst)          10.51
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
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)                7.00
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
| stand-ins:morepeeks cpu0 bd-config bo-config #3 | move | 18.78 | 19.88 | 63.00 | 0.86 | 0.89 | 0.00 | 5.71 | 5.75 | 10.51 | 10.51 | 0.37 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.40 | 7.00 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          62.53
  command p90 ms                          64.77
  frames                                  60.00
  tick ms/frame                            3.61
  frame p90 ms                             3.30
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.02
  interval p99 ms                         68.81
  interval max ms (median burst)          68.81
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
  dwm_flush ms (outside frames)            4.40
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
| stand-ins:morepeeks cpu0 bd-config bo-config #3 | resize | 62.53 | 64.77 | 60.00 | 3.61 | 3.30 | 4.00 | 5.72 | 6.02 | 68.81 | 68.81 | 2.24 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 4.40 | 17.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:morepeeks cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.46
  command p90 ms                          31.74
  frames                                  99.00
  tick ms/frame                            1.42
  frame p90 ms                             1.20
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         25.11
  interval max ms (median burst)          25.11
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
| stand-ins:morepeeks cpu0 bd-config bo-config #3 | switch | 30.46 | 31.74 | 99.00 | 1.42 | 1.20 | 3.00 | 5.71 | 5.75 | 25.11 | 25.11 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 6.30 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          56.98
  command p90 ms                          68.88
  frames                                  60.00
  tick ms/frame                            3.26
  frame p90 ms                             3.90
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.68
  interval p99 ms                         72.35
  interval max ms (median burst)          72.35
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
  dwm_flush ms (outside frames)           10.40
  cloak ms (outside frames)               16.20
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
| stand-ins:morepeeks cpu0 bd-config bo-config #3 | minimize | 56.98 | 68.88 | 60.00 | 3.26 | 3.90 | 4.00 | 5.72 | 6.68 | 72.35 | 72.35 | 2.14 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 10.40 | 16.20 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles main cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          14.74
  command p90 ms                          15.60
  frames                                  63.00
  tick ms/frame                            0.64
  frame p90 ms                             1.16
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                          7.29
  interval max ms (median burst)           7.29
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
  ovl_region ms/frame                      0.00
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            6.10
  cloak ms (outside frames)                9.00
  session_begin ms (outside frames)        0.90
  session_overlays ms (outside frames)     2.70
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
  input->first frame ms                   15.85
| main cpu0 bd-config bo-config #3 | move | 14.74 | 15.60 | 63.00 | 0.64 | 1.16 | 0.00 | 5.71 | 5.75 | 7.29 | 7.29 | - | - | - | - | - | - | - | - | - | - | - | - | 0.36 | 0.00 | 1.00 | 6.10 | 9.00 | 0.90 | 2.70 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.23 | 15.85 |
label=main cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.47
  command p90 ms                          32.12
  frames                                  63.00
  tick ms/frame                            2.34
  frame p90 ms                             4.30
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         34.37
  interval max ms (median burst)          34.37
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
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)               18.60
  session_begin ms (outside frames)        2.10
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
  input->anim start ms                     0.23
  input->first frame ms                   35.09
| main cpu0 bd-config bo-config #3 | resize | 30.47 | 32.12 | 63.00 | 2.34 | 4.30 | 2.00 | 5.71 | 5.77 | 34.37 | 34.37 | - | - | - | - | - | - | - | - | - | - | - | - | 0.96 | 0.05 | 2.00 | 5.70 | 18.60 | 2.10 | 7.40 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.23 | 35.09 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.21
  command p90 ms                          20.66
  frames                                  97.00
  tick ms/frame                            1.65
  frame p90 ms                             2.45
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         54.97
  interval max ms (median burst)          56.60
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
| main cpu0 bd-config bo-config #3 | switch | 20.21 | 20.66 | 97.00 | 1.65 | 2.45 | 1.00 | 5.71 | 5.75 | 54.97 | 56.60 | - | - | - | - | - | - | - | - | - | - | - | - | 0.64 | 0.02 | 3.00 | 5.80 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=main cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          17.06
  command p90 ms                          19.36
  frames                                 150.00
  tick ms/frame                            1.33
  frame p90 ms                             3.89
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         33.48
  interval max ms (median burst)          35.41
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
  batch_commit ms/frame                    0.69
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            5.50
  cloak ms (outside frames)               17.30
  session_begin ms (outside frames)        2.00
  session_overlays ms (outside frames)     7.50
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
  input->anim start ms                     6.15
  input->first frame ms                   43.27
| main cpu0 bd-config bo-config #3 | minimize | 17.06 | 19.36 | 150.00 | 1.33 | 3.89 | 3.00 | 5.72 | 5.76 | 33.48 | 35.41 | - | - | - | - | - | - | - | - | - | - | - | - | 0.69 | 0.03 | 4.00 | 5.50 | 17.30 | 2.00 | 7.50 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.15 | 43.27 |
tiles this cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.06
  command p90 ms                          18.44
  frames                                  63.00
  tick ms/frame                            0.38
  frame p90 ms                             0.50
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.38
  interval max ms (median burst)          10.38
  peek_frame ms/frame                      0.36
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.82
  pd_alpha ms (worst frame)                0.43
  pd_draw ms (worst frame)                 0.48
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
  command ms (WM side)                    15.80
  peek_new ms (outside frames)             2.00
  peek_place ms (outside frames)           1.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.30
  peek_handover ms (outside frames)        2.40
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              1.00
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     2.93
  input->first frame ms                   17.41
| this cpu0 bd-config bo-config #3 | move | 16.06 | 18.44 | 63.00 | 0.38 | 0.50 | 0.00 | 5.71 | 5.74 | 10.38 | 10.38 | 0.36 | 0.16 | 0.05 | 0.01 | 0.03 | 0.82 | 0.43 | 0.48 | - | - | - | - | - | - | - | 4.90 | 7.60 | - | - | - | 15.80 | 2.00 | 1.00 | - | - | - | 1.30 | 2.40 | 0.30 | 1.00 | - | - | - | - | - | 2.93 | 17.41 |
label=this cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          47.91
  command p90 ms                          54.51
  frames                                  61.00
  tick ms/frame                            2.47
  frame p90 ms                             3.61
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         48.49
  interval max ms (median burst)          48.49
  peek_frame ms/frame                      2.03
  peek_draw ms/frame                       1.23
  peek_track ms/frame                      0.33
  peek_ring ms/frame                       0.18
  peek_backdrop ms/frame                   0.15
  peek_draw ms (worst frame)              15.81
  pd_alpha ms (worst frame)                1.04
  pd_draw ms (worst frame)                14.78
  pd_fill ms (worst frame)                11.21
  fill_source ms (worst frame)             2.44
  ring_raise ms (worst frame)              5.24
  restack ms (worst frame)                 3.07
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.50
  cloak ms (outside frames)               18.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    47.50
  peek_new ms (outside frames)             3.30
  peek_place ms (outside frames)           2.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          11.20
  peek_handover ms (outside frames)       11.60
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.70
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.29
  input->first frame ms                   67.48
| this cpu0 bd-config bo-config #3 | resize | 47.91 | 54.51 | 61.00 | 2.47 | 3.61 | 3.00 | 5.72 | 5.77 | 48.49 | 48.49 | 2.03 | 1.23 | 0.33 | 0.18 | 0.15 | 15.81 | 1.04 | 14.78 | 11.21 | 2.44 | 5.24 | 3.07 | - | 0.05 | 2.00 | 9.50 | 18.20 | - | - | - | 47.50 | 3.30 | 2.00 | - | - | - | 11.20 | 11.60 | 0.10 | 0.70 | - | - | - | - | - | 3.29 | 67.48 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          27.17
  command p90 ms                          28.41
  frames                                  99.00
  tick ms/frame                            1.11
  frame p90 ms                             1.10
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         18.14
  interval max ms (median burst)          21.74
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
  dwm_flush ms (outside frames)            6.10
  cloak ms (outside frames)                0.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    26.90
  peek_new ms (outside frames)             0.00
  peek_place ms (outside frames)           0.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           0.00
  peek_handover ms (outside frames)        0.00
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.50
  restack ms (outside frames)              0.00
  ws_surrogates ms (outside frames)       10.20
  ws_park ms (outside frames)              1.80
  ws_show ms (outside frames)              8.80
  ws_start ms (outside frames)             0.40
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| this cpu0 bd-config bo-config #3 | switch | 27.17 | 28.41 | 99.00 | 1.11 | 1.10 | 3.00 | 5.71 | 5.76 | 18.14 | 21.74 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 6.10 | 0.00 | - | - | - | 26.90 | 0.00 | 0.00 | - | - | - | 0.00 | 0.00 | 0.00 | 0.50 | 0.00 | 10.20 | 1.80 | 8.80 | 0.40 | 0.00 | 0.00 |
label=this cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          51.19
  command p90 ms                          59.97
  frames                                  61.00
  tick ms/frame                            2.32
  frame p90 ms                             3.59
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         56.96
  interval max ms (median burst)          56.96
  peek_frame ms/frame                      1.95
  peek_draw ms/frame                       1.24
  peek_track ms/frame                      0.37
  peek_ring ms/frame                       0.15
  peek_backdrop ms/frame                   0.21
  peek_draw ms (worst frame)              18.49
  pd_alpha ms (worst frame)                0.74
  pd_draw ms (worst frame)                17.96
  pd_fill ms (worst frame)                13.27
  fill_source ms (worst frame)             4.64
  ring_raise ms (worst frame)              4.66
  restack ms (worst frame)                 2.57
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.02
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.70
  cloak ms (outside frames)               20.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    50.90
  peek_new ms (outside frames)             8.50
  peek_place ms (outside frames)           1.70
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          21.80
  peek_handover ms (outside frames)       14.30
  peek_raise ms (outside frames)           0.20
  surface ms (outside frames)              2.60
  restack ms (outside frames)              8.40
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.69
  input->first frame ms                   74.10
| this cpu0 bd-config bo-config #3 | minimize | 51.19 | 59.97 | 61.00 | 2.32 | 3.59 | 3.00 | 5.71 | 5.78 | 56.96 | 56.96 | 1.95 | 1.24 | 0.37 | 0.15 | 0.21 | 18.49 | 0.74 | 17.96 | 13.27 | 4.64 | 4.66 | 2.57 | - | 0.02 | 2.00 | 11.70 | 20.40 | - | - | - | 50.90 | 8.50 | 1.70 | - | - | - | 21.80 | 14.30 | 0.20 | 2.60 | 8.40 | - | - | - | - | 10.69 | 74.10 |
tiles start-latency cpu0 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=start-latency cpu0 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          19.35
  command p90 ms                          20.51
  frames                                  63.00
  tick ms/frame                            0.79
  frame p90 ms                             0.86
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                         10.78
  interval max ms (median burst)          10.78
  peek_frame ms/frame                      0.39
  peek_draw ms/frame                       0.15
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.02
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.76
  pd_alpha ms (worst frame)                0.42
  pd_draw ms (worst frame)                 0.46
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.00
  cloak ms (outside frames)                7.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    19.10
  peek_new ms (outside frames)             5.40
  peek_place ms (outside frames)           1.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.10
  peek_handover ms (outside frames)        2.30
  peek_raise ms (outside frames)           0.50
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     4.39
  input->first frame ms                   21.52
| start-latency cpu0 bd-config bo-config #3 | move | 19.35 | 20.51 | 63.00 | 0.79 | 0.86 | 0.00 | 5.71 | 5.74 | 10.78 | 10.78 | 0.39 | 0.15 | 0.05 | 0.02 | 0.03 | 0.76 | 0.42 | 0.46 | - | - | - | - | - | - | - | 4.00 | 7.30 | - | 0.00 | 0.00 | 19.10 | 5.40 | 1.20 | - | - | - | 1.10 | 2.30 | 0.50 | - | - | - | - | - | - | 4.39 | 21.52 |
label=start-latency cpu0 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.7s
  command p50 ms                          65.55
  command p90 ms                          66.20
  frames                                  61.00
  tick ms/frame                            3.20
  frame p90 ms                             3.19
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.95
  interval p99 ms                         32.45
  interval max ms (median burst)          32.45
  peek_frame ms/frame                      2.20
  peek_draw ms/frame                       1.27
  peek_track ms/frame                      0.26
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.12
  peek_draw ms (worst frame)              23.68
  pd_alpha ms (worst frame)                0.94
  pd_draw ms (worst frame)                22.43
  pd_fill ms (worst frame)                19.13
  fill_source ms (worst frame)             5.75
  ring_raise ms (worst frame)              8.59
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            9.10
  cloak ms (outside frames)               18.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    65.20
  peek_new ms (outside frames)            16.00
  peek_place ms (outside frames)           4.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           9.90
  peek_handover ms (outside frames)       12.10
  peek_raise ms (outside frames)           2.10
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     4.39
  input->first frame ms                   88.97
| start-latency cpu0 bd-config bo-config #3 | resize | 65.55 | 66.20 | 61.00 | 3.20 | 3.19 | 4.00 | 5.72 | 5.95 | 32.45 | 32.45 | 2.20 | 1.27 | 0.26 | 0.13 | 0.12 | 23.68 | 0.94 | 22.43 | 19.13 | 5.75 | 8.59 | - | - | 0.04 | 1.00 | 9.10 | 18.70 | - | 0.00 | 0.00 | 65.20 | 16.00 | 4.10 | - | - | - | 9.90 | 12.10 | 2.10 | - | - | - | - | - | - | 4.39 | 88.97 |
Moved 3 window(s) to workspace '2' to switch to.
label=start-latency cpu0 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          29.03
  command p90 ms                          30.51
  frames                                  99.00
  tick ms/frame                            1.42
  frame p90 ms                             1.21
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         24.26
  interval max ms (median burst)          24.46
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
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    28.70
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)       12.80
  ws_park ms (outside frames)              1.90
  ws_show ms (outside frames)              8.40
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| start-latency cpu0 bd-config bo-config #3 | switch | 29.03 | 30.51 | 99.00 | 1.42 | 1.21 | 3.00 | 5.71 | 5.76 | 24.26 | 24.46 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 5.10 | - | - | 0.00 | 0.00 | 28.70 | - | - | - | - | - | - | - | 0.00 | - | - | 12.80 | 1.90 | 8.40 | 0.40 | - | - |
label=start-latency cpu0 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          61.27
  command p90 ms                          63.99
  frames                                  61.00
  tick ms/frame                            3.01
  frame p90 ms                             3.76
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.16
  interval p99 ms                         59.34
  interval max ms (median burst)          59.34
  peek_frame ms/frame                      2.11
  peek_draw ms/frame                       1.25
  peek_track ms/frame                      0.35
  peek_ring ms/frame                       0.13
  peek_backdrop ms/frame                   0.20
  peek_draw ms (worst frame)              22.56
  pd_alpha ms (worst frame)                0.77
  pd_draw ms (worst frame)                21.69
  pd_fill ms (worst frame)                18.39
  fill_source ms (worst frame)             5.69
  ring_raise ms (worst frame)              7.30
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.20
  cloak ms (outside frames)               14.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    60.90
  peek_new ms (outside frames)            24.60
  peek_place ms (outside frames)           5.70
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          17.60
  peek_handover ms (outside frames)       13.70
  peek_raise ms (outside frames)           1.70
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     8.64
  input->first frame ms                   87.33
| start-latency cpu0 bd-config bo-config #3 | minimize | 61.27 | 63.99 | 61.00 | 3.01 | 3.76 | 4.00 | 5.72 | 7.16 | 59.34 | 59.34 | 2.11 | 1.25 | 0.35 | 0.13 | 0.20 | 22.56 | 0.77 | 21.69 | 18.39 | 5.69 | 7.30 | - | - | 0.03 | 2.00 | 8.20 | 14.40 | - | 0.00 | 0.00 | 60.90 | 24.60 | 5.70 | - | - | - | 17.60 | 13.70 | 1.70 | - | - | - | - | - | - | 8.64 | 87.33 |
tiles stand-ins cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-default-move-mid.png (mid at 61 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-default-move-rest.png
label=stand-ins cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          21.02
  command p90 ms                          21.50
  frames                                  63.00
  tick ms/frame                            0.93
  frame p90 ms                             0.97
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         12.92
  interval max ms (median burst)          12.92
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
| stand-ins cpu8 bd-config bo-config #1 | move | 21.02 | 21.50 | 63.00 | 0.93 | 0.97 | 0.00 | 5.71 | 5.75 | 12.92 | 12.92 | 0.39 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.40 | 7.40 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-default-resize-mid.png (mid at 82 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-default-resize-rest.png
label=stand-ins cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          71.19
  command p90 ms                          76.43
  frames                                  59.00
  tick ms/frame                            3.73
  frame p90 ms                             4.99
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.74
  interval p99 ms                         65.38
  interval max ms (median burst)          65.38
  peek_frame ms/frame                      2.56
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
  dwm_flush ms (outside frames)            5.70
  cloak ms (outside frames)               17.10
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
| stand-ins cpu8 bd-config bo-config #1 | resize | 71.19 | 76.43 | 59.00 | 3.73 | 4.99 | 4.00 | 5.72 | 6.74 | 65.38 | 65.38 | 2.56 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 5.70 | 17.10 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-default-switch-mid.png (mid at 79 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-default-switch-rest.png
label=stand-ins cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          31.57
  command p90 ms                          36.14
  frames                                  99.00
  tick ms/frame                            1.57
  frame p90 ms                             1.34
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         29.25
  interval max ms (median burst)          29.25
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
| stand-ins cpu8 bd-config bo-config #1 | switch | 31.57 | 36.14 | 99.00 | 1.57 | 1.34 | 3.00 | 5.71 | 5.75 | 29.25 | 29.25 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | - | 5.40 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-default-minimize-mid.png (mid at 67 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-default-minimize-rest.png
label=stand-ins cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          63.98
  command p90 ms                          68.92
  frames                                  60.00
  tick ms/frame                            3.61
  frame p90 ms                             4.56
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          9.29
  interval p99 ms                         84.90
  interval max ms (median burst)          84.90
  peek_frame ms/frame                      2.50
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
  dwm_flush ms (outside frames)            7.10
  cloak ms (outside frames)               16.20
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
| stand-ins cpu8 bd-config bo-config #1 | minimize | 63.98 | 68.92 | 60.00 | 3.61 | 4.56 | 4.00 | 5.71 | 9.29 | 84.90 | 84.90 | 2.50 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 7.10 | 16.20 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles stand-ins:morepeeks cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-move-mid.png (mid at 67 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-move-rest.png
label=stand-ins:morepeeks cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          22.02
  command p90 ms                          23.47
  frames                                  63.00
  tick ms/frame                            0.97
  frame p90 ms                             0.97
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.66
  interval max ms (median burst)          12.66
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
  dwm_flush ms (outside frames)            4.20
  cloak ms (outside frames)                7.20
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
| stand-ins:morepeeks cpu8 bd-config bo-config #1 | move | 22.02 | 23.47 | 63.00 | 0.97 | 0.97 | 0.00 | 5.71 | 5.76 | 12.66 | 12.66 | 0.39 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.20 | 7.20 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-resize-mid.png (mid at 68 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-resize-rest.png
label=stand-ins:morepeeks cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          72.19
  command p90 ms                          81.27
  frames                                  59.00
  tick ms/frame                            3.87
  frame p90 ms                             4.38
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.84
  interval p99 ms                         64.06
  interval max ms (median burst)          64.06
  peek_frame ms/frame                      2.58
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
  cloak ms (outside frames)               18.40
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
| stand-ins:morepeeks cpu8 bd-config bo-config #1 | resize | 72.19 | 81.27 | 59.00 | 3.87 | 4.38 | 4.00 | 5.72 | 5.84 | 64.06 | 64.06 | 2.58 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 10.70 | 18.40 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-switch-mid.png (mid at 82 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-switch-rest.png
label=stand-ins:morepeeks cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          34.33
  command p90 ms                          36.60
  frames                                  99.00
  tick ms/frame                            1.56
  frame p90 ms                             1.37
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         28.85
  interval max ms (median burst)          28.85
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
| stand-ins:morepeeks cpu8 bd-config bo-config #1 | switch | 34.33 | 36.60 | 99.00 | 1.56 | 1.37 | 3.00 | 5.71 | 5.75 | 28.85 | 28.85 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 6.00 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-minimize-mid.png (mid at 64 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-morepeeks-minimize-rest.png
label=stand-ins:morepeeks cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          60.45
  command p90 ms                          74.04
  frames                                  60.00
  tick ms/frame                            2.61
  frame p90 ms                             4.75
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          6.71
  interval p99 ms                         83.06
  interval max ms (median burst)          83.06
  peek_frame ms/frame                      2.35
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
  dwm_flush calls (in frames)              0.00
  dwm_flush ms (outside frames)           12.50
  cloak ms (outside frames)               21.20
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
| stand-ins:morepeeks cpu8 bd-config bo-config #1 | minimize | 60.45 | 74.04 | 60.00 | 2.61 | 4.75 | 4.00 | 5.71 | 6.71 | 83.06 | 83.06 | 2.35 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 0.00 | 12.50 | 21.20 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles main cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-main-default-move-mid.png (mid at 75 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-main-default-move-rest.png
label=main cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          15.31
  command p90 ms                          16.71
  frames                                  63.00
  tick ms/frame                            0.66
  frame p90 ms                             1.26
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                          9.13
  interval max ms (median burst)           9.13
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
  dwm_flush ms (outside frames)            5.80
  cloak ms (outside frames)                9.20
  session_begin ms (outside frames)        1.10
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
  input->anim start ms                     0.26
  input->first frame ms                   16.31
| main cpu8 bd-config bo-config #1 | move | 15.31 | 16.71 | 63.00 | 0.66 | 1.26 | 0.00 | 5.71 | 5.75 | 9.13 | 9.13 | - | - | - | - | - | - | - | - | - | - | - | - | 0.38 | 0.00 | 1.00 | 5.80 | 9.20 | 1.10 | 3.40 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.26 | 16.31 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-main-default-resize-mid.png (mid at 66 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-main-default-resize-rest.png
label=main cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          31.94
  command p90 ms                          35.99
  frames                                  63.00
  tick ms/frame                            2.45
  frame p90 ms                             4.30
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         28.85
  interval max ms (median burst)          28.85
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
  batch_commit ms/frame                    1.08
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.30
  cloak ms (outside frames)               18.50
  session_begin ms (outside frames)        2.60
  session_overlays ms (outside frames)     7.60
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
  input->anim start ms                     0.28
  input->first frame ms                   35.90
| main cpu8 bd-config bo-config #1 | resize | 31.94 | 35.99 | 63.00 | 2.45 | 4.30 | 2.00 | 5.71 | 5.77 | 28.85 | 28.85 | - | - | - | - | - | - | - | - | - | - | - | - | 1.08 | 0.03 | 2.00 | 4.30 | 18.50 | 2.60 | 7.60 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.28 | 35.90 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-main-default-switch-mid.png (mid at 64 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-main-default-switch-rest.png
label=main cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.48
  command p90 ms                          22.30
  frames                                  97.00
  tick ms/frame                            1.90
  frame p90 ms                             2.76
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         59.29
  interval max ms (median burst)          59.29
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
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            4.00
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
| main cpu8 bd-config bo-config #1 | switch | 20.48 | 22.30 | 97.00 | 1.90 | 2.76 | 1.00 | 5.71 | 5.75 | 59.29 | 59.29 | - | - | - | - | - | - | - | - | - | - | - | - | 0.71 | 0.03 | 3.00 | 4.00 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-main-default-minimize-mid.png (mid at 65 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-main-default-minimize-rest.png
label=main cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          19.03
  command p90 ms                          21.44
  frames                                 149.00
  tick ms/frame                            1.67
  frame p90 ms                             4.33
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         42.88
  interval max ms (median burst)          42.88
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
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)               24.30
  session_begin ms (outside frames)        4.40
  session_overlays ms (outside frames)     7.80
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
  input->anim start ms                     7.40
  input->first frame ms                   59.47
| main cpu8 bd-config bo-config #1 | minimize | 19.03 | 21.44 | 149.00 | 1.67 | 4.33 | 2.00 | 5.71 | 5.77 | 42.88 | 42.88 | - | - | - | - | - | - | - | - | - | - | - | - | 0.82 | 0.03 | 4.00 | 5.40 | 24.30 | 4.40 | 7.80 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 7.40 | 59.47 |
tiles this cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-this-move-mid.png (mid at 68 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-this-move-rest.png
label=this cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.82
  command p90 ms                          18.46
  frames                                  63.00
  tick ms/frame                            0.42
  frame p90 ms                             0.53
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         12.36
  interval max ms (median burst)          12.36
  peek_frame ms/frame                      0.31
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.06
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.85
  pd_alpha ms (worst frame)                0.52
  pd_draw ms (worst frame)                 0.48
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
  command ms (WM side)                    17.50
  peek_new ms (outside frames)             2.40
  peek_place ms (outside frames)           1.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.30
  peek_handover ms (outside frames)        3.20
  peek_raise ms (outside frames)           0.50
  surface ms (outside frames)              1.30
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.39
  input->first frame ms                   19.08
| this cpu8 bd-config bo-config #1 | move | 17.82 | 18.46 | 63.00 | 0.42 | 0.53 | 0.00 | 5.71 | 5.75 | 12.36 | 12.36 | 0.31 | 0.16 | 0.06 | 0.01 | 0.04 | 0.85 | 0.52 | 0.48 | - | - | - | - | - | - | - | 5.20 | 8.20 | - | - | - | 17.50 | 2.40 | 1.10 | - | - | - | 1.30 | 3.20 | 0.50 | 1.30 | - | - | - | - | - | 3.39 | 19.08 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-this-resize-mid.png (mid at 66 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-this-resize-rest.png
label=this cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          51.59
  command p90 ms                          56.56
  frames                                  61.00
  tick ms/frame                            2.56
  frame p90 ms                             4.14
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.78
  interval p99 ms                         42.17
  interval max ms (median burst)          42.17
  peek_frame ms/frame                      2.14
  peek_draw ms/frame                       1.45
  peek_track ms/frame                      0.33
  peek_ring ms/frame                       0.15
  peek_backdrop ms/frame                   0.16
  peek_draw ms (worst frame)              17.69
  pd_alpha ms (worst frame)                1.26
  pd_draw ms (worst frame)                16.36
  pd_fill ms (worst frame)                12.44
  fill_source ms (worst frame)             2.56
  ring_raise ms (worst frame)              6.01
  restack ms (worst frame)                 0.00
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            6.00
  cloak ms (outside frames)               18.10
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    51.20
  peek_new ms (outside frames)             3.80
  peek_place ms (outside frames)           2.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          12.00
  peek_handover ms (outside frames)       12.70
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.90
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.68
  input->first frame ms                   70.59
| this cpu8 bd-config bo-config #1 | resize | 51.59 | 56.56 | 61.00 | 2.56 | 4.14 | 3.00 | 5.72 | 5.78 | 42.17 | 42.17 | 2.14 | 1.45 | 0.33 | 0.15 | 0.16 | 17.69 | 1.26 | 16.36 | 12.44 | 2.56 | 6.01 | 0.00 | - | 0.04 | 2.00 | 6.00 | 18.10 | - | - | - | 51.20 | 3.80 | 2.20 | - | - | - | 12.00 | 12.70 | 0.10 | 0.90 | - | - | - | - | - | 3.68 | 70.59 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-this-switch-mid.png (mid at 67 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-this-switch-rest.png
label=this cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.51
  command p90 ms                          30.85
  frames                                  99.00
  tick ms/frame                            1.22
  frame p90 ms                             1.21
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         23.29
  interval max ms (median burst)          23.29
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
  dwm_flush ms (outside frames)            5.10
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
  ws_surrogates ms (outside frames)       12.80
  ws_park ms (outside frames)              2.30
  ws_show ms (outside frames)              8.00
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| this cpu8 bd-config bo-config #1 | switch | 30.51 | 30.85 | 99.00 | 1.22 | 1.21 | 3.00 | 5.71 | 5.76 | 23.29 | 23.29 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | - | 5.10 | - | - | - | - | 30.20 | - | - | - | - | - | - | - | 0.00 | 0.60 | - | 12.80 | 2.30 | 8.00 | 0.40 | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-this-minimize-mid.png (mid at 63 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-this-minimize-rest.png
label=this cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          56.75
  command p90 ms                          66.58
  frames                                  61.00
  tick ms/frame                            2.52
  frame p90 ms                             4.15
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         68.50
  interval max ms (median burst)          68.50
  peek_frame ms/frame                      2.10
  peek_draw ms/frame                       1.35
  peek_track ms/frame                      0.41
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.22
  peek_draw ms (worst frame)              15.60
  pd_alpha ms (worst frame)                1.14
  pd_draw ms (worst frame)                14.64
  pd_fill ms (worst frame)                10.77
  fill_source ms (worst frame)             2.20
  ring_raise ms (worst frame)              5.45
  restack ms (worst frame)                 2.56
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           13.90
  cloak ms (outside frames)               20.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    56.40
  peek_new ms (outside frames)            13.70
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          23.70
  peek_handover ms (outside frames)       16.40
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              5.10
  restack ms (outside frames)              9.70
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.56
  input->first frame ms                   77.23
| this cpu8 bd-config bo-config #1 | minimize | 56.75 | 66.58 | 61.00 | 2.52 | 4.15 | 3.00 | 5.71 | 5.78 | 68.50 | 68.50 | 2.10 | 1.35 | 0.41 | 0.14 | 0.22 | 15.60 | 1.14 | 14.64 | 10.77 | 2.20 | 5.45 | 2.56 | - | 0.03 | 2.00 | 13.90 | 20.80 | - | - | - | 56.40 | 13.70 | 1.80 | - | - | - | 23.70 | 16.40 | 0.40 | 5.10 | 9.70 | - | - | - | - | 10.56 | 77.23 |
tiles start-latency cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-start-latency-default-move-mid.png (mid at 74 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-start-latency-default-move-rest.png
label=start-latency cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.46
  command p90 ms                          23.18
  frames                                  63.00
  tick ms/frame                            0.97
  frame p90 ms                             0.98
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.96
  interval max ms (median burst)          12.96
  peek_frame ms/frame                      0.36
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.87
  pd_alpha ms (worst frame)                0.46
  pd_draw ms (worst frame)                 0.46
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.40
  cloak ms (outside frames)                7.40
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    20.20
  peek_new ms (outside frames)             5.80
  peek_place ms (outside frames)           1.30
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.20
  peek_handover ms (outside frames)        3.20
  peek_raise ms (outside frames)           0.50
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     4.27
  input->first frame ms                   22.52
| start-latency cpu8 bd-config bo-config #1 | move | 20.46 | 23.18 | 63.00 | 0.97 | 0.98 | 0.00 | 5.71 | 5.76 | 12.96 | 12.96 | 0.36 | 0.16 | 0.05 | 0.01 | 0.03 | 0.87 | 0.46 | 0.46 | - | - | - | - | - | - | - | 4.40 | 7.40 | - | 0.00 | 0.00 | 20.20 | 5.80 | 1.30 | - | - | - | 1.20 | 3.20 | 0.50 | - | - | - | - | - | - | 4.27 | 22.52 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-start-latency-default-resize-mid.png (mid at 84 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-start-latency-default-resize-rest.png
label=start-latency cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          71.87
  command p90 ms                          78.56
  frames                                  59.00
  tick ms/frame                            3.68
  frame p90 ms                             4.55
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          6.79
  interval p99 ms                         46.11
  interval max ms (median burst)          46.11
  peek_frame ms/frame                      2.48
  peek_draw ms/frame                       1.50
  peek_track ms/frame                      0.31
  peek_ring ms/frame                       0.16
  peek_backdrop ms/frame                   0.16
  peek_draw ms (worst frame)              28.42
  pd_alpha ms (worst frame)                1.11
  pd_draw ms (worst frame)                27.37
  pd_fill ms (worst frame)                23.48
  fill_source ms (worst frame)             7.13
  ring_raise ms (worst frame)             10.05
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.20
  cloak ms (outside frames)               18.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    71.50
  peek_new ms (outside frames)            19.10
  peek_place ms (outside frames)           5.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          11.10
  peek_handover ms (outside frames)       14.10
  peek_raise ms (outside frames)           2.60
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     5.10
  input->first frame ms                  101.31
| start-latency cpu8 bd-config bo-config #1 | resize | 71.87 | 78.56 | 59.00 | 3.68 | 4.55 | 4.00 | 5.71 | 6.79 | 46.11 | 46.11 | 2.48 | 1.50 | 0.31 | 0.16 | 0.16 | 28.42 | 1.11 | 27.37 | 23.48 | 7.13 | 10.05 | - | - | 0.04 | 2.00 | 8.20 | 18.20 | - | 0.00 | 0.00 | 71.50 | 19.10 | 5.10 | - | - | - | 11.10 | 14.10 | 2.60 | - | - | - | - | - | - | 5.10 | 101.31 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-start-latency-default-switch-mid.png (mid at 79 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-start-latency-default-switch-rest.png
label=start-latency cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          32.39
  command p90 ms                          36.93
  frames                                  99.00
  tick ms/frame                            1.56
  frame p90 ms                             1.44
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         27.55
  interval max ms (median burst)          27.55
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
  dwm_flush ms (outside frames)            5.20
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    32.10
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)       14.80
  ws_park ms (outside frames)              2.40
  ws_show ms (outside frames)              8.50
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| start-latency cpu8 bd-config bo-config #1 | switch | 32.39 | 36.93 | 99.00 | 1.56 | 1.44 | 3.00 | 5.71 | 5.76 | 27.55 | 27.55 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | - | 5.20 | - | - | 0.00 | 0.00 | 32.10 | - | - | - | - | - | - | - | 0.00 | - | - | 14.80 | 2.40 | 8.50 | 0.40 | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-start-latency-default-minimize-mid.png (mid at 71 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-start-latency-default-minimize-rest.png
label=start-latency cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          67.95
  command p90 ms                          74.80
  frames                                  59.00
  tick ms/frame                            3.32
  frame p90 ms                             5.09
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          7.66
  interval p99 ms                         83.63
  interval max ms (median burst)          83.63
  peek_frame ms/frame                      2.49
  peek_draw ms/frame                       1.44
  peek_track ms/frame                      0.39
  peek_ring ms/frame                       0.16
  peek_backdrop ms/frame                   0.23
  peek_draw ms (worst frame)              30.25
  pd_alpha ms (worst frame)                0.94
  pd_draw ms (worst frame)                29.56
  pd_fill ms (worst frame)                25.11
  fill_source ms (worst frame)            10.36
  ring_raise ms (worst frame)              8.47
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.70
  cloak ms (outside frames)               16.10
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    67.60
  peek_new ms (outside frames)            31.20
  peek_place ms (outside frames)           7.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          21.40
  peek_handover ms (outside frames)       15.40
  peek_raise ms (outside frames)           1.70
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.21
  input->first frame ms                  102.52
| start-latency cpu8 bd-config bo-config #1 | minimize | 67.95 | 74.80 | 59.00 | 3.32 | 5.09 | 4.00 | 5.72 | 7.66 | 83.63 | 83.63 | 2.49 | 1.44 | 0.39 | 0.16 | 0.23 | 30.25 | 0.94 | 29.56 | 25.11 | 10.36 | 8.47 | - | - | 0.03 | 2.00 | 8.70 | 16.10 | - | 0.00 | 0.00 | 67.60 | 31.20 | 7.00 | - | - | - | 21.40 | 15.40 | 1.70 | - | - | - | - | - | - | 9.21 | 102.52 |
tiles stand-ins:nopeeks cpu8 bd-config bo-config #1 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-move-mid.png (mid at 72 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-move-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.07
  command p90 ms                          18.87
  frames                                  63.00
  tick ms/frame                            0.59
  frame p90 ms                             1.19
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                          8.72
  interval max ms (median burst)           8.72
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
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.10
  cloak ms (outside frames)                9.00
  session_begin ms (outside frames)        0.70
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
  input->anim start ms                     0.32
  input->first frame ms                   17.77
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | move | 17.07 | 18.87 | 63.00 | 0.59 | 1.19 | 0.00 | 5.71 | 5.75 | 8.72 | 8.72 | - | - | - | - | - | - | - | - | - | - | - | - | 0.35 | 0.00 | - | 4.10 | 9.00 | 0.70 | 4.50 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.32 | 17.77 |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-resize-mid.png (mid at 67 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-resize-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          43.48
  command p90 ms                          46.35
  frames                                  62.00
  tick ms/frame                            1.89
  frame p90 ms                             4.77
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.78
  interval p99 ms                         17.87
  interval max ms (median burst)          17.87
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
  batch_commit ms/frame                    1.10
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.20
  cloak ms (outside frames)               22.00
  session_begin ms (outside frames)        1.90
  session_overlays ms (outside frames)    17.40
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
  input->anim start ms                     0.36
  input->first frame ms                   47.91
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | resize | 43.48 | 46.35 | 62.00 | 1.89 | 4.77 | 2.00 | 5.71 | 5.78 | 17.87 | 17.87 | - | - | - | - | - | - | - | - | - | - | - | - | 1.10 | 0.03 | - | 4.20 | 22.00 | 1.90 | 17.40 | 0.05 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.36 | 47.91 |
Moved 3 window(s) to workspace '2' to switch to.
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-switch-mid.png (mid at 90 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-switch-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.14
  command p90 ms                          35.05
  frames                                  99.00
  tick ms/frame                            1.36
  frame p90 ms                             1.41
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         25.87
  interval max ms (median burst)          25.87
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
  dwm_flush ms (outside frames)            5.90
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
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | switch | 34.14 | 35.05 | 99.00 | 1.36 | 1.41 | 3.00 | 5.71 | 5.76 | 25.87 | 25.87 | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | - | 5.90 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
shots C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-minimize-mid.png (mid at 69 ms), C:\Users\googlers\.glzr\glazewm\compare-shots\20261117-110257\r1-cpu8-bdconfigboconfig-stand-ins-nopeeks-minimize-rest.png
label=stand-ins:nopeeks cpu8 bd-config bo-config #1 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          26.36
  command p90 ms                          31.05
  frames                                 143.00
  tick ms/frame                            1.37
  frame p90 ms                             3.17
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         59.77
  interval max ms (median burst)          59.77
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
  dwm_flush ms (outside frames)            4.30
  cloak ms (outside frames)               29.60
  session_begin ms (outside frames)        5.20
  session_overlays ms (outside frames)    19.60
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
  input->anim start ms                     6.48
  input->first frame ms                   69.33
| stand-ins:nopeeks cpu8 bd-config bo-config #1 | minimize | 26.36 | 31.05 | 143.00 | 1.37 | 3.17 | 1.00 | 5.71 | 5.77 | 59.77 | 59.77 | - | - | - | - | - | - | - | - | - | - | - | - | 0.75 | 0.02 | 1.00 | 4.30 | 29.60 | 5.20 | 19.60 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.48 | 69.33 |
tiles stand-ins:morepeeks cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:morepeeks cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          22.23
  command p90 ms                          23.50
  frames                                  63.00
  tick ms/frame                            0.95
  frame p90 ms                             1.08
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         12.86
  interval max ms (median burst)          12.86
  peek_frame ms/frame                      0.42
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
  cloak ms (outside frames)                7.70
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
| stand-ins:morepeeks cpu8 bd-config bo-config #2 | move | 22.23 | 23.50 | 63.00 | 0.95 | 1.08 | 0.00 | 5.71 | 5.75 | 12.86 | 12.86 | 0.42 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 3.80 | 7.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          71.02
  command p90 ms                          78.60
  frames                                  58.00
  tick ms/frame                            4.02
  frame p90 ms                             4.43
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.37
  interval p99 ms                         74.81
  interval max ms (median burst)          74.81
  peek_frame ms/frame                      2.57
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
  cloak ms (outside frames)               21.10
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
| stand-ins:morepeeks cpu8 bd-config bo-config #2 | resize | 71.02 | 78.60 | 58.00 | 4.02 | 4.43 | 4.00 | 5.72 | 6.37 | 74.81 | 74.81 | 2.57 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 10.30 | 21.10 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:morepeeks cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.35
  command p90 ms                          36.18
  frames                                 100.00
  tick ms/frame                            1.62
  frame p90 ms                             1.39
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         26.95
  interval max ms (median burst)          30.68
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
| stand-ins:morepeeks cpu8 bd-config bo-config #2 | switch | 34.35 | 36.18 | 100.00 | 1.62 | 1.39 | 4.00 | 5.72 | 5.76 | 26.95 | 30.68 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | - | 5.40 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          68.21
  command p90 ms                          73.51
  frames                                  60.00
  tick ms/frame                            3.67
  frame p90 ms                             4.29
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          8.88
  interval p99 ms                         85.43
  interval max ms (median burst)          85.43
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
  dwm_flush ms (outside frames)            8.00
  cloak ms (outside frames)               16.70
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
| stand-ins:morepeeks cpu8 bd-config bo-config #2 | minimize | 68.21 | 73.51 | 60.00 | 3.67 | 4.29 | 5.00 | 5.71 | 8.88 | 85.43 | 85.43 | 2.52 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 8.00 | 16.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles main cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.41
  command p90 ms                          18.20
  frames                                  63.00
  tick ms/frame                            0.67
  frame p90 ms                             1.26
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          7.97
  interval max ms (median burst)           7.97
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
  dwm_flush ms (outside frames)            5.50
  cloak ms (outside frames)                9.20
  session_begin ms (outside frames)        1.10
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
  input->anim start ms                     0.27
  input->first frame ms                   17.10
| main cpu8 bd-config bo-config #2 | move | 16.41 | 18.20 | 63.00 | 0.67 | 1.26 | 0.00 | 5.71 | 5.74 | 7.97 | 7.97 | - | - | - | - | - | - | - | - | - | - | - | - | 0.39 | 0.00 | 1.00 | 5.50 | 9.20 | 1.10 | 3.40 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.27 | 17.10 |
label=main cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          31.75
  command p90 ms                          35.37
  frames                                  63.00
  tick ms/frame                            2.46
  frame p90 ms                             4.23
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         29.19
  interval max ms (median burst)          29.19
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
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)               18.80
  session_begin ms (outside frames)        2.60
  session_overlays ms (outside frames)     7.90
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
  input->anim start ms                     0.27
  input->first frame ms                   38.09
| main cpu8 bd-config bo-config #2 | resize | 31.75 | 35.37 | 63.00 | 2.46 | 4.23 | 2.00 | 5.71 | 5.77 | 29.19 | 29.19 | - | - | - | - | - | - | - | - | - | - | - | - | 1.06 | 0.05 | 2.00 | 5.10 | 18.80 | 2.60 | 7.90 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.27 | 38.09 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.47
  command p90 ms                          27.23
  frames                                  97.00
  tick ms/frame                            1.85
  frame p90 ms                             3.02
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         59.04
  interval max ms (median burst)          60.41
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
| main cpu8 bd-config bo-config #2 | switch | 20.47 | 27.23 | 97.00 | 1.85 | 3.02 | 1.00 | 5.71 | 5.75 | 59.04 | 60.41 | - | - | - | - | - | - | - | - | - | - | - | - | 0.71 | 0.02 | 3.00 | 4.50 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=main cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          20.43
  command p90 ms                          21.55
  frames                                 149.00
  tick ms/frame                            1.64
  frame p90 ms                             4.70
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         35.75
  interval max ms (median burst)          35.75
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
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.60
  cloak ms (outside frames)               16.30
  session_begin ms (outside frames)        2.60
  session_overlays ms (outside frames)     9.20
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
  input->anim start ms                     7.07
  input->first frame ms                   45.88
| main cpu8 bd-config bo-config #2 | minimize | 20.43 | 21.55 | 149.00 | 1.64 | 4.70 | 3.00 | 5.71 | 5.77 | 35.75 | 35.75 | - | - | - | - | - | - | - | - | - | - | - | - | 0.81 | 0.04 | 4.00 | 4.60 | 16.30 | 2.60 | 9.20 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 7.07 | 45.88 |
tiles this cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          17.68
  command p90 ms                          18.73
  frames                                  63.00
  tick ms/frame                            0.42
  frame p90 ms                             0.54
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         12.45
  interval max ms (median burst)          12.45
  peek_frame ms/frame                      0.34
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.06
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.89
  pd_alpha ms (worst frame)                0.50
  pd_draw ms (worst frame)                 0.45
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
  command ms (WM side)                    17.30
  peek_new ms (outside frames)             2.30
  peek_place ms (outside frames)           1.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.20
  peek_handover ms (outside frames)        3.40
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              1.30
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.18
  input->first frame ms                   19.00
| this cpu8 bd-config bo-config #2 | move | 17.68 | 18.73 | 63.00 | 0.42 | 0.54 | 0.00 | 5.71 | 5.75 | 12.45 | 12.45 | 0.34 | 0.16 | 0.06 | 0.01 | 0.04 | 0.89 | 0.50 | 0.45 | - | - | - | - | - | - | - | 5.20 | 8.20 | - | - | - | 17.30 | 2.30 | 1.00 | - | - | - | 1.20 | 3.40 | 0.40 | 1.30 | - | - | - | - | - | 3.18 | 19.00 |
label=this cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          50.07
  command p90 ms                          57.45
  frames                                  60.00
  tick ms/frame                            2.63
  frame p90 ms                             4.01
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         36.32
  interval max ms (median burst)          36.32
  peek_frame ms/frame                      2.21
  peek_draw ms/frame                       1.49
  peek_track ms/frame                      0.30
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.14
  peek_draw ms (worst frame)              22.25
  pd_alpha ms (worst frame)                1.23
  pd_draw ms (worst frame)                20.93
  pd_fill ms (worst frame)                16.85
  fill_source ms (worst frame)             5.53
  ring_raise ms (worst frame)              5.48
  restack ms (worst frame)                 2.70
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            9.40
  cloak ms (outside frames)               17.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    49.70
  peek_new ms (outside frames)             3.30
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          12.00
  peek_handover ms (outside frames)       13.20
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.90
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.33
  input->first frame ms                   78.34
| this cpu8 bd-config bo-config #2 | resize | 50.07 | 57.45 | 60.00 | 2.63 | 4.01 | 3.00 | 5.71 | 5.79 | 36.32 | 36.32 | 2.21 | 1.49 | 0.30 | 0.14 | 0.14 | 22.25 | 1.23 | 20.93 | 16.85 | 5.53 | 5.48 | 2.70 | - | 0.03 | 2.00 | 9.40 | 17.90 | - | - | - | 49.70 | 3.30 | 2.10 | - | - | - | 12.00 | 13.20 | 0.10 | 0.90 | - | - | - | - | - | 3.33 | 78.34 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          28.19
  command p90 ms                          32.46
  frames                                  99.00
  tick ms/frame                            1.24
  frame p90 ms                             1.26
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         24.01
  interval max ms (median burst)          24.01
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
  dwm_flush ms (outside frames)            5.10
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    27.90
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.70
  restack ms (outside frames)              0.00
  ws_surrogates ms (outside frames)       12.60
  ws_park ms (outside frames)              2.20
  ws_show ms (outside frames)              7.60
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| this cpu8 bd-config bo-config #2 | switch | 28.19 | 32.46 | 99.00 | 1.24 | 1.26 | 4.00 | 5.71 | 5.76 | 24.01 | 24.01 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.04 | - | 5.10 | - | - | - | - | 27.90 | - | - | - | - | - | - | - | 0.00 | 0.70 | 0.00 | 12.60 | 2.20 | 7.60 | 0.40 | - | - |
label=this cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          58.24
  command p90 ms                          64.95
  frames                                  61.00
  tick ms/frame                            2.62
  frame p90 ms                             4.47
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.79
  interval p99 ms                         70.26
  interval max ms (median burst)          70.26
  peek_frame ms/frame                      2.19
  peek_draw ms/frame                       1.34
  peek_track ms/frame                      0.38
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.23
  peek_draw ms (worst frame)              17.83
  pd_alpha ms (worst frame)                0.93
  pd_draw ms (worst frame)                16.93
  pd_fill ms (worst frame)                12.75
  fill_source ms (worst frame)             2.88
  ring_raise ms (worst frame)              6.03
  restack ms (worst frame)                 3.08
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.80
  cloak ms (outside frames)               20.50
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    58.00
  peek_new ms (outside frames)             9.00
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          25.70
  peek_handover ms (outside frames)       19.10
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              2.30
  restack ms (outside frames)             10.20
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.22
  input->first frame ms                   75.97
| this cpu8 bd-config bo-config #2 | minimize | 58.24 | 64.95 | 61.00 | 2.62 | 4.47 | 3.00 | 5.71 | 5.79 | 70.26 | 70.26 | 2.19 | 1.34 | 0.38 | 0.14 | 0.23 | 17.83 | 0.93 | 16.93 | 12.75 | 2.88 | 6.03 | 3.08 | - | 0.03 | 2.00 | 10.80 | 20.50 | - | - | - | 58.00 | 9.00 | 1.80 | - | - | - | 25.70 | 19.10 | 0.40 | 2.30 | 10.20 | - | - | - | - | 10.22 | 75.97 |
tiles start-latency cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=start-latency cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.53
  command p90 ms                          22.41
  frames                                  63.00
  tick ms/frame                            0.96
  frame p90 ms                             0.92
  frames >2x budget                        1.00
  interval p50 ms                          5.72
  interval p90 ms                          5.76
  interval p99 ms                         13.35
  interval max ms (median burst)          13.35
  peek_frame ms/frame                      0.37
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.03
  peek_draw ms (worst frame)               0.85
  pd_alpha ms (worst frame)                0.47
  pd_draw ms (worst frame)                 0.44
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.10
  cloak ms (outside frames)                7.30
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    20.20
  peek_new ms (outside frames)             6.20
  peek_place ms (outside frames)           1.30
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.10
  peek_handover ms (outside frames)        3.10
  peek_raise ms (outside frames)           0.50
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     4.68
  input->first frame ms                   22.74
| start-latency cpu8 bd-config bo-config #2 | move | 20.53 | 22.41 | 63.00 | 0.96 | 0.92 | 1.00 | 5.72 | 5.76 | 13.35 | 13.35 | 0.37 | 0.16 | 0.05 | 0.01 | 0.03 | 0.85 | 0.47 | 0.44 | - | - | - | - | - | - | - | 4.10 | 7.30 | - | 0.00 | 0.00 | 20.20 | 6.20 | 1.30 | - | - | - | 1.10 | 3.10 | 0.50 | - | - | - | - | - | - | 4.68 | 22.74 |
label=start-latency cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          69.66
  command p90 ms                          81.70
  frames                                  59.00
  tick ms/frame                            3.86
  frame p90 ms                             5.01
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          6.99
  interval p99 ms                         50.45
  interval max ms (median burst)          50.45
  peek_frame ms/frame                      2.75
  peek_draw ms/frame                       1.59
  peek_track ms/frame                      0.39
  peek_ring ms/frame                       0.21
  peek_backdrop ms/frame                   0.19
  peek_draw ms (worst frame)              31.55
  pd_alpha ms (worst frame)                1.12
  pd_draw ms (worst frame)                30.42
  pd_fill ms (worst frame)                26.42
  fill_source ms (worst frame)             9.13
  ring_raise ms (worst frame)             10.11
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.06
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)               19.10
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    69.30
  peek_new ms (outside frames)            19.50
  peek_place ms (outside frames)           5.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          11.10
  peek_handover ms (outside frames)       13.70
  peek_raise ms (outside frames)           2.70
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     5.01
  input->first frame ms                  102.02
| start-latency cpu8 bd-config bo-config #2 | resize | 69.66 | 81.70 | 59.00 | 3.86 | 5.01 | 5.00 | 5.71 | 6.99 | 50.45 | 50.45 | 2.75 | 1.59 | 0.39 | 0.21 | 0.19 | 31.55 | 1.12 | 30.42 | 26.42 | 9.13 | 10.11 | - | - | 0.06 | 2.00 | 4.90 | 19.10 | - | 0.00 | 0.00 | 69.30 | 19.50 | 5.10 | - | - | - | 11.10 | 13.70 | 2.70 | - | - | - | - | - | - | 5.01 | 102.02 |
Moved 3 window(s) to workspace '2' to switch to.
label=start-latency cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          34.41
  command p90 ms                          35.89
  frames                                  99.00
  tick ms/frame                            1.61
  frame p90 ms                             1.60
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         30.29
  interval max ms (median burst)          30.53
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
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    33.80
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)       15.00
  ws_park ms (outside frames)              2.20
  ws_show ms (outside frames)              9.60
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| start-latency cpu8 bd-config bo-config #2 | switch | 34.41 | 35.89 | 99.00 | 1.61 | 1.60 | 3.00 | 5.71 | 5.75 | 30.29 | 30.53 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 6.00 | - | - | 0.00 | 0.00 | 33.80 | - | - | - | - | - | - | - | 0.00 | - | - | 15.00 | 2.20 | 9.60 | 0.40 | - | - |
label=start-latency cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          66.74
  command p90 ms                          72.63
  frames                                  59.00
  tick ms/frame                            3.40
  frame p90 ms                             5.25
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          8.37
  interval p99 ms                         81.66
  interval max ms (median burst)          81.66
  peek_frame ms/frame                      2.44
  peek_draw ms/frame                       1.40
  peek_track ms/frame                      0.40
  peek_ring ms/frame                       0.16
  peek_backdrop ms/frame                   0.24
  peek_draw ms (worst frame)              27.93
  pd_alpha ms (worst frame)                0.86
  pd_draw ms (worst frame)                26.41
  pd_fill ms (worst frame)                21.57
  fill_source ms (worst frame)             6.60
  ring_raise ms (worst frame)              8.97
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            8.80
  cloak ms (outside frames)               16.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    66.30
  peek_new ms (outside frames)            29.50
  peek_place ms (outside frames)           6.20
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          21.30
  peek_handover ms (outside frames)       16.80
  peek_raise ms (outside frames)           1.80
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.70
  input->first frame ms                   97.74
| start-latency cpu8 bd-config bo-config #2 | minimize | 66.74 | 72.63 | 59.00 | 3.40 | 5.25 | 5.00 | 5.71 | 8.37 | 81.66 | 81.66 | 2.44 | 1.40 | 0.40 | 0.16 | 0.24 | 27.93 | 0.86 | 26.41 | 21.57 | 6.60 | 8.97 | - | - | 0.03 | 2.00 | 8.80 | 16.80 | - | 0.00 | 0.00 | 66.30 | 29.50 | 6.20 | - | - | - | 21.30 | 16.80 | 1.80 | - | - | - | - | - | - | 10.70 | 97.74 |
tiles stand-ins:nopeeks cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          16.69
  command p90 ms                          17.10
  frames                                  63.00
  tick ms/frame                            0.59
  frame p90 ms                             1.18
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                          8.53
  interval max ms (median burst)           8.53
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
  ovl_region ms/frame                      0.01
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.70
  cloak ms (outside frames)                8.40
  session_begin ms (outside frames)        0.80
  session_overlays ms (outside frames)     4.90
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
  input->first frame ms                   17.24
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | move | 16.69 | 17.10 | 63.00 | 0.59 | 1.18 | 0.00 | 5.71 | 5.76 | 8.53 | 8.53 | - | - | - | - | - | - | - | - | - | - | - | - | 0.35 | 0.01 | - | 3.70 | 8.40 | 0.80 | 4.90 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.32 | 17.24 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          46.00
  command p90 ms                          47.38
  frames                                  63.00
  tick ms/frame                            2.04
  frame p90 ms                             5.31
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          6.15
  interval p99 ms                         21.61
  interval max ms (median burst)          21.61
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
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.60
  cloak ms (outside frames)               22.50
  session_begin ms (outside frames)        2.20
  session_overlays ms (outside frames)    18.30
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
  input->first frame ms                   50.64
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | resize | 46.00 | 47.38 | 63.00 | 2.04 | 5.31 | 2.00 | 5.71 | 6.15 | 21.61 | 21.61 | - | - | - | - | - | - | - | - | - | - | - | - | 1.12 | 0.03 | - | 3.60 | 22.50 | 2.20 | 18.30 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.32 | 50.64 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          32.73
  command p90 ms                          34.16
  frames                                 100.00
  tick ms/frame                            1.35
  frame p90 ms                             1.54
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         27.34
  interval max ms (median burst)          27.34
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
  input->anim start ms                     0.00
  input->first frame ms                    0.00
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | switch | 32.73 | 34.16 | 100.00 | 1.35 | 1.54 | 3.00 | 5.71 | 5.76 | 27.34 | 27.34 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.04 | - | 5.80 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          26.80
  command p90 ms                          30.01
  frames                                 142.00
  tick ms/frame                            1.34
  frame p90 ms                             3.52
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         49.16
  interval max ms (median burst)          49.16
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
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            3.20
  cloak ms (outside frames)               20.10
  session_begin ms (outside frames)        2.60
  session_overlays ms (outside frames)    20.10
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
  input->anim start ms                     6.72
  input->first frame ms                   57.53
| stand-ins:nopeeks cpu8 bd-config bo-config #2 | minimize | 26.80 | 30.01 | 142.00 | 1.34 | 3.52 | 0.00 | 5.71 | 5.76 | 49.16 | 49.16 | - | - | - | - | - | - | - | - | - | - | - | - | 0.72 | 0.02 | 1.00 | 3.20 | 20.10 | 2.60 | 20.10 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.72 | 57.53 |
tiles stand-ins cpu8 bd-config bo-config #2 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins cpu8 bd-config bo-config #2 scenario=move target=chrome bursts=10 load=0 reports=11 wall=12.3s
  command p50 ms                          21.01
  command p90 ms                          23.06
  frames                                  63.00
  tick ms/frame                            0.90
  frame p90 ms                             0.92
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         12.32
  interval max ms (median burst)          12.32
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
  dwm_flush ms (outside frames)            4.90
  cloak ms (outside frames)                7.70
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
| stand-ins cpu8 bd-config bo-config #2 | move | 21.01 | 23.06 | 63.00 | 0.90 | 0.92 | 0.00 | 5.71 | 5.75 | 12.32 | 12.32 | 0.37 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.90 | 7.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu8 bd-config bo-config #2 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          71.05
  command p90 ms                          76.44
  frames                                  60.00
  tick ms/frame                            3.99
  frame p90 ms                             4.09
  frames >2x budget                        4.00
  interval p50 ms                          5.72
  interval p90 ms                          6.64
  interval p99 ms                         78.06
  interval max ms (median burst)          78.06
  peek_frame ms/frame                      2.56
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
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.70
  cloak ms (outside frames)               21.10
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
| stand-ins cpu8 bd-config bo-config #2 | resize | 71.05 | 76.44 | 60.00 | 3.99 | 4.09 | 4.00 | 5.72 | 6.64 | 78.06 | 78.06 | 2.56 | - | - | - | - | - | - | - | - | - | - | - | - | 0.05 | 2.00 | 10.70 | 21.10 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins cpu8 bd-config bo-config #2 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.80
  command p90 ms                          35.43
  frames                                  99.00
  tick ms/frame                            1.60
  frame p90 ms                             1.33
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         28.63
  interval max ms (median burst)          28.63
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
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.90
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
| stand-ins cpu8 bd-config bo-config #2 | switch | 34.80 | 35.43 | 99.00 | 1.60 | 1.33 | 4.00 | 5.71 | 5.75 | 28.63 | 28.63 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | - | 5.90 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu8 bd-config bo-config #2 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          66.93
  command p90 ms                          74.41
  frames                                  59.00
  tick ms/frame                            3.58
  frame p90 ms                             4.77
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          8.91
  interval p99 ms                         84.06
  interval max ms (median burst)          84.06
  peek_frame ms/frame                      2.42
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
  dwm_flush ms (outside frames)           12.00
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
| stand-ins cpu8 bd-config bo-config #2 | minimize | 66.93 | 74.41 | 59.00 | 3.58 | 4.77 | 5.00 | 5.71 | 8.91 | 84.06 | 84.06 | 2.42 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 12.00 | 18.00 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles main cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=main cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          15.88
  command p90 ms                          16.68
  frames                                  63.00
  tick ms/frame                            0.66
  frame p90 ms                             1.27
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.74
  interval p99 ms                          8.03
  interval max ms (median burst)           8.03
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
  dwm_flush ms (outside frames)            5.40
  cloak ms (outside frames)                8.80
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
  input->anim start ms                     0.25
  input->first frame ms                   16.83
| main cpu8 bd-config bo-config #3 | move | 15.88 | 16.68 | 63.00 | 0.66 | 1.27 | 0.00 | 5.71 | 5.74 | 8.03 | 8.03 | - | - | - | - | - | - | - | - | - | - | - | - | 0.38 | 0.00 | 1.00 | 5.40 | 8.80 | 1.00 | 2.80 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.25 | 16.83 |
label=main cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.4s
  command p50 ms                          30.97
  command p90 ms                          34.45
  frames                                  63.00
  tick ms/frame                            2.45
  frame p90 ms                             4.30
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         30.16
  interval max ms (median burst)          30.16
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
  batch_commit ms/frame                    1.07
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.00
  cloak ms (outside frames)               18.30
  session_begin ms (outside frames)        2.60
  session_overlays ms (outside frames)     8.30
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
  input->first frame ms                   34.98
| main cpu8 bd-config bo-config #3 | resize | 30.97 | 34.45 | 63.00 | 2.45 | 4.30 | 2.00 | 5.71 | 5.77 | 30.16 | 30.16 | - | - | - | - | - | - | - | - | - | - | - | - | 1.07 | 0.04 | 2.00 | 4.00 | 18.30 | 2.60 | 8.30 | 0.03 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.23 | 34.98 |
Moved 3 window(s) to workspace '2' to switch to.
label=main cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          21.27
  command p90 ms                          23.14
  frames                                  97.00
  tick ms/frame                            1.84
  frame p90 ms                             2.74
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         61.45
  interval max ms (median burst)          61.45
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
  dwm_flush calls (in frames)              3.00
  dwm_flush ms (outside frames)            5.40
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
| main cpu8 bd-config bo-config #3 | switch | 21.27 | 23.14 | 97.00 | 1.84 | 2.74 | 1.00 | 5.71 | 5.75 | 61.45 | 61.45 | - | - | - | - | - | - | - | - | - | - | - | - | 0.70 | 0.03 | 3.00 | 5.40 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=main cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.2s
  command p50 ms                          18.32
  command p90 ms                          20.76
  frames                                 150.00
  tick ms/frame                            1.67
  frame p90 ms                             4.35
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          6.05
  interval p99 ms                         34.55
  interval max ms (median burst)          34.55
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
  batch_commit ms/frame                    0.84
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              4.00
  dwm_flush ms (outside frames)            4.50
  cloak ms (outside frames)               15.80
  session_begin ms (outside frames)        2.30
  session_overlays ms (outside frames)     7.90
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
  input->anim start ms                     7.49
  input->first frame ms                   44.51
| main cpu8 bd-config bo-config #3 | minimize | 18.32 | 20.76 | 150.00 | 1.67 | 4.35 | 2.00 | 5.71 | 6.05 | 34.55 | 34.55 | - | - | - | - | - | - | - | - | - | - | - | - | 0.84 | 0.03 | 4.00 | 4.50 | 15.80 | 2.30 | 7.90 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 7.49 | 44.51 |
tiles this cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=this cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          18.23
  command p90 ms                          22.56
  frames                                  63.00
  tick ms/frame                            0.42
  frame p90 ms                             0.51
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         12.05
  interval max ms (median burst)          12.05
  peek_frame ms/frame                      0.30
  peek_draw ms/frame                       0.17
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.86
  pd_alpha ms (worst frame)                0.48
  pd_draw ms (worst frame)                 0.44
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            5.80
  cloak ms (outside frames)                8.70
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    17.90
  peek_new ms (outside frames)             2.20
  peek_place ms (outside frames)           1.00
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.50
  peek_handover ms (outside frames)        3.20
  peek_raise ms (outside frames)           0.40
  surface ms (outside frames)              1.20
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.21
  input->first frame ms                   19.30
| this cpu8 bd-config bo-config #3 | move | 18.23 | 22.56 | 63.00 | 0.42 | 0.51 | 0.00 | 5.71 | 5.75 | 12.05 | 12.05 | 0.30 | 0.17 | 0.05 | 0.01 | 0.04 | 0.86 | 0.48 | 0.44 | - | - | - | - | - | - | - | 5.80 | 8.70 | - | - | - | 17.90 | 2.20 | 1.00 | - | - | - | 1.50 | 3.20 | 0.40 | 1.20 | - | - | - | - | - | 3.21 | 19.30 |
label=this cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.6s
  command p50 ms                          49.92
  command p90 ms                          57.42
  frames                                  61.00
  tick ms/frame                            2.65
  frame p90 ms                             3.99
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.80
  interval p99 ms                         33.50
  interval max ms (median burst)          33.50
  peek_frame ms/frame                      2.15
  peek_draw ms/frame                       1.46
  peek_track ms/frame                      0.30
  peek_ring ms/frame                       0.14
  peek_backdrop ms/frame                   0.14
  peek_draw ms (worst frame)              21.25
  pd_alpha ms (worst frame)                1.37
  pd_draw ms (worst frame)                19.76
  pd_fill ms (worst frame)                15.33
  fill_source ms (worst frame)             4.59
  ring_raise ms (worst frame)              7.19
  restack ms (worst frame)                 3.09
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            5.80
  cloak ms (outside frames)               17.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    49.60
  peek_new ms (outside frames)             3.70
  peek_place ms (outside frames)           2.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          12.10
  peek_handover ms (outside frames)       13.10
  peek_raise ms (outside frames)           0.10
  surface ms (outside frames)              0.80
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     3.51
  input->first frame ms                   72.96
| this cpu8 bd-config bo-config #3 | resize | 49.92 | 57.42 | 61.00 | 2.65 | 3.99 | 3.00 | 5.72 | 5.80 | 33.50 | 33.50 | 2.15 | 1.46 | 0.30 | 0.14 | 0.14 | 21.25 | 1.37 | 19.76 | 15.33 | 4.59 | 7.19 | 3.09 | - | 0.04 | 2.00 | 5.80 | 17.80 | - | - | - | 49.60 | 3.70 | 2.10 | - | - | - | 12.10 | 13.10 | 0.10 | 0.80 | - | - | - | - | - | 3.51 | 72.96 |
Moved 3 window(s) to workspace '2' to switch to.
label=this cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          30.06
  command p90 ms                          31.20
  frames                                  99.00
  tick ms/frame                            1.23
  frame p90 ms                             1.21
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         22.13
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
  dwm_flush ms (outside frames)            6.00
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    29.70
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)              0.70
  restack ms (outside frames)              0.00
  ws_surrogates ms (outside frames)       12.00
  ws_park ms (outside frames)              2.20
  ws_show ms (outside frames)              8.70
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| this cpu8 bd-config bo-config #3 | switch | 30.06 | 31.20 | 99.00 | 1.23 | 1.21 | 3.00 | 5.71 | 5.76 | 22.13 | 23.82 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.03 | - | 6.00 | - | - | - | - | 29.70 | - | - | - | - | - | - | - | 0.00 | 0.70 | 0.00 | 12.00 | 2.20 | 8.70 | 0.40 | - | - |
label=this cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          52.41
  command p90 ms                          61.12
  frames                                  61.00
  tick ms/frame                            2.54
  frame p90 ms                             4.05
  frames >2x budget                        3.00
  interval p50 ms                          5.72
  interval p90 ms                          5.77
  interval p99 ms                         62.63
  interval max ms (median burst)          62.63
  peek_frame ms/frame                      2.06
  peek_draw ms/frame                       1.35
  peek_track ms/frame                      0.41
  peek_ring ms/frame                       0.15
  peek_backdrop ms/frame                   0.23
  peek_draw ms (worst frame)              19.48
  pd_alpha ms (worst frame)                0.95
  pd_draw ms (worst frame)                18.52
  pd_fill ms (worst frame)                14.46
  fill_source ms (worst frame)             4.69
  ring_raise ms (worst frame)              5.12
  restack ms (worst frame)                 2.70
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           11.50
  cloak ms (outside frames)               20.20
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)        -
  session_overlays ms/frame                   -
  command ms (WM side)                    52.10
  peek_new ms (outside frames)             8.10
  peek_place ms (outside frames)           1.80
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          23.40
  peek_handover ms (outside frames)       17.40
  peek_raise ms (outside frames)           0.30
  surface ms (outside frames)              2.40
  restack ms (outside frames)              9.70
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     9.56
  input->first frame ms                   76.44
| this cpu8 bd-config bo-config #3 | minimize | 52.41 | 61.12 | 61.00 | 2.54 | 4.05 | 3.00 | 5.72 | 5.77 | 62.63 | 62.63 | 2.06 | 1.35 | 0.41 | 0.15 | 0.23 | 19.48 | 0.95 | 18.52 | 14.46 | 4.69 | 5.12 | 2.70 | - | 0.03 | 2.00 | 11.50 | 20.20 | - | - | - | 52.10 | 8.10 | 1.80 | - | - | - | 23.40 | 17.40 | 0.30 | 2.40 | 9.70 | - | - | - | - | 9.56 | 76.44 |
tiles start-latency cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=start-latency cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          22.88
  command p90 ms                          23.48
  frames                                  63.00
  tick ms/frame                            0.93
  frame p90 ms                             0.92
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         13.08
  interval max ms (median burst)          13.08
  peek_frame ms/frame                      0.38
  peek_draw ms/frame                       0.16
  peek_track ms/frame                      0.05
  peek_ring ms/frame                       0.01
  peek_backdrop ms/frame                   0.04
  peek_draw ms (worst frame)               0.86
  pd_alpha ms (worst frame)                0.46
  pd_draw ms (worst frame)                 0.46
  pd_fill ms (worst frame)                    -
  fill_source ms (worst frame)                -
  ring_raise ms (worst frame)                 -
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                         -
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.60
  cloak ms (outside frames)                7.80
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    22.60
  peek_new ms (outside frames)             5.50
  peek_place ms (outside frames)           1.40
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)           1.20
  peek_handover ms (outside frames)        3.00
  peek_raise ms (outside frames)           0.60
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     4.29
  input->first frame ms                   25.36
| start-latency cpu8 bd-config bo-config #3 | move | 22.88 | 23.48 | 63.00 | 0.93 | 0.92 | 1.00 | 5.71 | 5.75 | 13.08 | 13.08 | 0.38 | 0.16 | 0.05 | 0.01 | 0.04 | 0.86 | 0.46 | 0.46 | - | - | - | - | - | - | - | 4.60 | 7.80 | - | 0.00 | 0.00 | 22.60 | 5.50 | 1.40 | - | - | - | 1.20 | 3.00 | 0.60 | - | - | - | - | - | - | 4.29 | 25.36 |
label=start-latency cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          72.47
  command p90 ms                          76.96
  frames                                  59.00
  tick ms/frame                            3.67
  frame p90 ms                             4.28
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          6.55
  interval p99 ms                         53.20
  interval max ms (median burst)          53.20
  peek_frame ms/frame                      2.54
  peek_draw ms/frame                       1.54
  peek_track ms/frame                      0.31
  peek_ring ms/frame                       0.17
  peek_backdrop ms/frame                   0.15
  peek_draw ms (worst frame)              30.95
  pd_alpha ms (worst frame)                1.22
  pd_draw ms (worst frame)                29.60
  pd_fill ms (worst frame)                25.56
  fill_source ms (worst frame)             8.46
  ring_raise ms (worst frame)             10.25
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.05
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)            4.80
  cloak ms (outside frames)               17.90
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    72.10
  peek_new ms (outside frames)            18.80
  peek_place ms (outside frames)           5.10
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          11.40
  peek_handover ms (outside frames)       14.00
  peek_raise ms (outside frames)           2.70
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                     4.73
  input->first frame ms                  103.55
| start-latency cpu8 bd-config bo-config #3 | resize | 72.47 | 76.96 | 59.00 | 3.67 | 4.28 | 4.00 | 5.71 | 6.55 | 53.20 | 53.20 | 2.54 | 1.54 | 0.31 | 0.17 | 0.15 | 30.95 | 1.22 | 29.60 | 25.56 | 8.46 | 10.25 | - | - | 0.05 | 2.00 | 4.80 | 17.90 | - | 0.00 | 0.00 | 72.10 | 18.80 | 5.10 | - | - | - | 11.40 | 14.00 | 2.70 | - | - | - | - | - | - | 4.73 | 103.55 |
Moved 3 window(s) to workspace '2' to switch to.
label=start-latency cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          33.13
  command p90 ms                          36.75
  frames                                  99.00
  tick ms/frame                            1.59
  frame p90 ms                             1.45
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         27.85
  interval max ms (median burst)          29.26
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
  dwm_flush ms (outside frames)            5.80
  cloak ms (outside frames)                   -
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    32.80
  peek_new ms (outside frames)                -
  peek_place ms (outside frames)              -
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)              -
  peek_handover ms (outside frames)           -
  peek_raise ms (outside frames)           0.00
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)       14.80
  ws_park ms (outside frames)              2.30
  ws_show ms (outside frames)              9.40
  ws_start ms (outside frames)             0.40
  input->anim start ms                        -
  input->first frame ms                       -
| start-latency cpu8 bd-config bo-config #3 | switch | 33.13 | 36.75 | 99.00 | 1.59 | 1.45 | 3.00 | 5.71 | 5.76 | 27.85 | 29.26 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | - | - | 0.04 | - | 5.80 | - | - | 0.00 | 0.00 | 32.80 | - | - | - | - | - | - | - | 0.00 | - | - | 14.80 | 2.30 | 9.40 | 0.40 | - | - |
label=start-latency cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          62.58
  command p90 ms                          68.71
  frames                                  60.00
  tick ms/frame                            3.44
  frame p90 ms                             4.89
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          8.23
  interval p99 ms                         83.20
  interval max ms (median burst)          83.20
  peek_frame ms/frame                      2.58
  peek_draw ms/frame                       1.46
  peek_track ms/frame                      0.39
  peek_ring ms/frame                       0.16
  peek_backdrop ms/frame                   0.22
  peek_draw ms (worst frame)              28.35
  pd_alpha ms (worst frame)                0.94
  pd_draw ms (worst frame)                27.63
  pd_fill ms (worst frame)                22.79
  fill_source ms (worst frame)             6.16
  ring_raise ms (worst frame)              9.11
  restack ms (worst frame)                    -
  batch_commit ms/frame                       -
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)              2.00
  dwm_flush ms (outside frames)           10.00
  cloak ms (outside frames)               15.00
  session_begin ms (outside frames)           -
  session_overlays ms (outside frames)     0.00
  session_overlays ms/frame                0.00
  command ms (WM side)                    62.30
  peek_new ms (outside frames)            26.30
  peek_place ms (outside frames)           6.30
  pd_fill ms (outside frames)                 -
  fill_source ms (outside frames)             -
  ring_raise ms (outside frames)              -
  peek_cover ms (outside frames)          20.10
  peek_handover ms (outside frames)       15.20
  peek_raise ms (outside frames)           2.40
  surface ms (outside frames)                 -
  restack ms (outside frames)                 -
  ws_surrogates ms (outside frames)           -
  ws_park ms (outside frames)                 -
  ws_show ms (outside frames)                 -
  ws_start ms (outside frames)                -
  input->anim start ms                    10.74
  input->first frame ms                   97.23
| start-latency cpu8 bd-config bo-config #3 | minimize | 62.58 | 68.71 | 60.00 | 3.44 | 4.89 | 5.00 | 5.71 | 8.23 | 83.20 | 83.20 | 2.58 | 1.46 | 0.39 | 0.16 | 0.22 | 28.35 | 0.94 | 27.63 | 22.79 | 6.16 | 9.11 | - | - | 0.03 | 2.00 | 10.00 | 15.00 | - | 0.00 | 0.00 | 62.30 | 26.30 | 6.30 | - | - | - | 20.10 | 15.20 | 2.40 | - | - | - | - | - | - | 10.74 | 97.23 |
tiles stand-ins:nopeeks cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          15.82
  command p90 ms                          18.24
  frames                                  63.00
  tick ms/frame                            0.60
  frame p90 ms                             1.20
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                          8.11
  interval max ms (median burst)           8.11
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
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.50
  cloak ms (outside frames)                8.20
  session_begin ms (outside frames)        0.70
  session_overlays ms (outside frames)     4.80
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
  input->first frame ms                   16.59
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | move | 15.82 | 18.24 | 63.00 | 0.60 | 1.20 | 0.00 | 5.71 | 5.75 | 8.11 | 8.11 | - | - | - | - | - | - | - | - | - | - | - | - | 0.35 | 0.00 | - | 3.50 | 8.20 | 0.70 | 4.80 | 0.02 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.32 | 16.59 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          44.48
  command p90 ms                          45.53
  frames                                  62.00
  tick ms/frame                            1.83
  frame p90 ms                             4.61
  frames >2x budget                        2.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         17.93
  interval max ms (median burst)          17.93
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
  batch_commit ms/frame                    1.10
  ovl_region ms/frame                      0.03
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            3.30
  cloak ms (outside frames)               21.40
  session_begin ms (outside frames)        1.90
  session_overlays ms (outside frames)    17.50
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
  input->first frame ms                   48.78
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | resize | 44.48 | 45.53 | 62.00 | 1.83 | 4.61 | 2.00 | 5.71 | 5.77 | 17.93 | 17.93 | - | - | - | - | - | - | - | - | - | - | - | - | 1.10 | 0.03 | - | 3.30 | 21.40 | 1.90 | 17.50 | 0.06 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.31 | 48.78 |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          34.59
  command p90 ms                          36.42
  frames                                  99.00
  tick ms/frame                            1.38
  frame p90 ms                             1.51
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         27.00
  interval max ms (median burst)          27.00
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
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | switch | 34.59 | 36.42 | 99.00 | 1.38 | 1.51 | 3.00 | 5.71 | 5.76 | 27.00 | 27.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.03 | - | 6.00 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 0.00 | 0.00 |
label=stand-ins:nopeeks cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          26.76
  command p90 ms                          30.77
  frames                                 143.00
  tick ms/frame                            1.33
  frame p90 ms                             3.07
  frames >2x budget                        1.00
  interval p50 ms                          5.71
  interval p90 ms                          5.77
  interval p99 ms                         52.81
  interval max ms (median burst)          52.81
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
  dwm_flush calls (in frames)              1.00
  dwm_flush ms (outside frames)            5.30
  cloak ms (outside frames)               25.90
  session_begin ms (outside frames)        3.20
  session_overlays ms (outside frames)    17.70
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
  input->anim start ms                     6.57
  input->first frame ms                   64.12
| stand-ins:nopeeks cpu8 bd-config bo-config #3 | minimize | 26.76 | 30.77 | 143.00 | 1.33 | 3.07 | 1.00 | 5.71 | 5.77 | 52.81 | 52.81 | - | - | - | - | - | - | - | - | - | - | - | - | 0.73 | 0.02 | 1.00 | 5.30 | 25.90 | 3.20 | 17.70 | 0.04 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 6.57 | 64.12 |
tiles stand-ins cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.71
  command p90 ms                          22.30
  frames                                  63.00
  tick ms/frame                            0.97
  frame p90 ms                             1.05
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         13.05
  interval max ms (median burst)          13.05
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
  dwm_flush ms (outside frames)            4.40
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
| stand-ins cpu8 bd-config bo-config #3 | move | 20.71 | 22.30 | 63.00 | 0.97 | 1.05 | 0.00 | 5.71 | 5.75 | 13.05 | 13.05 | 0.37 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.40 | 7.40 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          66.40
  command p90 ms                          72.91
  frames                                  59.00
  tick ms/frame                            3.78
  frame p90 ms                             4.24
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          6.04
  interval p99 ms                         58.86
  interval max ms (median burst)          58.86
  peek_frame ms/frame                      2.45
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
  dwm_flush ms (outside frames)            3.70
  cloak ms (outside frames)               19.20
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
| stand-ins cpu8 bd-config bo-config #3 | resize | 66.40 | 72.91 | 59.00 | 3.78 | 4.24 | 4.00 | 5.71 | 6.04 | 58.86 | 58.86 | 2.45 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 3.70 | 19.20 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          33.36
  command p90 ms                          36.41
  frames                                 100.00
  tick ms/frame                            1.62
  frame p90 ms                             1.47
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.75
  interval p99 ms                         24.75
  interval max ms (median burst)          29.23
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
| stand-ins cpu8 bd-config bo-config #3 | switch | 33.36 | 36.41 | 100.00 | 1.62 | 1.47 | 3.00 | 5.71 | 5.75 | 24.75 | 29.23 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | - | 5.80 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          63.14
  command p90 ms                          74.62
  frames                                  60.00
  tick ms/frame                            3.61
  frame p90 ms                             4.42
  frames >2x budget                        5.00
  interval p50 ms                          5.72
  interval p90 ms                          7.28
  interval p99 ms                         89.09
  interval max ms (median burst)          89.09
  peek_frame ms/frame                      2.29
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
  dwm_flush ms (outside frames)           11.30
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
| stand-ins cpu8 bd-config bo-config #3 | minimize | 63.14 | 74.62 | 60.00 | 3.61 | 4.42 | 5.00 | 5.72 | 7.28 | 89.09 | 89.09 | 2.29 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 11.30 | 17.70 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
tiles stand-ins:morepeeks cpu8 bd-config bo-config #3 ws 1: Notepad, WindowsTerminal, WindowsTerminal, chrome, chrome, FPilot
label=stand-ins:morepeeks cpu8 bd-config bo-config #3 scenario=move target=chrome bursts=10 load=0 reports=10 wall=12.3s
  command p50 ms                          20.93
  command p90 ms                          21.43
  frames                                  63.00
  tick ms/frame                            0.96
  frame p90 ms                             1.04
  frames >2x budget                        0.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         12.95
  interval max ms (median burst)          12.95
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
  cloak ms (outside frames)                7.20
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
| stand-ins:morepeeks cpu8 bd-config bo-config #3 | move | 20.93 | 21.43 | 63.00 | 0.96 | 1.04 | 0.00 | 5.71 | 5.76 | 12.95 | 12.95 | 0.39 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | 4.10 | 7.20 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu8 bd-config bo-config #3 scenario=resize target=chrome bursts=10 load=0 reports=10 wall=12.8s
  command p50 ms                          70.01
  command p90 ms                          78.18
  frames                                  59.00
  tick ms/frame                            3.74
  frame p90 ms                             4.04
  frames >2x budget                        5.00
  interval p50 ms                          5.71
  interval p90 ms                          5.80
  interval p99 ms                         56.32
  interval max ms (median burst)          56.32
  peek_frame ms/frame                      2.54
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
  dwm_flush ms (outside frames)            5.80
  cloak ms (outside frames)               18.10
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
| stand-ins:morepeeks cpu8 bd-config bo-config #3 | resize | 70.01 | 78.18 | 59.00 | 3.74 | 4.04 | 5.00 | 5.71 | 5.80 | 56.32 | 56.32 | 2.54 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | 2.00 | 5.80 | 18.10 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
Moved 3 window(s) to workspace '2' to switch to.
label=stand-ins:morepeeks cpu8 bd-config bo-config #3 scenario=switch target=1<->2 bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          32.81
  command p90 ms                          34.45
  frames                                 100.00
  tick ms/frame                            1.57
  frame p90 ms                             1.33
  frames >2x budget                        3.00
  interval p50 ms                          5.71
  interval p90 ms                          5.76
  interval p99 ms                         28.23
  interval max ms (median burst)          29.15
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
  ovl_region ms/frame                      0.04
  dwm_flush calls (in frames)                 -
  dwm_flush ms (outside frames)            4.80
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
| stand-ins:morepeeks cpu8 bd-config bo-config #3 | switch | 32.81 | 34.45 | 100.00 | 1.57 | 1.33 | 3.00 | 5.71 | 5.76 | 28.23 | 29.15 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | 0.04 | - | 4.80 | - | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |
label=stand-ins:morepeeks cpu8 bd-config bo-config #3 scenario=minimize target=chrome bursts=10 load=0 reports=10 wall=12.5s
  command p50 ms                          64.68
  command p90 ms                          72.45
  frames                                  60.00
  tick ms/frame                            3.53
  frame p90 ms                             5.05
  frames >2x budget                        4.00
  interval p50 ms                          5.71
  interval p90 ms                          7.72
  interval p99 ms                         83.01
  interval max ms (median burst)          83.01
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
  dwm_flush ms (outside frames)            8.90
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
| stand-ins:morepeeks cpu8 bd-config bo-config #3 | minimize | 64.68 | 72.45 | 60.00 | 3.53 | 5.05 | 4.00 | 5.71 | 7.72 | 83.01 | 83.01 | 2.43 | - | - | - | - | - | - | - | - | - | - | - | - | 0.03 | 2.00 | 8.90 | 16.10 | - | 0.00 | 0.00 | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - | - |

## move

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 17.41 | 21.55 | 16.72 | — | — | 15.08 |
| cpu0 bd-config bo-config | frame p90 ms | 0.50 | 0.87 | 1.09 | 0.89 | 0.92 | 1.16 |
| cpu0 bd-config bo-config | interval p90 ms | 5.74 | 5.74 | 5.74 | 5.75 | 5.75 | 5.74 |
| cpu0 bd-config bo-config | interval p99 ms | 10.29 | 10.95 | 7.67 | 10.82 | 10.91 | 7.47 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 2.00 | 5.40 | — | — | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | — | 0.70 | — | — | 0.90 |
| cpu0 bd-config bo-config | restack ms (outside frames) | — | — | — | — | — | — |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 19.08 | 22.74 | 17.24 | — | — | 16.83 |
| cpu8 bd-config bo-config | frame p90 ms | 0.53 | 0.92 | 1.19 | 0.97 | 1.04 | 1.26 |
| cpu8 bd-config bo-config | interval p90 ms | 5.75 | 5.76 | 5.75 | 5.75 | 5.76 | 5.74 |
| cpu8 bd-config bo-config | interval p99 ms | 12.36 | 13.08 | 8.53 | 12.92 | 12.86 | 8.03 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 2.30 | 5.80 | — | — | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | — | 0.70 | — | — | 1.10 |
| cpu8 bd-config bo-config | restack ms (outside frames) | — | — | — | — | — | — |

### move, cpu0 bd-config bo-config

| metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 16.06 | 19.59 | 16.17 | 19.23 | 19.52 | 14.35 |
| command p90 ms | 18.44 | 20.51 | 17.18 | 20.57 | 20.72 | 15.60 |
| frames | 63.00 | 63.00 | 62.00 | 63.00 | 63.00 | 63.00 |
| tick ms/frame | 0.38 | 0.84 | 0.54 | 0.84 | 0.86 | 0.63 |
| frame p90 ms | 0.50 | 0.87 | 1.09 | 0.89 | 0.92 | 1.16 |
| frames >2x budget | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.74 | 5.74 | 5.74 | 5.75 | 5.75 | 5.74 |
| interval p99 ms | 10.29 | 10.95 | 7.67 | 10.82 | 10.91 | 7.47 |
| interval max ms (median burst) | 10.29 | 10.95 | 7.67 | 10.82 | 10.91 | 7.47 |
| peek_frame ms/frame | 0.36 | 0.39 | — | 0.38 | 0.38 | — |
| peek_draw ms/frame | 0.16 | 0.16 | — | — | — | — |
| peek_track ms/frame | 0.05 | 0.05 | — | — | — | — |
| peek_ring ms/frame | 0.01 | 0.02 | — | — | — | — |
| peek_backdrop ms/frame | 0.03 | 0.03 | — | — | — | — |
| peek_draw ms (worst frame) | 0.75 | 0.76 | — | — | — | — |
| pd_alpha ms (worst frame) | 0.41 | 0.41 | — | — | — | — |
| pd_draw ms (worst frame) | 0.48 | 0.46 | — | — | — | — |
| pd_fill ms (worst frame) | — | — | — | — | — | — |
| fill_source ms (worst frame) | — | — | — | — | — | — |
| ring_raise ms (worst frame) | — | — | — | — | — | — |
| restack ms (worst frame) | — | — | — | — | — | — |
| batch_commit ms/frame | — | — | 0.32 | — | — | 0.36 |
| ovl_region ms/frame | — | — | 0.00 | — | — | 0.00 |
| dwm_flush calls (in frames) | — | — | — | — | — | 1.00 |
| dwm_flush ms (outside frames) | 4.90 | 4.40 | 4.70 | 4.80 | 4.50 | 5.60 |
| cloak ms (outside frames) | 7.60 | 7.30 | 9.10 | 7.70 | 7.50 | 8.30 |
| session_begin ms (outside frames) | — | — | 0.70 | — | — | 0.90 |
| session_overlays ms (outside frames) | — | 0.00 | 4.50 | 0.00 | 0.00 | 3.00 |
| session_overlays ms/frame | — | 0.00 | 0.02 | 0.00 | 0.00 | 0.02 |
| command ms (WM side) | 15.80 | 19.30 | — | — | — | — |
| peek_new ms (outside frames) | 2.00 | 5.40 | — | — | — | — |
| peek_place ms (outside frames) | 0.90 | 1.20 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — | — |
| peek_cover ms (outside frames) | 1.20 | 1.10 | — | — | — | — |
| peek_handover ms (outside frames) | 2.60 | 2.50 | — | — | — | — |
| peek_raise ms (outside frames) | 0.30 | 0.50 | — | — | — | — |
| surface ms (outside frames) | 1.00 | — | — | — | — | — |
| restack ms (outside frames) | — | — | — | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — | — | — | — |
| ws_park ms (outside frames) | — | — | — | — | — | — |
| ws_show ms (outside frames) | — | — | — | — | — | — |
| ws_start ms (outside frames) | — | — | — | — | — | — |
| input->anim start ms | 2.89 | 4.29 | 0.30 | — | — | 0.23 |
| input->first frame ms | 17.41 | 21.55 | 16.72 | — | — | 15.08 |

### move, cpu8 bd-config bo-config

| metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 17.82 | 20.53 | 16.69 | 21.01 | 22.02 | 15.88 |
| command p90 ms | 18.73 | 23.18 | 18.24 | 22.30 | 23.47 | 16.71 |
| frames | 63.00 | 63.00 | 63.00 | 63.00 | 63.00 | 63.00 |
| tick ms/frame | 0.42 | 0.96 | 0.59 | 0.93 | 0.96 | 0.66 |
| frame p90 ms | 0.53 | 0.92 | 1.19 | 0.97 | 1.04 | 1.26 |
| frames >2x budget | 0.00 | 1.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.75 | 5.76 | 5.75 | 5.75 | 5.76 | 5.74 |
| interval p99 ms | 12.36 | 13.08 | 8.53 | 12.92 | 12.86 | 8.03 |
| interval max ms (median burst) | 12.36 | 13.08 | 8.53 | 12.92 | 12.86 | 8.03 |
| peek_frame ms/frame | 0.31 | 0.37 | — | 0.37 | 0.39 | — |
| peek_draw ms/frame | 0.16 | 0.16 | — | — | — | — |
| peek_track ms/frame | 0.06 | 0.05 | — | — | — | — |
| peek_ring ms/frame | 0.01 | 0.01 | — | — | — | — |
| peek_backdrop ms/frame | 0.04 | 0.03 | — | — | — | — |
| peek_draw ms (worst frame) | 0.86 | 0.86 | — | — | — | — |
| pd_alpha ms (worst frame) | 0.50 | 0.46 | — | — | — | — |
| pd_draw ms (worst frame) | 0.45 | 0.46 | — | — | — | — |
| pd_fill ms (worst frame) | — | — | — | — | — | — |
| fill_source ms (worst frame) | — | — | — | — | — | — |
| ring_raise ms (worst frame) | — | — | — | — | — | — |
| restack ms (worst frame) | — | — | — | — | — | — |
| batch_commit ms/frame | — | — | 0.35 | — | — | 0.38 |
| ovl_region ms/frame | — | — | 0.00 | — | — | 0.00 |
| dwm_flush calls (in frames) | — | — | — | — | — | 1.00 |
| dwm_flush ms (outside frames) | 5.20 | 4.40 | 3.70 | 4.40 | 4.10 | 5.50 |
| cloak ms (outside frames) | 8.20 | 7.40 | 8.40 | 7.40 | 7.20 | 9.20 |
| session_begin ms (outside frames) | — | — | 0.70 | — | — | 1.10 |
| session_overlays ms (outside frames) | — | 0.00 | 4.80 | 0.00 | 0.00 | 3.40 |
| session_overlays ms/frame | — | 0.00 | 0.02 | 0.00 | 0.00 | 0.02 |
| command ms (WM side) | 17.50 | 20.20 | — | — | — | — |
| peek_new ms (outside frames) | 2.30 | 5.80 | — | — | — | — |
| peek_place ms (outside frames) | 1.00 | 1.30 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — | — |
| peek_cover ms (outside frames) | 1.30 | 1.20 | — | — | — | — |
| peek_handover ms (outside frames) | 3.20 | 3.10 | — | — | — | — |
| peek_raise ms (outside frames) | 0.40 | 0.50 | — | — | — | — |
| surface ms (outside frames) | 1.30 | — | — | — | — | — |
| restack ms (outside frames) | — | — | — | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — | — | — | — |
| ws_park ms (outside frames) | — | — | — | — | — | — |
| ws_show ms (outside frames) | — | — | — | — | — | — |
| ws_start ms (outside frames) | — | — | — | — | — | — |
| input->anim start ms | 3.21 | 4.29 | 0.32 | — | — | 0.26 |
| input->first frame ms | 19.08 | 22.74 | 17.24 | — | — | 16.83 |

## resize

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 67.48 | 88.97 | 42.90 | — | — | 35.09 |
| cpu0 bd-config bo-config | frame p90 ms | 3.53 | 3.27 | 4.39 | 3.80 | 3.30 | 3.99 |
| cpu0 bd-config bo-config | interval p90 ms | 5.77 | 5.95 | 5.77 | 5.90 | 5.86 | 5.76 |
| cpu0 bd-config bo-config | interval p99 ms | 40.26 | 50.49 | 25.61 | 52.27 | 51.04 | 34.37 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 3.30 | 16.20 | — | — | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | — | 1.70 | — | — | 2.10 |
| cpu0 bd-config bo-config | restack ms (outside frames) | — | — | — | — | — | — |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 72.96 | 102.02 | 48.78 | — | — | 35.90 |
| cpu8 bd-config bo-config | frame p90 ms | 4.01 | 4.55 | 4.77 | 4.24 | 4.38 | 4.30 |
| cpu8 bd-config bo-config | interval p90 ms | 5.79 | 6.79 | 5.78 | 6.64 | 5.84 | 5.77 |
| cpu8 bd-config bo-config | interval p99 ms | 36.32 | 50.45 | 17.93 | 65.38 | 64.06 | 29.19 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 3.70 | 19.10 | — | — | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | — | 1.90 | — | — | 2.60 |
| cpu8 bd-config bo-config | restack ms (outside frames) | — | — | — | — | — | — |

### resize, cpu0 bd-config bo-config

| metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 46.64 | 64.61 | 39.15 | 64.77 | 62.68 | 30.47 |
| command p90 ms | 54.51 | 66.20 | 43.21 | 69.78 | 65.30 | 32.12 |
| frames | 61.00 | 61.00 | 62.00 | 60.00 | 60.00 | 63.00 |
| tick ms/frame | 2.42 | 3.33 | 1.86 | 3.53 | 3.52 | 2.28 |
| frame p90 ms | 3.53 | 3.27 | 4.39 | 3.80 | 3.30 | 3.99 |
| frames >2x budget | 3.00 | 4.00 | 2.00 | 4.00 | 4.00 | 2.00 |
| interval p50 ms | 5.72 | 5.72 | 5.71 | 5.72 | 5.72 | 5.71 |
| interval p90 ms | 5.77 | 5.95 | 5.77 | 5.90 | 5.86 | 5.76 |
| interval p99 ms | 40.26 | 50.49 | 25.61 | 52.27 | 51.04 | 34.37 |
| interval max ms (median burst) | 40.26 | 50.49 | 25.61 | 52.27 | 51.04 | 34.37 |
| peek_frame ms/frame | 2.01 | 2.25 | — | 2.36 | 2.24 | — |
| peek_draw ms/frame | 1.23 | 1.27 | — | — | — | — |
| peek_track ms/frame | 0.30 | 0.28 | — | — | — | — |
| peek_ring ms/frame | 0.17 | 0.13 | — | — | — | — |
| peek_backdrop ms/frame | 0.13 | 0.13 | — | — | — | — |
| peek_draw ms (worst frame) | 15.81 | 23.73 | — | — | — | — |
| pd_alpha ms (worst frame) | 1.04 | 0.94 | — | — | — | — |
| pd_draw ms (worst frame) | 14.78 | 22.91 | — | — | — | — |
| pd_fill ms (worst frame) | 11.21 | 19.74 | — | — | — | — |
| fill_source ms (worst frame) | 3.63 | 5.75 | — | — | — | — |
| ring_raise ms (worst frame) | 5.22 | 8.59 | — | — | — | — |
| restack ms (worst frame) | 2.88 | — | — | — | — | — |
| batch_commit ms/frame | — | — | 0.98 | — | — | 0.96 |
| ovl_region ms/frame | 0.05 | 0.04 | 0.04 | 0.04 | 0.04 | 0.05 |
| dwm_flush calls (in frames) | 2.00 | 1.00 | — | 2.00 | 2.00 | 2.00 |
| dwm_flush ms (outside frames) | 7.30 | 8.70 | 4.80 | 7.80 | 4.70 | 5.10 |
| cloak ms (outside frames) | 16.20 | 18.70 | 20.50 | 18.70 | 17.70 | 18.10 |
| session_begin ms (outside frames) | — | — | 1.70 | — | — | 2.10 |
| session_overlays ms (outside frames) | — | 0.00 | 15.00 | 0.00 | 0.00 | 6.90 |
| session_overlays ms/frame | — | 0.00 | 0.06 | 0.00 | 0.00 | 0.04 |
| command ms (WM side) | 46.30 | 64.20 | — | — | — | — |
| peek_new ms (outside frames) | 3.30 | 16.20 | — | — | — | — |
| peek_place ms (outside frames) | 1.90 | 4.10 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — | — |
| peek_cover ms (outside frames) | 11.20 | 9.90 | — | — | — | — |
| peek_handover ms (outside frames) | 11.60 | 12.10 | — | — | — | — |
| peek_raise ms (outside frames) | 0.10 | 2.20 | — | — | — | — |
| surface ms (outside frames) | 0.70 | — | — | — | — | — |
| restack ms (outside frames) | — | — | — | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — | — | — | — |
| ws_park ms (outside frames) | — | — | — | — | — | — |
| ws_show ms (outside frames) | — | — | — | — | — | — |
| ws_start ms (outside frames) | — | — | — | — | — | — |
| input->anim start ms | 3.27 | 4.39 | 0.28 | — | — | 0.22 |
| input->first frame ms | 67.48 | 88.97 | 42.90 | — | — | 35.09 |

### resize, cpu8 bd-config bo-config

| metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 50.07 | 71.87 | 44.48 | 71.05 | 71.02 | 31.75 |
| command p90 ms | 57.42 | 78.56 | 46.35 | 76.43 | 78.60 | 35.37 |
| frames | 61.00 | 59.00 | 62.00 | 59.00 | 59.00 | 63.00 |
| tick ms/frame | 2.63 | 3.68 | 1.89 | 3.78 | 3.87 | 2.45 |
| frame p90 ms | 4.01 | 4.55 | 4.77 | 4.24 | 4.38 | 4.30 |
| frames >2x budget | 3.00 | 4.00 | 2.00 | 4.00 | 4.00 | 2.00 |
| interval p50 ms | 5.72 | 5.71 | 5.71 | 5.72 | 5.72 | 5.71 |
| interval p90 ms | 5.79 | 6.79 | 5.78 | 6.64 | 5.84 | 5.77 |
| interval p99 ms | 36.32 | 50.45 | 17.93 | 65.38 | 64.06 | 29.19 |
| interval max ms (median burst) | 36.32 | 50.45 | 17.93 | 65.38 | 64.06 | 29.19 |
| peek_frame ms/frame | 2.15 | 2.54 | — | 2.56 | 2.57 | — |
| peek_draw ms/frame | 1.46 | 1.54 | — | — | — | — |
| peek_track ms/frame | 0.30 | 0.31 | — | — | — | — |
| peek_ring ms/frame | 0.14 | 0.17 | — | — | — | — |
| peek_backdrop ms/frame | 0.14 | 0.16 | — | — | — | — |
| peek_draw ms (worst frame) | 21.25 | 30.95 | — | — | — | — |
| pd_alpha ms (worst frame) | 1.26 | 1.12 | — | — | — | — |
| pd_draw ms (worst frame) | 19.76 | 29.60 | — | — | — | — |
| pd_fill ms (worst frame) | 15.33 | 25.56 | — | — | — | — |
| fill_source ms (worst frame) | 4.59 | 8.46 | — | — | — | — |
| ring_raise ms (worst frame) | 6.01 | 10.11 | — | — | — | — |
| restack ms (worst frame) | 2.70 | — | — | — | — | — |
| batch_commit ms/frame | — | — | 1.10 | — | — | 1.07 |
| ovl_region ms/frame | 0.04 | 0.05 | 0.03 | 0.04 | 0.04 | 0.04 |
| dwm_flush calls (in frames) | 2.00 | 2.00 | — | 2.00 | 2.00 | 2.00 |
| dwm_flush ms (outside frames) | 6.00 | 4.90 | 3.60 | 5.70 | 10.30 | 4.30 |
| cloak ms (outside frames) | 17.90 | 18.20 | 22.00 | 19.20 | 18.40 | 18.50 |
| session_begin ms (outside frames) | — | — | 1.90 | — | — | 2.60 |
| session_overlays ms (outside frames) | — | 0.00 | 17.50 | 0.00 | 0.00 | 7.90 |
| session_overlays ms/frame | — | 0.00 | 0.06 | 0.00 | 0.00 | 0.03 |
| command ms (WM side) | 49.70 | 71.50 | — | — | — | — |
| peek_new ms (outside frames) | 3.70 | 19.10 | — | — | — | — |
| peek_place ms (outside frames) | 2.10 | 5.10 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — | — |
| peek_cover ms (outside frames) | 12.00 | 11.10 | — | — | — | — |
| peek_handover ms (outside frames) | 13.10 | 14.00 | — | — | — | — |
| peek_raise ms (outside frames) | 0.10 | 2.70 | — | — | — | — |
| surface ms (outside frames) | 0.90 | — | — | — | — | — |
| restack ms (outside frames) | — | — | — | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — | — | — | — |
| ws_park ms (outside frames) | — | — | — | — | — | — |
| ws_show ms (outside frames) | — | — | — | — | — | — |
| ws_start ms (outside frames) | — | — | — | — | — | — |
| input->anim start ms | 3.51 | 5.01 | 0.32 | — | — | 0.27 |
| input->first frame ms | 72.96 | 102.02 | 48.78 | — | — | 35.90 |

## switch

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 0.00 | 0.00 | 0.00 | — | — | 0.00 |
| cpu0 bd-config bo-config | frame p90 ms | 1.08 | 1.21 | 1.19 | 1.17 | 1.20 | 2.44 |
| cpu0 bd-config bo-config | interval p90 ms | 5.76 | 5.76 | 5.76 | 5.75 | 5.75 | 5.75 |
| cpu0 bd-config bo-config | interval p99 ms | 18.95 | 24.26 | 22.92 | 24.34 | 25.11 | 53.11 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 0.00 | 0.00 | — | — | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | — | 0.00 | — | — | 0.00 |
| cpu0 bd-config bo-config | restack ms (outside frames) | 0.00 | — | — | — | — | — |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | — | — | 0.00 | — | — | 0.00 |
| cpu8 bd-config bo-config | frame p90 ms | 1.21 | 1.45 | 1.51 | 1.34 | 1.37 | 2.76 |
| cpu8 bd-config bo-config | interval p90 ms | 5.76 | 5.76 | 5.76 | 5.75 | 5.76 | 5.75 |
| cpu8 bd-config bo-config | interval p99 ms | 23.29 | 27.85 | 27.00 | 28.63 | 28.23 | 59.29 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | — | — | — | — | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | — | — | — | — | 0.00 |
| cpu8 bd-config bo-config | restack ms (outside frames) | 0.00 | — | — | — | — | — |

### switch, cpu0 bd-config bo-config

| metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 26.40 | 30.23 | 30.21 | 30.22 | 30.46 | 20.21 |
| command p90 ms | 28.41 | 31.46 | 31.55 | 31.40 | 31.74 | 20.66 |
| frames | 99.00 | 99.00 | 99.00 | 99.00 | 99.00 | 97.00 |
| tick ms/frame | 1.11 | 1.42 | 1.23 | 1.41 | 1.42 | 1.62 |
| frame p90 ms | 1.08 | 1.21 | 1.19 | 1.17 | 1.20 | 2.44 |
| frames >2x budget | 3.00 | 3.00 | 3.00 | 3.00 | 3.00 | 1.00 |
| interval p50 ms | 5.71 | 5.71 | 5.72 | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.76 | 5.76 | 5.76 | 5.75 | 5.75 | 5.75 |
| interval p99 ms | 18.95 | 24.26 | 22.92 | 24.34 | 25.11 | 53.11 |
| interval max ms (median burst) | 21.74 | 25.04 | 22.92 | 25.31 | 25.11 | 53.11 |
| peek_frame ms/frame | 0.00 | 0.00 | — | 0.00 | 0.00 | — |
| peek_draw ms/frame | 0.00 | 0.00 | — | — | — | — |
| peek_track ms/frame | 0.00 | 0.00 | — | — | — | — |
| peek_ring ms/frame | 0.00 | 0.00 | — | — | — | — |
| peek_backdrop ms/frame | 0.00 | 0.00 | — | — | — | — |
| peek_draw ms (worst frame) | 0.00 | 0.00 | — | — | — | — |
| pd_alpha ms (worst frame) | 0.00 | 0.00 | — | — | — | — |
| pd_draw ms (worst frame) | 0.00 | 0.00 | — | — | — | — |
| pd_fill ms (worst frame) | 0.00 | 0.00 | — | — | — | — |
| fill_source ms (worst frame) | 0.00 | 0.00 | — | — | — | — |
| ring_raise ms (worst frame) | 0.00 | 0.00 | — | — | — | — |
| restack ms (worst frame) | — | — | — | — | — | — |
| batch_commit ms/frame | — | — | 0.00 | — | — | 0.62 |
| ovl_region ms/frame | 0.03 | 0.03 | 0.03 | 0.03 | 0.03 | 0.02 |
| dwm_flush calls (in frames) | — | — | — | — | — | 3.00 |
| dwm_flush ms (outside frames) | 5.40 | 5.30 | 5.90 | 6.20 | 5.80 | 5.80 |
| cloak ms (outside frames) | 0.00 | 0.00 | 0.00 | 0.00 | — | 0.00 |
| session_begin ms (outside frames) | — | — | 0.00 | — | — | 0.00 |
| session_overlays ms (outside frames) | — | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| session_overlays ms/frame | — | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| command ms (WM side) | 26.10 | 29.90 | — | — | — | — |
| peek_new ms (outside frames) | 0.00 | 0.00 | — | — | — | — |
| peek_place ms (outside frames) | 0.00 | 0.00 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — | — |
| peek_cover ms (outside frames) | 0.00 | 0.00 | — | — | — | — |
| peek_handover ms (outside frames) | 0.00 | 0.00 | — | — | — | — |
| peek_raise ms (outside frames) | 0.00 | 0.00 | — | — | — | — |
| surface ms (outside frames) | 0.50 | — | — | — | — | — |
| restack ms (outside frames) | 0.00 | — | — | — | — | — |
| ws_surrogates ms (outside frames) | 10.20 | 12.80 | — | — | — | — |
| ws_park ms (outside frames) | 2.00 | 1.90 | — | — | — | — |
| ws_show ms (outside frames) | 8.00 | 9.20 | — | — | — | — |
| ws_start ms (outside frames) | 0.40 | 0.40 | — | — | — | — |
| input->anim start ms | 0.00 | 0.00 | 0.00 | — | — | 0.00 |
| input->first frame ms | 0.00 | 0.00 | 0.00 | — | — | 0.00 |

### switch, cpu8 bd-config bo-config

| metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 30.06 | 33.13 | 34.14 | 33.36 | 34.33 | 20.48 |
| command p90 ms | 31.20 | 36.75 | 35.05 | 36.14 | 36.18 | 23.14 |
| frames | 99.00 | 99.00 | 99.00 | 99.00 | 100.00 | 97.00 |
| tick ms/frame | 1.23 | 1.59 | 1.36 | 1.60 | 1.57 | 1.85 |
| frame p90 ms | 1.21 | 1.45 | 1.51 | 1.34 | 1.37 | 2.76 |
| frames >2x budget | 3.00 | 3.00 | 3.00 | 3.00 | 3.00 | 1.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.76 | 5.76 | 5.76 | 5.75 | 5.76 | 5.75 |
| interval p99 ms | 23.29 | 27.85 | 27.00 | 28.63 | 28.23 | 59.29 |
| interval max ms (median burst) | 23.82 | 29.26 | 27.00 | 29.23 | 29.15 | 60.41 |
| peek_frame ms/frame | 0.00 | 0.00 | — | 0.00 | 0.00 | — |
| peek_draw ms/frame | 0.00 | 0.00 | — | — | — | — |
| peek_track ms/frame | 0.00 | 0.00 | — | — | — | — |
| peek_ring ms/frame | 0.00 | 0.00 | — | — | — | — |
| peek_backdrop ms/frame | 0.00 | 0.00 | — | — | — | — |
| peek_draw ms (worst frame) | 0.00 | 0.00 | — | — | — | — |
| pd_alpha ms (worst frame) | 0.00 | 0.00 | — | — | — | — |
| pd_draw ms (worst frame) | 0.00 | 0.00 | — | — | — | — |
| pd_fill ms (worst frame) | 0.00 | 0.00 | — | — | — | — |
| fill_source ms (worst frame) | 0.00 | 0.00 | — | — | — | — |
| ring_raise ms (worst frame) | 0.00 | 0.00 | — | — | — | — |
| restack ms (worst frame) | — | — | — | — | — | — |
| batch_commit ms/frame | — | — | 0.00 | — | — | 0.71 |
| ovl_region ms/frame | 0.04 | 0.04 | 0.04 | 0.04 | 0.04 | 0.03 |
| dwm_flush calls (in frames) | — | — | — | — | — | 3.00 |
| dwm_flush ms (outside frames) | 5.10 | 5.80 | 5.90 | 5.80 | 5.40 | 4.50 |
| cloak ms (outside frames) | — | — | — | — | — | 0.00 |
| session_begin ms (outside frames) | — | — | — | — | — | 0.00 |
| session_overlays ms (outside frames) | — | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| session_overlays ms/frame | — | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| command ms (WM side) | 29.70 | 32.80 | — | — | — | — |
| peek_new ms (outside frames) | — | — | — | — | — | — |
| peek_place ms (outside frames) | — | — | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — | — |
| peek_cover ms (outside frames) | — | — | — | — | — | — |
| peek_handover ms (outside frames) | — | — | — | — | — | — |
| peek_raise ms (outside frames) | 0.00 | 0.00 | — | — | — | — |
| surface ms (outside frames) | 0.70 | — | — | — | — | — |
| restack ms (outside frames) | 0.00 | — | — | — | — | — |
| ws_surrogates ms (outside frames) | 12.60 | 14.80 | — | — | — | — |
| ws_park ms (outside frames) | 2.20 | 2.30 | — | — | — | — |
| ws_show ms (outside frames) | 8.00 | 9.40 | — | — | — | — |
| ws_start ms (outside frames) | 0.40 | 0.40 | — | — | — | — |
| input->anim start ms | — | — | 0.00 | — | — | 0.00 |
| input->first frame ms | — | — | 0.00 | — | — | 0.00 |

## minimize

Medians over rounds; —: in no report of that build (no such stage, or it never ran).

| config | metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- | --- |
| cpu0 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 | 3 |
| cpu0 bd-config bo-config | input->first frame ms | 70.99 | 90.10 | 55.39 | — | — | 49.42 |
| cpu0 bd-config bo-config | frame p90 ms | 3.59 | 3.76 | 3.16 | 3.82 | 3.90 | 3.89 |
| cpu0 bd-config bo-config | interval p90 ms | 5.78 | 7.68 | 5.76 | 7.00 | 7.04 | 5.76 |
| cpu0 bd-config bo-config | interval p99 ms | 56.96 | 74.22 | 48.04 | 77.51 | 72.42 | 37.66 |
| cpu0 bd-config bo-config | peek_new ms (outside frames) | 8.50 | 25.50 | — | — | — | — |
| cpu0 bd-config bo-config | session_begin ms (outside frames) | — | — | 2.20 | — | — | 2.10 |
| cpu0 bd-config bo-config | restack ms (outside frames) | 8.60 | — | — | — | — | — |
| cpu8 bd-config bo-config | runs | 3 | 3 | 3 | 3 | 3 | 3 |
| cpu8 bd-config bo-config | input->first frame ms | 76.44 | 97.74 | 64.12 | — | — | 45.88 |
| cpu8 bd-config bo-config | frame p90 ms | 4.15 | 5.09 | 3.17 | 4.56 | 4.75 | 4.35 |
| cpu8 bd-config bo-config | interval p90 ms | 5.78 | 8.23 | 5.77 | 8.91 | 7.72 | 5.77 |
| cpu8 bd-config bo-config | interval p99 ms | 68.50 | 83.20 | 52.81 | 84.90 | 83.06 | 35.75 |
| cpu8 bd-config bo-config | peek_new ms (outside frames) | 9.00 | 29.50 | — | — | — | — |
| cpu8 bd-config bo-config | session_begin ms (outside frames) | — | — | 3.20 | — | — | 2.60 |
| cpu8 bd-config bo-config | restack ms (outside frames) | 9.70 | — | — | — | — | — |

### minimize, cpu0 bd-config bo-config

| metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 51.19 | 60.89 | 23.18 | 61.68 | 57.43 | 16.96 |
| command p90 ms | 58.00 | 64.18 | 25.21 | 65.65 | 64.86 | 19.00 |
| frames | 61.00 | 60.00 | 144.00 | 60.00 | 60.00 | 150.00 |
| tick ms/frame | 2.33 | 3.02 | 1.16 | 3.20 | 3.25 | 1.43 |
| frame p90 ms | 3.59 | 3.76 | 3.16 | 3.82 | 3.90 | 3.89 |
| frames >2x budget | 3.00 | 4.00 | 0.00 | 4.00 | 4.00 | 3.00 |
| interval p50 ms | 5.72 | 5.72 | 5.71 | 5.72 | 5.72 | 5.71 |
| interval p90 ms | 5.78 | 7.68 | 5.76 | 7.00 | 7.04 | 5.76 |
| interval p99 ms | 56.96 | 74.22 | 48.04 | 77.51 | 72.42 | 37.66 |
| interval max ms (median burst) | 56.96 | 74.22 | 48.04 | 77.51 | 72.42 | 37.66 |
| peek_frame ms/frame | 1.88 | 2.19 | — | 2.24 | 2.14 | — |
| peek_draw ms/frame | 1.20 | 1.26 | — | — | — | — |
| peek_track ms/frame | 0.35 | 0.34 | — | — | — | — |
| peek_ring ms/frame | 0.12 | 0.13 | — | — | — | — |
| peek_backdrop ms/frame | 0.21 | 0.20 | — | — | — | — |
| peek_draw ms (worst frame) | 17.31 | 26.52 | — | — | — | — |
| pd_alpha ms (worst frame) | 0.81 | 0.73 | — | — | — | — |
| pd_draw ms (worst frame) | 16.50 | 25.91 | — | — | — | — |
| pd_fill ms (worst frame) | 13.15 | 20.97 | — | — | — | — |
| fill_source ms (worst frame) | 3.60 | 6.80 | — | — | — | — |
| ring_raise ms (worst frame) | 4.47 | 7.30 | — | — | — | — |
| restack ms (worst frame) | 2.63 | — | — | — | — | — |
| batch_commit ms/frame | — | — | 0.63 | — | — | 0.69 |
| ovl_region ms/frame | 0.02 | 0.03 | 0.02 | 0.02 | 0.03 | 0.03 |
| dwm_flush calls (in frames) | 2.00 | 2.00 | 1.00 | 2.00 | 2.00 | 4.00 |
| dwm_flush ms (outside frames) | 11.10 | 9.40 | 3.90 | 8.50 | 11.70 | 4.90 |
| cloak ms (outside frames) | 19.00 | 16.40 | 22.00 | 14.70 | 16.20 | 17.30 |
| session_begin ms (outside frames) | — | — | 2.20 | — | — | 2.10 |
| session_overlays ms (outside frames) | — | 0.00 | 17.20 | 0.00 | 0.00 | 7.30 |
| session_overlays ms/frame | — | 0.00 | 0.04 | 0.00 | 0.00 | 0.02 |
| command ms (WM side) | 50.90 | 60.60 | — | — | — | — |
| peek_new ms (outside frames) | 8.50 | 25.50 | — | — | — | — |
| peek_place ms (outside frames) | 1.70 | 5.90 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — | — |
| peek_cover ms (outside frames) | 23.00 | 18.40 | — | — | — | — |
| peek_handover ms (outside frames) | 15.40 | 14.70 | — | — | — | — |
| peek_raise ms (outside frames) | 0.20 | 1.50 | — | — | — | — |
| surface ms (outside frames) | 2.60 | — | — | — | — | — |
| restack ms (outside frames) | 8.60 | — | — | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — | — | — | — |
| ws_park ms (outside frames) | — | — | — | — | — | — |
| ws_show ms (outside frames) | — | — | — | — | — | — |
| ws_start ms (outside frames) | — | — | — | — | — | — |
| input->anim start ms | 10.69 | 8.64 | 5.57 | — | — | 6.15 |
| input->first frame ms | 70.99 | 90.10 | 55.39 | — | — | 49.42 |

### minimize, cpu8 bd-config bo-config

| metric | this | start-latency | stand-ins:nopeeks | stand-ins | stand-ins:morepeeks | main |
| --- | --- | --- | --- | --- | --- | --- |
| runs | 3 | 3 | 3 | 3 | 3 | 3 |
| command p50 ms | 56.75 | 66.74 | 26.76 | 63.98 | 64.68 | 19.03 |
| command p90 ms | 64.95 | 72.63 | 30.77 | 74.41 | 73.51 | 21.44 |
| frames | 61.00 | 59.00 | 143.00 | 60.00 | 60.00 | 149.00 |
| tick ms/frame | 2.54 | 3.40 | 1.34 | 3.61 | 3.53 | 1.67 |
| frame p90 ms | 4.15 | 5.09 | 3.17 | 4.56 | 4.75 | 4.35 |
| frames >2x budget | 3.00 | 5.00 | 1.00 | 5.00 | 4.00 | 2.00 |
| interval p50 ms | 5.71 | 5.71 | 5.71 | 5.71 | 5.71 | 5.71 |
| interval p90 ms | 5.78 | 8.23 | 5.77 | 8.91 | 7.72 | 5.77 |
| interval p99 ms | 68.50 | 83.20 | 52.81 | 84.90 | 83.06 | 35.75 |
| interval max ms (median burst) | 68.50 | 83.20 | 52.81 | 84.90 | 83.06 | 35.75 |
| peek_frame ms/frame | 2.10 | 2.49 | — | 2.42 | 2.43 | — |
| peek_draw ms/frame | 1.35 | 1.44 | — | — | — | — |
| peek_track ms/frame | 0.41 | 0.39 | — | — | — | — |
| peek_ring ms/frame | 0.14 | 0.16 | — | — | — | — |
| peek_backdrop ms/frame | 0.23 | 0.23 | — | — | — | — |
| peek_draw ms (worst frame) | 17.83 | 28.35 | — | — | — | — |
| pd_alpha ms (worst frame) | 0.95 | 0.94 | — | — | — | — |
| pd_draw ms (worst frame) | 16.93 | 27.63 | — | — | — | — |
| pd_fill ms (worst frame) | 12.75 | 22.79 | — | — | — | — |
| fill_source ms (worst frame) | 2.88 | 6.60 | — | — | — | — |
| ring_raise ms (worst frame) | 5.45 | 8.97 | — | — | — | — |
| restack ms (worst frame) | 2.70 | — | — | — | — | — |
| batch_commit ms/frame | — | — | 0.73 | — | — | 0.82 |
| ovl_region ms/frame | 0.03 | 0.03 | 0.02 | 0.03 | 0.03 | 0.03 |
| dwm_flush calls (in frames) | 2.00 | 2.00 | 1.00 | 2.00 | 2.00 | 4.00 |
| dwm_flush ms (outside frames) | 11.50 | 8.80 | 4.30 | 11.30 | 8.90 | 4.60 |
| cloak ms (outside frames) | 20.50 | 16.10 | 25.90 | 17.70 | 16.70 | 16.30 |
| session_begin ms (outside frames) | — | — | 3.20 | — | — | 2.60 |
| session_overlays ms (outside frames) | — | 0.00 | 19.60 | 0.00 | 0.00 | 7.90 |
| session_overlays ms/frame | — | 0.00 | 0.04 | 0.00 | 0.00 | 0.02 |
| command ms (WM side) | 56.40 | 66.30 | — | — | — | — |
| peek_new ms (outside frames) | 9.00 | 29.50 | — | — | — | — |
| peek_place ms (outside frames) | 1.80 | 6.30 | — | — | — | — |
| pd_fill ms (outside frames) | — | — | — | — | — | — |
| fill_source ms (outside frames) | — | — | — | — | — | — |
| ring_raise ms (outside frames) | — | — | — | — | — | — |
| peek_cover ms (outside frames) | 23.70 | 21.30 | — | — | — | — |
| peek_handover ms (outside frames) | 17.40 | 15.40 | — | — | — | — |
| peek_raise ms (outside frames) | 0.40 | 1.80 | — | — | — | — |
| surface ms (outside frames) | 2.40 | — | — | — | — | — |
| restack ms (outside frames) | 9.70 | — | — | — | — | — |
| ws_surrogates ms (outside frames) | — | — | — | — | — | — |
| ws_park ms (outside frames) | — | — | — | — | — | — |
| ws_show ms (outside frames) | — | — | — | — | — | — |
| ws_start ms (outside frames) | — | — | — | — | — | — |
| input->anim start ms | 10.22 | 10.70 | 6.57 | — | — | 7.40 |
| input->first frame ms | 76.44 | 97.74 | 64.12 | — | — | 45.88 |
