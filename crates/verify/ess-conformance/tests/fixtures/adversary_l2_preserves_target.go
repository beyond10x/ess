// An in-memory implementation of `adversary-l2-preserves.yaml` (beyond10x/ess#536, adversary pass 1).
//
// `adversary-l2-preserves-target.mjs` is the same target in TypeScript; its header lists the modes.
// TestAdversaryL2Plan writes the plan each mode of exploration draws from, sequential and
// concurrent, so `tests/adversary_l2_pass1.rs` can read which commands each leaves out and why.

package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"testing"
)

type advL2Unsupported struct{ reason string }

func (e advL2Unsupported) Error() string { return e.reason }
func (e advL2Unsupported) Unwrap() error { return ErrUnsupported }

type advL2Doc struct {
	id    string
	state string
	title Node
}

type advL2Target struct {
	mode    string
	docs    []*advL2Doc
	version int
}

func newAdvL2Target(mode string) *advL2Target { return &advL2Target{mode: mode} }

func (t *advL2Target) reset() { t.docs = nil; t.version = 0 }

func (t *advL2Target) find(id Node) *advL2Doc {
	for _, doc := range t.docs {
		if doc.id == id {
			return doc
		}
	}
	return nil
}

func (t *advL2Target) answered(outcome, event, id string) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: map[string]Node{"doc_id": id}}},
	}
}

func (t *advL2Target) kept(outcome string) CommandResult {
	t.version++
	return CommandResult{Outcome: outcome, Consistency: fmt.Sprintf("v%d", t.version), DirectEvents: []ObservedEvent{}}
}

func advL2Refused(outcome, err string) CommandResult {
	return CommandResult{Outcome: outcome, Error: err, DirectEvents: []ObservedEvent{}}
}

func (t *advL2Target) Identity() (Identity, error) {
	return Identity{Name: "adversary-l2-preserves-target", Version: "1"}, nil
}

func (t *advL2Target) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *advL2Target) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *advL2Target) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	doc := t.find(input["doc_id"])
	if request.Command != "advkeep.docs.Create" && doc == nil {
		return CommandResult{}, fmt.Errorf("no document %v", input["doc_id"])
	}
	switch request.Command {
	case "advkeep.docs.Create":
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", len(t.docs)+1)
		t.docs = append(t.docs, &advL2Doc{id: id, state: "Draft", title: input["title"]})
		return t.answered("created", "advkeep.docs.Created", id), nil
	case "advkeep.docs.Retitle":
		if equal(doc.title, input["title"]) {
			if t.mode == "same-title-writes" {
				doc.title = fmt.Sprint(input["title"]) + "!"
			}
			return t.kept("same-title"), nil
		}
		doc.title = input["title"]
		return t.answered("retitled", "advkeep.docs.Retitled", doc.id), nil
	case "advkeep.docs.Touch":
		if input["dry"] == true {
			if t.mode == "dry-run-writes" {
				doc.title = fmt.Sprint(input["title"]) + "!"
			}
			return t.kept("dry-run"), nil
		}
		doc.title = input["title"]
		return t.answered("touched", "advkeep.docs.Touched", doc.id), nil
	case "advkeep.docs.Publish":
		switch doc.state {
		case "Published":
			if t.mode == "already-archives" {
				doc.state = "Archived"
			}
			return t.kept("already-published"), nil
		case "Draft":
			doc.state = "Published"
			return t.answered("published", "advkeep.docs.Published", doc.id), nil
		default:
			return advL2Refused("not-draft", "advkeep.docs.NotDraft"), nil
		}
	case "advkeep.docs.Archive":
		if doc.state != "Published" {
			return advL2Refused("not-published", "advkeep.docs.NotPublished"), nil
		}
		doc.state = "Archived"
		return t.answered("archived", "advkeep.docs.Archived", doc.id), nil
	default:
		return CommandResult{}, advL2Unsupported{request.Command + " is not a command of adversary-l2-preserves.yaml"}
	}
}

func (t *advL2Target) QueryView(request ViewRequest) (ViewResult, error) {
	rows := []Row{}
	switch request.View {
	case "advkeep.docs.Docs":
		for _, doc := range t.docs {
			rows = append(rows, Row{"doc_id": doc.id, "state": doc.state, "title": doc.title})
		}
	case "advkeep.docs.PublishedDocs":
		for _, doc := range t.docs {
			if doc.state == "Published" {
				rows = append(rows, Row{"doc_id": doc.id, "title": doc.title})
			}
		}
	default:
		return ViewResult{}, advL2Unsupported{request.View + " is not a view of adversary-l2-preserves.yaml"}
	}
	return ViewResult{Rows: rows}, nil
}

func (t *advL2Target) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *advL2Target) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return advL2Unsupported{"no external outcomes"}
}

func (t *advL2Target) RedeliverEvent(RedeliveryRequest) error {
	return advL2Unsupported{"no redelivery"}
}

func (t *advL2Target) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, advL2Unsupported{"no bindings"}
}

func TestAdversaryL2Plan(t *testing.T) {
	ir, err := exploreLoad(exploreIR, exploreSuite)
	if err != nil {
		t.Fatal(err)
	}
	for _, mode := range []struct {
		name       string
		concurrent bool
	}{{"sequential", false}, {"concurrent", true}} {
		p := explorePlanAs(ir, mode.concurrent)
		drawn := []string{}
		for _, command := range p.commands {
			drawn = append(drawn, command.name)
		}
		bytes, err := json.Marshal(map[string]any{"excluded": p.excluded, "drawn": drawn})
		if err != nil {
			t.Fatal(err)
		}
		file := filepath.Join(os.Getenv("ESS_EXPLORE_OUT"), "plan-"+mode.name+".json")
		if err := os.WriteFile(file, bytes, 0o644); err != nil {
			t.Fatal(err)
		}
	}
}
