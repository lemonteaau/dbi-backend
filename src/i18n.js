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
  },
  en: {
    'header.subtitle': 'Switch USB Installer',
    'status.disconnected': 'Disconnected',
    'status.connected': 'Connected',
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
