# virtuoso-cli — 开发指南

供 Claude Code 使用的项目开发指引。用户文档见 [README.md](README.md)。

## Build

```bash
cargo build                              # 开发构建 (产物: target/debug/vcli)
cargo build --features daemon            # 同时编译 virtuoso-daemon
cargo test                               # 测试
cargo clippy                             # 检查
```

**两个二进制产物：**
- `vcli` — 主 CLI (`src/main.rs`)
- `virtuoso-daemon` — TCP 监听守护进程，由 `ramic_bridge.il` 启动 (`src/daemon/main.rs`)

## 源码结构

```
src/
├── main.rs               # CLI 入口 (clap)，命令路由
├── daemon/main.rs        # TCP daemon，接收 SKILL 命令并执行
├── client/
│   ├── bridge.rs         # VirtuosoClient — 连接、execute_skill()、会话路由
│   ├── editor.rs         # Layout/Schematic editor traits
│   ├── layout_ops.rs     # Layout 操作
│   └── schematic_ops.rs  # Schematic 操作
├── commands/
│   ├── cell.rs           # virtuoso cell open/save/close/info
│   ├── design.rs         # virtuoso design size/explore (gm/Id 查表)
│   ├── init.rs           # virtuoso init (.env 模板)
│   ├── process.rs        # virtuoso process char (gm/Id lookup 生成)
│   ├── schema.rs         # virtuoso schema (输出 JSON 命令文档)
│   ├── session.rs        # virtuoso session list/show
│   ├── sim.rs            # virtuoso sim run/measure/sweep/corner
│   ├── skill.rs          # virtuoso skill exec/load
│   └── tunnel.rs         # virtuoso tunnel start/stop/status
├── config.rs             # Config::from_env()，读取 .env / 环境变量
├── error.rs              # VirtuosoError (thiserror)，exit_code 映射
├── models.rs             # SessionInfo, VirtuosoResult, TunnelState
├── output.rs             # OutputFormat, print_json, CliError
├── ocean/                # Ocean script 生成 (AC/DC/Tran 分析)
├── spectre/              # 直接调用 spectre 的 runner + PSF 解析
└── transport/            # SSH 隧道 (tunnel.rs, ssh.rs)

resources/
└── ramic_bridge.il       # Cadence SKILL bridge — 在 Virtuoso CIW 中 load
process_data/             # PDK gm/Id lookup 数据 (JSON)
```

## 关键架构

### 通信协议

```
Virtuoso CIW
  → load ramic_bridge.il → RBStart()
  → 启动 virtuoso-daemon (ipcBeginProcess)
  → daemon 绑定端口，打印 "PORT:N" 到 stderr
  → ramic_bridge.il 写 session 文件 (~/.cache/virtuoso_bridge/sessions/<id>.json)

vcli → 读 session 文件 → TCP connect → 发 JSON {"skill": "...", "timeout": N}
     → daemon 执行 SKILL → 返回 STX+结果 或 NAK+错误
```

**协议字节：**
- `STX (0x02)` — 成功响应，payload = SKILL 返回值字符串
- `NAK (0x15)` — 错误响应，payload = 错误信息

**重要陷阱：** `VirtuosoResult::ok()` 只检查传输层状态 (STX vs NAK)。SKILL 函数返回 `nil`（如 `design()` 找不到 cell）仍然返回 STX，`ok()` 为 true。凡涉及 SKILL 调用结果成功与否的地方，需同时检查 `result.output.trim() == "nil"`。

### Session 路由 (VirtuosoClient::from_env)

```
1. VB_SESSION 环境变量 或 --session 参数 → 加载指定 session 文件
2. 未指定 session → 扫描 sessions/ 目录:
   - 恰好 1 个 → 自动选择
   - 多个 → 报错，要求指定 --session
   - 0 个 → 回退到 VB_PORT (向后兼容)
```

### 配置 (src/config.rs)

