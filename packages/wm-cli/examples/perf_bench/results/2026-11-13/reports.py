"""Profiler reports in perf.log, mapped to the bench runs that produced them.

Usage:
  reports.py <date> <perf.log> <run log>... -- medians [scenario...]
  reports.py <date> <perf.log> <run log>... -- show <mode> <round> <scenario> [pick]
  reports.py <date> <perf.log> <run log>... -- perburst <scenario> <bursts> [stage...]
  reports.py <date> <perf.log> <run log>... -- series <scenario> <stage>

`<date>` is the local date of the runs (YYYY-MM-DD). Run logs are the
timestamped console output of `isolate_modes.ps1`/`bench-run.ps1`, in
12-hour ("9:56:06 PM") or 24-hour ("21:56:06") form. Each report is mapped
to the `Measuring '<scenario>'` window it falls in.

`medians` prints, per scenario and mode, the median over all its reports of
each "outside frames" stage, `to first frame`, interval p99/max, and the
worst `[sync]` row. `show` prints one full report: `pick` is `max` (the
longest interval max, default) or `median` (the median one by it).
"""

import datetime
import re
import statistics
import sys
from collections import defaultdict

LINE = re.compile(r"^(\d{1,2}:\d{2}:\d{2}(?: [AP]M)?) (.*)$")
ROUND = re.compile(r"Round (\d+)/\d+: restarting GlazeWM as '([^']+)'")
MEASURE = re.compile(r"Measuring '([^']+)'")
HEADER = re.compile(r"^(\S+Z)\s+INFO\s+\S+: perf \[")
STAGE2 = re.compile(r"^\s+(\S+)\s+(\d+)\s+([\d.]+)ms\s*$")
INTERVAL = re.compile(
    r"^\s*frame interval\s+([\d.]+)ms\s+([\d.]+)ms\s+([\d.]+)ms\s+([\d.]+)ms"
)
FIRST = re.compile(r"^\s*to first frame\s+\d+\s+([\d.]+)ms")
PROC = re.compile(r"^\s*(\S+) \[(sync|async)\]\s+(\d+)\s+([\d.]+)ms\s+([\d.]+)ms\s+([\d.]+)ms")


def parse_time(date, text):
    fmt = "%I:%M:%S %p" if text.endswith("M") else "%H:%M:%S"
    time = datetime.datetime.strptime(text, fmt).time()
    local = datetime.datetime.combine(date, time)
    return datetime.datetime.fromtimestamp(local.timestamp(), datetime.timezone.utc)


def read_windows(date, path):
    events = []
    with open(path, encoding="utf-8-sig", errors="replace") as file:
        for line in file:
            if m := LINE.match(line.rstrip()):
                events.append((parse_time(date, m.group(1)), m.group(2)))
    windows = []
    mode = round_ = None
    for i, (time, text) in enumerate(events):
        if m := ROUND.search(text):
            round_, mode = m.group(1), m.group(2)
        elif (m := MEASURE.search(text)) and mode:
            end = events[i + 1][0] if i + 1 < len(events) else time
            windows.append((time, end + datetime.timedelta(seconds=1), mode, round_, m.group(1)))
    return windows


def read_reports(path):
    reports, current = [], None
    with open(path, encoding="utf-8", errors="replace") as file:
        for raw in file:
            line = re.sub(r"\x1b\[[0-9;]*m", "", raw.rstrip("\n"))
            if m := HEADER.match(line):
                current = {"time": datetime.datetime.fromisoformat(m.group(1).replace("Z", "+00:00")),
                           "text": [line], "outside": {}, "procs": [], "section": None}
                reports.append(current)
                continue
            if current is None:
                continue
            if line and not line.startswith(" "):
                current = None
                continue
            current["text"].append(line)
            if "-- outside frames" in line:
                current["section"] = "outside"
            elif line.strip().startswith("--"):
                current["section"] = "other"
            elif current["section"] == "outside" and (m := STAGE2.match(line)):
                current["outside"][m.group(1)] = float(m.group(3))
            if m := INTERVAL.match(line):
                current["p99"], current["max"] = float(m.group(3)), float(m.group(4))
            if m := FIRST.match(line):
                current["first"] = float(m.group(1))
            if m := PROC.match(line):
                current["procs"].append((m.group(1), m.group(2), float(m.group(6))))
    return reports


def assign(reports, windows):
    grouped = defaultdict(list)
    for report in reports:
        for start, end, mode, round_, scenario in windows:
            if start <= report["time"] <= end:
                grouped[(mode, round_, scenario)].append(report)
                break
    return grouped


