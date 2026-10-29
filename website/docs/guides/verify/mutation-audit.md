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
*survivor* is a declared rule that no synthesized scenario pins down. The twelve classes are
`from-drop`, `transition-to`, `guard-boundary`, `sets-retarget`, `guard-negate`,
`guard-connective`, `error-swap`, `emit-drop`, `order-flip`, `sets-drop`, `precedence-swap` and
`emit-swap`;
`--class` selects some of them and repeats. Every class *changes* the specification rather than
weakening it: a mutant that only says less could never be killed by a correct target.

- `guard-boundary` moves a boundary three ways: it swaps the strictness of `<`, `<=`, `>` or `>=`;
  it moves the integer literal of a `>=` or `<=` one step outward (`amount >= 10` becomes
  `amount >= 9`), the direction the swap does not take; and it flips `==`↔`!=` on a comparison that
  is not the whole guard, which `guard-negate` already covers.
- `guard-boundary`, `guard-negate` and `guard-connective` edit the predicate of a `when_subject:`
  over the stored subject, and the predicate of a `when_related:` over the row an input identity
  names, as they edit a `when:`. Such a mutant's id names the key after the outcome:
  `guard-negate/<command>/<outcome>/when_subject`,
  `guard-boundary/<command>/<outcome>/when_related/0`. A `when:` mutant keeps the id it had. The
  `{field, equals}` form of `when_subject:` and a row set's `where`, `count` and `forall` are not
  edited this way; a row set's `count:` bound has its own `guard-boundary` mutant.
- `sets-drop` removes one `sets: field: input.x` entry from a branch that updates or moves an
  existing row. There the drop is not weaker: the field keeps what the row held instead of taking
  the input. Synthesis sends an input no `sets:` entry reads apart from what the row holds in the
  field of the same name and type, so a view reading the field tells the two apart. Where the
  dropped input feeds a field of another name that already holds the same value, the mutant can
  survive: that survivor is a weak witness, not a correct implementation. Every other `sets:`
  entry has no site, and the audit says why of each one: after its own output, on stderr and in
  every format, `--target` and `--emit` print one note per entry, such as
  `note: sets-drop/<command>/<outcome>/<field> has no mutant (creates_row): …`. The reasons are
  `creates_row` (a creating branch: a dropped write only leaves the value to the implementation, a
  weaker rule no correct target fails, and an outcome whose guard synthesis arranges through the
  field loses its scenario), `source_not_input` (a literal or any other source than `input.<f>`,
  which synthesis cannot send apart from what the row held), and `optional_field` (a row nothing
  wrote holds an `Optional` field absent, and no scenario asserts an absence). A model whose every
  write is on a creating branch therefore has no `sets-drop` mutant, and `--class sets-drop`
  refuses with `ESS-MUTATE-003` and then names each entry. The report and the manifest carry no
  such list, so their formats are unchanged.
- `precedence-swap` swaps two adjacent branches guarded by their input alone, both accepting or
  both refusing, so the second answers where both guards hold. Where no input satisfies both
  guards, the swap decides nothing and is *equivalent* (below). A branch the held state, a stored
  or related row, a provider or a replay also decides is left out.
- `emit-swap` replaces the only event of an outcome that names no error with another declared
  event: one with exactly the same fields (names, and types down to the named type, `Optional` and
  containers), published by every component that accepts the command, the first such in byte order
  of name that compiles in its place. The outcome's `payload:` entry is renamed and keeps its
  values. A site where no event qualifies, including every site of a model that declares no
  component, is *unavailable* (`no_compatible_event_alternative`): no mutant exists, so the event
  there is not audited by substitution, and the audit cannot succeed (below).

A mutant the model refuses is *stillborn*, with the refusing check's own code. For example, a
transition sent to another state is stillborn wherever its old arrival state has no other way in
(`ESS-ENTITY-011`), and so is dropping the only event of an outcome that names no error
(`ESS-COMMAND-007`); `emit-swap` audits those outcomes instead, or lists them as unavailable. A
stillborn mutant says something about the operator, not about the suite, and does not change the
exit status.

A mutant can make one of its own outcomes unsatisfiable, so synthesis refuses that outcome's
scenario and the mutant's suite is the baseline's minus it. Such a mutant is *unwitnessed*
(`ESS-MUTATE-004`) unless a scenario still kills it: what it changed has no scenario, so the rest
passing is not a survivor. Its entry lists each refusal it added, by code and scenario, and so does
the entry of a killed mutant that added one.

