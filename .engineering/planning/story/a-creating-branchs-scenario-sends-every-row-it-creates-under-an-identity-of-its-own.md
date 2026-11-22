---
format: aep.planning-md/3
id: story:a-creating-branchs-scenario-sends-every-row-it-creates-under-an-identity-of-its-own
kind: story
status: draft
title: A creating branch's scenario sends every row it creates under an identity of its own
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/repeated_identity.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: inferred
  path: crates/verify/ess-conformance/tests/synthesis.rs
- confidence: cited
  path: website/docs/reference/predicates.md
revision: 8
---
## Outcome

A command that creates a row and guards the creation on that same row's absence
(`when_related: {via: input.<the identity it creates>, exists: false}`) gets its scenarios, as the
same command written with `existing_instance:` does. The further run a creating branch makes for an
absent optional reference sends an identity of its own. No synthesized scenario is withdrawn as
creating a second row under one identity where the specification's guards leave other identities
free.

## Need

A downstream specification got 85 refusals on 0.97.3 after one domain was added; 30 of its 34
`ESS-SYNTH-001` and all 5 of its `ESS-SYNTH-003` come from the first shape below, and 1 (inferred 3)
from the second. Reproduced on the released 0.97.3 binary, paths relative to the scratch directory:

**A. Create guarded on its own identity's absence.** `models/g1-create-if-absent/`:
`OpenRoom/opened` has `when_related: {via: input.room_id, exists: false}` and `creates:
desk.room.Room` with `instance: room_id`; the default `room-exists` is the error.

```text
refused: refusal[ESS-SYNTH-001]: outcome desk.room.OpenRoom/opened has no scenario `desk.room.OpenRoom/outcome/opened`
  no witness: `room_id` is `desk.room.RoomId`, which names one row per value, and this scenario creates a second row under a value a row it created still holds: no value the guards admit keeps its rows apart
  help: the guards on the identity leave every row this scenario arranges one value; cover the branch with an authored scenario (ess-scenario/1) that names each row by an identity of its own
5 scenario(s) (0 authored), 1 refusal(s), written to suite.json
```

The same meaning written as create-or-refuse, `models/g1b-existing-instance/` (`opened` with no
guard, `{name: room-exists, existing_instance: true, error: …}`): `6 scenario(s) (0 authored), 0
refusal(s)`. `exists: true` on the same guard is refused at validation (`ESS-COMMAND-004`,
`models/g1c-exists-true/`), so the `exists: false` form is the only spelling of this guard, and the
reference says the guard "composes with … any subject a branch names, a `creates:` included".
Every command that needs such a row inherits the refusal through its arrangement: in the
downstream specification, a parent created this way was required by 7 commands, and their
outcome, transition and grant scenarios were all withdrawn; its two `when_related` predicate
branches over that parent were refused as `ESS-SYNTH-003` ("no candidate of the 1 tried satisfies …
stored related row selecting this branch"). Rewriting the two creating commands with
`existing_instance:` on a private copy took that specification from 123 scenarios and 85 refusals
to 154 scenarios and 51 refusals, `ESS-SYNTH-001` from 34 to 5 and `ESS-SYNTH-003` from 5 to 0.

**B. The absent-reference run reuses the identity.** `models/g6-detail-view/`: `FileCard` creates
a `Card` with an `Optional` reference `room_id` guarded by `no-room` and `room-closed`; the only
view of `Card` is read by its identity (`card_id == param.card_id`).

```text
refused: refusal[ESS-SYNTH-001]: outcome desk.room.FileCard/filed has no scenario `desk.room.FileCard/outcome/filed`
  no witness: `card_id` is `desk.room.CardId`, which names one row per value, and this scenario creates a second row under a value a row it created still holds: no value the guards admit keeps its rows apart
```

No guard reads `card_id`. With the views of `models/g2-optional-parent/` instead (no identity
parameter) the absent-reference run sends `card_id-1` beside the subject's `card_id` and the
scenario stands.

## Fit review

Adopted as a defect. Both shapes validate, and synthesis's own rules say what to send: the reference
(`website/docs/reference/predicates.md`, "Synthesis sends the `exists: false` branch an identity no
row carries, beside two rows of the entity that carry others") and the module comment of
`synthesize/repeated_identity.rs` (rows are kept apart by `own_identity`; only guards that leave
one value withdraw a scenario). Covers the downstream findings: 30 `ESS-SYNTH-001` naming
`names one row per value` on two parents created this way, the 5 `ESS-SYNTH-003`
over one of those parents' predicates, and 1 `ESS-SYNTH-001` of shape B; 2 more of
`names one row per value` in another domain of the same specification are inferred to be shape B
(an `Optional` self-reference on the creating command) and are to be confirmed by the first
acceptance run. Considered and rejected: refusing shape A at validation in favour of
`existing_instance:` (it would refuse what every release since `ess/16` accepted, and the meaning
is identical); telling the adopter to rewrite only (the help line sends authors to an authored
scenario, which is wrong for both shapes).

## Design

- Shape A: where a `when_related: {via: input.f, exists: false}` branch `creates:` the entity
  `via` resolves to and `f` is its `instance:` field, synthesis treats the branch's existence test
  as the addressed identity's: the branch is sent an identity no row carries, its decoys are
  created under distinctions of their own, and the default branch is witnessed as
  `existing_instance:` is (create, snapshot, send the identity again, expect the error, no event,
  row unchanged). Arrangements that need the row create it once through that branch.
- Shape B: the absent-reference further run (`ABSENT_REFERENCE_WITNESS` in `synthesize.rs`)
  takes its own distinction for every identity input, including where a view parameter is bound
  to the identity.
- The `names one row per value` refusal stays for guards that really leave one value; its help
  names `existing_instance:` where the withdrawn creation is shape A.
- No new validation code. No format consequence: suites of models without these shapes keep their
  bytes; models with them gain scenarios (a synthesis change, `AGENTS.md` § Determinism and
  formats needs no new `ess/N`).

## Acceptance

- `ess verify conform synthesize --path models/g1-create-if-absent --out suite.json` prints
  `0 refusal(s)` and the suite holds `desk.room.OpenRoom/outcome/opened` and
  `desk.room.OpenRoom/outcome/room-exists`; `ess verify conform run --target interpreted` passes
  both.
- The same for `models/g6-detail-view`: `desk.room.FileCard/outcome/filed` is in the suite and no
  refusal names `names one row per value`; its absent-reference run sends a `card_id` no other run
  of the scenario sends.
- A model whose guard on the identity leaves one value (`item_id.count > 10`, the existing
  `repeated_identity` fixtures) is still refused with the same text.
- `synthesis_suite_bytes_table` and `synthesis_suite_bytes_table_cli_read` keep every pinned row;
  new rows pin g1 and g6.

## Decisions

- D1 (shape A): (a) synthesize it as written, help names `existing_instance:` (recommended);
  (b) also add a validation warning recommending `existing_instance:`; (c) refuse it at
  validation under a new format version.
- D2: (a) one story for both shapes, as here (recommended: same withdrawal rule, same tests);
  (b) shape B as a story of its own.
