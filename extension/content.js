function findLoginForm() {
  const pw = document.querySelector('input[type="password"]');
  if (!pw) return null;
  const form = pw.closest('form');
  const user = form?.querySelector('input[type="email"],input[type="text"],[name*="user"],[name*="email"]')
             ?? document.querySelector('input[type="email"]');
  return { pw, user, form };
}

function injectFillButton(pwField) {
  if (document.getElementById('rl-fill')) return;
  const btn = document.createElement('button');
  btn.id = 'rl-fill';
  btn.textContent = '🔑';
  btn.title = 'Fill with Rustlock';
  btn.style.cssText = 'position:absolute;z-index:9999;background:#1e1e2e;color:#cdd6f4;border:none;border-radius:4px;padding:2px 6px;cursor:pointer;font-size:13px;';
  const r = pwField.getBoundingClientRect();
  btn.style.left = `${window.scrollX + r.right - 32}px`;
  btn.style.top  = `${window.scrollY + r.top + 2}px`;
  btn.onclick = (e) => { e.preventDefault(); chrome.runtime.sendMessage({ action: 'open_popup' }); };
  document.body.appendChild(btn);
}

// Fill command from popup
chrome.runtime.onMessage.addListener((msg) => {
  if (msg.action !== 'fill') return;
  const f = findLoginForm();
  if (!f) return;
  if (f.user) { f.user.value = msg.username; f.user.dispatchEvent(new Event('input', { bubbles: true })); }
  f.pw.value = msg.password;
  f.pw.dispatchEvent(new Event('input', { bubbles: true }));
});

// Offer to save on form submit
document.addEventListener('submit', (e) => {
  const form = e.target;
  const pw   = form.querySelector('input[type="password"]');
  const user = form.querySelector('input[type="email"],input[type="text"]');
  if (pw?.value && user?.value) {
    chrome.runtime.sendMessage({
      action: 'offer_save',
      website: location.hostname,
      username: user.value,
      password: pw.value,
    });
  }
}, true);

if (findLoginForm()) injectFillButton(findLoginForm().pw);
