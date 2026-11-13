
# peek-load0

## move: medians over reports (outside-frames ms unless noted)

| mode | n | command | platform_sync | redraw | redraw_loop | anim_step | peek_new | rd_frozen | take_over | preposition | cloak | dwm_flush | session_begin | session_overlays | peek_cover | peek_handover | peek_raise | ovl_place | ovl_blend | sync_tail | batch_commit | first frame | int p99 | int max | worst sync row |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks | 41 | 14.1 | 14.7 | 13.7 | 4.3 | 1.1 | 0.0 | 2.3 | 1.6 | 0.0 | 8.5 | 4.4 | 0.7 | 3.3 | 0.0 | 0.0 | 0.0 | 2.2 | 1.1 | 1.2 | 0.0 | 15.2 | 7.7 | 7.7 | none |
| nopeeks+keepoverlays | 43 | 11.8 | 12.3 | 10.9 | 3.0 | 1.1 | 0.0 | 1.3 | 1.6 | 0.0 | 7.7 | 4.7 | 0.7 | 1.9 | 0.0 | 0.0 | 0.0 | 0.7 | 1.1 | 1.2 | 0.0 | 12.7 | 7.5 | 7.5 | none |
| default | 41 | 18.7 | 19.4 | 18.9 | 7.8 | 7.0 | 5.2 | 0.0 | 2.8 | 0.1 | 7.5 | 4.5 | 0.0 | 0.0 | 1.0 | 2.7 | 0.4 | 0.8 | 0.0 | 0.4 | 0.0 | 19.9 | 11.2 | 11.2 | Notepad 110 (84 rows) |
| keeppeeks | 42 | 18.8 | 19.6 | 19.0 | 7.5 | 6.5 | 4.9 | 0.0 | 2.9 | 0.1 | 7.8 | 4.8 | 0.0 | 0.0 | 1.0 | 2.7 | 0.4 | 0.8 | 0.0 | 0.4 | 0.0 | 19.6 | 10.1 | 10.1 | Notepad 140 (89 rows) |
| asynchandback | 43 | 19.3 | 20.0 | 19.5 | 8.0 | 7.1 | 5.5 | 0.0 | 2.8 | 0.1 | 7.8 | 4.8 | 0.0 | 0.0 | 1.1 | 2.6 | 0.4 | 0.8 | 0.0 | 0.4 | 0.0 | 20.5 | 10.8 | 10.8 | none |
| asynclanding | 42 | 18.4 | 19.1 | 18.6 | 7.7 | 6.8 | 5.3 | 0.0 | 2.8 | 0.1 | 7.0 | 4.0 | 0.0 | 0.0 | 1.1 | 2.6 | 0.4 | 0.8 | 0.0 | 0.4 | 0.0 | 19.5 | 11.2 | 11.2 | WindowsTerminal 5 (88 rows) |
| earlylanding | 42 | 19.2 | 20.0 | 19.5 | 8.0 | 7.1 | 5.3 | 0.0 | 2.8 | 0.1 | 7.6 | 4.5 | 0.0 | 0.0 | 1.0 | 2.5 | 0.4 | 0.7 | 0.0 | 0.4 | 0.0 | 20.3 | 10.4 | 10.4 | chrome 6 (88 rows) |
| earlylanding+asynchandback+keeppeeks | 42 | 18.4 | 19.1 | 18.6 | 7.3 | 6.5 | 5.0 | 0.0 | 2.8 | 0.1 | 7.5 | 4.5 | 0.0 | 0.0 | 1.0 | 2.6 | 0.3 | 0.7 | 0.0 | 0.4 | 0.0 | 19.3 | 10.5 | 10.5 | none |

# peek-load0

## resize: medians over reports (outside-frames ms unless noted)

