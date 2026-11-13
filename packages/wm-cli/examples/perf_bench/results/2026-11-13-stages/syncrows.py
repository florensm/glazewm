"""`[sync]` rows of `rd_apply by process` per mode: in how many reports, and their worst."""
import statistics, sys
sys.argv = sys.argv[:1] + sys.argv[1:]
import reports as R
import datetime
date = datetime.date.fromisoformat(sys.argv[1]); perf = sys.argv[2]; logs = sys.argv[3:-1]; scenario = sys.argv[-1]
grouped = R.assign(R.read_reports(perf), [w for log in logs for w in R.read_windows(date, log)])
modes = list(dict.fromkeys(k[0] for k in grouped))
print(f"| mode | reports | with sync rows | sync rows | worst per report (median of those with any) | worst | by process (rows, worst ms) | interval max median | interval max worst |")
print("| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: |")
for mode in modes:
    reps = [r for k, rs in grouped.items() if k[0] == mode and k[2] == scenario for r in rs]
    syncs = [[p for p in r["procs"] if p[1] == "sync"] for r in reps]
    with_ = [s for s in syncs if s]
    worst = [max(p[2] for p in s) for s in with_]
    by = {}
    for s in syncs:
        for name, _, w in s:
            n, m = by.get(name, (0, 0.0)); by[name] = (n + 1, max(m, w))
    bytext = ", ".join(f"{k} {n}x/{m:.0f}" for k, (n, m) in sorted(by.items(), key=lambda kv: -kv[1][1]))
    maxes = [r["max"] for r in reps if "max" in r]
    print(f"| {mode} | {len(reps)} | {len(with_)} | {sum(len(s) for s in syncs)} | "
          f"{statistics.median(worst) if worst else 0:.1f} | {max(worst) if worst else 0:.1f} | {bytext} | "
          f"{statistics.median(maxes):.1f} | {max(maxes):.1f} |")
