const HOST = "com.rustlock.host";

chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
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
