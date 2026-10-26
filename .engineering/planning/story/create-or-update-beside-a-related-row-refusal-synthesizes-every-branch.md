---
format: aep.planning-md/3
id: story:create-or-update-beside-a-related-row-refusal-synthesizes-every-branch
kind: story
status: active
title: Create or update beside a related row refusal synthesizes every branch
summary: 'Adopter report G6: Create-or-update beside a related-row refusal does not synthesize every branch (ess#641).'
refs:
- provider: github
  reference: beyond10x/ess#641
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:18:58Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-26T09:18:58Z", actor: "human:timo", revision: 7}
---
## Problem

A create-or-update command (an `unknown_instance: true` creator beside update and preserve branches) with a `when_related: {via: input.<id>, exists: false}` refusal validates, but synthesis finds no candidate for the creator (ESS-SYNTH-003: no stored related row selecting the branch, 1 arrangement tried) and then cascades ESS-SYNTH-004 onto the refusal and every route through the command. Without the related refusal the command synthesizes. Adopter report G6, ess 0.68.0; the adopter's call scenarios dropped from 156 to 107.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary. The
fixture is `crates/verify/ess-conformance/tests/fixtures/create-or-update-beside-related-refusal.yaml`
(the scratch reproduction's four branches), pinned in both repository-wide tables.

- `crates/verify/ess-conformance/tests/related_guard_unknown_instance.rs`:
  `a_create_or_update_beside_a_related_refusal_synthesizes_every_branch`: synthesis refuses
  nothing, and `Bind/outcome/{bound,no-agent,rebound,already-bound}` are filed.
- same binary, `the_creator_is_sent_with_an_agent_no_row_carries`: `Bind/outcome/bound` sends an
  unknown seat, and no step before it creates an `Agent` named by its `agent_id`.
- same binary, `the_related_refusal_is_witnessed_on_a_seat_the_creator_stored`: `no-agent` runs
  after a `bound` send of the same seat, with an agent naming no row and differing from the stored
  one.
- same binary, `every_create_or_update_branch_passes_on_the_interpreted_target`: the four pass. With
  the phases exchanged through `precedence::with_phase_order` (`precedence.rs:596`; present-related
  before existence), `bound` fails. Against a target skipping the related check, `no-agent` fails.
- same binary, `the_precedence_plan_answers_the_unknown_seat_before_the_missing_agent`:
  `PrecedencePlan::phase_of(bound)` is `Existence` and `phase_of(no-agent)` is `PresentRelated`.
- `crates/generate/ess-synth/tests/related_guard_unknown_instance_served.rs`:
  `a_generated_create_or_update_beside_a_related_refusal_passes_every_branch` (inferred: that the
  generated server serves the shape is not checked yet).

## Fit review

Checked 2026-10-26 against main `a3bc2df3d` (ess 0.69.2) with a brand-free reproduction in scratch
(`fit/repro-g6/`, outside the repository). A seat is bound to an agent: `Bind(seat_id, agent_id)`
with `no-agent` (`when_related: {via: input.agent_id, exists: false}`), `rebound` (`updates:`) and
`bound` (`unknown_instance: true`, `creates:`). The three-branch file `noalready.yaml` is the
smallest failing shape: `bound` is refused ESS-SYNTH-003 ("`desk.seat.Agent` stored related row
selecting this branch, over the rows 1 bounded arrangements left") and the other two cascade
ESS-SYNTH-004. With `already-bound` added (the adopter's four branches) the result is the same, one
more 004. Without `no-agent` the command synthesizes 3 scenarios with 0 refusals. Not fixed on main.

1. **Need.** A create-or-update command that also refuses a missing related row must be witnessed
   on every branch. Requester's expectation (issue G6, "Expected"): "`bound` with an `Agent` row
   arranged for `input.agent_id`, `no-agent` with none".
2. **Class.** Defect. Validation admits the shape from `ess/27` (below it ESS-COMMAND-004, observed
   at `ess/26`), and the one rule orders it: `unknown_instance:` answers at step 2
   (`ess-domain/src/command/precedence.rs:515`) and an input-named `exists: false` answers at
   step 4 (`:487`). Synthesis cannot witness step 2.
3. **Cause (code reading, not traced on a run).** `existence::creating_input`
   (`ess-conformance/src/synthesize/existence.rs:1226`, branch at `:1233`) sends a creator through
   `related_guard::prepare_at_in` (`related_guard.rs:2377`) on every command reading a related row.
   That is the pre-`ess/27` rule ("which only an arrangement of that row reaches", doc comment above
   `:1226`). The goal is `related_guard::selects`, which drops `UnknownInstance` branches on a
   present row (`related_guard.rs:1183`), so no arrangement can select `bound`. Its doc comment
   still states the #211 order (`:1149`). Expressible today: no.
4. **Fit.** No grid cell opens. This is precedence step 2 against step 4, already decided (§ G4
   "The rule"). The shape `unknown_instance:` error + `when_related` is already witnessed
   (`tests/related_guard_unknown_instance.rs`); this extends that to an accepting creator. Entity
   Runtime and generated Rust read the same `PrecedencePlan`. I don't know whether generated Rust
   serves this exact shape; that is checked by the `ess-synth` case below.
5. **Second adopter.** A ticket is put on a queue: a new ticket is filed, an existing one is
   moved, and the move is refused when the named queue does not exist.
6. **Cost.** Synthesis only: no source format, no IR, no suite major, no diagnostic. One new
   fixture, which adds rows to the two repository-wide tables
   (`ess-compiler/tests/fixtures/selection-precedence-table.tsv`,
   `ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv`, as `904befcb9` did).
7. **Alternatives.** (a) Change nothing: a third of the adopter's call scenarios stay unwitnessed
   (156 to 107). (b) The requester's witness, `bound` with an `Agent` arranged: it passes, but a
   target reading the agent before the seat's existence passes it too. (c) Witness `bound` with an
   unknown seat and an `agent_id` naming no row (chosen). Only a target that follows step 2 answers
   `bound` there, so it is the witness that tells the orders apart.

## Decisions

- Accept, redesigned: no new syntax. Under `ess/27`+ the creator's input is not searched through
  the related-row arrangement. `bound` is sent with an unknown seat and an agent naming no row.
  `no-agent` is sent on a `Binding` the creator stored, with an agent naming no row and a value
  that differs from the stored one, so step 3 does not take it. `rebound` gets an `Agent` row
  arranged, and `already-bound` resends the stored agent. Below `ess/27` the shape stays refused as
  ESS-COMMAND-004.
- Changed from the request: `bound` gets no `Agent` row. Reply to the adopter: under the one rule
  a seat no record carries is created whether or not its agent exists. If the intent was to refuse
  a new seat for a missing agent too, that is a precedence question (related existence before
  addressed existence). It belongs with `story:a-related-row-may-answer-before-input-guards`
  (beyond10x/ess#554), not here. The issue does not say which is meant: unclear, ask them.
- Correct `related_guard::selects`'s doc comment (`:1149`) to the one rule.

## Scope

| unit | files | confidence |
|---|---|---|
| creator input | `crates/verify/ess-conformance/src/synthesize/existence.rs` (`creating_input` `:1226-1260`) | cited |
| related arrangement | `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (`selects` `:1149-1235`, `prepare_at_in` `:2377`) | cited |
| update-branch routes | `existence.rs` `:1321` (`point_at_missing`), `subject_fact.rs` `search_rows` `:5520-5640` | inferred |
| tests | `crates/verify/ess-conformance/tests/related_guard_unknown_instance.rs`, new fixture beside `fixtures/related-guard-unknown-instance.yaml` | cited |
| tables | `crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv`, `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv` | cited |
| generated server | `crates/generate/ess-synth/tests/related_guard_unknown_instance_served.rs` | inferred |

Crates: ess-conformance (synthesis), with ess-compiler and ess-entity-runtime test tables only.
Format gate: none (no `ess/32`, no suite major; the shape is admitted from `ess/27`). Overlaps:
`story:related-rows-of-one-command-share-their-owner` and
`story:a-related-owner-is-moved-after-the-subject-is-created` (both active; same `related_guard.rs`
arrangement), `story:an-identity-guard-beside-a-row-set-validates` (unit U16, same command family,
`row_set.rs` not `related_guard.rs`), `story:unknown-instance-and-input-guard-agree-on-order` and
`story:a-related-row-may-answer-before-input-guards` (precedence witnesses). Serialize with the
G-U16 pair. Sizing: M.

confidence: medium
