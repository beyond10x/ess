---
format: aep.planning-md/3
id: story:a-list-parameter-selects-rows-outside-a-group-key
kind: story
status: draft
title: A list parameter selects rows outside a group key, with absent meaning unfiltered
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#492
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/aggregate_group_selection.rs
- confidence: cited
  path: docs/design/aggregate-group-selection.md
revision: 6
---
## Outcome

Gap 3 of https://github.com/beyond10x/ess/issues/492: the list selector of #438 is arranged over a
row filter of an ungrouped or grouped view, and `not defined(param.<p>)` is accepted as the absent
rule beside `param.<p>.count == 0`. Today `ess verify conform synthesize` refuses the view with
`ESS-SYNTH-017`.

Spec first; fit review owed.

## Fit review

Reproductions: `target/wave-scratch/fit/492/g3/` (ess 0.56.0, `format: ess/23`).

1. **Need.** A read takes a list of key values. If the list is absent, every row counts. If it is
   present and empty, no row counts. Otherwise the rows whose key is in the list count. The key is a
   row field, not a group key. Minimal reproduction (`g3/requested.yaml`):

   ```yaml
   params:
     - {name: queue_ids, type: Optional<List<Integer>>}
   filter:
     any:
       - not defined(param.queue_ids)
       - exists: {in: param.queue_ids, as: q, that: queue_id == q}
   fields:
     - {name: sessions, type: Integer, aggregate: {count_distinct: session_id, skip_absent: true}}
   ```

   `ess specify validate` reports `fit v1 — 1 file(s), valid`. `ess verify conform synthesize`
   reports `refusal[ESS-SYNTH-017]: view fit.segments.SessionsInQueues has no scenario
   ... the parameter `queue_ids` is read other than by one top-level `field == param.queue_ids`
   conjunct over a field that is not a group key`.
   Requester's syntax (theirs): the same filter over `queue_id`. Expected: "the list selector
   arranged over a row filter of an ungrouped or grouped view, and the `not defined(param.<p>)`
   absent rule accepted beside `count == 0`".

