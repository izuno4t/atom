#!/bin/sh
set -eu

if [ "$#" -ne 3 ]; then
  echo "usage: atom-eval-pandoc <input> <output-md> <report-json>" >&2
  exit 64
fi

input="$1"
output="$2"
report="$3"

mkdir -p "$(dirname "$output")" "$(dirname "$report")"
pandoc "$input" -t gfm -o "$output"
bytes="$(wc -c < "$output" | tr -d ' ')"
printf '{"tool":"pandoc","output":"%s","bytes":%s}\n' "$output" "$bytes" > "$report"
printf '{"tool":"pandoc","status":"ok","output":"%s"}\n' "$output"
