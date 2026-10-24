---
format: aep.planning-md/3
id: story:related-rows-keep-distinct-names-and-named-refusals
kind: story
status: draft
title: Related rows keep distinct names, existence refusals name their guard, and a references-only relation is arranged
summary: 'Follow-up to the U6 related-guard stories: a suite-level no-double-capture check after per-row distinctions, the existence-outcome refusal test, and TagMachine on a references-only relation (0.70.0).'
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Need

Left open by unit U6 of the 0.69.0 batch (tracker rows 13, 17, 18), 2026-10-23:

1. **A suite never captures one name twice.** `story:a-row-arranged-beside-is-created-once-under-its-name`
   fixed the row arranged beside a searched row, but four passing scenarios in
   `crates/verify/ess-conformance/tests/fixtures/arrangement-through-writers.yaml` still capture one
   name twice: three create `review-194041` again for each decoy, and `Seal/not-approved` does so
   because `successors` calls `stored::step` with `Distinction::PLAIN`. A suite-level check that
   withdraws such a scenario would withdraw all four, so nested related rows first need a
   distinction of their own per row.
2. **An existence outcome's refusal names the guard.** The third acceptance line of
   `story:an-existence-outcome-passes-a-related-list-membership-guard` (where no row can satisfy
   the membership guard, the refusal names that guard) has no test; no variant tried reached the
   refusal path.
3. **A references-only relation arranges `TagMachine`.** With a `references` relation instead of
   `owns`, `TagMachine`'s `elsewhere`, `not-alpha` and `tagged` branches are refused as
   ESS-SYNTH-003, on the 0.69.0 base too.

## Acceptance

- Nested related rows get a per-row distinction, and a suite check refuses any scenario that
  captures one name twice; the four scenarios above are synthesized with distinct names.
- A test reaches the existence-outcome refusal and asserts it names the guard, or the line is
  withdrawn from the earlier story with the reason.
- The references-only `TagMachine` shape gets a scenario for each of the three branches, or each
  is refused naming why no arrangement exists.

## Left open by the U6 adversary

The U6 adversary pass (2026-10-23) found two more lines; their tests were removed from the unit
because they fall outside its three stories:

4. **Nested-list membership.** A related-list membership guard over a list nested inside the related
   row is not arranged; the existence-outcome fix covers a top-level list member only.
5. **Unknown-site on an existing subject.** Where the subject row exists in the wrong state, the
   wrong-state refusal answers first and the unknown-site branch is never synthesized.

Class gaps the unit fixed only at the two sites the reports reached:

- Distinction blocks in `synthesize/related_guard.rs` (`BESIDE` 13, `BESIDE_DRIVEN` 17, `OVERLAP` 21)
  and `related_guard/stored.rs` (`STORED` = `OVERLAP + 5`) are held apart by a const assertion, but
  `COPIED` blocks (`subject_fact.rs`, from 13 + d) overlap `BESIDE` at d = 0, and the related blocks
  `1 + d + 8 * nth` have no upper bound.
- The other `with_row` call sites in `related_guard.rs` (lines 1274, 1325, 1734, 2615 at the merge)
  were not surveyed for the `ordered`/`order` split U6 introduced.

## Further acceptance

- A test arranges a membership guard over a nested list, or the refusal names the nested path.
- A test reaches the unknown-site branch on a subject that exists in a wrong state.
- One allocation hands out every distinction block, so two blocks cannot overlap by construction;
  a test asserts disjointness over the full range of `d` and `nth` the synthesizer can reach.
- Each `with_row` call site is either moved to `with_row_where` or the story records why its order
  is safe.
