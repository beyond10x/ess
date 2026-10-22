---
format: aep.planning-md/3
id: story:a-set-effect-selection-reads-the-rows-state
kind: story
status: implemented
title: 'An affects: or instances: where reads the candidate row''s lifecycle state'
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#564
relations:
- decomposes: epic:generic-row-queries
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src
- confidence: cited
  path: crates/specify/ess-domain/src/command/set_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T22:07:58Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-21T22:07:59Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-22T01:34:59Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

From `format: ess/30`, the `where:` of a set effect's selection, `affects: [{entity, where, …}]`
and `instances: {where: …}`, may read the candidate row's lifecycle state (`state == Pending`), as
a row-set guard's `where:` already may (`crates/specify/ess-domain/src/command/row_set.rs:242-250`,
`candidate_fields` includes the held state as `state`). Today it is refused ESS-COMMAND-003
(sizing pass 2, 2026-10-21; refusal site `crates/specify/ess-domain/src/command/set_effects.rs`,
near `:1748`, per the scope table). Below `ess/30` the refusal stays, its hint naming `ess/30`.

Binding design: `docs/design/generic-language-features.md` § G2, the table at lines 267-268: an
`instances: {where: p}` effect and an `affects: [{entity, where, sets}]` entry are both
`each r in rows(E where p)`, and `rows(E where p)` is the one selection construct whose `where:`
reads a candidate row as a row-set guard does. The design implies yes; this story builds it.

Finding 13 of https://github.com/beyond10x/ess/issues/564 (downstream report, 2026-10-19).
Reproduction: `.engineering/repro/affects-filter-state/system.yaml` (cancelling a feed discards
its pending items: `where: {all: ["feed_id == input.feed_id", "state == Pending"]}`). Idiom today:
`.engineering/repro/affects-filter-state/idiom-from-states.yaml`, which narrows the transition's
`from:` to `[Pending]` and so changes the lifecycle (a played item can no longer be discarded).

## Fit review

1. **Need.** A cascade that changes only the selected rows in a given state: discard the pending
   items of a cancelled feed, keep the played ones.
2. **Class.** Gap: the selection construct reads state in a guard and not in an effect.
3. **Already expressible?** Only by narrowing the transition's `from:`, which forbids the move
   everywhere else (the idiom above). A `moves:` already skips rows outside its `from:` states
   (`set_effects.rs:9`, module doc), so the idiom only works where the lifecycle can be narrowed.
4. **Fit.** One rule: every `rows(E where p)` reads the same candidate fields. No new key.
5. **Second adopter.** Closing a project archives its open tasks and leaves the done ones.
6. **Cost.** A format gate; synthesis arranges a decoy row in another state beside each selected
   row; the interpreter and generated Rust and Go set effects read the held state; one diff
   class if the selector's meaning changes (none expected: the key is unchanged).
7. **Alternatives.** (a) Change nothing: the idiom changes the lifecycle. (b) A separate
   `when_state:` on the entry: a second spelling of a predicate. (c) `state` readable in the
   effect's `where:` (chosen).

## Decisions

- Accept, as the design's table reads: from `ess/30` `state` is a candidate field in every
  set-effect selection, with the same meaning as in a row-set guard (the state the row holds
  before the outcome, read from the pre-selection snapshot, § G2 "One construct").
- `state` compared with a state the entity does not declare is refused as in a row-set guard; no
  new diagnostic code.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/specify/ess-domain/tests/set_effects.rs`:
  `from_ess_30_an_affects_where_reads_the_rows_state` (the reproduction validates),
  `from_ess_30_an_instances_where_reads_the_rows_state`, and
  `below_ess_30_state_in_a_set_effect_where_names_ess_30`.
- `crates/verify/ess-conformance/tests/interpreted_set_effects.rs`:
  `affects_where_on_state_moves_only_rows_in_that_state`: cancelling discards the pending item
  and leaves the played item played.
- `crates/verify/ess-conformance/tests/set_effects.rs`:
  `affects_where_on_state_is_witnessed_with_a_decoy_in_another_state`: synthesis arranges a
  played decoy beside a pending row, refuses nothing, and a mutant dropping the state conjunct
  turns a scenario red.
- `crates/verify/ess-conformance/tests/set_effects_go.rs`:
  `generated_go_affects_where_reads_the_rows_state`.

## Scope

| unit | files | confidence |
|---|---|---|
| validation | `crates/specify/ess-domain/src/command/set_effects.rs` (filter fields, near `:1748`), `command/row_set.rs` (`candidate_fields`, `:242-250`, reused) | cited (scope table, read) |
| interpreter | `crates/verify/ess-conformance/src/interpret/` set-effect evaluation | inferred |
| synthesis | `crates/verify/ess-conformance/src/synthesize/` set-effect arrangement | inferred |
| generation | `crates/generate/ess-synth/src/` set-effect queries (Rust, Go) | inferred |
| docs | `website/docs/reference/spec-versions.md` (`ess/30` row) | inferred |

Crates: ess-domain, ess-conformance, ess-synth (sizing: S).

## Order

Lane L4b: this story, then `story:navigation-reaches-any-depth-in-view-filters-and-guards`, then
`story:a-view-source-list-is-refused-by-name`. Serialized with the L4a lane on ess-domain
related-guard and row-set code.
