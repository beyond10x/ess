// An in-memory implementation of `explore-selected-rows-unknown.yaml`, as the interpreter runs it.
//
// `explore-selected-rows-unknown-target.mjs` is the same target in TypeScript.

package essconform

import (
	"errors"
	"fmt"
)

type exploreUnkUnsupported struct{ reason string }

func (e exploreUnkUnsupported) Error() string { return e.reason }
func (e exploreUnkUnsupported) Unwrap() error { return ErrUnsupported }

type exploreUnkTarget struct {
	seats   []string
	version int
}

func newExploreUnkTarget(string) *exploreUnkTarget { return &exploreUnkTarget{} }

func (t *exploreUnkTarget) answered(outcome string, events []ObservedEvent) CommandResult {
	t.version++
	return CommandResult{Outcome: outcome, Consistency: fmt.Sprintf("v%d", t.version), DirectEvents: events}
}

func (t *exploreUnkTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-selected-rows-unknown-target", Version: "1"}, nil
}

func (t *exploreUnkTarget) BeginScenario(ScenarioContext) error {
	t.seats, t.version = nil, 0
	return nil
}

func (t *exploreUnkTarget) EndScenario(ScenarioContext) error {
	t.seats, t.version = nil, 0
	return nil
}

func (t *exploreUnkTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	switch request.Command {
	case "exploreunk.desk.Start":
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", len(t.seats)+1)
		t.seats = append(t.seats, id)
		return t.answered("started", []ObservedEvent{{Event: "exploreunk.desk.Started", Payload: map[string]Node{"seat_id": id}}}), nil
	case "exploreunk.desk.Sweep":
		if len(t.seats) > 0 {
			return CommandResult{}, errors.New("sweep reads a zone no outcome wrote")
		}
		return t.answered("swept", []ObservedEvent{{Event: "exploreunk.desk.Swept", Payload: map[string]Node{"team": input["team"]}}}), nil
	case "exploreunk.desk.SweepTagged":
		return t.answered("swept-tagged", []ObservedEvent{{Event: "exploreunk.desk.Swept", Payload: map[string]Node{"team": input["team"]}}}), nil
	default:
		return CommandResult{}, exploreUnkUnsupported{request.Command + " is not a command of explore-selected-rows-unknown.yaml"}
	}
}

func (t *exploreUnkTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "exploreunk.desk.Seats" {
		return ViewResult{}, exploreUnkUnsupported{request.View + " is not a view"}
	}
	rows := []Row{}
	for _, id := range t.seats {
		rows = append(rows, Row{"seat_id": id, "state": "Open"})
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreUnkTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreUnkTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreUnkUnsupported{"no external outcomes"}
}

func (t *exploreUnkTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreUnkUnsupported{"no redelivery"}
}

func (t *exploreUnkTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreUnkUnsupported{"no bindings"}
}
