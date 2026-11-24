---
format: aep.planning-md/3
id: story:every-row-of-a-selection-has-a-related-row-that-passes-is-stated-with-a-row-set-guard-and-a-rows-quantifier
kind: story
status: draft
title: '"Every row of a selection has a related row that passes" is stated with a row-set guard and a rows quantifier, documented with its empty-set reading and pinned by a test'
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A downstream application states a precondition of the form "for every active required row of a
set, a passing row of another entity exists for this request": a shipment is dispatched only when
every active hard requirement of its order has a passing inspection of this shipment. The
application believes ESS has `exists: true|false` over a row set and no "for all", so it states a
weaker rule (no failing inspection exists) and leaves "a requirement with no inspection at all"
unchecked by the specification.

The language already holds the construct, since `ess/27`:

- a row-set guard takes `forall: <predicate>` beside `exists` and `count`
  (`crates/specify/ess-domain/src/command/row_set.rs:20-22`, `docs/design/filtered-related-reads.md:38-51`);
- a selector's `where:` and `forall:` admit the rows quantifier `exists: {in: {entity: E}, as, that}`
  tied to the candidate (`crates/specify/ess-domain/src/command/related_rows.rs:1-17`,
  `docs/design/related-rows-quantifier.md:174-180`);
- `forall` over an empty selection is `true` (`filtered-related-reads.md:51`,
  `website/docs/reference/predicates.md:30`).

What is missing is the idiom on the page and a test that holds it. The reference section on the rows
quantifier shows one example, a head over slots (`predicates.md:1860-1912`), says `forall` over rows
is refused with its `not exists` rewrite (`:1902-1906`), and nowhere shows "every candidate has a
related row", so an author reading it concludes the claim cannot be stated. The same section prints
one sentence twice (`predicates.md:1870-1871`).

Minimal reproduction, written for this story (not the requester's files; not run here, the
assessment was read-only). Entities `demo.ship.Shipment {shipment_id, order_id}`,
`demo.ship.Requirement {requirement_id, order_id, strength: hard|soft}` with lifecycle
`Active -> Retired`, `demo.ship.Inspection {inspection_id, requirement_id, shipment_id, result:
pass|fail}`. Two spellings of one rule on `demo.ship.Dispatch(shipment_id)`:

```yaml
# A: the refusal names the gap, the accepting branch is the default
- name: requirement-not-passed
  when_related:
    entity: demo.ship.Requirement
    where:
      all:
        - order_id == subject.order_id
        - state == Active
        - strength == hard
        - not:
            exists:
              in: {entity: demo.ship.Inspection}
              as: inspection
              that: {all: [inspection.requirement_id == requirement_id, inspection.shipment_id == input.shipment_id, inspection.result == pass]}
    exists: true
  error: demo.ship.RequirementNotPassed
- name: dispatched
  moves: demo.ship.Shipment.dispatch
  instance: shipment_id

# B: the accepting branch states "for all", the refusal is the default
- name: dispatched
  when_related:
    entity: demo.ship.Requirement
    where: {all: [order_id == subject.order_id, state == Active, strength == hard]}
    forall:
      exists:
        in: {entity: demo.ship.Inspection}
        as: inspection
        that: {all: [inspection.requirement_id == requirement_id, inspection.shipment_id == input.shipment_id, inspection.result == pass]}
  moves: demo.ship.Shipment.dispatch
  instance: shipment_id
- name: requirement-not-passed
  error: demo.ship.RequirementNotPassed
```

The requester's words, their identifiers replaced: "every Active hard requirement of the order has
a `pass` inspection" (report item 9), "ESS has `exists: true|false` over a row set, not 'for
all'".

## Outcome

Both spellings above validate, synthesize and pass on the interpreted target at `ess/27` and later,
and a test in the existing binaries holds that. The reference states the idiom, its reading over an
empty selection, and which branch is the complement. No construct, key, format or diagnostic is
added.

## Acceptance

- Red first: `crates/specify/ess-domain/tests/related_rows_quantifier.rs` gains
  `every_candidate_has_a_passing_related_row_validates_in_both_spellings`, which validates the
  reproduction's spellings A and B at `ess/27` and at the newest format. If either is refused, the
  refusal is the defect this story fixes, named in its report; if both validate on the base, the
  test is green from the start and the story is the documentation and the conformance case below.
