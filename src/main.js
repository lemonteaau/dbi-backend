import { t, getLang, setLang, applyI18n } from './i18n.js';

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { open } = window.__TAURI__.dialog;

// DOM Elements
const statusIndicator = document.getElementById('statusIndicator');
const statusText = document.getElementById('statusText');
const tipBanner = document.getElementById('tipBanner');
const tipClose = document.getElementById('tipClose');
const fileCount = document.getElementById('fileCount');
const fileList = document.getElementById('fileList');
const fileListEmpty = document.getElementById('fileListEmpty');
const dropZone = document.getElementById('dropZone');
const logContainer = document.getElementById('logContainer');
const transferInfo = document.getElementById('transferInfo');
const btnAddFolder = document.getElementById('btnAddFolder');
const btnAddFiles = document.getElementById('btnAddFiles');
const btnClear = document.getElementById('btnClear');
const btnClearLog = document.getElementById('btnClearLog');
const btnStart = document.getElementById('btnStart');
const btnStartText = document.getElementById('btnStartText');
const langToggle = document.getElementById('langToggle');
const logReady = document.getElementById('logReady');

let serverRunning = false;
let isConnected = false;

// SVG icons
const ICON_PLAY = `<svg width="18" height="18" viewBox="0 0 18 18" fill="none"><polygon points="5,3 15,9 5,15" fill="currentColor"/></svg>`;
const ICON_STOP = `<svg width="18" height="18" viewBox="0 0 18 18" fill="none"><rect x="4" y="4" width="10" height="10" rx="2" fill="currentColor"/></svg>`;

// ===== Language Toggle =====

function updateLangToggle() {
  const lang = getLang();
  langToggle.textContent = lang === 'zh' ? '中' : 'EN';
}

function switchLanguage() {
  const lang = getLang() === 'zh' ? 'en' : 'zh';
  setLang(lang);
  applyI18n();
  updateLangToggle();
  // Update dynamic content
  updateDynamicTexts();
}

/** Re-apply texts that are set dynamically (not via data-i18n attributes). */
function updateDynamicTexts() {
  // Status text
  statusText.textContent = isConnected ? t('status.connected') : t('status.disconnected');
  // Start button
  if (serverRunning) {
    btnStartText.textContent = t('btn.stopServer');
  } else {
    btnStartText.textContent = t('btn.startServer');
  }
  // File count
  const items = fileList.querySelectorAll('li');
  const count = fileListEmpty.style.display === 'none' ? items.length : 0;
  fileCount.textContent = t('fileList.count', { n: count });
  // Ready log line
  if (logReady) {
    logReady.textContent = t('log.ready');
  }
}

langToggle.addEventListener('click', switchLanguage);

// ===== Event Listeners from Rust backend =====

listen('log', (event) => {
  appendLog(event.payload.message, event.payload.level || 'info');
});

listen('connection-status', (event) => {
  isConnected = event.payload.connected;
  statusIndicator.classList.toggle('connected', isConnected);
  statusText.textContent = isConnected ? t('status.connected') : t('status.disconnected');
});

listen('transfer-progress', (event) => {
  const { file, bytes_sent, total_bytes } = event.payload;
  const pct = total_bytes > 0 ? ((bytes_sent / total_bytes) * 100).toFixed(1) : 0;
  const sent = formatSize(bytes_sent);
  const total = formatSize(total_bytes);
  transferInfo.textContent = `${file} — ${pct}% (${sent} / ${total})`;
});

listen('server-stopped', (event) => {
  serverRunning = false;
  setUIRunning(false);

  const summary = event.payload?.summary;
  if (summary) {
    appendLog(summary, 'success');
  }
  transferInfo.textContent = '';
});

// ===== UI State =====

function setUIRunning(running) {
  if (running) {
    btnStart.classList.add('running');
    btnStart.querySelector('svg')?.remove();
    btnStart.insertAdjacentHTML('afterbegin', ICON_STOP);
    btnStartText.textContent = t('btn.stopServer');
    btnStart.disabled = false;
    btnAddFolder.disabled = true;
    btnAddFiles.disabled = true;
    btnClear.disabled = true;
  } else {
    btnStart.classList.remove('running');
    btnStart.querySelector('svg')?.remove();
    btnStart.insertAdjacentHTML('afterbegin', ICON_PLAY);
    btnStartText.textContent = t('btn.startServer');
    btnAddFolder.disabled = false;
    btnAddFiles.disabled = false;
    btnClear.disabled = false;
    refreshStartButton();
  }
}

function refreshStartButton() {
  btnStart.disabled = fileListEmpty.style.display !== 'none';
}

