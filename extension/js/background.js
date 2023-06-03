// Use static import
import initWasmModule, { hello_background } from './wasm/wasm_mod.js';


(async () => {
    await initWasmModule();
    hello_background();
})();