/**
 * Minimal i18n module for DBI Backend.
 *
 * Supports Chinese (zh) and English (en).
 * Language preference is persisted in localStorage.
 */

const translations = {
  zh: {
    'header.subtitle': 'Switch USB 安装器',
    'status.disconnected': '未连接',
    'status.connected': '已连接',
    'link.github': '在 GitHub 上打开项目',
    'tip.title': '开始前：',
    'tip.content': '在 Switch 上打开 DBI，选择 "从DBIbackend安装"，然后连接 USB 线缆。',
    'panel.files': '文件列表',
    'panel.log': '日志',
    'btn.addFolder': '添加文件夹',
    'btn.addFiles': '添加文件',
    'btn.clearAll': '清空',
    'btn.clearLog': '清除',
    'btn.startServer': '启动服务',
    'btn.stopServer': '停止服务',
    'btn.stopping': '正在停止...',
    'fileList.empty': '拖放 NSP / NSZ 文件到此处',
    'fileList.emptyHint': '或使用上方按钮添加',
    'fileList.count': '{n} 个文件',
    'log.ready': '就绪。添加文件后启动服务。',
    'log.dropped': '已拖入 {n} 个项目',
    'log.stopping': '正在停止服务...',
    'log.selectOnSwitch': '下一步：请在 Switch 的 DBI 里勾选要安装的游戏并开始安装，之后才会开始传输。',
    'info.waitingForSwitch': '等待在 Switch 上选择游戏并开始安装…',
    'hint.noDbiCommand': '已识别到 Switch 的 USB 设备，但 DBI 还没有发来指令。请确认 Switch 上打开的是 DBI，并已进入"从DBIbackend安装"界面（不是 MTP 模式，也不是其他安装软件）。',
    'hint.notDbi': '收到的不是 DBI 的数据：Switch 上可能开着其他 USB 安装软件（如 Goldleaf、Tinfoil）。请改用 DBI 的"从DBIbackend安装"。',
  },
  en: {
    'header.subtitle': 'Switch USB Installer',
    'status.disconnected': 'Disconnected',
    'status.connected': 'Connected',
    'link.github': 'Open project on GitHub',
    'tip.title': 'Before you start:',
    'tip.content': 'Open DBI on your Switch, select "Install from DBIbackend", then connect USB cable.',
    'panel.files': 'Files',
    'panel.log': 'Log',
    'btn.addFolder': 'Add Folder',
    'btn.addFiles': 'Add Files',
    'btn.clearAll': 'Clear All',
    'btn.clearLog': 'Clear',
    'btn.startServer': 'Start Server',
    'btn.stopServer': 'Stop Server',
    'btn.stopping': 'Stopping...',
    'fileList.empty': 'Drop NSP / NSZ files here',
    'fileList.emptyHint': 'or use the buttons above',
    'fileList.count': '{n} file(s)',
    'log.ready': 'Ready. Add files and start the server.',
    'log.dropped': 'Dropped {n} item(s)',
    'log.stopping': 'Stopping server...',
    'log.selectOnSwitch': 'Next: in DBI on your Switch, select the games to install and start the installation. The transfer begins after that.',
    'info.waitingForSwitch': 'Waiting for you to select games and start the install on the Switch…',
    'hint.noDbiCommand': 'Found the Switch USB device, but DBI has not sent any command yet. Make sure DBI is open on the Switch and in "Install from DBIbackend" (not MTP mode or another installer).',
    'hint.notDbi': 'Received data that is not from DBI: another USB installer (e.g. Goldleaf, Tinfoil) may be running on the Switch. Use "Install from DBIbackend" in DBI instead.',
  },
};

let currentLang = localStorage.getItem('dbi-lang') || 'en';

/**
 * Get translated string by key, with optional interpolation.
 * Usage: t('fileList.count', { n: 5 }) => "5 file(s)"
 */
export function t(key, params) {
  let str = translations[currentLang]?.[key] || translations['en']?.[key] || key;
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      str = str.replace(`{${k}}`, v);
    }
  }
  return str;
}

/** Get the current language code. */
export function getLang() {
  return currentLang;
}

/** Set language and persist. Does NOT update the DOM — call applyI18n() after. */
export function setLang(lang) {
  if (translations[lang]) {
    currentLang = lang;
    localStorage.setItem('dbi-lang', lang);
  }
}

/**
 * Apply translations to all elements with `data-i18n` attribute.
 * The attribute value is used as the translation key.
 * For elements with `data-i18n-placeholder`, the placeholder is set instead.
 */
export function applyI18n() {
  document.querySelectorAll('[data-i18n]').forEach((el) => {
    const key = el.getAttribute('data-i18n');
    el.textContent = t(key);
  });
  document.querySelectorAll('[data-i18n-title]').forEach((el) => {
    const key = el.getAttribute('data-i18n-title');
    el.title = t(key);
  });
}
