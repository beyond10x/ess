---
format: aep.planning-md/3
id: review-result:recorded-history-validation-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, story:recorded-history-validation
relations:
- reviews: story:recorded-history-validation
revision: 1
---
# Adversary pass 2 — story:recorded-history-validation

Dispatched as `aep:adversary` after correction round 1, 2026-09-28. Header and findings verbatim.

unit: story:recorded-history-validation, uncommitted working tree on impl/recorded-history-validation (base c589fddb0)
verdict: NEEDS-CHANGE
cases: executed 1649→1655, red 6
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 root, the assigned scratch (all cleaned), plus the assigned build dir
needs-coordinator: whether `--output` should also refuse the `--path` specification files (F4). This is the same kind of clash the pass-1 correction fixed for the log and the adapter.

Cases: tests/recorded_adversary_2.rs — a duplicate identity on three lines is refused naming all
three (RED); an upper-case identity is refused by the importer as a malformed field (RED).
crates/edge/ess-cli/tests/import_history_adversary_2.rs — an output hard-linked to the log is refused
(RED); a gaps file hard-linked to the adapter is refused (RED); an output naming the specification is
refused (RED); an unwritable gaps file leaves no history written (RED).

Suites: ess-conformance 889 passed, 2 failed; ess-cli 760 passed, 4 failed (the six new).

Not broken: symlinks and `..` paths refused; existing directory refused; byte-identical output across
runs; line mapping after CRLF and blank lines; LostUpdate exit 1 and clean exit 0 through the CLI;
`rows` of the wrong type refused; the implementor's own test edit not weakened.

Coordinator routing: all five `introduced` → correction round 2. Decided: `--output` may not name any
file of the `--path` specification; the history and gaps file are written as a pair.

```findings
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 736
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "a duplicate identity is refused at its second occurrence, so a third line carrying it is never named, contrary to the correction's 'naming every line'"
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 499
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "an upper-case operation id passes the importer's check and is refused later as history.non-canonical-uuid, not as import.field-malformed as the correction states"
- file: crates/edge/ess-cli/src/main.rs
  line: 3134
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "same_file compares canonical paths, so an --output or gaps file hard-linked to --log or --adapter overwrites it with exit 0"
- file: crates/edge/ess-cli/src/main.rs
  line: 3152
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "--output naming a file of the --path specification overwrites the specification with the history and exits 0"
- file: crates/edge/ess-cli/src/main.rs
  line: 3189
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "the history is written before its gaps file, so a failed gaps write exits 2 with the history already replaced and no gaps record beside it"
```
