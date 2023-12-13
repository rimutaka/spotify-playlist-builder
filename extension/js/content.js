// console.log("Hello from content");

// this makes it universal for chrome/ff
// FF understands `chrome`, so this is not used in other parts of the code
// there are some differences how FF treats `chrome` vs `browser`, see MDN for more
const runtime = chrome.runtime || browser.runtime;

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