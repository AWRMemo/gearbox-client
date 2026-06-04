const DEFAULT_ENDPOINT = 'http://127.0.0.1:3000/v1/extension/capture';
const MAX_RETRIES = 3;
const RETRY_DELAY_MS = 1000;

function getEndpoint() {
  return new Promise((resolve) => {
    chrome.storage.local.get({ relayEndpoint: DEFAULT_ENDPOINT }, (result) => {
      resolve(result.relayEndpoint);
    });
  });
}

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: 'capture-to-relay',
    title: 'Capture to Relay',
    contexts: ['selection'],
  });
});

chrome.contextMenus.onClicked.addListener((info, tab) => {
  if (info.menuItemId === 'capture-to-relay' && info.selectionText) {
    captureText(info.selectionText, tab?.url);
  }
});

chrome.commands.onCommand.addListener((command) => {
  if (command === 'capture-selection') {
    chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
      chrome.scripting.executeScript({
        target: { tabId: tabs[0].id },
        func: () => window.getSelection().toString(),
      }, (results) => {
        if (results?.[0]?.result) {
          captureText(results[0].result, tabs[0]?.url);
        }
      });
    });
  }
});

async function captureText(text, sourceUrl) {
  const endpoint = await getEndpoint();
  let lastError = null;
  for (let attempt = 0; attempt < MAX_RETRIES; attempt++) {
    try {
      const resp = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ text, source_url: sourceUrl || '' }),
      });
      if (resp.ok) {
        const data = await resp.json();
        chrome.storage.local.get({ captures: [] }, (result) => {
          const captures = [data, ...result.captures].slice(0, 3);
          chrome.storage.local.set({ captures });
        });
        return;
      }
      lastError = `HTTP ${resp.status}`;
    } catch (e) {
      lastError = e.message;
    }
    if (attempt < MAX_RETRIES - 1) {
      await new Promise(r => setTimeout(r, RETRY_DELAY_MS * Math.pow(2, attempt)));
    }
  }
  chrome.storage.local.get({ queue: [] }, (result) => {
    result.queue.push({ text, source_url: sourceUrl || '', timestamp: Date.now() });
    chrome.storage.local.set({ queue: result.queue });
  });
}
