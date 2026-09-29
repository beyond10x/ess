---
title: Getting started
sidebar_position: 2
description: Install ess, write a one-file specification, generate a contract from it, and hold a small implementation to the conformance suite it obliges.
---

# Getting started

This page takes you from an installed `ess` to three things: a specification you wrote, an OpenAPI
document generated from it, and a passing conformance run of an implementation against the suite
the specification obliges. The second half walks through the larger example in the repository.

## Install the command

The preferred installation is a verified release archive. Each release publishes the `ess` binary
for four native targets:

| Machine | Target |
|---|---|
| Linux x86-64 | `x86_64-unknown-linux-gnu` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` |
| macOS Intel | `x86_64-apple-darwin` |
| macOS Apple Silicon | `aarch64-apple-darwin` |

Choose the target for your machine. This example downloads the latest published release, `0.43.0`,
for Apple Silicon, verifies the archive before extracting it, and runs the binary in place:

```shell-session
$ version=0.43.0
$ target=aarch64-apple-darwin
$ archive="ess-${version}-${target}.tar.gz"
$ base="https://github.com/beyond10x/ess/releases/download/${version}"
$ curl --fail --location --remote-name "${base}/${archive}"
$ curl --fail --location --remote-name "${base}/SHA256SUMS"
$ grep -F "  ${archive}" SHA256SUMS | shasum -a 256 --check
ess-0.43.0-aarch64-apple-darwin.tar.gz: OK
$ tar -xzf "${archive}"
$ "./ess-${version}-${target}/ess" --version
ess 0.43.0
```

`SHA256SUMS` covers all four archives. Filtering the exact filename lets the checksum tool verify
the one archive you downloaded without treating the other three as missing files. Put the extracted
`ess` on your `PATH`; the rest of this page calls it `ess`.

If your machine is not in the release matrix, or you need to work from current `main`, build the
locked Rust workspace from a source checkout:

```shell-session
$ cargo build --locked --release --bin ess
$ ./target/release/ess --version
ess 0.43.0
```

The first level of `ess` is four areas: `specify` (write and resolve a specification), `generate`
(turn it into artifacts), `verify` (hold an implementation or a later revision to it) and `infra`
(read an observed cluster). `ess <area> --help` lists what each holds, and the
[CLI reference](./reference/cli.md) describes every command.

## Write a specification

Make a directory for the project and write one file, `spec/system.yaml`. It describes a list of
tasks: a task is added with a title and a priority, and can be completed once.

```shell-session
$ mkdir -p tasks/spec && cd tasks
```

```yaml title="spec/system.yaml"
format: ess/1
system: tasks
version: v1
summary: A list of tasks that can be completed.
domains: [tasks.list]
domain: tasks.list

types:
  - {name: tasks.list.TaskId, kind: newtype, of: Uuid}
  - {name: tasks.list.Title, kind: newtype, of: String}

entities:
  - name: tasks.list.Task
    identity: {name: task_id, type: tasks.list.TaskId}
    fields:
      - {name: title, type: tasks.list.Title}
      - {name: priority, type: Integer}
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions:
        - {name: complete, from: [Open], to: Done}

actors:
  - name: tasks.list.User
    may: [tasks.list.AddTask, tasks.list.CompleteTask]

errors:
  - name: tasks.list.InvalidPriority
    summary: The priority is below zero.
    fields:
      - {name: submitted, type: Integer}
  - name: tasks.list.AlreadyDone
    summary: The task is already done.
    fields:
      - {name: state, type: tasks.list.Task.State}

commands:
  - name: tasks.list.AddTask
    input:
      - {name: title, type: tasks.list.Title}
      - {name: priority, type: Integer}
    outcomes:
      - name: added
        when: priority >= 0
        creates: tasks.list.Task
        instance: task_id
        sets: {title: input.title, priority: input.priority}
        emits: [tasks.list.TaskAdded]
        payload:
          tasks.list.TaskAdded: {title: input.title}
      - name: rejected
        error: tasks.list.InvalidPriority

  - name: tasks.list.CompleteTask
    input:
      - {name: task_id, type: tasks.list.TaskId}
    outcomes:
      - name: completed
        moves: tasks.list.Task.complete
        instance: task_id
        emits: [tasks.list.TaskCompleted]
        payload:
          tasks.list.TaskCompleted: {task_id: input.task_id}
      - name: already-done
        wrong_state: true
        error: tasks.list.AlreadyDone

events:
  - name: tasks.list.TaskAdded
    fields:
      - {name: task_id, type: tasks.list.TaskId}
      - {name: title, type: tasks.list.Title}
  - name: tasks.list.TaskCompleted
    fields:
      - {name: task_id, type: tasks.list.TaskId}

views:
  - name: tasks.list.OpenTasks
    source: tasks.list.Task
    consistency: read_your_writes
    filter: state == Open
    fields:
      - {name: task_id, type: tasks.list.TaskId}
      - {name: title, type: tasks.list.Title}
      - {name: priority, type: Integer}

components:
  - component: task-service
    owns: {domains: [tasks.list]}
    accepts: {commands: [tasks.list.AddTask, tasks.list.CompleteTask]}
    publishes: {events: [tasks.list.TaskAdded, tasks.list.TaskCompleted]}
    reached_by: network
