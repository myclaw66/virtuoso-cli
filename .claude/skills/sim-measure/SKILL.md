---
name: sim-measure
description: Extract waveform measurements from Virtuoso simulation results. Use when measuring voltage, current, gm, gm/Id, bandwidth, settling time, or any simulation metric.
allowed-tools: Bash(*/virtuoso *)
---

# Measure Simulation Results

Extract metrics from completed simulations via `virtuoso sim measure`.

## Usage

```bash
virtuoso sim measure --analysis <TYPE> --expr '<EXPRESSION>' [--expr '<EXPR2>'] --format json
```

## DC operating point measurements

```bash
# Voltage at a node
virtuoso sim measure --analysis dcOp --expr 'value(VDC("/net1"))' --format json

# Drain current
virtuoso sim measure --analysis dcOp --expr 'value(IDC("/NM0/D"))' --format json

# Operating point parameter (gm, gds, vth, etc.)
virtuoso sim measure --analysis dcOp --expr 'value(getData("/NM0:gm" ?result "dcOpInfo"))' --format json
virtuoso sim measure --analysis dcOp --expr 'value(getData("/NM0:ids" ?result "dcOpInfo"))' --format json
virtuoso sim measure --analysis dcOp --expr 'value(getData("/NM0:vth" ?result "dcOpInfo"))' --format json

# Multiple measurements at once
virtuoso sim measure --analysis dcOp \
  --expr 'value(VDC("/net1"))' \
  --expr 'value(IDC("/NM0/D"))' \
  --expr 'value(getData("/NM0:gm" ?result "dcOpInfo"))' \
  --format json
```

## Transient measurements

```bash
# Peak voltage
virtuoso sim measure --analysis tran --expr 'ymax(VT("/OUT"))' --format json

# Minimum voltage
virtuoso sim measure --analysis tran --expr 'ymin(VT("/OUT"))' --format json

# Crossing time (settling)
virtuoso sim measure --analysis tran --expr 'cross(VT("/OUT") 0.6 1 "rising")' --format json

# Value at specific time
virtuoso sim measure --analysis tran --expr 'value(VT("/OUT") 10e-6)' --format json
```

## AC measurements

```bash
# Gain at DC
virtuoso sim measure --analysis ac --expr 'value(dB20(VF("/OUT")) 1)' --format json

# Unity-gain bandwidth
virtuoso sim measure --analysis ac --expr 'cross(dB20(VF("/OUT")) 0 1 "falling")' --format json

# Phase margin
virtuoso sim measure --analysis ac --expr 'value(phase(VF("/OUT")) cross(dB20(VF("/OUT")) 0 1 "falling"))' --format json
```

## Available signal accessors

| Function | Domain | Description |
|----------|--------|-------------|
| `VDC("/node")` | dc | DC voltage |
| `IDC("/inst/term")` | dc | DC current |
| `VT("/node")` | tran | Transient voltage |
| `IT("/inst/term")` | tran | Transient current |
| `VF("/node")` | ac | AC voltage (complex) |
| `IF("/inst/term")` | ac | AC current (complex) |
| `getData("/inst:param" ?result "dcOpInfo")` | dcOp | Operating point param |

## Waveform math functions

| Function | Description |
|----------|-------------|
| `ymax(wave)` | Maximum value |
| `ymin(wave)` | Minimum value |
| `value(wave time)` | Value at time/freq |
| `cross(wave val n dir)` | Time when wave crosses val |
| `dB20(wave)` | 20*log10(magnitude) |
| `phase(wave)` | Phase in degrees |
| `bandwidth(wave level)` | -3dB bandwidth |
