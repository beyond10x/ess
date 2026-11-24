---
format: aep.planning-md/3
id: story:from-ess-38-an-outcome-group-s-external-refusal-may-be-marked-before-row-and-each-member-answers-it-first
kind: story
status: active
title: 'From ess/38 an outcome group''s external refusal may be marked before_row: true, and each member then answers it before its own branches'
tags:
- external-request
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#816
relations:
- serves: vision:O2
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
- depends_on: story:the-guards-page-says-a-before-row-external-declared-above-the-input-refusals-answers-before-them
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:24:50Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T03:24:52Z", actor: "human:timo", revision: 3}
---
## Need

beyond10x/ess#816 item 2. A refusal shared by many commands, such as a missing session, is declared
once in an outcome group (`ess/12`). An authenticate-first refusal is an `external:` marked
`before_row: true` (step 15). The group cannot carry the key: `RawGroupOutcome`
(`crates/specify/ess-domain/src/outcome_group.rs:105-121`) is `deny_unknown_fields` and has
`name`, `external`, `error`, `summary`, `refs` only, and `written()` (125-162) sets
`before_row: false` (149). The author must copy the refusal into every command.

Placing it is the second half. A member gains the group's outcomes after its own
(`expand_marking`, 316-322: `command.outcomes.extend(...)`; the guide says so,
`website/docs/guides/specify/guards-and-predicates.md:371-372`). A `before_row` refusal declared
after an accepting branch is refused: "`<name>` is answered before the row (`before_row: true`), at
step 1 of the precedence order, but is declared after `<accepting>`, which a later step answers"
(`validate_before_row_order`, `crates/specify/ess-domain/src/command.rs:3725`, the refusal in its
`else if let Some(earlier) = later` arm). Every member with an accepting branch would be refused if
the outcome were appended, so the key needs a placement as well as a field.

## Reproduction

On 0.98.0 a group outcome with `before_row: true` is a parse refusal (serde unknown field
`before_row`, expected one of `name`, `external`, `error`, `summary`, `refs`), at every format.
Model for the tests (generic names, `format: ess/38` once the format is open):

```yaml
outcome_groups:
  - name: signed-in
    commands: [desk.a.Open, desk.a.Close]
    outcomes:
      - name: unauthenticated
        external: the request carries no valid session
        before_row: true
        error: desk.a.Unauthenticated
```

with `desk.a.Open(label: String)` declaring `label-too-long` (`when: input.label.count > 8`) and
`opened`, and `desk.a.Close(desk_id)` declaring `unknown-desk` (`unknown_instance: true`) and
`closed`.

## Outcome

From `ess/38` a group outcome may declare `before_row: true`. Each member gains such an outcome
before its own outcomes (choice G108, decided A below), in the group order the expansion already
uses; outcomes without the key are appended as today. The expanded command is then what the author
would have written by hand with the refusal first, so validation, compilation, synthesis and every
generator see an ordinary `before_row` external refusal. Below `ess/38` the key is refused as
`unsupported_format_version` naming `ess/38`, at
`outcome_groups.<group>.outcomes.<outcome>.before_row`.

## Acceptance

1. Red today: `outcome_groups.rs` validates the model above at `ess/38` and expects it valid; today
   it fails with the unknown-field parse refusal for `before_row`.
2. Expanded, `desk.a.Open`'s outcomes are, in order, `unauthenticated`, `label-too-long`,
   `opened`, and `desk.a.Close`'s are `unauthenticated`, `unknown-desk`, `closed`; the
   `unauthenticated` outcome of each has `before_row: true`, and each member's outcomes equal, field
   by field, those of the same command with the refusal written by hand as its first outcome.
3. The same model at `ess/37` is refused as `unsupported_format_version` at
   `outcome_groups.signed-in.outcomes.unauthenticated.before_row`, the message naming `ess/38`, and
   no member gains the outcome.
4. A group outcome without the key keeps today's position (after the member's own) and today's
   expansion bytes: the existing `outcome_groups.rs` cases and the fixture
   `tests/fixtures/outcome-groups/` stay green unchanged.
5. `ess specify compile` of the model gives the IR of the hand-written model, apart from the
   source map and digest (one assertion in the same target if `ess-domain`'s tests can reach the
   compiler; otherwise in `ess-cli`'s existing outcome-group or compile target, named in the
   commit).
6. The guide's section "One outcome for many commands" says that a group outcome marked
   `before_row: true` answers before each member's own branches, from `ess/38`.

## Scope

- `crates/specify/ess-domain/src/outcome_group.rs`:
  - `RawGroupOutcome` (107-121): `#[serde(default)] pub before_row: bool`, with the field's
    rustdoc;
  - `written()` (125-162): passes it on (line 149);
  - `expand_marking` (249-326): the format gate beside the `ess/12` gate (280-291), reading
    `declared.format`, and at 316-322 the outcomes with the key go before the member's own;