Where a guard mutant leaves its outcome's guard satisfied by no input, the mutant is *equivalent*
(`ESS-MUTATE-005`) unless a scenario still kills it: turning `any: [status == Paid, status ==
Shipped]` into `all:` writes a rule that can never be taken, so there is nothing for a scenario to
catch. Its entry names the guard as `unsatisfiable_guard`. The guard is decided only for
equality, membership and truth tests of input fields against literals, and `defined` tests of
input fields, by trying every combination of each field's values that could matter: a boolean's
two, an enum's variants the guard names and one it does not (any other variant answers alike),
and for any other field its literals and one value none of them equals,
plus absence for an `Optional` field, up to 64 combinations. A `precedence-swap` of two branches
no input selects together, such as `defined(x)` and `not defined(x)`, is equivalent on the same
rule. So is a guard mutant whose `when:` sits beside a `when_subject:` predicate that can hold
only by comparing a stored field with an input field the mutated `when:` leaves absent, as
`not defined(expected_version)` beside `version != input.expected_version` does: a comparison with
an absent input is unknown on every row, never true, and the entry names the `when:` and the
stored predicate together. A mutant of the `when_subject:` predicate itself is decided the same
way where a `when:` sits beside it; one with no `when:` beside it, and every mutant of a
`when_related:` predicate, is not decided and is scored as before. An
ordering, a text match, a quantifier, or a guard with more combinations is not decided, and such a
mutant is scored as before, except in a `precedence-swap`, where a part of the two guards that
decides is enough: two pinned values that contradict each other (`flag == false` and
`flag == true`) decide it whatever an ordering or a text match beside them says, an enum attribute
is read over the enum's variants, and an `any:` is decided where each of its alternatives is. An
`Optional` input that is absent and compared with a literal in both guards selects neither branch.
The entry names the part that decides. A swap where one guard holds while the other is unknown
keeps its scoring: read first, the unknown guard refuses the request the other would have
answered. A mutant with a changed scenario that was not scored stays
*inconclusive*: that scenario might have killed it.

A `precedence-swap` of two branches that answer alike is equivalent too, unless a scenario still
kills it: the same error, the same or no payload, and neither branch changing state, setting or
emitting anything, so either order gives the caller the same answer where both guards hold. Its
entry names that answer as `identical_answer`, apart from `unsatisfiable_guard`, so a reader
tells "these guards never overlap" from "these answers are identical". Two refusals that differ in
error or payload are scored as before, and so are two that a binding invoking the command answers
differently: a failure policy that drops one and escalates the other, or a bounded retry that
names only one of them `final`, since the binding then publishes or retries differently whichever
branch answers.

A `precedence-swap` whose overlap is already taken is equivalent as well, unless a scenario still
kills it: where every input both branches select also selects a branch answered before both, the
two never answer the same request. With `done-paused-default` (`pause == true` and `code` absent or
`""`) declared before `done-paused` (`pause == true`) and `done-default` (`code` absent or `""`),
swapping the last two decides nothing. Only branches selected by their input guard alone are
counted, decided as a dead guard is; a branch the held state, a stored or related row, a provider
or a replay also decides is not, and a swap whose overlap is partly left keeps its scoring. Two
branches acting on different stored rows, or one on a stored row and the other on none, keep their
scoring too: on a command that reads a row set, their order decides which row a request neither
of them takes reads. The entry names those branches as `overlap_taken_by`, apart from `unsatisfiable_guard` and
`identical_answer`.

A mutant on an outcome whose scenario the baseline suite already refuses, such as a branch no
arrangement reaches (`ESS-SYNTH-003`), cannot be killed by either suite. It is *unwitnessed* as
well, and its entry names the baseline refusal as `baseline_refusals`. So is a `from-drop` or
`transition-to` mutant on a transition that only such outcomes perform.

