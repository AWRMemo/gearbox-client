const endpointInput = document.getElementById('endpoint');
const statusEl = document.getElementById('status');

chrome.storage.local.get({ relayEndpoint: 'http://127.0.0.1:3000/v1/extension/capture' }, (result) => {
  endpointInput.value = result.relayEndpoint;
});

document.getElementById('save').addEventListener('click', () => {
  const endpoint = endpointInput.value.trim();
  chrome.storage.local.set({ relayEndpoint: endpoint }, () => {
    statusEl.textContent = 'Saved.';
    setTimeout(() => { statusEl.textContent = ''; }, 2000);
  });
});
