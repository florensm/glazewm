"""Profiler stages in perf.log, per report, mapped to the bench runs.

Usage:
  stages.py <date> <perf.log> <run log>... -- split <scenario> <mode>...
  stages.py <date> <perf.log> <run log>... -- first <scenario> <mode>...
  stages.py <date> <perf.log> <run log>... -- peeknew <scenario> <mode>...
  stages.py <date> <perf.log> <run log>... -- gaps <scenario> <mode>...
  stages.py <date> <perf.log> <run log>... -- perburst <scenario> <bursts> <mode>...
  stages.py <date> <perf.log> <run log>... -- show <scenario> <mode> median|max

Reports are mapped to runs as in `reports.py`, whose `read_windows` it uses.
All values are medians over a mode's reports (one per burst on `resize`).

`split`: the second-round stages in the slowest input's timeline, before the
command is answered and inside the first `peek_frame`. The timeline leaves
out entries under 0.3 ms, so its calls count only those; the outside-frames
columns are exact but also hold the syncs after the command.
`first`: the first `peek_frame`: total, the wait from the timer's wake to the
WM thread's tick, `peek_draw`, `peek_track`, the time from the last draw's
end to the frame's (bounds `peek_track` when under 0.3 ms),
`peek_ring`/`peek_backdrop` in it, and `peek_frame` - `peek_draw` -
`peek_track`.
`peeknew`: outside-frames `peek_new` against its children.
`gaps`: parent minus its children, in the frame tree and outside frames.
`perburst`: outside-frames sums over each run / bursts, median over rounds.
`show`: one full report of a mode, over all its rounds: the one with the
median interval max (a typical burst) or the longest.
"""

import re
import statistics
import sys
from collections import defaultdict

from reports import read_windows
import datetime

HEADER = re.compile(r"^(\S+Z)\s+INFO\s+\S+: perf \[")
TREE = re.compile(r"^  ( *)(\S+)\s+(\d+)\s+([\d.]+)ms\s+([\d.]+)ms\s+([\d.]+)ms\s*$")
FLAT = re.compile(r"^  (\S+)\s+(\d+)\s+([\d.]+)ms\s*$")
TRACE = re.compile(r"^  (\* )?( *)(\S+)\s+([\d.]+)ms\s+([\d.]+)ms\s*$")
INTERVAL = re.compile(r"^\s*frame interval\s+([\d.]+)ms\s+([\d.]+)ms\s+([\d.]+)ms\s+([\d.]+)ms")
FIRST = re.compile(r"^\s*to first frame\s+\d+\s+([\d.]+)ms")

NEW = ["peek_draw", "peek_track", "peek_ring", "peek_backdrop", "peek_styles", "peek_warm",
       "sg_window", "sg_thumb", "ws_surrogates", "ws_park", "ws_show", "ws_start",
       "sync_focus", "tab_bars", "overview", "guards"]

# Outside frames the table is flat: parents and the children they hold.
OUTSIDE_TREE = {
    "platform_sync": ["redraw", "sync_tail"],
    "redraw": ["redraw_prep", "redraw_loop", "peek_cover", "peek_handover", "peek_raise",
               "surrogate_flush", "session_overlays"],
    "redraw_prep": ["ws_surrogates", "ws_park", "ws_show", "ws_start"],
    "redraw_loop": ["anim_step", "rd_frozen", "rd_apply"],
    "rd_apply": ["rp_query", "rp_swp", "rp_visible", "uncloak"],
    "sync_tail": ["sync_focus", "tab_bars", "overview", "guards"],
    "peek_new": ["peek_styles", "peek_warm", "sg_window", "sg_thumb"],
}


