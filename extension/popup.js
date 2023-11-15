console.log("popup.js loaded");

// Popups cannot have any inline scripts with our security policies.
// Click handlers should be added when the popup is opened.
document.addEventListener('DOMContentLoaded', function () {
  console.log("Toolbar button clicked");

  let btn = document.getElementById("btn_add");
  btn.addEventListener("click", async () => {
    console.log("btn_add button clicked");

    await chrome.runtime.sendMessage({ action: "btn_add" });
  });
});