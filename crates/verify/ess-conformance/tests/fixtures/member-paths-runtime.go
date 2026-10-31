package essconform

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"testing"
)

// pathsTarget returns a checkpoint whose members other than its id differ call by call.
type pathsTarget struct{ calls int }

func (*pathsTarget) Identity() (Identity, error) {
	return Identity{Name: "paths", Version: "1"}, nil
}
func (*pathsTarget) FixtureValues(ScenarioContext, FixtureContract) (map[string]Node, error) {
	return map[string]Node{}, nil
}
func (*pathsTarget) BeginScenario(ScenarioContext) error { return nil }
func (*pathsTarget) EndScenario(ScenarioContext) error   { return nil }
func (p *pathsTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	if request.Command != "relay.loop.Defer" {
		return CommandResult{}, ErrUnsupported
	}
	p.calls++
	format := fmt.Sprintf("f-%d", p.calls)
	fmt.Printf("deferred with format %s\n", format)
	return CommandResult{
		Outcome: "deferred",
		Response: map[string]Node{
			"stop": "later",
			"checkpoint": map[string]Node{
				"format":        format,
				"checkpoint_id": "cp-1",
				"call":          request.Input["call"],
			},
		},
	}, nil
}
func (*pathsTarget) QueryView(ViewRequest) (ViewResult, error) { return ViewResult{}, ErrUnsupported }
func (*pathsTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return ErrUnsupported
}
func (*pathsTarget) RedeliverEvent(RedeliveryRequest) error { return ErrUnsupported }
func (*pathsTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, ErrUnsupported
}
func (*pathsTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}
func TestMemberPaths(t *testing.T) {
	Run(t, func() Target { return &pathsTarget{} })
}

func pathsContract(t *testing.T, expected map[string]any) map[string]any {
	raw, err := os.ReadFile("contract.json")
	if err != nil {
		t.Fatal(err)
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	var contract map[string]any
	if err = decoder.Decode(&contract); err != nil {
		t.Fatal(err)
	}
	if expected != nil {
		contract["expected"] = expected
	}
	return contract
}

func pathsReturn(format string, id string) map[string]Node {
	return map[string]Node{
		"stop":       "later",
		"checkpoint": map[string]Node{"format": format, "checkpoint_id": id, "call": "lookup"},
	}
}

// A response member path compares that member alone from suite/62 and is refused below it.
func TestMemberPathContract(t *testing.T) {
	contract, err := admitDirectResponse(pathsContract(t, nil))
	if err != nil {
		t.Fatalf("suite/62 admits a response member path: %v", err)
	}
	if err = contract.compare(pathsReturn("other", "cp-1")); err != nil {
		t.Fatalf("a return whose other members differ failed: %v", err)
	}
	if err = contract.compare(pathsReturn("f-1", "cp-2")); err == nil {
		t.Fatal("a return whose named member differs passed")
	}
	if err = memberPathsMajor(pathsContract(t, nil), 62); err != nil {
		t.Fatalf("suite/62 refused a member path: %v", err)
	}
	if err = memberPathsMajor(pathsContract(t, nil), 61); err == nil {
		t.Fatal("suite/61 admitted a response member path")
	}
	for _, path := range []string{"checkpoint.checkpoint", "stop.length", "checkpoint."} {
		if _, err = admitDirectResponse(pathsContract(t, map[string]any{path: "cp-1"})); err == nil {
			t.Fatalf("the path %s was admitted", path)
		}
	}
	fmt.Println("member path admission checked")
}
