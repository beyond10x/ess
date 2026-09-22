package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"testing"
)

type fixtureTarget struct {
	mode                   string
	values                 map[string]Node
	resolved, begun, ended int
}

func (*fixtureTarget) Identity() (Identity, error) {
	return Identity{Name: "fixtures", Version: "1"}, nil
}
func (f *fixtureTarget) FixtureValues(_ ScenarioContext, contract FixtureContract) (map[string]Node, error) {
	f.resolved++
	contract.Fields[0].Type = "Boolean"
	f.values = map[string]Node{"current-principal-id": "d248c830-37a4-466b-8319-af71667f1364", "current-principal-email": "principal-1@example.com"}
	switch f.mode {
	case "invalid":
		f.values["current-principal-id"] = true
	case "missing":
		delete(f.values, "current-principal-email")
	case "unknown":
		f.values["unknown"] = nil
	case "unsupported":
		return nil, ErrUnsupported
	}
	return f.values, nil
}
func (f *fixtureTarget) BeginScenario(ScenarioContext) error {
	fmt.Println("fixture session begun")
	f.begun++
	if f.resolved != 1 {
		return fmt.Errorf("fixtures must resolve once before session startup")
	}
	f.values["current-principal-email"] = "changed@example.com"
	return nil
}
func (f *fixtureTarget) EndScenario(ScenarioContext) error {
	f.ended++
	fmt.Println("fixture session ended")
	return nil
}
func (f *fixtureTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	if request.Input["email"] != "principal-1@example.com" {
		return CommandResult{}, fmt.Errorf("expected independent fixture input")
	}
	payload := request.Input
	result := CommandResult{Outcome: "opened"}
	if f.mode == "wrong-then-right" {
		other := map[string]Node{}
		for key, value := range payload {
			other[key] = value
		}
		other["email"] = "wrong@example.com"
		result.DirectEvents = append(result.DirectEvents, ObservedEvent{Event: "fixturetest.session.Opened", Payload: other})
	}
	if f.mode == "split-output" {
		other := map[string]Node{}
		for key, value := range payload {
			other[key] = value
		}
		other["region"] = "wrong-region"
		result.DirectEvents = append(result.DirectEvents, ObservedEvent{Event: "fixturetest.session.Opened", Payload: other})
	}
	if f.mode == "wrong-output" || f.mode == "split-output" {
		payload["email"] = "different@example.com"
	}
	result.DirectEvents = append(result.DirectEvents, ObservedEvent{Event: "fixturetest.session.Opened", Payload: payload})
	return result, nil
}
func (*fixtureTarget) QueryView(ViewRequest) (ViewResult, error)             { return ViewResult{}, ErrUnsupported }
func (*fixtureTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error { return ErrUnsupported }
func (*fixtureTarget) RedeliverEvent(RedeliveryRequest) error                { return ErrUnsupported }
func (*fixtureTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, ErrUnsupported
}
func (*fixtureTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}
func TestFixtureExecution(t *testing.T) {
	Run(t, func() Target { return &fixtureTarget{mode: os.Getenv("ESS_FIXTURE_CASE")} })
}
func TestFixtureCopiesAndExactNumbers(t *testing.T) {
	source := map[string]Node{"nested": map[string]any{"email": "original@example.com"}}
	copied, err := copyFixtureValue(source)
	if err != nil {
		t.Fatal(err)
	}
	source["nested"].(map[string]any)["email"] = "changed@example.com"
	if copied.(map[string]any)["nested"].(map[string]any)["email"] != "original@example.com" {
		t.Fatal("fixture aliases provider storage")
	}
	if equal(json.Number("9007199254740993"), json.Number("9007199254740992")) {
		t.Fatal("distinct integers collapsed")
	}
	if !equal(json.Number("1.0"), float64(1)) {
		t.Fatal("equal integers differ")
	}
	if equal("1", json.Number("1")) {
		t.Fatal("string coerced to number")
	}
}
