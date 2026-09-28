---
format: aep.planning-md/3
id: review-result:adversary-chc-adapter-pass-1
kind: review-result
status: active
title: Adversary pass 1, concurrent-history wave 5 unit adapter
relations:
- reviews: story:recorded-log-adapter-domain
revision: 1
---
# Adversary pass 1 — story:recorded-log-adapter-domain

Dispatched as `aep:adversary` against `ess-chc-adapter` (uncommitted tree on
`impl/recorded-log-adapter-domain`, base `e1b9468159` plus the coordinator's merge fix), 2026-09-28.
Header and findings verbatim.

unit: story:recorded-log-adapter-domain, uncommitted working tree on impl/recorded-log-adapter-domain (base e1b9468159)
verdict: NEEDS-CHANGE
cases: executed 920→925, red 5
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: none (scratch test dirs and logs were deleted)
needs-coordinator: whether "no behaviour change to import" (decision 3) means the reader must refuse again what the base refused

Cases (tests/recorded_adapter_adversary.rs), all RED: a tagged pointer is refused; a tagged values
map is refused; a completion value written as a YAML tag is refused; a JSON surrogate-pair escape is
the same document as the raw character; a numeric completion word the adapter maps is mapped. Plus 7
table rows (pointer without a leading slash for seven sources), green, guarding the prefix list.

Suite: ess-conformance 920 passed, 5 failed (the five new); ess-domain, ess-xtask adapter tests and
the ess-cli import tests green.

Coordinator routing: all `introduced` → same implementor. Decided: refuse YAML tags; integer
completion words map integer log values; `{`-leading documents parse as JSON; the prefix drift check
names every pointer field.

```findings
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 155
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "Reading a source's mapping in place drops YAML custom tags, so `pointer: !x /a`, `values: !x {…}` and `ok: !Returned` are now admitted where the base untagged reader refused them, contrary to decision 3 and the reader's own 'no other spelling' doc."
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 155
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The in-place reader coerces YAML integer map keys, so `values: {200: Returned}` (refused at base) is admitted as \"200\" and every log line whose status is the number 200 is then refused as '200 is not a word the adapter's `values` maps'."
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 442
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "A JSON adapter spelling a non-BMP character as an RFC 8259 surrogate-pair escape is admitted by schemas/ess-history-adapter.schema.json and refused by the reader, which parses JSON through serde_yaml."
- file: crates/edge/ess-xtask/tests/adapter_model.rs
  line: 678
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The prefix drift check passes if `.starts_with('/')` appears anywhere in recorded.rs and the table covered only client and completion, so dropping any of seven sources from the checked pointer list was invisible until the seven added table rows."
```