| mode | n | command | platform_sync | redraw | redraw_loop | anim_step | peek_new | rd_frozen | take_over | preposition | cloak | dwm_flush | session_begin | session_overlays | peek_cover | peek_handover | peek_raise | ovl_place | ovl_blend | sync_tail | batch_commit | first frame | int p99 | int max | worst sync row |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks | 39 | 39.2 | 42.7 | 42.3 | 13.2 | 2.1 | 0.0 | 7.1 | 8.2 | 0.3 | 20.7 | 4.7 | 1.8 | 14.8 | 0.0 | 0.0 | 0.0 | 12.1 | 0.1 | 0.4 | 0.1 | 44.2 | 17.3 | 17.3 | none |
| nopeeks+keepoverlays | 37 | 28.0 | 31.2 | 30.4 | 9.4 | 2.1 | 0.0 | 3.6 | 8.5 | 0.3 | 16.5 | 4.7 | 1.7 | 6.1 | 0.0 | 0.0 | 0.0 | 3.8 | 0.0 | 0.7 | 0.2 | 36.2 | 18.2 | 18.2 | none |
| default | 39 | 63.9 | 70.2 | 68.0 | 27.3 | 22.9 | 17.7 | 0.0 | 13.5 | 0.2 | 19.1 | 10.4 | 0.0 | 0.0 | 10.3 | 10.7 | 1.6 | 0.0 | 0.1 | 1.2 | 0.0 | 89.9 | 43.3 | 43.3 | Notepad 122 (120 rows) |
| keeppeeks | 38 | 60.1 | 66.5 | 64.0 | 24.6 | 20.0 | 15.3 | 0.0 | 10.4 | 0.2 | 14.9 | 4.0 | 0.0 | 0.0 | 10.6 | 11.8 | 1.6 | 0.0 | 0.0 | 0.7 | 0.0 | 86.0 | 43.9 | 43.9 | Notepad 122 (116 rows) |
| asynchandback | 37 | 64.0 | 69.5 | 66.4 | 27.6 | 22.6 | 17.7 | 0.0 | 10.9 | 0.2 | 18.6 | 5.5 | 0.0 | 0.0 | 10.6 | 13.3 | 1.9 | 0.0 | 0.1 | 1.0 | 0.0 | 90.6 | 36.4 | 36.4 | none |
| asynclanding | 38 | 62.8 | 70.4 | 67.0 | 27.9 | 22.5 | 17.6 | 0.0 | 12.1 | 0.2 | 18.0 | 10.6 | 0.0 | 0.0 | 10.8 | 11.3 | 1.4 | 0.0 | 0.1 | 0.9 | 0.0 | 91.9 | 35.3 | 35.3 | FPilot 12 (152 rows) |
| earlylanding | 38 | 61.5 | 66.8 | 64.4 | 26.0 | 21.6 | 17.1 | 0.0 | 10.6 | 0.3 | 14.9 | 4.1 | 0.0 | 0.0 | 10.2 | 11.4 | 1.9 | 0.0 | 0.0 | 0.6 | 0.0 | 86.9 | 30.4 | 30.4 | WindowsTerminal 5 (152 rows) |
| earlylanding+asynchandback+keeppeeks | 38 | 59.6 | 64.2 | 62.6 | 23.6 | 19.6 | 14.9 | 0.0 | 10.4 | 0.2 | 16.4 | 4.8 | 0.0 | 0.0 | 10.1 | 10.9 | 1.9 | 0.0 | 0.0 | 0.6 | 0.0 | 87.4 | 29.6 | 29.6 | none |

# peek-cpu8

## move: medians over reports (outside-frames ms unless noted)

