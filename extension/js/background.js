// Use static import
import * as mod from './wasm/wasm_mod.js';


(async () => {
    // Manually load the Wasm module as ArrayBuffer
    const response = await fetch('./wasm/wasm_mod_bg.wasm');
    const buffer   = await response.arrayBuffer();

    // In addition to the default initialization function that we used in the external script, 
    // the "wasm_mod.js" exports the initSync function, which accepts a Wasm ArrayBuffer
    const { hello_background } = mod.initSync(buffer);

    hello_background();
})();

// wasm-pack build --release --target web