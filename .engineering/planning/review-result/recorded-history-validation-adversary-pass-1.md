---
format: aep.planning-md/3
id: review-result:recorded-history-validation-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, story:recorded-history-validation
relations:
- reviews: story:recorded-history-validation
revision: 1
---
# Adversary pass 1 — story:recorded-history-validation

Dispatched as `aep:adversary` against `ess-chc-recorded` (uncommitted tree on
`impl/recorded-history-validation`, base `c589fddb0`), 2026-09-28. Header and findings verbatim.

unit: story:recorded-history-validation, uncommitted working tree on impl/recorded-history-validation (base c589fddb0)
verdict: NEEDS-CHANGE
cases: executed 1641→1648, red 6
origin: introduced 8 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 (both deleted), plus the assigned build dir
needs-coordinator: whether `rows` belongs in the adapter (F2). Also one rule broken: my first isolated run started at 11G free, under the 12G floor.

Cases: tests/recorded_adversary.rs — a numbered identity never collides with a carried one (RED); a
view read whose rows are not imported gets a gap (RED); an out-of-range instant names its line (RED);
an Indeterminate line carrying a return instant names its line (RED); a duplicate carried identity
names both lines (RED); escaped pointers resolve (green). crates/edge/ess-cli/tests/
import_history_adversary.rs — an output naming the log is refused (RED: log overwritten, exit 0).

Suites: ess-conformance 883 passed, 5 failed; ess-cli 759 passed, 1 failed (the six new).

Not broken: LostUpdate exit 1 and clean exit 0 through the CLI; missing returned_at refused per line;
unmapped completion word, null, string or float instant, value above 2^53 refused; pointer escapes;
CRLF and blank lines; client numbering.

Coordinator routing: all `introduced` → same implementor. Decided: fallback ids in a namespace no
writer uses; `rows` declarable in the adapter with a gap when absent; refusals mapped to log lines;
`--output` may not name the log or the adapter; gaps also written beside the output.

```findings
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 325
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the fabricated line-numbered operation id uses the recorder's own spelling, so a log carrying 00000000-0000-4000-8000-000000000002 on one line and no id on line 2 is refused as history.duplicate-operation"
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 110
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the adapter cannot declare rows and import reports no rows coverage gap, so every imported view read silently arrives unjudged even when the log carries its rows"
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 546
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "refusals from history::read (integer-out-of-range, indeterminate-with-return-instant, duplicate-operation) name operations[i] or an operation id instead of the log line the brief requires"
- file: crates/edge/ess-cli/src/main.rs
  line: 3124
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "import-history with --output equal to --log overwrites the recorded log with the history and exits 0"
- file: crates/verify/ess-conformance/src/recorded.rs
  line: 540
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "seed 0 and line-numbered operation ids are written into the persisted document indistinguishable from carried values; the gap survives only on stderr"
- file: crates/verify/ess-conformance/tests/recorded_history.rs
  line: 153
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the fixture's carried operation ids equal the numbered fallback, so a mutant that always numbers by line and pushes no gap stays green (reasoned, mutant not built)"
```
