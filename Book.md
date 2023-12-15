# How to build a cross-browser extension with Rust and WASM

What the extension does and how to use it is described in [README.md](./README.md).

This document explains the inner workings of this extension for Rust programmers interested in building extensions with WASM.

What you will learn:

* toolchain 
* architecture of the extension
* how WASM, background and content scripts communicate
* intercepting session tokens to impersonate the user
* debugging
* making it work for Chrome and Firefox
* listing in Google and Mozilla addon stores

This guide expects you to be a proficient Rust and web coder, but not necessarily familiar with how extensions work.

## Toolchain

The main tool to build, test and publish WASM: https://rustwasm.github.io/docs/wasm-pack/
```
cargo install wasm-pack
```
See [build.sh](./build.sh) for an example of [build command](https://rustwasm.github.io/docs/wasm-pack/commands/build.html).

_cargo_ installs [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/) on the first compilation to facilitate high-level interactions between wasm modules and JavaScript.

References:

* https://rustwasm.github.io/wasm-bindgen/ - Rust/WASM book with examples, explanations and details references. Read about:
  * Hello, World!
  * console.log
  * Without a bundler
* https://rustwasm.github.io/wasm-bindgen/api/web_sys/index.html - a Rust wrapper around [Web APIs](https://developer.mozilla.org/en-US/docs/Web/API). APIs of interest:
  * Service Workers API
  * Fetch API
* https://rustwasm.github.io/wasm-bindgen/api/js_sys/index.html - a Rust wrapper around JS types and objects



## Architecture of the extension

### Folders

* extension: the code of the extension as it is loaded into the browser
* media: various media files for publishing the extension, not essential
* wasm_mod: Rust code to be compiled into WASM 
* wasm_mod/samples: Spotify request/response captures, not essential


### Manifest

There are multiple manifest files for different browsers because of incompatible features. They are renamed into _manifest.json_ by _build.sh_ for publishing. See [README.md](./README.md) for details.

* action/show_matches - when the browser should make the extension button active
* action/default_popup - what should happen when the user clicks on the extension button
* background/service_worker - the name of the script to run as a background service worker
* content_security_policy - declares when the extension can do, e.g. load scripts or WASM

The docs on V3 can be confusing. Chrome has a hard-to-navigate object model reference that you have to translate into JSON format in your head to construct the manifest file. 

### Background script

`extension/js/background.js` acts as a Service Worker. The name "background script" is somewhat historical and can be used interchangeably.

What `background.js` does:

* loads and inits WASM
* listens to messages from WASM and the popup page
* sends error messages to the popup page
* captures session token (`captureSessionToken()`)
* fetches user details (`fetchUserDetails()`)
* extracts the current playlist ID from the active tab URL (`getPlaylistIdFromCurrentTabUrl()`)
* calls WASM functions in response to user actions
* controls the extension icon in the toolbar (`toggleToolbarBadge()`)

Service Workers have their own [lifecycle](https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/lifecycle). Once running, service workers continue to run independently of the page they were started from. The service worker will continue running if the user navigates elsewhere or closes the tab.

`background.js` is loaded when the browser is started and performs:
* WASM module initialisation
* adds listeners for messaging and token capture
and goes to sleep.

More on service workers: [MDN](), [Chrome](https://developer.chrome.com/docs/extensions/develop/concepts/service-workers)

### Popup page

The manifest file instructs the browser to open a popup when the extension is activated by the user, e.g. with a click on the extension icon in the toolbar.
```
"default_popup": "js/popup.html"
```
The HTML file loads `<script type="module" src="popup.js">` because only _src_ scripts are allowed by `content_security_policy/extension_pages` (inline scripts are ignored).

The popup lives only while it is being displayed. It cannot host any long-running processes and it does not retain the state from one activation to the other. All `DOMContentLoaded` events fire every time it is activated.

What `popup.js` does:

* attaches event handlers to buttons and links
* handles on-click for links because browsers do not open link URLs from popups (see `chrome.tabs.create()`)
* listens to messages from `background.js` and WASM (see `chrome.runtime.onMessage.addListener()`)
* displays an activity log from the messages it received

### Content script

Content scripts interact with the UI and DOM. They run in the same context as the page and have access to [Window](https://developer.mozilla.org/en-US/docs/Web/API/Window) object.

This extension does not interact with the Spotify tab and has no need for content scripts.

## Messaging between the scripts

Communication between different scripts (background, popup, WASM) is done via Message Passing. It is possible to invoke `background.js` functions from `popup.js`, but they will be running in the context of the caller and die as soon as the popup is closed.

`background.js` is a long-running process. It listens to messages sent to it from other scripts. See `chrome.runtime.onMessage.addListener()` functions for details.
When a message arrives it checks the payload and acts accordingly.

* messages are objects
* the only metadata attached to the message is the sender details
* structure the message to include any additional metadata, e.g. the type of the payload
* always catch `chrome.runtime.sendMessage()` errors because there is no guarantee of delivery
* an error is raised if you send a message and there is no active listener
* message senders cannot receive their own messages, so if you send and listen within the same script, there will be no message notification to self, only to other listeners

More on message passing between different scripts: [MDN](), [Chrome](https://developer.chrome.com/docs/extensions/develop/concepts/messaging).

### Messaging examples

`background.js` sends out error messages, `popup.js` listens and displays them if `popup.html` is open. If the popup is not open, the sender gets an error and ignores it.
```
chrome.runtime.sendMessage("Already running. Restart the browser if stuck on this message.").then(onSuccess, onError);
```
where both `onSuccess` and `onError` do nothing, but have to be there for the workflow to work.

`popup.js` sends a message out when the user clicks _Add tracks_ button, `background.js` listens and invokes WASM to do the work.
Technically, the popup could call a function in the background script to invoke WASM or even call WASM directly, but the popup lives only if open. Also, it wouldn't have access to the token which is stored in the context of `background.js`.

So, we have a long-running b/g script that lives independently of the tabs or the popup. When a message from the popup arrives, it calls WASM and continues running even if the caller no longer exists.

WASM sends out messages via JS `report_progress()` function located in `wasm_mod/src/progress.js` script. That function is imported into `wasm_mod/src/lib.rs` as

```
#[wasm_bindgen(module = "/src/progress.js")]
extern "C" {
    pub fn report_progress(msg: &str);
}
```
and is called from Rust as a native Rust function.

The progress reporting from WASM to the popup delivers messages in near-real-time as WASM.

## WASM

wasm_mod folder contains the WASM part of the extension.

`crate-type = ["cdylib", "rlib"]`

* cdylib: used to create a dynamic system library, e.g. .so or .dll, but for WebAssembly target it creates a *.wasm file without a start function
* rlib: optional, used to create an intermediate "Rust library" for unit testing with `wasm-pack test`

More on targets: https://doc.rust-lang.org/reference/linkage.html

### Dependencies

* wasm-bindgen: contains the runtime support for #[wasm_bindgen] attribute, JsValue interface and other JS bindings ([crate docs](https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/index.html))
* js-sys: bindings to JS types, e.g. Array, Date and Promise ([crate docs](https://rustwasm.github.io/wasm-bindgen/api/js_sys/index.html))
* web-sys: raw API bindings for Web APIs, 1:1, e.g. the browser Window object is web_sys::Window with all the methods, properties and events available from JS ([crate docs](https://rustwasm.github.io/wasm-bindgen/api/web_sys/index.html))

More about how Rust binds to browser and JS APIs: https://rustwasm.github.io/wasm-pack/book/tutorials/npm-browser-packages/template-deep-dive/cargo-toml.html

### lib.rs

This is the entry point for the library.

This snippet makes JS `console.log` available in Rust:
```
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=console)]
    fn log(s: &str);
}
```

This snippet makes a Rust function available to `background.js`:
```
#[wasm_bindgen]
pub fn hello_wasm() {
    log("Hello from WASM!");
}
```

The main entry point that does the work of adding tracks to the current playlist is `pub async fn add_random_tracks(...)`. It obtains a reference to the browser runtime and passes it to other functions.

`async fn get_runtime()` attempts to get a reference to `WorkerGlobalScope` first. If that fails it tries to get a reference to `Window` object. One works in Chrome and the other in Firefox.

The difference between the two runtimes is that `WorkerGlobalScope` is only available in Service Workers and `Window` in content scripts. Firefox uses `Window` for both.

`pub fn report_progress(msg: &str)` is a proxy for `report_progress()` in _progress.js_.
It is called from various locations in Rust code to send progress and error messages to `popup.js`.

### other .rs files

* client.rs - a single high-level function that brings everything together
* api_wrappers.rs - a collection of wrappers for Spotify API, called from client.rs
* constants.rs - shared constants, utility functions
* models.rs - Rust structures for Spotify requests and responses 