| mode | n | command | platform_sync | redraw | redraw_loop | anim_step | peek_new | rd_frozen | take_over | preposition | cloak | dwm_flush | session_begin | session_overlays | peek_cover | peek_handover | peek_raise | ovl_place | ovl_blend | sync_tail | batch_commit | first frame | int p99 | int max | worst sync row |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks | 43 | 15.0 | 15.8 | 14.5 | 5.0 | 1.1 | 0.0 | 2.9 | 1.7 | 0.0 | 8.8 | 4.0 | 0.7 | 3.6 | 0.0 | 0.0 | 0.0 | 2.6 | 1.1 | 1.3 | 0.0 | 16.2 | 8.6 | 8.6 | none |
| nopeeks+keepoverlays | 42 | 12.6 | 13.1 | 12.1 | 3.4 | 1.1 | 0.0 | 1.5 | 1.8 | 0.0 | 8.6 | 5.2 | 0.7 | 1.9 | 0.0 | 0.0 | 0.0 | 0.8 | 1.2 | 1.3 | 0.0 | 13.6 | 8.2 | 8.2 | none |
| default | 41 | 20.1 | 20.9 | 20.3 | 8.7 | 7.8 | 5.8 | 0.0 | 3.0 | 0.1 | 7.1 | 4.0 | 0.0 | 0.0 | 1.2 | 3.1 | 0.4 | 1.0 | 0.0 | 0.4 | 0.0 | 21.4 | 12.2 | 12.2 | Notepad 122 (84 rows) |
| keeppeeks | 41 | 19.5 | 20.3 | 19.6 | 7.5 | 6.6 | 4.8 | 0.0 | 3.0 | 0.1 | 7.3 | 4.2 | 0.0 | 0.0 | 1.1 | 3.2 | 0.4 | 0.9 | 0.0 | 0.4 | 0.0 | 20.4 | 11.3 | 11.3 | Notepad 123 (84 rows) |
| asynchandback | 42 | 19.6 | 20.5 | 20.0 | 8.4 | 7.4 | 5.7 | 0.0 | 3.1 | 0.1 | 7.2 | 4.1 | 0.0 | 0.0 | 1.1 | 3.0 | 0.4 | 0.9 | 0.0 | 0.4 | 0.0 | 21.2 | 12.2 | 12.2 | none |
| asynclanding | 42 | 20.2 | 21.1 | 20.5 | 8.5 | 7.5 | 5.8 | 0.0 | 3.0 | 0.1 | 7.3 | 4.0 | 0.0 | 0.0 | 1.1 | 3.1 | 0.4 | 0.9 | 0.0 | 0.4 | 0.0 | 21.1 | 12.7 | 12.7 | WindowsTerminal 11 (88 rows) |
| earlylanding | 43 | 19.9 | 20.6 | 20.1 | 8.6 | 7.6 | 5.8 | 0.0 | 3.1 | 0.1 | 7.2 | 4.0 | 0.0 | 0.0 | 1.1 | 3.1 | 0.4 | 0.9 | 0.0 | 0.4 | 0.0 | 21.2 | 11.8 | 11.8 | WindowsTerminal 4 (92 rows) |
| earlylanding+asynchandback+keeppeeks | 41 | 19.4 | 20.2 | 19.6 | 8.0 | 7.1 | 5.2 | 0.0 | 3.0 | 0.1 | 7.3 | 4.2 | 0.0 | 0.0 | 1.1 | 3.1 | 0.4 | 0.9 | 0.0 | 0.4 | 0.0 | 20.2 | 12.6 | 12.6 | none |

# peek-cpu8

## resize: medians over reports (outside-frames ms unless noted)