def med(values):
    return statistics.median(values) if values else float("nan")


def medians(grouped, scenarios):
    modes = list(dict.fromkeys(k[0] for k in grouped))
    scenarios = scenarios or list(dict.fromkeys(k[2] for k in grouped))
    stages = ["command", "platform_sync", "redraw", "redraw_loop", "anim_step", "peek_new",
              "rd_frozen", "take_over", "preposition", "cloak", "dwm_flush", "session_begin",
              "session_overlays", "peek_cover", "peek_handover", "peek_raise", "ovl_place",
              "ovl_blend", "sync_tail", "batch_commit"]
    for scenario in scenarios:
        print(f"\n## {scenario}: medians over reports (outside-frames ms unless noted)\n")
        print("| mode | n | " + " | ".join(stages) + " | first frame | int p99 | int max | worst sync row |")
        print("|" + " --- |" * (len(stages) + 6))
        for mode in modes:
            reps = [r for k, rs in grouped.items() if k[0] == mode and k[2] == scenario for r in rs]
            if not reps:
                continue
            cells = [f"{med([r['outside'].get(s, 0.0) for r in reps]):.1f}" for s in stages]
            syncs = [p for r in reps for p in r["procs"] if p[1] == "sync"]
            worst = max(syncs, key=lambda p: p[2]) if syncs else None
            sync_text = f"{worst[0]} {worst[2]:.0f} ({len(syncs)} rows)" if worst else "none"
            print(f"| {mode} | {len(reps)} | " + " | ".join(cells) +
                  f" | {med([r['first'] for r in reps if 'first' in r]):.1f}"
                  f" | {med([r['p99'] for r in reps if 'p99' in r]):.1f}"
                  f" | {med([r['max'] for r in reps if 'max' in r]):.1f} | {sync_text} |")


def per_burst(grouped, scenario, bursts, stages):
    """Outside-frames stage totals per burst: summed over each run's
    reports, divided by its bursts, then the median over rounds. Reports
    don't map one-to-one to bursts (one can span several), sums do."""
    modes = list(dict.fromkeys(k[0] for k in grouped))
    stages = stages or sorted({s for rs in grouped.values() for r in rs for s in r["outside"]})
    print(f"\n## {scenario}: outside-frames ms per burst (median over rounds)\n")
    print("| stage | " + " | ".join(modes) + " |")
    print("|" + " --- |" * (len(modes) + 1))
    for stage in stages:
        cells = []
        for mode in modes:
            runs = [sum(r["outside"].get(stage, 0.0) for r in rs) / bursts
                    for k, rs in grouped.items() if k[0] == mode and k[2] == scenario]
            cells.append(f"{med(runs):.1f}" if runs else "")
        print(f"| {stage} | " + " | ".join(cells) + " |")


def series(grouped, scenario, stage):
    """One outside-frames stage per report, in order, per mode and round."""
    for (mode, round_, scen), reps in grouped.items():
        if scen != scenario:
            continue
        values = " ".join(f"{r['outside'].get(stage, 0.0):.1f}" for r in reps)
        print(f"{mode} #{round_}: {values}")


def show(grouped, mode, round_, scenario, pick):
    reps = [r for r in grouped.get((mode, round_, scenario), []) if "max" in r]
    if not reps:
        print(f"no reports for {mode} #{round_} {scenario}")
        return
    reps.sort(key=lambda r: r["max"])
    report = reps[-1] if pick == "max" else reps[len(reps) // 2]
    print(f"=== {mode} #{round_} {scenario} ({pick} by interval max: {report['max']:.2f} ms, "
          f"{len(reps)} reports)")
    print("\n".join(report["text"]))


def main():
    args = sys.argv[1:]
    split = args.index("--")
    date = datetime.date.fromisoformat(args[0])
    perf, logs, command = args[1], args[2:split], args[split + 1:]
    windows = [w for log in logs for w in read_windows(date, log)]
    grouped = assign(read_reports(perf), windows)
    if command[0] == "medians":
        medians(grouped, command[1:])
    elif command[0] == "series":
        series(grouped, command[1], command[2])
    elif command[0] == "perburst":
        per_burst(grouped, command[1], int(command[2]), command[3:])
    else:
        show(grouped, command[1], command[2], command[3], command[4] if len(command) > 4 else "max")


if __name__ == "__main__":
    main()
