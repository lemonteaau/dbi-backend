# DBI Backend

[中文](#中文) | [English](#english)

---

## 中文

一个基于 Tauri v2 的桌面应用，用于通过 USB 将 Nintendo Switch 游戏文件（NSP/NSZ/XCI/XCZ）安装到 Switch。替代原有的 Python `dbibackend` 脚本，提供原生性能和现代化 GUI。

### 功能特性

- 通过 USB 向 Switch 上的 DBI 传输游戏文件
- 支持 NSP、NSZ、XCI、XCZ 格式
- 拖放添加文件或文件夹
- 实时传输进度和日志
- 一键启动 / 停止服务
- 中英文界面切换
- 原生跨平台桌面应用（macOS / Windows / Linux）

### 使用前提

1. Switch 上已安装 [DBI](https://github.com/rashevskyv/dbi) 自制应用
2. 在 Switch 上打开 DBI，选择 **"从DBIbackend安装"**
3. 通过 USB 线缆连接 Switch 到电脑

### 安装方式

从 [Releases](../../releases) 页面下载对应平台的安装包：

| 平台 | 格式 |
|------|------|
| macOS | `.dmg` |
| Windows | `.msi` / `.exe` |
| Linux | `.deb` / `.AppImage` |

### 从源码构建

**依赖：**

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (18+)
- [pnpm](https://pnpm.io/)

```bash
# 克隆仓库
git clone https://github.com/lemonteaau/dbi-backend.git
cd dbi-backend

# 安装前端依赖
pnpm install

# 开发模式运行
pnpm dev

# 构建生产版本
pnpm build
```

构建产物位于 `src-tauri/target/release/bundle/`。

### 使用方法

1. 在 Switch 上打开 DBI，选择 "从DBIbackend安装"
2. 通过 USB 连接 Switch 到电脑
3. 打开 DBI Backend 应用
4. 拖放或点击按钮添加游戏文件
5. 点击 **"启动服务"** 开始传输
6. 等待传输完成，Switch 端会自动安装

### 技术栈

- **前端**: HTML / CSS / JavaScript (Vanilla)
- **后端**: Rust + [nusb](https://github.com/kevinmehall/nusb) (纯 Rust USB 库)
- **框架**: [Tauri v2](https://v2.tauri.app/)
- **协议**: DBI USB 自定义协议（16 字节头部，批量传输端点）

### 许可证

MIT

---

## English

A Tauri v2 desktop application for installing Nintendo Switch game files (NSP/NSZ/XCI/XCZ) via USB. Replaces the Python `dbibackend` script with native performance and a modern GUI.

### Features

- Transfer game files to DBI on Switch via USB
- Supports NSP, NSZ, XCI, XCZ formats
- Drag-and-drop files or folders
- Real-time transfer progress and logging
- One-click start / stop server
- Chinese / English UI toggle
- Native cross-platform desktop app (macOS / Windows / Linux)

### Prerequisites

1. [DBI](https://github.com/rashevskyv/dbi) homebrew installed on your Switch
2. Open DBI on Switch, select **"Install from DBIbackend"**
3. Connect Switch to your computer via USB cable

### Installation

Download the installer for your platform from the [Releases](../../releases) page:

| Platform | Format |
|----------|--------|
| macOS | `.dmg` |
| Windows | `.msi` / `.exe` |
| Linux | `.deb` / `.AppImage` |

### Building from Source

**Requirements:**

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (18+)
- [pnpm](https://pnpm.io/)

```bash
# Clone the repo
git clone https://github.com/lemonteaau/dbi-backend.git
cd dbi-backend

# Install frontend dependencies
pnpm install

# Run in development mode
pnpm dev

# Build for production
pnpm build
```

Build artifacts are located in `src-tauri/target/release/bundle/`.

### Usage

1. Open DBI on your Switch, select "Install from DBIbackend"
2. Connect Switch to your computer via USB
3. Launch the DBI Backend app
4. Drag-and-drop or click buttons to add game files
5. Click **"Start Server"** to begin transferring
6. Wait for the transfer to complete — DBI will install automatically on the Switch

### Tech Stack

- **Frontend**: HTML / CSS / JavaScript (Vanilla)
- **Backend**: Rust + [nusb](https://github.com/kevinmehall/nusb) (pure Rust USB library)
- **Framework**: [Tauri v2](https://v2.tauri.app/)
- **Protocol**: DBI USB custom protocol (16-byte headers, bulk transfer endpoints)

### License

MIT
