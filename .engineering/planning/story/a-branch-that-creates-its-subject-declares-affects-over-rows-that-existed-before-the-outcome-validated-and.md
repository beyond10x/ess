---
format: aep.planning-md/3
id: story:a-branch-that-creates-its-subject-declares-affects-over-rows-that-existed-before-the-outcome-validated-and
kind: story
status: draft
title: 'A branch that creates its subject declares affects: over rows that existed before the outcome, validated and interpreted from the new format'
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:create-beside-a-move-on-the-same-key
- depends_on: story:create-beside-a-move-on-the-same-key
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 3
---
## Need

One decision that creates a row and changes other existing rows cannot be one outcome. `affects:`
beside `creates:` is refused (`crates/specify/ess-domain/src/command/set_effects.rs:1104-1155`,
`affects_beside`) as `unsupported_construct` (ESS-COMMAND-009, class 9,
`crates/specify/ess-compiler/src/resolve.rs:317`): "outcome `failed` creates its subject and
declares `affects:`; a secondary effect is admitted beside `moves:` and `updates:`, and, from
ess/23, `deletes:`, whose subject exists before the outcome", hint "move `affects:` to the branch
that moves, updates or deletes an existing row".

Where the limit was set: `docs/design/set-effects-over-filtered-instances.md:41-58` (#175) admits
`affects:` on "an outcome with one existing subject" and refuses it "on a creation or
preservation". The reason the message gives is that `where:` reads `subject.<field>`, the
subject's stored value before the outcome, and a creation has none. `ess/23` lifted the same refusal
for `deletes:` (`set_effects.rs:2584-2600`, `affects_beside_deletion_below_ess_23`;
beyond10x/ess#452), the precedent this story follows. The interpreter holds the same assumption:
"secondary effects require an existing subject"
(`crates/verify/ess-conformance/src/interpret/execute/set_effects.rs:441-461`).

The same need was reported before and is a draft in the store:
`story:create-beside-a-move-on-the-same-key` (a new record supersedes the current record on the
same key, ESS 0.59.0). It is this story's parent; its format line (`ess/25`) is out of date.

Minimal reproduction, written for this story (not run here). `demo.ship.RecordInspection
(inspection_id, shipment_id, result)`:

```yaml
outcomes:
  - name: failed
    when: input.result == fail
    creates: demo.ship.Inspection
    instance: inspection_id
    sets: {shipment_id: input.shipment_id, result: input.result}
    affects:
      - entity: demo.ship.Shipment
        where: shipment_id == input.shipment_id
        moves: demo.ship.Shipment.hold
  - name: passed
    creates: demo.ship.Inspection
    instance: inspection_id
    sets: {shipment_id: input.shipment_id, result: input.result}
```

and the parent's shape, a creation that supersedes the current row on its key:

```yaml
  - name: accepted
    creates: demo.ship.Rate
    instance: rate_id
    affects:
      - entity: demo.ship.Rate
        where: {all: [lane_id == input.lane_id, state == Current]}
        moves: demo.ship.Rate.supersede
```

The requester's workaround is two commands with a window between them in which the rule does not
yet hold.

## Outcome

From the format named under Decisions, a `creates:` branch declares `affects:` with the meaning it
has beside `moves:`, `updates:` and `deletes:`, and these rules:

- the rows are the store immediately before the outcome, so the created row is never a candidate,
  also when its entity is the entry's entity;
- zero selected rows is allowed and changes nothing (the first record on a key);
- `subject.` in an entry's `where:` or `sets:` is refused by name: a creation has no prior subject;
- `each:` entries are admitted beside a creation too, under the same refusal of `subject.`;
- a `moves:` in an entry counts as the cause of its transition, as today;
- the events publish in the order a moving branch with `affects:` publishes today.

`affects:` beside `preserves:` and beside `instances:` stay refused, unchanged. Below the format the
refusal keeps its message, its hint naming the format.

## Acceptance

- Red first: `crates/specify/ess-domain/tests/set_effects.rs` (existing binary) gains
  `a_creation_declares_affects_at_the_new_format`: both reproductions validate at the new format,
  and the supersede transition counts as caused (no ESS-ENTITY-005); on the base they fail with the
  ESS-COMMAND-009 refusal quoted under Need.
- Below the new format the same text is refused with that message unchanged and the hint
  "write `format: ess/<n>` on the source that declares the system, where `affects:` stands beside
  `creates:`; or move `affects:` to the branch that moves, updates or deletes an existing row",
  one test.
- `subject.` in an entry beside a creation is refused, one test each for `where:` and `sets:`,
  naming the entry and "a creation has no subject before the outcome"; an `each:` entry beside a
  creation validates, one test.
- `affects:` beside `preserves:` and beside `instances:` keep their refusals, unchanged; one test
  each pins the text.
- The interpreter applies the entries beside a creation: a case in
  `crates/verify/ess-conformance/tests/interpreted_set_effects.rs` (existing binary) executes the
  first reproduction against a hand-built store, no synthesis: a `fail` result creates the
  inspection and moves the shipment to `held`; a `pass` creates only. The second reproduction with
  no current rate creates only; with one current rate, the old one is `Superseded` and the new one
  is the only row of the lane in its initial state. The history checker accepts both histories.
- IR: no new shape (`affects` beside a subject whose effect is `creates` is representable today);
  an old-reader test shows a model without the pair compiles to the same bytes.
- `docs/design/set-effects-over-filtered-instances.md` § "Secondary effect (#175)" gets a
  paragraph with the rules above; `website/docs/guides/specify/selection-effects.md` and the format
  row of `website/docs/reference/formats.md` and the format history in `system.rs` name the
  construct; `CHANGELOG.md` `### Added` gets one line with its effect × target cell (policy 3).
- Every new test under 5 s or named with its time; no new test binary.

## Fit review

Tree `bcfc51555` (`integration/wave-2026-11-24`), read 2026-11-24.

1. **Need.** One decision creates a row and changes rows that existed before it, atomically: a
   failed inspection holds its shipment; a new rate supersedes the current one. Reproductions above.
   The requester's words: "a secondary effect beside a create" (report item 12).
2. **Class.** Gap. The refusal's reason (no prior subject for `subject.`) applies only to entries
   that read `subject.`; an entry reading `input.` alone has a meaning.
3. **Already expressible?** No. Two commands lose atomicity, and the second's transition needs a
   cause of its own (the parent story measured ESS-ENTITY-005 for that shape). The requester's
   workaround leaves a window in which the rule does not hold; every later command has to guard on
   it.
4. **Fit.** Effect × target grid of G2 (`docs/design/generic-language-features.md` § G2, the
   `affects:` row "`each r in rows(E where p)`"): the creation column is the one closed cell left
   beside moves, updates and deletes (`ess/23`). Same key, same entries, same snapshot rule as every
   selector (`docs/design/filtered-related-reads.md:20-51`). Siblings: `preserves:` raises a
   different question (a branch that changes nothing of its own subject but others' rows) and stays
   refused, decision `P2A-D5`. Targets: interpreter and history here; synthesis in
   `g12-create-affects-synthesis`; generated Rust and Go (today `affects` beside a creation is owed,
   `crates/generate/ess-synth/src/determined.rs:1919-1921`) and Entity Runtime (set effects refused,
   `crates/generate/ess-entity-runtime/src/subset.rs:98`) in `g12-create-affects-targets`.
5. **Second adopter.** Opening a new billing period closes the open one of the same account.
6. **Cost.** One format gate, one refusal of `subject.` beside a creation; no key, no diagnostic
   code, no IR shape.
7. **Considered.** (a) Change nothing: rejected, the two-command split has a window and an
   uncaused transition. (b) An `affects:` entry that creates (the parent's other half): not asked
   here and a different cell. (c) Chosen: `ess/23`'s lift for `deletes:`, applied to `creates:`.

## Decisions

**Accept, redesigned** (the request names no syntax; the parent draft's shape is taken unchanged).
Report item 12 covered, and `story:create-beside-a-move-on-the-same-key`'s need. Open: `P2A-D5`
(which siblings; default A, `creates:` with `where:` and `each:` entries, `preserves:` stays refused)
and `P2A-D6` (format). The parent's format line (`ess/25`) is superseded by `P2A-D6`.

## Scope

- cited: `crates/specify/ess-domain/src/command/set_effects.rs` (`affects_beside` 1104-1170,
  `affects_beside_deletion_below_ess_23` 2584-2600, `admits_deletions` 92)
- inferred: `crates/specify/ess-domain/src/command/set_effects.rs` (the `subject.` refusal beside a
  creation, in the entry checks)
- cited: `crates/verify/ess-conformance/src/interpret/execute/set_effects.rs` (`apply` 43,
  `subject` 441-461)
- inferred: `crates/verify/ess-conformance/src/interpret/execute/history.rs`
- inferred: `crates/specify/ess-domain/src/system.rs` (format history row, gate)
- cited: `crates/specify/ess-domain/tests/set_effects.rs`,
  `crates/verify/ess-conformance/tests/interpreted_set_effects.rs` (existing binaries)
- cited: `docs/design/set-effects-over-filtered-instances.md`,
  `website/docs/guides/specify/selection-effects.md`, `website/docs/reference/formats.md`,
  `CHANGELOG.md`
- Size: M (about 450 changed lines).

## Depends on

- `story:create-beside-a-move-on-the-same-key` (parent, draft; this unit is derived from it).
- `story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess`
  (s0, the format gate). No file of s1 to s6 is touched.

## Source

A downstream application, 2026-11-24, report item 12 (its G2).

## Downstream report 2026-11-26

A downstream application reported the same need on 0.99.2 (gap 2 of a report of ten): a creating
command cannot also change another existing row in the same outcome, so the application splits one
decision over two commands or keeps the change outside the model. On `main` at `c0b5f560cb`
`affects:` still sits only beside `moves:`, `updates:` and `deletes:` with an existing subject
(`website/docs/guides/specify/selection-effects.md:36`, `:55`), and beside `creates:` it is refused
as `ESS-COMMAND-009`. The report is a further instance of this story, not a new one; it also blocks
a creating command from keeping a counter on the row it files the new row under, which the draft
story on an aggregate test of a row-set guard names as the idiom it replaces.

## A second consumer, 2026-11-28

A second consumer reported this on 2026-11-28, on 0.99.2, and 0.99.9 refuses it the same way: an outcome that creates its subject and declares `affects:` is refused `ESS-COMMAND-009` ("creates its subject and declares `affects:`"). A specification of reusable building blocks hit it on one creating outcome that also had to change rows that existed before it. Nothing in the story changes; the second consumer is a reason to keep its order.
