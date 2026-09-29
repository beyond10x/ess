---
title: Audit a suite with specification mutants
sidebar_position: 4
description: Measure whether a suite notices a changed specification.
---

# Audit a suite with specification mutants

## Audit the suite with specification mutants

A green run shows only that the suite asks for nothing the target cannot answer. `mutate` asks the
other question: does the suite notice a wrong specification?

```shell-session
$ ess verify conform mutate \
    --path examples/billing \
    --target billing \
    --report-out target/billing-mutation.json
```

It changes the **specification**, one edit per mutant, and runs each mutant's freshly synthesized
suite against the unchanged reference target. A mutant is *killed* when its suite fails there. A
*survivor* is a declared rule that no synthesized scenario pins down. The nine classes are
`from-drop`, `transition-to`, `guard-boundary`, `sets-retarget`, `guard-negate`,
`guard-connective`, `error-swap`, `emit-drop` and `order-flip`; `--class` selects some of them and
repeats. Every class *changes* the specification rather than weakening it: a mutant that only says
less could never be killed by a correct target.

A mutant the model refuses is *stillborn*, with the refusing check's own code. For example, a
transition sent to another state is stillborn wherever its old arrival state has no other way in
(`ESS-ENTITY-011`), and so is dropping the only event of an outcome that names no error
(`ESS-COMMAND-007`). A stillborn mutant says something about the operator, not about the suite, and
does not change the exit status.

A mutant can make one of its own outcomes unsatisfiable, so synthesis refuses that outcome's
scenario and the mutant's suite is the baseline's minus it. Such a mutant is *unwitnessed*
(`ESS-MUTATE-004`) unless a scenario still kills it: what it changed has no scenario, so the rest
passing is not a survivor. Its entry lists each refusal it added, by code and scenario, and so does
the entry of a killed mutant that added one.

Where a guard mutant leaves its outcome's guard satisfied by no input, the mutant is *equivalent*
(`ESS-MUTATE-005`) unless a scenario still kills it: turning `any: [status == Paid, status ==
Shipped]` into `all:` writes a rule that can never be taken, so there is nothing for a scenario to
catch. Its entry names the guard as `unsatisfiable_guard`. The guard is decided only for
equality, membership and truth tests of input fields against literals, by trying every
combination of each field's values that could matter: a boolean's two, an enum's variants, and for
any other field its literals and one value none of them equals, up to 64 combinations. An
ordering, a text match, a quantifier, or a guard with more combinations is not decided, and such a
mutant is scored as before. A mutant with a changed scenario that was not scored stays
*inconclusive*: that scenario might have killed it.

A mutant on an outcome whose scenario the baseline suite already refuses, such as a branch no
arrangement reaches (`ESS-SYNTH-003`), cannot be killed by either suite. It is *unwitnessed* as
well, and its entry names the baseline refusal as `baseline_refusals`. So is a `from-drop` or
`transition-to` mutant on a transition that only such outcomes perform.

The baseline is red only when a scenario failed or ended `error`. A scenario the target reports
`unsupported`, or the runner `skipped`, is listed as not scored, and every mutant is scored on the
scenarios the baseline executed. The scenarios the baseline did not execute are listed, not scored:
a mutant scenario the baseline did not execute is excluded and listed on the mutant. A mutant that
nothing killed is *inconclusive*, not a survivor, when an excluded scenario is one it changed: that
scenario might have killed it. An excluded scenario the mutant holds exactly as the baseline does
asks the target what the baseline asked, so it cannot, and the mutant can still survive. A
scenario new to a mutant's suite is scored. A baseline that executed nothing scores nothing.

**A survivor is not answered by authoring a scenario.** An authored scenario's expectations are its
author's, not the model's, so it runs identically in every mutant's suite and can never kill one;
`mutate` runs none. Answer a survivor by declaring what makes the rule observable, such as a view
projecting the field a `sets` entry writes, or by filing a synthesis gap.

| Exit | When |
|---|---|
| 0 | No baseline scenario failed or ended `error`, at least one mutant that is not equivalent ran, every scored mutant was killed or equivalent, and none is inconclusive or unwitnessed. Baseline scenarios that were not executed are listed, not scored. |
| 1 | The specification did not load, or at least one mutant survived. |
| 3 | `ESS-MUTATE-001` (a baseline scenario failed or ended `error`), nothing scored (the baseline executed no scenario), `ESS-MUTATE-003` (no site), or no survivor and at least one mutant unwitnessed or inconclusive, or every mutant stillborn or equivalent. |

## Audit your own implementation

`--target` runs only the targets built into `ess`, and each of those passes only its own example
specification. For your implementation, split the audit in two and run the suites yourself:

```shell-session
$ ess verify conform mutate --path examples/billing --emit target/mutants \
    --class guard-boundary --class guard-negate
emitted 4 mutant(s) of billing v3 (0 stillborn, no suite; 2 with synthesis refusals the baseline does not have; 0 with a guard no input satisfies) and the baseline to target/mutants
run each <dir>/suite.json and write its conformance report to <dir>/report.json, then `ess verify conform mutate --collect target/mutants`
```

`--emit` runs nothing. It needs a new or empty directory and writes:

- `baseline/suite.json`, the unmutated suite;
- one directory per mutant, named by its id (`guard-negate/billing.invoice.PayInvoice/settled/`),
  holding its `suite.json`, the compact model `ir.json` a generated Go or TypeScript package
  embeds beside the suite, and `mutant.json` describing the change;
- `manifest.json`, an `ess-mutation-manifest/3` listing all of them. A stillborn mutant has an
  entry and no suite.

Run your runner over every `suite.json` and write its conformance report to `report.json` in the
same directory. The generated Go and TypeScript packages write the report named by
`ESS_REPORT_OUT` (`ESS_REPORT_FORMAT=2` for report/2); either report version is accepted. Then
score the reports:

```shell-session
$ ess verify conform mutate --collect target/mutants
mutation audit of billing v3 against billing-reference 0.43.0: 4 mutant(s), 3 killed, 0 survived, 1 inconclusive, 0 stillborn, 0 unwitnessed, 0 equivalent (baseline: 32 scenario(s), 0 refusal(s))
inconclusive guard-negate/billing.invoice.CreateInvoice/accepted: `when: amount.amount > 0` becomes `when: not (amount.amount > 0)`
killed guard-boundary/billing.invoice.CreateInvoice/accepted/0: `amount.amount > 0` becomes `amount.amount >= 0` — by billing.invoice.CreateInvoice/outcome/accepted (1 in total); …
…
```

That run used the built-in runner (`ess verify conform run --suite … --target billing
--report-out …`) as "your runner" and deleted one mutant's report on purpose. A mutant whose
report is missing, unreadable, written for another suite, or answered by another implementation
than the baseline's is inconclusive, and the JSON report says why under `unscored`. Each report is
scored only against the suite beside it. The baseline report must have passed, as with `--target`,
and the exit statuses are the ones in the table above: this run exits 3, because nothing survived
and one mutant is inconclusive.

## Read the output

The text output prints one summary line, then the baseline scenarios not scored, then survivors,
unwitnessed, inconclusive, equivalent, stillborn and killed mutants, one line each. `--report-out` writes an
[`ess-mutation-report/3`](../../reference/formats.md#change-and-conformance-records) document, and
`--format json` prints the same bytes.
