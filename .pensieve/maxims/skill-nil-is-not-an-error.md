---
id: skill-nil-is-not-an-error
type: maxim
title: SKILL nil return is not a transport error
status: active
created: 2026-04-07
tags: [maxim, skill, error-handling]
---

# SKILL nil return is not a transport error

## Statement

`VirtuosoResult::ok()` reflects the bridge transport status (STX vs NAK), not the SKILL
function's return value. A SKILL function that returns `nil` on failure still produces
`ok() == true`.

## Consequence

Guards of the form `if !result.ok() { return Err(...) }` do **not** catch SKILL-level
failures. They only catch daemon errors and timeouts.

## Fix Pattern

```rust
let result = client.execute_skill("design(...)", None)?;
if !result.ok() || result.output.trim() == "nil" {
    return Err(VirtuosoError::NotFound("...".into()));
}
```

## When It Matters

Any `execute_skill` call where the return value signals success/failure:
- `design()` → nil if cell not found
- `dbOpenCellViewByType()` → nil if cell not found
- `getData()` → nil if signal not found (already checked via parse in process.rs)

## Discovered

Code review of `src/commands/process.rs:28` — `char()` guard silently passes when
lib/cell/view doesn't exist, allowing `desVar()` calls to mutate the wrong design.
