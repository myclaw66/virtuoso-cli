# Simulation Automation Module Design

## Motivation

Manual ADE-L workflow for analog simulation involves repetitive clicking: setting up testbench, changing parameters, running, extracting waveform metrics, and recording results. This is especially painful for parameter sweeps and PVT corner analysis, where the same sequence must be repeated dozens of times.

## Approach: Ocean Script via SKILL Bridge

Generate Ocean SKILL code and execute it through the existing virtuoso-cli bridge. Ocean API functions (`simulator`, `design`, `analysis`, `run`, `openResults`, `getData`, `value`, `cross`, `ymax`, `ymin`) are all verified available in the target environment (IC23.1).

This keeps full compatibility with existing ADE setups — same simulator, same models, same results directory structure.

## Command Tree

```
virtuoso sim
├── setup --lib L --cell C [--simulator spectre]
│   Set simulator and design target. Validates cell exists.
│
├── run --analysis tran|dc|ac [--stop T] [--from F] [--to T] [--step S] [--params K=V...]
│   Execute a single simulation with specified analysis type.
│
├── sweep --var X --from A --to B --step S --measure EXPR [--format json|csv|table]
│   Parameter sweep: iterate variable, run sim at each point, extract measure.
│
├── corner --file corners.json [--format json|csv|table]
│   PVT corner matrix: run all corners, extract all measures, output summary table.
│
├── measure --expr EXPR [--expr EXPR...] [--format json]
│   Extract waveform metrics from last simulation results.
│
└── results [--format json|csv]
    List or export results from the results directory.
```

## Typical Workflows

### Quick single simulation

```bash
virtuoso sim setup --lib FT0001A_SH --cell Bandgap_LDO
virtuoso sim run --analysis tran --stop 10u
virtuoso sim measure --expr 'ymax(VT("/OUT"))' --expr 'cross(VT("/OUT") 0.6 1 "rising")'
```

### Parameter sweep

```bash
virtuoso sim setup --lib FT0001A_SH --cell Bandgap_LDO
virtuoso sim sweep --var "W" --from 1u --to 10u --step 1u \
  --measure 'ymax(VT("/OUT"))' --format table
```

Output:
```
W        ymax(VT("/OUT"))
1.0u     1.042
2.0u     1.098
...
```

### PVT corner analysis

```bash
virtuoso sim corner --file corners.json --format table
```

Output:
```
Corner    Temp    VDD    Vout_max    Vout_settle
tt        27      1.20   1.183       2.31ns
ss_hot    125     1.08   1.072       4.87ns
ff_cold   -40     1.32   1.298       1.56ns
```

## Corner Configuration File

```json
{
  "simulator": "spectre",
  "design": {
    "lib": "FT0001A_SH",
    "cell": "Bandgap_LDO",
    "view": "schematic"
  },
  "model_file": "/path/to/models/spectre/all.scs",
  "analysis": {
    "type": "tran",
    "stop": "10u"
  },
  "corners": [
    {"name": "tt",      "section": "tt", "temp": 27,   "vdd": 1.20},
    {"name": "ss_hot",  "section": "ss", "temp": 125,  "vdd": 1.08},
    {"name": "ff_cold", "section": "ff", "temp": -40,  "vdd": 1.32}
  ],
  "measures": [
    {"name": "Vout_max",     "expr": "ymax(VT(\"/OUT\"))"},
    {"name": "Vout_settle",  "expr": "cross(VT(\"/OUT\") 0.6 1 \"rising\")"}
  ]
}
```

## Implementation Architecture

### New Files

| File | Responsibility |
|------|---------------|
| `src/commands/sim.rs` | CLI entry point, argument parsing, dispatches to ocean module |
| `src/ocean/mod.rs` | Ocean SKILL code generator, orchestrator |
| `src/ocean/analysis.rs` | Analysis type configs (tran/dc/ac/stb params) |
| `src/ocean/sweep.rs` | Parameter sweep + corner matrix iteration |
| `src/ocean/measure.rs` | Waveform measurement expression builder |
| `src/ocean/results.rs` | Result parsing + JSON/CSV/table output |

### Data Flow

```
CLI command
    │
    ▼
Generate Ocean SKILL code (src/ocean/*.rs)
    │
    ▼
Execute via VirtuosoClient::execute_skill()  (existing bridge)
    │
    ▼
Parse SKILL return value → serde_json::Value
    │
    ▼
Format output (JSON / CSV / table)
```

### Ocean SKILL Generation Examples

**`sim setup`** generates:
```skill
simulator('spectre)
design("FT0001A_SH" "Bandgap_LDO" "schematic")
```

**`sim run --analysis tran --stop 10u`** generates:
```skill
analysis('tran ?stop "10u")
run()
```

**`sim measure --expr 'ymax(VT("/OUT"))'`** generates:
```skill
selectResult('tran)
ymax(VT("/OUT"))
```

**`sim corner`** generates a loop:
```skill
foreach(corner '(("tt" 27 1.2) ("ss" 125 1.08) ("ff" -40 1.32))
  modelFile('("/path/models" "") 'section car(corner))
  temp(cadr(corner))
  desVar("vdd" caddr(corner))
  run()
  ;; extract measures
)
```

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Cell not found | Error at setup with `exit_code=3`, suggestion: check lib/cell name |
| Convergence failure | Return `"status": "convergence_error"` with key log lines |
| Wave not found | Return missing net name + available nets as suggestion |
| Model file missing | Validate at setup, fail early |
| Corner JSON invalid | Schema validation with specific error location |
| Simulation timeout | Existing timeout mechanism (try_wait loop + kill) |

## Output Format

All commands follow the existing CLI conventions:
- `--format json` (default in pipe) — structured JSON to stdout
- `--format table` (default in TTY) — human-readable table
- `--format csv` — for spreadsheet import
- Errors as structured JSON to stderr (with `suggestion` and `retryable` fields)

## Environment Verification

Verified on IC23.1 (Apr 2026):

| API | Available |
|-----|-----------|
| `simulator('spectre)` | Yes |
| `design(lib cell view)` | Yes |
| `analysis('tran ...)` | Yes |
| `run()` | Yes |
| `modelFile(...)` | Yes |
| `desVar(...)` | Yes |
| `temp(...)` | Yes |
| `openResults(...)` | Yes |
| `getData(...)` | Yes |
| `value(...)` | Yes |
| `cross(...)` | Yes |
| `ymax(...)` / `ymin(...)` | Yes |
| `selectResult(...)` | Yes |
| `ocnxlRun(...)` | Yes |
