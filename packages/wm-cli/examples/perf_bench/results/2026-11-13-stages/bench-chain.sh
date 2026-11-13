#!/usr/bin/env bash
# Runs the 2026-11-13 stages bench: resize and switch on this branch, then
# main as the reference, at load 0 then CPU 8. Each run's console log is
# timestamped; its slice of perf.log is copied out by byte offset (Zed holds
# ~/.glzr/glazewm's files open, so they are neither moved nor written by
# PowerShell).
set -u
S="$(cd "$(dirname "$0")" && pwd)"
REPO=/c/Users/googlers/glazewm
PB=packages/wm-cli/examples/perf_bench
OUT="${OUT:?set OUT to a folder outside ~/.glzr/glazewm}"
DATA=~/.glzr/glazewm
LOGS=$DATA/bench-2026-11-13-stages/logs
ORDER=Notepad,WindowsTerminal,WindowsTerminal,chrome,chrome,FPilot
RESIZE_MODES=nopeeks,nopeeks+nokeepoverlays,default,noasynchandback
SWITCH_MODES=nopeeks,default,noslidepeeks
mkdir -p "$OUT" "$LOGS"

ts() { while IFS= read -r line; do printf '%s %s\n' "$(date +%T)" "$line"; done; }
size() { stat -c %s "$1" 2>/dev/null || echo 0; }

# Notes the time and the sizes of errors.log and perf.log.
mark() {
  echo "$1 $(date +%T) errors.log $(size $DATA/errors.log) perf.log $(size $DATA/perf.log)" >> "$OUT/marks.txt"
}

# Copies the part of perf.log written since `$1` bytes to perf-<name>.log.
slice() {
  local from=$1 name=$2
  tail -c +$((from + 1)) "$DATA/perf.log" > "$LOGS/perf-$name.log"
}

iso() {
  local name=$1; shift
  local from; from=$(size $DATA/perf.log)
  mark "start $name"
  (cd "$REPO" && powershell -NoProfile -ExecutionPolicy Bypass -File "$PB/isolate_modes.ps1" \
    -Target chrome -Rounds 4 -Bursts 10 -TileOrder "$ORDER" \
    -ResultsFile "$(cygpath -w "$OUT/$name.txt")" "$@" 2>&1) | ts > "$OUT/log-$name.txt"
  mark "end $name"
  slice "$from" "$name"
}

main_run() {
  local name=$1; shift
  local from; from=$(size $DATA/perf.log)
  mark "start $name"
  (cd "$S" && powershell -NoProfile -ExecutionPolicy Bypass -File bench-run.ps1 \
    -Target chrome -ResultsFile "$(cygpath -w "$OUT/$name.txt")" -Modes main,nopeeks,default \
    -Scenarios resize,switch -Rounds 4 -Bursts 10 -TileOrder "$ORDER" "$@" 2>&1) | ts > "$OUT/log-$name.txt"
  mark "end $name"
  slice "$from" "$name"
}

# The runs named as arguments, in order; all six without any.
runs=("$@")
[ ${#runs[@]} -eq 0 ] && runs=(resize-load0 switch-load0 main-load0 resize-cpu8 switch-cpu8 main-cpu8)
for run in "${runs[@]}"; do
  case $run in
    resize-load0) iso "$run" -Scenarios resize -Modes "$RESIZE_MODES" ;;
    switch-load0) iso "$run" -Scenarios switch -Modes "$SWITCH_MODES" ;;
    main-load0) main_run "$run" ;;
    resize-cpu8) iso "$run" -Scenarios resize -Modes "$RESIZE_MODES" -CpuLoad 8 ;;
    switch-cpu8) iso "$run" -Scenarios switch -Modes "$SWITCH_MODES" -CpuLoad 8 ;;
    main-cpu8) main_run "$run" -CpuLoad 8 ;;
    *) echo "unknown run $run" >&2; exit 1 ;;
  esac
done
echo "chain done $(date +%T)" >> "$OUT/marks.txt"
