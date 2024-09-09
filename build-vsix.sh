
cd riscv_analysis_lsp &&
wasm-pack build --target nodejs &&
cd .. &&
rm -rf lsp/server/pkg &&
cp -r riscv_analysis_lsp/pkg lsp/server &&
cd lsp &&
vsce package &&
cd ..
