#!/usr/bin/env python3
"""Summarizes `scene-bench.ps1` results as a markdown table.

Usage: python scene-bench-summary.py [results dir]
"""
import collections
import glob
import os
import re
import statistics
import sys

RESULTS = sys.argv[1] if len(sys.argv) > 1 else os.path.expanduser(
    '~/gw-bench/results_lite')
METRICS = {
    'busy p50': 'dwm gpu busy p50 ms',
    'busy p90': 'dwm gpu busy p90 ms',
    'missed/burst': 'dwm missed vblanks/burst',
    'frames/burst': 'dwm frames/burst',
}


def metric(text, label):
    for line in text.splitlines():
        line = line.strip()
        if line.startswith(label):
            numbers = re.findall(r'-?\d+\.?\d*', line[len(label):])
            if numbers:
                return float(numbers[0])
    return None


def phases(text):
    """Median GPU busy of the first, middle and last third of each burst."""
    bursts = collections.defaultdict(list)
    for match in re.finditer(r'^FRAME\s+(\d+)\s+([\d.]+)\s+([\d.]+)', text, re.M):
        bursts[match.group(1)].append((float(match.group(2)), float(match.group(3))))
    parts = ([], [], [])
    for frames in bursts.values():
        frames.sort()
        third = len(frames) // 3
        if third == 0:
            continue
        parts[0].extend(busy for _, busy in frames[:third])
        parts[1].extend(busy for _, busy in frames[third:-third])
        parts[2].extend(busy for _, busy in frames[-third:])
    return [statistics.median(part) if part else float('nan') for part in parts]


rows = collections.OrderedDict()
for path in sorted(glob.glob(os.path.join(RESULTS, '*', '*', '*.txt'))):
    build, config = path.split(os.sep)[-3:-1]
    cell, _, _ = os.path.basename(path)[:-4].rpartition('_')
    text = open(path, encoding='utf-8', errors='replace').read()
    row = rows.setdefault((build, config, cell), {'text': '', **{k: [] for k in METRICS}})
    row['text'] += '\n' + text
    for key, label in METRICS.items():
        value = metric(text, label)
        if value is not None:
            row[key].append(value)

mean = lambda values: sum(values) / len(values) if values else float('nan')
header = ['build', 'config', 'cell', *METRICS, 'first', 'middle', 'last', 'rounds p50']
print('| ' + ' | '.join(header) + ' |')
print('|' + '---|' * len(header))
for (build, config, cell), row in rows.items():
    first, middle, last = phases(row['text'])
    rounds = '/'.join(f'{v:.1f}' for v in row['busy p50'])
    print(f"| {build} | {config} | {cell} | {mean(row['busy p50']):.1f} | "
          f"{mean(row['busy p90']):.1f} | {mean(row['missed/burst']):.0f} | "
          f"{mean(row['frames/burst']):.0f} | {first:.1f} | {middle:.1f} | "
          f"{last:.1f} | {rounds} |")
