async function getDerivedKey() {
  const { dk } = await chrome.storage.session.get('dk');
  return dk ?? null;
}

async function send(msg) {
  const dk = await getDerivedKey();
  return chrome.runtime.sendMessage({ ...msg, derived_key: dk });
}

async function showCredentials() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  const resp = await send({ action: 'get_credentials', url: tab.url });
  const box = document.getElementById('creds');
  box.innerHTML = '';
  if (!resp.ok) { document.getElementById('status').textContent = resp.error; return; }
  if (!resp.credentials?.length) {
    box.textContent = 'No credentials for this site.';
    return;
  }
  for (const c of resp.credentials) {
    const div = document.createElement('div');
    div.textContent = `${c.website} — ${c.username}`;
    div.onclick = async () => {
      const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
      await chrome.tabs.sendMessage(tab.id, { action: 'fill', username: c.username, password: c.password });
      window.close();
    };
    box.appendChild(div);
  }
}

async function checkPendingSave() {
  const { pendingSave } = await chrome.storage.session.get('pendingSave');
  if (!pendingSave) return;
  const banner = document.getElementById('save-banner');
  banner.style.display = 'block';
  banner.innerHTML = `Save <b>${pendingSave.username}</b> for <b>${pendingSave.website}</b>?
    <button id="save-yes" style="width:auto;padding:4px 10px;margin:4px 4px 0 0">Save</button>
    <button id="save-no" style="width:auto;padding:4px 10px;background:#45475a">Skip</button>`;
  document.getElementById('save-yes').onclick = async () => {
    await send({ action: 'save_credentials', ...pendingSave });
    await chrome.storage.session.remove('pendingSave');
    banner.style.display = 'none';
  };
  document.getElementById('save-no').onclick = async () => {
    await chrome.storage.session.remove('pendingSave');
    banner.style.display = 'none';
  };
}

document.getElementById('unlock-btn').onclick = async () => {
  const mp = document.getElementById('mp').value;
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  // Derive the key from the master password; do not store the password itself.
  const resp = await chrome.runtime.sendMessage({ action: 'derive_key', master_password: mp });
  if (!resp.ok) {
    document.getElementById('status').textContent = resp.error;
    return;
  }
  await chrome.storage.session.set({ dk: resp.key });
  document.getElementById('unlock-view').style.display = 'none';
  document.getElementById('main-view').style.display = 'block';
  await showCredentials();
  await checkPendingSave();
};

document.getElementById('lock-btn').onclick = async () => {
  await chrome.storage.session.remove('dk');
  document.getElementById('main-view').style.display = 'none';
  document.getElementById('unlock-view').style.display = 'block';
};

(async () => {
  const dk = await getDerivedKey();
  if (dk) {
    document.getElementById('unlock-view').style.display = 'none';
    document.getElementById('main-view').style.display = 'block';
    await showCredentials();
    await checkPendingSave();
  }
})();
