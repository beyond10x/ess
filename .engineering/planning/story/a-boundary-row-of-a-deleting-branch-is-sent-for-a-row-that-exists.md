---
format: aep.planning-md/3
id: story:a-boundary-row-of-a-deleting-branch-is-sent-for-a-row-that-exists
kind: story
status: active
title: A boundary row of a deleting branch is sent for a row that exists
summary: Synthesis sends a deleting branch's boundary row for the row its own act deleted; the scenario fails on a conforming target
relations:
- serves: vision:O2
- informed_by: story:an-accepting-branch-below-unknown-instance-is-sent-for-a-stored-row
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T05:57:25Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-03T05:57:25Z", actor: "human:timo", revision: 9}
---
## Problem

Found on 2026-10-29 while preparing
`story:an-accepting-branch-below-unknown-instance-is-sent-for-a-stored-row`, and not part of
beyond10x/ess#645. A deleting default branch whose sibling has an orderable input guard is further
witnessed at that guard's boundary. Synthesis sends the boundary row in the same scenario, for the
row the scenario's own act has just deleted, so the target cannot answer the deleting branch.

Minimal model (neutral names): `CloseNote(note_id, count: Integer)` with
`dropped: {when: 'count > 3', accepts: nothing}` and a default `closed` that
`deletes: demo.notes.Note` on `instance: note_id`. `OpenNote` creates a note.

Checked on `ess 0.77.0` (a repro outside the repository), without any `unknown_instance:` branch:

```text
demo v1 against interpreted 0.77.0 — failed
  failed demo.notes.CloseNote/outcome/closed
  passed demo.notes.CloseNote/outcome/dropped
  passed demo.notes.OpenNote/outcome/opened
```

The `closed` scenario opens a note, closes it with `count = 1`, then sends `count = 3` for the same
captured note and expects `closed`. The target reports "no declared outcome was reached". With an
`unknown_instance:` branch declared, it observes `unknown`.

Read in the tree (not confirmed by a build): `boundaries` (`crates/verify/ess-conformance/src/
synthesize.rs:17008`) arranges a fresh row for each boundary only for a `moves:` branch
(`subject.effect.transition().is_some()`, `:17117-17120`; `ResolvedEffect::transition` is `None`
for `deletes:` in `crates/specify/ess-compiler/src/ir.rs:783-787`). Its doc names `moves:` and
`updates:` only.

## Acceptance

- In the model above, the `closed` scenario's boundary send (`count = 3`) goes to a note that
  exists when it is sent, and the scenario passes on the interpreted target.
- A `moves:` or `updates:` branch keeps its boundary arrangement byte for byte, and the existing
  rows of the synthesis bytes tables do not move except where a repository model has a deleting
  branch with a boundary row. Any such row is listed in the fit review.

## Scope

Derived 2026-11-03 by `aep:story-scoper` for wave W2 on b89faaa41. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong). The story's line numbers are from 0.77.0 and have drifted: `boundaries` is now at synthesize.rs:17751 (was :17008), the transition filter at :17880-17883 (was :17117-17120), `ResolvedEffect::transition` at ir.rs:791-797 (was :783-787).

- **Primary surface:** `crates/verify/ess-conformance` (synthesis of boundary rows) — cited
- **Files:** `crates/verify/ess-conformance/src/synthesize.rs` (`boundaries` :17751, transition filter :17880-17883, `identity_boundary_row` :17943) — cited
- **Symbols:** `boundaries`, `identity_boundary_row`, `ResolvedEffect::transition` (ir.rs:791), `ResolvedEffect::Deletes` — cited
- **Also likely:** `crates/verify/ess-conformance/src/synthesize/existence.rs` (`unknown_answers_before` :50, a gate the fix may widen) — inferred
- **Also likely:** a new acceptance test file under `crates/verify/ess-conformance/tests/` — inferred
- **Documents:** none
- **Confidence:** high — story and tree agree (`deletes:` has no `transition()`, so no fresh row is arranged); not re-run at 0.84.0
- **Would collide with:** any unit touching the boundary-row arrangement in synthesize.rs (`boundaries`, `identity_boundary_row`), and the synthesis bytes tables for a model with a deleting branch carrying a boundary row — inferred
- **Safety fact:** a `deletes:` branch takes no `transition()` (ir.rs:791-797), so the `moves:`/`updates:` arrangement at synthesize.rs:17880-17883 is keyed on that and a `Deletes`-only arm leaves it alone — inferred
- **Already in the tree (partly):** `identity_boundary_row` treats a deleted identity as needing a fresh row (synthesize.rs:17975-17979), but only when `existence::unknown_answers_before` holds (:17956, existence.rs:50); the story's model has no `unknown_instance:` branch — cited
- **Size:** medium, about 30 source lines plus a test of about 150; no format version (`COVERAGE_SUITE_FORMAT`, coverage.rs:15) — inferred
- **Targeted run:** ess-conformance `--test accepting_branch_below_unknown_instance`, `related_guard_absent_input`, `synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_claim_fixtures`, `synthesis_suite_bytes_table_claim_search`, `synthesis_suite_bytes_table_cli_read`, plus the new test — inferred
