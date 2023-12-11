## --release or --dev - exclude/include debug info
## --no-typescript - disable .d.ts files output
## --out-dir - where to write the compiled files
## --out-name - force output file names
## --target - always use "web"!
## See https://rustwasm.github.io/wasm-pack/book/commands/build.html
echo Building wasm module...
wasm-pack build wasm_mod --dev --no-typescript --out-dir "../extension/js/wasm" --out-name "wasm_mod" --target web

## wasm-pack creates bunch of useless files:
echo Removing trash files...
rm -f extension/js/wasm/.gitignore
rm -f extension/js/wasm/package.json

## create chrome package and exclude manifest for firefox
## see ReadMe for more info on manifest config
## subshell call with cd is required to avoid placing /extension/ folder as the root
rm -f chrome.zip && \
(cd extension && zip -rq ../chrome.zip . -x manifest_ff.json -x manifest.json) && \
printf "@ manifest_cr.json\n@=manifest.json\n" | zipnote -w chrome.zip && \
echo Chrome package: chrome.zip

## create firefox package, exclude chrome manifest and rename FF manifest to its default file name
rm -f firefox.zip && \
(cd extension && zip -rq ../firefox.zip . -x manifest_cr.json -x manifest.json) && \
printf "@ manifest_ff.json\n@=manifest.json\n" | zipnote -w firefox.zip && \
echo Firefox package: firefox.zip