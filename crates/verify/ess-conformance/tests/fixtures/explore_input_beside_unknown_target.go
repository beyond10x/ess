// An in-memory implementation of `explore-input-beside-unknown.yaml`. It is copied into the package
// as `explore_external_target_test.go`, which `explore_external_driver_test.go` runs.
//
// The mode is the order `Nudge` and `Route` ask their questions in: "" (the declaration order: the
// amount, the ticket, then the desk) or "ticket-first" (the ticket before the amount).
//
// `explore-input-beside-unknown-target.mjs` is the same target in TypeScript, line for line.

package essconform

import (
	"encoding/json"
	"fmt"
)

type exploreExtUnsupported struct{ reason string }

func (e exploreExtUnsupported) Error() string { return e.reason }
func (e exploreExtUnsupported) Unwrap() error { return ErrUnsupported }

type exploreExtTarget struct {
	mode    string
	tickets map[string]bool
	desks   map[string]bool
	created int
	version int
}

func newExploreExtTarget(mode string) *exploreExtTarget {
	target := &exploreExtTarget{mode: mode}
	target.reset()
	return target
}

func (t *exploreExtTarget) reset() {
	t.tickets = map[string]bool{}
	t.desks = map[string]bool{}
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

func exploreExtRefused(outcome, err string) CommandResult {
	return CommandResult{Outcome: outcome, Error: err, DirectEvents: []ObservedEvent{}}
}

func (t *exploreExtTarget) identity() string {
	t.created++
	return fmt.Sprintf("00000000-0000-4000-8000-%012d", t.created)
}

func exploreExtAmount(value Node) float64 {
	switch amount := value.(type) {
	case json.Number:
		parsed, _ := amount.Float64()
		return parsed
	case float64:
		return amount
	case int:
		return float64(amount)
	case int64:
		return float64(amount)
	}
	panic(fmt.Sprintf("amount %#v is not a number", value))
}

// answer is `Nudge`, or `Route` where routed is set, which reads the desk as well.
func (t *exploreExtTarget) answer(input map[string]Node, routed bool) CommandResult {
	ticket, _ := input["ticket_id"].(string)
	desk, _ := input["desk_id"].(string)
	if t.mode == "ticket-first" && !t.tickets[ticket] {
		return exploreExtRefused("no-ticket", "exploreiu.desk.NoTicket")
	}
	if exploreExtAmount(input["amount"]) > 100 {
		return exploreExtRefused("too-much", "exploreiu.desk.TooMuch")
	}
	if !t.tickets[ticket] {
		return exploreExtRefused("no-ticket", "exploreiu.desk.NoTicket")
	}
	if !routed {
		return t.answered("nudged", "exploreiu.desk.Nudged", map[string]Node{"ticket_id": ticket})
	}
	if !t.desks[desk] {
		return exploreExtRefused("no-desk", "exploreiu.desk.NoDesk")
	}
	return t.answered("routed", "exploreiu.desk.Routed", map[string]Node{"ticket_id": ticket})
}

func (t *exploreExtTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-input-beside-unknown-target", Version: "1"}, nil
}

func (t *exploreExtTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreExtTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreExtTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	switch request.Command {
	case "exploreiu.desk.MakeDesk":
		id := t.identity()
		t.desks[id] = true
		return t.answered("made", "exploreiu.desk.DeskMade", map[string]Node{"desk_id": id}), nil
	case "exploreiu.desk.OpenTicket":
		id := t.identity()
		t.tickets[id] = true
		return t.answered("opened", "exploreiu.desk.Opened", map[string]Node{"ticket_id": id}), nil
	case "exploreiu.desk.Nudge":
		return t.answer(request.Input, false), nil
	case "exploreiu.desk.Route":
		return t.answer(request.Input, true), nil
	default:
		return CommandResult{}, exploreExtUnsupported{request.Command + " is not a command of explore-input-beside-unknown.yaml"}
	}
}

func (t *exploreExtTarget) QueryView(request ViewRequest) (ViewResult, error) {
	return ViewResult{}, exploreExtUnsupported{request.View + " is not a view of explore-input-beside-unknown.yaml"}
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
