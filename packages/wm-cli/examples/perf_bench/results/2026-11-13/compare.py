"""Each GLAZEWM_ISOLATE word against its baseline, from perf_bench results.

Usage: compare.py <load0 file> <cpu8 file> <baseline>:<mode>[,<mode>...] ...

Reads every `| label | scenario | ... |` row of both files (closing tables
repeat rows; each counts once). Per scenario, prints for each mode the
median over rounds (range beside it) at load 0 and CPU 8, the change from
the baseline, and whether the rounds separate: every round of one mode is
on the same side of every round of the other (`sep`), else the change is
noise (`~`).
"""

import statistics
import sys

METRICS = [
    "input->first frame ms",
    "command p50 ms",
    "command ms (WM side)",
    "frame p90 ms",
    "interval p90 ms",
    "interval p99 ms",
    "interval max ms (median burst)",
    "frames",
    "peek_new ms (outside frames)",
    "peek_cover ms (outside frames)",
    "peek_handover ms (outside frames)",
    "peek_raise ms (outside frames)",
    "cloak ms (outside frames)",
    "session_overlays ms (outside frames)",
]


def read(path):
    header, rows = None, {}
    with open(path, encoding="utf-8-sig") as file:
        for line in file:
            line = line.strip()
            if not line.startswith("| ") or line.startswith("| ---"):
                continue
            cells = [c.strip() for c in line.strip("|").split("|")]
            if cells[0] == "label":
                header = cells
            elif header and len(cells) == len(header):
                rows[(cells[0], cells[1])] = dict(zip(header, cells))
    data = {}
    for (label, scenario), row in rows.items():
        mode = label.rpartition(" #")[0]
        for metric in METRICS:
            try:
                data.setdefault((mode, scenario, metric), []).append(float(row[metric]))
            except (KeyError, ValueError):
                pass
    return data


def cell(data, base, mode, scenario, metric):
    word = data.get((mode, scenario, metric))
    if not word:
        return "", ""
    value = statistics.median(word)
    shown = f"{value:.2f} <sub>{min(word):.2f}-{max(word):.2f}</sub>"
    if mode == base:
        return shown, ""
    ref = data.get((base, scenario, metric))
    if not ref:
        return shown, ""
    ref_value = statistics.median(ref)
    delta = value - ref_value
    separate = max(word) < min(ref) or min(word) > max(ref)
    pct = f" {delta / ref_value * 100:+.0f}%" if ref_value else ""
    return shown, f"{delta:+.2f}{pct} {'**sep**' if separate else '~'}"


def main():
    load0, cpu8 = read(sys.argv[1]), read(sys.argv[2])
    groups = [arg.split(":") for arg in sys.argv[3:]]
    scenarios = list(dict.fromkeys(k[1] for k in load0))
    for base, modes in groups:
        modes = [base] + modes.split(",")
        for scenario in scenarios:
            if (base, scenario, "command p50 ms") not in load0:
                continue
            print(f"\n### {scenario}: vs `{base}`\n")
            print("| metric | mode | load 0 | change | CPU 8 | change |")
            print("| --- | --- | ---: | --- | ---: | --- |")
            for metric in METRICS:
                for mode in modes:
                    a, da = cell(load0, base, mode, scenario, metric)
                    b, db = cell(cpu8, base, mode, scenario, metric)
                    if a or b:
                        print(f"| {metric} | {mode} | {a} | {da} | {b} | {db} |")


if __name__ == "__main__":
    main()
