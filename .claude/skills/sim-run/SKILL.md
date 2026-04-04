---
name: sim-run
description: Run circuit simulation (DC, tran, AC) on Virtuoso. Use when executing Spectre simulation, running analysis, or checking simulation results.
allowed-tools: Bash(*/virtuoso *)
---

# Run Simulation

Execute Spectre simulation via `virtuoso sim run`.

## Prerequisites

Simulation must be set up first (see `/sim-setup`):
- `simulator('spectre)` configured
- `design(lib cell view)` set
- `modelFile(...)` configured
- `desVar(...)` set for any parameterized variables
- `resultsDir(...)` set

## Run analysis

```bash
# DC operating point
virtuoso sim run --analysis dc --param saveOppoint=t --timeout 120 --format json

# Transient
virtuoso sim run --analysis tran --stop 10u --timeout 300 --format json

# AC
virtuoso sim run --analysis ac --start 1 --stop 1e9 --dec 10 --timeout 300 --format json
```

## Important: resultsDir

`sim run` auto-creates a resultsDir if none is set, but **always set it explicitly before first run**:

```bash
virtuoso skill exec 'resultsDir("/tmp/my_sim")' --format json
```

If `sim setup` shows `results_dir: "spectre"` (relative path), the dir is not properly configured — set it explicitly.

## Verify success

After `run()`, check spectre.out for errors:

```bash
# Check the log
virtuoso skill exec 'resultsDir()' --format json
# Then read the spectre.out file at <resultsDir>/psf/spectre.out

# A successful run has PSF data files (dcOp.dc, tran.tran, etc.)
# A failed run only has artistLogFile, simRunData, variables_file
```

## Key indicators

| run() output | Meaning |
|-------------|---------|
| Returns resultsDir path | Simulation completed |
| Returns nil, takes <0.01s | Analysis not configured or session lost |
| Returns nil, takes >0.1s | Netlisting ran but spectre may have failed |

## Common errors in spectre.out

| Error | Fix |
|-------|-----|
| SFE-868: Cannot open input file | Model path wrong — verify file exists |
| SFE-675: no valid section name | Empty section `""` in modelFile for .lib — remove it |
| SFE-1997: parameter not assigned | Set `desVar()` for the missing parameter |
| OSSHNL-116: Cannot descend into views | Subcell missing spectre view — remove instance or add view |
