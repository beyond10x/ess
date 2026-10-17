---
format: aep.planning-md/3
id: story:a-mutant-whose-suite-equals-the-baseline-is-unwitnessed
kind: story
status: active
title: A mutant whose suite equals the baseline's is unwitnessed, not survived
tags:
- adopter-report
- defect
refs:
- provider: github
  reference: beyond10x/ess#586
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-16T10:20:21Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-16T10:20:21Z", actor: "human:timo", revision: 3}
---
## Outcome

`ess verify conform mutate` scores a mutant whose synthesized suite equals the baseline's (every
scenario and step identical, digests aside) as `unwitnessed`, naming the reason "the suite does not
change", instead of `survived`. A survivor then always means a suite that differs from the
baseline and still passes: a scenario could have killed it.

Findings covered: https://github.com/beyond10x/ess/issues/586, option 2 (a downstream audit on ess
0.60.0). The rule generalises the special cases already shipped one at a time:
https://github.com/beyond10x/ess/issues/218 (unsatisfiable guard),
https://github.com/beyond10x/ess/issues/203 (gained refusals), https://github.com/beyond10x/ess/issues/517
and https://github.com/beyond10x/ess/issues/539 (equivalent precedence swaps).

## Fit review

1. **Need.** A `guard-boundary` mutant on `expiry < now` produces a suite that differs from the
   baseline only in `contract_digest` and `spec_digest`, and is reported survived. Nothing the
   suite could run distinguishes it, so "survived" claims a missing scenario that synthesis could
   not write. Minimal reproduction: the model of `story:a-suite-fixes-the-targets-clock`.
2. **Class.** Defect in the report's meaning: `survived` is documented as a mutant a scenario could
   have killed and did not (`story:unkillable-mutants-are-not-scored-survived`, implemented).
3. **Already expressible?** No.
4. **Fit.** The existing verdicts already include `unwitnessed`
   (`crates/verify/ess-conformance/src/mutate.rs:2950-3010`, `Verdict`). A suite-equality check
   before classification covers every shape where the mutant changes nothing synthesis can witness,
   without one equivalence rule per mutant class; the class-specific rules
   (`unsatisfiable_guard`, `identical_answer`) stay, because they name a better reason
   (`equivalent`) where they apply. Precedence: `killed` and `equivalent` beat this; it beats
   `survived` and `inconclusive`.
5. **Second adopter.** Any `now` guard (today's off-boundary witnesses), any mutant on a field no
   view observes.
6. **Cost.** A new reason in `ess-mutation-report` and the manifest (format consequence decided
   explicitly: a new reason value moves the report format if readers refuse unknown reasons); exit
   status follows the existing `unwitnessed` mapping.
7. **Alternatives.** (a) Change nothing. (b) Another class-specific rule for `now` boundaries: the
   next class needs one too. (c) Suite equality as a general rule (chosen).

## Decisions

Accept, redesigned from "classify the identical boundary mutant" to "any mutant whose suite equals
the baseline's is unwitnessed".

## Acceptance

- `identical_suite_mutant_is_unwitnessed`: on the clocked reproduction, the `guard-boundary` mutant
  is `unwitnessed` with the reason naming suite equality, in `--target`, `--emit` and `--collect`.
- `differing_suite_mutant_still_survives`: a mutant whose suite differs and passes stays `survived`.
- `class_specific_equivalence_wins`: the #517 and #539 fixtures keep `equivalent`.

## Scope (inferred)

`crates/verify/ess-conformance/src/mutate.rs` (`evaluate`, `collect`, `Verdict`), the report schema,
`website/docs/reference/formats.md`, CHANGELOG.
