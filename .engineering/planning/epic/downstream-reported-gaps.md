---
format: aep.planning-md/3
id: epic:downstream-reported-gaps
kind: epic
status: active
title: Downstream-reported synthesis, binding and grant gaps are closed
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:18Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-30T13:04:29Z", actor: "human:timo", revision: 3}
---
## Outcome

A downstream specification synthesizes, runs and records conformance without refusals that come from ESS rather than from the model: related-row guards and values, aggregate group keys, bindings that move state, authorization, and the binding language gaps it reported are closed.

## Scope

Issues beyond10x/ess#229 (state inside when_related), #257, #265, #266, #267, #268, #269, #270, #271, #272. Each story names its conformance scenarios; the downstream mini specifications under the reporter's scratch reproduce #270–#272 and are not copied into this repository.

## Order

Scheduled by `aep plan artifact waves` from each story's scope; the synthesis stories that share `synthesize/aggregate.rs` or the relation-guard arrangement run in sequence.

## Downstream report 2026-10-19 (triage)

Eighteen findings from one downstream specification (ess 0.60.0; reported 2026-10-19), triaged on
0.60.0, 0.62.0 and an integration-branch build (0.62.0 plus the 2026-10-13 port). Neutral
reproductions under `.engineering/repro/`. Where each finding went:

| # | finding (short) | answer |
|---|---|---|
| 1 | `affects:` cascade onto rows whose creator has a guarded sibling: ESS-SYNTH-001 | `story:a-cascade-onto-rows-of-a-guarded-creator-is-witnessed` |
| 2 | `--synthesis-seed` cannot reach those rows | same story, Decisions (seeds stay subject-only) |
| 3 | authored scenarios do not discharge obligations | `specification:authored-scenarios-are-selected-by-scenarios-and-discharge-no-obligation` |
| 4 | `ess-inputs.yaml` `scenarios:` not read by synthesize/run | same specification (documented; `--scenarios <model-dir>`) |
| 5 | row-set `when_related` beside identity-addressed: ESS-COMMAND-009 | `epic:generic-row-queries` (wave 4 item 4, #473) |
| 6 | two related rows of one entity: every outcome ESS-SYNTH-003 | needs the reporter's failing command (below) |
| 7 | owned row's stored `via` compared with an input | `story:an-owner-link-guard-beside-an-owner-existence-guard-is-witnessed` |
| 8 | `{related:}` copy read later | row-set half fixed by #601 (`tests/row_set_related_copy.rs`); `affects:` half in finding 1's story; stored list / four records: needs repro |
| 9 | related row's `state` as a value | `story:row-queries-navigate-a-reference` |
| 10 | `when_related` one hop; `subject.` forms | hint: `story:a-subject-prefixed-via-hint-names-the-bare-field`; row-set: `story:a-related-selector-compares-a-row-field-with-a-subject-field`; depth: `story:row-queries-navigate-a-reference` |
| 11 | two identity-addressed rows in one branch | `story:row-queries-navigate-a-reference` |
| 12 | refusals answered before a narrower success branch | `story:declaration-order-agrees-with-every-step-pair` |
| 13 | `state` in an `affects:` `where:` | `epic:generic-row-queries` |
| 14 | `<Command>/outcome/wrong-state` id disappears | `specification:the-wrong-state-scenario-id-is-the-unknown-identity-fallback` |
| 15 | every entity needs a lifecycle | `specification:a-stateless-record-declares-a-one-state-lifecycle` |
| 16 | validate admits a write breaking a field invariant | `story:an-authored-text-literal-outside-its-alphabet-is-refused` (entity-invariant literals) |
| 17 | Integer literals written as floats | `story:an-integer-literal-is-an-integer-token-in-a-suite` |
| 18 | `ess verify diff` rates most changes unknown | `story:verify-diff-decides-what-the-model-settles` |

Needs the reporter's reproduction:

- **6.** Two related rows of one entity with existence and direction guards on both
  (`.engineering/repro/two-related-rows/system.yaml`, `ported.yaml`) synthesize with 0 refusals on
  all three binaries. Missing: the exact failing command (its guards and the refusal text); the
  report's own wording ("modality, encoding and pipeline membership … compare two rows") suggests a
  cross-row comparison, which is finding 11.
- **8, second half.** "A guard over a stored list" and "a guard over four related records": the
  report gives neither the command nor the refusal text.