```

A few things in it are required rather than stylistic:

- `AddTask` can be refused, so it declares the refusal as an outcome (`rejected`). The guard
  `when: priority >= 0` says which input takes which branch.
- `CompleteTask` declares what it answers when the task is not `Open` (`wrong_state: true`). The
  lifecycle has no arrow out of `Done`, so completing a task twice is illegal without a second rule
  saying so.
- `sets:` says what the new task holds, and `payload:` says what the event carries. Without them a
  generated test could find the task and say nothing about its contents. `task_id` has no line in
  either: the implementation assigns it, so the suite checks its presence and type only.

Validate it. Validation resolves every name and reports every problem in one run:

```shell-session
$ ess specify validate --path spec
tasks v1 — 1 file(s), valid
```

Misspell `tasks.list.TaskAdded` in the `emits:` list and run it again to see what a refusal looks
like: the command exits `1`, and each problem names its place in the model, the file and line, and a
hint. The
[specification guide](./guides/write-a-specification.md) covers the rest of the language.

## Generate a contract

```shell-session
$ ess generate --path spec --kind openapi --out generated
openapi/task-service.yaml — 10736 byte(s)
1 artifact(s), written to generated
```

The OpenAPI document is a projection of the validated model, one per component. Regenerating after a
change to the specification replaces it; it is never edited by hand. `--kind` also accepts `docs`,
`site`, `schema` and `asyncapi`, and omitting it writes all five. See
[Generate contracts and documentation](./guides/generate-artifacts.md).

## Hold an implementation to the specification

The specification obliges a conformance suite: one scenario for each outcome, each lifecycle move
and each move that must be refused. `ess` writes that suite together with a runner, as a Go or a
TypeScript test package. This example uses TypeScript and needs Node.js 20 or later:

```shell-session
$ ess verify conform synthesize --path spec --target typescript --out conformance
6 scenario(s) (0 authored), 0 refusal(s), 13 file(s) written to conformance
$ cd conformance/essconform
$ npm install
```

The package asks the implementation questions through one interface, `Target`: run this command,
read this view. Write a test file that answers them. The implementation below keeps tasks in a
`Map`; a real target would call your service instead.

```ts title="conformance/essconform/src/conformance.test.ts"
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
      if (request.view !== "tasks.list.OpenTasks") throw ErrUnsupported;
      const rows = [...tasks.values()]
        .filter((task) => task.state === "Open")
        .map(({ task_id, title, priority }) => ({ task_id, title, priority }));
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

Run it, asking for a report file:

```shell-session
$ ESS_REPORT_OUT=$PWD/report.json npm test
…
    ok 1 - tasks.list.AddTask/outcome/added
    ok 2 - tasks.list.AddTask/outcome/rejected
    ok 3 - tasks.list.CompleteTask/outcome/already-done
    ok 4 - tasks.list.CompleteTask/outcome/completed
    ok 5 - tasks.list.Task/state/Done/refuses/tasks.list.CompleteTask
    ok 6 - tasks.list.Task/transition/complete/by/tasks.list.CompleteTask/completed
ok 1 - conformance
# report: passed, 6 scenario(s), 0 not passed, written to …/report.json
```

`report.json` is an `ess-conformance-report/1` naming the specification, its digest, the
implementation and the result. A skipped scenario, from a method that threw `ErrUnsupported`, makes
the report `inconclusive` rather than `passed`.

To see that the suite tests something, break the implementation. Leave the task `Open` in
`CompleteTask` (`task.state = "Open"`) and run `npm test` again: three scenarios fail, one of them
with ``step 6: `tasks.list.CompleteTask` took `completed`, and the specification says
`already-done` ``.

Regenerating the package after a change to the specification keeps `src/conformance.test.ts`, which
`ess` did not write. [Verify conformance](./guides/verify-conformance.md#hold-your-own-implementation-to-the-suite)
describes the whole `Target` interface and the Go package.

## Explore the repository example

The repository's `examples/billing/` is a larger specification: two domains, cross-domain bindings,
both view consistencies and a type of every kind. From a checkout of the repository, the commands
below run against it. Replace `ess` with `cargo run --quiet --locked --bin ess --` to use the
checkout's own build.

```shell-session
$ ess specify validate --path examples/billing
billing v3 — 5 file(s), valid

$ ess specify compile --path examples/billing --out target/billing.ir.json
billing v3 — 5 file(s), 26 declaration(s), compiled to target/billing.ir.json
```

Compilation writes canonical JSON. Running it twice with the same input produces the same bytes.

Inspect one declaration, or the interaction graph. Names resolve before inspection, so an unknown or
ambiguous name is a refusal rather than an empty result:

```shell-session
$ ess specify inspect --path examples/billing billing.invoice.Invoice
$ ess specify graph --path examples/billing --format mermaid
```

Generate documentation. `docs` writes Markdown with Mermaid diagrams; `site` renders the same pages
as a browsable HTML site with its own stylesheet and diagram renderer:

```shell-session
$ ess generate --path examples/billing --kind docs --out target/projections
$ ess generate --path examples/billing --kind site --out target/site
```

The entry pages are `target/projections/docs/index.md` and `target/site/index.html`.
ESS generates documentation from the typed specification; it does not read Markdown as a
specification, and it does not host the site.

Run the billing suite against the repository's own reference implementation of billing, which is
built into `ess`:

```shell-session
$ ess verify conform synthesize --path examples/billing --out target/billing-suite.json
32 scenario(s) (0 authored), 0 refusal(s), written to target/billing-suite.json
$ ess verify conform run --suite target/billing-suite.json --target billing
…
  32 scenarios: 32 passed, 0 failed, 0 error, 0 unsupported
```

The built-in targets (`billing`, `oracle-fixture`, `interpreted`) exist to demonstrate the runner;
your own implementation is held to its suite through a generated package, as above.

## Continue

- [Write a specification](./guides/write-a-specification.md)
- [Generate artifacts](./guides/generate-artifacts.md)
- [Verify conformance](./guides/verify-conformance.md)
- [Track specification change](./guides/track-change.md)
- [Import or project infrastructure](./guides/check-infrastructure.md)
- [Use the complete CLI reference](./reference/cli.md)
