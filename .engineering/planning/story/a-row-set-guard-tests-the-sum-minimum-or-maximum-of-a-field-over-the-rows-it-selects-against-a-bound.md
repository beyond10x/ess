---
format: aep.planning-md/3
id: story:a-row-set-guard-tests-the-sum-minimum-or-maximum-of-a-field-over-the-rows-it-selects-against-a-bound
kind: story
status: draft
title: A row-set guard tests the sum, minimum or maximum of a field over the rows it selects against a bound read from the request
summary: A guard cannot total related rows, so a parent's limit on its children's total needs a hand-kept counter (downstream report 2026-11-26, gap 1; RA-D1 to RA-D4)
tags:
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:related-rows-quantifier-in-a-selector
- informed_by: story:aggregate-views
revision: 3
---
## Need

A command refuses when the rows related to a parent would together pass a limit: a new child whose
length would take the sum of the lengths of its parent's children past the parent's budget. Reported
2026-11-26 by a downstream application on 0.99.2 (gap 1 of a report of ten); the application keeps a
counter on the parent by hand.

A row-set guard (`when_related: {entity, where, …}`, `ess/22`) tests the rows its selector selects by
exactly one of `exists: true|false`, `count: {<op>: <n>}` with a literal whole number, or
`forall: <predicate>` (`crates/specify/ess-domain/src/command/row_set.rs:21-23`;
`website/docs/reference/predicates.md` § "A guard over the rows a selector selects"). No test reads a
figure computed over the selected rows. A view can sum them (`sum`, `avg`, `min`, `max` measures,
`docs/design/language-grids.md` Grid O); a guard cannot.

Minimal shape, written for this story and not run (the spelling of the test is a sketch for RA-D1,
not a decided form):

```yaml
# demo.plan: Plan {plan_id, budget: Duration}; Slot {slot_id, plan_id, length: Duration}
- name: over-budget
  when_related:
    entity: demo.plan.Slot
    where: plan_id == input.plan_id
    sum: {of: length, gt: "input.budget_left"}
  error: demo.plan.OverBudget
- name: added
  creates: demo.plan.Slot
  instance: slot_id
  sets: {plan_id: input.plan_id, length: input.length}
```

## Fit review

1. **Need.** A command decides on a total of a numeric field over the rows a selector selects. The
   reporter proposed no syntax; the report states the gap and the hand-kept counter it uses.
2. **Class: gap.** Grid R, row "row-set `where:` / `forall:`", reads the selection itself; Grid O
   admits `sum`/`avg` as view measures only (`docs/design/language-grids.md`). No page promises it,
   so it is not a defect.
3. **Already expressible? No.** `count:` takes a literal (`row_set.rs:21-22`). The counter idiom
   moves the checked figure out of the specification into a second statement of a fact the rows
   hold, with nothing checking that the two agree; `docs/design/related-rows-quantifier.md`
   alternative (e) rejects it for that reason, and `generic-language-features.md` § Policy, policy 1,
   makes such an idiom no answer. A creating command cannot even keep the counter today:
   `affects:` beside `creates:` is refused (`ESS-COMMAND-009`, tracked by
   `story:a-branch-that-creates-its-subject-declares-affects-over-rows-that-existed-before-the-outcome-validated-and`).
4. **Fit.** Cells: Grid R row-set row × Grid O measure rows. Vocabulary: the measure names of
   aggregate views (`sum`, `min`, `max`) and the comparison keys of `count:` (`eq`, `ne`, `lt`, `lte`,
   `gt`, `gte`). Composition: the rows are the store just before the branch (predicates.md, same
   section), so the new row is not among them and the bound says so; a row-set refusal answers at
   step 4 of the one precedence rule; `ess/32` already pairs a row set with rows the input names, so
   the bound can read the parent named by the input. Siblings: `count:` gains the same bound (RA-D4);
   set-effect filters and invariants stay refused, invariants by the grid's own rule. Targets: the
   interpreter and synthesis decide it; generated Rust and Go keep the command an obligation named by
   the row set, as for every row-set guard; Entity Runtime refuses it as `RowSetUnsupported`;
   `ess verify diff` and generated docs print the test.