| mode | n | command | platform_sync | redraw | redraw_loop | anim_step | peek_new | rd_frozen | take_over | preposition | cloak | dwm_flush | session_begin | session_overlays | peek_cover | peek_handover | peek_raise | ovl_place | ovl_blend | sync_tail | batch_commit | first frame | int p99 | int max | worst sync row |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nopeeks | 37 | 40.6 | 44.2 | 43.7 | 14.4 | 2.2 | 0.0 | 8.4 | 8.0 | 0.3 | 19.6 | 2.7 | 1.7 | 17.1 | 0.0 | 0.0 | 0.0 | 14.1 | 0.0 | 0.6 | 0.1 | 47.2 | 17.8 | 17.8 | none |
| nopeeks+keepoverlays | 38 | 30.0 | 34.0 | 33.3 | 10.9 | 2.4 | 0.0 | 4.5 | 10.3 | 0.2 | 18.5 | 4.4 | 2.0 | 6.6 | 0.0 | 0.0 | 0.0 | 3.6 | 0.0 | 0.5 | 0.1 | 37.2 | 19.7 | 19.7 | none |
| default | 39 | 67.9 | 76.8 | 71.9 | 28.8 | 24.5 | 18.7 | 0.0 | 11.5 | 0.2 | 18.4 | 5.2 | 0.0 | 0.0 | 11.2 | 12.3 | 1.8 | 0.0 | 0.0 | 0.9 | 0.0 | 97.5 | 39.9 | 39.9 | Notepad 124 (120 rows) |
| keeppeeks | 39 | 65.7 | 74.0 | 69.2 | 27.4 | 23.1 | 17.6 | 0.0 | 10.8 | 0.2 | 17.3 | 5.2 | 0.0 | 0.0 | 11.4 | 11.4 | 1.8 | 0.0 | 0.0 | 0.8 | 0.0 | 96.2 | 46.2 | 46.2 | Notepad 118 (120 rows) |
| asynchandback | 38 | 67.7 | 76.6 | 71.0 | 27.6 | 23.5 | 18.2 | 0.0 | 12.8 | 0.2 | 18.9 | 5.2 | 0.0 | 0.0 | 11.6 | 11.3 | 1.9 | 0.0 | 0.1 | 1.0 | 0.0 | 97.7 | 39.6 | 39.6 | none |
| asynclanding | 38 | 67.9 | 75.4 | 72.2 | 29.6 | 25.0 | 19.1 | 0.0 | 11.4 | 0.2 | 17.6 | 3.6 | 0.0 | 0.0 | 11.3 | 11.7 | 1.7 | 0.0 | 0.1 | 0.9 | 0.0 | 97.5 | 38.1 | 38.1 | Notepad 115 (143 rows) |
| earlylanding | 37 | 67.5 | 73.5 | 71.6 | 28.4 | 24.0 | 18.6 | 0.0 | 11.5 | 0.3 | 17.1 | 4.2 | 0.0 | 0.0 | 10.9 | 12.6 | 1.9 | 0.0 | 0.0 | 0.7 | 0.0 | 98.8 | 30.8 | 30.8 | WindowsTerminal 5 (148 rows) |
| earlylanding+asynchandback+keeppeeks | 39 | 65.5 | 70.9 | 68.6 | 26.8 | 22.2 | 16.4 | 0.0 | 11.9 | 0.3 | 17.8 | 5.6 | 0.0 | 0.0 | 11.0 | 12.9 | 1.8 | 0.0 | 0.0 | 0.7 | 0.0 | 97.2 | 32.2 | 32.2 | none |

# more-load0

## resize: medians over reports (outside-frames ms unless noted)

| mode | n | command | platform_sync | redraw | redraw_loop | anim_step | peek_new | rd_frozen | take_over | preposition | cloak | dwm_flush | session_begin | session_overlays | peek_cover | peek_handover | peek_raise | ovl_place | ovl_blend | sync_tail | batch_commit | first frame | int p99 | int max | worst sync row |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| morepeeks | 38 | 62.7 | 69.8 | 66.2 | 25.5 | 21.1 | 16.2 | 0.0 | 11.1 | 0.2 | 17.9 | 5.4 | 0.0 | 0.0 | 10.2 | 11.2 | 1.6 | 0.0 | 0.1 | 0.9 | 0.0 | 86.8 | 38.6 | 38.6 | FPilot 19 (114 rows) |
| morepeeks+keeppeeks | 39 | 59.9 | 65.8 | 62.6 | 24.2 | 19.6 | 14.8 | 0.0 | 10.9 | 0.1 | 16.8 | 4.1 | 0.0 | 0.0 | 10.4 | 9.8 | 1.3 | 0.0 | 0.0 | 0.6 | 0.0 | 84.6 | 40.0 | 40.0 | WindowsTerminal 19 (117 rows) |
| morepeeks+earlylanding+asynchandback+keeppeeks | 37 | 58.7 | 63.9 | 62.0 | 23.1 | 18.7 | 13.8 | 0.0 | 10.8 | 0.2 | 15.9 | 4.2 | 0.0 | 0.0 | 10.1 | 10.5 | 1.7 | 0.0 | 0.0 | 0.6 | 0.0 | 85.4 | 27.3 | 27.3 | none |