2. **Class: gap in what synthesis can check, not in what the language can say.** The language states
   the fact: it validates, and the interpreter evaluates it as the requester means (question 3). An
   authored scenario can check it, so this is not a gap for authoring. Synthesis cannot witness it,
   and ESS treats synthesized witnesses as its own obligation: the same request for a group key was
   accepted as a synthesis-only change (#438, `docs/design/aggregate-group-selection.md:42-50`), and
   the arranger refusal for a value filter was recorded as owed follow-up work
   (`docs/design/read-api-view-idioms.md:111-115`). The refusal is documented, so it is not a
   defect: "Every other read of a list parameter keeps the `ESS-SYNTH-017` refusal"
   (`aggregate-group-selection.md:49-50`).

3. **Already expressible?** Yes, as a specification. Checkable today only by hand.
   - Authored scenario `g3/scenarios/list-filter.yaml`: four segments across three queues, with reads
     `{}` → 3, `{queue_ids: []}` → 0, `[1]` → 2 and `[3, 2]` → 2. `ess verify conform run --target
     interpreted --report-format 2 --path requested.yaml --scenarios scenarios/list-filter.yaml`
     reports `2 scenarios: 2 passed`. The same scenario expecting 3 for `[]`
     (`g3/scenarios-wrong/`) reports `1 failed`, so the scenario discriminates.
   - What synthesis covers today (one `synthesize` run each):

     | shape | file | synthesis |
     |---|---|---|
     | list over a group key, `List`, `count == 0` disjunct (#438) | `a-group-count0.yaml` | 2 scenarios, 0 refusals; reads `[0]`, `[1]`, `[3]`, `[2]`, `[3,1,0]`, `[]` |
     | list over a group key, `Optional<List>`, `not defined` disjunct | `b-group-notdefined.yaml` | `ESS-SYNTH-017` |
     | list over a row field, `List`, `count == 0` | `c-ungrouped-count0.yaml` | `ESS-SYNTH-017` |
     | list over a row `String` field | `f-ungrouped-list-string.yaml` | `ESS-SYNTH-017` |
     | scalar `field == param` over a row `String` field | `e-ungrouped-scalar-string.yaml` | 2 scenarios, 0 refusals |
     | scalar `field == param` over a row `Integer` field | `d-ungrouped-scalar.yaml` | `ESS-SYNTH-017`: "not a `String` or `Uuid` field the creating command sets from its input" |
     | the same filter on a row view | `g-rowview-list.yaml` | no refusal; its one read sends no `queue_ids`, so the list is never exercised |

   - Code: the list selector requires a declared `List` and a group key
     (`crates/verify/ess-conformance/src/synthesize/aggregate.rs:1566-1568`). A non-group scalar
     needs `scopable` (`:1593-1604`).

4. **Fit.**
   - Vocabulary: no authored change. The filter already says what the requester means, in the
     predicate spelling (`exists`, `not defined`).
   - Composition: the range selector already arranges rows of a non-group field and refutes the
     outside rows by the bound alone (`aggregate-group-selection.md:52-66`). A row list selector is
     the same arrangement with membership instead of order.
   - Siblings:
     - the scalar equality over a non-group `Integer` (row d) is refused for the same reason and
       should move with it;
     - the `not defined` absent rule belongs on the group-key selector too (row b);
     - a row view (row g) silently leaves the list unexercised. It needs the same reads, or a
       recorded reason why not.
   - Targets: none change. Interpreter, Rust, Go and TypeScript runners evaluate the filter already.
     Only the synthesized suite gains scenarios.

5. **Second adopter.** A ticket read API takes `assignee_ids`: absent means everyone, `[]` means
   no one, and a list means those assignees. The same goes for an inventory API filtered by
   warehouse ids. A list-of-ids filter over a row field is the common read-API shape.

6. **Cost.**
   - Source, IR, suite steps, runners: no source format, IR, suite-step or runner change.
   - Suites: models whose view was refused gain an aggregate scenario. Views that synthesized before
     keep their bytes, as #438 did (`aggregate-group-selection.md:208-209`).
   - Diagnostics: no new codes. `ESS-SYNTH-017` keeps every other shape.
   - Synthesis work: arrangement under `Empty` authority for a walked non-group key, computing
     expectations through the existing `shows` seam (`aggregate-group-selection.md:106-111`).

7. **Considered.**
   - (a) Change nothing; adopters write an authored scenario per view (shown above): kept as the
     interim route. It is not the answer, because every adopter re-derives the decoys, reversed lists
     and nonmatching values that synthesis already owns for a group key, and a target matching only
     the first element passes a scenario that never sends two.
   - (b) A source key declaring "absent means unfiltered": refused. The filter already states it,
     and two spellings would disagree.
   - (c) The requester's design: taken, and widened to the scalar sibling (row d), the group-key
     absent rule (row b) and row views (row g).

## Decisions

**Accept, redesigned.**

Synthesis-only. There is no source, format, IR or suite-step change. In
`crates/verify/ess-conformance/src/synthesize/aggregate.rs`, under `Empty` authority:

- **Row list selector.** A parameter typed `List<T>` or `Optional<List<T>>`, read only by one
  top-level membership conjunct over a field the creating command sets from its input, either way
  round, alone or in a two-way `any:` with `param.p.count == 0` or, for an `Optional` parameter,
  `not defined(param.p)`.
- **Reads.**
  - each arranged value as a list of one;
  - every value in one list, in reverse order;
  - one list holding only a value no row has;
  - `[]` asserting the empty result: an ungrouped view gives one row of zero aggregates and a grouped
    view gives zero rows, unless a `count == 0` disjunct makes `[]` mean all rows;
  - the parameter left out, asserting all rows, where `not defined` is written.
- **Arrangement.** Outside rows hold every other conjunct, so membership alone refutes them, as the
  range selector does.
- **Group keys.** The same absent rule on the #438 selector.
- **Scalar sibling.** A scalar `field == param.p` over a walked non-group key moves with it.
- **Row views.** A row view with such a filter gets the same reads, or the story records why not.

Docs: `docs/design/aggregate-group-selection.md` (list selector paragraph).

Acceptance:

- `g3/requested.yaml` synthesizes an aggregate scenario with no refusal.
- Mutants that ignore the list, match only its first element, read `[]` as unfiltered, or read
  absent as empty fail.

Until it lands, the authored scenario above is the route.
