---
format: aep.planning-md/3
id: story:feature-request-461
kind: story
status: implemented
title: A refusal selected by a stored field loses its unchanged-subject check, and cannot sit beside a state-guarded update
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#461
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-429
- depends_on: story:feature-request-454
scope:
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_fact.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/subject_fact_lifecycle_mix.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/subject-fact-complete-refusal.yaml
- confidence: inferred
  path: crates/verify/ess-conformance/tests/subject_fact_complete_refusal.rs
- confidence: cited
  path: docs/design/cross-record-and-stored-field-guards.md
- confidence: cited
  path: website/docs/guides/specify/guards-and-predicates.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T23:49:34Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-05T23:49:34Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-06T17:49:20Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Outcome
Resolve beyond10x/ess#461: A refusal selected by a stored field loses its unchanged-subject check, and cannot sit beside a state-guarded update.

## Origin
beyond10x/ess#461, filed 2026-10-05; found while settling a downstream specification whose update command refuses a changed stored field and updates only an active record. The downstream now holds the lost check in its own test.

## Fit review
1. Need: a command that (a) refuses when a stored field disagrees with the input, (b) updates the record only in one lifecycle state, and (c) refuses in every other state. Its suite must prove that each refusal changed nothing on the record and that every non-accepting state refuses. Minimal reproduction, written fresh: `Instance {name, description, seed_digest}`, states `Active, Removed` (and a variant with `Suspended`), `UpdateInstance {name, description, seed_digest}`. Probes are in `<fit-review scratch>/probe-461/` and ran on the installed `ess 0.53.0` (`ess-version.out`):
   - `natural/`: a `when_subject` refusal beside `when_subject_state: Active` is refused `ESS-COMMAND-004` "subject fact and lifecycle guards cannot be combined in one command".
   - `idiom/`: `not-active` written as `when_subject: {predicate: state != Active}`, with `updated` as the default. It validates. Neither refusal carries `snapshot_complete_subject`/`expect_complete_subject_unchanged`, and the row after `seed-change-refused` is asserted on `name`, `state` and `seed_digest` only. The command was sent `description: description-1`, so a target that writes it while refusing passes.
   - `stateonly/` (no stored-field guard): `not-active` and `Instance/state/Removed/refuses/UpdateInstance` each carry one snapshot and one complete-unchanged step.
   - `idiom3/` and `stateonly3/` add `Suspended`. The idiom witnesses `not-active` only on a `Removed` row. The state-guarded form has `state/Suspended/refuses` and `state/Removed/refuses`. So a target that updates a suspended record passes the idiom's suite.
   The requester offered two fixes (theirs): (1) admit `when_subject:` and `when_subject_state:` on different branches, with a joint partition; or (2) give every `when_subject`-selected refusal the complete-subject snapshot.
2. Class: defect plus gap.
   - Defect: the guide says "A refused parcel is asserted unchanged" (`website/docs/guides/specify/guards-and-predicates.md:137`), and a refusal changes nothing (`website/docs/reference/spec-versions.md:61`). The suite asserts only identity, state and the guarded fields. `observe_unchanged` builds the row from `guarded_fields` only (`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:4744-4754`, `:4990-5011`).
   - Gap: a `state`-reading refusal on a command with no lifecycle move is witnessed in one state, not each state it claims. Per-state scenarios exist only for wrong states of a command that moves (`docs/design/cross-record-and-stored-field-guards.md:425-433`).
   - Request (1) is a convenience. The same partition is expressible with `state` in `when_subject` (`ess/18`, `spec-versions.md:57`).
