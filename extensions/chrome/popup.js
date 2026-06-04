chrome.storage.local.get({ captures: [] }, (result) => {
  const el = document.getElementById('captures');
  if (!result.captures || result.captures.length === 0) return;
  el.classList.remove('empty');
  el.innerHTML = result.captures.map(c => `
    <div class="item">
      <div class="summary">${escapeHtml(c.summary || c.text?.slice(0, 80) || 'Captured')}</div>
      ${c.source_url ? `<div class="source">${escapeHtml(c.source_url)}</div>` : ''}
      ${c.tags ? `<div class="tags">${c.tags.map(t => `<span>${escapeHtml(t)}</span>`).join('')}</div>` : ''}
    </div>`).join('');
});

function escapeHtml(s) {
  const div = document.createElement('div');
  div.textContent = s;
  return div.innerHTML;
}
