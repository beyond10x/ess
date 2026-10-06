---
format: aep.planning-md/3
id: story:feature-request-279
kind: story
status: implemented
title: A stored-guarded moving command does not count as rewriting a group key
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#279
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: docs/design/aggregate-views.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T06:31:26Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-10-06T09:46:17Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:46:17Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A command that moves a row without writing its group key does not make the aggregate witness refuse the view, whatever guards it carries.

## Acceptance

- A minimal specification (key set from input by the creating command; a moving command with two `when_subject` guards that does not touch the key) synthesizes `<view>/aggregate` with no ESS-SYNTH-017.

## Origin

beyond10x/ess#279, reported downstream on 0.48.0.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`: class defect (synthesis only; no authored surface).

## Decisions

- **accept once reduced** (coordinator, 2026-09-30): the implementor builds the minimal reproduction first; if it does not reproduce, the story is closed with that evidence.

## Scope

Derived 2026-10-01 by `aep:story-scoper`; **cited** = read in the tree, **inferred** = a reading. Coordinator-owned at merge, not scope entries: `CHANGELOG.md`, `changes/`, derived outputs.

- **Files:** `crates/verify/ess-conformance/src/synthesize/aggregate.rs:1829` (`kept_as_planned`), from `arrange_row` (:1550) after `advance(…, &[])` (:1521) — cited, the "a move … rewrites" refusal (:1868)
- **Files:** `crates/verify/ess-conformance/src/synthesize.rs:3447` (`advance`, :3461-3471) — inferred cause: for a `subject_fact::uses` command, a failed `subject_fact::step` with nothing arranging returns `reach_state`, a freshly searched row that discards the row created with the pattern's input
- **Also likely:** `synthesize/subject_fact.rs` (`step` :3454, `inputs_for` :1248); `docs/design/aggregate-views.md` — inferred
- **Confidence:** medium — the refusal site is read; the cause is a code walk; the issue is not reduced
- **Would collide with:** 266 (`synthesize.rs` `advance`, the same function); 257, 272 (`aggregate.rs`); 266, 271, 272, 229 (`subject_fact.rs`)
- **Safety fact:** the `reach_state` fallback is reached only with nothing arranging, and the aggregate path is the caller that passes `&[]`; changing that path leaves the other `advance` callers (:3418, :7298) as they are — unproven
