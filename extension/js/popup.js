console.log("popup.js loaded");

// A placeholder for OnSuccess in .then
function onSuccess(message) {
  // console.log(`Send OK: ${JSON.stringify(message)}`);
}

// A placeholder for OnError in .then
function onError(error) {
  console.error(`Promise error: ${error}`);
}

// Popups cannot have any inline scripts with our security policies.
// Click handlers should be added when the popup is opened.
document.addEventListener('DOMContentLoaded', function () {
  // console.log("Toolbar button clicked");

  // contact us / feedback link
  // links cannot be open on-click as in a normal web page
  // they have to go through an API and open in a new tab
  // https://developer.chrome.com/docs/extensions/reference/tabs/#opening-an-extension-page-in-a-new-tab
  let btn = document.getElementById("btn_contact");
  btn.addEventListener("click", async () => {
    chrome.tabs.create({ url: "https://github.com/rimutaka/spotify-playlist-builder/issues" });
  });

  btn = document.getElementById("btn_info");
  btn.addEventListener("click", async () => {
    chrome.tabs.create({ url: "https://github.com/rimutaka/spotify-playlist-builder/#readme" });
  });

  // add tracks button
  btn = document.getElementById("btn_add");
  btn.addEventListener("click", async (evt) => {
    console.log("btn_add button clicked");
    evt.currentTarget.disabled = true;
    await chrome.runtime.sendMessage({ action: "btn_add" });
  });

  chrome.action.getBadgeText({}).then((badgeText) => {
    // console.log(`Badge: ${badgeText}`);
    if (badgeText) {
      btn.disabled = true;
      document.getElementById("log-summary").innerText = "Waiting for progress update ..."
    };
  }, onError)

});

// listens for msgs from WASM
chrome.runtime.onMessage.addListener((msg, sender) => {

  // background.js may send a status update as boolean because
  // there is no badge change event
  if (typeof msg === "boolean") {
    document.getElementById("btn_add").disabled = msg;
    return;
  }

  // if it's not a bool, then it is a log entry as a string
  const log = document.getElementById("log");

  const lastMsg = document.getElementById("log-summary").innerText;
  if (lastMsg) {
    const p = document.createElement("p");
    p.innerText = lastMsg;

    log.insertBefore(p, log.firstChild);
  }

  document.getElementById("log-summary").innerText = msg;
}
);