- `crates/verify/ess-conformance/tests/related_rows_quantifier.rs` gains one case that synthesizes
  spelling A once and runs the suite on the interpreted target with 0 refusals and every scenario
  passing. The suite holds: an order with no active hard requirement (dispatched: the empty
  selection is true), every requirement passed (dispatched), one requirement with no inspection
  (refused), one with only a failing inspection (refused), and one whose passing inspection
  belongs to another shipment (refused). A target that drops the `shipment_id` conjunct, or the
  `result == pass` conjunct, fails a scenario. One synthesis per test, the test under 5 s or named
  with its time; no new test binary.
- `website/docs/reference/predicates.md` § "From `ess/27`: a test over each candidate's related
  rows" gains "Every candidate has a related row": both spellings, the sentence "over an empty
  selection `forall` is true and `exists: true` is false, so both spellings accept a request whose
  selection holds no row", the complement in each spelling (A: the default accepts; B: the default
  refuses, and it is taken only where `forall` is definitely false, never on an unknown row), and
  how to require a non-empty selection (an `exists: false` refusal over the same `where:`, declared
  first). The duplicated sentence at lines 1870-1871 is removed.
- No `CHANGELOG.md` language line (a documentation change, not counted by policy 3).

## Fit review

Tree `bcfc51555` (`integration/wave-2026-11-24`), read 2026-11-24.

1. **Need.** A universal claim over a row set whose body is the existence of a related row: "every
   active hard requirement of the order has a passing inspection of this shipment". Reproduction
   above. The requester's words are quoted separately under Need.
2. **Class.** Convenience: expressible since `ess/27`, the idiom is undocumented. Not a defect: no
   page promises otherwise.
3. **Already expressible?** Yes, by the reproduction's two spellings (citations under Need). The
   idiom keeps the figure in the specification and the suite checks it (synthesis arranges the
   failing candidates, `related-rows-quantifier.md:332-344`), so policy 1
   (`docs/design/generic-language-features.md:590-597`) admits it as the answer. Not run here.
4. **Fit.** Grid R row "row-set `where:` / `forall:`" already reads related rows through the
   quantifier (`docs/design/language-grids.md`, grid R). No cell opens.
   - A dedicated `forall: {in: {entity: E}}` was refused by the quantifier's design ("`exists`
     only", `related-rows-quantifier.md:154-159`): a universal over rows needs the tie as an
     antecedent and the predicate grammar has no implication. Inside a row-set `forall:` the
     antecedent is the selector itself, which is why spelling B needs no new construct.
   - Composition: a row-set guard does not share a command with a `when_subject*` guard today
     (`row_set.rs:313-326`); a command needing both waits on
     `story:when-subject-beside-a-row-set-validates-under-the-g2-format` (active). It composes with
     `moves:` and `wrong_state:` today (`filtered-related-reads.md:82-92`).
   - Targets: interpreter and synthesis handle it (`related-rows-quantifier.md:373-390`); generated
     Rust and Go keep a row-set command an owed obligation, named
     (`crates/generate/ess-synth/src/determined.rs:210-217`); Entity Runtime refuses a row set by
     name (`RowSetUnsupported`).
5. **Second adopter.** A release is published only when every required check of its pipeline has a
   green run on this commit.
6. **Cost.** No format, key, diagnostic or IR change. One documentation section and two test cases.
7. **Considered.** (a) Change nothing: rejected, the report shows an author concluded the claim is
   inexpressible. (b) A `forall` over `{entity: E}` in the predicate grammar: rejected by the
   quantifier's design, and the row-set `forall:` already says it. (c) Chosen: document and pin the
   idiom.

## Decisions

**Decline, with the idiom.** The requester's need is met by `ess/27`. Report item 9 covered.
Open questions: `P2A-D1` (adopt the idiom, add a construct, or decline; default A, the idiom) and
`P2A-D2` (empty-set reading; default A, vacuously true, the language's meaning today). The
complement is stated in the Acceptance's documentation bullet.

## Scope

- cited: `website/docs/reference/predicates.md` (1860-1912)
- cited: `crates/specify/ess-domain/tests/related_rows_quantifier.rs` (existing binary, one case)
- cited: `crates/verify/ess-conformance/tests/related_rows_quantifier.rs` (existing binary, one case)
- Size: S (about 150 changed lines).

## Depends on

Nothing. Touches no file of the wave of 2026-11-24 and may land beside it.

## Source

A downstream application, 2026-11-24, report item 9 (its G4).
