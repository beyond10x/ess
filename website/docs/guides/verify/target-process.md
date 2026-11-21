---
title: An implementation in any language
sidebar_position: 4
description: "Hold a program written in any language to a suite: ess starts your adapter, asks it the suite's questions as JSON lines on its standard input and output, and judges the answers itself."
---

# An implementation in any language

`ess verify conform run --target process` holds an implementation written in any language to its
suite. `ess` starts a program you write, the adapter, and asks it the suite's questions as one JSON
object per line on its standard input; the adapter answers each with one JSON line on its standard
output, by driving your implementation. `ess` judges the answers itself and writes the report a
built-in target gets: the adapter reports what it observed, never a verdict.

## Run it

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

## The messages

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
`target_error`. Requests are open at every depth: ignore a member you do not know, wherever it sits
in the request. Answers are closed: an unknown member, a missing required one, a member name
repeated in one object or a wrong type is a fault.

The first request is the handshake. It names the versions this `ess` reads and the suite; the
answer names the one version the adapter speaks and the implementation, which the report names.
`name` and `version` are each at most 256 bytes, and a longer one fails the handshake. Where either
holds a control, line-separator or bidirectional-override character, the report shows it as an
escape such as `\u{1b}`, and it shows a backslash doubled:

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

Each scenario that reaches it is reported `unsupported` with that reason, which fails the run. Every
string of a `target_error` is cut at 4096 bytes, ending ` … (<n> bytes more)`, and shown with the
same escapes. `query_view_as` and `query_view_anonymous` answered `unsupported` are sent again as `query_view`,
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

A fault costs the scenario's wait: an adapter that never answers holds each scenario it is asked in
for `--target-timeout`, and the next scenario's restart handshake too, so a run in which every
scenario times out takes about scenarios × `--target-timeout`. Lower the option for a quicker
feedback loop while you build the adapter.

If `ess` receives SIGINT, SIGTERM or SIGHUP during a run, it kills the adapter's process group and
then ends by that signal, writing no report, so a shell reports 128 plus the signal number (130,
143 or 129) and a Ctrl-C still stops a shell loop over runs. SIGKILL
cannot be handled; an adapter whose `ess` was killed that way, or crashed, ends only because its
standard input closes.

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

## A complete adapter

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
