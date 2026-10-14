// An in-memory implementation of `explore-preserves.yaml` (beyond10x/ess#536).
//
// `explore-preserves-target.mjs` is the same target in TypeScript, line for line; its header lists
// the modes.

package essconform

import "fmt"

type exploreKeepUnsupported struct{ reason string }

func (e exploreKeepUnsupported) Error() string { return e.reason }
func (e exploreKeepUnsupported) Unwrap() error { return ErrUnsupported }

type exploreKeepSession struct {
	id    string
	state string
	label Node
}

type exploreKeepTarget struct {
	mode     string
	sessions []*exploreKeepSession
	version  int
}

func newExploreKeepTarget(mode string) *exploreKeepTarget {
	return &exploreKeepTarget{mode: mode}
}

func (t *exploreKeepTarget) reset() {
	t.sessions = nil
	t.version = 0
}

func (t *exploreKeepTarget) find(id Node) *exploreKeepSession {
	for _, session := range t.sessions {
		if session.id == id {
			return session
		}
	}
	return nil
}

func exploreKeepRefused(outcome, err string) CommandResult {
	return CommandResult{Outcome: outcome, Error: err, DirectEvents: []ObservedEvent{}}
}

func (t *exploreKeepTarget) answered(outcome, event, id string) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: map[string]Node{"session_id": id}}},
	}
}

func (t *exploreKeepTarget) kept(outcome string) CommandResult {
	return CommandResult{Outcome: outcome, Consistency: fmt.Sprintf("v%d", t.version), DirectEvents: []ObservedEvent{}}
}

func (t *exploreKeepTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-preserves-target", Version: "1"}, nil
}

func (t *exploreKeepTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreKeepTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreKeepTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	session := t.find(input["session_id"])
	switch request.Command {
	case "explorekeep.desk.Start":
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", len(t.sessions)+1)
		t.sessions = append(t.sessions, &exploreKeepSession{id: id, state: "Active", label: input["label"]})
		return t.answered("started", "explorekeep.desk.Started", id), nil
	case "explorekeep.desk.Pause":
		if session == nil {
			return CommandResult{}, fmt.Errorf("no session %v", input["session_id"])
		}
		if session.state == "Paused" {
			if t.mode == "paused-relabels" {
				session.label = ""
			}
			return t.kept("already-paused"), nil
		}
		session.state = "Paused"
		return t.answered("paused", "explorekeep.desk.Paused", session.id), nil
	case "explorekeep.desk.PauseAgain":
		if session == nil {
			return CommandResult{}, fmt.Errorf("no session %v", input["session_id"])
		}
		if session.state != "Paused" {
			return exploreKeepRefused("not-paused", "explorekeep.desk.NotPaused"), nil
		}
		if t.mode == "again-resumes" {
			session.state = "Active"
		}
		return t.kept("still-paused"), nil
	case "explorekeep.desk.Resume":
		if session == nil {
			return CommandResult{}, fmt.Errorf("no session %v", input["session_id"])
		}
		if session.state != "Paused" {
			return exploreKeepRefused("not-active", "explorekeep.desk.NotActive"), nil
		}
		session.state = "Active"
		return t.answered("resumed", "explorekeep.desk.Resumed", session.id), nil
	default:
		return CommandResult{}, exploreKeepUnsupported{request.Command + " is not a command of explore-preserves.yaml"}
	}
}

func (t *exploreKeepTarget) QueryView(request ViewRequest) (ViewResult, error) {
	rows := []Row{}
	switch request.View {
	case "explorekeep.desk.Sessions":
		for _, session := range t.sessions {
			rows = append(rows, Row{"session_id": session.id, "state": session.state, "label": session.label})
		}
	case "explorekeep.desk.PausedSessions":
		for _, session := range t.sessions {
			if session.state == "Paused" {
				rows = append(rows, Row{"session_id": session.id, "label": session.label})
			}
		}
	default:
		return ViewResult{}, exploreKeepUnsupported{request.View + " is not a view of explore-preserves.yaml"}
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreKeepTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreKeepTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreKeepUnsupported{"no external outcomes"}
}

func (t *exploreKeepTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreKeepUnsupported{"no redelivery"}
}

func (t *exploreKeepTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreKeepUnsupported{"no bindings"}
}
