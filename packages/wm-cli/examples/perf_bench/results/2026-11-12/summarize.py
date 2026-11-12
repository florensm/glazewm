"""Turns perf_bench result files into per-scenario Markdown tables.

Usage: summarize.py <title> <file> [<file> ...]

Every `| label | scenario | ... |` row of the files is read (rows repeated
in a file's closing tables count once). Each cell is the median over rounds,
with the rounds' range beside it. Comparisons call a difference real only
when the two modes' round ranges do not overlap.
"""

import statistics
import sys

MODES = ["main", "nopeeks", "default", "morepeeks"]
SCENARIOS = ["move", "float", "minimize", "resize", "switch"]
METRICS = [
    "interval p50 ms",
    "interval p90 ms",
    "frame p90 ms",
    "frames >2x budget",
    "tick ms/frame",
    "peek_frame ms/frame",
    "input->first frame ms",
    "command p50 ms",
]
# Compared in the verdict matrix. Interval p50 is the refresh period in
# every mode, so it decides nothing.
# peek_frame is left out: it runs only in peek modes, and includes the
# wait for the WM thread, so it is no cost to compare against `tick`.
JUDGED = [
    m for m in METRICS if m not in ("interval p50 ms", "peek_frame ms/frame")
]


def not_measured(mode, metric):
    """Cells a build cannot report, rather than a measured zero."""
    if metric == "peek_frame ms/frame":
        return mode in ("main", "nopeeks")
    if metric == "input->first frame ms":
        # The peek path never calls `perf::note_animation_start`, so no
        # input is paired with its first frame.
        return mode in ("default", "morepeeks")
    return False


def read_rows(paths):
    header = None
    rows = {}
    for path in paths:
        with open(path, encoding="utf-8-sig") as file:
            for line in file:
                line = line.strip()
                if not line.startswith("| ") or line.startswith("| ---"):
                    continue
                cells = [c.strip() for c in line.strip("|").split("|")]
                if cells[0] == "label":
                    header = cells
                    continue
                if header is None or len(cells) != len(header):
                    continue
                rows[(cells[0], cells[1])] = dict(zip(header, cells))
    data = {}
    for (label, scenario), row in rows.items():
        mode, _, round_ = label.partition(" #")
        for metric in METRICS:
            try:
                value = float(row[metric])
            except (KeyError, ValueError):
                continue
            data.setdefault((mode, scenario, metric), []).append(value)
    return data


def fmt(value):
    return f"{value:.2f}"


def values_of(data, mode, scenario, metric):
    """Measured values, or `None` where the metric was not recorded."""
    if not_measured(mode, metric):
        return None
    values = data.get((mode, scenario, metric))
    # Latency is only paired for inputs whose animation went through
    # `note_animation_start`; a switch never does, and reports 0.
    if metric == "input->first frame ms" and values and not any(values):
        return None
    return values


def cell(data, mode, scenario, metric):
    values = values_of(data, mode, scenario, metric)
    if values is None:
        return "n/a"
    if not values:
        return "–"
    median = statistics.median(values)
    if len(values) == 1:
        return fmt(median)
    return f"{fmt(median)} <sub>{fmt(min(values))}–{fmt(max(values))}</sub>"


def compare(data, mode, other, scenario, metric):
    a = values_of(data, mode, scenario, metric)
    b = values_of(data, other, scenario, metric)
    if a is None or b is None:
        return "n/a"
    if not a or not b:
        return "–"
    ma, mb = statistics.median(a), statistics.median(b)
    if max(a) < min(b):
        word = "better"
    elif min(a) > max(b):
        word = "worse"
    else:
        return "≈"
    if mb == 0:
        return f"{word} ({fmt(ma)} vs 0)"
    change = (ma - mb) / mb * 100
    # Separated, but too small to matter (e.g. 5.76 vs 5.75 ms).
    if abs(change) < 5:
        return "≈"
    return f"{word} ({change:+.0f}%)"


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    title, paths = sys.argv[1], sys.argv[2:]
    data = read_rows(paths)
    modes = [m for m in MODES if any(k[0] == m for k in data)]
    rounds = max((len(v) for v in data.values()), default=0)
    print(f"## {title}")
    print()
    print(
        f"Median over rounds (up to {rounds}), round range in small print. "
        "Lower is better, except interval p50, which is the refresh period."
    )
    for scenario in SCENARIOS:
        if not any(k[1] == scenario for k in data):
            continue
        print()
        print(f"### {scenario}")
        print()
        print("| metric | " + " | ".join(modes) + " |")
        print("| --- |" + " ---: |" * len(modes))
        for metric in METRICS:
            cells = [cell(data, mode, scenario, metric) for mode in modes]
            print(f"| {metric} | " + " | ".join(cells) + " |")
        pairs = [
            (a, b)
            for a in ("default", "morepeeks")
            for b in ("main", "nopeeks")
            if a in modes and b in modes
        ]
        if "default" in modes and "morepeeks" in modes:
            pairs.append(("morepeeks", "default"))
        # The tryout build's own change, with peek drawing off.
        if "nopeeks" in modes and "main" in modes:
            pairs.append(("nopeeks", "main"))
        if not pairs:
            continue
        print()
        print(
            "| vs (ranges must not overlap) | "
            + " | ".join(f"{a} vs {b}" for a, b in pairs)
            + " |"
        )
        print("| --- |" + " --- |" * len(pairs))
        for metric in JUDGED:
            cells = [compare(data, a, b, scenario, metric) for a, b in pairs]
            print(f"| {metric} | " + " | ".join(cells) + " |")


main()