通过环境变量或 `.env` 文件配置：

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `VB_REMOTE_HOST` | — | 远程 EDA 服务器，留空则本地模式 |
| `VB_REMOTE_USER` | — | SSH 用户名 |
| `VB_PORT` | 65432 | 回退端口 (session 模式下不用) |
| `VB_TIMEOUT` | 30 | 默认超时秒数 |
| `VB_SESSION` | — | 指定 session ID |
| `VB_SPECTRE_CMD` | spectre | spectre 可执行文件路径 |
| `VB_SPECTRE_ARGS` | — | 额外 spectre 参数 |

### gm/Id 查表 (process char)

```bash
# Virtuoso 模式 (需要 Virtuoso 运行)
vcli process char --lib myLib --cell gmid --inst /NM0 --type nmos

# Netlist 模式 (直接调 spectre，无需 Virtuoso)
vcli process char --netlist \
  --type pmos \
  --model-file /path/models.lib --model-section tt \
  --pmos-model p12 \      # PDK 相关，不同厂商不同
  --vdd 1.2 --vds 0.6 \
  --output process_data/smic13mmrf
```

Lookup 数据格式见 `process_data/smic13mmrf/*.json`。
sizing 公式: `W(μm) = Id_needed(A) / id_lookup(A)`，其中 `id_lookup` 是 W=1μm testbench 下的电流。

## 开发规范

### Rust 约定

- `snake_case` 函数/变量，`CamelCase` 类型/trait
- 参数优先 `&str` 而非 `String`；避免不必要堆分配
- 生产代码不用 `unwrap()`，除非已证明不可能失败 (如 u16 → SocketAddr)
- 最小化 `pub`；迭代器优于手动循环
- 导入顺序: `std` → 外部 crate → `crate::`/`super::`
- 100% safe Rust，无 `unsafe`

### 错误处理

- 用 `VirtuosoError` (见 `src/error.rs`)，不用 `anyhow`
- `VirtuosoError::Execution` — SKILL/外部工具运行时错误
- `VirtuosoError::Config` — 配置/参数错误 (exit 2)
- `VirtuosoError::NotFound` — cell/session 不存在 (exit 3)
- 系统边界校验 (用户输入、文件 I/O、外部工具调用)
- EDA 工具调用前检查可用性；长进程加 `--timeout`

### PDK 参数不硬编码

不在主程序 binary 里硬编码任何 PDK 厂商相关参数（设备模型名、电源电压、偏置等）。这些通过 CLI 参数传入，由技能文档 (`.claude/skills/`) 提供各 PDK 的推荐值。

### 代码风格

- **不过度工程化。** 只做当前任务需要的
- **不投机抽象。** 三行相似代码 > 过早的 helper
- **不加无用注释/兼容 shim。** 删除未使用代码
- 遵循 `cargo clippy`

### 安全

- 不提交凭据/license 路径/fab 工艺数据
- 清洗 SKILL 字符串：`escape_skill_string()` (见 `bridge.rs`)
- 无命令注入 — `Command::new()` + 独立参数，不用 shell 拼接
- SKILL 危险命令检查：`check_blocking_skill()` (阻止 `find /` 挂死 daemon)

## 添加新命令

1. `src/commands/` 新增 `xxx.rs`，`pub fn do_thing(...) -> Result<Value>`
2. `src/commands/mod.rs` 加 `pub mod xxx;`
3. `src/main.rs` 加 `Commands::Xxx` variant (clap) 和路由分支
4. 若需要连接 Virtuoso：`VirtuosoClient::from_env()?`

## 重要备忘

- `VirtuosoResult::ok()` ≠ SKILL 调用成功，只是传输层成功。需额外检查 `output != "nil"`
- SKILL 中 `sh()` 返回 `t`/`nil` 而非 stdout；`fprintf` 可能写 0 字节文件
- `ipcBeginProcess` 需要绝对路径
- session 文件位于 `~/.cache/virtuoso_bridge/sessions/<id>.json`
- daemon 绑定 port=0 时由 OS 分配端口，打印 `PORT:N` 到 stderr 供 bridge.il 读取
