---
format: aep.planning-md/3
id: story:a-mutant-whose-suite-equals-the-baseline-is-unwitnessed
kind: story
status: implemented
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
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-16T10:20:21Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-16T10:20:21Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-02T20:40:24Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
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

## A survivor the baseline suite kills (downstream report, 2026-11-27)

## A survivor the baseline suite kills (downstream report, 2026-11-27)

A downstream hardening pass ran `ess verify conform mutate --target interpreted` on the part of its specification that
validates alone (22 commands, 95 outcomes, 5 entities) and reported 203 mutants: 109 killed, 4 survived, 25 stillborn,
58 unwitnessed, 7 equivalent. It then ran the baseline suite against each survivor's interpreter and found all four
killed. This section extends the rule above ("a survivor always means a suite that differs from the baseline and still
passes: a scenario could have killed it"): a differing suite is not the only suite there is.

### Need

On 0.99.2, and on `main` at 0.99.8 (`CHANGELOG.md` 0.99.3 to 0.99.8 and `[Unreleased]` name no change to how `mutate`
scores a mutant), `mutate` scores a mutant by the suite synthesized for that mutant's model, run against the unchanged
target. The mutant's synthesis chooses its inputs by the mutant's guards, so it can choose inputs at which the mutant
and the specification agree, and the baseline's scenario that tells them apart is no longer in the suite it runs. The
four survivors were of two shapes:

- A `sets-drop`: the baseline scales a stored value from 3 to 1 and expects 1; the mutant's suite creates the row with
  1 and scales it to 1. Three baseline scenarios fail on the mutant.
- Three `guard-boundary` mutants over a time guard (`<= now` to `< now`, `>` to `>=`): the mutant's suite drops the
  scenario that pins the clock exactly at the boundary, and the baseline's scenario fails on the mutant.

The shapes and counts are the report's, measured on 0.99.2; the minimal model of the acceptance below is the first step
of the story, red before any change.

The audit reports a missing scenario ("a scenario could have killed it and did not"), and the scenario exists: it is in
the baseline suite. The downstream checked each survivor by hand and has no way to make the audit do it. The verdict
the audit prints is a false positive of the audit, not a gap in the suite.

### Fit review

1. **Need.** An audit whose `survived` verdict means no scenario of the suite kills the mutant, so a survivor is a
   specification or suite gap to act on. The consumer is "a downstream control plane".
2. **Class.** Defect in the meaning of `survived`: the audit tests a mutant against the one suite synthesis wrote for it,
   not against the suite the implementation is held to.
3. **Already expressible?** `--emit` and `--collect` run each suite elsewhere, and a hand run of the baseline suite
   against a copy of the model with the one edit does it, with the suite's `provenance.spec_digest` set to the mutant's,
   which `run` otherwise refuses. A hand edit of a digest is not a verb a gate can call.
4. **Fit.** Reuses the report's `killed` verdict and the interpreted target, which can run any model. An external
   `--target` runs a real implementation that cannot be mutated by editing the model, so it is outside this section.
5. **Second adopter.** Any audit with a `guard-boundary` mutant over a clock or a `sets-drop` on an updated field.
6. **Cost.** One run of the baseline suite per mutant that survived, no synthesis, so it does not add to the cost of a
   mutant (`story:a-mutation-audit-of-a-larger-model-ends-in-bounded-time-and-shows-its-progress-while-it-runs`, about
   20 s on that model). The report and the manifest carry a new reason or field for it, which is a format question
   (D2).
7. **Alternatives.** (a) Change nothing: each survivor is checked by hand. (b) Run the baseline suite against each
   survivor (chosen, D1). (c) Run it against every mutant: it would also settle `unwitnessed` mutants, and costs a
   baseline run for each.
8. **Generic, not per finding.** One extra check covers the `sets-drop` and the boundary survivors, and every class
   whose own suite drops the scenario that told it apart; it is not a rule per class.

### Decisions

- **D1. When is the baseline suite run against a mutant?** (a) For every mutant the mutant's own suite leaves
  `survived`. (b) For every mutant that is not stillborn. (c) Never; document that a survivor is checked by hand.
  **Recommended: (a).** It only changes the verdict that is wrong, costs a suite run for 4 of 203 mutants in the
  report, and leaves `killed`, `unwitnessed` and `equivalent` as they are; (b) also reclassifies `unwitnessed` mutants
  and can be a follow-up.
- **D2. What does the report say?** (a) A survivor the baseline suite kills becomes `killed`, with its reason naming the
  baseline scenario ids that failed. (b) A new verdict, so a reader can tell the two kinds of kill apart, and the
  exit status maps it like `killed`. **Recommended: (a),** with the scenario ids in the existing killed-by field; a new
  verdict is a larger format change for a distinction a reader can already read from the ids. Either way the format
  consequence is decided explicitly (a new reason or field moves `ess-mutation-report/<n>` and the manifest if a reader
  refuses an unknown value).
- **D3. How does `--emit` and `--collect` carry it?** The manifest lists each mutant's model and suite so a collector
  can score offline. Recommended: the manifest also names, for a survivor candidate, the baseline suite and the
  mutant model, so `--collect` runs no synthesis; the shape of that entry is decided in the story, with the old manifest
  still readable.

### Acceptance

- On a model with one command that updates a stored integer and a view that reads it, `mutate --class sets-drop
  --target interpreted` reports the mutant whose own suite leaves it surviving as `killed`, naming the failing
  baseline scenario ids, and exits as `killed` does.
- On a model with a `now` guard, the `guard-boundary` mutant whose own suite drops the boundary scenario is `killed` by
  the baseline's boundary scenario.
- A mutant that neither suite kills stays `survived`; `unwitnessed`, `equivalent` and `stillborn` verdicts do not move.
- `--emit` then `--collect` give the same verdicts as the direct run.
- The report's text and the guide page `website/docs/guides/verify/mutation-audit.md` say which suites a verdict is
  over.
- No test over 60 s.
