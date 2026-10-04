---
title: Runners and reports
sidebar_position: 3
description: Run a suite against a built-in target, a generated Go or TypeScript package, or a Rust target, and read what the report counts and proves.
---

# Runners and reports

For response fields that must never be disclosed again after issuance, see
[One-time response values](./one-time-responses.md).

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

`ObserveEvents` must read the implementation's event log for the requested event and correlation,
including repeated occurrences; it cannot return only the last command's direct events. The runner
uses it even when the command answers `not_granted`. Before sending a command whose next step is
`expect_not_granted`, it counts every event named in that step's `unpublished` list. After the
refusal, it observes those events again in the same correlation context and fails if any count grew.
A command that publishes and then refuses therefore fails even when its answer contains no events.

A custom suite runner, one that reads `suite.json` itself instead of using the generated package,
must do the same. For every `execute_command` or `execute_command_without_input` step whose next
step is `expect_not_granted`:

1. Look ahead before sending. Read the next step and take its `unpublished` list. The suite has no
   separate observation step; the pre-send count belongs to `expect_not_granted`.
2. Count before the send. Observe each listed event once, with the scenario's correlation (the one
   `BeginScenario` received and every command carries), and count the occurrences of that event.
   The count reads what the log already holds, so the generated runners pass a deadline that does
   not wait (`attempts: 1`).
3. Send the command in that correlation. The step requires the standard refusal naming the actor
   the step names. A command that ran, or a refusal that hands back events, fails the step.
4. Count again after the refusal. Observe each listed event with the same correlation, and fail the
   step if any count is higher than before. Count occurrences rather than comparing sets or
   payloads: a refused send that publishes a second occurrence equal to an earlier one has still
   added one.
5. If the target cannot observe the log, before or after, the scenario is `unsupported` and never
   passed. If the pre-send count is unavailable, the command is not sent.

Neither the command's answer nor the views can stand in for these observations. The answer carries
only direct events, so it misses an event published elsewhere before the refusal. Comparing views
misses an event that changes no view. A runner that checks only the answer or the views has not
verified `unpublished` and must not report the scenario passed.

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

[Getting started](../../start/first-conformance-run.md) builds a
complete TypeScript target for a small specification and runs it green. For separate passed, failed
and skipped counts, see [explicit outcome counts](#opt-into-explicit-outcome-counts). A target
reports what it observed; it must not report its own unobserved success.

## A target in Rust

The generated Go and TypeScript packages [above](#hold-your-own-implementation-to-the-suite) are the
route for an implementation outside this repository. A Rust implementation can instead implement
the `ConformanceTarget` trait of the `ess-conformance` crate, which is what the built-in targets do.
It must preserve scenario identity, request/response correlation and refusal semantics, and it must
not report its own unobserved success.

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
`passed`, `failed`, `error`, `unsupported` and `skipped` counts and sorted scenario IDs. Rust and
newly generated Go and TypeScript retain all five categories. Go and TypeScript use
`go-scenario-status/2`; older `go-scenario-status/1` reports retain their original restricted
categories. Unsupported observations fail execution; errors or explicit skips make execution
inconclusive unless another result has already failed it. The report
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

`ess verify impact --suite-input` accepts complete admitted coverage and reports its selection separately.
Unknown or incomplete inventory and missing parents refuse. Persisted output remains `ess-impact/3`
with its existing fields; invalidation within a selection is not whole-system execution evidence.

## What the report proves

The report proves what its scenarios observed against the named target and specification digest. It
does not prove that the target is independently operated, deployed in production, or free from
behavior the suite never exercised. A consumer may translate this report into its own evidence
vocabulary at that consumer’s boundary; ESS itself publishes no workflow or planning record.
