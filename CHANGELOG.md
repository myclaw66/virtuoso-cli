# Changelog

All notable changes to this project will be documented in this file.

## [0.1.5] - 2026-04-15

### Added
- **`Orient` enum** for schematic instance orientation — type-safe replacement for `String`, derives `clap::ValueEnum` + `serde::Deserialize` so both CLI (`--orient`) and JSON spec (`build --spec`) reject invalid values at the boundary. Accepts exactly the 8 Cadence orientations: R0, R90, R180, R270, MX, MY, MXR90, MYR90
- **`maestro add-output` now resolves setup name from session internally** — previously passed session ID as SKILL output name and user name as setup name, causing `maeAddOutput` to always return nil

### Fixed
- **`sim::job_list` no longer uses `unwrap_or_default()`** — propagates serialization errors via `VirtuosoError::Execution` per project convention
- **`parse_skill_json` returns `Result<Value>`** instead of silently falling back to `{"raw": output}` — surfaces SKILL output corruption instead of masking it; all call sites updated to propagate the error
- **`cv_guard` injection in `schematic_ops.rs`** — every schematic operation now validates the `RB_SCH_CV` global SKILL variable is bound before use, surfacing clear errors instead of cryptic SKILL failures

### Refactored
- **`main.rs` dispatcher extraction** — 239-line central match reduced to 12 lines by extracting 9 `dispatch_*` functions (one per command group)
- **`measure` expression validation** — new `validate_measure_expr` blocks destructive SKILL calls (`system`, `ipcBeginProcess`, `deleteFile`, `load`, `evalstring`, …) before execution

## [0.1.4] - 2026-04-15

### Added
- **`vcli window` subcommand group** — `list`, `dismiss-dialog`, `screenshot`
  - `list`: enumerate all open Virtuoso windows with derived mode labels (`ade-editing`, `ade-reading`, `schematic`, `layout`, `other`); handles SKILL octal escapes (`\256` = ®) that break standard JSON parsers
  - `dismiss-dialog [--action ok|cancel] [--dry-run]`: programmatically cancel or confirm a blocking GUI dialog
  - `screenshot --path FILE [--window PATTERN]`: capture via X11 ImageMagick `import -window root` (IC23.1 fallback — `hiGetWindowScreenDump` is IC25+ only)
- **`vcli maestro set-analysis`** — enable an analysis type (ac/dc/tran/noise/…) on a setup by session name; resolves setup internally via `maeGetSetup`

### Fixed
- **`maestro add-output`** — parameter order was completely wrong: session ID was passed as SKILL output name and user-supplied name as setup name, causing `maeAddOutput` to always return nil; now resolves setup from session automatically
- **`maestro get-analyses`** — `maeGetEnabledAnalysis` takes a positional setup name (not `?session` keyword) in IC23.1; setup name is now resolved via `maeGetSetup` internally
- **`--session` global arg no longer clobbers `VB_SESSION`** — bridge session ID and Maestro session name can coexist without conflict

## [0.1.3] - 2026-04-15

### Fixed
- **format tracing::debug line in bridge.rs** — fix log formatting issue
- **maestro: align SKILL function signatures with IC25.1 official documentation** — fixes Maestro operations compatibility

### Added
- **New skills** — `circuit-optimizer`, `sim-plot`, `schematic-gen`, `spectre-netlist-gotchas` — see [.claude/skills/](.claude/skills/)
- **Maestro skill** and Virtuoso reference documentation

### Dependencies
- Updated various dependencies for stability

## [0.1.2] - 2026-04-13

### Added
- **Interactive TUI Dashboard** — `vtui` binary with Sessions/Jobs/Config tabs
- **Remote Session Auto-Discovery** — `vcli tunnel start` syncs remote sessions
- **Remote Async Spectre Simulation** — `vcli sim run-async` works via SSH nohup
- **SSH Configuration** — `VB_SSH_PORT`, `VB_SSH_KEY` support
- **IC23.1+ Maestro Explorer Support** — `vcli maestro` commands

## [0.1.1] - Previous release

See [release history](https://github.com/deanyou/virtuoso-cli/releases)
