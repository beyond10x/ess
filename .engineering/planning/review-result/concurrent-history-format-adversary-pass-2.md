---
format: aep.planning-md/3
id: review-result:concurrent-history-format-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, story:concurrent-history-format
relations:
- reviews: story:concurrent-history-format
revision: 1
---
# Adversary pass 2 — story:concurrent-history-format

Dispatched as `aep:adversary` against `ess-chc-format` after correction round 1, 2026-09-27. Report as returned (abridged prose; header and findings block verbatim).

unit: story:concurrent-history-format, uncommitted working tree `ess-chc-format` on impl/concurrent-history-format (base 472d35fbe)
verdict: NEEDS-CHANGE
cases: executed 1872→1880, red 6
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: 9 paths (part 6)
needs-coordinator: none

Cases: six `adversary_*` tests appended to crates/edge/ess-xtask/tests/history_model.rs (lines 689–748),
all RED — serde(skip) on History.seed; serde(default) on seed; skip_deserializing on
Completion::Indeterminate; serde(other) on Completion::Indeterminate; cfg_attr-wrapped rename_all on
Operation; schema properties of the wrong type. Two completion-matrix files
(tests/history_completion_matrix.rs, ess-domain tests/ess_history_schema_completion_matrix.rs), 18
combinations each, GREEN.

Suites (--locked --no-fail-fast): ess-conformance 813 passed 0 failed exit 0; ess-xtask 305 passed 6
failed exit 101; ess-domain 756 passed 0 failed exit 0.

Attacked, not broken: draft-04 oneOf vs reader across 18 combinations; the five pass-1 drift-mutant
tests each go red against the scan mutant they target; case-folding refusal order matches the doc;
determinism (BTreeSet); schema vs reader on UUIDs, digest, non-empty names.

Coordinator routing: F1–F3 introduced → correction round 2 (same implementor). J1–J3 decided by the
coordinator: integers capped at 2^53−1 with a named refusal; `read` made the only way in; SpecDigest
rule declared in the model where ESS can express it.

```findings
- file: crates/edge/ess-xtask/tests/history_model.rs
  line: 247
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the drift scan ignores serde skip, skip_deserializing, default and other, so a field removed from or made optional on the wire, or an enum opened to any value, passes it
- file: crates/edge/ess-xtask/tests/history_model.rs
  line: 229
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: a serde attribute inside cfg_attr is not read, so the rename_all the scan refuses passes when wrapped
- file: crates/edge/ess-xtask/tests/history_model.rs
  line: 478
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the schema drift scan compares property names and optionality only, although the file's doc promises it fails on a wrong field type
- file: crates/verify/ess-conformance/src/history.rs
  line: 169
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: instants have no declared unit and a TypeScript writer cannot emit integers above 2^53 exactly, so nanosecond instants would be corrupted or refused
- file: crates/verify/ess-conformance/src/history.rs
  line: 137
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: History derives a public Deserialize that bypasses every check in read, contradicting the module doc that read is the one way in
- file: models/concurrent-history/domains/history.yaml
  line: 8
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the model declares SpecDigest as any non-empty string while the reader admits only 16-64 lowercase hex characters, and the drift test does not compare invariants
```
