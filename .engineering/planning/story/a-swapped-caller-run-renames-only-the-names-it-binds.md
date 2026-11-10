---
format: aep.planning-md/3
id: story:a-swapped-caller-run-renames-only-the-names-it-binds
kind: story
status: active
title: A swapped-caller run renames only the instance and instant names it binds
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#766
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/singleton.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/caller_swap_authored_field_names.rs
- confidence: inferred
  path: docs/design/caller-values.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:41:59Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:42:00Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

On a model whose actors carry caller attributes, synthesis appends a second run of each
caller-reading scenario with the callers swapped, and renames the instance and instant names that
run binds with the suffix `-swapped` so they bind apart from the first run's. The rename walks the
serialized steps and suffixes every string found under any JSON key named `instance` or `instant`
(`rename`, `crates/verify/ess-conformance/src/synthesize/caller.rs:3000-3015`, called at line 1390).
Authored data is serialized into the same tree under its authored field names, so an event field,
error field, caller attribute or struct member that happens to be named `instance` or `instant` is
rewritten too. The command input it was copied from is a `{kind: literal, value: ...}` object, not a
string, so it keeps its value, and the swapped run then expects a payload the specification does not
oblige. The interpreter, which is correct, fails the scenario.

Minimal reproduction (0.92.0), `order.yaml`:

```yaml
format: ess/16
system: example
version: v1
summary: An event field named instance, on a model whose commands read the caller.
domain: example.order
types:
  - {name: example.order.OrderId, kind: newtype, of: Uuid}
  - {name: example.order.AccountId, kind: newtype, of: Uuid}
entities:
  - name: example.order.Order
    identity: {name: order_id, type: example.order.OrderId}
    fields:
      - {name: account_id, type: example.order.AccountId}
      - {name: instance, type: String}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
actors:
  - name: example.order.AccountUser
    attributes:
      - {name: account_id, type: example.order.AccountId}
    may: [example.order.PlaceOrder]
commands:
  - name: example.order.PlaceOrder
    input:
      - {name: instance, type: String}
    outcomes:
      - name: placed
        creates: example.order.Order
        instance: order_id
        sets: {account_id: {caller: account_id}, instance: input.instance}
        emits: [example.order.OrderPlaced]
        payload:
          example.order.OrderPlaced: {order_id: {generated: true}, account_id: {caller: account_id}, instance: input.instance}
events:
  - name: example.order.OrderPlaced
    fields:
      - {name: order_id, type: example.order.OrderId}
      - {name: account_id, type: example.order.AccountId}
      - {name: instance, type: String}
views:
  - name: example.order.OrderDetails
    source: example.order.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: example.order.OrderId}
      - {name: account_id, type: example.order.AccountId}
      - {name: instance, type: String}
      - {name: state, type: example.order.Order.State}
```

```console
$ ess verify conform run --path order.yaml --target interpreted --report-format 2
example v1 against interpreted 0.92.0 — failed
  failed example.order.PlaceOrder/outcome/placed
  1 scenarios: 0 passed, 1 failed, 0 error, 0 unsupported
```

The synthesized suite shows the cause: the swapped run sends `"instance": {"kind": "literal",
"value": "instance"}` and expects the event payload `"instance": "instance-swapped"`. The same
specification with the field named `instant` fails the same way; with the field named `label` it
passes (`1 passed`).

Two sibling walks read authored data as suite structure in the same way and are part of this need:

- `rename(value, from, to)` in `crates/verify/ess-conformance/src/synthesize/singleton.rs:404-422`
  replaces any string under a key `instance` that equals the instance name being replaced;
- `observed` and `replace_observed` in `crates/verify/ess-conformance/src/synthesize/caller.rs:1892-1933`
  treat any object with `kind: observed` as an observed reference, including a struct literal whose
  own member `kind` holds the text `observed`.

## Acceptance

- `ess verify conform run --path order.yaml --target interpreted --report-format 2` on the
  specification above exits 0 and prints `1 scenarios: 1 passed, 0 failed, 0 error, 0 unsupported`;
  the same holds with the field, input and payload key renamed to `instant`.
- In `ess verify conform synthesize --path order.yaml --format json`, the second `expect_event`
  step of `example.order.PlaceOrder/outcome/placed` carries, under `payload.instance`, exactly the
  value its preceding `execute_command` step sends as `input.instance.value`, and no value under an
  authored field name ends in `-swapped`; the step `capture_instance` still binds
  `observed-1-swapped` and the view expectation still reads `{kind: instance, instance: observed-1-swapped}`.
- A test in `ess-conformance` synthesizes a caller-attribute model with a field named `instance` in
  each authored position (event payload field, error field, caller attribute, a member of a struct
  input, and a struct whose members are `kind` and `instance`) and asserts every scenario passes
  against the interpreter; the test fails on 0.92.0.
- A unit test of the singleton rename shows an authored String value equal to the replaced instance
  name, held under an authored field named `instance`, is left unchanged.
