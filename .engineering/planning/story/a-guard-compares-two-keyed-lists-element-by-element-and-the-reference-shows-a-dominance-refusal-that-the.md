---
format: aep.planning-md/3
id: story:a-guard-compares-two-keyed-lists-element-by-element-and-the-reference-shows-a-dominance-refusal-that-the
kind: story
status: draft
title: A guard compares two keyed lists element by element, and the reference shows a dominance refusal that the interpreter decides
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A rule over two lists of keyed values, one on a stored row and one in the input (or on the
addressed subject), compared element by element with the elements matched by key:

- Pareto dominance: an offer already standing in a round beats a new offer when it is at least as
  good on every criterion and better on one, each criterion with its own better side;
- a price list may replace another only when every product's new price is at or below its old
  price;
- a rating may be recorded only when no criterion falls below the floor the same criterion has on
  the template row.

A downstream application reported that ESS has no comparison across the list elements of two rows,
and it keeps dominance as an `external:` branch. Read in the source, the grammar already says it:

- a predicate quantifier over a collection, `forall:` / `exists: {in, as, that}`, nests, and an
  inner body reads the outer binder (`crates/specify/ess-primitives/src/predicate.rs:59-84`,
  `:1876`, `:2630`);
- a binder on the right of a comparison is a fact from `ess/22`, so `theirs.value >= mine.value`
  compares two elements (`docs/design/expression-family-source22.md:38`; binder against binder,
  beyond10x/ess#289, `crates/verify/ess-conformance/tests/binder_operand.rs`);
- a row-set selector reads every declared field of its candidate, `List` fields included
  (`crates/specify/ess-domain/src/command/row_set.rs:247-256`,
  `crates/specify/ess-domain/src/command/subject_fact.rs:429-443`), and `input.` and `subject.`
  (`row_set.rs:12-16`);
- several row-set guards over different entities may stand in one command: only a subject guard,
  a held-state guard, a state-change guard and `input_absent` are refused beside a row set
  (`row_set.rs:314-325`).

What is missing is any statement that this works: no reference page, test or example compares
two keyed lists, and nothing says how a per-key better side is read.

Minimal reproduction, written fresh and not run (the assessment was read-only, with no build):

```yaml
format: ess/37
system: demo
version: v1
domain: demo.offers
types:
  - {name: demo.offers.RoundId, kind: newtype, of: Uuid}
  - {name: demo.offers.OfferId, kind: newtype, of: Uuid}
  - {name: demo.offers.CriterionId, kind: newtype, of: Uuid}
  - {name: demo.offers.Sense, kind: enum, variants: [higher_better, lower_better]}
  - name: demo.offers.Score
    kind: struct
    fields:
      - {name: criterion_id, type: demo.offers.CriterionId}
      - {name: value, type: Decimal}
      - {name: sense, type: demo.offers.Sense}
entities:
  - name: demo.offers.Criterion
    identity: {name: criterion_id, type: demo.offers.CriterionId}
    fields: [{name: sense, type: demo.offers.Sense}]
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.offers.Offer
    identity: {name: offer_id, type: demo.offers.OfferId}
    fields:
      - {name: round_id, type: demo.offers.RoundId}
      - {name: scores, type: 'List<demo.offers.Score>'}
    lifecycle: {initial: Standing, states: [Standing], terminal: [Standing]}
commands:
  - name: demo.offers.Submit
    input:
      - {name: offer_id, type: demo.offers.OfferId}
      - {name: round_id, type: demo.offers.RoundId}
      - {name: scores, type: 'List<demo.offers.Score>'}
    outcomes:
      - name: wrong-sense            # a score's copied sense is its criterion's
        when_related:
          entity: demo.offers.Criterion
          where:
            exists:
              in: input.scores
              as: s
              that: {all: [s.criterion_id == criterion_id, s.sense != sense]}
          exists: true
        error: demo.offers.WrongSense
      - name: beaten                 # a standing offer is as good on every criterion, better on one
        when_related:
          entity: demo.offers.Offer
          where:
            all:
              - round_id == input.round_id
              - state == Standing
              - forall:
                  in: scores
                  as: theirs
                  that:
                    exists:
                      in: input.scores
                      as: mine
                      that:
                        all:
                          - mine.criterion_id == theirs.criterion_id
                          - any:
                              - {all: [theirs.sense == higher_better, theirs.value >= mine.value]}
                              - {all: [theirs.sense == lower_better, theirs.value <= mine.value]}
              - exists:
                  in: scores
                  as: theirs
                  that:
                    exists:
                      in: input.scores
                      as: mine
                      that:
                        all:
                          - mine.criterion_id == theirs.criterion_id
                          - any:
                              - {all: [theirs.sense == higher_better, theirs.value > mine.value]}
                              - {all: [theirs.sense == lower_better, theirs.value < mine.value]}
          exists: true
        error: demo.offers.Beaten
      - name: submitted
        creates: demo.offers.Offer
        instance: offer_id
        sets: {round_id: input.round_id, scores: input.scores}
```

(Errors, the creating event and the actor are left out here; the fixture declares them.)

The requester's words: "ESS has no comparison across list elements of two rows"; they proposed no
syntax.

## Outcome

The keyed-list comparison is a documented, tested idiom: a nested quantifier in a row-set
selector, the elements matched by an equality on their key, with the per-key better side carried
in each element and checked against the row that owns it. The interpreted target decides it
correctly, every generated target either evaluates it or names it, and the reference page shows
the dominance example. No key, keyword or format is added.

## Acceptance

- Red first: `crates/verify/ess-conformance/tests/keyed_list_compare.rs` (new binary, or a module
  of the crate's existing integration binary where one exists) loads the fixture
  `crates/verify/ess-conformance/tests/fixtures/keyed-list-compare.yaml` (the reproduction, with
  three criteria) and asserts that `ess specify validate` exits 0 and that the interpreted target
  answers four arranged stores: a standing offer better on one criterion and equal on the others
  gives `beaten`; one equal on every criterion gives `submitted` (the strict half); one better on
  two criteria and worse on the middle one gives `submitted` (the universal half, the deciding
  element neither first nor last); one whose list holds a criterion the input lacks gives
  `submitted` (the key tie). A refusal or a wrong answer found here is fixed in this story, and
  the story's report names it and its cause. If all four already pass, the report says so.
- The same file holds the subject form: a command addressing an existing offer compares
  `subject.scores` with a candidate's `scores`, and the interpreted target answers it the same way.
- With a score whose copied `sense` differs from its criterion's, the interpreted target answers
  `wrong-sense`.
- Each generated target's answer for the fixture is pinned by a test: generated Rust and Go either
  evaluate the guard or keep the command an obligation that names it; Entity Runtime lowers it or
  refuses it by name. No target ignores it silently.
- `website/docs/reference/predicates.md` gains "Comparing two keyed lists": the dominance example,
  why the key equality stands inside the inner quantifier, why the sense is copied into each
  element and how a row-set guard checks the copy, and the subject form.
- `docs/design/language-grids.md` is unchanged: no row or column of either grid opens. The story's
  report says so.
- A test asserts that the fixture is `ess/37` and validates: no format version moves, and no
  committed example or suite changes its bytes.

## Fit review

Tree `bcfc51555` (branch `integration/wave-2026-11-24`), read-only, nothing built or run. Received
2026-11-24 from a downstream application, report item 14.

1. **Need.** Compare two keyed lists element by element, with the elements matched by key, under a
   universal and an existential quantifier, a comparison sense per key, and the two lists on two
   rows or on a row and the input. Reproduction above. The requester proposed no syntax.
2. **Class.** By reading, *convenience*: every part is in the grammar (Need, the four citations).
   It is also part *gap*: nothing documents or tests it, and the witness is open (see
   `g14-keyed-list-witness`). This class rests on reading only: the first acceptance runs it.
3. **Already expressible?** Yes by reading: a row-set guard, where an outer `forall:` over the
   candidate's list holds an inner `exists:` over the input's list. Policy 1
   (`docs/design/generic-language-features.md:590-595`) holds: the figure stays in the
   specification, and the suite checks it once `g14-keyed-list-witness` lands.
4. **Fit.** Grid O row `forall exists distinct` over List / Map is "yes"
   (`docs/design/language-grids.md:79`), and grid R row-set `where:` reads input, subject and
   candidate (`language-grids.md:26`). No cell opens.
   - Vocabulary: the same `{in, as, that}` quantifier the related-rows design reused
     (`docs/design/related-rows-quantifier.md:104-112`); no `candidate.` root, no second spelling.
   - A per-key sense read from a third row (the criterion) would need a rows quantifier inside a
     list quantifier, refused today ("one level", the tie to a candidate field,
     `crates/specify/ess-domain/src/command/related_rows.rs:10-17`; synthesis takes a rows
     quantifier only as a top-level conjunct,
     `crates/verify/ess-conformance/src/synthesize/row_set/related_rows.rs:152`). The idiom copies
     the sense into the element and checks the copy with a row-set guard over the criterion rows.
   - Siblings: P2A's report item 9 (`g09-every-row-related`) states "every row of a selection has
     a passing related row" with the same quantifier and `in: {entity}`. One quantifier design,
     over rows there and over list fields here; neither designs a second one.
   - Targets: I don't know whether generated Rust and Go evaluate a list quantifier inside a
     row-set `where:`. The acceptance pins each target's answer.
5. **Second adopter.** A price list that may replace another only when no product's price rises
   (`forall` new line, `exists` old line of the same product, `new.price <= old.price`).
6. **Cost.** No key, keyword, diagnostic, format or IR change. One reference section, one fixture,
   one test file. Size S.
7. **Considered.** (a) Change nothing: the adopter keeps `external:` and the figure stays out of the
   suite. Rejected under policy 1. (b) A named construct, `dominates: {left, right, key, value,
   sense}`: a one-to-one keyword for one rule, a second spelling of a nested quantifier, and every
   target and synthesis would need it. (c) A derived relation that ESS computes: derived fields
   were retired on 2026-09-15 (`story:a-field-may-be-derived-rather-than-stored`, archived). (d)
   Chosen: the idiom, documented and pinned, with its witness in `g14-keyed-list-witness`.

## Decisions

**Accept, redesigned:** no construct; the idiom plus its witness. Operator decision `P2B-D1`.

- **D1. How is the comparison stated?** (a) The nested-quantifier idiom (this story and
  `g14-keyed-list-witness`). (b) A named construct. (c) A relation ESS computes. (d) Decline.
  **Recommended: (a).**
- **D2. Where does a per-key better side come from?** (a) It is copied into each element, and a
  row-set guard over the owning rows refuses a wrong copy. (b) Open a rows quantifier inside a
  list quantifier. That is a design of its own, and it would be shared with report item 9's
  quantifier. (c) The writer stores every value normalised to "higher is better". **Recommended:
  (a).** It adds nothing to the grammar. (c) hides the sense from the reader.

## Scope

- inferred: `crates/verify/ess-conformance/tests/keyed_list_compare.rs` (new)
- inferred: `crates/verify/ess-conformance/tests/fixtures/keyed-list-compare.yaml` (new)
- cited: `website/docs/reference/predicates.md` (new section beside the quantifier sections)
- cited, only if the first acceptance finds a refusal: `crates/specify/ess-domain/src/command/row_set.rs`,
  `crates/specify/ess-domain/src/expression.rs` (quantifier typing, `quantified` at `:2993`)
- inferred: one pin per generated target in `crates/generate/ess-synth/tests/` and
  `crates/generate/ess-entity-runtime/tests/`
- Size: S (about 250 changed lines, more if the first acceptance finds a refusal).

## Depends on

None. `g14-keyed-list-witness` depends on this story.

## Source

A downstream application, 2026-11-24, report item 14.
