---
format: aep.planning-md/3
id: story:requirement-refs-reach-suite-and-report
kind: story
status: draft
title: ess verify diff rates a refs-only change as documentation
tags:
- defect
- severity-medium
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#503
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: crates/verify/ess-diff/src/compatibility.rs
- confidence: cited
  path: crates/verify/ess-diff/src/delta.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/impact.rs
- confidence: inferred
  path: crates/verify/ess-diff/tests/refs_changes.rs
revision: 12
---
## Outcome

`ess verify diff` rates a change to `refs` alone as documentation, as it rates a `summary` change,
instead of `system/<name>/unclassified-changed` ("unknown"). This is the defect half of
https://github.com/beyond10x/ess/issues/503 (part B): the diff's own documentation-only rule
(`documentation_only`, `crates/verify/ess-diff/src/compatibility.rs:812-814`) covers display names,
summaries and examples, and `refs` is kept residual on purpose (`crates/verify/ess-diff/src/diff.rs:2929`,
"`refs` is deliberately absent: it stays residual"), so a refs-only edit falls to the residual
comparison and is reported as an unknown change. Refs live on commands, outcomes, bindings and
components in the compiled IR (`Refs` fields in `crates/specify/ess-compiler/src/ir.rs:1010,2079,3051,3207`).

The feature half of part B (authored scenarios accept `refs:`, synthesized scenarios inherit them,
the suite and the run report carry them) is split off as its own backlog draft outside this epic.

## Problem

Reproduced on 0.92.0 with the workbench base specification
(`.engineering/repro/workbench-2026-11-09/examples/00-base/spec`) given `refs: [std:REQ-1]` on
`gap.demo.CreateParent` and `refs: [std:REQ-2]` on its `created` outcome (`ess specify validate`:
valid). Changing only the outcome's ref (`std:REQ-2` to `std:REQ-3`), `ess verify diff --from v1
--to v2 --compatibility` reports `system/gap/unclassified-changed`, "unknown for callers, readers,
history".

## Fit review

1. **Need.** A refs-only edit, which no caller, reader or stored value depends on, is reported as
   documentation, not as an unknown change that fails `--fail-on breaking-or-unknown`.
2. **Class.** Defect against the diff's own documentation-only rule: a refs-only edit is
   `unclassified-changed`, where it belongs with `summary` changes (`documentation_only`,
   `crates/verify/ess-diff/src/compatibility.rs:812-814`).
3. **Already expressible?** No: the residual comparison cannot tell a refs edit from any other
   uncompared change.
4. **Fit.** Same rule as the summary and display-name changes; one change kind per construct that
   carries `refs`.
5. **Second adopter.** A service tracing outcomes to an internal control catalogue, whose
   catalogue renumbering today fails its compatibility gate.
6. **Cost.** A new delta version, `ess-diff/24`, carrying the new change kinds; no source format, no
   suite format.
7. **Alternatives.** (a) Change nothing. (b) Drop refs from the residual comparison without a
   change kind: the edit would vanish from the delta. (c) This story.

## Decisions

- A refs-only edit is a change kind of its own on the construct that carries the refs, written as
  `ess-diff/24` (the current newest is `/23`, `crates/verify/ess-diff/src/delta.rs:18`), because a
  delta at `/23` or below has no entry for it.
- `ess-diff/24` is this epic's one new delta version: another story of the epic that needs a delta
  version takes the same `/24`.
- Recording a run as AEP evidence is the AEP side's work; ESS has no AEP dependency.
- Part C (a requirements register) is `story:requirement-completeness-without-prose-in-the-model`.

## Acceptance

All tests are in `crates/verify/ess-diff/tests/refs_changes.rs` (new), run with
`cargo test -p ess-diff --locked --test refs_changes`, on the v1/v2 pair above; each is red on 0.92.0.

- `an_outcome_refs_change_is_documentation`: the delta of v1 to v2 is `ess-diff/24` and holds
  exactly one change, `command/gap.demo.CreateParent/outcome-refs-changed/created`, classified
  compatible for callers, readers and history (documentation), and no
  `system/gap/unclassified-changed`.
- `a_command_refs_change_is_documentation`: changing only `gap.demo.CreateParent`'s ref
  (`std:REQ-1` to `std:REQ-4`) gives exactly `command/gap.demo.CreateParent/refs-changed`,
  classified documentation.
- `a_binding_and_a_component_refs_change_is_documentation`: a refs-only edit on a binding and on a
  component gives `binding/<name>/refs-changed` and `component/<name>/refs-changed`, each
  classified documentation.
- `a_refs_change_fails_no_gate`: `ess verify diff --from v1 --to v2 --compatibility --fail-on
  breaking-or-unknown` exits 0.
- `cargo test -p ess-diff --locked` passes with every committed delta fixture unchanged: a pair
  without a refs edit keeps its delta bytes and its `ess-diff/` version.

## Scope

- cited: crates/verify/ess-diff/src/compatibility.rs
- cited: crates/verify/ess-diff/src/diff.rs
- cited: crates/verify/ess-diff/src/change.rs
- cited: crates/verify/ess-diff/src/delta.rs
- inferred: crates/verify/ess-diff/src/impact.rs
- inferred: crates/verify/ess-diff/tests/refs_changes.rs

confidence: medium

`diff.rs` compares `refs` instead of leaving it residual; `change.rs` gains the change kinds and
their minimum format; `delta.rs` admits `ess-diff/24`; `impact.rs` stops reading a refs-only edit
as an uncompared family. Shared surfaces: proposes a `CHANGELOG.md` sentence.
