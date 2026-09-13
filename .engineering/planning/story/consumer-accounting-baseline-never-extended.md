---
format: aep.planning-md/1
id: story:consumer-accounting-baseline-never-extended
kind: story
status: draft
title: The consumer accounting baseline was never extended for the thirteen gaps
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/enforce.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/initial-baseline.json
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/mod.rs
- confidence: inferred
  path: docs/design/cli-schema-metadata-accounting.md
revision: 3
---
# The consumer accounting baseline was never extended for the thirteen gaps

`task consumer-check` refuses on `main`, and has since `9572af9b` closed thirteen measured gaps as
one source change on 2026-09-11. Waves 22 to 25 each recorded it as *known red at base* and each
left it there.

`777ffc87` cleared the first three refusal stages: 596 unclassified entries, one classification for
a function wave 25 deleted, 63 stale reviewed cases and ten claimed models that no longer exist. The
fourth stage is what is left, and it is not the same kind of thing.

## What it refuses with

```
{"BaselineUnknown":147931,"Refused":0,"Supported":0,"accounting_complete":false,
 "bound_profiles":90,"discovered_models":2421,"executed_cases":0,"stage":"execution-plan"}
```

Every new model is unaccounted against every profile that could consume it. The list names each
pair: `unaccounted wire:RawSpecFile#/definitions/SelectionMapping/type synthesis-go-emission`, and
147,930 more.

## Why it is not bookkeeping

`initial-baseline.json` is byte-pinned — `consumer_coverage/mod.rs:417` refuses any content whose
SHA256 is not `3dd8dff59335c8a77c93c2734118566fd1b2d5165c0590d0aa9be397374a47de`. A cell in it is a
claim about what one consumer does with one model. Filling 147,931 of them by rule would assert
support that nobody established, which is the one thing this lane exists to prevent.

## What this costs today

CI does not run it: `.github/workflows` invokes `task check SKIP_CONSUMER_CHECKS=true`, disabled by
explicit operator request, and `AGENTS.md` records that. So the lane is red only locally, and
`AGENTS.md`'s own release rule — *"Before pushing a release tag, run `task check`"* — cannot be
satisfied as written while it is.

## Acceptance

`task consumer-check` exits 0 on `main`, with every admitted cell carrying a reviewed basis rather
than a generated one — or the lane's obligation is restated so that a model nobody has reviewed is
refused individually rather than making the whole accounting incomplete.

## Scope

- `crates/edge/ess-xtask/src/consumer_coverage/initial-baseline.json` — `cited`, byte-pinned
- `crates/edge/ess-xtask/src/consumer_coverage/mod.rs` — `cited`, holds the pin at `:417`
- `crates/edge/ess-xtask/src/consumer_coverage/enforce.rs` — `cited`, decides accounting completeness
- `docs/design/cli-schema-metadata-accounting.md` — `inferred`, the decision record this would extend
