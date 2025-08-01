#!/usr/bin/env bash

# Get the directory of the currently executing script
SCRIPT_DIR=$(dirname "$(readlink -f "$0")" )
pushd $SCRIPT_DIR

# For every test case that exists
for item in "$SCRIPT_DIR/riscv_analysis_cli/resources/test"/*; do
  cargo run -- lint --yaml --no-output "$item/code.s" > "$item/raw.yaml"
done

popd