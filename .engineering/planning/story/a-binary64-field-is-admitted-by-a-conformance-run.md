---
format: aep.planning-md/3
id: story:a-binary64-field-is-admitted-by-a-conformance-run
kind: story
status: implemented
title: A binary64 field is admitted by a conformance run
summary: 'Adopter report R8: A Binary64 field blocks a whole conformance run (ess#646).'
refs:
- provider: github
  reference: beyond10x/ess#646
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:18:54Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-26T09:18:54Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-26T15:50:35Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Problem

`ess verify conform run` refuses a whole model that declares any `Binary64` field (`UnsupportedPrimitive`, `crates/verify/ess-conformance/src/admission.rs:1146-1157`), even where nothing a scenario sends or reads reaches it, and the suite format cannot carry a `Binary64` value at all. `Decimal` runs but generates a string, wrong for a JSON-number wire field. Adopter report R8, ess 0.68.0.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/verify/ess-conformance/tests/suite.rs`:
  `an_unreached_binary64_type_does_not_block_a_run`: the minimal reproduction synthesizes every
  scenario of its command, with 0 refusals and one note naming `types.demo.notes.Rate.fields.per_second`,
  and the run on the interpreted target reports all of them passed.
  `sparse_models_cannot_publish_an_empty_success_for_binary64` is restated: an empty suite with a
  note, never a pass.
- Same file: `a_reached_binary64_still_refuses_by_name`: `Binary64` in a command input, a view
  field, an event field or an entity field still refuses the model with its path, as today.
- `crates/verify/ess-conformance/tests/binary64_count_adversary.rs`:
  `unified_model_issues_survive_authored_synthesis_and_web_boundaries` moves its three positions
  into a command input, so they stay reached and refused across authored, synthesis and web. A
  sibling case keeps them in an unreferenced type and expects admission everywhere.
- `crates/edge/ess-cli/tests/binary64_publication.rs`: `binary64_sparse_model_refuses_every_unsupported_publication_route` keeps its refusals for a reached position; a new `an_unreached_binary64_type_publishes_every_route`, and `validate_completeness.rs`, show `conform run`, `synthesize` and validate completeness admit the reproduction.

## Fit review

Reviewed 2026-10-26 on main `a3bc2df3d` (0.69.2) with `ess` 0.69.2. Not fixed on main.

1. **Need.** Two needs. (i) A `Binary64` in a type that no command, event, error, view or entity
   reaches must not refuse the whole model. Minimal reproduction: one note entity and command, plus
   an unreferenced `struct Rate {per_second: Binary64}`. `ess verify conform run --target
   interpreted` and `synthesize` print `error: types.demo.notes.Rate.fields.per_second:
   UnsupportedPrimitive: finite Binary64 is not admitted by the current conformance suite and
   codecs` and run nothing. (ii) A reached `Binary64` (a JSON-number float on the wire) is carried
   by a suite and compared by value. The requester proposed no syntax. Its aside that `Decimal`
   generates a string is by design (`crates/generate/ess-gen/src/types.rs:31`, `:132`) and is not
   part of this story.
2. **Class.** (i) Gap: the boundary is "selected Binary64 use" (`docs/design/model-binary64.md:155-157`),
   but `admission::model` refuses every declared position (`crates/verify/ess-conformance/src/admission.rs:808-810`
   over `ess-compiler/src/binary64.rs:11-28`). (ii) Gap, deliberately left open: "either qualify it
   with a separately coordinated suite-format change or explicitly refuse" (`model-binary64.md:84`);
   `docs/design/binary64-structural-codecs.md:9-10` kept the refusals.
3. **Already expressible?** (i) Only by deleting the type or moving it to another specification.
   (ii) `Json` passes but checks no number, and `Decimal` changes the wire type (issue text). Neither
   keeps the checked figure, so neither is an idiom.
4. **Fit.** No authored surface: `Binary64` is a source type since `ess/2`. (i) is synthesis and
   admission only. (ii) is the suite contract that `model-binary64.md:168-190` leaves to a suite
   version: finite IEEE equality with +0 equal to -0, no epsilon, overflow refused. Every Binary64
   site has to move together: the witness (`witness.rs:6077-6078`), the input binding
   (`input.rs:1452-1455`), the suite serde gate (`admission.rs:1163-1182`), the exact observers
   (`replay.rs:334-336`), and the Go (`go/runtime.go:6660`, `go/replay.go:468`) and TypeScript
   (`ts/response.ts:376-383`) runners.
5. **Second adopter.** A sensor reading published as a JSON float (`temperature: 21.5`) that a
   command records and a view lists.
6. **Cost.** (i): no source or suite format. A model without Binary64 keeps its bytes, and an
   unreached position becomes a synthesis note instead of an error. Tests that pin the whole-model
   refusal change: `tests/suite.rs:340` and `tests/binary64_count_adversary.rs:174`. (ii): a
   suite pair (next free `ess-conformance/58`/`59`, shared with `story:a-view-may-publish-a-decimal`),
   three runners, witnesses, and a design section. Size L.
7. **Alternatives.** (a) Change nothing. (b) Admit unreached types only (chosen for this story). (c)
   Full suite carriage in the same story: the requester's shape. It is changed to a split, because
   (i) is S and unblocks the reported run today, while (ii) is L and must move every runner at once
   (`model-binary64.md:186`: no new primitive through an old suite contract).

## Decisions

- Accept, redesigned (split). This story keeps (i): a Binary64 position whose owner is a declared
  type outside the type closure of every command, event, error, view and entity
  (`synthesize.rs:13089` `reachable_types`) is admitted. Synthesis names it in a note. Entity fields
  stay "reached", conservatively.
- The structural inventory `binary64::locations` stays as it is: `ess-gen`'s codecs read it
  (`binary64-structural-codecs.md:12-13`). Conformance uses a new reached-only inventory beside it.
- (ii) moves to a new story, proposed id
  `a-binary64-value-round-trips-through-a-conformance-suite`, with acceptance lines 2-4 of this
  story. Lines 3 and 4 are corrected there: "non-finite" means an overflowing token such as `1e400`,
  since JSON has no NaN token. The new story amends `model-binary64.md` § "Admission, equality and
  conformance publication decisions" with the suite contract, and opens `Binary64` in the exact
  observers that `story:a-view-may-publish-a-decimal` leaves refused.
- A model whose only Binary64 is unreached and which has nothing else still yields an empty suite,
  and that stays a finding (`scenario.rs:200-203`), not a success.

## Scope

| unit | files | confidence |
|---|---|---|
| inventory | `crates/specify/ess-compiler/src/binary64.rs` (`uses_of`, `:28`): a reached-only variant beside `locations` | cited |
| admission | `crates/verify/ess-conformance/src/admission.rs` `model` (`:808-810`) | cited |
| synthesis | `crates/verify/ess-conformance/src/synthesize.rs` (`:2740-2752`, early return; `reachable_types` `:13089`) | cited |
| other callers | `crates/verify/ess-conformance/src/authored.rs:2699`, `web.rs:93`, `coverage_build.rs:50,180,200`, `mutate.rs:4636,5499`; `crates/edge/ess-cli/src/main.rs:4240,4980,5162,5253,5482` | cited (call sites), inferred (whether each changes or inherits from `model`) |
| docs | `docs/design/model-binary64.md` § First implementation boundary, `website/docs/guides/` Binary64 section | inferred |

Crates: ess-compiler, ess-conformance, ess-cli (sizing: S after the split; L as written). Format:
none for (i), no `ess/32`, no suite format. The split-off (ii) needs the suite pair
`ess-conformance/58`/`59`, shared with `story:a-view-may-publish-a-decimal`. Test binaries:
`ess-conformance` `suite.rs`, `binary64_count_adversary.rs`; `ess-cli` `binary64_publication.rs`, `validate_completeness.rs`. Overlaps:
`story:a-view-may-publish-a-decimal` (only for (ii): `replay.rs:334-336` and the suite pair).

confidence: medium
