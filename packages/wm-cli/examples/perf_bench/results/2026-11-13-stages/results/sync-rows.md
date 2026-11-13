
## resize-load0

| mode | reports | with sync rows | sync rows | worst per report (median of those with any) | worst | by process (rows, worst ms) | interval max median | interval max worst |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| nopeeks | 40 | 0 | 0 | 0.0 | 0.0 |  | 14.3 | 18.6 |
| nopeeks+nokeepoverlays | 40 | 0 | 0 | 0.0 | 0.0 |  | 13.9 | 18.8 |
| default | 40 | 0 | 0 | 0.0 | 0.0 |  | 35.2 | 44.1 |
| noasynchandback | 40 | 40 | 120 | 9.7 | 16.5 | WindowsTerminal 40x/16, FPilot 40x/16, chrome 40x/8 | 42.6 | 66.4 |

## resize-cpu8

| mode | reports | with sync rows | sync rows | worst per report (median of those with any) | worst | by process (rows, worst ms) | interval max median | interval max worst |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |
| nopeeks | 40 | 0 | 0 | 0.0 | 0.0 |  | 25.9 | 91.4 |
| nopeeks+nokeepoverlays | 40 | 0 | 0 | 0.0 | 0.0 |  | 26.4 | 49.4 |
| default | 40 | 0 | 0 | 0.0 | 0.0 |  | 51.7 | 69.7 |
| noasynchandback | 40 | 40 | 120 | 12.9 | 24.5 | FPilot 40x/25, chrome 40x/23, WindowsTerminal 40x/19 | 55.4 | 99.3 |
