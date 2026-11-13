#!/usr/bin/env bash
# Runs the 2026-11-13 bench: peek words, morepeeks and main, at load 0 then
# CPU 8. Each run's console log is timestamped; its perf.log is moved aside.
set -u
S="$(cd "$(dirname "$0")" && pwd)"
REPO=/c/Users/googlers/glazewm
PB=packages/wm-cli/examples/perf_bench
OUT="$S/bench"
LOGS=~/.glzr/glazewm/bench-2026-11-13/logs
ORDER=Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot
PEEK_MODES=nopeeks,nopeeks+keepoverlays,default,keeppeeks,asynchandback,asynclanding,earlylanding,earlylanding+asynchandback+keeppeeks
MORE_MODES=morepeeks,morepeeks+keeppeeks,morepeeks+earlylanding+asynchandback+keeppeeks
mkdir -p "$OUT" "$LOGS"

ts() { while IFS= read -r line; do printf '%s %s\n' "$(date +%T)" "$line"; done; }

# Archives the live perf.log as perf-<name>.log and notes errors.log's size.
mark() {
  echo "$1 $(date +%T) errors.log $(stat -c %s ~/.glzr/glazewm/errors.log)" >> "$OUT/marks.txt"
}
archive() {
  [ -f ~/.glzr/glazewm/perf.log ] && mv ~/.glzr/glazewm/perf.log "$LOGS/perf-$1.log"
}

iso() {
  local name=$1; shift
  mark "start $name"
  (cd "$REPO" && powershell -NoProfile -ExecutionPolicy Bypass -File "$PB/isolate_modes.ps1" \
    -Target chrome -Rounds 4 -Bursts 10 -TileOrder "$ORDER" -Launch none "$@" 2>&1) | ts > "$OUT/log-$name.txt"
  cp ~/.glzr/glazewm/isolate-bench.txt "$OUT/$name.txt"
  mark "end $name"
  archive "$name"
}

main_run() {
  local name=$1; shift
  mark "start $name"
  (cd "$REPO/$PB/results/2026-11-13" && powershell -NoProfile -ExecutionPolicy Bypass -File bench-run.ps1 \
    -Target chrome -ResultsFile "$(cygpath -w "$OUT/$name.txt")" -Modes main,nopeeks,default \
    -Scenarios move,resize -Rounds 4 -Bursts 10 -TileOrder "$ORDER" "$@" 2>&1) | ts > "$OUT/log-$name.txt"
  mark "end $name"
  archive "$name"
}

iso peek-load0 -Scenarios move,resize -Modes "$PEEK_MODES"
iso more-load0 -Scenarios float,resize -Modes "$MORE_MODES"
main_run main-load0
iso peek-cpu8 -Scenarios move,resize -Modes "$PEEK_MODES" -CpuLoad 8
iso more-cpu8 -Scenarios float,resize -Modes "$MORE_MODES" -CpuLoad 8
main_run main-cpu8 -CpuLoad 8
echo "chain done $(date +%T)" >> "$OUT/marks.txt"
