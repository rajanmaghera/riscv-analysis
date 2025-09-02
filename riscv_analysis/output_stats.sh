#!/usr/bin/env bash

shopt -s lastpipe

# Hard-coded map: id -> nice name
declare -A BENCH_NAMES=(
  ["sized_group_505/parse_from_string"]="Parse assembly"
  ["sized_group_505/cfg_from_parsed"]="Create CFG structures"
  ["sized_group_505/add_direction"]="Add directions"
  ["sized_group_505/markup_function_info"]="Group into functions"
  ["sized_group_505/add_available_value"]="\\textbf{Run value analysis}"
  ["sized_group_505/add_liveness_information"]="\\textbf{Run register liveness analysis}"
  ["sized_group_505/lint_from_cfg"]="Produce lint reports"
)

echo "\\toprule"
echo "& 505.mcf\\_r \\\\"
echo "\\midrule"

declare -A SUM="0"

jq -r '
  [.id, .typical.estimate]
  | @tsv
' | while IFS=$'\t' read -r id estimate; do

  estimate=${estimate:-0}
  if (( $(echo "$estimate == 0" | bc -l) )); then
      continue
  fi

  millis=$(echo "scale=2; $estimate / 1000000.0" | bc)
  nice_name=${BENCH_NAMES[$id]:-$id}
  echo "$nice_name & ${millis} ms \\\\"
  SUM=$(echo "$SUM + $estimate" | bc)
done
SUM=$(echo "scale=2; $SUM/ 1000000.0" | bc)
echo "\\midrule"
echo "Sum of individual components & $SUM ms \\\\"
echo "Execution time & 536 ms \\\\"
echo "\\bottomrule"
