const HOST = "com.rustlock.host";

// Handle offer_save locally — do NOT forward to native host.
chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
  if (msg.action === 'offer_save') {
    chrome.storage.session.set({
      pendingSave: { website: msg.website, username: msg.username, password: msg.password }
    });
    sendResponse({ ok: true });
    return false;
  }

  // Relay all other messages to the native host.
  const port = chrome.runtime.connectNative(HOST);
  port.postMessage(msg);
  port.onMessage.addListener((response) => {
    sendResponse(response);
    port.disconnect();
  });
  port.onDisconnect.addListener(() => {
    if (chrome.runtime.lastError) {
      sendResponse({ ok: false, error: chrome.runtime.lastError.message });
    }
  });
  return true; // keep channel open for async sendResponse
});
