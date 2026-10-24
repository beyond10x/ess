---
format: aep.planning-md/3
id: story:when-subject-reads-the-addressed-identity
kind: story
status: draft
title: when_subject reads the addressed row's identity
summary: 'Adopter report R3: a when_subject predicate on the identity field is refused as ESS-COMMAND-003.'
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Report

An adopter retrofitting an existing service (2026-10-23, ess 0.67.0, ess/29): a `when_subject` predicate naming the addressed entity's identity field (`parent_id == input.parent_id`, or `defined(parent_id)` as the stored-row test) is refused as ESS-COMMAND-003, because the identity is not a stored field; the same command draws ESS-COMMAND-005. Related: #505 (same refusal for `when_related: {via: …}`, closed).

## Acceptance (draft, fit review pending)

- `when_subject` reads the addressed row's identity as a stored fact, or the refusal names the construct that states the same thing.

## Fit review

Request (an adopter, verbatim): `when_subject: {predicate: parent_id == input.parent_id}` on the
identity field is refused: "error[ESS-COMMAND-003]: `parent_id` reads `parent_id`, which is not a
declared observable root". The same command also draws ESS-COMMAND-005. Need: the identity readable
as a stored fact; `defined(parent_id)` as the "a row is stored" test.

## Fit review

1. **Need.** Branch one command between "the addressed row is stored" (update) and "it is not"
   (create). The predicate on the identity is the adopter's spelling of that test, not the need.

2. **Class.** Convenience. The addressed row is selected by the identity, so on that row
   `parent_id == input.parent_id` is true by construction. A `when_subject:` branch is reached only
   for a stored row (step 3 of the G4 order, after existence at step 2,
   docs/design/generic-language-features.md:440-480), so `defined(parent_id)` is also always true
   there. Neither predicate can distinguish anything.

3. **Already expressible?** Yes. "Stored / not stored" is the existence step: `unknown_instance:` /
   `existing_instance:`, and create-or-update since ess/25 (#530; fixture
   crates/specify/ess-compiler/tests/fixtures/create-or-update-stored-fields.yaml). Reproduction:

   ```yaml
   outcomes:
     - name: relabelled
       when_subject: {predicate: parent_id == input.parent_id}   # refused, 003 + 005
       updates: demo.reg.Parent
       instance: parent_id
       sets: {label: input.label}
     - name: registered
       unknown_instance: true
       creates: demo.reg.Parent
       instance: parent_id
       sets: {label: input.label}
   ```

   ess 0.68.0 (ess/30): ESS-COMMAND-003 with hint "stored fields: `label`; the input is read as
   `input.<field>` (ess/15) or in `when:`…", plus ESS-COMMAND-005 "`parent_id` has no closed set of
   values; declare a genuine default" (fit/repro/r3-adopter-shape.yaml). `defined(parent_id)` gives
   the same pair (fit/repro/r3-defined.yaml). The idiom drops `when_subject:` from `relabelled`, so it
   is the default for a stored row and `registered` answers the unknown one: valid, 2 scenarios
   synthesized, both pass on the interpreted target (fit/repro/r3-idiom.yaml).

4. **Fit with grids and targets.** Grid R (docs/design/language-grids.md) lists what `when_subject:`
   reads: input (ess/15), subject fields and state (ess/18), `now`. The identity is excluded by
   design: synthesize/subject_fact.rs `check` is "The environment entity.rs builds for invariants,
   minus the identity". The identity is readable where it varies: a row-set `where:` (candidate
   row) and set-effect filters (#547). A related identity read in `when_related` is #505, closed
   NOT_PLANNED and moved to story related-identity-readable-in-when-related (G2 wave 4). Admitting it
   on the addressed row would add a grid cell whose every value is a tautology.

5. **Second adopter.** None needed: the semantics already exist under another spelling.

6. **Cost.** Admitting: an identity root in the subject environment, a finite-domain case for a
   tautology, synthesis that must refuse to witness the false side. Declining: a hint and a
   suppressed duplicate diagnostic.

7. **Alternatives.** (a) Admit the identity read: rejected, every predicate over it is constant.
   (b) Admit it and refuse the constant predicate with a new code: rejected, same answer as today
   at higher cost. (c) Decline, name the idiom in the refusal and stop the 005 noise: chosen.

## Decisions

**Decline, with the idiom.** "A row is stored" is the existence step; write the update branch
without `when_subject:` beside an `unknown_instance: true` creator.

- **003 hint.** When the refused root equals the subject entity's identity name, the hint reads:
  "`<id>` is the identity that addresses this row, so it equals `input.<id>` on every row
  `when_subject:` sees; branch on whether the row is stored with `unknown_instance: true` beside an
  update branch (create-or-update, ess/25)". Location: the 003 hint in
  crates/specify/ess-domain/src/command/subject_fact.rs (around lines 520-540), message from
  crates/specify/ess-domain/src/expression.rs:940-990.
- **005 suppression.** The coverage proof (subject_fact.rs:590-630) is skipped for a command whose
  `when_subject:` predicate was already refused as 003, so one cause yields one diagnostic.
- **Format.** None. No IR change, no new ValidationCode.
- **Pins.** The hint text moves; run `task test-xtask` and
  `cargo test -p ess-compiler --test typed_diagnostics`.
- **Crates.** ess-domain (gate also ess-cli, per AGENTS.md).
- **Reply to the adopter.** "The identity selects the row, so on that row it always equals the
  input and is always defined. Stored vs not stored is the existence step: drop `when_subject:`
  from the update and keep `unknown_instance: true` on the create. That is create-or-update since
  ess/25, and the suite witnesses both sides."
- **Size.** S.

## Acceptance

In crates/specify/ess-domain/tests/subject_guard_input.rs:

- `an_identity_read_in_when_subject_names_the_existence_step` (first red test): the adopter shape
  yields exactly one diagnostic, ESS-COMMAND-003, whose hint names `unknown_instance: true` and
  create-or-update.
- `a_refused_when_subject_predicate_draws_no_coverage_refusal`: no ESS-COMMAND-005 for that command.
- `a_non_identity_unknown_root_keeps_the_stored_fields_hint`: an unknown non-identity root keeps
  today's hint text.
- In crates/verify/ess-conformance/tests/synthesis_subject_fact_plan.rs:
  `the_identity_idiom_synthesizes_both_sides`, the idiom model yields one scenario per branch.
