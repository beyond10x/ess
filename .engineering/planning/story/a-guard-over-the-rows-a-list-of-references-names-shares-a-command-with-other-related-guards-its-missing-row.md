---
format: aep.planning-md/3
id: story:a-guard-over-the-rows-a-list-of-references-names-shares-a-command-with-other-related-guards-its-missing-row
kind: story
status: active
title: A guard over the rows a list of references names shares a command with other related guards, its missing row answered among theirs in declaration order
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:at-ess-38-a-when-related-via-an-all-element-and-a-missing-payload-source-name-a-member-of-a-struct-input-a
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:52:02Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-24T03:52:02Z", actor: "human:timo", revision: 6}
---
## Need

A command that takes a list of references and also names other rows cannot refuse a list element
that names no row. `when_related: {via: input.<list>, exists: false}` (`ess/28`) is admitted only on
a command that reads no other related row:

- `crates/specify/ess-domain/src/command/related_list.rs:138-151`: where the command reads a second
  list or any other identity-addressed row, every list branch is refused as
  `conflicting_declaration` (ESS-COMMAND-004, class 4, `crates/specify/ess-compiler/src/resolve.rs:307`):
  "`demo.ship.Plan` guards on the rows the list `input.line_ids` names and on another related row;
  a command guarding on a list of references reads no other related row in this cut", hint "split
  the guards across commands, or drop one".
- The module states the cut (`related_list.rs:9-14`): "In this cut a list is tested for a missing
  row alone". The design page that set it, `docs/design/generic-language-features.md:255-259`
  (§ G2, "First cut, `ess/28`"), reserves the next cut only for "a predicate over each listed row",
  which "waits for navigation in guard predicates". No page names a reason for refusing the list
  beside another row; the one precedence rule (§ G4, `:484-497`) already orders it: "first
  existence (`exists: false` over a row the input or a stored field names, in declaration order)".

Minimal reproduction, written for this story (not run here). `demo.ship.Plan(order_id: OrderId,
line_ids: List<LineId>)`:

```yaml
outcomes:
  - name: no-order
    when_related: {via: input.order_id, exists: false}
    error: demo.ship.OrderNotFound
  - name: unknown-line
    when_related: {via: input.line_ids, exists: false}
    error: demo.ship.LineNotFound
    payload:
      demo.ship.LineNotFound: {line_id: {missing: input.line_ids}}
  - name: order-closed
    when_related: {via: input.order_id, predicate: state == Closed}
    error: demo.ship.OrderClosed
  - name: planned
    creates: demo.ship.Plan
    instance: plan_id
```

Today `unknown-line` is refused as above. The requester's workaround moves the missing element into
an `external:` branch, so no scenario checks it.

## Outcome

From the format named under Decisions (`ess/38` if unreleased when this lands, else `ess/39`), a
list guard sits beside identity-addressed `when_related:` guards, row-set guards and a second list
guard in one command. Every missing row, the list's first missing element included, is answered at
step 4 of the precedence rule in the declaration order of the `exists: false` branches, before any
predicate or row-set refusal over present rows. Below that format the refusal keeps its message,
and its hint names the format.

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

- Red first: `crates/specify/ess-domain/tests/related_list.rs` (existing binary) gains
  `a_list_guard_beside_another_related_row_validates_at_the_new_format`: the reproduction validates
  at the new format; on the base it fails with the ESS-COMMAND-004 refusal quoted under Need.
- The same text below the new format is refused with that message unchanged and the hint
  "write `format: ess/<n>` on the source that declares the system, where the one precedence rule
  answers the list's missing row among the other rows' in declaration order; or split the guards
  across commands, or drop one" (`FormatVersion::declaration_hint`).
- At the new format a list guard also validates beside a row-set guard
  (`when_related: {entity, where, exists}`) and beside a second list guard; a test each.
- Kept refused, unchanged at every format, a test each: a list in `all:` (`related_list.rs:105-119`)
  and a `predicate:` over the listed rows (`:120-137`).
- The precedence plan places the list's `exists: false` at step 4 among the existence answers in
  declaration order: a unit test in `crates/specify/ess-domain/src/command/precedence.rs` pins the
  plan of the reproduction as `no-order`, `unknown-line`, `order-closed`, `planned`, and with
  `unknown-line` declared first, `unknown-line` before `no-order`.
- Each branch's `{missing: input.<list>}` reads its own list, and is refused on a branch not guarded
  by that list (`related_list.rs` text unchanged).
- The interpreter answers the reproduction in that order: a case in
  `crates/verify/ess-conformance/tests/related_list.rs` (existing binary) executes four requests
  against a hand-built store, with no synthesis: order missing and a line missing (`no-order`),
  order present and a line missing (`unknown-line` publishing the first missing element), every
  line present and the order `Closed` (`order-closed`), all present (`planned`).
- A model without the pair keeps its IR bytes (`cargo xtask` pins and `typed_diagnostics`
  unchanged; no new `ValidationCode`).
