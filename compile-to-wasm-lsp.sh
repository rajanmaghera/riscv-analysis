

wasm-pack build --target nodejs
rm -rf ~/Documents/GitHub/vscode-extension-samples/lsp-sample/server/pkg
cp -r pkg ~/Documents/GitHub/vscode-extension-samples/lsp-sample/server

