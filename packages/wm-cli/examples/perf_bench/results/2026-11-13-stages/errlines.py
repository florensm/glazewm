"""errors.log warnings per mode, from a restart to the next.

Usage: errlines.py <date> <errors.log slice> <run log>...

Each line is put in the mode whose restart it follows (until the run log's
last line), with its numbers and ids masked so alike lines count together.
"""

import datetime
import re
import sys
from collections import Counter, defaultdict

from reports import LINE, ROUND, parse_time

STAMP = re.compile(r"^(\S+Z)\s+(WARN|ERROR)\s+(\S+): (.*)$")


def periods(date, path):
    events = []
    with open(path, encoding="utf-8-sig", errors="replace") as file:
        for line in file:
            if m := LINE.match(line.rstrip()):
                events.append((parse_time(date, m.group(1)), m.group(2)))
    starts = [(t, m.group(2)) for t, text in events if (m := ROUND.search(text))]
    end = events[-1][0] if events else None
    return [(t, starts[i + 1][0] if i + 1 < len(starts) else end, mode)
            for i, (t, mode) in enumerate(starts)]


def mask(text):
    text = re.sub(r"[0-9a-f]{8}-[0-9a-f-]{27}", "<id>", text)
    text = re.sub(r"0x[0-9a-fA-F]+", "<hwnd>", text)
    return re.sub(r"\d+(\.\d+)?", "#", text)


def main():
    sys.stdout.reconfigure(encoding="utf-8")
    date = datetime.date.fromisoformat(sys.argv[1])
    spans = [p for log in sys.argv[3:] for p in periods(date, log)]
    counts = defaultdict(Counter)
    with open(sys.argv[2], encoding="utf-8", errors="replace") as file:
        for raw in file:
            line = re.sub(r"\x1b\[[0-9;]*m", "", raw.rstrip("\n"))
            if not (m := STAMP.match(line)):
                continue
            time = datetime.datetime.fromisoformat(m.group(1).replace("Z", "+00:00"))
            mode = next((mode for start, end, mode in spans if start <= time <= end), None)
            if mode:
                counts[mode][f"{m.group(2)} {m.group(3)}: {mask(m.group(4))[:110]}"] += 1
    for mode, counter in counts.items():
        print(f"\n### `{mode}`\n")
        for text, n in counter.most_common():
            print(f"- {n}× {text}")


if __name__ == "__main__":
    main()
