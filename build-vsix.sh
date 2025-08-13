#!/usr/bin/env bash
set -e

# Get the directory of the currently executing script
SCRIPT_DIR=$(dirname "$(readlink -f "$0")" )
pushd $SCRIPT_DIR

pushd riscv_analysis_lsp
wasm-pack build --target nodejs
popd

rm -rf lsp/server/pkg
cp -r riscv_analysis_lsp/pkg lsp/server

pushd lsp
npm install
vsce package
popd

popd
