

wasm-pack build --target nodejs
rm -rf lsp/server/pkg
cp -r pkg lsp/server

