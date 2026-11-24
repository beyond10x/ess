---
format: aep.planning-md/3
id: story:synthesis-witnesses-a-when-related-chain-with-a-decoy-at-each-step-and-a-missing-middle-row-and-generated-rust
kind: story
status: active
title: Synthesis witnesses a when_related chain with a decoy at each step and a missing middle row, and generated Rust, Go and Entity Runtime name the chain they do not read
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:navigation-reaches-any-depth-in-guards
- depends_on: story:at-the-new-format-a-when-related-via-is-a-chain-of-declared-references-validated-compiled-and-interpreted-its
- depends_on: story:synthesis-witnesses-an-instance-and-an-each-in-that-name-a-member-of-a-struct-input-on-the-parent-s-minimal
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:25:00Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T03:25:01Z", actor: "human:timo", revision: 3}
---
## Need

`g13-guard-chain-domain` admits `when_related: {via: [<first>, <ref>, …]}`. The related-guard
strategy (`crates/verify/ess-conformance/src/synthesize/related_guard.rs`) arranges one row an
input or stored field names, between decoys; the rows a value source's two-hop chain reads are
arranged apart, in `synthesize/related.rs` (`{related: {via, field}}`, `ess/16`, chain `ess/22`),
which this unit reuses. A chained guard needs every row of the chain arranged, through the
creating commands that write each reference field, and decoys at every step, or a target that reads
the near row's own state, or the right middle row's wrong neighbour, passes.

## Outcome

On the reproduction of `g13-guard-chain-domain`, `ess verify conform synthesize` writes a scenario
for every branch with 0 refusals and the suite passes on the interpreted target:

- `plan-frozen` with the far row `Frozen`, `added` with it `Draft`;
- a decoy far row at each step: a second plan in the guarded state that another route names, so a
  target that reads "some frozen plan" or follows the wrong reference fails;
- a missing middle row answering `no-plan` (a route whose `plan_id` names no plan), and a missing
  first row answering `no-route`;
- a target that tests the near row (`route.state`) instead of the far row fails.

Where a step's reference field is written by no creating command from its input, the scenario is
refused by name (`RefusalCause::NoWitness`, naming the field), never written to fail.

Generated Rust and Go keep the command an owed obligation whose text names the chain; Entity
Runtime keeps `RelatedGuardUnsupported` (`docs/design/generic-language-features.md:568-580`).

## Acceptance

- Red first: `crates/verify/ess-conformance/tests/interpreted_related_guards.rs` (existing binary)
  gains `a_when_related_chain_is_witnessed_with_a_decoy_at_each_step`: one synthesis of the
  reproduction, then the interpreted run; on the base (after `g13-guard-chain-domain`) it fails
  with refusals; after, 0 refusals, every scenario passes, and the suite holds the cases above. A
  mutated interpreted target reading the near row's state fails a scenario.
- A three-step chain (`via: [input.stop_id, route_id, plan_id]`) is witnessed the same way, one
  test.
- An unwritable reference step is refused by name, one test.
- `crates/generate/ess-synth/tests/related_guard_several_rows.rs` (existing binary): the plan owes
  the reproduction, the obligation naming the chain (`crates/generate/ess-synth/src/determined.rs`,
  beside the list and several-rows arms at 218-235).
- `crates/generate/ess-entity-runtime`: a case in an existing binary asserts
  `RelatedGuardUnsupported` for the reproduction.
- Models without a chain keep their suite bytes: repository-wide byte pins unchanged
  (`synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes`, `external_beside_held_guard`; `ess-synth` `go`,
  `served_codec_names`, `adversary_related_guard_behaviour_pass1`); `synthesis-sources.digest`
  re-stamped by the task.
- Every new test under 5 s or named with its time; one synthesis per test; no new test binary.

## Fit review

The parent's (`story:navigation-reaches-any-depth-in-guards`) applies; report item 13 covered. No
suite major moves: no step vocabulary is added.

## Decisions

None of its own; generated Rust and Go reading a chain is outside this feature, as the parent
decided ("generated Rust and Go keep the command an obligation naming the chain").

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/related.rs`,
  `crates/verify/ess-conformance/src/synthesize/related_guard.rs`
- inferred: `crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs` (a chain from a
  stored field)
- cited: `crates/generate/ess-synth/src/determined.rs` (218-235)
- cited: `crates/verify/ess-conformance/tests/interpreted_related_guards.rs`,
  `crates/generate/ess-synth/tests/related_guard_several_rows.rs` (existing binaries)
- inferred: an existing `crates/generate/ess-entity-runtime/tests/` binary (one case)
- inferred: `synthesis-sources.digest` (re-stamped)
- Size: M (about 500 changed lines: synthesis 400, target cases 100).
- Overlap: `g10-list-beside-synthesis` and `g11-subject-read-synthesis` change
  `synthesize/related_guard.rs`; land one after another in report order.

## Depends on

- `g13-guard-chain-domain`.
- `story:synthesis-witnesses-an-instance-and-an-each-in-that-name-a-member-of-a-struct-input-on-the-parent-s-minimal`
  (s4, shared synthesis files).

## Source

A downstream application, 2026-11-24, report item 13 (its G8); github:beyond10x/ess#564.
