---
format: aep.planning-md/3
id: story:the-mutation-audit-scores-the-suite-the-gate-runs-with-its-authored-scenarios-and-its-known-failures
kind: story
status: draft
title: 'The mutation audit counts the suite the gate runs: authored scenarios and a known-failures declaration are accepted together, bound to one suite digest'
summary: mutate refuses --scenarios beside --known-failing, and a declaration written from the gate's suite is refused by mutate's own suite digest (0.99.9)
tags:
- gap
- gap-reports-2026-11-28
relations:
- serves: vision:O2
- informed_by: story:feature-request-294
- informed_by: story:mutate-covers-subject-related-guards-sets-and-authored
revision: 1
---
## Need

A downstream meta-model specification (ess 0.99.9; interpreted target; 173 mutants, 137 of 138
scored killed) holds one synthesized scenario that fails on the interpreter, a known defect of
ESS itself, and holds authored scenarios that kill mutants the synthesized ones cannot. Two limits
of `ess verify conform mutate` leave it with no audit that counts both:

1. **`--scenarios` is refused beside `--known-failing`; without the declaration the baseline
   refuses.** While one synthesized scenario is a known failure, no authored scenario can kill a
   mutant: `mutate --scenarios` refuses the audit `ESS-MUTATE-001` on the failing baseline, and
   `mutate --known-failing` is the only way past it, which clap refuses together with
   `--scenarios` (`crates/edge/ess-cli/src/main.rs:1067`, `conflicts_with_all`). The audit can have
   the declaration or the authored scenarios, not both.
2. **A declaration is bound to a suite digest, and the suite `mutate` synthesizes is not the
   suite the gate runs, so the declaration is rebound by hand.** The gate runs the specification
   with its authored scenarios; `mutate` synthesizes its own baseline. A declaration written from
   the gate's report is refused by `mutate`, naming both digests.

Neutral reproduction on 0.99.9, in
`.engineering/repro/the-mutation-audit-scores-the-suite-the-gate-runs-with-its-authored-scenarios-and-its-known-failures/`
(`README.md` has the commands): `system.yaml` is a model whose synthesized suite has 2 failing
scenarios on the interpreted target (the reduction of
`story:a-scenario-arranged-around-a-related-row-passes-the-commands-own-subject-refusals-and-sends-only-identities-some-row-carries`),
`scenarios/` holds one authored scenario that passes, `known-failing.json` declares the 2 failures
bound to the synthesized suite's digest.

```text
$ ess verify conform mutate --path . --target interpreted
refusal[ESS-MUTATE-001]: the unmutated suite did not pass against `interpreted` ... 2 scenario(s) failed or ended `error`
$ ess verify conform mutate --path . --target interpreted --scenarios scenarios --known-failing known-failing.json
error: the argument '--scenarios <SCENARIOS>' cannot be used with '--known-failing <KNOWN_FAILING>'
$ ess verify conform mutate --path . --target interpreted --known-failing known-failing-from-the-gate.json
the known-failure declaration is refused: known-failures.identity: the declaration is bound to suite bytes `sha256:0572...`, and the suite is `sha256:5f58...`
```

The digests are those of `ess verify conform run --scenarios scenarios` (12 scenarios: the 11
synthesized and the authored one) and of the suite `mutate` synthesizes (11). With the declaration
for the synthesized suite alone, the audit runs and reports the 2 declared scenarios as known
failures; three mutants are `inconclusive` because their only witnesses are the declared scenarios
(`no witness under the declaration`), which is the case an authored scenario would settle.

## Outcome

One audit counts the suite the gate runs: `mutate --scenarios` and `--known-failing` are accepted
together; the declaration binds to the digest of the baseline suite the audit runs, which equals
the digest of the suite `run --scenarios` runs over the same inputs, so one declaration serves
both commands. A mutant is killed by an authored scenario while a synthesized scenario is a known
failure.

## Acceptance

- On the reproduction, `mutate --target interpreted --scenarios scenarios --known-failing
  known-failing.json` does not exit 2, reports the 2 declared scenarios as known failures and the
  authored scenario as a scored witness.
- The suite digest `run --scenarios <dir>` writes in its report equals the baseline digest
  `mutate --scenarios <dir>` checks a declaration against, on the reproduction and on a model with
  a `--component`; a test compares the two over one fixture synthesized once.
- A declaration bound to another suite is still refused, naming both digests; a scenario that
  passes is still refused as stale.
- A mutant whose only synthesized witnesses are declared failures, and which an authored scenario
  separates, is `killed` under the declaration (today: `inconclusive`).
- `--emit` and `--collect` keep their bytes; the manifest version moves only where a declaration
  and authored scenarios are both named, and the story's pull-request line says which.

## Fit review

1. Need: an audit that counts the authored scenarios and excuses a declared known failure in the
   same run. Reproduction above. The requester's proposal is not recorded (a report of two
   limits).
2. Class: gap in the audit's tooling; no authored specification surface.
3. Already expressible: `--known-failing` alone (no authored witness), `--scenarios` alone (no
   known failure). Neither keeps the figure; a baseline with a failure the audit cannot name is
   refused by design (`story:feature-request-294`).
4. Fit: the shared scorer already admits only baseline-passing scenarios
   (`docs/design/mutation-scope-and-known-failures.md`); an authored scenario is one more
   scenario of the baseline suite (`story:mutate-covers-subject-related-guards-sets-and-authored`).
   The conflict is in the command line, and the digest mismatch is in which suite `mutate` binds.
5. Second adopter: any repository whose gate holds authored scenarios and one known failure
   tracked in an issue, such as the order service in the fit review of `story:feature-request-294`.
6. Cost: a CLI constraint lifted, one digest rule in `mutate`, a test that two commands agree on a
   digest; no ESS source format change.
7. Considered: keep both refusals (today); lift the conflict and bind to the baseline suite
   `mutate` runs (A); a verb that rebinds a declaration to the current suite after checking that
   exactly the declared scenarios fail (B).

## Decisions

- MG1 (open): A, accept `--scenarios` beside `--known-failing`, with the declaration bound to the
  baseline suite the audit runs and that suite byte-equal to `run --scenarios`' over the same
  inputs; B, add a verb that rebinds a declaration to the current suite; C, keep the refusals.
  Recommended: A. B turns "a changed suite does not inherit its declaration" into an explicit act
  a script can repeat, which weakens the rule that exists so a new failure is not excused. A and C
  fail closed (a digest that differs refuses); B fails closed only if it refuses when a scenario
  outside the declaration fails.

## Scope

- inferred: crates/edge/ess-cli/src/main.rs
- inferred: crates/verify/ess-conformance/src/mutate.rs
- inferred: crates/verify/ess-conformance/tests/
- inferred: docs/design/mutation-scope-and-known-failures.md
