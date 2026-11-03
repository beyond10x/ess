// An in-memory implementation of the `front` component of `explore-component.yaml`. It is copied
// into the package as `explore_external_target_test.go`, which `explore_external_driver_test.go`
// runs. `WriteNote` belongs to `back`, so it is answered as unsupported.
//
// `explore-component-target.mjs` is the same target in TypeScript, line for line.

package essconform

import "fmt"

type exploreExtUnsupported struct{ reason string }

func (e exploreExtUnsupported) Error() string { return e.reason }
func (e exploreExtUnsupported) Unwrap() error { return ErrUnsupported }

type exploreExtTarget struct {
	states  map[string]string
	created int
	version int
}

func newExploreExtTarget(string) *exploreExtTarget {
	target := &exploreExtTarget{}
	target.reset()
	return target
}

func (t *exploreExtTarget) reset() {
	t.states = map[string]string{}
	t.created = 0
	t.version = 0
}

func (t *exploreExtTarget) answered(outcome, event string, payload map[string]Node) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: payload}},
	}
}

func (t *exploreExtTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-component-target", Version: "1"}, nil
}

func (t *exploreExtTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreExtTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreExtTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	switch request.Command {
	case "explorecmp.desk.OpenTicket":
		t.created++
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", t.created)
		t.states[id] = "Open"
		return t.answered("opened", "explorecmp.desk.Opened", map[string]Node{"ticket_id": id}), nil
	case "explorecmp.desk.CloseTicket":
		id, _ := request.Input["ticket_id"].(string)
		state, held := t.states[id]
		if !held {
			return CommandResult{}, fmt.Errorf("no ticket %s", id)
		}
		if state != "Open" {
			return CommandResult{Outcome: "not-open", Error: "explorecmp.desk.NotOpen", DirectEvents: []ObservedEvent{}}, nil
		}
		t.states[id] = "Closed"
		return t.answered("closed", "explorecmp.desk.Closed", map[string]Node{"ticket_id": id}), nil
	default:
		return CommandResult{}, exploreExtUnsupported{request.Command + " is not a command of the front component"}
	}
}

func (t *exploreExtTarget) QueryView(request ViewRequest) (ViewResult, error) {
	return ViewResult{}, exploreExtUnsupported{request.View + " is not a view of the front component"}
}

func (t *exploreExtTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreExtTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreExtUnsupported{"no external branches"}
}

func (t *exploreExtTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreExtUnsupported{"no redelivery"}
}

func (t *exploreExtTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreExtUnsupported{"no bindings"}
}
