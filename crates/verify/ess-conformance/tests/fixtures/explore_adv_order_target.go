// An in-memory implementation of `explore-adv-order.yaml` (adversary pass 1, beyond10x/ess#235),
// deciding each command in Entity Runtime's order: input-guarded refusals, then the guarded and
// external branches (an arranged verdict), then the default, then the wrong-state answer for a move
// from a state no move starts from. An arrangement holds for its command until the scenario ends.
//
// `mode` is empty, `no-close` (Close is not exposed, so only Rate is explored) or `no-rate`.
// `explore-adv-order-target.mjs` is the same target in TypeScript.

package essconform

import "fmt"

type exploreAdvUnsupported struct{ reason string }

func (e exploreAdvUnsupported) Error() string { return e.reason }
func (e exploreAdvUnsupported) Unwrap() error { return ErrUnsupported }

type exploreAdvTarget struct {
	mode    string
	states  map[string]string
	created int
	version int
	forced  map[string]string
}

func newExploreAdvTarget(mode string) *exploreAdvTarget {
	t := &exploreAdvTarget{mode: mode}
	t.reset()
	return t
}

func (t *exploreAdvTarget) reset() {
	t.states = map[string]string{}
	t.created = 0
	t.version = 0
	t.forced = map[string]string{}
}

func exploreAdvRefused(outcome, err string) CommandResult {
	return CommandResult{Outcome: outcome, Error: err, DirectEvents: []ObservedEvent{}}
}

func exploreAdvScore(value any) float64 {
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

func (t *exploreAdvTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-adv-order-target", Version: "1"}, nil
}

func (t *exploreAdvTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreAdvTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreAdvTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	switch request.Command {
	case "exploreadv.desk.Open":
		t.created++
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", t.created)
		t.states[id] = "Open"
		t.version++
		return CommandResult{
			Outcome:      "opened",
			Consistency:  fmt.Sprintf("v%d", t.version),
			DirectEvents: []ObservedEvent{{Event: "exploreadv.desk.Opened", Payload: map[string]Node{"ticket_id": id}}},
		}, nil
	case "exploreadv.desk.Close":
		if t.mode == "no-close" {
			return CommandResult{}, exploreAdvUnsupported{"Close is not exposed in mode no-close"}
		}
		id := fmt.Sprint(input["ticket_id"])
		state, ok := t.states[id]
		if !ok {
			return CommandResult{}, fmt.Errorf("no ticket %s", id)
		}
		// The provider's verdict is a guard sorted before the default: it answers in every state.
		if t.forced[request.Command] == "bounced" {
			return exploreAdvRefused("bounced", "exploreadv.desk.Bounced"), nil
		}
		if state != "Open" {
			return exploreAdvRefused("wrong-state", "exploreadv.desk.StateConflict"), nil
		}
		t.states[id] = "Closed"
		t.version++
		return CommandResult{
			Outcome:      "closed",
			Consistency:  fmt.Sprintf("v%d", t.version),
			DirectEvents: []ObservedEvent{{Event: "exploreadv.desk.Closed", Payload: map[string]Node{"ticket_id": id}}},
		}, nil
	case "exploreadv.desk.Rate":
		if t.mode == "no-rate" {
			return CommandResult{}, exploreAdvUnsupported{"Rate is not exposed in mode no-rate"}
		}
		if exploreAdvScore(input["score"]) > 50 {
			return exploreAdvRefused("too-high", "exploreadv.desk.TooHigh"), nil
		}
		if t.forced[request.Command] == "throttled" {
			return exploreAdvRefused("throttled", "exploreadv.desk.Throttled"), nil
		}
		t.version++
		return CommandResult{
			Outcome:      "rated",
			Consistency:  fmt.Sprintf("v%d", t.version),
			DirectEvents: []ObservedEvent{{Event: "exploreadv.desk.Rated", Payload: map[string]Node{"score": input["score"]}}},
		}, nil
	case "exploreadv.desk.Shut", "exploreadv.desk.Resolve":
		// explore-adv-overlap.yaml: `Resolve` below 0 is the default refusal in every state; otherwise
		// the first declared guard (`quick`) moves, which only `Open` admits.
		id := fmt.Sprint(input["ticket_id"])
		state, ok := t.states[id]
		if request.Command == "exploreadv.desk.Resolve" && exploreAdvScore(input["level"]) < 0 {
			return exploreAdvRefused("unrated", "exploreadv.desk.Unrated"), nil
		}
		if !ok {
			return CommandResult{}, fmt.Errorf("no ticket %s", id)
		}
		if state != "Open" {
			return exploreAdvRefused("wrong-state", "exploreadv.desk.StateConflict"), nil
		}
		t.states[id] = "Closed"
		t.version++
		outcome, event := "shut", "exploreadv.desk.ShutDown"
		if request.Command == "exploreadv.desk.Resolve" {
			outcome, event = "quick", "exploreadv.desk.Resolved"
		}
		return CommandResult{
			Outcome:      outcome,
			Consistency:  fmt.Sprintf("v%d", t.version),
			DirectEvents: []ObservedEvent{{Event: event, Payload: map[string]Node{"ticket_id": id}}},
		}, nil
	default:
		return CommandResult{}, exploreAdvUnsupported{request.Command + " is not a command of explore-adv-order.yaml"}
	}
}

func (t *exploreAdvTarget) QueryView(request ViewRequest) (ViewResult, error) {
	return ViewResult{}, exploreAdvUnsupported{request.View + " is not a view of explore-adv-order.yaml"}
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
