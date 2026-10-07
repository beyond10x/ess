---
format: aep.planning-md/3
id: story:entity-runtime-lowering-reads-selection-plan
kind: story
status: active
title: The Entity Runtime lowering orders branches by the selection plan
relations:
- decomposes: epic:one-selection-plan
- serves: vision:O2
- depends_on: story:selection-plan-design-and-type
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests
- confidence: inferred
  path: website/docs/reference/entity-runtime-lowering.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-07T13:18:12Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-10-07T13:18:13Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":3}}}
---
# Story: The Entity Runtime lowering orders branches by the selection plan

## Why

`epic:one-selection-plan`. The lowering sorts compiled branches by a local category
(`crates/generate/ess-entity-runtime/src/lib.rs`, the `sort_by_key` after slot canonicalisation):
input refusals, then every other guarded branch by declaration index, the default, `wrong_state`.
That category put held-state and accepting branches in one class, which was half of
beyond10x/ess#486.

## What it delivers

The sort key comes from `story:selection-plan-design-and-type`'s plan.

## Acceptance

- A digest table of the lowered definitions of every repository model that lowers is written at the
  story's base before `lib.rs` changes. After the change every line is identical except the epic's
  named exception (a held-state branch declared after an accepting branch the finite prover shows
  disjoint); each such command is listed in the PR with its reordered branches, and its answers on
  `entity_core::Runtime` are unchanged.
- A test using `story:selection-plan-design-and-type`'s phase-order override, with the held-state and
  accepting/external phases exchanged, shows the lowered branch order change with it, run on
  `entity_core::Runtime` as `crates/generate/ess-entity-runtime/tests/selection_precedence.rs`
  (added by #487) does.
- The plan supplies the phase order and the order within each phase. Two target rules stay in the
  lowering, each named in a comment at the sort: entity-core requires the default last among the
  non-`wrong_state` branches, and drops `wrong_state` from selection. No other
  branch-selection-order rule remains in `lower_command`; slot numbering by declaration index
  (`lib.rs:1794-1808`) is naming, not selection order, and stays.

## Scope

Derived 2026-10-07 by `aep:story-scoper` on `985f58cc3a`. Every line is **cited** (read from the
story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/generate/ess-entity-runtime` — cited
- **Files:** `crates/generate/ess-entity-runtime/src/lib.rs:1809-1832` (`Projector::lower_command`, the `compiled.sort_by_key` and its comment) — cited
- **Order encoded there:** category 0 an error branch whose condition is `When` (`lib.rs:1820-1823`, step 2); category 1 every other guarded branch, held state (step 4) with accepting and external (step 6), the class behind #486 (`lib.rs:1824-1825`); category 2 `is_default_branch()` (`lib.rs:1825`); category 3 `wrong_state` (`lib.rs:1818`); ties keep declaration index (`lib.rs:1827`) — cited
- **Target rules under that order:** entity-core refuses a definition whose default is not last among the non-`wrong_state` branches (`entity-core/src/definition.rs:780-782`, `validation.rs:887-889`, rev `4746bd7`); entity-core drops `wrong_state` from selection (`runtime.rs:1085`, `:1524`), so its position affects bytes only — cited
- **Refusal-first preloading is entity-core's:** `decide_before_load` walks lowered branches in order and loads the row at the first `in_state` or row-reading guard (`runtime.rs:1082-1094`); `tests/refusal_beside_state.rs:1-8` pins it — cited
- **Steps 1, 3 and 5 never reach the sort:** related-row and row-set guards refuse the command (`lib.rs:1504-1533`); `UnknownInstance`, `ExistingInstance`, `InputAbsent` are refused by diagnostic (`lib.rs:2376-2410`) — cited
- **Order that is not selection order:** slot numbering follows declaration index (`lib.rs:1794-1808`, `SlotBook::canonicalize` `:3260-3271`), as does the walk of a refused command (`lib.rs:1580`) — cited
- **Byte pinning today:** no test pins lowered bytes for every model; `tests/lowering.rs:126-159` checks billing and gatepass lower deterministically; `canonical_definition_bytes` is private (`lib.rs:587`) — cited
- **Pinning precedent:** `crates/verify/ess-conformance/tests/external_beside_held_guard.rs:845-1060` (`walk`, `models`, `digest`, a base `.tsv` written through an env var before the change) — cited
- **Also likely:** a new test and digest fixture under `crates/generate/ess-entity-runtime/tests/`; `src/subset.rs:208-235` order wording, repeated in `website/docs/reference/entity-runtime-lowering.md` — inferred
- **Documents:** `docs/design/input-guard-overlap-precedence.md:308-317`, `docs/design/mutation-audit-and-model-runner.md:501` describe the sort — cited, editing optional
- **Coupling to `story:generated-behaviour-reads-selection-plan`:** none at file level; `ess_synth` is used only for `SynthesisPlan::of` feeding `ess_service_contract::extract` (`lib.rs:39`, `:426-428`) and `PlannedCapability` (`lib.rs:89`), both capability dispositions — cited
- **Confidence:** high. One sort, named by the story.
- **Would collide with:** any unit editing `lib.rs` `lower_command`, `src/subset.rs`, or `crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json` (lists private fns) — cited / inferred
- **Byte-identity risk:** #487's rule refuses only overlapping pairs. A held-state branch declared after an accepting branch the finite prover shows disjoint stays valid; the plan would move it earlier — same answers, different lowered bytes. Only a base digest table taken before the change shows whether a repository model has one — inferred

### Design decisions for the implementor (inferred)

1. Lowered order: input refusals (step 2), held state (step 4), accepting and external (step 6), default, `wrong_state`; declaration index within each. The plan supplies the default and `wrong_state` positions, or a target rule stays in the lowering.
2. A refusal guarded by the held state stays in step 4; moving it to the input-refusal phase changes bytes, and preloading loads the row there anyway (`runtime.rs:1087`).
3. Match plan phases with no wildcard arm; phases 1, 3 and 5 are unreachable here.
4. Keep slot canonicalisation on declaration index: it is naming, not selection.
5. Digest table: per repository model and component, `lower_component` with empty options, read the closure from the `MissingDefinitionVersion` paths (`lib.rs:788-797`), lower again, digest `serde_json::to_vec(definition.as_definition())` per entity or the sorted refusal codes. Write the base before touching `lib.rs`.
6. The exchanged-phase test drives `entity_core::Runtime` through a crate-private seam.
7. A moved base table is a named change in the PR, never a silent re-pin.
