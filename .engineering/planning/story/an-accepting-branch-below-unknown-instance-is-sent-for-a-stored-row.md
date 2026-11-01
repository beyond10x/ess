---
format: aep.planning-md/3
id: story:an-accepting-branch-below-unknown-instance-is-sent-for-a-stored-row
kind: story
status: implemented
title: An accepting branch naming no subject below unknown_instance is sent for a stored row
summary: 'Synthesis arranges the addressed row for a subject-less accepting when: or default that the existence branch answers before (ess#645)'
tags:
- ess-issues-3
refs:
- provider: github
  reference: beyond10x/ess#645
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: approval-record:d214-batch-3-consumer-blocked-first
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T17:06:40Z", actor: "human:timo", revision: 9, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
- {from: "proposed", to: "active", at: "2026-10-29T17:06:40Z", actor: "human:timo", revision: 10, executor: "agent:ess-issues-lead", correlation: "ess-issues-3"}
- {from: "active", to: "implemented", at: "2026-11-01T01:28:13Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1}}}
---
## Problem

beyond10x/ess#645. A command declares an accepting branch that names no subject (a `when:` branch
with `accepts: nothing`, or a default with no effect) together with a non-creating
`unknown_instance:` branch. Validation admits the shape. Under the precedence rule, existence
answers before every accepting branch, so the accepting branch is reachable only for a row that is
stored. Synthesis writes its scenario with a literal identity and arranges no row, so the
interpreted target answers the existence branch and the scenario fails.

Minimal model (neutral names):

```yaml
  - name: demo.notes.CloseNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: why, type: Optional<String>, presence: omitted_when_absent}
    outcomes:
      - name: dropped
        when: 'not defined(why)'
        accepts: nothing
      - name: unknown
        unknown_instance: true
        refuses: false
      - name: closed
        deletes: demo.notes.Note
        instance: note_id
        emits: [demo.notes.NoteClosed]
        payload:
          demo.notes.NoteClosed: {note_id: input.note_id}
```

Checked on `ess 0.77.0` (a repro outside the repository):

```text
$ ess verify conform synthesize --path . --out suite.json
4 scenario(s) (0 authored), 0 refusal(s), written to suite.json
$ ess verify conform run --target interpreted --path . --suite suite.json --report-format 2
demo v1 against interpreted 0.77.0 — failed
  passed demo.notes.CloseNote/outcome/closed
  failed demo.notes.CloseNote/outcome/dropped
  passed demo.notes.CloseNote/outcome/unknown
  passed demo.notes.OpenNote/outcome/opened
  4 scenarios: 3 passed, 1 failed, 0 error, 0 unsupported
```

The `dropped` scenario sends `{"note_id": {"kind": "literal", "value": "note_id"}}` with no
`OpenNote` step before it; the target answers `unknown`. A subject-less accepting default declared
below `unknown_instance:` (`kept: {accepts: nothing}` beside `closed: {when: 'defined(why)', ...}`)
fails the same way on 0.77.0.

The issue's Expected: synthesis and the interpreter apply one order, so the `dropped` scenario is
sent against a stored note (or the specification is refused with a hint), and no valid
specification yields a failing scenario on the interpreted target. Related: beyond10x/ess#554.

## Fit review

Checked 2026-10-29 on the `ess 0.77.0` release binary and on this tree (`integration/ess-issues-3`,
0.77.0 merged). `synthesize.rs` moved 25 lines between 0.74.0 and 0.77.0; the rest of the code read
here is unchanged since 0.74.0 (`git diff --stat 0.74.0 HEAD`).

1. **Need.** A command accepts a request without acting on a row (`dropped`, `accepts: nothing`)
   only for a row that is stored, and answers `unknown_instance:` for one that is not. The
   specification says so, and synthesis writes a scenario that contradicts it: the `dropped`
   scenario sends a literal `note_id` and arranges no `OpenNote` before it. On 0.77.0,
   `ess verify conform run --target interpreted` reports "failed
   demo.notes.CloseNote/outcome/dropped … 4 scenarios: 3 passed, 1 failed". The same happens for
   a subject-less accepting default (`kept: {accepts: nothing}`) below `unknown_instance:`, and for
   the guard `count > 3` on an Integer input in place of `not defined(why)`. All are minimal neutral
   models from a repro outside the repository. The requester proposed no syntax. Their Expected
   offers two options: send against a stored note, or refuse the specification with a hint.
