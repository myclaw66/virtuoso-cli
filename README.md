# Virtuoso CLI

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.75+-blue.svg" alt="Rust 1.75+"/></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green.svg" alt="License: MIT"/></a>
</p>

从任何地方控制 Cadence Virtuoso，本地或远程均可。为 AI Agent 和人类共同设计。

---

## 简介

Virtuoso CLI 是一个用 Rust 重写的轻量级桥接工具，用于在 Virtuoso 外部执行 SKILL 代码。它基于 `ipcBeginProcess` + `evalstring` 机制，与 [virtuoso-bridge-lite](https://github.com/Arcadia-1/virtuoso-bridge-lite) 使用相同的核心协议。

### 核心特性

- **三种编程方式** — 原始 SKILL 表达式、高阶 API、或直接加载 .il 文件
- **本地+远程模式** — 支持本地直连或 SSH 隧道远程控制
- **Agent 原生 CLI** — noun-verb 命令结构、JSON 结构化输出、schema 自省、语义化退出码
- **混合 Daemon 部署** — 自动检测 Python 版本，无 Python 时回退到 Rust 二进制
- **Spectre 仿真集成** — 内置本地/远程仿真运行器和 PSF 结果解析

---

## 安装

```bash
git clone https://github.com/your-repo/virtuoso-cli.git
cd virtuoso-cli
cargo build --release

# 可选：安装到系统
# cp target/release/virtuoso /usr/local/bin/
```

---

## 快速开始

### 本地模式

```bash
# 1. 在 Virtuoso CIW 中加载 SKILL bridge
#    load("/path/to/ramic_bridge.il")
#    （daemon 会在端口 65432 上自动启动）

# 2. 测试连接
virtuoso tunnel status

# 3. 执行 SKILL
virtuoso skill exec "1+2"
virtuoso skill exec "geGetEditCellView()~>cellName"

# 4. 打开 cellview
virtuoso cell open --lib myLib --cell myCell
```

### 远程模式

```bash
# 1. 初始化配置文件
virtuoso init

# 2. 编辑 .env（见下方配置说明）
#    至少设置 VB_REMOTE_HOST

# 3. 启动 SSH 隧道 + 部署 daemon
virtuoso tunnel start

# 4. 检查状态
virtuoso tunnel status

# 5. 执行 SKILL（与本地相同）
virtuoso skill exec "dbOpenCellViewByType(\"myLib\" \"myCell\" \"layout\" \"r\")"

# 6. 停止隧道
virtuoso tunnel stop
```

---

## 命令参考

```
virtuoso
├── init                              创建 .env 配置模板
├── tunnel                            管理 SSH 隧道
│   ├── start [--timeout N] [--dry-run]   启动隧道 + 部署 daemon
│   ├── stop [--force] [--dry-run]        停止隧道
│   ├── restart [--timeout N]             重启隧道
│   └── status                            检查连接状态
├── skill                             执行 SKILL 代码
│   ├── exec <code> [--timeout N]         执行 SKILL 表达式
│   └── load <file>                       上传并加载 .il 文件
├── cell                              管理 cellview
│   ├── open --lib L --cell C [--view V] [--mode M] [--dry-run]
│   ├── save                              保存当前 cellview
│   ├── close                             关闭当前 cellview
│   └── info                              查看当前 cellview 信息
└── schema [--all] [noun] [verb]      输出命令 schema（供 Agent 发现）
```

### 全局参数

| 参数 | 说明 |
|------|------|
| `--format json\|table` | 输出格式（TTY 默认 table，管道默认 json） |
| `--no-color` | 禁用彩色输出 |
| `--quiet` / `-q` | 静默模式 |
| `--verbose` / `-v` | 调试日志 |

### 退出码

| 退出码 | 含义 |
|--------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 参数/用法错误 |
| 3 | 资源未找到 |
| 5 | 冲突（如 .env 已存在） |
| 10 | dry-run 通过 |

---

## 配置

运行 `virtuoso init` 生成 `.env` 配置模板。所有配置通过环境变量或 `.env` 文件设置。

### 配置变量

| 变量 | 默认值 | 必填 | 说明 |
|------|--------|------|------|
| `VB_REMOTE_HOST` | - | **是**（远程模式） | SSH 远程主机名或别名（来自 `~/.ssh/config`） |
| `VB_REMOTE_USER` | 当前用户 | 否 | SSH 登录用户名 |
| `VB_PORT` | `65432` | 否 | daemon TCP 端口（1-65535） |
| `VB_JUMP_HOST` | - | 否 | 跳板机/堡垒机地址 |
| `VB_JUMP_USER` | - | 否 | 跳板机用户名 |
| `VB_TIMEOUT` | `30` | 否 | 连接/执行超时（秒） |
| `VB_KEEP_REMOTE_FILES` | `false` | 否 | 停止时是否保留远程部署文件 |
| `VB_SPECTRE_CMD` | `spectre` | 否 | Spectre 可执行文件路径 |
| `VB_SPECTRE_ARGS` | - | 否 | Spectre 额外参数（支持 shell 引号语法） |

### 配置示例

**最简配置**（直连局域网服务器）：

```env
VB_REMOTE_HOST=my-eda-server
```

**完整配置**（通过跳板机连接）：

```env
VB_REMOTE_HOST=eda-workstation
VB_REMOTE_USER=designer
VB_PORT=65432
VB_JUMP_HOST=bastion.company.com
VB_JUMP_USER=designer
VB_TIMEOUT=60
VB_KEEP_REMOTE_FILES=false
VB_SPECTRE_CMD=spectre
VB_SPECTRE_ARGS=-64 +aps
```

### SSH 配置建议

推荐在 `~/.ssh/config` 中为远程主机配置别名，简化 `VB_REMOTE_HOST` 设置：

```ssh-config
Host eda-workstation
    HostName 10.0.1.100
    User designer
    IdentityFile ~/.ssh/id_ed25519
    ServerAliveInterval 30
    ServerAliveCountMax 3

Host bastion.company.com
    User designer
    IdentityFile ~/.ssh/id_ed25519
```

然后只需设置 `VB_REMOTE_HOST=eda-workstation`。

---

## Agent 集成

Virtuoso CLI 遵循 [Agent CLI Design Guide](https://github.com/Johnixr/agent-cli-guide) 的全部 10 条原则，专为 AI Agent 自动化设计。

### 命令发现

```bash
# Agent 可以通过 schema 命令发现所有可用命令及参数
virtuoso schema --all

# 查看特定命令的参数定义
virtuoso schema tunnel start
```

### 结构化输出

```bash
# 管道模式自动输出 JSON
virtuoso tunnel status | jq '.daemon.responsive'

# 显式指定格式
virtuoso skill exec "1+1" --format json
```

### Dry-Run

```bash
# 预览操作而不实际执行（返回退出码 10）
virtuoso tunnel start --dry-run --format json
virtuoso cell open --lib myLib --cell myCell --dry-run
```

### 错误处理

错误输出包含机器可读的结构化信息：

```json
{
  "error": "connection_failed",
  "message": "connection failed: Connection refused",
  "suggestion": "Run: virtuoso tunnel start",
  "retryable": true
}
```

---

## 工作原理

```
本机                                远程 Virtuoso 服务器
────                                ────────────────────

virtuoso skill exec "1+2"
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

## 构建 Rust Daemon

当远程主机没有 Python 时，CLI 会回退到部署预编译的 Rust daemon：

```bash
# 构建 daemon binary
cargo build --features daemon --release

# 复制到 resources 目录
cp target/release/virtuoso-daemon resources/daemons/virtuoso-daemon-x86_64
# 或 aarch64:
# cp target/release/virtuoso-daemon resources/daemons/virtuoso-daemon-aarch64
```

---

## 许可证

MIT License - 详见 LICENSE 文件
