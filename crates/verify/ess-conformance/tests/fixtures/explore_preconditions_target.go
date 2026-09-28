// An in-memory implementation of `explore-preconditions.yaml` (ess/15, beyond10x/ess#152).
//
// `explore-preconditions-target.mjs` is the same target in TypeScript, line for line; its header
// lists the modes.

package essconform

import "fmt"

type explorePreUnsupported struct{ reason string }

func (e explorePreUnsupported) Error() string { return e.reason }
func (e explorePreUnsupported) Unwrap() error { return ErrUnsupported }

type explorePreTarget struct {
	mode    string
	users   []Row
	version int
}

func newExplorePreTarget(mode string) *explorePreTarget {
	return &explorePreTarget{mode: mode}
}

func (t *explorePreTarget) reset() {
	t.users = nil
	t.version = 0
}

func (t *explorePreTarget) answered(outcome, event string, payload map[string]Node) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: payload}},
	}
}

func (t *explorePreTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-preconditions-target", Version: "1"}, nil
}

func (t *explorePreTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *explorePreTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *explorePreTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	switch request.Command {
	case "explorepre.desk.OpenSession":
		if t.mode == "refuses-session" {
			return CommandResult{DirectEvents: []ObservedEvent{}}, nil
		}
		accounts := request.Input["accounts"]
		if t.mode == "drops-accounts" {
			accounts = []Node{}
		}
		t.users = append(t.users, Row{"user_id": request.Input["user_id"], "accounts": accounts})
		return t.answered("opened", "explorepre.desk.SessionOpened", map[string]Node{"user_id": request.Input["user_id"]}), nil
	case "explorepre.desk.ClearWrapUp":
		if len(t.users) == 0 {
			return CommandResult{DirectEvents: []ObservedEvent{}}, nil
		}
		return t.answered("cleared", "explorepre.desk.Cleared", map[string]Node{}), nil
	default:
		return CommandResult{}, explorePreUnsupported{request.Command + " is not a command of explore-preconditions.yaml"}
	}
}

func (t *explorePreTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "explorepre.desk.Users" {
		return ViewResult{}, explorePreUnsupported{request.View + " is not a view of explore-preconditions.yaml"}
	}
	rows := []Row{}
	for _, user := range t.users {
		rows = append(rows, Row{"user_id": user["user_id"], "accounts": user["accounts"], "state": "Active"})
	}
	return ViewResult{Rows: rows}, nil
}

func (t *explorePreTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *explorePreTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return explorePreUnsupported{"no external outcomes"}
}

func (t *explorePreTarget) RedeliverEvent(RedeliveryRequest) error {
	return explorePreUnsupported{"no redelivery"}
}

func (t *explorePreTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, explorePreUnsupported{"no bindings"}
}
