// An in-memory implementation of `explore-before-row.yaml`, a test double that arranges external
// outcomes the way the synthesized scenarios ask it to. It is copied into the package as
// `explore_external_target_test.go`, which `explore_external_driver_test.go` runs.
//
// `explore-before-row-target.mjs` is the same target in TypeScript, line for line; its header
// lists the modes.

package essconform

import "fmt"

type exploreExtUnsupported struct{ reason string }

func (e exploreExtUnsupported) Error() string { return e.reason }
func (e exploreExtUnsupported) Unwrap() error { return ErrUnsupported }

type exploreExtSeat struct {
	id    string
	label Node
	state string
}

type exploreExtTarget struct {
	mode    string
	seats   map[string]*exploreExtSeat
	order   []string
	created int
	version int
	forced  map[string]string
}

func newExploreExtTarget(mode string) *exploreExtTarget {
	target := &exploreExtTarget{mode: mode}
	target.reset()
	return target
}

func (t *exploreExtTarget) reset() {
	t.seats = map[string]*exploreExtSeat{}
	t.order = nil
	t.created = 0
	t.version = 0
	t.forced = map[string]string{}
}

func (t *exploreExtTarget) answered(outcome, event string, payload map[string]Node) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: payload}},
	}
}

func exploreExtRefused(outcome, err string) CommandResult {
	return CommandResult{Outcome: outcome, Error: err, DirectEvents: []ObservedEvent{}}
}

func (t *exploreExtTarget) arranged(command, outcome string) bool {
	return t.forced[command] == outcome
}

func (t *exploreExtTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-before-row-target", Version: "1"}, nil
}

func (t *exploreExtTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreExtTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreExtTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	switch request.Command {
	case "exploreext.desk.OpenSeat":
		t.created++
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", t.created)
		t.seats[id] = &exploreExtSeat{id: id, label: input["label"], state: "Open"}
		t.order = append(t.order, id)
		return t.answered("opened", "exploreext.desk.SeatOpened", map[string]Node{"seat_id": id}), nil
	case "exploreext.desk.SignIn":
		if t.mode != "row-first" && t.arranged(request.Command, "bad-credential") {
			return exploreExtRefused("bad-credential", "exploreext.desk.BadCredential"), nil
		}
		id, _ := input["seat_id"].(string)
		seat, held := t.seats[id]
		if !held {
			return exploreExtRefused("no-seat", "exploreext.desk.NoSeat"), nil
		}
		if t.arranged(request.Command, "bad-credential") {
			return exploreExtRefused("bad-credential", "exploreext.desk.BadCredential"), nil
		}
		if seat.state == "Open" {
			seat.state = "Busy"
			return t.answered("taken", "exploreext.desk.Taken", map[string]Node{"seat_id": seat.id}), nil
		}
		seat.state = "Open"
		return t.answered("released", "exploreext.desk.Released", map[string]Node{"seat_id": seat.id}), nil
	default:
		return CommandResult{}, exploreExtUnsupported{request.Command + " is not a command of explore-before-row.yaml"}
	}
}

func (t *exploreExtTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "exploreext.desk.Seats" {
		return ViewResult{}, exploreExtUnsupported{request.View + " is not a view of explore-before-row.yaml"}
	}
	rows := []Row{}
	for _, id := range t.order {
		seat := t.seats[id]
		rows = append(rows, Row{"seat_id": seat.id, "label": seat.label, "state": seat.state})
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreExtTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreExtTarget) ConfigureExternalOutcome(control ExternalOutcomeControl) error {
	t.forced[control.Command] = control.Outcome
	return nil
}

func (t *exploreExtTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreExtUnsupported{"no redelivery"}
}

func (t *exploreExtTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreExtUnsupported{"no bindings"}
}