- `schemas/generated/ess.schema.json`, regenerated by `cargo xtask schema` (`RawGroupOutcome` is
  part of `RawSpecFile`);
- `crates/specify/ess-domain/src/system.rs`: the `ess/38` row's sentence, proposed for the
  coordinator (append-only shared surface);
- `website/docs/guides/specify/guards-and-predicates.md` § "One outcome for many commands"
  (350-384).

## Owned files

- `crates/specify/ess-domain/src/outcome_group.rs`
- `crates/specify/ess-domain/tests/outcome_groups.rs` (new `#[test]`s only)
- `schemas/generated/ess.schema.json` (regenerated)
- `website/docs/guides/specify/guards-and-predicates.md` (the outcome-group section only; shared
  with steps 15 and W5, serial)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (`ess-domain/src` is
  hashed by the stamp; rewritten by the command under Gate)
- `CHANGELOG.md` (one bullet)

## Not yours

- Any change to `validate_before_row_order` or `validate_before_row` (`command.rs:3725`, `:5045`):
  the expanded command satisfies both as written.
- Any other key on a group outcome (`when:`, a payload): a group outcome stays an external refusal.
- Synthesis code: the expansion runs before compilation, so synthesis sees an ordinary outcome.

## Tests first

In `crates/specify/ess-domain/tests/outcome_groups.rs` (existing target; helpers `groups`,
`expanded`, `outcomes_of`, `refused`, `only`):

- `a_before_row_group_outcome_is_admitted_from_ess_38` (acceptance 1).
- `a_before_row_group_outcome_answers_before_each_members_own_branches` (acceptance 2).
- `a_before_row_group_outcome_below_ess_38_names_ess_38` (acceptance 3).

Each is one in-process expansion and validation, well under 1 s (bound 60 s per test, 120 s per
test step). Time them and name any over 5 s.

## Gate

One at a time; no job in the build pool over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-domain --all-targets --locked -- -D warnings`
- `cargo test -p ess-domain --locked --test outcome_groups`
- `cargo test -p ess-cli --locked --test <the outcome-group or compile target named in the commit>`
  (the table: `ess-domain` also gates `ess-cli`; one named target at a time, never all of `ess-cli`)
- `cargo xtask schema`, then `task projection-check` (`RawSpecFile` changed)
- `task test-xtask` (a refusal text and `website/` changed)
- `cargo test -p ess-compiler --locked --test typed_diagnostics`
- the sources stamp, rewritten in this change:
  `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`;
  re-run after every rebase, never merge the stamp by hand
- byte pins (`ess-domain` is read by every synthesis): `cargo test -p ess-conformance --locked
  --test synthesis_suite_bytes_table`, `--test synthesis_suite_bytes_table_cli_read`,
  `--test adversary_related_values_all_bytes`, `--test external_beside_held_guard`, each alone;
  none should move (acceptance 4)

## Depends on

- `story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess`
  (opens `ess/38`; replaces ORDER's "48").
- Step 15 (the page subsection this section links to).

## Changelog

Under `Added`:

- From `ess/38` an outcome group's refusal may be marked `before_row: true`; each member then
  answers it before its own branches (#816).

## Fit review

1. Need: one shared refusal, decided by the request alone, answered first by many commands. The
   reporter's proposal, theirs: admit `before_row` in `outcome_groups`.
2. Class: gap for groups. A command can say it (step 15); a group, the construct for "one refusal
   for many commands", cannot.
3. Already expressible: only by copying the refusal into every member, which a group exists to
   avoid (`guards-and-predicates.md:350-354`).
4. Fit: the key is the one commands already use, with the same meaning; the group stays an
   external refusal (`outcome_group.rs:105`). It composes with the precedence rule by placement,
   since the order check reads declaration order (`command.rs:3725`). Siblings: a group outcome
   has no `when:`, so no other key needs the same treatment.
5. A second adopter: every command a tenant-scoped API serves refuses a suspended tenant before it
   reads the body.
6. Cost: `ess/38` (the pull request's one format); one key on a group outcome; no new code; the
   schema regenerates.
7. Considered: change nothing (copying stays); append as other group outcomes are (refused for
   every member with an accepting branch, above); insert first (chosen).

## Decisions

- Decision: accept as proposed, with the placement added.
- G108 (decided: A, 2026-11-24): where a member gains a `before_row` group outcome.
  - A (decided): before all of the member's own outcomes, so the group's refusal answers first
    even beside a `before_row` refusal of the member's own.
  - B: after the member's own `before_row` refusals and before its input refusals.
  - (Appending after the member's own outcomes is no option: refused by `validate_before_row_order`
    wherever the member has an accepting branch.)
