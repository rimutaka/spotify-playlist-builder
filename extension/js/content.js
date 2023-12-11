// console.log("Hello from content");

// this makes it universal for chrome/ff
// FF understands `chrome`, so this is not used in other parts of the code
// there are some differences how FF treats `chrome` vs `browser`, see MDN for more
const runtime = chrome.runtime || browser.runtime;

// location of the wasm file, e.g. moz-extension://99df57d7-296d-402f-b722-8aada17fdee1/js/wasm/wasm_mod.js
// FF uses Internal UUID for the URL for debugging, so there is no way to use a literal
// when the extension package is uploaded, Moz marks it as a security warning
// CR uses a URL like this chrome-extension://bdainkgdpcndhjiffodhkonfpoepjenn/js/wasm/wasm_mod.js
// where the ID is generated as well
const WASM_MOD_URL = runtime.getURL('js/wasm/wasm_mod.js');

// Import Wasm module binding using dynamic import
// "init" may fail if the current site CSP restricts the use of Wasm (e.g. any github.com page)
// In this case instantiate module in the background worker (see background.js) and use message passing
const loadWasmModule = async () => {
  const { default: init } = await import(WASM_MOD_URL);

  return init().catch(() => null);
};

// current client creds fetched from Spotify request headers by a b/g script
let authHeaderValue = ''
let tokenHeaderValue = ''

// listens for msgs from the b/g script to update creds
chrome.runtime.onMessage.addListener((request) => {

  // something else may send a message - expect anything
  if (!request) {
    console.log("Blank creds msg payload")
    return
  }

  // only process if the message looks valid
  if (request.authHeaderValue && request.tokenHeaderValue) {
    authHeaderValue = request.authHeaderValue
    tokenHeaderValue = request.tokenHeaderValue
  }
  else {
    console.log("Unexpected creds msg format")
    console.log(request);
  }
});

(async () => {
  const mod = await loadWasmModule();

  // If the module is successfully initialized,
  // import entities from the module
  if (mod) {
    const { hello_content } = mod;

    hello_content();
  }
})();