- `website/docs/reference/predicates.md` § "The rows a list of references names" (from line 1663)
  says the list shares a command with other related guards from the new format and how the
  missing rows are ordered; `docs/design/language-grids.md` grid R row `when_related:` drops "no:
  one kind of related row per command" for the list; `docs/design/generic-language-features.md`
  § G2 gets one sentence naming the lift; the format's row in `website/docs/reference/formats.md`
  and the format history in `crates/specify/ess-domain/src/system.rs` name the construct;
  `CHANGELOG.md` `## [Unreleased]` `### Added` gets one line with its grid row (policy 3).
- Every new test under 5 s or named with its time in the report; no new test binary.

## Fit review

Tree `bcfc51555` (`integration/wave-2026-11-24`), read 2026-11-24.

1. **Need.** A command naming one row and a list of rows refuses a missing list element by name,
   beside the refusals over the other row. Reproduction above. The requester's words: "a
   list-reference guard beside another related guard" (report item 10).
2. **Class.** Gap. The refusal is a scope cut ("in this cut", `related_list.rs:146`), not a
   semantic conflict: the precedence rule has the step it belongs to.
3. **Already expressible?** No. Splitting the command changes the domain (two requests for one
   decision). The requester's `external:` workaround moves the figure out of the specification,
   which policy 1 refuses as an idiom (`generic-language-features.md:590-597`).
4. **Fit.** Grid R row "`when_related:` predicate", column "related row": the row's "one kind of
   related row per command" cell (`docs/design/language-grids.md`, grid R) is the cell
   `ess/32` already opened for identity-addressed beside row-set guards
   (`crates/specify/ess-domain/src/command/row_set.rs:328-334`) and
   `story:when-subject-beside-a-row-set-validates-under-the-g2-format` (active) opens for subject
   guards. This story closes the remaining list part of it. Same keys, no new spelling; composes
   with `{missing: …}`, `all:` (still refused for lists), row sets and the precedence rule.
   Targets: interpreter here; synthesis, generated Rust and Go (the list guard is already an owed
   obligation, `crates/generate/ess-synth/src/determined.rs:226-235`) and Entity Runtime
   (`RelatedGuardUnsupported` for every `when_related:`) in `g10-list-beside-synthesis`.
5. **Second adopter.** Booking several seats of one event: `no-event` beside `unknown-seat` over
   `seat_ids`.
6. **Cost.** No key, no diagnostic code, no IR shape (the list guard is `Related` with a list
   `via` already; the precedence plan is derived, never serialised,
   `generic-language-features.md:578-581`). One format gate.
7. **Considered.** (a) Change nothing: rejected, the workaround leaves the figure unchecked. (b)
   The list beside identity-addressed rows only: rejected, the closing rule opens the whole cell
   (`generic-language-features.md:211-216`, closing rule). (c) Chosen: beside every
   related guard kind, ordered by the existing rule.

## Decisions

**Accept, redesigned** (the request names no syntax; the design is the existing keys under the
existing precedence rule). Report item 10 covered. `P2A-D4` (scope) is decided: A, 2026-11-24: beside every
related guard kind and a second list. `P2A-D6` (format) is decided: A, `ess/38` (D77, 2026-11-24). Not in this story, kept refused by name: a predicate over each listed row
(waits for navigation in guard predicates, `generic-language-features.md:258-259`).

- FMT-B (decided: A, 2026-11-24): this story moves to `ess/38` with the rest of the pull request (D77: no `ess/39`); the Acceptance line on the format says so.

## Scope

- cited: `crates/specify/ess-domain/src/command/related_list.rs` (`gates` 57-155; the module
  doc 9-14)
- cited: `crates/specify/ess-domain/src/command/precedence.rs` (placement test; `place`)
- inferred: `crates/specify/ess-domain/src/command/related_guard.rs` (`read_vias`; joint
  partition takes no part for the list, its rows only answer existence)
- inferred: `crates/specify/ess-domain/src/system.rs` (format history row; the gate constant if
  this unit opens `ess/39`)
- cited: `crates/specify/ess-domain/tests/related_list.rs`,
  `crates/verify/ess-conformance/tests/related_list.rs` (existing binaries)
- inferred: `crates/verify/ess-conformance/src/interpret/execute/related.rs` (`first_missing`
  94-110, `reads_in_order` 119-141) if the order is not already the plan's
- cited: `website/docs/reference/predicates.md`, `website/docs/reference/formats.md`,
  `docs/design/language-grids.md`, `docs/design/generic-language-features.md`, `CHANGELOG.md`
- Size: M (about 350 changed lines; about 550 if this is the unit that opens `ess/39`, as
  `story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess`
  did for `ess/38`).
- Overlap: s1 of the wave of 2026-11-24 rewrites `is_list`, `uses` and line 83 of `gates` in
  `related_list.rs`; this unit lands after it.

## Depends on

- `story:at-ess-38-a-when-related-via-an-all-element-and-a-missing-payload-source-name-a-member-of-a-struct-input-a`
  (s1, same file).
- `story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess`
  (s0, the format gate).

## Source

A downstream application, 2026-11-24, report item 10 (its G1).