3. Existing idiom: the probe's `idiom/` form expresses the command exactly. Refusals are checked in declaration order (`docs/design/input-guard-overlap-precedence.md`), and `state != Active` covers every non-active state. This is also the idiom story:feature-request-454 documents. The design keeps the two strategies apart on purpose: "the refusal on mixing them in one command stands" (`cross-record-and-stored-field-guards.md:337-350`, again `:414-415`). The third arrangement the requester tried, `updated` guarded by `when_subject: {predicate: state == Active}` with an unguarded refusal default, validates but refuses `no-such-instance` with ESS-SYNTH-008 (`third/synth.out`; raised as `StrategyWithoutGuard` from `synthesize/existence.rs`, which site I don't know). It is not needed once the idiom witnesses fully.
4. Fit: redesign request (2) on existing pieces, and decline (1).
   - **Snapshot.** The compiler already mints `complete_refusal` for a named wrong-state refusal from `ess/7` (`crates/specify/ess-compiler/src/resolve.rs:2184-2186`; doc `crates/specify/ess-compiler/src/ir.rs:888-891`). From `ess/23` it also mints the flag for every `error:` outcome selected by `when_subject`, in both the predicate and the `{field, equals}` shape. `around_row` then brackets such a refusal with `preserve_refused_subject` (`subject_fact.rs:6170-6176`). That gives the complete snapshot, or the published-field fallback with unobserved fields named (beyond10x/ess#132), as a wrong-state refusal gets (`synthesize.rs:11433-11451`).
   - **Per-state witness.** From `ess/23`, a refusal whose predicate reads `state` is witnessed on one row in each declared state where the predicate, with `state` bound alone, is not false. This is the rule `:425-433` already applies to wrong states. All of it uses existing steps (`scenario.rs:2044-2057`), which the Go and TypeScript runners already execute (`crates/verify/ess-conformance/src/go/runtime.go`, `src/ts/runtime.ts`).
   - **Unchanged elsewhere.** The interpreter reports subject-guard scenarios unsupported (`:435-436`). Entity Runtime lowering and generated code are untouched. `ess verify diff` already classifies a `complete_refusal` change (`crates/verify/ess-diff/src/diff.rs:2649`, `:2676-2680`).
   - **Request (1) declined.** It adds a second spelling of `state ==` within one command and reopens the strategy split the design refused (`:337-350`). It would also need a source-format gate, because it admits what `ESS-COMMAND-004` refuses at every format today.
   - **Hint.** `ESS-COMMAND-004` gains a hint naming the idiom at both emit sites (`crates/specify/ess-domain/src/command.rs:3285-3299`, `crates/specify/ess-domain/src/command/subject_fact.rs:196-205`). Code and message stay the same.
5. Second adopter: a subscription whose plan family is fixed at creation (refuse when `plan_family != input.plan_family`), whose billing contact changes only while `Active`, and which refuses in `Paused`, `Cancelled` and `Expired`. A target that refuses a `Paused` subscription's family change but still writes the new contact must fail, and so must one that updates `Paused` rows.
6. Cost: source format `ess/23`, shared with story:feature-request-429 (no new bump). This story reports one sentence for the `ess/23` row and does not edit `system.rs` or `spec-versions.md`. No `ess-conformance/N` bump, because the steps exist since suite 12. No keyword and no new diagnostic code. `ESS-COMMAND-004` gains a hint. IR bytes change only for `ess/23` documents with a `when_subject` refusal. Suites of `ess/≤22` documents keep their bytes, so no existing suite loses or gains a synthesis refusal. Size L: the per-state arrangement reuses the stored-field search (`search_within`, `subject_fact.rs:3752`), but needs a row per state.
7. Alternatives: (a) Change nothing. The idiom validates, but its suite passes the two mutants above. (b) The requester's (1): rejected under Q4. (c) Snapshot every `when_subject` refusal at every format. Suites of unchanged documents would change, and the complete observer may refuse where today's partial one passes (`subject_fact.rs:6248-6257`). That is what `ess/7` gated for wrong-state refusals. (d) Chosen: the requester's (2) plus the per-state witness, gated at `ess/23`, plus a hint that names the idiom.

## Decisions
accept, redesigned. From `ess/23`, every refusal selected by `when_subject` compiles with `complete_refusal`. Its scenario snapshots the complete subject before the command and requires it unchanged after. A refusal whose predicate reads `state` is witnessed on a row in each state it claims. `when_subject` beside `when_subject_state` stays `ESS-COMMAND-004`, whose hint now names the `state` predicate idiom. The guide (`guards-and-predicates.md:120-142`) and the design note (`cross-record-and-stored-field-guards.md:261-275`, `:337-350`) say so.
Format: shares `ess/23`, introduced by story:feature-request-429. This story reports one sentence for the `ess/23` row, and the coordinator merges it.
- depends_on story:feature-request-429, which owns `FormatVersion::V23`.
- depends_on story:feature-request-454, unit W2-4, in review. It edits the wrong-state and existence witnesses in `synthesize/subject_fact.rs` and `synthesize/existence.rs`, and owns the guide section that states the idiom this story's hint points to.
- No edge to #455. Its seams are `synthesize.rs:6017`, `:11934`, `:12175`, and it already follows #454.
- No edge to #456. Its subsection sits in `## Select an outcome from the held subject state`, this story edits `## Guard an outcome by the subject's stored fields`, and the coordinator dry-runs `git merge-tree`.
- No edge to #448, unit W2-3. It edits `command.rs:5456-6070`, while this story touches the hint at `:3285-3299`. Dry-run before the second merge.
The ESS-SYNTH-008 refusal of the third arrangement (Q3) is noted here and not fixed in this story.

## Acceptance
- when_subject_refusal_observes_complete_subject: with the committed fixture at `ess/23`, the `seed-change-refused` and `not-active` scenarios each carry `snapshot_complete_subject` before the command and `expect_complete_subject_unchanged` after it.
- refusal_that_writes_an_unguarded_field_fails: a scripted target that answers `seed-change-refused` but stores the sent `description` fails that scenario. An honest target passes.
- state_reading_refusal_witnessed_in_every_claimed_state: with states `Active, Suspended, Removed`, `not-active` is witnessed on a `Suspended` row and on a `Removed` row, each with the complete snapshot. A target that updates `Suspended` rows fails.
- complete_refusal_minted_from_ess23_only: the compiled IR carries `complete_refusal: true` on both refusals at `ess/23` and on neither at `ess/22`. `ess verify diff` reports the header change as the existing outcome-observation change.
- below_ess23_suites_unchanged: the fixture at `ess/22`, and every repository model below `ess/23`, synthesize byte-identical suites.
- partial_observer_keeps_published_fallback: where no immediate view projects every field, the refusal keeps the published-field observation and names the unobserved field, with no new synthesis refusal.
- lifecycle_mix_hint_names_state_predicate: the `natural/` model is still refused `ESS-COMMAND-004` with the same message, and its hint contains "`when_subject: {predicate: state`".
- refused_row_sentence_states_complete_observation: `guards-and-predicates.md` states that from `ess/23` a refused record is asserted unchanged in every field. A case in the new conformance test file reads the page and fails without that sentence.

## Scope
- crates/specify/ess-compiler/src/resolve.rs  cited — `complete_refusal` rule (2184-2186) gains `when_subject` refusals from `ess/23`
- crates/specify/ess-compiler/src/ir.rs  cited — `complete_refusal` doc (888-891)
- crates/verify/ess-conformance/src/synthesize/subject_fact.rs  cited — `around_row` (4957-5017), `observe_unchanged` (4744-4754), `preserve_refused_subject` (6170-6176); per-state rows through `search_within` (3752)
- crates/specify/ess-domain/src/command.rs  cited — ESS-COMMAND-004 hint (3285-3299)
- crates/specify/ess-domain/src/command/subject_fact.rs  cited — ESS-COMMAND-004 hint (196-205)
- crates/verify/ess-conformance/tests/subject_fact_complete_refusal.rs  inferred — new cases
- crates/verify/ess-conformance/tests/fixtures/subject-fact-complete-refusal.yaml  inferred — the fit-review model at `ess/23`
- crates/specify/ess-domain/tests/subject_fact_lifecycle_mix.rs  inferred — the hint case
- docs/design/cross-record-and-stored-field-guards.md  cited — Observation (261-275), One strategy or two (337-350)
- website/docs/guides/specify/guards-and-predicates.md  cited — refused-row sentence (137-138) and the idiom paragraph in its section (83-142)
