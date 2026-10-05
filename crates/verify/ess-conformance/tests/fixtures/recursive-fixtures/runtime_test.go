package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"testing"
)

// The finite tree the provider supplies, and the malformed variants it may supply instead.
const finiteTree = `{"text":"root","children":[{"text":"a","children":[]},{"text":"b","children":[{"text":"c","children":[]}]}]}`

type recursiveTarget struct {
	mode     string
	supplied string
}

func (*recursiveTarget) Identity() (Identity, error) {
	return Identity{Name: "recursive-fixtures", Version: "1"}, nil
}
func (r *recursiveTarget) FixtureValues(_ ScenarioContext, contract FixtureContract) (map[string]Node, error) {
	r.supplied = finiteTree
	switch r.mode {
	case "wrong-leaf":
		r.supplied = strings.Replace(finiteTree, `"text":"c"`, `"text":5`, 1)
	case "too-deep":
		r.supplied = strings.Repeat(`{"text":"deep","children":[`, 70) + strings.Repeat(`]}`, 70)
	}
	var value Node
	if err := json.Unmarshal([]byte(r.supplied), &value); err != nil {
		return nil, err
	}
	return map[string]Node{"finite-value": value}, nil
}
func (*recursiveTarget) BeginScenario(ScenarioContext) error {
	fmt.Println("recursive session begun")
	return nil
}
func (*recursiveTarget) EndScenario(ScenarioContext) error { return nil }
func (r *recursiveTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	received, err := json.Marshal(request.Input["value"])
	if err != nil {
		return CommandResult{}, err
	}
	var want, got any
	if err := json.Unmarshal([]byte(r.supplied), &want); err != nil {
		return CommandResult{}, err
	}
	if err := json.Unmarshal(received, &got); err != nil {
		return CommandResult{}, err
	}
	wanted, _ := json.Marshal(want)
	gotten, _ := json.Marshal(got)
	if string(wanted) != string(gotten) {
		return CommandResult{}, fmt.Errorf("the command did not receive the supplied tree: %s", gotten)
	}
	fmt.Println("recursive fixture received")
	return CommandResult{
		Outcome:      "accepted",
		DirectEvents: []ObservedEvent{{Event: "fixtureprobe.recursive.Accepted", Payload: map[string]Node{"accepted": true}}},
	}, nil
}
func (*recursiveTarget) QueryView(ViewRequest) (ViewResult, error)             { return ViewResult{}, ErrUnsupported }
func (*recursiveTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error { return ErrUnsupported }
func (*recursiveTarget) RedeliverEvent(RedeliveryRequest) error                { return ErrUnsupported }
func (*recursiveTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, ErrUnsupported
}
func (*recursiveTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}
func TestRecursiveFixture(t *testing.T) {
	Run(t, func() Target { return &recursiveTarget{mode: os.Getenv("ESS_FIXTURE_CASE")} })
}
