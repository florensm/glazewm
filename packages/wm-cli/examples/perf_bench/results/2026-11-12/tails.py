"""Frame interval tails (p99, max) per mode and scenario, from perf.log.

Usage: tails.py <title> <run log> [<run log> ...]

perf_bench only keeps interval p50/p90. This maps each perf.log report to
the scenario run it belongs to, using the timestamped run logs (the
"Round ..."/"Measuring ..." lines of a bench script's output), and
summarises p99 and max of the frame interval: whether long frames actually
missed vblanks.
"""

import datetime
import os
import re
import statistics
import sys

MODES = ["main", "nopeeks", "default", "morepeeks"]
SCENARIOS = ["move", "float", "minimize", "resize", "switch"]
DATE = datetime.date(2026, 11, 12)

LINE = re.compile(r"^(\d{1,2}:\d{2}:\d{2} [AP]M) (.*)$")
ROUND = re.compile(r"Round (\d+)/\d+: restarting GlazeWM as '([^']+)'")
MEASURE = re.compile(r"Measuring '([^']+)'")
REPORT = re.compile(r"^(\S+Z)\s+INFO\s+\S+: perf \[")
INTERVAL = re.compile(r"^\s*frame interval\s+([\d.]+)ms\s+([\d.]+)ms\s+([\d.]+)ms\s+([\d.]+)ms")


def local_to_utc(text):
    time = datetime.datetime.strptime(text, "%I:%M:%S %p").time()
    local = datetime.datetime.combine(DATE, time)
    return datetime.datetime.fromtimestamp(local.timestamp(), datetime.timezone.utc)


def read_windows(path):
    """(start, end, mode, round, scenario) per measured scenario."""
    events = []
    with open(path, encoding="utf-8-sig", errors="replace") as file:
        for line in file:
            match = LINE.match(line.rstrip())
            if match:
                events.append((local_to_utc(match.group(1)), match.group(2)))
    windows = []
    mode = round_ = None
    for index, (time, text) in enumerate(events):
        if m := ROUND.search(text):
            round_, mode = m.group(1), m.group(2)
        elif (m := MEASURE.search(text)) and mode:
            # The next line is printed once perf_bench has finished.
            end = events[index + 1][0] if index + 1 < len(events) else time
            windows.append((time, end + datetime.timedelta(seconds=1), mode, round_, m.group(1)))
    return windows


def read_reports():
    # The copy saved beside this script, else the live log.
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "logs", "perf.log")
    if not os.path.exists(path):
        path = os.path.expanduser("~/.glzr/glazewm/perf.log")
    reports = []
    current = None
    with open(path, encoding="utf-8", errors="replace") as file:
        for line in file:
            line = re.sub(r"\x1b\[[0-9;]*m", "", line)
            if m := REPORT.match(line):
                current = datetime.datetime.fromisoformat(m.group(1).replace("Z", "+00:00"))
            elif current and (m := INTERVAL.match(line)):
                reports.append((current, float(m.group(3)), float(m.group(4))))
                current = None
    return reports


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    title, paths = sys.argv[1], sys.argv[2:]
    windows = [w for path in paths for w in read_windows(path)]
    reports = read_reports()
    # (mode, scenario) -> per-round medians of p99 and of max.
    data = {}
    for start, end, mode, round_, scenario in windows:
        inside = [r for r in reports if start <= r[0] <= end]
        if not inside:
            continue
        cell = data.setdefault((mode, scenario), ([], []))
        cell[0].append(statistics.median(r[1] for r in inside))
        cell[1].append(statistics.median(r[2] for r in inside))
    modes = [m for m in MODES if any(k[0] == m for k in data)]

    def fmt(values):
        if not values:
            return "–"
        return (
            f"{statistics.median(values):.1f} "
            f"<sub>{min(values):.1f}–{max(values):.1f}</sub>"
        )

    print(f"#### {title}: frame interval tails")
    print()
    print(
        "From `perf.log`: per run, the median over its burst reports of "
        "interval p99 and max (ms). Then median over rounds, with the "
        "rounds' range. 5.75 ms is one vblank, ~11.5 ms one missed."
    )
    print()
    header = " | ".join(f"{m} p99 | {m} max" for m in modes)
    print(f"| scenario | {header} |")
    print("| --- |" + " ---: |" * (2 * len(modes)))
    for scenario in SCENARIOS:
        cells = []
        for mode in modes:
            p99, peak = data.get((mode, scenario), ([], []))
            cells += [fmt(p99), fmt(peak)]
        print(f"| {scenario} | " + " | ".join(cells) + " |")


main()
