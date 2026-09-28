---
format: aep.planning-md/3
id: story:recorded-log-adapter-domain
kind: story
status: draft
title: The recorded-log adapter has an ESS domain
summary: ess-history-adapter/1 is a new noun with no model; add one and hold recorded.rs to it
relations:
- serves: vision:O2
- depends_on: story:recorded-history-validation
scope:
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: crates/edge/ess-xtask/tests/adapter_model.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/ess_history_adapter_schema.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/recorded.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/recorded/adapter.yaml
- confidence: cited
  path: models/recorded-log-adapter
- confidence: inferred
  path: models/recorded-log-adapter/domains
- confidence: inferred
  path: models/recorded-log-adapter/system.yaml
- confidence: inferred
  path: schemas/ess-history-adapter.schema.json
revision: 4
---
# Story: the recorded-log adapter has an ESS domain

## Outcome

`ess verify conform import-history` reads an adapter document, `ess-history-adapter/1`, that says
for each `ess-history/1` field which JSON pointer in a log line carries it, or that the log does not
carry it (`crates/verify/ess-conformance/src/recorded.rs`, merged in `fe16e2f78`). The document is a
new noun with no ESS domain: the implementor of `story:recorded-history-validation` reported it as
not done ("The adapter format is not specified in `models/` or `schemas/`", 2026-09-28). The
workspace rule is that a new noun gets its ESS domain (`AGENTS.md`, "ESS drives every product
repository").

The adapter gets a domain under `models/` in the way `models/concurrent-history/` specifies
`ess-history/1`, and the Rust types in `recorded.rs` are held to it by a drift test, the pattern of
`crates/edge/ess-xtask/tests/history_model.rs`.

## Acceptance

- `ess specify validate --path models/recorded-log-adapter` reports the domain valid.
- A drift test fails when the Rust adapter type gains or loses a field or an enum value the model
  does not declare, and passes on the merged code.
- A JSON Schema for the adapter under `schemas/` admits exactly the documents `recorded.rs` admits,
  held by a test over a table of admitted and refused adapter documents.

## Scope

Derived 2026-09-28 by `aep:story-scoper`. Every line is **cited** (read from the story or the tree) or
**inferred** (a reading that could be wrong).

- **Primary surface:** `models/recorded-log-adapter` (new ESS domain) — cited, the acceptance names `ess specify validate --path models/recorded-log-adapter`
- **Files:** `crates/verify/ess-conformance/src/recorded.rs:81-176` (`ADAPTER_FORMAT`, `AdapterFormat`, `Absent`, `FieldSource`, `Pointer`, `CompletionSource`, `MappedCompletion`, `Fields`, `Adapter`) — cited
- **Files:** `models/recorded-log-adapter/system.yaml`, `models/recorded-log-adapter/domains/*.yaml` — inferred, following `models/concurrent-history/`
- **Files:** `crates/edge/ess-xtask/tests/adapter_model.rs` (new drift test) — inferred, the file name is a guess
- **Files:** `schemas/ess-history-adapter.schema.json` (new) — inferred, the file name is a guess
- **Files:** `crates/specify/ess-domain/tests/ess_history_adapter_schema.rs` (new, admitted/refused table) — inferred, `jsonschema` is a dev-dependency there (`crates/specify/ess-domain/Cargo.toml:29`)
- **Files:** `Taskfile.yml:147-150` (one more `ess validate --path models/...` line in `check`) — inferred
- **Also likely:** `crates/verify/ess-conformance/tests/fixtures/recorded/adapter.yaml` as a known-good row — inferred
- **Documents:** `website/docs/guides/verify-conformance.md`, `website/docs/reference/cli.md` may link the model — inferred, optional
- **Confidence:** high — only the names of new files are guessed
- **Would collide with:** any unit touching the adapter types in `recorded.rs`, the `check` task's model list in `Taskfile.yml`, or `ess-xtask`'s `history_model.rs`
- **Safety fact:** `history_model.rs`'s `ALLOWED_KEYS` (`:246-251`) refuses `#[serde(untagged)]` (`recorded.rs:101`, `:123`) and `#[serde(default = "absent")]` (`recorded.rs:163`), so the adapter needs its own drift test — inferred
- **Open:** `MappedCompletion.values` holds `concurrent.history.Completion` (`recorded.rs:75`); whether the new domain may refer to another system's type or must redeclare it is not established