- No committed suite or generated tree changes bytes: the existing byte-comparison tests of
  `ess-conformance` and `ess-synth` pass unchanged.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/caller.rs (the swapped-run `rename` at 3000-3015, called at 1390; `observed` and `replace_observed` at 1891-1933, called at 1837, 1843 and 1847 inside `recaptured` at 1831; its `mod tests` at 3022)
- cited: crates/verify/ess-conformance/src/synthesize/singleton.rs (`rename` at 403-422, called at 260; no `mod tests` yet, so the singleton unit test is a new module in this file)
- inferred: crates/verify/ess-conformance/tests/caller_swap_authored_field_names.rs (new)
- inferred: docs/design/caller-values.md (line 138 already states the fixed rule; at most one sentence added)

confidence: high — the story names the defect site for all three walks and each was found where it says; `synthesize.rs` is not touched (the three walks are private, reached only through `caller::synthesize` and `singleton::withdraw_second_creations`) (story-scoper, 2026-11-10)

Coordinator decision, 2026-11-10: the typed walk over the scenario steps is written in `caller.rs` (an exhaustive `match` over the step and value variants there), not as a method in `crates/verify/ess-conformance/src/scenario.rs`, which `story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs` edits in the same turn (`SUPPORTED_SUITE_FORMATS`); the scoper found either placement works. `scenario.rs` (`InstanceName` at 1547, `InstantName` at 1635) is read only.

Shared surfaces this story moves (epic § Shared surfaces, not scope entries): `CHANGELOG.md`; the byte-pin tables must stay unchanged.

## Fit review

1. **The need.** A behaviour ESS gets wrong: a synthesized suite expects a value the specification
   does not oblige when an authored field is named `instance` or `instant` on a caller-attribute
   model, so a conforming target fails. Reproduction above; `ess verify conform run` on it prints
   `0 passed, 1 failed` with 0.92.0. The requester's ask, labelled as theirs: "the swap renames the
   caller identity it swaps, by path, not every string under any key named `instance`". No syntax is
   proposed.
2. **Class: defect.** The suite's own value encoding is tagged precisely because "a declared struct
   may perfectly well have a field called `instance`"
   (`crates/verify/ess-conformance/src/scenario.rs:1837-1839`), and the design says the second run
   renames "every instance and instant it binds" (`docs/design/caller-values.md:138`), not authored
   values. The rename at `caller.rs:3000-3015` contradicts both.
3. **Already expressible?** Only by renaming the authored field (the reporter's workaround), which
   forces a domain name to change for a suite-internal reason; `ess specify validate --path
   order.yaml` answers `example v1 — 1 file(s), valid`, so the language admits the name and the
   suite must check it. That is no idiom.
4. **Fit.** No authored construct, key or value is added or refused, so no cell of grid R, grid O or
   the effect × target grid moves (`docs/design/language-grids.md`). The fix is in synthesis only:
   instance and instant names are renamed at the typed positions that hold them (`InstanceName` in
   `EstablishEntity`, `CaptureInstance`, `ScenarioValue::Instance`; `InstantName` in `MarkInstant`,
   `ExpectNotBefore`, `ExpectWithin`, `ExpectQuiet`; `crates/verify/ess-conformance/src/scenario.rs:2244`
   onward), never inside a `Node` or a map keyed by authored field names. It is general across
   siblings: the singleton rename and the observed-reference walk get the same treatment. Every
   target is unaffected in code: Rust, Go, TypeScript runners and the interpreter read the suite
   unchanged; only the bytes synthesis emits for an affected model move.
5. **A second adopter.** A deployment model with an event `example.host.DiskRequested` carrying
   `instance: String` (the name of a machine instance), or a scheduling model with a field
   `instant: Timestamp`-like text. Both are ordinary domain nouns.
6. **Cost.** No source format, no suite format version, no keyword, no diagnostic, no migration, no
   diff classification. Suite bytes change only for a model with an authored `instance` or `instant`
   name in a swapped run, which today is a failing suite; a search of `examples/`, `models/` and the
   crate's test fixtures for `name: instance` and `name: instant` finds none, so no committed bytes
   move.
7. **Alternatives.** (a) Change nothing and document the rename: refused, the language admits the
   name and a conforming target fails. (b) Refuse the field names `instance` and `instant` at
   validation: refused, it adds a reserved word and a migration for a suite-internal detail. (c) Keep
   the JSON walk and skip known authored subtrees (`payload`, `caller`, `value`, `fields`,
   `context`): works for today's steps but silently regresses when a step gains a new authored map.
   (d) Chosen: rename at the typed positions, so a new step field holding authored data is safe by
   construction. This is the requester's ask, taken as stated.

## Decisions

accept as proposed. The swapped-caller run renames only the instance and instant names it binds, at
the typed positions of the scenario steps; authored values under any field name are left as sent.
The same rule is applied to the singleton instance rename and the observed-reference rewrite. No
format version changes.

## Source

github:beyond10x/ess#766
