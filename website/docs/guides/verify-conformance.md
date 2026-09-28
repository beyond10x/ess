---
title: Verify conformance
sidebar_position: 3
description: Generate the semantic suite a specification requires, run it, and emit a standalone ESS conformance report.
---

# Verify conformance

A specification declares more than an interface shape. Command outcomes, refused branches, state
transitions, emitted events, and invariants become semantic scenarios an implementation can be
checked against.

## Generate the suite

```shell-session
$ ess verify conform synthesize \
    --path examples/billing \
    --out target/billing-suite.json
```

The suite is deterministic. Its provenance names the model and contract digests from which it was
derived.

A generated scenario pins what the specification declares, not only that a command answered:

| declaration | what the suite requires |
| --- | --- |
| a transition `B → C`, where a view projects `state` | the row is read back in `C` |
| `from: [A, B]` on a transition | the move runs from `A` and from `B`, each on its own instance |
| an update's `sets:` | each field is sent a value the row does not already hold: not what the setup wrote, and not an enum's first variant where nothing wrote it |
| `when: n >= 1` over an input | the branch is taken at `n: 1`, and its default refuses at `n: 0` with every other conjunct satisfied |

Boundaries are probed for comparisons of a number or a timestamp with a literal. Text and equality
comparisons are not. The extra checks reuse the existing steps and scenario ids, so the suite format
does not change.

Add `--compact` to fresh `synthesize` output to write deterministic JSON without
indentation, followed by one newline. The decoded suite is unchanged; its exact
byte digest changes, so retain that original compact file when producing reports
or selecting child suites. Pretty output remains the default. This option does
not rewrite an already committed suite.

## Select authored scenarios explicitly

Scenarios a person wrote (`ess-scenario/*` documents) join the generated ones only when
`--scenarios` names them. It takes one file or one directory:

| `--scenarios` names | What is read |
|---|---|
| a file | that file, whatever its extension |
| a directory with an `ess-inputs.yaml` | exactly the files its `scenarios:` list names, nested or not, of any extension |
| a directory without one | its immediate lowercase `.yaml` and `.yml` files; subdirectories are not searched |
| nothing | no authored scenarios, even when the model's `ess-inputs.yaml` lists some |

