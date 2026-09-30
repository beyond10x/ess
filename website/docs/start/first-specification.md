---
title: Write your first specification
description: Write a one-file specification of a task list, validate it, and generate an OpenAPI contract from it.
---

# Write your first specification

This page writes a specification of a small system, validates it, and generates an OpenAPI
contract from it. It continues in the `tasks` directory that [Install ess](./install.md#pin-the-release-for-a-project)
made and pinned, whose `ess-inputs.yaml` already names `spec/system.yaml`.

## Write the specification

Make the `spec` directory:

```shell-session ess-tutorial
$ mkdir -p spec
```

Write one file, `spec/system.yaml`. It describes a list of tasks: a task is added with a title and
a priority, and can be completed once.

```yaml ess-tutorial file=tasks/spec/system.yaml title="spec/system.yaml"
format: ess/19
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
          tasks.list.TaskAdded: {task_id: {generated: true}, title: input.title}
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
  - name: tasks.list.Tasks
    source: tasks.list.Task
    consistency: read_your_writes
    fields:
      - {name: task_id, type: tasks.list.TaskId}
      - {name: title, type: tasks.list.Title}
      - {name: priority, type: Integer}
      - {name: state, type: tasks.list.Task.State}

components:
  - component: task-service
    owns: {domains: [tasks.list]}
    accepts: {commands: [tasks.list.AddTask, tasks.list.CompleteTask]}
    publishes: {events: [tasks.list.TaskAdded, tasks.list.TaskCompleted]}
    reached_by: network
```

`format: ess/19` is the newest version of the specification language; the
[format history](../reference/spec-versions.md) lists what each version admits. A few things in the
file are required rather than stylistic:

- `AddTask` can be refused, so it declares the refusal as an outcome (`rejected`). The guard
  `when: priority >= 0` says which input takes which branch.
- `CompleteTask` declares what it answers when the task is not `Open` (`wrong_state: true`). The
  lifecycle has no arrow out of `Done`, so completing a task twice is illegal without a second rule
  saying so.
- `sets:` says what the new task holds, and `payload:` gives every field of each event a source.
  Without them a generated test could find the task and say nothing about its contents.
  `task_id: {generated: true}` says the implementation assigns the identity, so the suite checks
  its presence and type only.
- The view `tasks.list.Tasks` shows each task with its `state`. A refusal such as completing a
  `Done` task is observed through a view that projects the entity's identity and state; without
  one, synthesis refuses that scenario and says so.

## Validate it

Passing the directory makes `ess` read its `ess-inputs.yaml`. Validation resolves every name and
reports every problem in one run:

```shell-session ess-tutorial
$ ess specify validate --path .
tasks v1 — 1 file(s), valid
```

Misspell `tasks.list.TaskAdded` in the `emits:` list and run it again to see what a refusal looks
like: the command exits `1`, and each problem names its place in the model, the file and line, and a
hint. The [specification guide](../guides/write-a-specification.md) covers the rest of the language.

## Generate a contract

```shell-session ess-tutorial
$ ess generate --path . --kind openapi --out generated
openapi/task-service.yaml — 14996 byte(s)
1 artifact(s), written to generated
```

The OpenAPI document is a projection of the validated model, one per component. Regenerating after
a change to the specification replaces it; it is never edited by hand. `--kind` also accepts `docs`
(Markdown), `site` (the same pages as a browsable HTML site), `docs-ir`, `schema` and `asyncapi`;
omitting it writes all of them except `docs-ir`. See
[Generate contracts and documentation](../guides/generate-artifacts.md).

Next: [Run your first conformance suite](./first-conformance-run.md) holds an implementation to
this specification.
