---
title: The Go runner
description: The files ess writes for a Go conformance package, the Target interface an implementation provides, and a complete in-memory implementation run with go test.
---

# The Go runner

`ess verify conform synthesize --target go` writes the suite a specification obliges as a Go
package that `go test` runs. This page holds a small Go implementation of the `tasks` specification
from [Write your first specification](../first-specification.md) to it. The generated package
imports the standard library only, so it adds nothing to your `go.mod`.

## What ess writes

The package goes inside your module. Here the module is a new directory, `go`, in the `tasks`
project:

```shell-session ess-tutorial
$ cd ~/tasks
$ mkdir -p go
```

```text ess-tutorial file=tasks/go/go.mod title="go/go.mod"
module example.com/tasks

go 1.22
```

```shell-session ess-tutorial
$ ess verify conform synthesize --path . --target go --out go
note: every declared actor may invoke `tasks.list.AddTask`, so no actor is refused it and no `tasks.list.AddTask/grant/denied` scenario is owed
note: every declared actor may invoke `tasks.list.CompleteTask`, so no actor is refused it and no `tasks.list.CompleteTask/grant/denied` scenario is owed
6 scenario(s) (0 authored), 0 refusal(s), 7 file(s) written to go
```

The package is `essconform`, one directory below `--out`, so this module imports it as
`example.com/tasks/essconform`:

```text ess-tutorial files=tasks/go/essconform
README.md
explore.go
ir.json
predicate.go
runtime.go
suite.go
suite.json
```

| File | What it is |
|---|---|
| `suite.json` | the suite: one scenario per outcome, lifecycle move and refusal |
| `suite.go` | embeds `suite.json`, so the package needs no working directory to find it |
| `ir.json` | the compiled specification, which the explorer interprets as its reference model |
| `runtime.go` | the runner: `Run`, the `Target` interface and its request and result types |
| `predicate.go` | the three-valued evaluator the runner checks guards and invariants with |
| `explore.go` | `Explore` and `ExploreConcurrent`, seeded random command sequences |
| `README.md` | the wiring for this suite, and the report format it requires |

Beside the package, `go/.ess-output/state.json` records which files `ess` wrote there. Nothing in
`essconform` is edited by hand; regenerate it after every change to the specification.

## What your implementation provides

A value satisfying `Target`, exactly as `runtime.go` declares it:

```go ess-tutorial interface=tasks/go/essconform/runtime.go
type Target interface {
	Identity() (Identity, error)
	BeginScenario(scenario ScenarioContext) error
	EndScenario(scenario ScenarioContext) error
	ExecuteCommand(request CommandRequest) (CommandResult, error)
	QueryView(request ViewRequest) (ViewResult, error)
	ObserveEvents(request EventObservationRequest) ([]ObservedEvent, error)
	ConfigureExternalOutcome(control ExternalOutcomeControl) error
	RedeliverEvent(request RedeliveryRequest) error
	ObserveInvocations(request InvocationObservationRequest) ([]Invocation, error)
}
```

