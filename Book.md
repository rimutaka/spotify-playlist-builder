# How to build a cross-browser extension with Rust and WASM

This document explains the inner workings of this extension for Rust programmers interested in building extensions with WASM.

What the extension does and how to use it is described in [README.md](./README.md).

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
  * [Service Workers API](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API)
  * [Fetch API](https://developer.mozilla.org/en-US/docs/Web/API/Fetch_API)
* https://rustwasm.github.io/wasm-bindgen/api/js_sys/index.html - a Rust wrapper around JS types and objects



## Architecture of the extension

### Folders

* _extension_: the code of the extension as it is loaded into the browser
* _media_: various media files for publishing the extension, not essential
* _wasm_mod_: Rust code to be compiled into WASM 
* _wasm_mod/samples_: Spotify request/response captures, not essential


### Manifest V3

There are multiple manifest files for different browsers because of incompatible features:

* _extension/manifest_cr.json_: Chrome version
* _extension/manifest_ff.json_: Firefox version

They are renamed into _manifest.json_ by _build.sh_ as explained in _Debugging_ and _Packaging_ sections of this document.

List of manifest properties to pay attention to:

* _action/show_matches_ - when the browser should make the extension button active
* _action/default_popup_ - what should happen when the user clicks on the extension button
* _background/service_worker_ - the name of the script to run as a background service worker
* _content_security_policy_ - declares what the extension can do, e.g. load scripts or WASM

Other properties are either self-explanatory or not as important. This extension can function with the current level of V3 support.

Manifest V3 docs: [MDN](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/manifest.json) / [Chrome](https://developer.chrome.com/docs/extensions/reference)

### Background script

[extension/js/background.js](extension/js/background.js) acts as a Service Worker. The name "background script" is historical and can be used interchangeably with "service worker".

What `background.js` does:

* loads and initializes WASM module
* listens to messages from WASM and the popup page ([extension/js/popup.html](extension/js/popup.html))
* sends error messages to the popup page ([extension/js/popup.js](extension/js/popup.js))
* captures session token (`captureSessionToken()`)
* fetches user details (`fetchUserDetails()`)
* extracts the current playlist ID from the active tab URL (`getPlaylistIdFromCurrentTabUrl()`)
* calls WASM functions in response to user actions (`add_random_tracks(...)`)
* controls the extension icon in the toolbar (`toggleToolbarBadge()`)

Service Workers have their own [lifecycle](https://developer.chrome.com/docs/extensions/develop/concepts/service-workers/lifecycle), separate from any document lifecycle.

Once running, a Service Worker continues to run independently of the page it was started from. A service worker will continue running even if the user navigates elsewhere or closes the tab that started it.

`background.js` is loaded when the browser is started and performs:
* WASM module initialisation
* adding listeners for messaging and token capture

More on service workers: [MDN](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers) / [Chrome](https://developer.chrome.com/docs/extensions/develop/concepts/service-workers)

### Popup page

The manifest file instructs the browser to open a popup when the extension is activated by the user, e.g. with a click on the extension icon in the toolbar.
```
"default_popup": "js/popup.html"
```
Any inline JS in _popup.html_ is ignored. Popup scripts have to be loaded externally with `<script type="module" src="popup.js">` because only _src_ scripts are allowed by `content_security_policy/extension_pages` entry of the manifest.

The popup lives only while it is being displayed. It cannot host any long-running processes and it does not retain the state from one activation to the other. All `DOMContentLoaded` events fire every time it is activated.

What `popup.js` does:

* attaches event handlers to its buttons and links
* handles _on-click_ for links because browsers do not open link URLs from popups (see `chrome.tabs.create()`)
* listens to messages from `background.js` and WASM (see `chrome.runtime.onMessage.addListener()`)
* displays an activity log from the messages it receives

### Content script

Content scripts interact with the UI and DOM. They run in the same context as the page and have access to [Window](https://developer.mozilla.org/en-US/docs/Web/API/Window) object.

This extension does not interact with the Spotify tab and does not need content scripts.

## Messaging between the scripts

Communication between different scripts (background, popup, WASM) is done via asynchronous message passing. A script A sends a message to a shared pool hoping that the intended recipient is listening and can understand the message. This means that if there are multiple listeners, all of them get the message notification. The sender does not get notifications for the messages it sends out.

It is possible to invoke functions from a different script, but they run in the context of the caller. For example, `popup.js` can call a `background.js` function when the user clicks on a button inside the popup. The invocation will work in the context of the popup and die as soon as the popup is closed.

`background.js` is a long-running process. It listens to messages sent to it from other scripts. E.g. `chrome.runtime.onMessage.addListener()` function checks the message payload and acts depending on the message contents.

Key messaging concepts:

* messages are objects
* the only metadata attached to the message is the sender's details
* structure the message to include any additional metadata, e.g. the type of the payload
* always catch `chrome.runtime.sendMessage()` errors because there is no guarantee of delivery
* an error is raised if you send a message and there is no active listener
* message senders cannot receive their own messages, so if you send and listen within the same script, there will be no message notification to self, only to other listeners

More on message passing between different scripts: https://developer.chrome.com/docs/extensions/develop/concepts/messaging.

### Messaging examples

#### From background.js to popup.js

`background.js` sends error messages out to anyone who listens.

`popup.js` listens and displays them if `popup.html` is open. If the popup is not open, there is no listener and the sender gets an error. These errors should be handled for the rest of the sender's script to work.

```
chrome.runtime.sendMessage("Already running. Restart the browser if stuck on this message.").then(onSuccess, onError);
```
where both `onSuccess` and `onError` do nothing, but have to be there for the script to work in either case.

#### From popup.js to background.js

`popup.js` sends a message out when the user clicks _Add tracks_ button.

`background.js` listens and invokes WASM to do the work.

The popup script could call a function in the background script to invoke WASM or even call WASM directly, but the popup lives only if open. Also, it wouldn't have access to the token stored in the context of the long-running `background.js` process.

So, we have a long-running background script that lives independently of the tabs or the popup. When a message from the popup arrives, _background.js_ calls WASM and continues running even if the caller no longer exists.

#### Sending messages from WASM to JS scripts

WASM sends out messages via `report_progress()` function located in `wasm_mod/src/progress.js` script. That function is imported into `wasm_mod/src/lib.rs` as

```
#[wasm_bindgen(module = "/src/progress.js")]
extern "C" {
    pub fn report_progress(msg: &str);
}
```
and is called from other Rust functions as a native Rust function.

The progress reporting from WASM to the popup delivers messages in near-real-time while the WASM process continues running.

## WASM

wasm_mod folder contains the WASM part of the extension.

### Inside Cargo.toml

`crate-type = ["cdylib", "rlib"]`

* _cdylib_: used to create a dynamic system library, e.g. .so or .dll, but for WebAssembly target it creates a *.wasm file without a start function
* _rlib_: optional, used to create an intermediate "Rust library" for unit testing with `wasm-pack test`

More on targets: https://doc.rust-lang.org/reference/linkage.html

### Dependencies

* _wasm-bindgen_: contains the runtime support for `#[wasm_bindgen]` attribute, _JsValue_ interface and other JS bindings ([crate docs](https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/index.html))
* _js-sys_: bindings to JS types, e.g. Array, Date and Promise ([crate docs](https://rustwasm.github.io/wasm-bindgen/api/js_sys/index.html))
* _web-sys_: raw API bindings for Web APIs, 1:1, e.g. the browser Window object is web_sys::Window with all the methods, properties and events available from JS ([crate docs](https://rustwasm.github.io/wasm-bindgen/api/web_sys/index.html))

Remember to add WebAPI classes and interfaces as _web-sys_ features in Cargo.toml. E.g. if you want to use _Window_ class in Rust code it has to be added to _web-sys_ features first.

More about how Rust binds to browser and JS APIs: https://rustwasm.github.io/wasm-pack/book/tutorials/npm-browser-packages/template-deep-dive/cargo-toml.html

### Calling WASM from JS example

_background.js_ makes a call to `hello_wasm()` from _lib.rs_ that logs a greeting in the browser console for demo purposes.

#### `hello_wasm()` sequence diagram

![hello_wasm() call stack](media/book/wasm-invocation.png)

#### The sequence of calls in detail

_lib.rs_ imports WebAPI `console.log()` function and makes it available in Rust:
```
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=console)]
    fn log(s: &str);
}
```

_lib.rs_ also exports `hello_wasm()` Rust function and makes it available to `background.js`:
```
#[wasm_bindgen]
pub fn hello_wasm() {
    log("Hello from WASM!");
}
```

_wasm-bindgen_ and _wasm-pack_ generate _wasm_mod.js_ which:

* initializes the WASM module
* exports `wasm.hello_wasm()` function for _background.js_
* creates a proxy for `console.log()` as 
```
imports.wbg.__wbg_log_1d3ae0273d8f4f8a = function() { return logError(function (arg0) {
    console.log(getObject(arg0));
}, arguments) };
```

Every time the browser activates the extension, it runs the following code from _background.js_:

```
import initWasmModule, { hello_wasm } from './wasm/wasm_mod.js';
(async () => {
    await initWasmModule();
    hello_wasm(); 
})();
```





### lib.rs

_lib.rs_ is the top-level file for the WASM module. It exposes Rust functions and binds to JS interfaces.

The main entry point that does the work of adding tracks to the current playlist is `pub async fn add_random_tracks(...)`. It obtains a reference to the browser runtime (`get_runtime()`) and passes it to other functions.

`async fn get_runtime()` attempts to get a reference to `WorkerGlobalScope` first. If that fails it tries to get a reference to `Window` object. One works in Chrome and the other in Firefox.

The difference between the two runtimes is that `WorkerGlobalScope` is only available in Service Workers and `Window` in content scripts. Firefox uses `Window` for both.

`pub fn report_progress(msg: &str)` is a proxy for `report_progress()` in _progress.js_.
It is called from various locations in Rust code to send progress and error messages to `popup.js`.

### other .rs files

* _client.rs_ - a single high-level function that brings everything together
* _api_wrappers.rs_ - a collection of wrappers for Spotify API, called from client.rs
* _constants.rs_ - shared constants, utility functions
* _models.rs_ - Rust structures for Spotify requests and responses

## Token capture

Spotify maintains an active session when the user is logged in and a *.spotify.com page is open. This extension captures all request headers sent to `https://api-partner.spotify.com/pathfinder/v1/query` endpoint. The headers are stored in local variables and are copied into requests made by the extension to mimic the Spotify app.



`captureSessionToken()` function from _background.js_ does the header extraction when _onBeforeSednHeaders_ event is triggered:

```
chrome.webRequest.onBeforeSednHeaders.addListener(captureSessionToken, { urls: ['https://api-partner.spotify.com/pathfinder/v1/query*'] }, ["requestHeaders"])
```

* _onBeforeSendHeaders_ listener accesses, but does not modify the headers on `https://api-partner.spotify.com/pathfinder/v1/query*`. The `*` at the end of the URL is necessary for the pattern to work.
* `["requestHeaders"]` param instructs the browser to include all the headers in the request details object passed onto _captureSessionToken_ handler. Only the URL and a few common headers are included If that param is omitted.
* `host_permissions": ["*://*.spotify.com/*"]` in _manifest.json_ is required for _onBeforeSendHeaders_ to work.

All extracted headers are stored in `headers` variable inside _background.js_. The tokens are stored in `auth` and `token` vars. None of the headers or tokens are persisted in storage.

The tokens are passed onto WASM:
```
#[wasm_bindgen]
pub async fn add_random_tracks(
    auth_header_value: &str,
    token_header_value: &str,
    playlist_id: &str,
    user_uri: &str,
)
```

## Debugging

Extensions can be loaded from source code in Firefox and Chrome for testing and debugging.

### Firefox

1. run `. build.sh` to build the WASM code
2. go to _about:debugging#/runtime/this-firefox_ or get there the long way:
   * click on _extensions_ icon in the toolbar
   * click on _Manage extensions_
   * click on _settings_ gear-like icon at the top right of the page
   * click on _Debug Add-ons_
3. click on _Load temporary Add-on_ button
4. Firefox opens a file selector popup asking to select _manifest.json_ file. Remember to rename _manifest_ff.json_ into _manifest.json_.

If the extension was loaded correctly, its details should appear inside the _Temporary Extensions_ section on the same page.

### Chrome

The process is very similar to the one in Firefox.

1. run `. build.sh` to build the WASM code
2. go to _chrome://extensions/_ or get there the long way:
   * click on `...` to open Chrome options icon in the toolbar
   * click on _Extensions / Managed Extensions_
   * Chrome will open chrome://extensions/ page
3. Turn on _Developer mode_ toggle at the top-right corner of the page
4. Cick on _Load unpacked_ and select the folder with _manifest.json_. Remember to rename _manifest_cr.json_ into _manifest.json_.

If the extension was loaded correctly, its details should appear in the list of extensions on the same page. See [Chrome docs](https://developer.chrome.com/docs/extensions/get-started/tutorial/hello-world#load-unpacked) for more info.

### Making changes and reloading

Changes to code located inside _wasm_mod/src_ folder require running _build.sh_ script to rebuild the WASN code. 

Changes to files inside _extension_ folder do not require a new build and are picked up by the browser when you reload the extension.

Click on _Reload_ icon for the extension to load the latest changes.

### Viewing logs

Content scripts output log messages (e.g. via `console.log()`) to the same log as the web page. View them in _DevTools / Console_ tab.

Background scripts, including WASM, send messages to a separate console log. 
* _Chrome_: click on _Inspect views_ link in the extension details panel (e.g. _Inspect views service worker (Inactive)_)
![chrome devtools](media/book/chrome-devtools.png)

* _Firefox_: click on _Inspect_ button in the extension details panel
![firefox devtools](media/book/firefox-devtools.png)

If you follow the steps above, Firefox should open a new DevTools window with Console, Network and other tabs.

Popup windows (e.g. _popup.html_) in Firefox log messages into the same console as the background scripts. 

In Chrome, you need to right-click on the open popup and select _Inspect_. It will open a new DevTools window with both background and popup logs. Closing the popup window kills the DevTools window as well.

![popup devtools](media/book/chrome-devtools-popup.png)

Network requests and responses from _background.js_ and WASM appear in the background DevTools window.

## Cross-browser compatibility

Most of the extension code works in both, Firefox and Chrome. There are a few small differences that have to be kept separate: _manifest_, _global context_ and _host_permissions_.

### manifest.json

`background.js` lives under different manifest property names:
* Firefox: _background/scripts_
* Chrome: _background/service_worker_

Firefox complains about `minimum_chrome_version`, `offline_enabled` and `show_matches`.

Chrome rejects `browser_specific_settings`.

See the full list of manifest differences between Firefox and Chrome with this CLI command:
```
git diff --no-index --word-diff extension/manifest_cr.json extension/manifest_ff.json
```

This project is configured to have two manifests in separate files: _manifest_cr.json_ and _manifest_ff.json_. Rename one of them manually into _manifest.json_ for local debugging and then revert to the browser-specific file when finished. _build.sh_ script renames them on the fly during packaging.

### Global context

Firefox and Chrome use different global context class names to access methods such as `fetch()`:
* `WorkerGlobalScope`
* `Window`

Both classes are part of the standardized WebAPI. The difference is in how they are used in content and background scripts.

__Chrome__

* `web_sys::window()` function returns _Some(Window)_ for content scripts and _None_ for Service Workers
* `js_sys::global().dyn_into::<WorkerGlobalScope>()` returns _Ok(WorkerGlobalScope)_ for Service Workers and _Err_ for content scripts

__Firefox__

* `web_sys::window()` function returns _Some(Window)_ for both, context and Service Worker scripts
* `js_sys::global().dyn_into::<WorkerGlobalScope>()` always returns _None_


See `get_runtime()` function in [lib.rs](wasm_mod/src/lib.rs) for more implementation details. Also, see [Window](https://developer.mozilla.org/en-US/docs/Web/API/Window) and [WorkerGlobalScope](https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope) docs on MDN.

### Host permissions

Both Chrome and Firefox have this entry in the manifest:
```json
"host_permissions": [
    "*://*.spotify.com/*"
]
```

The format is the same, but Chrome grants this permission on install and Firefox treats it as an optional permission that has to be requested at runtime.

The extension code handles this discrepancy gracefully at the cost of some complexity. More info:

* the implementation of `btn_add` _click_ listener in [popup.js](extension/js/popup.js) has detailed comments
* MDN docs: https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/API/permissions/request
* Firefox strange behavior: https://stackoverflow.com/questions/47723297/firefox-extension-api-permissions-request-may-only-be-called-from-a-user-input


## Listing WASM extensions in Google and Mozilla addon stores

There are no special requirements for extensions containing WASM code.

Both Google and Mozilla may request the source code and build instructions for the WASM part, which may delay the review process. This extension is 100% open source and was always approved within 24hrs.

### Packaging

_build.sh_ script packages the extension into _chrome.zip_ and _firefox.zip_ files. The packaging steps include:

* building the WASM code with _wasm-pack_
* removing unnecessary files
* renaming browser-specific manifest files to _manifest.json_
* zipping up _extension_ folder into _chrome.zip_ and _firefox.zip_ with the right manifest

_wasm-pack_ copies .js files from _wasm_mod/src_ folder to _extension/js/wasm/snippets/wasm_mod/src_  if they are used for binding to Rust.

See detailed packaging instructions for [Firefox](https://extensionworkshop.com/documentation/publish/package-your-extension/) and [Chrome](https://developer.chrome.com/docs/extensions/how-to/distribute) for more info.

### Listing

The listing has no WASM-specific questions or options. Make sure you do not have any unused permissions in the manifest and give clear explanations why you need the rest.

![chrome permissions](media/book/chrome-permissions.png)

#### Useful links

_Firefox_

* Example: https://addons.mozilla.org/en-US/firefox/addon/spotify-playlist-builder-addon/
* Sign up here: https://addons.mozilla.org/en-US/developers/
* List your extensions: https://addons.mozilla.org/en-US/developers/addons
* Uploading listing images may only be available after the extension is approved.

_Chrome_

* Example: https://chromewebstore.google.com/detail/spotify-playlist-builder/kmbnbjbfpnchgmmkbeidpllpamcahljn
* Sign up and list your extensions: https://chrome.google.com/webstore/devconsole

#### Listing process

* Sign up, create a new listing, describe the extension and upload the _.zip_ for review
* Check any errors and warnings generated by the store website after uploading the _.zip_
* Approval time for both stores is about 24 hours, from personal experience.
* Any change in the code or the listing requires a store review before it is published

Both stores allow listing extensions without them appearing in the public directory while users with the extension URL can still access it.