def read_reports(path):
    reports, cur = [], None
    with open(path, encoding="utf-8", errors="replace") as file:
        for raw in file:
            line = re.sub(r"\x1b\[[0-9;]*m", "", raw.rstrip("\n"))
            if m := HEADER.match(line):
                cur = {"time": datetime.datetime.fromisoformat(m.group(1).replace("Z", "+00:00")),
                       "text": [line], "tree": [], "cross": {}, "outside": {}, "trace": [],
                       "section": "tree"}
                reports.append(cur)
                continue
            if cur is None:
                continue
            if line and not line.startswith(" "):
                cur = None
                continue
            cur["text"].append(line)
            stripped = line.strip()
            if stripped.startswith("-- called from several parents"):
                cur["section"] = "cross"
            elif stripped.startswith("-- outside frames"):
                cur["section"] = "outside"
            elif stripped.startswith("-- slowest input timeline"):
                cur["section"] = "trace"
            elif stripped.startswith("--"):
                cur["section"] = "other"
            elif cur["section"] == "tree" and (m := TREE.match(line)):
                cur["tree"].append((len(m.group(1)) // 2, m.group(2), int(m.group(3)),
                                    float(m.group(4))))
            elif cur["section"] == "cross" and (m := TREE.match(line)):
                cur["cross"][m.group(2)] = (int(m.group(3)), float(m.group(4)))
            elif cur["section"] == "outside" and (m := FLAT.match(line)):
                cur["outside"][m.group(1)] = (int(m.group(2)), float(m.group(3)))
            elif cur["section"] == "trace" and (m := TRACE.match(line)) and m.group(3) != "stage":
                cur["trace"].append({"cross": bool(m.group(1)), "depth": len(m.group(2)) // 2,
                                     "stage": m.group(3), "at": float(m.group(4)),
                                     "took": float(m.group(5))})
            if m := INTERVAL.match(line):
                cur["p99"], cur["max"] = float(m.group(3)), float(m.group(4))
            if m := FIRST.match(line):
                cur["first"] = float(m.group(1))
    return reports


def grouped_reports(argv):
    split = argv.index("--")
    date = datetime.date.fromisoformat(argv[0])
    perf, logs, command = argv[1], argv[2:split], argv[split + 1:]
    windows = [w for log in logs for w in read_windows(date, log)]
    grouped = defaultdict(list)
    for report in read_reports(perf):
        for start, end, mode, round_, scenario in windows:
            if start <= report["time"] <= end:
                grouped[(mode, round_, scenario)].append(report)
                break
    return grouped, command


def of(grouped, scenario, mode):
    return [r for (m, _, s), rs in grouped.items() if m == mode and s == scenario for r in rs]


def med(values):
    values = [v for v in values if v is not None]
    return statistics.median(values) if values else None


def fmt(value, digits=2):
    return "–" if value is None else f"{value:.{digits}f}"


def timeline(report):
    """The trace's command, the first `peek_frame` and the tick it waited for."""
    trace = report["trace"]
    command = next((e for e in trace if e["stage"] == "command" and e["depth"] == 0), None)
    frames = [e for e in trace if e["stage"] == "peek_frame"]
    frame = frames[-1] if frames else None
    tick = None
    if frame:
        tick = next((e for e in trace if e["stage"] == "tick" and e["depth"] == 0
                     and e["at"] >= frame["at"]), None)
    return command, frame, tick


