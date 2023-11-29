// Copied this solution from https://medium.com/geekculture/rusting-javascript-with-webassembly-632405ba5a42
// The proper way of doing this is probably via https://developer.mozilla.org/en-US/docs/Web/API/Channel_Messaging_API
// also see https://developer.chrome.com/docs/extensions/mv3/messaging/

// from content to popup: https://www.reddit.com/r/chrome_extensions/comments/sjfl02/chrome_extension_v3_sending_data_from_content_to/

export function report_progress(msg) {
  console.log(`Progress: ${msg}`)
  chrome.runtime.sendMessage(msg);
}

export function report_error(msg) {
  console.error(`Error in WASM: ${msg}`)
  chrome.runtime.sendMessage(msg)
}