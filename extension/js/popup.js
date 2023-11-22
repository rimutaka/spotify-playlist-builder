console.log("popup.js loaded");

// Popups cannot have any inline scripts with our security policies.
// Click handlers should be added when the popup is opened.
document.addEventListener('DOMContentLoaded', function () {
  console.log("Toolbar button clicked");

  // contact us / feedback link
  // links cannot be open on-click as in a normal web page
  // they have to go through an API and open in a new tab
  // https://developer.chrome.com/docs/extensions/reference/tabs/#opening-an-extension-page-in-a-new-tab
  let btn = document.getElementById("btn_contact");
  btn.addEventListener("click", async () => {
    console.log("btn_contact button clicked");
    chrome.tabs.create({ url: "https://github.com/rimutaka/spotify-playlist-builder/issues" });
  });

  btn = document.getElementById("btn_info");
  btn.addEventListener("click", async () => {
    console.log("btn_info button clicked");
    chrome.tabs.create({ url: "https://github.com/rimutaka/spotify-playlist-builder/#readme" });
  });

  // add tracks button
  btn = document.getElementById("btn_add");
  btn.addEventListener("click", async () => {
    console.log("btn_add button clicked");

    await chrome.runtime.sendMessage({ action: "btn_add" });
  });
});