// ===== UI Functions =====

function appendLog(message, level = 'info') {
  const div = document.createElement('div');
  div.className = `log-line log-${level}`;
  const time = new Date().toLocaleTimeString('en-US', { hour12: false });
  div.textContent = `[${time}] ${message}`;
  logContainer.appendChild(div);
  logContainer.scrollTop = logContainer.scrollHeight;

  // Keep log under 500 entries
  while (logContainer.children.length > 500) {
    logContainer.removeChild(logContainer.firstChild);
  }
}

function formatSize(bytes) {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + ' GB';
}

async function refreshFileList() {
  try {
    const files = await invoke('get_file_list');
    fileList.innerHTML = '';

    if (files.length === 0) {
      fileListEmpty.style.display = 'flex';
      fileCount.textContent = t('fileList.count', { n: 0 });
      if (!serverRunning) btnStart.disabled = true;
    } else {
      fileListEmpty.style.display = 'none';
      fileCount.textContent = t('fileList.count', { n: files.length });
      if (!serverRunning) btnStart.disabled = false;

      files.forEach((f) => {
        const li = document.createElement('li');
        li.innerHTML = `
          <span class="file-icon">
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none"><path d="M4 2h5l3 3v9c0 .6-.4 1-1 1H4c-.6 0-1-.4-1-1V3c0-.6.4-1 1-1z" stroke="currentColor" stroke-width="1.5"/></svg>
          </span>
          <span class="file-name" title="${f.path}">${f.name}</span>
          <span class="file-size">${formatSize(f.size)}</span>
          <button class="file-remove" data-name="${f.name}" title="Remove">&times;</button>
        `;
        fileList.appendChild(li);
      });
    }
  } catch (e) {
    appendLog('Failed to refresh file list: ' + e, 'error');
  }
}

// ===== Button Handlers =====

btnAddFolder.addEventListener('click', async () => {
  try {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      await invoke('add_folder', { path: selected });
      await refreshFileList();
    }
  } catch (e) {
    appendLog('Error: ' + e, 'error');
  }
});

btnAddFiles.addEventListener('click', async () => {
  try {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'Switch Games', extensions: ['nsp', 'nsz', 'xci', 'xcz'] }]
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      await invoke('add_files', { paths });
      await refreshFileList();
    }
  } catch (e) {
    appendLog('Error: ' + e, 'error');
  }
});

btnClear.addEventListener('click', async () => {
  await invoke('clear_files');
  await refreshFileList();
});

btnClearLog.addEventListener('click', () => {
  logContainer.innerHTML = '';
});

btnStart.addEventListener('click', async () => {
  if (serverRunning) {
    // ---- Stop ----
    btnStart.disabled = true;
    btnStartText.textContent = t('btn.stopping');
    try {
      await invoke('stop_server');
      appendLog(t('log.stopping'), 'warn');
    } catch (e) {
      appendLog('Stop error: ' + e, 'error');
      btnStart.disabled = false;
    }
    return;
  }

  // ---- Start ----
  serverRunning = true;
  setUIRunning(true);

  try {
    await invoke('start_server');
  } catch (e) {
    appendLog('Server error: ' + e, 'error');
    serverRunning = false;
    setUIRunning(false);
  }
});

// ===== Tip Banner =====
tipClose.addEventListener('click', () => {
  tipBanner.classList.add('hidden');
});

// ===== File Remove (delegated) =====
fileList.addEventListener('click', async (e) => {
  const removeBtn = e.target.closest('.file-remove');
  if (removeBtn) {
    const name = removeBtn.dataset.name;
    await invoke('remove_file', { name });
    await refreshFileList();
  }
});

// ===== Drag & Drop =====
document.addEventListener('dragover', (e) => e.preventDefault());
document.addEventListener('drop', (e) => e.preventDefault());

listen('tauri://drag-enter', (_event) => {
  dropZone.classList.add('drag-over');
});

listen('tauri://drag-leave', (_event) => {
  dropZone.classList.remove('drag-over');
});

listen('tauri://drag-drop', async (event) => {
  dropZone.classList.remove('drag-over');
  try {
    const paths = event.payload.paths;
    if (paths && paths.length > 0) {
      appendLog(t('log.dropped', { n: paths.length }), 'info');
      await invoke('add_paths', { paths });
      await refreshFileList();
    }
  } catch (e) {
    appendLog('Drop error: ' + e, 'error');
  }
});

// ===== Init =====
applyI18n();
updateLangToggle();
logReady.textContent = t('log.ready');
refreshFileList();