2. **Class: defect.** `place` puts a non-erroring `when:` at `Phase::Accepting`
   (`crates/specify/ess-domain/src/command/precedence.rs:496-498`) and `unknown_instance:` at
   `Phase::Existence` (`:515-523`). The design page orders existence before the accepting branches
   (`docs/design/cross-record-and-stored-field-guards.md:822`, `:852`). Validation admits a marker
   below a guarded branch on purpose (`crates/specify/ess-domain/src/command.rs:3897-3898`: "A
   marker below a guarded branch is not refused"). The interpreter follows the rule and synthesis
   does not.
3. **Already expressible.** Not applicable to a defect. The workaround (`error:` in place of
   `accepts: nothing`) changes the domain fact. It turns an accepted drop into an input refusal at
   `Phase::InputRefusal` (`precedence.rs:512`), which answers before existence.
4. **Fit.** No authored surface is added and no grid cell opens. The defect is in `arranged`
   (`crates/verify/ess-conformance/src/synthesize.rs:4946-4979`). That function already sends a
   subject-less accepting `When`/`Otherwise` branch for an existing row, but only on a row-set
   command (`:4962-4977`, beyond10x/ess#552, through `addressed_row` `:5328`). Every other command
   takes `arranged_plain` (`:5514`). There `prepare(ir, outcome, …)` (`:6145`) arranges no row for
   a branch that names no subject, and `reach` (`:7948`) picks a literal identity. Every non-creating
   `unknown_instance:` command has a supplied non-creating sibling subject for `addressed`
   (`:5264`) to find, because validation refuses one without it as `unreachable_branch`
   (ESS-COMMAND-012, checked on 0.77.0 with a creating default). Other targets are untouched:
   the interpreter is already right, and generated code, `ess verify diff` and the entity runtime
   read no synthesized arrangement.
5. **Second adopter.** A support desk's `Acknowledge(ticket_id, duplicate)` has
   `when: duplicate == true, accepts: nothing` above `unknown_instance:`, and a default that moves
   the ticket. It fails the same way.
6. **Cost.** No format, keyword or diagnostic code. Suite bytes change only for a command with this
   shape. A text scan of the repository's YAML found none. 64 files declare `unknown_instance:`,
   with 83 commands carrying a non-creating one, and none of those commands also has a
   subject-less accepting `when:` or default. So `synthesis-suite-bytes-table.tsv` and
   `synthesis-component-suites-table.tsv` should not move. This is inferred from a scan of the
   text, not from a build. One case gains a refusal in place of a failing scenario: no command
   creates the addressed entity. On 0.77.0 that model writes `dropped` and it fails, while `closed`
   is already refused ESS-SYNTH-004.
7. **Considered.**
   - (a) Change nothing: a valid specification keeps a scenario that fails on a conforming target.
   - (b) Refuse the specification with a hint (the issue's second option): this contradicts
     `command.rs:3897-3898`, which admits the shape on purpose, and it would be a new refusal of
     input that 0.77.0 accepts.
   - (c) Chosen: arrange the addressed row, as the #552 clause already does for row sets. This is
     the issue's first option, taken as proposed.

Out of scope, found while checking: with the guard `count > 3`, the `closed` scenario's boundary
row (`count = 3`) is sent for the note the scenario has just deleted, and fails. The same happens
without `unknown_instance:` ("no declared outcome was reached"). That is a separate defect in
`boundaries` (`synthesize.rs:17008`, which arranges a fresh row only for a `moves:` branch,
`:17117-17120`). It is split out as
`story:a-boundary-row-of-a-deleting-branch-is-sent-for-a-row-that-exists`.

## Decisions

**Accept as proposed.** The issue's first option is taken: the scenario is sent against a stored
row. Its second option, refusing the specification, is declined (Fit review Q7b).

- In `arranged` (`synthesize.rs:4946`), the #552 clause's condition is widened. Today it reads "a
  row-set command whose row sets do not route this branch". It becomes that, or "the precedence
  plan reads a non-creating `unknown_instance:` branch before `outcome`". The second test is a
  small predicate in `synthesize/existence.rs`, built on `precedence::answers_before`
  (`synthesize/precedence.rs:32`) and `existence::creates_unknown` (`existence.rs:36`). The rest of
  the clause stays the same: `outcome` names no subject and no error, and its condition is `When`
  or `Otherwise`. Reading the plan instead of matching condition kinds keeps the clause right for a
  row-set upsert, where `unknown_instance:` answers at `PresentRelated`
  (`ess-domain` `precedence.rs:520-521`).
- For that new case, a row that `addressed_row` cannot arrange in any state is refused with the
  first `held_row` cause (for a model with no creator, the ESS-SYNTH-004 its sibling `closed`
  already gets). The unarranged send is not kept. The #552 row-set case keeps its current
  fall-back unchanged, so its bytes do not move.
- `existing_instance:` is not covered. It answers for a row that exists, so a literal identity is
  already the right send for an accepting branch below it.
- D135 / D214: no wire bytes, generated API, format or documented contract changes. Suite bytes
  change only for a command with this shape, and no repository model has one (Fit review Q6).
  Such a scenario failed on a conforming target before the change and passes after it, so a
  consumer that pinned its bytes pinned a failing scenario. This is not breaking. The refusal for
  a model with no creator is a synthesis refusal, not a refusal of input: `ess specify validate`
  still accepts the model, and `synthesize` exits 0. If the batch's adopter replay finds an adopter
  suite that gains this refusal, it ships as a synthesis `note:` that keeps the 0.77.0 scenario,
  per D214, and `no_creator_refuses_dropped_by_name` is changed to pin the note.

## Acceptance

New test file `crates/verify/ess-conformance/tests/accepting_branch_below_unknown_instance.rs`. Its
models are the neutral shapes in `## Problem`, as fixtures under `tests/fixtures/` or inline.

- `dropped_is_sent_for_a_stored_note`: for the `CloseNote` model, the synthesized
  `CloseNote/outcome/dropped` executes `OpenNote` and captures the note before its act. It sends
  `note_id` as that captured instance and leaves `why` out.
  `cargo test -p ess-conformance --locked --test accepting_branch_below_unknown_instance dropped_is_sent_for_a_stored_note`
- `interpreted_target_passes_every_close_note_scenario`: the same suite run against the
  interpreted target passes all 4 scenarios, where 0.77.0 has 3 passed and 1 failed.
  `cargo test -p ess-conformance --locked --test accepting_branch_below_unknown_instance interpreted_target_passes_every_close_note_scenario`
- `a_default_naming_no_subject_is_sent_for_a_stored_row`: in the model whose default
  `kept: {accepts: nothing}` sits below `unknown_instance:`, `kept` is sent for a captured note, and
  every scenario passes on the interpreted target.
  `cargo test -p ess-conformance --locked --test accepting_branch_below_unknown_instance a_default_naming_no_subject_is_sent_for_a_stored_row`
- `an_orderable_guard_is_sent_for_a_stored_note`: with `when: 'count > 3'` on an Integer input,
  `dropped` is sent for a captured note and passes on the interpreted target. Only `dropped` is
  asserted: that model's `closed` boundary row belongs to
  `story:a-boundary-row-of-a-deleting-branch-is-sent-for-a-row-that-exists`.
  `cargo test -p ess-conformance --locked --test accepting_branch_below_unknown_instance an_orderable_guard_is_sent_for_a_stored_note`
- `an_input_refusal_beside_unknown_instance_keeps_its_send`: with `error:` in place of
  `accepts: nothing`, the scenario's JSON is identical to the one 0.77.0 writes (no `OpenNote`,
  literal `note_id`), because an input refusal answers before existence.
  `cargo test -p ess-conformance --locked --test accepting_branch_below_unknown_instance an_input_refusal_beside_unknown_instance_keeps_its_send`
- `no_creator_refuses_dropped_by_name`: with no command creating a note, no `dropped` scenario is
  written, and a refusal for `CloseNote/dropped` names `demo.notes.Note`. The code is expected to
  be ESS-SYNTH-004, as for `closed`; that is inferred and the test pins whatever code the
  `held_row` cause gives.
  `cargo test -p ess-conformance --locked --test accepting_branch_below_unknown_instance no_creator_refuses_dropped_by_name`
- Bytes tables: `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table`. Every
  existing row of `synthesis-suite-bytes-table.tsv` and `synthesis-component-suites-table.tsv` is
  unchanged. A new fixture carrying `format: ess/` adds rows of its own (regenerated with
  `SYNTHESIS_SUITE_BYTES_TABLE_WRITE` / `SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE`), and the diff
  shows added lines and the header count only.
- Kept green, one `--test` at a time (AGENTS.md package-gate table, synthesis row):
  `cargo test -p ess-conformance --locked --test minted_head_siblings` (the #552 clause),
  `--test precedence_order`, `--test precedence_plan_answers`,
  `--test adversary_row_set_upsert_462`, `--test explore_optional_unknown`; then
  `cargo test -p ess-cli --locked` and
  `cargo clippy -p ess-conformance -p ess-cli --locked --all-targets -- -D warnings`.

## Scope

Derived 2026-10-29 from this tree. Each line is **cited** (read in the tree) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize.rs`, `arranged`
  (`:4946-4979`, the #552 clause at `:4962-4977`). Cited.
- **Symbols:** `arranged`, `addressed_row` (`:5328`, which gains a form that returns the
  `held_row` cause), `addressed` (`:5264`), `held_row` (`:5367`), `precedence::answers_before`
  (`synthesize/precedence.rs:32`), `existence::creates_unknown` (`synthesize/existence.rs:36`).
  Cited.
- **Also:** `crates/verify/ess-conformance/src/synthesize/existence.rs`, a small predicate for
  "a non-creating `unknown_instance:` answers before this branch". Inferred placement.
- **New test:** `crates/verify/ess-conformance/tests/accepting_branch_below_unknown_instance.rs`
  plus its fixtures. Inferred.
- **Not touched:** the further-witness arrangements `arranged_without_fallbacks`,
  `arranged_with_union_variant`, `arranged_at_expression_point` and `arranged_inside_later`
  (`arranged_as`, `synthesize.rs:10348-10382`) bypass `arranged`. None of the reproductions writes
  one for `dropped`. If the implementor finds one that sends a subject-less accepting branch below
  `unknown_instance:` a literal identity, it gets the same clause; otherwise it stays out (inferred).
- **Size:** S.
- **Collisions in batch 3, by function:**
  - U11 itself: the #693 story stays in `synthesize/related_guard.rs` and does not touch
    `arranged`, so the two stories share no function.
  - U13 (`story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children`):
    its deleter case ("an accepting scenario past a `when_subject:` refusal") is arranged by the
    `routed` branch at the head of `arranged` (`subject_fact::prepare`, `synthesize.rs:4953-4957`)
    or by `arranged_plain`'s `prepare_state_input`. Same function, different arm. Merge after U13
    or rebase onto it. Inferred: U13 has no scope recorded yet.
  - U14 (`story:a-cascade-onto-rows-whose-creator-needs-the-subject-active-is-witnessed`): its
    surface is the cascade's creator arrangement (`synthesize/set_effects.rs`, and
    `related_guard::present_in`/`drive`), not `arranged`. No shared function (inferred).
  - U15: authored scenario reading, no shared file.
- **Bytes tables:** existing rows do not move (Fit review Q6, a text scan, level 3, unproven
  until the table test runs).
