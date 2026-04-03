# Virtuoso CLI

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.75+-blue.svg" alt="Rust 1.75+"/></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green.svg" alt="License: MIT"/></a>
</p>

从任何地方控制 Cadence Virtuoso，本地或远程均可。支持 macOS、Windows 和 Linux。

---

## 简介 | Introduction

Virtuoso CLI 是一个用 Rust 重写的轻量级桥接工具，用于在 Virtuoso 外部执行 SKILL 代码。它基于 `ipcBeginProcess` + `evalstring` 机制，与 [virtuoso-bridge-lite](https://github.com/Arcadia-1/virtuoso-bridge-lite) 使用相同的核心协议。

Virtuoso CLI is a lightweight Rust reimplementation of the virtuoso bridge, enabling SKILL code execution outside Virtuoso. It uses the same core mechanism (`ipcBeginProcess` + `evalstring`) as virtuoso-bridge-lite.

### 核心特性 | Key Features

- **三种编程方式** — 原始 SKILL 表达式、Python 风格 API、或直接加载 .il 文件
- **本地+远程模式** — 支持本地直连或 SSH 隧道远程控制
- **AI 原生设计** — CLI 优先，支持代理自动化
- **混合 Daemon 部署** — 自动检测 Python 版本，无 Python 时回退到 Rust 二进制
- **Spectre 仿真集成** — 内置本地/远程仿真运行器和 PSF 结果解析

- **Three ways to program** — raw SKILL expressions, Pythonic APIs, or load .il files
- **Local + Remote** — direct local connection or SSH tunnel for remote control
- **AI-native design** — CLI-first, agent automatable
- **Hybrid daemon deployment** — auto-detects Python, falls back to Rust binary if unavailable
- **Spectre simulation** — built-in local/remote runner with PSF result parsing

---

## 快速开始 | Quick Start

### 安装 | Installation

```bash
# 克隆并构建
git clone https://github.com/your-repo/virtuoso-cli.git
cd virtuoso-cli
cargo build --release

# 安装到系统（可选）
# sudo cp target/release/virtuoso /usr/local/bin/
```

### 本地模式 | Local Mode

```bash
# 1. 在 Virtuoso CIW 中加载 SKILL bridge
#    load("/path/to/ramic_bridge.il")
#    （daemon 会在端口 65432 上自动启动）

# 2. 测试连接
virtuoso status

# 3. 执行 SKILL
virtuoso exec "1+2"
virtuoso exec "hiGetCurrentWindow()"
virtuoso exec "geGetEditCellView()~>cellName"

# 4. 打开 cellview
virtuoso open --lib myLib --cell myCell --view layout
```

### 远程模式 | Remote Mode

```bash
# 1. 初始化配置文件
virtuoso init
# 生成 .env 模板

# 2. 编辑 .env
# VB_REMOTE_HOST=my-server     # SSH 主机别名（来自 ~/.ssh/config）
# VB_REMOTE_USER=myuser        # 可选
# VB_JUMP_HOST=jump-server     # 可选，跳板机

# 3. 启动 SSH 隧道 + 部署 daemon
virtuoso start

# 4. 检查状态
virtuoso status

# 5. 执行 SKILL（与本地相同）
virtuoso exec "dbOpenCellViewByType(\"myLib\" \"myCell\" \"layout\" \"r\")"
```

---

## 命令参考 | Command Reference

| 命令 | 说明 | Description |
|------|------|-------------|
| `virtuoso init` | 创建 .env 模板 | Create .env template |
| `virtuoso start` | 启动 SSH 隧道 + 部署 daemon | Start SSH tunnel + deploy daemon |
| `virtuoso stop` | 停止隧道 | Stop tunnel |
| `virtuoso restart` | 重启隧道 + daemon | Restart tunnel + daemon |
| `virtuoso status` | 检查连接状态 | Check connection status |
| `virtuoso exec <code>` | 执行 SKILL 代码 | Execute SKILL code |
| `virtuoso open --lib L --cell C --view V` | 打开 cellview | Open a cellview |

---

## 配置变量 | Environment Variables

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `VB_REMOTE_HOST` | - | SSH 远程主机（必填 for remote mode） |
| `VB_REMOTE_USER` | 当前用户 | SSH 用户名 |
| `VB_PORT` | 65432 | TCP 端口 |
| `VB_JUMP_HOST` | - | 跳板机/堡垒机 |
| `VB_JUMP_USER` | - | 跳板机用户名 |
| `VB_TIMEOUT` | 30 | 超时时间（秒） |
| `VB_KEEP_REMOTE_FILES` | false | 停止时保留远程文件 |
| `VB_SPECTRE_CMD` | spectre | Spectre 命令 |
| `VB_SPECTRE_ARGS` | - | Spectre 额外参数 |

---

## 工作原理 | How It Works

```
本机                                远程 Virtuoso 服务器
────                                ────────────────────

virtuoso exec "1+2"
      │
      │ TCP: {"skill":"1+2"}
      ├──── SSH tunnel ───────► ramic_daemon.py
      │     (localhost:65432)         │
      │                               │ stdout: "1+2"
      │                               ├──► evalstring("1+2")
      │                               │        │
      │                               │        ▼
      │                               │ stdin: "\x02 3 \x1e"
      │                               ◄──┘
      │ TCP: "\x02 3"
      ◄──── SSH tunnel ────────────┘
      │
      ▼
     "3"
```

---

## 构建 Rust Daemon | Building Rust Daemon

```bash
# 构建 daemon binary
cargo build --features daemon --release

# 复制到 resources 目录（用于远程无 Python 时回退）
cp target/release/virtuoso-daemon resources/daemons/virtuoso-daemon-x86_64
```

---

## 许可证 | License

MIT License - 详见 LICENSE 文件