A mutant whose synthesized suite is the baseline's, scenario for scenario and step for step (the
digests aside), is *unwitnessed* too, where it would otherwise have survived or been inconclusive:
run on the target, that suite asks exactly what the baseline asked, so no scenario of it could kill
the mutant. Turning `expiry < now` into `expiry <= now` is one: the two differ only where the
stored expiry is the target's `now`, and synthesis keeps its witnesses a second from that
boundary. Its entry carries `unchanged_suite: true` and its text line ends `— ESS-MUTATE-004: the
suite does not change`. A killed or equivalent mutant keeps its verdict, and a run with authored
`--scenarios` never makes one, since those run on the mutant's own interpreter. So a survivor
always has a suite that differs from the baseline's and still passes
(beyond10x/ess#586).

The baseline is red only when a scenario failed or ended `error`. A scenario the target reports
`unsupported`, or the runner `skipped`, is listed as not scored, and every mutant is scored on the
scenarios the baseline executed. The scenarios the baseline did not execute are listed, not scored:
a mutant scenario the baseline did not execute is excluded and listed on the mutant. A mutant that
nothing killed is *inconclusive*, not a survivor, when an excluded scenario is one it changed: that
scenario might have killed it. An excluded scenario the mutant holds exactly as the baseline does
asks the target what the baseline asked, so it cannot, and the mutant can still survive. A
scenario new to a mutant's suite is scored. A baseline that executed nothing scores nothing.

**A survivor, or a mutant whose suite does not change, is first answered in the specification.**
Declare what makes the rule observable,
such as a view projecting the field a `sets` entry writes, or file a synthesis gap. An authored
scenario put in a mutant's suite could not answer it: its expectations are its author's, not the
model's, so against the unchanged target it runs identically in every mutant's suite.

| Exit | When |
|---|---|
| 0 | No baseline scenario failed or ended `error`, at least one mutant that is not equivalent ran, every scored mutant was killed or equivalent, none is inconclusive or unwitnessed, and no selected site is unavailable. Baseline scenarios that were not executed are listed, not scored. |
| 1 | The specification did not load, or at least one mutant survived. |
| 3 | `ESS-MUTATE-001` (a baseline scenario failed or ended `error`), nothing scored (the baseline executed no scenario), `ESS-MUTATE-003` (no site), or no survivor and at least one mutant unwitnessed or inconclusive or a selected site unavailable, or every mutant stillborn or equivalent. |

### Run your authored scenarios against each mutant

Some rules no synthesized scenario reaches: a counter limit farther than synthesis follows a
counter, for example, makes the baseline refuse the outcome's scenario, and every mutant on that
outcome is unwitnessed. An `ess-scenario/1` scenario you wrote can reach it. `--scenarios DIR`
runs those scenarios with `--target interpreted`:

```shell-session
$ ess verify conform mutate \
    --path model \
    --target interpreted \
    --scenarios scenarios
```

`DIR` is read, and its documents refused, exactly as `ess verify conform run --scenarios` reads
them; a refused document exits 1 before anything runs. Each authored scenario runs against the
interpreter of your specification as part of the baseline, and against the interpreter of each
mutant beside that mutant's synthesized suite, on the clock `run --target interpreted` uses: a
`now` guard is decided at the instant of the step, so a scenario `run` passes executes here too.
The interpreter of a mutant is an implementation of the mutant, so an authored scenario that fails
there kills it, and the report names it among the killers as it names a synthesized scenario
(`killed … — by <domain>/authored/<scenario>`). An authored scenario the baseline fails refuses the
audit with `ESS-MUTATE-001`, as a synthesized one does. An authored scenario the baseline executed that requires an outcome makes that
outcome witnessed, so a mutant on it is scored on what the scenarios observed rather than being unwitnessed
for the baseline's refusal alone; one that passes against the mutant leaves it surviving. One the
baseline did not execute (`unsupported` or `skipped`) witnesses nothing, and since it would have run
against the mutant's interpreter, it leaves a mutant nothing killed `inconclusive` rather than
surviving. A `precedence-swap` reported `equivalent` because both branches give the same answer stays
equivalent: an authored scenario that fails against it differs only in the outcome's name, which
names the branch and is no part of the answer.

`--scenarios` is refused with `--target billing` and `--target oracle-fixture`, which implement no
mutant, and with `--emit`, `--collect` and `--known-failing`: a runner of your own implements the
unchanged specification, where an authored scenario could never fail. Without `--scenarios` no
authored scenario runs, and the report is what it was.

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
- `manifest.json`, an `ess-mutation-manifest/3` listing all of them, or `/4` where it holds a
  `sets-drop`, `precedence-swap` or `emit-swap` mutant or an unavailable site, or names a component.
  A stillborn mutant has an entry and no suite; an unavailable site is listed under
  `unavailable_sites` and has no directory.

Run your runner over every `suite.json` and write its conformance report to `report.json` in the
same directory. The generated Go and TypeScript packages write the report named by
`ESS_REPORT_OUT` (`ESS_REPORT_FORMAT=2` for report/2); either report version is accepted. Then
score the reports:

```shell-session
$ ess verify conform mutate --collect target/mutants
mutation audit of billing v3 against billing-reference 0.51.0: 4 mutant(s), 3 killed, 0 survived, 1 inconclusive, 0 stillborn, 0 unwitnessed, 0 equivalent (baseline: 32 scenario(s), 0 refusal(s))
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

### Audit one component

A repository that implements one component of a larger system runs only that component's
scenarios. Scope the emission to it:

```shell-session
$ ess verify conform mutate --path examples/billing --emit target/mutants \
    --component invoice-service
```

Every suite is then the component's, exactly as `ess verify conform synthesize --component
invoice-service` writes it. A mutant is the component's when the site it mutates belongs to the
component, by the same membership that command uses: an outcome's guard, `sets`, error or events
and two outcomes' order belong to the component that handles the command (accepts it, or owns its
domain), a view's ranking to the component that owns the view, and a transition to a component
that handles a command performing it. Such a mutant is scored, and a survivor there is counted and
exits 1 like any other. A mutant on another component's site is that component's to answer: it is
marked `out_of_scope` in the `ess-mutation-manifest/4`, gets no suite, and `--collect` lists it in
an `ess-mutation-report/4` naming the component instead of scoring it. Out-of-scope mutants do not
change the exit status. An unavailable `emit-swap` site whose command another component handles is
listed with `outside_component` instead, and does not change the exit status either.
`--collect --component NAME` refuses an emission made for another component or for the whole
system, and `--target` takes no `--component`: the built-in targets
implement whole systems.

### Audit past known failures

A retrofit describes the behaviour a system is meant to have, and some of it is not there yet. One
failing baseline scenario refuses the whole audit with `ESS-MUTATE-001`, and it still does by
default. To audit the rest, declare exactly the scenarios that build is known to fail in an
[`ess-known-failures/1`](../../reference/formats.md#change-and-conformance-records) document, outside
the specification:

```json
{
  "failures": [
    {
      "reason": "cancelling also announces the invoice paid",
      "scenario": "billing.invoice.CancelInvoice/outcome/cancelled",
      "tracking": "ORDERS-412"
    }
  ],
  "format": "ess-known-failures/1",
  "implementation": "billing-service 4.2.0",
  "implementation_build": "sha256:…",
  "spec_digest": "…",
  "suite_digest": "sha256:…"
}
```

The declaration is bound to everything that produced the failure: the specification digest, the
SHA-256 of the exact baseline suite bytes (`baseline/suite.json` of an emission), the
implementation label the report names, and the public build, which the host states before the run
and never takes from the target. Scenario IDs match exactly. Pass it with `--known-failing FILE`:

- `--emit DIR --known-failing FILE` checks it against the baseline it writes, copies it to
  `DIR/known-failures.json` and binds it in an `ess-mutation-manifest/4`. Run each suite with your
  runner and also write the host's
  [`ess-conformance-execution/1`](../../reference/formats.md#change-and-conformance-records) beside
  each `report.json` as `execution.json`; the generated Go and TypeScript runners do when
  `ESS_IMPLEMENTATION_BUILD` and `ESS_EXECUTION_CONTEXT_OUT` are set. `--collect DIR` then scores
  under the bound declaration only: a different file given again, or one given to an emission that
  bound none, is refused, so the audit cannot change after its reports exist.
- `--target … --known-failing FILE` binds the running `ess` executable's SHA-256 as the build.

Each declared scenario must have failed in the baseline. One that passed is stale and refused, and
one that ended `error`, `unsupported` or `skipped` is refused; a failure the declaration does not
name still refuses with `ESS-MUTATE-001`. Each mutant is then scored on the scenarios the baseline
passed and nothing else: a declared scenario never kills, nor does a scenario the baseline's suite
does not hold. Such a scenario that the mutant changed, or added, could have killed it had the
baseline passed it, so the mutant is inconclusive instead of a survivor; one with no eligible
scenario at all is inconclusive, unless a gained or baseline synthesis refusal makes it unwitnessed. The report is an `ess-mutation-report/4` naming the declaration
and each mutant's `exclusions`, the text lists every known failure straight after its summary line,
and a refused declaration exits 2.

A matching declaration never makes conformance pass. `ess verify conform run` and `report` take the
same `--known-failing FILE` with `--accounting-out FILE` and write the failures, split into the
declared and the unexpected, in a separate
[`ess-known-failure-accounting/1`](../../reference/formats.md#change-and-conformance-records); the
report, its verdict and the exit status are what they are without it. For a report a generated
runner wrote with its execution context, `ess verify conform report --suite SUITE --observed-report
REPORT --execution-context CONTEXT --known-failing FILE --accounting-out FILE` writes the
accounting alone and rewrites nothing.

## Read the output

The text output prints one summary line, then any declared known failures, then the baseline
scenarios not scored, then out-of-scope mutants, then survivors, unwitnessed and inconclusive
mutants, then unavailable sites, then equivalent, stillborn and killed mutants, one line each. An
unavailable site's line reads `unavailable <id>: no_compatible_event_alternative — single-event
substitution was not audited here: …`, and the summary line ends `; N unavailable`.
`--report-out` writes an
[`ess-mutation-report/3`](../../reference/formats.md#change-and-conformance-records) document, or
`/4` with a component, a known-failure declaration or `unavailable_sites`, and `--format json`
prints the same bytes.