The methods answer the same questions as the TypeScript runner's; its
[method table](./typescript.md#what-your-implementation-provides) says what each one answers. A
method the implementation cannot answer returns `essconform.ErrUnsupported`, and the scenario is
reported as `unsupported` in report/2; the default strict run fails. Ordinary target errors remain
`error` and make execution inconclusive. Specifications that declare more ask for optional interfaces a target may
also implement: `EntitySetupTarget`, `AbsentInputTarget`, `RepeatedOutcomeTarget`,
`PeriodicTarget`, `ClockReadingTarget` and `InterleavedTarget`. The generated `README.md` names
the ones its suite needs.

This implementation keeps tasks in a map; a real target calls your service instead:

```go ess-tutorial file=tasks/go/conformance_test.go title="go/conformance_test.go"
package tasks_test

import (
	"crypto/rand"
	"fmt"
	"strconv"
	"testing"

	"example.com/tasks/essconform"
)

type task struct {
	id       string
	title    essconform.Node
	priority essconform.Node
	state    string
}

// memory is an in-memory implementation of the tasks specification. A real target calls your
// service here.
type memory struct {
	tasks  map[string]*task
	order  []string
	writes int
}

func newTarget() *memory { return &memory{tasks: map[string]*task{}} }

func newID() string {
	b := make([]byte, 16)
	rand.Read(b)
	b[6] = b[6]&0x0f | 0x40
	b[8] = b[8]&0x3f | 0x80
	return fmt.Sprintf("%x-%x-%x-%x-%x", b[0:4], b[4:6], b[6:8], b[8:10], b[10:])
}

func (m *memory) Identity() (essconform.Identity, error) {
	return essconform.Identity{Name: "tasks-in-memory", Version: "dev"}, nil
}

func (m *memory) BeginScenario(essconform.ScenarioContext) error { return nil }
func (m *memory) EndScenario(essconform.ScenarioContext) error   { return nil }

func (m *memory) ExecuteCommand(r essconform.CommandRequest) (essconform.CommandResult, error) {
	// Every answer names the write a read_your_writes read of tasks.list.Tasks is made no older than.
	m.writes++
	token := strconv.Itoa(m.writes)
	switch r.Command {
	case "tasks.list.AddTask":
		priority, err := strconv.ParseFloat(fmt.Sprint(r.Input["priority"]), 64)
		if err != nil || priority < 0 {
			return essconform.CommandResult{Outcome: "rejected", Error: "tasks.list.InvalidPriority", Consistency: token}, nil
		}
		t := &task{id: newID(), title: r.Input["title"], priority: r.Input["priority"], state: "Open"}
		m.tasks[t.id] = t
		m.order = append(m.order, t.id)
		return essconform.CommandResult{
			Outcome:     "added",
			Consistency: token,
			DirectEvents: []essconform.ObservedEvent{{
				Event:   "tasks.list.TaskAdded",
				Payload: map[string]essconform.Node{"task_id": t.id, "title": t.title},
			}},
		}, nil
	case "tasks.list.CompleteTask":
		t, ok := m.tasks[fmt.Sprint(r.Input["task_id"])]
		if !ok || t.state != "Open" {
			return essconform.CommandResult{Outcome: "already-done", Error: "tasks.list.AlreadyDone", Consistency: token}, nil
		}
		t.state = "Done"
		return essconform.CommandResult{
			Outcome:     "completed",
			Consistency: token,
			DirectEvents: []essconform.ObservedEvent{{
				Event:   "tasks.list.TaskCompleted",
				Payload: map[string]essconform.Node{"task_id": t.id},
			}},
		}, nil
	}
	return essconform.CommandResult{}, essconform.ErrUnsupported
}

func (m *memory) QueryView(r essconform.ViewRequest) (essconform.ViewResult, error) {
	if r.View != "tasks.list.Tasks" {
		return essconform.ViewResult{}, essconform.ErrUnsupported
	}
	rows := []essconform.Row{}
	for _, id := range m.order {
		t := m.tasks[id]
		rows = append(rows, essconform.Row{
			"task_id": t.id, "title": t.title, "priority": t.priority, "state": t.state,
		})
	}
	return essconform.ViewResult{Rows: rows}, nil
}

func (m *memory) ObserveEvents(essconform.EventObservationRequest) ([]essconform.ObservedEvent, error) {
	return nil, nil
}

func (m *memory) ConfigureExternalOutcome(essconform.ExternalOutcomeControl) error {
	return essconform.ErrUnsupported
}

func (m *memory) RedeliverEvent(essconform.RedeliveryRequest) error {
	return essconform.ErrUnsupported
}

func (m *memory) ObserveInvocations(essconform.InvocationObservationRequest) ([]essconform.Invocation, error) {
	return nil, essconform.ErrUnsupported
}

func TestConformance(t *testing.T) {
	essconform.Run(t, func() essconform.Target { return newTarget() })
}
```

`Run` builds one target per scenario, so no scenario sees another's tasks. `tasks.list.Tasks` is
`read_your_writes`, so every command answer carries a `Consistency` token, refusals included, and
the runner reads the view no older than it; an answer without one fails the view's next check.

## Run it

Set `ESS_REPORT_FORMAT=2`, which this suite's format requires; without it `Run` stops before the
first scenario and says so. `ESS_REPORT_OUT` names a file for the report:

```shell-session ess-tutorial requires=go
$ cd go
$ ESS_REPORT_FORMAT=2 ESS_REPORT_OUT=report.json go test -count=1 -v ./...
=== RUN   TestConformance
…
=== RUN   TestConformance/tasks.list.AddTask/outcome/added
…
=== RUN   TestConformance/tasks.list.AddTask/outcome/rejected
…
=== RUN   TestConformance/tasks.list.CompleteTask/outcome/already-done
…
=== RUN   TestConformance/tasks.list.CompleteTask/outcome/completed
…
=== RUN   TestConformance/tasks.list.Task/state/Done/refuses/tasks.list.CompleteTask
…
=== RUN   TestConformance/tasks.list.Task/transition/complete/by/tasks.list.CompleteTask/completed
…
PASS
…
```

Every scenario passes, and `go/report.json` is the same `ess-conformance-report/2` the TypeScript
run writes. [Verify conformance](../../guides/verify/runners.md#hold-your-own-implementation-to-the-suite)
covers authored scenarios, coverage and the report in full.