An empty selection is refused before anything is written or run. The
[mixed-layout example](write-a-specification.md#keep-sources-and-generated-output-together) keeps
the model and the scenarios in one directory, so the same directory serves both options:

```shell-session
$ ess verify conform author --path . --scenarios . --suite-format 5 --out output/authored.json
```

`ess-inputs.yaml` refuses duplicate paths, a path listed as both a specification and a scenario,
a path that leaves the directory, and a symlink. A coverage suite (`--suite-format 5`) records each
scenario's relative path and source text: moving the directory or reordering the list leaves the
suite unchanged, and changing line endings changes it. Its coverage describes the selected
scenarios, not every scenario file below the directory.

`ess verify conform run --suite FILE` runs the committed suite as it is and selects nothing.

## Establish backend state in an authored scenario

`ess-scenario/2`, introduced in 0.23.0, supports typed setup for entities whose rows arrive
from an upstream system. A scenario can establish those rows and query their
view without inventing a creator command:

```yaml
type: ess-scenario/2
domain: calls.history
scenario: recent-call
summary: A stored call appears in history.
arrange:
  - instance: recent
    entity: calls.history.CallRecord
    setup:
      identity: 00000000-0000-4000-8000-000000000001
      fields: {started_at: '2026-01-05T09:00:00Z', duration_seconds: 12}
      state: Completed
assert:
  - view: calls.history.CallHistory
    contains: {call_id: {$instance: recent}, duration_seconds: 12}
```

The model must declare that entity, its field types, lifecycle state and view.
Setup validates identity, required fields, nested values and invariants. Duplicate
qualified identities, null identities and inconclusive invariants refuse.

The adapter must establish actual isolated backend state and make it visible
before acknowledging setup. Setup emits no command or event and claims no
lifecycle path. Rust and generated Go expose an optional setup capability;
unsupported adapters produce a non-passing result. These steps use suite/6 or
coverage suite/7 and require explicit report/2. Source scenario/1 refuses setup.

## Observe outcomes selected by held state

For a command declaring `when_subject_state`, synthesis establishes a real
reachable state, queries a declared immediate view exposing identity and state,
invokes the command, and checks the resulting row. The same input can therefore
prove different outcomes from different held states. An implementation that
returns the expected outcome while incorrectly changing the row fails.

The initial adapter requires an unfiltered immediate view without parameters.
Missing observation authority or an unreachable required state produces an
explicit synthesis refusal. This uses existing command and view assertions;
the state guard alone does not require a newer suite vocabulary. The source
declaration requires `ess/3`.

## Observe retries of the original result

Source `ess/7`, introduced in 0.29.0, lets an outcome declare `replays` with
the name of an earlier successful outcome in the same command. The generated
witness executes that original outcome, captures its actual result and subject
identity, then retries the same input with the same actor. The retry must return
the exact original typed response without an error, direct event or subject
change.

Synthesis must establish that the retry's own condition holds immediately after
the original command. An unreachable or unprovable immediate retry produces a
named refusal. A valid model can still require an authored witness for a retry
that becomes eligible only after later activity.

The target must expose the complete subject through declared immediate,
unfiltered views. Complete observations check required fields and their types
in both actual query results before comparing all returned fields. Returning
only the identity fails even when that partial row stays unchanged. Optional
absence is distinct from null; extra returned fields also participate in the
comparison. Declared Integer values must reach the adapter without rounding.
Recursive Decimal and Binary64 response or complete-row positions are outside
this observation profile and refuse.

Source `ess/7` uses the same complete observations for ordinary `wrong_state`
refusal witnesses. Existing independent snapshot steps remain available with
their original, weaker contract. A generic error assertion alone does not
establish subject preservation.

These observations select suite/12 or declared-coverage suite/13 and require
report/2 in Rust and generated Go. TypeScript and browser runners refuse these
envelopes before invoking the target. An immediate retry witness does not prove
restart recovery, retries after a later head, or absence of physical writes;
those remain implementation-specific acceptance.

## Observe selection, periodic activity and clock evidence

Selection observations, introduced in 0.23.0, compare actual source occurrences and selected
indices, preserving optional absence and occurrence-based exclusion. A host
conversion remains an explicit obligation; declaring the conversion does not
execute or prove it.

Periodic checks exercise the named host under controlled target time: ready,
initially inactive, failed first read and slow first read. The runner examines
actual scoped timer, read and invocation facts, including the complete live
interval and a window after stop acknowledgement. Missing ticks, overlapping
work, stale mapped inputs and post-stop activity fail. Unsupported authority
produces a non-passing result. The bounded check observes at most five live
periods; it does not sleep in the runner.

Clock comparisons use the typed `ExpectReadingOrder` operation with references
to already observed event members. The adapter supplies occurrence-scoped
process, epoch, origin and formatter facts; the runner normalizes and compares.
Declared alternative origins are requirements, not evidence. Wrong occurrence,
unknown offset, differing process/epoch or mutated request requirements cannot
establish success. Authored YAML convenience syntax for this comparison is not
provided; construct and persist the typed operation through the admitted writer.

These operations use suite/6 or declared-coverage suite/7 and explicit report/2.
Previous readers refuse their vocabulary. Controlled adapter tests do not prove
that an unrelated production adapter implements these capabilities.

## Run a supported target

```shell-session
$ ess verify conform run \
    --suite target/billing-suite.json \
    --target billing \
    --report-out target/billing-conformance.json
```

The command executes the generated or committed scenarios against the selected reference target.
The built-in choices are `billing`, `oracle-fixture` and `interpreted`; a production adapter must
establish its own execution boundary. `billing` and `oracle-fixture` are hand-written reference
implementations of their examples. `interpreted` selects the specification itself, named by
`--path`, and refuses one whose `spec_digest` is not the suite's. It executes commands from the
model — outcomes, transitions, `sets:` writes, emitted events and declared refusals — and does not
yet interpret views or bindings, so a scenario that reads one comes back as an unsatisfied
obligation, and a run over a suite holding at least one such scenario fails.
The default standalone report is `ess-conformance-report/1`. Its historical `scenarios_failed`
count includes every non-pass, including Go skips and Rust errors or unsupported results. Those
legacy bytes and meanings remain unchanged.

To run directly from a specification instead of a pre-generated suite:

```shell-session
$ ess verify conform run \
    --path examples/billing \
    --target billing \
    --format json
```

## Hold your own implementation to the suite

`ess verify conform run` runs a suite only against the targets built into `ess`. Your own
implementation is held to its suite through a test package that `ess` writes in Go or TypeScript:

```shell-session
$ ess verify conform synthesize --path spec --target go --out internal/conformance
$ ess verify conform synthesize --path spec --target typescript --out conformance
```

Each writes one package, `essconform`, below `--out`: the suite (`suite.json`), the compiled model
(`ir.json`), the runner, a predicate evaluator and a `README.md` describing the wiring. Nothing in it
is edited by hand; regenerate it after every change to the specification. Files you add beside the
generated ones, such as your test file, are kept.

The runner asks the implementation questions through one interface, `Target`, and asserts the
answers itself:

| Method (Go / TypeScript) | What it answers |
|---|---|
| `Identity` / `identity` | the implementation's name and version, for the report |
| `BeginScenario`, `EndScenario` / `beginScenario`, `endScenario` | bracket one scenario; state from one scenario must not satisfy another |
| `ExecuteCommand` / `executeCommand` | run one command as the named actor; return the outcome taken, the declared error if it refused, and the events it emitted directly |
| `QueryView` / `queryView` | read one view; a `read_your_writes` view must already show the command that just returned |
| `ObserveEvents` / `observeEvents` | the events seen for one activity, away from the command that caused them |
| `ConfigureExternalOutcome` / `configureExternalOutcome` | force an outcome declared `external:` |
| `RedeliverEvent` / `redeliverEvent` | deliver an event a second time, for `delivery: at_least_once` |
| `ObserveInvocations` / `observeInvocations` | the commands one binding invoked and what it passed |

A refused command reports both the outcome name and the error, for example
`{outcome: "rejected", error: "tasks.list.InvalidPriority"}`. A method the implementation cannot
answer returns `ErrUnsupported` (Go) or throws it (TypeScript). The scenario is then reported as
skipped, which is a different fact from failed, and a run with a skipped scenario is
`inconclusive`, not `passed`. Specifications that declare backend setup, clock readings or periodic
hosts ask for further optional interfaces (`EntitySetupTarget`, `ClockReadingTarget`,
`PeriodicTarget`); the generated `README.md` names the ones a suite needs.

Hand a factory to the runner from one test. The runner builds one target per scenario:

```go
func TestConformance(t *testing.T) {
    essconform.Run(t, func() essconform.Target { return newTarget() })
}
```

```ts
await test("conformance", async (t) => {
  await run(t, (): Target => newTarget());
});
```

Then run the language's own test command. For a suite at `ess-conformance/5` or later, the
generated runner executes only with `ESS_REPORT_FORMAT=2` set, and without it stops before the first
scenario; the package's `README.md` names the suite's version and the command. `ESS_REPORT_OUT`
names a file for the standalone report:

```shell-session
$ ESS_REPORT_FORMAT=2 ESS_REPORT_OUT=$PWD/report.json go test ./...
$ ESS_REPORT_FORMAT=2 ESS_REPORT_OUT=$PWD/report.json npm test
```

[Getting started](../getting-started.md#hold-an-implementation-to-the-specification) builds a
complete TypeScript target for a small specification and runs it green. For separate passed, failed
and skipped counts, see [explicit outcome counts](#opt-into-explicit-outcome-counts). A target
reports what it observed; it must not report its own unobserved success.

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

The baseline is red only when a scenario failed or ended `error`. A scenario the target reports
`unsupported`, or the runner `skipped`, is listed as not scored, and every mutant is scored on the
scenarios the baseline executed; a mutant scenario the baseline did not execute is excluded and
listed on the mutant. A scenario new to a mutant's suite is scored. A baseline that executed nothing
scores nothing.

**A survivor is not answered by authoring a scenario.** An authored scenario's expectations are its
author's, not the model's, so it runs identically in every mutant's suite and can never kill one;
`mutate` runs none. Answer a survivor by declaring what makes the rule observable, such as a view
projecting the field a `sets` entry writes, or by filing a synthesis gap.

| Exit | When |
|---|---|
| 0 | The baseline passed, at least one mutant ran, and every mutant that ran was killed. |
| 1 | The specification did not load, or at least one mutant survived. |
| 3 | `ESS-MUTATE-001` (a baseline scenario failed or ended `error`), nothing scored (the baseline executed no scenario), `ESS-MUTATE-003` (no site), or no survivor and at least one mutant unwitnessed or inconclusive, or every mutant stillborn. |

The text output prints one summary line, then the baseline scenarios not scored, then survivors,
unwitnessed, inconclusive, stillborn and killed mutants, one line each. `--report-out` writes an
[`ess-mutation-report/2`](../reference/formats.md#change-and-conformance-records) document, and
`--format json` prints the same bytes. Only the built-in targets are supported; replaying mutant
suites in an adopter's own language is not implemented yet.

## Explore random command sequences

Generated and authored scenarios are short, fixed paths. The TypeScript and Go packages that
`ess verify conform synthesize --target typescript|go` writes also carry an explorer: seeded random
walks over the commands, driven through the same `Target` you implement for the suite, and checked
after every step against a reference model interpreted from the specification. It finds faults
that only show later in a sequence: a view that drops rows after the fifth, a refusal that still
writes, an identity reused on the fourth create.

```ts
import { assertExplored, explore } from './index.js';

const result = await explore(() => newTarget(), { seeds: 200, steps: 60 });
assertExplored(result);                          // fails on a disagreement or an unreached outcome
assertExplored(result, { allowExcluded: true }); // also accepts outcomes the explorer left out
```

```go
result, err := essconform.Explore(func() essconform.Target { return newTarget() },
    essconform.ExploreOptions{Seeds: 200, Steps: 60})
if err != nil { t.Fatal(err) }
essconform.AssertExplored(t, result, essconform.AssertOptions{})
```

`seeds` sequences run, seeded 1 to `seeds`, each on a fresh target; `steps` is the number of
commands in each (defaults 200 and 60). A failure names its seed, and `seed` runs exactly that one
sequence again. One seed draws the same sequence in both languages.

After every step the explorer compares the outcome, the error, the direct events and every payload
field the specification determines, then every view without parameters over an entity: its row
count, identities, determined fields and `order_by`. A `read_your_writes` view is read once with the
command's consistency token; an `eventual` view is polled until it agrees, up to the eight attempts
an `eventually` step allows. Last, every invariant is evaluated over the model's records; a record
that breaks one is reported as a specification defect, because the guards allowed a sequence the
invariants forbid. A failure is shrunk by removing steps while the shorter trace still fails the
same way, for at most 1,000 replays.

`assertExplored` (`AssertExplored`) fails on a disagreement, on a declared outcome of an included
command that no sequence reached, and on the outcomes of an excluded command. The explorer models a
subset: `when`, `otherwise` and `wrong_state` conditions; `creates` with an observed identity and
`moves`/`updates` of a supplied subject; integer, boolean, string and UUID inputs, their newtypes,
enums and structs of them. Anything else is excluded with the reason in `excluded`, and accepting
that is an explicit `allowExcluded`. Where two guards both hold — which the model admits over an
infinite domain — the draw is reported in `ambiguous` and redrawn rather than decided; a view
filter or invariant over a field no command set is reported in `undetermined`. Neither fails.

The model is `ir.json`, the compact IR the suite's `spec_digest` is taken over. The explorer refuses
a package whose `ir.json` does not hash to `suite.json`'s digest; regenerate the package rather than
editing either file.

## Check a concurrent history

A suite and the explorer drive a target one call at a time, so a race between two clients never
happens under them. `check-history` reads an
`ess-history/1` document, one run of several clients with each call's invoke and return instants,
and searches for an order of the calls that the specification's own model accepts, answer for
answer.

```shell-session
$ ess verify conform check-history \
    --path examples/billing \
    --history target/history.json
```

| exit | meaning |
|---|---|
| 0 | `Linearizable`: some order of the calls explains every recorded answer. |
| 1 | `Violation`: no order does. The report names the longest partial order found and a shrunk history that is still a violation. |
| 3 | `Unknown`: the search spent `--budget` model executions (default 1,000,000) first. Unknown is not a pass. |
| 2 | The specification did not load, or the history was refused, for example because it was recorded against another specification. |

The search is split by subject: calls on different instances are checked apart. A call that never
answered may have taken effect or not, and is placed after every other call. A history records no
inputs, so a call is explained by any input the suite would submit for its command. A read of a view
that records its rows is judged at the consistency the view declares: under `read_your_writes` no
client reads a state older than its own last write, and under `eventual` each client's reads converge
once its first `--settle` reads after the last write (default 4) are past. Reads that cannot be judged
are listed with their reason. The same history and budget always print the same report;
`--format json` prints it as JSON.

### Draw a history as client lanes

```shell-session
$ ess verify conform web \
    --path examples/billing \
    --history target/history.json \
    --out target/lanes
```

`web --history` checks the history as `check-history` does and writes one `index.html`, or prints
it when `--out` is absent. Each client is a lane, each call a bar from its invoke to its return, and
each call the search placed carries its position in the order found. A history that declares more
than 16 clients draws a lane for each client that made a call and counts the rest in one row. For a violation, the page
marks the call where the search failed. Where one other call explains the failure, it names that
call too, with the state each of the two needed and the state the other order left. Below that is
the shrunk history, drawn the same way. The page carries its stylesheet and no script, so it opens
from disk and fetches nothing. The same history renders to the same bytes. It exits 0 whatever the
verdict; the verdict as an exit status is `check-history`'s.

`--out` replaces the files `ess` owns in that directory, including a scenario player's, so write
history pages and the player to different directories.
### Import a recorded log

A service that logs its calls can be judged from its log. `import-history` reads a JSON Lines log,
one call per line in the log's own shape, through an adapter you write, and writes `ess-history/1`:

```yaml
format: ess-history-adapter/1
fields:
  operation_id: { pointer: /correlation }
  client: { pointer: /request/client }
  command: { pointer: /request/command }
  subject_key: { pointer: /request/subject }
  invoked_at: { pointer: /request/at_ms }
  returned_at: { pointer: /response/at_ms }
  outcome: { pointer: /response/outcome }
  completion:
    pointer: /response/status
    values: { ok: Returned, timeout: Indeterminate }
```

```shell-session
$ ess verify conform import-history --path examples/billing \
    --log calls.jsonl --adapter adapter.yaml --output target/history.json
$ ess verify conform check-history --path examples/billing --history target/history.json
```

Every field is either a JSON pointer or `absent`. Nothing is guessed. If a line lacks a field that
the call cannot be judged without, the import is refused (exit 2) and each such field is named on
its line. Those fields are the client, command, subject, invoke instant and completion, plus the
return instant and outcome of a `Returned` call. Refusals name the log line and the field. Other
fields can be missing, and each case is reported as a `coverage-gap`:

- A missing `operation_id` is given a generated version-8 UUID. No ESS writer uses version 8, so a
  generated ID cannot collide with a carried one.
- `rows` is optional in the adapter. A view read without rows is imported, but `check-history` will
  not judge it.
- The document's `seed` is never carried and is written as 0.

Gaps are printed on stderr. With `--output FILE`, they are also written as a JSON array to
`FILE.gaps.json`. This file is always written; it is never empty because `seed` is always a gap.
A history imported with gaps carries seed 0 and generated IDs by construction, and only the gaps
file records which values were not in the log. An `--output` is refused if it or its gaps file is
the `--log` or `--adapter` file (hard links included) or a file of the `--path` specification.
Both files are written to temporary siblings, and replace existing files only once both writes have
succeeded.

Instants must be unsigned integers, such as epoch milliseconds. Client labels are numbered in the
order they first appear.

## Opt into explicit outcome counts

### Where passed, failed and skipped live

`ess-conformance-report/1` has no passed or skipped count, and it lists a skip as `skipped <id>`
inside `failed_scenarios`. The three numbers are in `ess-conformance-report/2`, as `counts.passed`,
`counts.failed` and `counts.skipped`, and the skipped ids are in `outcomes.skipped`, apart from
`outcomes.failed`. A baseline that floors `passed + failed` and caps `skipped` reads them from
there. Select it for each runner:

| runner | selection |
|---|---|
| `ess verify conform run` | `--report-format 2 --report-out <file>` |
| generated Go | `ESS_REPORT_FORMAT=2 ESS_REPORT_OUT=<file> go test ./...` |
| generated TypeScript | `ESS_REPORT_FORMAT=2 ESS_REPORT_OUT=<file> npm test` |

`/1` stays the default and keeps its meaning. [Why the counts are a separate
version](https://github.com/beyond10x/ess/blob/main/docs/design/truthful-conformance-counts.md).

```shell-session
$ ess verify conform run \
    --suite target/billing-suite.json \
    --target billing \
    --report-format 2 \
    --allow-incomplete \
    --report-out target/billing-counts.json \
    --format json
```

`--report-format 2` selects `ess-conformance-report/2` for `--report-out`. It records separate
`passed`, `failed`, `error`, `unsupported` and `skipped` counts and sorted scenario IDs. Rust uses
error/unsupported; generated Go uses skipped and keeps ordinary target errors as failed. The report
binds every original suite byte, including its final newline, under `sha256-json-bytes/1`. Retain the
original suite beside the report; reformatting it changes that identity.

Legacy suite versions 1–4 have coverage exactly `unknown`, so even an empty or
all-pass execution has inconclusive conformance. `--allow-incomplete` explicitly chooses diagnostic
execution: Rust exits 0 for passed execution, 1 for failed and 3 for inconclusive execution.
`--strict` requires explicit `--report-format 2` and succeeds only for passed conformance; unknown
coverage therefore exits nonzero. Strict and allow-incomplete cannot be combined. Neither control
changes the report format automatically. Suite/4 and diagnostic report/1 remain the defaults.

With report format 2, `--format json` or `--format yaml` presents a separate closed
`ess-conformance-run/2` object containing `format`, the standalone `summary`, `started_at` and ordered
`scenarios`. `--report-out` always writes the standalone JSON summary. Both new formats preserve
unsigned epoch milliseconds exactly through the full u64 range. The default detailed output remains
the legacy unversioned structure.

Regenerate a Go package explicitly to use the new writer, then run:

```shell-session
$ ESS_REPORT_FORMAT=2 ESS_CONFORMANCE_ALLOW_INCOMPLETE=1 ESS_REPORT_OUT=report.json go test ./...
```

Generated Go defaults remain report/1 and diagnostic execution. `ESS_CONFORMANCE_STRICT=1` requires
report/2 and fails the enclosing test even when every subtest skipped. The strictness variables accept
only unset or `1`; conflicting settings refuse before target construction. Omitted or abnormally
terminated selected subtests, a negative report/2 clock or a failed report write cannot publish a
complete report. These checks also apply when no report destination is requested. Retained generated
packages keep their own runtime behavior until regenerated; upgrading the standalone binary does
not update them. Set `ESS_REPORT_OUT` as well as `ESS_REPORT_FORMAT=2` to retain the count report.
An absolute output path avoids package working-directory differences in `go test ./...`.
A failed conformance run still writes its separate counts; report delivery does not turn failures
or skipped scenarios into passing evidence.

## Opt into declared coverage

The suite/5, original-byte carrier and paired replay features in this section were introduced in
[0.21.0](https://github.com/beyond10x/ess/releases/tag/0.21.0). 0.20.0 establishes the report/2
count surface described above, not these later coverage features. The default suite/4 and report/1
paths remain unchanged.

```shell-session
$ ess verify conform synthesize --path examples/billing \
    --scenarios examples/billing-scenarios --suite-format 5 --out target/coverage-suite.json
$ ess verify conform run --suite target/coverage-suite.json --target billing \
    --report-format 2 --strict --report-out target/coverage-report.json
```

Suite/5 retains the declared scope, origins, selected IDs, known outside scenarios, every refusal
occurrence and every requested authored source's relative identity and exact byte digest. Generated
and authored origins can be selected independently; `--component` limits fresh synthesis to that
component. `conform author --suite-format 5` inventories only its supplied authored files. Library
callers can explicitly retain known excluded generated inventory with
`coverage_build::build_with_known_generated`.

A nonempty all-pass selection qualifies only when its inventory is complete and has no in-scope
refusal. Unknown inventory, an empty selection or a missing required check remains inconclusive.
Filtering cannot remove a refusal or promote unknown knowledge. Every identical refusal occurrence
remains visible; a useful passing scenario can have missing checks beside it. Strict mode fails
when conformance is inconclusive, while diagnostic mode retains the execution result's exit rules.
Suite/5 always requires explicit report/2, including with `--allow-incomplete` or no report output.

To narrow an admitted suite, supply a JSON array of sorted distinct scenario IDs; `[]` explicitly
selects none:

```shell-session
$ ess verify conform select --suite target/coverage-suite.json \
    --ids selected-ids.json --out target/coverage-input.json
$ ess verify conform run --suite-input target/coverage-input.json --target billing \
    --report-format 2 --strict --report-out target/selected-report.json
```

The closed `ess-conformance-input/1` retains the selected original JSON string and every original
parent string, nearest first. Repeated selection uses `--suite-input` and retains the full chain.
An explicit child cannot be run from its raw suite alone. Admission checks each full surviving
scenario definition, dependency, source map, refusal and selection against the exact parent.
Only the inner selected bytes are hashed; reformatting the carrier preserves that identity.

Authored directory discovery remains shallow. Identities use `/`-separated relative UTF-8 segments;
absolute paths, dot segments, backslashes, colons, controls, invalid UTF-8 filenames and selected
symlinks refuse. Relocating a root preserves identities; changing source newlines changes its digest.
The historical `ConformanceSuite` DTO remains unadmitted: it cannot recover discarded source fields
or become known coverage by changing its version. Original inputs use `AdmittedSuite` or
`coverage::AdmittedInput`; count reports require the immutable actual `ExecutedRun` capability.

Generate Go with `--suite-format 5 --target go`, or call `go::emit_input` with an admitted carrier.
The runtime checks every original parent before creating a target. Wire metadata retains the full
u64 range; only selected execution fields must fit the host's Go `int`. Unrepresentable selected
counts, indices or halt limits refuse before callbacks or reports. An oversized omitted parent
field does not block a representable child. The Go report clock supports nonnegative int64 values;
Rust report/run timestamps support the full u64 range. Payload Number meaning is a separate finite
binary64 boundary. Modeled Binary64 remains unsupported by conformance, including suite/5.

`conform web --suite-format 5` emits a paired `ess-conformance-replay/1` document and matching player.
The actual player checks the closed projection, exact suite reference and full input before creating
replay state, then displays selection and refusals. It emits no execution report. The reduced model
omits literal assignment values and full view evaluation; digest comparison neither reconstructs
the full compiled model nor authenticates its publisher. Existing players do not acquire these
checks when handed new metadata; regenerate and distribute the paired bundle together.

`ess impact --suite-input` accepts complete admitted coverage and reports its selection separately.
Unknown or incomplete inventory and missing parents refuse. Persisted output remains `ess-impact/3`
with its existing fields; invalidation within a selection is not whole-system execution evidence.

## Observe bounded binding accessors

The `ess/3` binding paths described in
[Write a specification](write-a-specification.md#read-a-field-inside-an-event-envelope)
require new suite vocabulary, introduced alongside them in 0.23.0. An ordinary suite retaining an
accessor uses `ess-conformance/6`; a declared-coverage suite retaining the new accessor
vocabulary uses `ess-conformance/7`. Models that do not retain that vocabulary
keep their existing suite formats. Both new versions require explicit
`--report-format 2` for execution. Report format 1 is refused before the target
runs, even without `--report-out` or when incomplete execution is allowed.

The observation checks the declared path against the triggering event and the
resulting command input. It distinguishes an unavailable path from a terminal
value and rejects missing required members, invalid union discriminators and wrong
payload kinds along the traversal. Whole-value collection copies do not validate
their elements. A conversion declaration supplies a reason
for a type crossing, not an executable conversion algorithm: a mapping whose
expected input cannot be determined receives a capability refusal.

Native Rust and Go values can distinguish nested Optional states that serialize
to the same JSON `null`. When that distinction is necessary to decide whether a
mapping is correct, conformance refuses the ambiguous observation instead of
guessing. Copying the typed native value remains distinct from proving that copy
through a JSON observation.

Ordinary suite/6 retains unknown coverage. Suite/7 uses the same exact-input and
parent-lineage rules as suite/5; selecting a child preserves its version and
requires the original parent chain. A passing run proves only the admitted
selection and does not resolve any recorded refusal.

## What the report proves

The report proves what its scenarios observed against the named target and specification digest. It
does not prove that the target is independently operated, deployed in production, or free from
behavior the suite never exercised. A consumer may translate this report into its own evidence
vocabulary at that consumer’s boundary; ESS itself publishes no workflow or planning record.

## A target in Rust

The generated Go and TypeScript packages [above](#hold-your-own-implementation-to-the-suite) are the
route for an implementation outside this repository. A Rust implementation can instead implement
the `ConformanceTarget` trait of the `ess-conformance` crate, which is what the built-in targets do.
It must preserve scenario identity, request/response correlation and refusal semantics, and it must
not report its own unobserved success.
