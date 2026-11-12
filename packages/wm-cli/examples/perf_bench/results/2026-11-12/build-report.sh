#!/usr/bin/env bash
# Assembles ~/.glzr/glazewm/bench-report.md from the result files.
set -euo pipefail
cd "$(dirname "$0")"
out="$HOME/.glzr/glazewm/bench-report.md"
end="${1:-$(date +%H:%M)}"
r=results

{
  sed "s/{{END}}/$end/" report-head.md
  echo
  echo "# Results: the requested runs"
  echo
  echo "Tryout modes from \`isolate_modes.ps1\`, then \`main\` in its own block."
  echo
  python summarize.py "No CPU load" $r/load0-main.txt $r/load0-isolate.txt
  echo
  python tails.py "No CPU load" $r/log-load0-main.txt $r/log-load0-isolate.txt
  echo
  python summarize.py "With CPU load (-CpuLoad 8)" $r/cpu8-main.txt $r/cpu8-isolate.txt
  echo
  python tails.py "With CPU load" $r/log-cpu8-main.txt $r/log-cpu8-isolate.txt
  if [ -f $r/fixed-load0.txt ]; then
    echo
    cat fixed-intro.md
    for load in load0 cpu8; do
      [ -f $r/fixed-$load.txt ] || continue
      title="Fixed tile order, no CPU load"
      [ $load = cpu8 ] && title="Fixed tile order, with CPU load (-CpuLoad 8)"
      echo
      python summarize.py "$title" $r/fixed-$load.txt
      if [ -f $r/log-fixed-$load.txt ]; then
        echo
        python tails.py "$title" $r/log-fixed-$load.txt
      fi
    done
  fi
  echo
  cat report-verdict.md
  if [ -f $r/parity-load0.txt ]; then
    echo
    cat parity-intro.md
    echo
    python summarize.py "Restart-order run, no CPU load" $r/parity-load0.txt
  fi
  echo
  echo "# Raw results"
  for f in "load0-isolate|isolate-bench.txt, no CPU load (21:05)" \
           "load0-main|main, no CPU load (bench-run.ps1)" \
           "cpu8-isolate|isolate-bench.txt, -CpuLoad 8 (21:19)" \
           "cpu8-main|main, -CpuLoad 8 (bench-run.ps1)" \
           "fixed-load0|Fixed tile order, no CPU load (bench-run.ps1)" \
           "fixed-cpu8|Fixed tile order, -CpuLoad 8 (bench-run.ps1)" \
           "parity-load0|Restart-order run, no CPU load (bench-run.ps1)"; do
    file="$r/${f%%|*}.txt"
    [ -f "$file" ] || continue
    echo
    echo "## ${f#*|}"
    echo
    echo '```text'
    sed 's/\r$//' "$file"
    echo '```'
  done
} > "$out"
echo "Wrote $out ($(wc -l < "$out") lines)"
