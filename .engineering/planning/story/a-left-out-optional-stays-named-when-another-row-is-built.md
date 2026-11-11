---
format: aep.planning-md/3
id: story:a-left-out-optional-stays-named-when-another-row-is-built
kind: story
status: draft
title: A left-out optional stays named when another row is built
summary: The remembered ESS-SYNTH-002 for a left-out Optional is dropped for ESS-SYNTH-003 when some candidate builds a row that does not serve (turn-2 adversary, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- informed_by: review-result:zi-t2-lane2-adv-1
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:an-absent-optional-under-false-and-unknown-is-arranged` made the arranging creator's
candidate loop (`crates/verify/ess-conformance/src/synthesize/subject_fact.rs`) pass over a
candidate whose guard is Unknown only because it left an `Optional` input out, remembering that
cause, and return it as ESS-SYNTH-002 where no candidate serves. The cause is returned only when no
candidate built any row (`out.len() == 1`, about line 5206). When some candidate builds a row that
does not serve the goal (every `kind: Accept` row, where the branch needs an `Erratum` row that
only a candidate leaving `check` out could create), the remembered cause is dropped and the
scenario is refused with ESS-SYNTH-003 ("no candidate of the 1 tried satisfies …"), which no longer
names the left-out input. Found by the turn-2 adversary (`review-result:zi-t2-lane2-adv-1`);
introduced by that story, beside its acceptance (which covers "every candidate Unknown").

## Acceptance

- `a_row_only_a_left_out_optional_could_create_refuses_naming_it` in
  `crates/verify/ess-conformance/tests/adversary_absent_optional_arrangement.rs` runs (no
  `#[ignore]`) and passes: the refusal is ESS-SYNTH-002 and names `check`.
- The other cases of that file and `arranged_row_beside_absent_optional_guard` still pass, every
  emitted scenario passing on the interpreted target.
- The synthesis byte pins are unchanged, or each moved row is named with this cause.

## Scope

- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (the arranging creator's candidate loop)
- `crates/verify/ess-conformance/tests/adversary_absent_optional_arrangement.rs`
