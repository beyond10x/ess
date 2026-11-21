---
title: Runners and reports
sidebar_position: 3
description: Run a suite against a built-in target, a generated Go or TypeScript package, an adapter process in any language, or a Rust target, and read what the report counts and proves.
---

# Runners and reports

For response fields that must never be disclosed again after issuance, see
[One-time response values](./one-time-responses.md).

## Run a supported target

```shell-session
$ ess verify conform run \
    --suite target/billing-suite.json \
    --target billing \
    --report-format 2 \
    --report-out target/billing-conformance.json
```

The command executes the generated or committed scenarios against the selected target.
The built-in choices are `billing`, `oracle-fixture` and `interpreted`, and the one other value,
`process`, starts a program of yours instead ([below](#an-implementation-in-any-language)); a
production adapter must establish its own execution boundary. `billing` and `oracle-fixture` are
hand-written reference implementations of their examples. `interpreted` selects the specification itself, named by
`--path`, and refuses one whose `spec_digest` is not the suite's. It executes commands from the
model — outcomes, transitions, `sets:` writes, emitted events and declared refusals — and does not
yet interpret views or bindings, so a scenario that reads one comes back as an unsatisfied
obligation, and a run over a suite holding at least one such scenario fails.
A freshly synthesized suite is `ess-conformance/34`, which runs only with `--report-format 2`.
The default standalone report, `ess-conformance-report/1`, is for suites up to
`ess-conformance/4`. Its historical `scenarios_failed` count includes every non-pass, including Go
skips and Rust errors or unsupported results. Those legacy bytes and meanings remain unchanged.

To run directly from a specification instead of a pre-generated suite:

```shell-session
$ ess verify conform run \
    --path examples/billing \
    --target billing \
    --report-format 2 \
    --format json
```

## Hold your own implementation to the suite

`ess verify conform run` runs a suite against a target built into `ess`, or against a program of
yours that answers its questions ([An implementation in any
language](#an-implementation-in-any-language)). An implementation in Go or TypeScript can instead be
held to its suite through a test package that `ess` writes in that language:

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

## An implementation in any language

`ess verify conform run --target process` holds an implementation written in any language to its
suite. `ess` starts a program you write, the adapter, and asks it the suite's questions as one JSON
object per line on its standard input; the adapter answers each with one JSON line on its standard
output, by driving your implementation. `ess` judges the answers itself and writes the report a
built-in target gets: the adapter reports what it observed, never a verdict.

### Run it

Put the `tasks` specification of [Write your first
specification](../../start/first-specification.md) in `spec/` and the [adapter
below](#a-complete-adapter) in `adapter/adapter.py`, then:

```shell-session
$ ess verify conform synthesize --path spec --suite-format 5 --out suite.json
6 selected scenario(s), 0 authored source(s), 0 refusal occurrence(s)
$ ess verify conform run --suite suite.json --target process \
    --target-dir adapter --target-timeout 30 \
    --report-format 2 --strict --report-out report.json \
    -- python3 adapter.py
Requires an empty logical modeled-instance/event/invocation namespace before each scenario setup; unrelated physical data need not be deleted.
Requires an empty logical modeled-instance/event/invocation namespace before each scenario setup; unrelated physical data need not be deleted.
tasks v1 against tasks-py 1.0.0 — passed
  passed tasks.list.AddTask/outcome/added
  passed tasks.list.AddTask/outcome/rejected
  passed tasks.list.CompleteTask/outcome/already-done
  passed tasks.list.CompleteTask/outcome/completed
  passed tasks.list.Task/state/Done/refuses/tasks.list.CompleteTask
  passed tasks.list.Task/transition/complete/by/tasks.list.CompleteTask/completed
  6 scenarios: 6 passed, 0 failed, 0 error, 0 unsupported
```

Everything after `--` is the adapter's command. It is executed directly, without a shell, in
`--target-dir` (the current directory when absent). A relative program containing `/`, such as
`./adapter`, is found there, and a bare name, such as `python3`, on `PATH`. The adapter inherits
the environment, and its standard error passes through to yours. `--target-timeout` is how many
seconds `ess` waits for each answer, 1 to 3600, 30 when absent.

With `--suite-format 5` and `--report-format 2 --strict`, the exit status is the verdict: 0 when
every scenario passed, 1 when one failed or was unsupported, 3 when the run is inconclusive because
a scenario ended `error`. Exit 2 is a refusal before any scenario, and writes no report: no command
after `--`, a program that cannot be started, or a failed handshake.

### The messages

`ess` sends one request and reads its answer before it sends the next. On the wire each message is
one line; they are shown wrapped here, with correlation ids and digests shortened:

```json
{"id": 2, "op": "execute_command", "limit_ms": 30000,
 "request": {"command": "tasks.list.AddTask", "actor": "tasks.list.User", "caller": null,
             "input": {"priority": 1, "title": "title"}, "correlation": "tasks-ae9ae1…-000001"}}
```

```json
{"id": 2, "ok": {"outcome": {"command": "tasks.list.AddTask", "outcome": "added"},
                 "consistency": "1",
                 "direct_events": [{"event": "tasks.list.TaskAdded",
                                    "payload": {"task_id": "00000000-0000-4000-8000-000000000001",
                                                "title": "title"}}]}}
```

`id` is 0 for the handshake, then 1, 2, 3…, and the answer repeats it. `op` names the operation,
`request` holds its members, and `limit_ms` is how many milliseconds `ess` waits for this answer.
An answer holds exactly one of `ok`, the result (`null` for an operation that returns nothing), and
`target_error`. Requests are open: ignore a member you do not know. Answers are closed: an unknown
member, a missing required one or a wrong type is a fault.

The first request is the handshake. It names the versions this `ess` reads and the suite; the
answer names the one version the adapter speaks and the implementation, which the report names:

```json
{"id": 0, "op": "handshake", "limit_ms": 30000,
 "request": {"reads": ["ess-conformance-target/1"],
             "suite": {"system": "tasks", "specification_version": "v1",
                       "suite_version": "ess-conformance/35", "spec_digest": "ae9ae1…",
                       "suite_digest": "sha256:97f31b…"}}}
```

```json
{"id": 0, "ok": {"speaks": "ess-conformance-target/1",
                 "implementation": {"name": "tasks-py", "version": "1.0.0"}}}
```

Besides the handshake, seven operations are required: `begin_scenario`, `execute_command`,
`query_view`, `observe_events`, `configure_external_outcome`, `redeliver_event` and `end_scenario`.
A suite sends only the ones its specification needs; the `tasks` suite sends `begin_scenario`,
`execute_command`, `query_view` and `end_scenario`. Answer every operation you do not implement,
required or optional, and an `op` you have never heard of, `unsupported` with a reason:

```json
{"id": 9, "target_error": {"kind": "unsupported", "observation": "observe_invocations",
                           "why": "this adapter does not implement observe_invocations"}}
```

Each scenario that reaches it is reported `unsupported` with that reason, which fails the run.
`query_view_as` and `query_view_anonymous` answered `unsupported` are sent again as `query_view`,
and `scan_view_as` and `scan_view_anonymous` as `scan_view`, so an adapter that checks no read
grant fails the suite's `<view>/grant/read/denied` scenarios.

A refusal the specification declares is an ordinary `ok` answer, with the refusing outcome and its
error:

```json
{"id": 7, "ok": {"outcome": {"command": "tasks.list.AddTask", "outcome": "rejected"},
                 "error": {"error": "tasks.list.InvalidPriority", "fields": {"submitted": -1}},
                 "consistency": "1", "direct_events": []}}
```

A command or read the actor is not granted is answered under `target_error` with
`{"kind": "not_granted", "actor": …}`, naming the refused actor, or `null` for a read sent as no
actor.

A fault is an answer `ess` cannot take: a line that is not JSON or does not match its operation,
another `id`, a number `ess` cannot hold exactly, no answer within `limit_ms`, or the adapter
exiting or closing its standard output before it answers. The scenario ends `error`, `ess` kills
the adapter's whole process group, and the next scenario starts a fresh process, which must answer
the handshake with the same implementation. So the adapter must be restartable at any point, and it
must exit when its standard input closes: that is how `ess` ends it after the last scenario, and
`ess` kills the process group 5 seconds later if it has not exited.

`begin_scenario` and `end_scenario` bracket each scenario, with the scenario's id and its
`correlation`, which every request of that scenario carries. Each scenario starts from the empty
state the run prints (`Requires an empty logical modeled-instance/event/invocation namespace…`):
reset or partition your implementation's state in `begin_scenario`, so nothing one scenario did
satisfies another, and release it in `end_scenario`.

Numbers travel by exact value: `ess` writes each number as the token of its exact value, and a
number in an answer that it cannot hold exactly ends the scenario `error`, never a rounded pass. If
your language parses JSON numbers as binary64, read them as decimals (Python:
`json.loads(line, parse_float=decimal.Decimal)`) and write each as its exact token.

`ess verify conform serve --target billing` is a reference peer: it answers these messages for the
built-in `billing` implementation on its standard input and output until input closes. Pipe request
lines into it to see the answers a correct target gives, or, from a checkout of this repository,
run the billing suite through it:

```shell-session
$ ess verify conform run --path examples/billing --target process --report-format 2 \
    -- ess verify conform serve --target billing
…
  32 scenarios: 32 passed, 0 failed, 0 error, 0 unsupported
```

Every operation and member is listed in [the protocol's design
page](https://github.com/beyond10x/ess/blob/main/docs/design/conformance-target-process.md#message-catalogue).

### A complete adapter

This adapter serves the `tasks` specification and holds the tasks in memory, so here it is itself
the implementation under test; a real adapter calls your service, command or library in each
method instead. It answers every operation it does not define `unsupported`, returns a consistency
token from every command because the `Tasks` view is `read_your_writes`, and mints identities from
a counter, so two runs write the same report.

```python title="adapter/adapter.py"
#!/usr/bin/env python3
"""An ess-conformance-target/1 adapter for the `tasks` specification, held in memory."""
import json
import sys
import uuid

SPEAKS = "ess-conformance-target/1"
L = "tasks.list."


class Unsupported(Exception):
    pass


class Tasks:
    def handshake(self, request):
        return {"speaks": SPEAKS, "implementation": {"name": "tasks-py", "version": "1.0.0"}}

    def begin_scenario(self, request):
        self.tasks, self.events, self.writes = {}, [], 0

    def end_scenario(self, request):
        self.tasks, self.events = {}, []

    def execute_command(self, request):
        command, values = request["command"], request["input"]
        self.writes += 1
        result = {"consistency": str(self.writes), "direct_events": []}

        def took(outcome, error=None, fields=None, event=None):
            result["outcome"] = {"command": command, "outcome": outcome}
            if error:
                result["error"] = {"error": L + error, "fields": fields}
            if event:
                published = {"event": L + event[0], "payload": event[1]}
                result["direct_events"].append(published)
                self.events.append(published)
            return result

        if command == L + "AddTask":
            if values["priority"] < 0:
                return took("rejected", "InvalidPriority", {"submitted": values["priority"]})
            task_id = str(uuid.UUID(int=len(self.tasks) + 1, version=4))
            self.tasks[task_id] = {"task_id": task_id, "title": values["title"],
                                   "priority": values["priority"], "state": "Open"}
            return took("added", event=("TaskAdded", {"task_id": task_id,
                                                      "title": values["title"]}))
        if command == L + "CompleteTask":
            task = self.tasks.get(values["task_id"])
            if task is None:  # `wrong_state: true` also covers a task that does not exist
                return took("already-done", "AlreadyDone", {})
            if task["state"] == "Done":
                return took("already-done", "AlreadyDone", {"state": "Done"})
            task["state"] = "Done"
            return took("completed", event=("TaskCompleted", {"task_id": task["task_id"]}))
        raise Unsupported("this adapter does not serve " + command)

    def query_view(self, request):
        if request["view"] != L + "Tasks":
            raise Unsupported("this adapter does not serve " + request["view"])
        return {"rows": sorted(self.tasks.values(), key=lambda row: row["task_id"])}

    def observe_events(self, request):
        return [event for event in self.events if event["event"] == request["event"]]


def main():
    target = Tasks()
    for line in sys.stdin:  # ends when ess closes standard input
        message = json.loads(line)
        answer = {"id": message["id"]}
        method = getattr(target, message["op"], None)
        try:
            if method is None:
                raise Unsupported("this adapter does not implement " + message["op"])
            answer["ok"] = method(message["request"])
        except Unsupported as why:
            answer["target_error"] = {"kind": "unsupported", "observation": message["op"],
                                      "why": str(why)}
        print(json.dumps(answer), flush=True)


if __name__ == "__main__":
    main()
```

Change `took("rejected"` to `took("added"` and run again: `tasks.list.AddTask/outcome/rejected`
fails and the run exits 1.

## A target in Rust

The generated Go and TypeScript packages [above](#hold-your-own-implementation-to-the-suite) and the
[process target](#an-implementation-in-any-language) are the routes for an implementation outside
this repository. A Rust implementation can instead implement
the `ConformanceTarget` trait of the `ess-conformance` crate, which is what the built-in targets do.
It must preserve scenario identity, request/response correlation and refusal semantics, and it must
not report its own unobserved success.

A suite for a specification whose actors' `may:` names a view sends reads of that view as an actor,
after a `read_as` step (beyond10x/ess#286), and also as no actor after `read_as` with
`actor: null`. The runner calls `query_view_as` with the actor, or `query_view_anonymous` for no
actor, and a target answers a read the grant does not admit with `TargetError::NotGranted` naming
the actor, or none, before reading anything. The defaults read as `query_view` does, so a target
that checks no read grant fails the suite's `<view>/grant/read/denied` scenarios. A read the
scenario needed answered and the target refused is `failed`. The generated Go and TypeScript
runtimes send the actor as `ViewRequest.Actor` and `actor`, set `Anonymous` and `anonymous` for a
read sent as no actor, and read a refusal from `NotGranted` and `notGranted` on the view result; a
target that ignores them serves the read and fails the same scenarios.

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

Under report/2, generated Go and TypeScript show each unsupported required scenario as a skipped
subtest or case carrying the target's reason, so a JUnit or test report does not count it as a
failed assertion. The run still fails once: the enclosing test fails with one message naming how
many required scenarios were unsupported, and the process exits nonzero. The report still counts
those scenarios `unsupported` with `execution_status` `failed`. A held scenario the target cannot
hold `now` for is skipped without failing the run. Report/1 is unchanged.

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

`conform web` emits a browser product: the original specification files, the admitted suite
(`suite.json`) or, with `--suite-format 5`, the complete coverage input (`input.json`), a
Rust-derived `declarations.json` and a `browser.json` manifest binding them by digest. The page
navigates every declaration without executing anything. Building the emitted `rust/browser_host.rs`
with your own target installation (see the emitted `README.md`) gives `runner.wasm`, which re-admits
the original bytes before any target call and runs the Rust runner; only those runs produce
reports. The historical `ess-conformance-replay/1` reader stays available from the library
(`web::emit_input`) and is no longer what this command writes.

`ess verify impact --suite-input` accepts complete admitted coverage and reports its selection separately.
Unknown or incomplete inventory and missing parents refuse. Persisted output remains `ess-impact/3`
with its existing fields; invalidation within a selection is not whole-system execution evidence.

## What the report proves

The report proves what its scenarios observed against the named target and specification digest. It
does not prove that the target is independently operated, deployed in production, or free from
behavior the suite never exercised. A consumer may translate this report into its own evidence
vocabulary at that consumer’s boundary; ESS itself publishes no workflow or planning record.
