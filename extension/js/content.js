const WASM_MOD_URL = chrome.runtime.getURL('js/wasm/wasm_mod.js');


// Import Wasm module binding using dynamic import
// "init" may fail if the current site CSP restricts the use of Wasm (e.g. any github.com page)
// In this case instantiate module in the background worker (see background.js) and use message passing
const loadWasmModule = async () => {
    const { default: init } = await import(WASM_MOD_URL);

    return init().catch(() => null);
};


(async () => {
    const mod = await loadWasmModule();

    // If the module is successfully initialized,
    // import entities from the module
    if (mod) {
        const { hello_content } = mod;

        hello_content();
    }
})();