def split(grouped, scenario, modes):
    for mode in modes:
        reps = [r for r in of(grouped, scenario, mode) if r["trace"]]
        print(f"\n### {scenario} `{mode}`: {len(reps)} reports\n")
        print("| stage | before answer ms | calls ≥0.3 ms | first peek_frame ms | calls ≥0.3 ms "
              "| outside frames ms | calls |")
        print("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
        rows = defaultdict(lambda: defaultdict(list))
        for r in reps:
            command, frame, tick = timeline(r)
            if not command:
                continue
            end = command["at"] + command["took"]
            start = tick["at"] if tick else None
            stop = frame["at"] + frame["took"] if frame else None
            for stage in NEW:
                entries = [e for e in r["trace"] if e["stage"] == stage]
                before = [e for e in entries if e["at"] + e["took"] <= end + 0.01]
                inside = [e for e in entries
                          if start is not None and start <= e["at"] <= stop]
                rows[stage]["before"].append(sum(e["took"] for e in before))
                rows[stage]["before_n"].append(len(before))
                rows[stage]["inside"].append(sum(e["took"] for e in inside) if frame else None)
                rows[stage]["inside_n"].append(len(inside) if frame else None)
                calls, total = r["outside"].get(stage, (0, 0.0))
                rows[stage]["out"].append(total)
                rows[stage]["out_n"].append(calls)
        for stage in NEW:
            c = rows[stage]
            print(f"| `{stage}` | {fmt(med(c['before']))} | {fmt(med(c['before_n']), 1)} "
                  f"| {fmt(med(c['inside']))} | {fmt(med(c['inside_n']), 1)} "
                  f"| {fmt(med(c['out']))} | {fmt(med(c['out_n']), 1)} |")


def first(grouped, scenario, modes):
    print(f"\n### {scenario}: first `peek_frame` (slowest input's timeline), medians\n")
    print("| mode | n | command | to first frame | peek_frame | wait (wake→tick) | peek_draw "
          "| draws | peek_track | after last draw | peek_ring in frame | peek_backdrop in frame "
          "| peek_frame − draw − track |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for mode in modes:
        cols = defaultdict(list)
        for r in of(grouped, scenario, mode):
            command, frame, tick = timeline(r)
            if not (command and frame and tick):
                continue
            stop = frame["at"] + frame["took"]
            def inside(stage):
                return [e for e in r["trace"] if e["stage"] == stage and tick["at"] <= e["at"] <= stop]
            draw = sum(e["took"] for e in inside("peek_draw"))
            track = sum(e["took"] for e in inside("peek_track"))
            cols["command"].append(command["took"])
            cols["first"].append(r.get("first"))
            cols["frame"].append(frame["took"])
            cols["wait"].append(tick["at"] - frame["at"])
            cols["draw"].append(draw)
            cols["draws"].append(len(inside("peek_draw")))
            cols["track"].append(track)
            draws = inside("peek_draw")
            cols["after"].append(stop - max(e["at"] + e["took"] for e in draws) if draws else None)
            cols["ring"].append(sum(e["took"] for e in inside("peek_ring")))
            cols["backdrop"].append(sum(e["took"] for e in inside("peek_backdrop")))
            cols["rest"].append(frame["took"] - draw - track)
        n = len(cols["frame"])
        print(f"| `{mode}` | {n} | " + " | ".join(
            fmt(med(cols[k]), 1 if k == "draws" else 2)
            for k in ["command", "first", "frame", "wait", "draw", "draws", "track", "after",
                      "ring", "backdrop", "rest"]) + " |")


def peeknew(grouped, scenario, modes):
    print(f"\n### {scenario}: `peek_new` outside frames, medians per report (ms, calls)\n")
    kids = OUTSIDE_TREE["peek_new"]
    print("| mode | n | peek_new | " + " | ".join(kids) + " | rest | rest % |")
    print("|" + " --- |" * (len(kids) + 5))
    for mode in modes:
        cols = defaultdict(list)
        for r in of(grouped, scenario, mode):
            if "peek_new" not in r["outside"]:
                continue
            calls, total = r["outside"]["peek_new"]
            cols["peek_new"].append(total)
            cols["peek_new_n"].append(calls)
            rest = total
            for kid in kids:
                kc, kt = r["outside"].get(kid, (0, 0.0))
                cols[kid].append(kt)
                cols[kid + "_n"].append(kc)
                rest -= kt
            cols["rest"].append(rest)
            cols["rest_pct"].append(rest / total * 100 if total else None)
        cells = [f"{fmt(med(cols[k]))} ({fmt(med(cols[k + '_n']), 0)})" for k in ["peek_new"] + kids]
        print(f"| `{mode}` | {len(cols['peek_new'])} | " + " | ".join(cells) +
              f" | {fmt(med(cols['rest']))} | {fmt(med(cols['rest_pct']), 0)} |")


def tree_gaps(report):
    """Frame tree: each stage's total minus its direct children's."""
    gaps = {}
    tree = report["tree"]
    for i, (depth, name, _, total) in enumerate(tree):
        kids = 0.0
        has = False
        for d, _, _, t in tree[i + 1:]:
            if d <= depth:
                break
            if d == depth + 1:
                kids += t
                has = True
        if has:
            gaps[f"frames: {name}"] = (total, kids)
    totals = {n: t for d, n, _, t in tree}
    cross = {k: v[1] for k, v in report["cross"].items()}
    if "peek_track" in totals:
        kids = cross.get("peek_ring", 0.0) + cross.get("peek_backdrop", 0.0)
        gaps["frames: peek_track (ring+backdrop)"] = (totals["peek_track"], kids)
    # `peek_frame` is listed beside `tick` but its drawing runs inside it.
    if "tick" in totals and "frames: tick" in gaps:
        kids = gaps["frames: tick"][1] + totals.get("peek_draw", 0.0) + totals.get("peek_track", 0.0)
        gaps["frames: tick (+peek_draw+peek_track)"] = (totals["tick"], kids)
    # Cross-cutting stages that can run directly in a parent, not in a child.
    if "frames: redraw" in gaps:
        kids = gaps["frames: redraw"][1] + cross.get("dwm_flush", 0.0) + cross.get("batch_commit", 0.0)
        gaps["frames: redraw (+all in-frame dwm_flush, batch_commit)"] = (totals["redraw"], kids)
    if "frames: sync_tail" in gaps:
        kids = gaps["frames: sync_tail"][1] + cross.get("backdrop_sync", 0.0) + cross.get("border_sync", 0.0)
        gaps["frames: sync_tail (+backdrop/border_sync)"] = (totals["sync_tail"], kids)
    return gaps


def outside_gaps(report):
    gaps = {}
    out = report["outside"]
    for parent, kids in OUTSIDE_TREE.items():
        if parent in out:
            gaps[f"outside: {parent}"] = (out[parent][1], sum(out.get(k, (0, 0.0))[1] for k in kids))
    if "redraw" in out:
        kids = OUTSIDE_TREE["redraw"] + ["cloak"]
        gaps["outside: redraw (+cloak)"] = (out["redraw"][1], sum(out.get(k, (0, 0.0))[1] for k in kids))
    if "sync_tail" in out:
        kids = OUTSIDE_TREE["sync_tail"] + ["backdrop_sync", "border_sync"]
        gaps["outside: sync_tail (+backdrop/border_sync)"] = (
            out["sync_tail"][1], sum(out.get(k, (0, 0.0))[1] for k in kids))
    return gaps


def gaps(grouped, scenario, modes):
    for mode in modes:
        cols = defaultdict(lambda: ([], []))
        for r in of(grouped, scenario, mode):
            for key, (total, kids) in {**tree_gaps(r), **outside_gaps(r)}.items():
                cols[key][0].append(total)
                cols[key][1].append(kids)
        print(f"\n### {scenario} `{mode}`: parent vs children, medians per report (ms)\n")
        print("| stage | total | children | gap | gap % |")
        print("| --- | ---: | ---: | ---: | ---: |")
        for key, (totals, kids) in cols.items():
            gap = [t - k for t, k in zip(totals, kids)]
            pct = [(t - k) / t * 100 if t else None for t, k in zip(totals, kids)]
            print(f"| {key} | {fmt(med(totals))} | {fmt(med(kids))} | {fmt(med(gap))} "
                  f"| {fmt(med(pct), 0)} |")


def perburst(grouped, scenario, bursts, modes):
    stages = ["platform_sync", "redraw", "redraw_prep", "ws_surrogates", "ws_park", "ws_show",
              "ws_start", "redraw_loop", "sync_tail", "sync_focus", "tab_bars", "overview",
              "guards", "backdrop_sync", "border_sync", "dwm_flush", "sg_window", "sg_thumb"]
    print(f"\n### {scenario}: outside-frames ms per burst (median over rounds)\n")
    print("| stage | " + " | ".join(f"`{m}`" for m in modes) + " |")
    print("|" + " --- |" * (len(modes) + 1))

    def runs(mode, value):
        return [value(rs) for (m, _, s), rs in grouped.items() if m == mode and s == scenario]

    def total(stage):
        return lambda rs: sum(r["outside"].get(stage, (0, 0.0))[1] for r in rs) / bursts

    for stage in stages:
        print(f"| {stage} | " + " | ".join(fmt(med(runs(m, total(stage))), 1) for m in modes) + " |")
    rest = lambda rs: (sum(r["outside"].get("platform_sync", (0, 0.0))[1] -
                           r["outside"].get("redraw", (0, 0.0))[1] for r in rs) / bursts)
    print("| platform_sync − redraw | " + " | ".join(fmt(med(runs(m, rest)), 1) for m in modes) + " |")
    tail = lambda rs: (sum(r["outside"].get("sync_tail", (0, 0.0))[1] - sum(
        r["outside"].get(k, (0, 0.0))[1] for k in ["sync_focus", "tab_bars", "overview", "guards",
                                                    "backdrop_sync", "border_sync"])
        for r in rs) / bursts)
    print("| sync_tail − children − backdrop/border_sync | " +
          " | ".join(fmt(med(runs(m, tail)), 1) for m in modes) + " |")
    reports = lambda rs: len(rs)
    print("| reports per run | " + " | ".join(fmt(med(runs(m, reports)), 0) for m in modes) + " |")


def show(grouped, scenario, mode, pick):
    reps = [(round_, r) for (m, round_, s), rs in grouped.items() if m == mode and s == scenario
            for r in rs if "max" in r]
    reps.sort(key=lambda item: item[1]["max"])
    round_, report = reps[-1] if pick == "max" else reps[(len(reps) - 1) // 2]
    print(f"=== {mode} #{round_} {scenario} ({pick} of {len(reps)} reports by interval max: "
          f"{report['max']:.2f} ms)")
    print("\n".join(report["text"]))


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    grouped, command = grouped_reports(sys.argv[1:])
    name, args = command[0], command[1:]
    if name == "split":
        split(grouped, args[0], args[1:])
    elif name == "first":
        first(grouped, args[0], args[1:])
    elif name == "peeknew":
        peeknew(grouped, args[0], args[1:])
    elif name == "gaps":
        gaps(grouped, args[0], args[1:])
    elif name == "perburst":
        perburst(grouped, args[0], int(args[1]), args[2:])
    elif name == "show":
        show(grouped, args[0], args[1], args[2])


if __name__ == "__main__":
    main()
