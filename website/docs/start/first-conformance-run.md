---
title: Run your first conformance suite
description: Synthesize the conformance suite a specification obliges as a TypeScript test package, hold a small implementation to it, and watch it fail when the implementation is wrong.
---

# Run your first conformance suite

This page holds an implementation to the specification from
[Write your first specification](./first-specification.md), in the same `tasks` directory. It needs
Node.js 20 or later.

## Synthesize the suite


The specification obliges a conformance suite: one scenario for each outcome, each lifecycle move
and each move that must be refused. `ess` writes that suite together with a runner, as a Go or a
TypeScript test package:

```shell-session ess-tutorial
$ ess verify conform synthesize --path . --target typescript --out conformance
note: every declared actor may invoke `tasks.list.AddTask`, so no actor is refused it and no `tasks.list.AddTask/grant/denied` scenario is owed
note: every declared actor may invoke `tasks.list.CompleteTask`, so no actor is refused it and no `tasks.list.CompleteTask/grant/denied` scenario is owed
6 scenario(s) (0 authored), 0 refusal(s), 15 file(s) written to conformance
```

## Implement Target

The package asks the implementation questions through one interface, `Target`: run this command,
read this view. Write a test file that answers them. The implementation below keeps tasks in a
`Map`; a real target would call your service instead.

```ts ess-tutorial file=tasks/conformance/essconform/src/conformance.test.ts title="conformance/essconform/src/conformance.test.ts"
import { test } from "node:test";
import { randomUUID } from "node:crypto";
import { ErrUnsupported, run } from "./index.js";
import type { CommandRequest, CommandResult, Target, ViewRequest, ViewResult } from "./index.js";

type Task = { task_id: string; title: string; priority: unknown; state: "Open" | "Done" };

// An in-memory implementation of the tasks specification. A real target calls your service here.
function newTarget(): Target {
  const tasks = new Map<string, Task>();
  return {
    identity: () => ({ name: "tasks-in-memory", version: "dev" }),
    beginScenario: () => {},
    endScenario: () => {},

    executeCommand(request: CommandRequest): CommandResult {
      const input = request.input;
      switch (request.command) {
        case "tasks.list.AddTask": {
          if (Number(String(input.priority)) < 0) {
            return { outcome: "rejected", error: "tasks.list.InvalidPriority" };
          }
          const task_id = randomUUID();
          tasks.set(task_id, { task_id, title: input.title, priority: input.priority, state: "Open" });
          return {
            outcome: "added",
            directEvents: [
              { event: "tasks.list.TaskAdded", payload: { task_id, title: input.title } },
            ],
          };
        }
        case "tasks.list.CompleteTask": {
          const task = tasks.get(input.task_id);
          if (task === undefined || task.state !== "Open") {
            return { outcome: "already-done", error: "tasks.list.AlreadyDone" };
          }
          task.state = "Done";
          return {
            outcome: "completed",
            directEvents: [
              { event: "tasks.list.TaskCompleted", payload: { task_id: task.task_id } },
            ],
          };
        }
        default:
          throw ErrUnsupported;
      }
    },

    queryView(request: ViewRequest): ViewResult {
      if (request.view !== "tasks.list.Tasks") throw ErrUnsupported;
      const rows = [...tasks.values()].map(({ task_id, title, priority, state }) => ({
        task_id,
        title,
        priority,
        state,
      }));
      return { rows };
    },

    observeEvents: () => [],
    configureExternalOutcome: () => {
      throw ErrUnsupported;
    },
    redeliverEvent: () => {
      throw ErrUnsupported;
    },
    observeInvocations: () => {
      throw ErrUnsupported;
    },
  };
}

await test("conformance", async (t) => {
  await run(t, (): Target => newTarget());
});
```

## Run it

Install the package's dependencies and run it, asking for a report file. This needs Node.js 20 or
later. `ESS_REPORT_FORMAT=2` selects the report format this suite requires; without it the runner
stops before the first scenario and says so.

```shell-session ess-tutorial requires=node
$ cd conformance/essconform
$ npm install
…
$ ESS_REPORT_FORMAT=2 ESS_REPORT_OUT=report.json npm test
…
    ok 1 - tasks.list.AddTask/outcome/added
…
    ok 2 - tasks.list.AddTask/outcome/rejected
…
    ok 3 - tasks.list.CompleteTask/outcome/already-done
…
    ok 4 - tasks.list.CompleteTask/outcome/completed
…
    ok 5 - tasks.list.Task/state/Done/refuses/tasks.list.CompleteTask
…
    ok 6 - tasks.list.Task/transition/complete/by/tasks.list.CompleteTask/completed
…
ok 1 - conformance
…
# tasks v1, 6 scenario(s), spec digest ae9ae1d659481748f19a00a1451b0a656fa7abec1474b3c51ff1dc1dd53f0919
# report/2: passed, written to report.json
…
```

`report.json` is an `ess-conformance-report/2` naming the specification, the implementation, and
the count of passed, failed, error, skipped and unsupported scenarios. Its `execution_status` is
`passed`. Its `conformance_status` is `inconclusive`, because this run states nothing about
coverage: [Verify conformance](../guides/verify-conformance.md) describes the stricter settings.

## Break it

To see that the suite tests something, break the implementation. Leave the task `Open` in
`CompleteTask` (`task.state = "Open"`) and run `npm test` again: three scenarios fail, one of them
with ``step 8: `tasks.list.CompleteTask` took `completed`, and the specification says
`already-done` ``.

Regenerating the package after a change to the specification keeps `src/conformance.test.ts`, which
`ess` did not write.

## Next

- [The TypeScript runner](./runners/typescript.md): every file the package holds and the whole
  `Target` interface.
- [The Go runner](./runners/go.md) and [the Rust runner](./runners/rust.md): the same suite for an
  implementation in Go or Rust.
- [Explore the repository example](./explore-the-example.md): a larger specification, its
  documentation and its reference implementation.
- [Verify conformance](../guides/verify-conformance.md): authored scenarios, coverage and the
  stricter report settings.
