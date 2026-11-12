# Side finding: tile order after a restart

This was the first interleaved run (4 rounds, no CPU load, tile order
left as each restart made it). It revealed that the tile order **flips
between two states on every restart**: chrome came up leftmost on
restarts 1, 3, 5, …, and rightmost (or next to it) on 2, 4, 6, ….
With 4 modes in turn, every mode therefore always got the same state:

| mode | chrome's position (of 7) in rounds 1–4 |
| --- | --- |
| `main` | 1, 1, 1, 1 |
| `nopeeks` | 7, 7, 7, 7 |
| `default` | 1, 1, 1, 1 |
| `morepeeks` | 7, 7, 7, 6 |

So only **`main`↔`default`** and **`nopeeks`↔`morepeeks`** compare like
with like in the tables below. The other pairs mostly measure chrome's
position. For example, `default` vs `nopeeks` on `move` shows +118%
`tick`, which comes from chrome's leftmost spot (one burst can't move
and the neighbour differs), not from the drawing. The same-position pairs
agree with everything else in this report. Peeks on `resize`: frame p90
−52% (`default` vs `main`) and −51% (`morepeeks` vs `nopeeks`).
Commands start 32–85% later.

The requested runs had the same flip, unlogged for the tryout modes.
With 3 modes, each mode's 3 rounds got a 2:1 mix of the two states
(`nopeeks`/`morepeeks` one way, `default` the other), and so did the
`main` block. That adds noise and could tilt a median, which is why the
fixed-order runs above take precedence where they disagree.
