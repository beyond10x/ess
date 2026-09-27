// An in-memory implementation of `explore-external-adversary.yaml` for the adversary lane of
// beyond10x/ess#156. An arrangement holds for its command until the scenario ends.

package essconform

import "fmt"

type exploreAdvUnsupported struct{ reason string }

func (e exploreAdvUnsupported) Error() string { return e.reason }
func (e exploreAdvUnsupported) Unwrap() error { return ErrUnsupported }

type exploreAdvTarget struct {
	states  map[string]string
	created int
	version int
	forced  map[string]string
}

func newExploreAdvTarget() *exploreAdvTarget {
	target := &exploreAdvTarget{}
	target.reset()
	return target
}

func (t *exploreAdvTarget) reset() {
	t.states = map[string]string{}
	t.created = 0
	t.version = 0
	t.forced = map[string]string{}
}

func (t *exploreAdvTarget) answered(outcome, event string, payload map[string]Node) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: payload}},
	}
}

func (t *exploreAdvTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-external-adversary-target", Version: "1"}, nil
}

func (t *exploreAdvTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreAdvTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreAdvTarget) conflict() CommandResult {
	return CommandResult{Outcome: "wrong-state", Error: "exploreadv.desk.TicketStateConflict", DirectEvents: []ObservedEvent{}}
}

func (t *exploreAdvTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	switch request.Command {
	case "exploreadv.desk.Open":
		t.created++
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", t.created)
		t.states[id] = "Open"
		return t.answered("opened", "exploreadv.desk.Opened", map[string]Node{"ticket_id": id}), nil
	case "exploreadv.desk.Hold":
		id, _ := input["ticket_id"].(string)
		if t.states[id] != "Open" {
			return t.conflict(), nil
		}
		t.states[id] = "Held"
		return t.answered("held", "exploreadv.desk.Held", map[string]Node{"ticket_id": id}), nil
	case "exploreadv.desk.Close":
		id, _ := input["ticket_id"].(string)
		switch t.states[id] {
		case "Open":
			t.states[id] = "Closed"
			return t.answered("closed", "exploreadv.desk.Closed", map[string]Node{"ticket_id": id}), nil
		case "Held":
			if t.forced[request.Command] == "overridden" {
				t.states[id] = "Closed"
				return t.answered("overridden", "exploreadv.desk.Overridden", map[string]Node{"ticket_id": id}), nil
			}
			return t.conflict(), nil
		default:
			return t.conflict(), nil
		}
	default:
		return CommandResult{}, exploreAdvUnsupported{request.Command + " is not a command of explore-external-adversary.yaml"}
	}
}

func (t *exploreAdvTarget) QueryView(request ViewRequest) (ViewResult, error) {
	return ViewResult{}, exploreAdvUnsupported{request.View + " is not a view of explore-external-adversary.yaml"}
}

func (t *exploreAdvTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreAdvTarget) ConfigureExternalOutcome(control ExternalOutcomeControl) error {
	t.forced[control.Command] = control.Outcome
	return nil
}

func (t *exploreAdvTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreAdvUnsupported{"no redelivery"}
}

func (t *exploreAdvTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreAdvUnsupported{"no bindings"}
}