# more-load0

## float: medians over reports (outside-frames ms unless noted)

| mode | n | command | platform_sync | redraw | redraw_loop | anim_step | peek_new | rd_frozen | take_over | preposition | cloak | dwm_flush | session_begin | session_overlays | peek_cover | peek_handover | peek_raise | ovl_place | ovl_blend | sync_tail | batch_commit | first frame | int p99 | int max | worst sync row |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| morepeeks | 42 | 61.2 | 67.0 | 63.8 | 25.4 | 21.6 | 16.8 | 0.0 | 7.8 | 0.1 | 17.4 | 7.2 | 0.0 | 0.0 | 11.5 | 5.5 | 1.4 | 0.0 | 0.0 | 0.8 | 0.0 | 84.9 | 33.3 | 33.3 | Notepad 94 (134 rows) |
| morepeeks+keeppeeks | 41 | 59.9 | 66.0 | 62.6 | 23.7 | 19.5 | 14.9 | 0.0 | 8.2 | 0.0 | 18.1 | 9.5 | 0.0 | 0.0 | 11.5 | 5.8 | 1.6 | 0.0 | 0.0 | 0.7 | 0.0 | 86.6 | 33.3 | 33.3 | Notepad 98 (131 rows) |
| morepeeks+earlylanding+asynchandback+keeppeeks | 43 | 60.1 | 64.3 | 62.7 | 23.1 | 19.6 | 14.8 | 0.0 | 7.9 | 0.0 | 18.7 | 10.7 | 0.0 | 0.0 | 11.4 | 5.5 | 1.6 | 0.0 | 0.0 | 0.7 | 0.0 | 85.0 | 25.7 | 25.7 | none |

# more-cpu8

## resize: medians over reports (outside-frames ms unless noted)

| mode | n | command | platform_sync | redraw | redraw_loop | anim_step | peek_new | rd_frozen | take_over | preposition | cloak | dwm_flush | session_begin | session_overlays | peek_cover | peek_handover | peek_raise | ovl_place | ovl_blend | sync_tail | batch_commit | first frame | int p99 | int max | worst sync row |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| morepeeks | 38 | 66.6 | 73.6 | 70.5 | 29.9 | 25.8 | 19.8 | 0.0 | 11.1 | 0.2 | 16.6 | 5.1 | 0.0 | 0.0 | 11.1 | 11.4 | 1.7 | 0.0 | 0.1 | 0.9 | 0.0 | 97.7 | 40.3 | 40.3 | FPilot 22 (114 rows) |
| morepeeks+keeppeeks | 37 | 64.9 | 72.6 | 68.5 | 27.7 | 22.9 | 16.8 | 0.0 | 11.3 | 0.3 | 18.1 | 4.6 | 0.0 | 0.0 | 10.8 | 12.4 | 1.7 | 0.0 | 0.0 | 0.8 | 0.0 | 94.8 | 42.6 | 42.6 | FPilot 28 (111 rows) |
| morepeeks+earlylanding+asynchandback+keeppeeks | 40 | 62.7 | 68.8 | 66.2 | 27.3 | 21.9 | 16.4 | 0.0 | 10.2 | 0.1 | 17.1 | 5.3 | 0.0 | 0.0 | 10.6 | 9.4 | 1.8 | 0.0 | 0.0 | 0.7 | 0.0 | 94.6 | 32.3 | 32.3 | none |

