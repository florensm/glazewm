"""One table per scenario from perf_bench results: median over rounds, load 0 / CPU 8.

Usage: tables.py <scenario> <load0 file>,<cpu8 file>[,<suffix>] ...

Each pair adds a row per mode found in it, the mode named `<mode><suffix>`
(e.g. ` (main run)` for the `nopeeks`/`default` rounds of the `main` run).
"""

import statistics
import sys

from compare import read

METRICS = [
    ("command p50 ms", "command p50"),
    ("input->first frame ms", "input→first frame"),
    ("frame p90 ms", "frame p90"),
    ("interval p90 ms", "interval p90"),
    ("interval p99 ms", "interval p99"),
    ("interval max ms (median burst)", "interval max (median burst)"),
]


def value(data, mode, scenario, metric):
    values = data.get((mode, scenario, metric))
    return f"{statistics.median(values):.2f}" if values else "–"


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    scenario = sys.argv[1]
    print(f"\n### {scenario} (ms, median over rounds, load 0 / CPU 8)\n")
    print("| mode | " + " | ".join(label for _, label in METRICS) + " |")
    print("| --- |" + " ---: |" * len(METRICS))
    for arg in sys.argv[2:]:
        parts = arg.split(",")
        load0, cpu8 = read(parts[0]), read(parts[1])
        suffix = parts[2] if len(parts) > 2 else ""
        modes = list(dict.fromkeys(k[0] for k in load0 if k[1] == scenario))
        for mode in modes:
            cells = [f"{value(load0, mode, scenario, m)} / {value(cpu8, mode, scenario, m)}"
                     for m, _ in METRICS]
            print(f"| `{mode}`{suffix} | " + " | ".join(cells) + " |")


if __name__ == "__main__":
    main()
