
## peek-load0 resize

| mode | reports | with sync rows | sync rows | worst per report (median of those with any) | worst | by process (rows, worst ms) | interval max median | interval max worst |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| nopeeks | 39 | 0 | 0 | 0.0 | 0.0 |  | 17.3 | 23.9 |
| nopeeks+keepoverlays | 37 | 0 | 0 | 0.0 | 0.0 |  | 18.2 | 24.6 |
| default | 39 | 39 | 120 | 9.5 | 122.3 | Notepad 3x/122, FPilot 39x/20, WindowsTerminal 39x/15, chrome 39x/9 | 43.3 | 170.4 |
| keeppeeks | 38 | 38 | 116 | 7.6 | 122.2 | Notepad 2x/122, WindowsTerminal 38x/19, FPilot 38x/13, chrome 38x/8 | 43.9 | 165.5 |
| asynchandback | 37 | 0 | 0 | 0.0 | 0.0 |  | 36.4 | 49.3 |
| asynclanding | 38 | 38 | 152 | 3.7 | 12.1 | FPilot 38x/12, chrome 38x/10, WindowsTerminal 38x/6, Notepad 38x/4 | 35.3 | 49.8 |
| earlylanding | 38 | 38 | 152 | 3.3 | 4.8 | WindowsTerminal 38x/5, chrome 38x/5, FPilot 38x/4, Notepad 38x/3 | 30.4 | 36.7 |
| earlylanding+asynchandback+keeppeeks | 38 | 0 | 0 | 0.0 | 0.0 |  | 29.6 | 37.4 |

## peek-cpu8 resize

| mode | reports | with sync rows | sync rows | worst per report (median of those with any) | worst | by process (rows, worst ms) | interval max median | interval max worst |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| nopeeks | 37 | 0 | 0 | 0.0 | 0.0 |  | 17.8 | 22.9 |
| nopeeks+keepoverlays | 38 | 0 | 0 | 0.0 | 0.0 |  | 19.7 | 25.3 |
| default | 39 | 39 | 120 | 7.5 | 124.0 | Notepad 3x/124, chrome 39x/23, FPilot 39x/22, WindowsTerminal 39x/6 | 39.9 | 192.7 |
| keeppeeks | 39 | 39 | 120 | 7.2 | 118.1 | Notepad 3x/118, FPilot 39x/16, WindowsTerminal 39x/15, chrome 39x/10 | 46.2 | 178.7 |
| asynchandback | 38 | 0 | 0 | 0.0 | 0.0 |  | 39.6 | 56.1 |
| asynclanding | 38 | 38 | 143 | 3.8 | 115.2 | Notepad 29x/115, WindowsTerminal 38x/15, chrome 38x/7, FPilot 38x/5 | 38.1 | 127.4 |
| earlylanding | 37 | 37 | 148 | 3.0 | 5.0 | WindowsTerminal 37x/5, Notepad 37x/4, FPilot 37x/4, chrome 37x/3 | 30.8 | 36.2 |
| earlylanding+asynchandback+keeppeeks | 39 | 0 | 0 | 0.0 | 0.0 |  | 32.2 | 36.6 |

## more-load0 resize

| mode | reports | with sync rows | sync rows | worst per report (median of those with any) | worst | by process (rows, worst ms) | interval max median | interval max worst |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| morepeeks | 38 | 38 | 114 | 7.1 | 19.1 | FPilot 38x/19, WindowsTerminal 38x/18, chrome 38x/8 | 38.6 | 63.5 |
| morepeeks+keeppeeks | 39 | 39 | 117 | 7.3 | 18.9 | WindowsTerminal 39x/19, FPilot 39x/18, chrome 39x/15 | 40.0 | 65.1 |
| morepeeks+earlylanding+asynchandback+keeppeeks | 37 | 0 | 0 | 0.0 | 0.0 |  | 27.3 | 34.4 |

## more-cpu8 resize

| mode | reports | with sync rows | sync rows | worst per report (median of those with any) | worst | by process (rows, worst ms) | interval max median | interval max worst |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| morepeeks | 38 | 38 | 114 | 7.2 | 21.5 | FPilot 38x/22, WindowsTerminal 38x/12, chrome 38x/9 | 40.3 | 65.5 |
| morepeeks+keeppeeks | 37 | 37 | 111 | 6.5 | 28.2 | FPilot 37x/28, WindowsTerminal 37x/15, chrome 37x/8 | 42.6 | 67.4 |
| morepeeks+earlylanding+asynchandback+keeppeeks | 40 | 0 | 0 | 0.0 | 0.0 |  | 32.3 | 36.5 |

## main-load0 resize

| mode | reports | with sync rows | sync rows | worst per report (median of those with any) | worst | by process (rows, worst ms) | interval max median | interval max worst |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| main | 37 | 0 | 0 | 0.0 | 0.0 |  | 20.7 | 31.2 |
| nopeeks | 37 | 0 | 0 | 0.0 | 0.0 |  | 16.8 | 20.7 |
| default | 38 | 38 | 116 | 6.6 | 119.8 | Notepad 2x/120, FPilot 38x/18, WindowsTerminal 38x/9, chrome 38x/9 | 39.0 | 169.9 |

## main-cpu8 resize

| mode | reports | with sync rows | sync rows | worst per report (median of those with any) | worst | by process (rows, worst ms) | interval max median | interval max worst |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| main | 40 | 0 | 0 | 0.0 | 0.0 |  | 26.2 | 39.3 |
| nopeeks | 36 | 0 | 0 | 0.0 | 0.0 |  | 19.2 | 25.3 |
| default | 40 | 40 | 124 | 7.4 | 123.2 | Notepad 4x/123, WindowsTerminal 40x/30, FPilot 40x/21, chrome 40x/19 | 49.8 | 183.3 |
