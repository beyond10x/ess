// An in-memory implementation of `explore-external-exits.yaml` for the second adversary pass of
// beyond10x/ess#156. An arrangement holds for its command until the scenario ends. `mode` is empty
// (arranges every external branch) or `cannot-arrange`.

package essconform

import (
	"fmt"
	"strings"
)

type exploreExitUnsupported struct{ reason string }

func (e exploreExitUnsupported) Error() string { return e.reason }
func (e exploreExitUnsupported) Unwrap() error { return ErrUnsupported }

type exploreExitTarget struct {
	mode    string
	states  map[string]string
	created int
	version int
	forced  map[string]string
}

func newExploreExitTarget(mode string) *exploreExitTarget {
	target := &exploreExitTarget{mode: mode}
	target.reset()
	return target
}

func (t *exploreExitTarget) reset() {
	t.states = map[string]string{}
	t.created = 0
	t.version = 0
	t.forced = map[string]string{}
}

func (t *exploreExitTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-external-exits-target", Version: "1"}, nil
}

func (t *exploreExitTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreExitTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func exploreExitRefused(outcome, err string) CommandResult {
	return CommandResult{Outcome: outcome, Error: err, DirectEvents: []ObservedEvent{}}
}

func (t *exploreExitTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	switch request.Command {
	case "exploreexit.desk.Open":
		t.created++
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", t.created)
		t.states[id] = "Open"
		t.version++
		return CommandResult{
			Outcome:      "opened",
			Consistency:  fmt.Sprintf("v%d", t.version),
			DirectEvents: []ObservedEvent{{Event: "exploreexit.desk.Opened", Payload: map[string]Node{"ticket_id": id}}},
		}, nil
	case "exploreexit.desk.Close":
		id, _ := input["ticket_id"].(string)
		if t.forced[request.Command] == "bounced" {
			return exploreExitRefused("bounced", "exploreexit.desk.Bounced"), nil
		}
		if t.states[id] == "Open" {
			t.states[id] = "Closed"
			t.version++
			return CommandResult{
				Outcome:      "closed",
				Consistency:  fmt.Sprintf("v%d", t.version),
				DirectEvents: []ObservedEvent{{Event: "exploreexit.desk.Closed", Payload: map[string]Node{"ticket_id": id}}},
			}, nil
		}
		return exploreExitRefused("undeclared-state", "exploreexit.desk.Bounced"), nil
	case "exploreexit.desk.Rate":
		if t.forced[request.Command] == "throttled" {
			return exploreExitRefused("throttled", "exploreexit.desk.Throttled"), nil
		}
		rated := "High"
		if exploreExitScore(input["score"]) <= 3 {
			rated = "Low"
		}
		t.version++
		return CommandResult{
			Outcome:      strings.ToLower(rated),
			Consistency:  fmt.Sprintf("v%d", t.version),
			DirectEvents: []ObservedEvent{{Event: "exploreexit.desk.Rated" + rated, Payload: map[string]Node{"score": input["score"]}}},
		}, nil
	default:
		return CommandResult{}, exploreExitUnsupported{request.Command + " is not a command of explore-external-exits.yaml"}
	}
}

func exploreExitScore(value any) float64 {
	var score float64
	switch value := value.(type) {
	case float64:
		score = value
	case int:
		score = float64(value)
	case int64:
		score = float64(value)
	default:
		fmt.Sscan(fmt.Sprint(value), &score)
	}
	return score
}

func (t *exploreExitTarget) QueryView(request ViewRequest) (ViewResult, error) {
	return ViewResult{}, exploreExitUnsupported{request.View + " is not a view of explore-external-exits.yaml"}
}

func (t *exploreExitTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreExitTarget) ConfigureExternalOutcome(control ExternalOutcomeControl) error {
	if t.mode == "cannot-arrange" {
		return exploreExitUnsupported{"this double arranges no external outcome"}
	}
	t.forced[control.Command] = control.Outcome
	return nil
}

func (t *exploreExitTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreExitUnsupported{"no redelivery"}
}

func (t *exploreExitTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreExitUnsupported{"no bindings"}
}
