// An in-memory implementation of `explore-refused.yaml` (beyond10x/ess#537).
//
// `explore-refused-target.mjs` is the same target in TypeScript, line for line; its header lists
// the modes.

package essconform

import "fmt"

type exploreRefusedUnsupported struct{ reason string }

func (e exploreRefusedUnsupported) Error() string { return e.reason }
func (e exploreRefusedUnsupported) Unwrap() error { return ErrUnsupported }

type exploreRefusedCall struct {
	id    string
	state string
	line  Node
}

type exploreRefusedTarget struct {
	mode    string
	calls   []*exploreRefusedCall
	version int
}

func newExploreRefusedTarget(mode string) *exploreRefusedTarget {
	return &exploreRefusedTarget{mode: mode}
}

func (t *exploreRefusedTarget) reset() {
	t.calls = nil
	t.version = 0
}

func (t *exploreRefusedTarget) find(id Node) *exploreRefusedCall {
	for _, call := range t.calls {
		if call.id == id {
			return call
		}
	}
	return nil
}

func (t *exploreRefusedTarget) answered(outcome, event, id string) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: map[string]Node{"call_id": id}}},
	}
}

func exploreRefusedRefused(outcome, err string) CommandResult {
	return CommandResult{Outcome: outcome, Error: err, DirectEvents: []ObservedEvent{}}
}

func (t *exploreRefusedTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-refused-target", Version: "1"}, nil
}

func (t *exploreRefusedTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreRefusedTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreRefusedTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	call := t.find(input["call_id"])
	switch request.Command {
	case "explorerefused.desk.Dial":
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", len(t.calls)+1)
		t.calls = append(t.calls, &exploreRefusedCall{id: id, state: "Ringing", line: input["line"]})
		return t.answered("dialled", "explorerefused.desk.Dialled", id), nil
	case "explorerefused.desk.Hold":
		if call == nil {
			return CommandResult{}, fmt.Errorf("no call %v", input["call_id"])
		}
		if call.state != "Ringing" {
			return exploreRefusedRefused("not-ringing", "explorerefused.desk.NotRinging"), nil
		}
		call.state = "Waiting"
		return t.answered("held", "explorerefused.desk.Held", call.id), nil
	case "explorerefused.desk.Answer":
		if call == nil {
			return CommandResult{}, fmt.Errorf("no call %v", input["call_id"])
		}
		if t.mode == "answer" {
			return CommandResult{}, exploreRefusedUnsupported{"Answer has no callback"}
		}
		if t.mode == "ringing" && call.state == "Ringing" {
			return CommandResult{}, exploreRefusedUnsupported{"Answer has no callback while the call rings"}
		}
		switch call.state {
		case "Ringing":
			call.state = "Connected"
			return t.answered("answered", "explorerefused.desk.Connected", call.id), nil
		case "Waiting":
			call.state = "Connected"
			return t.answered("picked-up", "explorerefused.desk.Connected", call.id), nil
		default:
			return exploreRefusedRefused("not-open", "explorerefused.desk.NotOpen"), nil
		}
	default:
		return CommandResult{}, exploreRefusedUnsupported{request.Command + " is not a command of explore-refused.yaml"}
	}
}

func (t *exploreRefusedTarget) QueryView(request ViewRequest) (ViewResult, error) {
	rows := []Row{}
	switch request.View {
	case "explorerefused.desk.Calls":
		for _, call := range t.calls {
			rows = append(rows, Row{"call_id": call.id, "state": call.state, "line": call.line})
		}
	default:
		return ViewResult{}, exploreRefusedUnsupported{request.View + " is not served"}
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreRefusedTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreRefusedTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreRefusedUnsupported{"no external outcomes"}
}

func (t *exploreRefusedTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreRefusedUnsupported{"no redelivery"}
}

func (t *exploreRefusedTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreRefusedUnsupported{"no bindings"}
}
