async function getMasterPassword() {
  const { mp } = await chrome.storage.session.get('mp');
  return mp ?? null;
}

async function setMasterPassword(pw) {
  await chrome.storage.session.set({ mp: pw });
}

async function send(msg) {
  const mp = await getMasterPassword();
  return chrome.runtime.sendMessage({ ...msg, master_password: mp });
}

async function showCredentials() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  const resp = await send({ action: 'get_credentials', url: tab.url });
  const box = document.getElementById('creds');
  box.innerHTML = '';
  if (!resp.ok) { document.getElementById('status').textContent = resp.error; return; }
  if (!resp.credentials.length) {
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
  const resp = await chrome.runtime.sendMessage({ action: 'get_credentials', url: tab.url, master_password: mp });
  if (!resp.ok) {
    document.getElementById('status').textContent = resp.error;
    return;
  }
  await setMasterPassword(mp);
  document.getElementById('unlock-view').style.display = 'none';
  document.getElementById('main-view').style.display = 'block';
  await showCredentials();
  await checkPendingSave();
};

document.getElementById('lock-btn').onclick = async () => {
  await chrome.storage.session.remove('mp');
  document.getElementById('main-view').style.display = 'none';
  document.getElementById('unlock-view').style.display = 'block';
};

(async () => {
  const mp = await getMasterPassword();
  if (mp) {
    document.getElementById('unlock-view').style.display = 'none';
    document.getElementById('main-view').style.display = 'block';
    await showCredentials();
    await checkPendingSave();
  }
})();

// Store pending save from content script
chrome.runtime.onMessage.addListener(async (msg) => {
  if (msg.action === 'offer_save') {
    await chrome.storage.session.set({ pendingSave: { website: msg.website, username: msg.username, password: msg.password } });
  }
});
