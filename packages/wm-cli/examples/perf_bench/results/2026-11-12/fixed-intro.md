# Expansion: all four builds interleaved, fixed tile order

These are the most controlled numbers in this report, and the verdict
leans on them first. They fix the two weaknesses of the requested runs:

- **Drift**: the WM restarts as `main`, `nopeeks`, `default`,
  `morepeeks` in turn, round after round, so drift over the session hits
  every build alike.
- **Tile order**: a restart flips the tiles between two orders, which
  changes chrome's neighbours. An earlier interleaved run, at the end of
  this report, showed this. After every restart the tiles are now
  arranged the same way: Windows Terminal, Notepad, **chrome**,
  Everything, Explorer, Settings, Store. That is 7 `move --direction
  left` commands at most, over one IPC connection. Chrome is never at an
  edge, so every `move` burst animates.

Same scenarios, bursts, windows and target as above, 4 rounds:
`bench-run.ps1 -Modes main,nopeeks,default,morepeeks -Rounds 4
-TileOrder …`, with and without `-CpuLoad 8`. With 4 rounds against 4,
separation by pure chance happens in under 3% of comparisons, against
10% with 3 rounds.