5. **Second adopter.** A warehouse refuses a reservation when the quantities of a bin's open
   reservations plus the new one pass the bin's capacity; an expense desk refuses a claim when one
   person's claims of a month pass a limit.
6. **Cost.** One test key on the row-set guard at the next open source format, refused below it
   naming that format. No new keyword beyond the measure names; I don't know whether a new
   `ValidationCode` is needed (`type_mismatch` and `unsupported_construct` cover the refusals named
   below). IR: one new variant of the row-set test, old bytes unchanged. Suite: no new step
   expected; synthesis arranges rows through creating commands as for `count:`. `ess verify diff`:
   I don't know whether the existing guard-change classification covers a changed aggregate.
7. **Designs.** (a) Change nothing: the hand-kept counter, rejected above. (b) A maintained
   aggregate field on the parent: rejected in `related-rows-quantifier.md` alternative (d), a new
   kind of stored field every target keeps current. (c) An aggregate test on the row-set guard:
   chosen. (d) An aggregate operand in the expression language, `sum(rows …)` at every predicate
   site: larger, and the grid has no second position asking for it yet.

## Decisions

Accept, redesigned: the need is taken as a generic row-set test; the reporter proposed no syntax.
Each option list names its default, the recommended option A.

- **RA-D1, spelling.** A: a fourth test beside `exists`, `count` and `forall`,
  `sum: {of: <field or expression of the candidate>, <op>: <bound>}`, and `min`, `max` in the same
  shape (recommended: reuses the measure names and the `count:` keys). B: one pair,
  `aggregate: {sum: <expr>}` with `compare: {<op>: <bound>}`. C: an aggregate operand in the
  expression language (design d).
- **RA-D2, what the bound reads.** A: literals, `input.`, `subject.` on a command that addresses an
  existing subject, and the fields of a row an input names (the `ess/32` pairing), the roots a
  selector already reads (recommended). B: literals and `input.` only.
- **RA-D3, absent values.** A: summing an `Optional` field is refused unless the test declares
  `skip_absent: true`, as a view measure does (recommended). B: an absent value counts as zero.
- **RA-D4, the count bound.** A: `count:` takes the same bound in the same change (recommended:
  the sibling asks the same question). B: `count:` keeps its literal.

- The bound may be a field of a row the command addresses or reaches (A), decided by the operator session, 2026-11-26: the downstream case compares the children's sum with the parent's declared length.

## Acceptance

1. At the next open source format the shape above validates; at the format below it is refused as
   `unsupported_format_version` naming the format.
2. The interpreter takes `over-budget` exactly when the sum over the rows the selector selects,
   in the store before the branch, compares with the bound as declared; a row of another parent does
   not count.
3. Synthesis arranges selected rows and a decoy that refutes the selector so the sum lies one unit
   below, on and one unit over the bound; a target that sums every row, reads `gte` for `gt`, or
   counts the rows instead of summing them fails the suite.
4. `sum` over a type other than Integer, Decimal or Duration, and a bound of another type than the
   sum, are refused as `type_mismatch` at the test's path.
5. Generated Rust and Go keep the command an obligation named by the row set; Entity Runtime refuses
   it by name.
6. `docs/design/language-grids.md` (Grid R row-set row, Grid O measure rows),
   `website/docs/reference/predicates.md` and `website/docs/reference/spec-versions.md` say it.

Every new test runs under 5 s; none may pass the 60 s bound, and the test step stays within 120 s.

Added 2026-11-26 (a downstream case): the bound may be a field of a row the command addresses or reaches, per RA-D2 A. Over a parent that declares a length and children that each declare theirs, the guard of a command that adds one child compares the sum of the selected children's lengths with the parent's declared length as written: synthesis arranges children whose sum lies one unit below, on and one unit over that length, and a target that compares with a figure from the input, or with another parent's length, fails the suite.
