@echo off

:: --release - use release profile build
:: --no-typescript - disable .d.ts files output
:: --out-dir - set output directory
:: --out-name - force output file names
:: --target - always use "web"!
:: See https://rustwasm.github.io/wasm-pack/book/commands/build.html
echo Building wasm module...
wasm-pack build --release --no-typescript --out-dir "../extension/js/wasm" --out-name "wasm_mod" --target web

:: wasm-pack creates bunch of useless files:
:: - Output of typescript files disabled by --no-typescript wasm-pack argument
:: - We should delete the .gitignore and package.json files ourselves
echo Removing trash files...
if exist "..\extension\js\wasm\.gitignore" del "..\extension\js\wasm\.gitignore"
if exist "..\extension\js\wasm\package.json" del "..\extension\js\wasm\package.json"

echo Done
pause