# more-cpu8

## float: medians over reports (outside-frames ms unless noted)

| mode | n | command | platform_sync | redraw | redraw_loop | anim_step | peek_new | rd_frozen | take_over | preposition | cloak | dwm_flush | session_begin | session_overlays | peek_cover | peek_handover | peek_raise | ovl_place | ovl_blend | sync_tail | batch_commit | first frame | int p99 | int max | worst sync row |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| morepeeks | 42 | 67.3 | 74.8 | 70.1 | 28.4 | 24.9 | 19.3 | 0.0 | 8.1 | 0.0 | 17.6 | 7.1 | 0.0 | 0.0 | 12.4 | 6.3 | 1.8 | 0.0 | 0.0 | 1.0 | 0.0 | 97.9 | 37.2 | 37.2 | Notepad 100 (134 rows) |
| morepeeks+keeppeeks | 43 | 61.6 | 68.9 | 64.5 | 26.7 | 22.9 | 17.2 | 0.0 | 8.5 | 0.0 | 17.7 | 8.2 | 0.0 | 0.0 | 11.7 | 6.3 | 2.0 | 0.0 | 0.0 | 0.8 | 0.0 | 88.9 | 37.6 | 37.6 | Notepad 104 (137 rows) |
| morepeeks+earlylanding+asynchandback+keeppeeks | 40 | 65.0 | 70.0 | 67.8 | 25.9 | 22.2 | 16.7 | 0.0 | 9.6 | 0.1 | 17.4 | 8.0 | 0.0 | 0.0 | 11.8 | 8.8 | 1.9 | 0.0 | 0.0 | 0.7 | 0.0 | 94.2 | 29.0 | 29.0 | none |

# main-load0

## move: medians over reports (outside-frames ms unless noted)

| mode | n | command | platform_sync | redraw | redraw_loop | anim_step | peek_new | rd_frozen | take_over | preposition | cloak | dwm_flush | session_begin | session_overlays | peek_cover | peek_handover | peek_raise | ovl_place | ovl_blend | sync_tail | batch_commit | first frame | int p99 | int max | worst sync row |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main | 43 | 0.0 | 13.5 | 12.4 | 3.1 | 1.0 | 0.0 | 1.3 | 0.0 | 0.0 | 8.7 | 5.1 | 0.6 | 1.8 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 13.6 | 7.6 | 7.6 | none |
| nopeeks | 43 | 15.8 | 16.4 | 14.9 | 4.3 | 1.0 | 0.0 | 2.4 | 1.6 | 0.0 | 9.6 | 5.2 | 0.6 | 3.4 | 0.0 | 0.0 | 0.0 | 2.3 | 1.2 | 1.2 | 0.0 | 16.7 | 7.9 | 7.9 | none |
| default | 42 | 18.9 | 19.7 | 19.1 | 8.0 | 7.2 | 5.3 | 0.0 | 2.7 | 0.1 | 7.5 | 4.5 | 0.0 | 0.0 | 1.0 | 2.7 | 0.4 | 0.8 | 0.0 | 0.4 | 0.0 | 20.0 | 10.8 | 10.8 | Notepad 117 (88 rows) |

# main-load0

## resize: medians over reports (outside-frames ms unless noted)

