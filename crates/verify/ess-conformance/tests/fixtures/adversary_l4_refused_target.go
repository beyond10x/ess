// Adversary pass 1 on beyond10x/ess#537: an in-memory implementation of `explore-refused.yaml`
// whose modes refuse a command in some held states and drive it in the others.
//
// `adversary-l4-refused-target.mjs` is the same target in TypeScript, line for line; its header
// lists the modes.

package essconform

import (
	"fmt"
	"sync"
)

type advL4Unsupported struct{ reason string }

func (e advL4Unsupported) Error() string { return e.reason }
func (e advL4Unsupported) Unwrap() error { return ErrUnsupported }

type advL4Call struct {
	id    string
	state string
	line  Node
}

type advL4Target struct {
	mode    string
	calls   []*advL4Call
	version int
}

// advL4Sent counts, per mode, the `Answer` steps sent for a ringing call across every target of
// the process, so `ringing-once` can tell a resend from the first refusal.
var (
	advL4Mu   sync.Mutex
	advL4Sent = map[string]int{}
)

func newAdversaryL4RefusedTarget(mode string) *advL4Target {
	return &advL4Target{mode: mode}
}

func (t *advL4Target) reset() {
	t.calls = nil
	t.version = 0
}

func (t *advL4Target) find(id Node) *advL4Call {
	for _, call := range t.calls {
		if call.id == id {
			return call
		}
	}
	return nil
}

func (t *advL4Target) answered(outcome, event, id string) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: map[string]Node{"call_id": id}}},
	}
}

func advL4Refused(outcome, err string) CommandResult {
	return CommandResult{Outcome: outcome, Error: err, DirectEvents: []ObservedEvent{}}
}

func (t *advL4Target) Identity() (Identity, error) {
	return Identity{Name: "adversary-l4-refused-target", Version: "1"}, nil
}

func (t *advL4Target) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *advL4Target) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *advL4Target) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	call := t.find(input["call_id"])
	switch request.Command {
	case "explorerefused.desk.Dial":
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", len(t.calls)+1)
		t.calls = append(t.calls, &advL4Call{id: id, state: "Ringing", line: input["line"]})
		return t.answered("dialled", "explorerefused.desk.Dialled", id), nil
	case "explorerefused.desk.Hold":
		if call == nil {
			return CommandResult{}, fmt.Errorf("no call %v", input["call_id"])
		}
		if t.mode == "hold-waiting" && call.state == "Waiting" {
			return CommandResult{}, advL4Unsupported{"Hold has no callback while the call waits"}
		}
		if t.mode == "hold-everywhere-answer-ringing" {
			return CommandResult{}, advL4Unsupported{"Hold has no callback"}
		}
		if call.state != "Ringing" {
			return advL4Refused("not-ringing", "explorerefused.desk.NotRinging"), nil
		}
		call.state = "Waiting"
		return t.answered("held", "explorerefused.desk.Held", call.id), nil
	case "explorerefused.desk.Answer":
		if call == nil {
			return CommandResult{}, fmt.Errorf("no call %v", input["call_id"])
		}
		if call.state == "Ringing" {
			switch t.mode {
			case "ringing-once":
				advL4Mu.Lock()
				advL4Sent[t.mode]++
				sent := advL4Sent[t.mode]
				advL4Mu.Unlock()
				if sent == 1 {
					return CommandResult{}, advL4Unsupported{"Answer has no callback while the call rings"}
				}
				return CommandResult{}, fmt.Errorf("Answer was sent again in Ringing after the target refused it there (send %d)", sent)
			case "ringing-connected", "hold-everywhere-answer-ringing":
				return CommandResult{}, advL4Unsupported{"Answer has no callback while the call rings"}
			}
		}
		if t.mode == "ringing-connected" && call.state == "Connected" {
			return CommandResult{}, advL4Unsupported{"Answer has no callback once connected"}
		}
		switch call.state {
		case "Ringing":
			call.state = "Connected"
			return t.answered("answered", "explorerefused.desk.Connected", call.id), nil
		case "Waiting":
			call.state = "Connected"
			return t.answered("picked-up", "explorerefused.desk.Connected", call.id), nil
		default:
			return advL4Refused("not-open", "explorerefused.desk.NotOpen"), nil
		}
	default:
		return CommandResult{}, advL4Unsupported{request.Command + " is not a command of explore-refused.yaml"}
	}
}

func (t *advL4Target) QueryView(request ViewRequest) (ViewResult, error) {
	rows := []Row{}
	switch request.View {
	case "explorerefused.desk.Calls":
		for _, call := range t.calls {
			rows = append(rows, Row{"call_id": call.id, "state": call.state, "line": call.line})
		}
	case "explorerefused.desk.RingingCalls":
		for _, call := range t.calls {
			if call.state == "Ringing" {
				rows = append(rows, Row{"call_id": call.id})
			}
		}
	default:
		return ViewResult{}, advL4Unsupported{request.View + " is not served"}
	}
	return ViewResult{Rows: rows}, nil
}

func (t *advL4Target) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *advL4Target) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return advL4Unsupported{"no external outcomes"}
}

func (t *advL4Target) RedeliverEvent(RedeliveryRequest) error {
	return advL4Unsupported{"no redelivery"}
}

func (t *advL4Target) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, advL4Unsupported{"no bindings"}
}