| mode | n | command | platform_sync | redraw | redraw_loop | anim_step | peek_new | rd_frozen | take_over | preposition | cloak | dwm_flush | session_begin | session_overlays | peek_cover | peek_handover | peek_raise | ovl_place | ovl_blend | sync_tail | batch_commit | first frame | int p99 | int max | worst sync row |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main | 37 | 0.0 | 29.8 | 29.6 | 8.8 | 1.9 | 0.0 | 3.5 | 0.0 | 0.0 | 14.5 | 3.9 | 1.5 | 7.7 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.1 | 31.4 | 20.7 | 20.7 | none |
| nopeeks | 37 | 39.2 | 42.6 | 42.2 | 12.7 | 2.1 | 0.0 | 6.9 | 9.6 | 0.3 | 20.6 | 4.3 | 1.7 | 14.3 | 0.0 | 0.0 | 0.0 | 11.1 | 0.0 | 0.4 | 0.1 | 43.5 | 16.8 | 16.8 | none |
| default | 38 | 59.9 | 65.8 | 62.8 | 24.4 | 20.5 | 15.9 | 0.0 | 11.7 | 0.3 | 15.7 | 3.0 | 0.0 | 0.0 | 10.0 | 11.5 | 1.7 | 0.0 | 0.0 | 1.1 | 0.0 | 85.9 | 39.0 | 39.0 | Notepad 120 (116 rows) |

# main-cpu8

## move: medians over reports (outside-frames ms unless noted)

| mode | n | command | platform_sync | redraw | redraw_loop | anim_step | peek_new | rd_frozen | take_over | preposition | cloak | dwm_flush | session_begin | session_overlays | peek_cover | peek_handover | peek_raise | ovl_place | ovl_blend | sync_tail | batch_commit | first frame | int p99 | int max | worst sync row |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main | 40 | 0.0 | 13.7 | 12.2 | 3.5 | 1.1 | 0.0 | 1.5 | 0.0 | 0.0 | 8.4 | 5.2 | 0.7 | 1.9 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 14.3 | 8.6 | 8.6 | none |
| nopeeks | 44 | 15.4 | 16.1 | 14.6 | 4.9 | 1.1 | 0.0 | 2.9 | 1.7 | 0.0 | 9.4 | 4.3 | 0.7 | 3.3 | 0.0 | 0.0 | 0.0 | 2.6 | 1.2 | 1.2 | 0.0 | 16.6 | 9.0 | 9.0 | none |
| default | 40 | 20.4 | 21.1 | 20.6 | 8.8 | 7.8 | 5.8 | 0.0 | 3.0 | 0.1 | 7.3 | 4.1 | 0.0 | 0.0 | 1.1 | 3.0 | 0.4 | 0.9 | 0.0 | 0.4 | 0.0 | 21.6 | 12.2 | 12.2 | chrome 2 (80 rows) |

# main-cpu8

## resize: medians over reports (outside-frames ms unless noted)

| mode | n | command | platform_sync | redraw | redraw_loop | anim_step | peek_new | rd_frozen | take_over | preposition | cloak | dwm_flush | session_begin | session_overlays | peek_cover | peek_handover | peek_raise | ovl_place | ovl_blend | sync_tail | batch_commit | first frame | int p99 | int max | worst sync row |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| main | 40 | 0.0 | 32.8 | 32.4 | 10.9 | 2.5 | 0.0 | 4.8 | 0.0 | 0.0 | 18.2 | 4.5 | 2.1 | 6.8 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.0 | 0.1 | 35.1 | 26.2 | 26.2 | none |
| nopeeks | 36 | 42.7 | 46.2 | 45.2 | 14.6 | 2.2 | 0.0 | 8.6 | 9.5 | 0.2 | 20.8 | 2.3 | 1.8 | 16.8 | 0.0 | 0.0 | 0.0 | 12.9 | 0.1 | 0.5 | 0.1 | 47.5 | 19.2 | 19.2 | none |
| default | 40 | 67.6 | 75.2 | 70.8 | 29.1 | 24.1 | 18.6 | 0.0 | 10.2 | 0.1 | 16.6 | 4.0 | 0.0 | 0.0 | 10.9 | 9.4 | 2.1 | 0.0 | 0.1 | 1.1 | 0.0 | 94.6 | 49.8 | 49.8 | Notepad 123 (124 rows) |
