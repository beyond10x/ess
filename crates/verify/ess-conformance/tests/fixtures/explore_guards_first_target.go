// An in-memory implementation of `explore-guards-first.yaml` (beyond10x/ess#235), deciding each
// command in Entity Runtime's order.
//
// `explore-guards-first-target.mjs` is the same target in TypeScript, line for line; its header
// lists the modes.

package essconform

import "fmt"

type exploreOrderUnsupported struct{ reason string }

func (e exploreOrderUnsupported) Error() string { return e.reason }
func (e exploreOrderUnsupported) Unwrap() error { return ErrUnsupported }

type exploreOrderTicket struct {
	id       string
	state    string
	priority Node
}

type exploreOrderTarget struct {
	mode    string
	tickets []*exploreOrderTicket
	version int
}

func newExploreOrderTarget(mode string) *exploreOrderTarget {
	return &exploreOrderTarget{mode: mode}
}

func (t *exploreOrderTarget) reset() {
	t.tickets = nil
	t.version = 0
}

func (t *exploreOrderTarget) find(id Node) *exploreOrderTicket {
	for _, ticket := range t.tickets {
		if ticket.id == id {
			return ticket
		}
	}
	return nil
}

func exploreOrderRefused(outcome, err string) CommandResult {
	return CommandResult{Outcome: outcome, Error: err, DirectEvents: []ObservedEvent{}}
}

func exploreOrderConflict() CommandResult {
	return exploreOrderRefused("wrong-state", "exploreorder.desk.TicketStateConflict")
}

func (t *exploreOrderTarget) answered(outcome, event, id string) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: map[string]Node{"ticket_id": id}}},
	}
}

func exploreOrderNumber(value any) float64 {
	var number float64
	switch value := value.(type) {
	case float64:
		number = value
	case int:
		number = float64(value)
	case int64:
		number = float64(value)
	default:
		fmt.Sscan(fmt.Sprint(value), &number)
	}
	return number
}

func (t *exploreOrderTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-guards-first-target", Version: "1"}, nil
}

func (t *exploreOrderTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreOrderTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreOrderTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	switch request.Command {
	case "exploreorder.desk.Open":
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", len(t.tickets)+1)
		t.tickets = append(t.tickets, &exploreOrderTicket{id: id, state: "Open", priority: float64(0)})
		return t.answered("opened", "exploreorder.desk.Opened", id), nil
	case "exploreorder.desk.Hold":
		ticket := t.find(input["ticket_id"])
		priority := exploreOrderNumber(input["priority"])
		if t.mode == "state-first" && ticket != nil && ticket.state == "Held" {
			return exploreOrderConflict(), nil
		}
		if priority < -5 && t.mode != "last-refusal" {
			return exploreOrderRefused("tiny", "exploreorder.desk.PriorityTiny"), nil
		}
		if priority < 0 {
			return exploreOrderRefused("negative", "exploreorder.desk.PriorityNegative"), nil
		}
		if priority > 100 && t.mode != "accept-first" {
			return exploreOrderRefused("capped", "exploreorder.desk.PriorityCapped"), nil
		}
		if priority == 0 {
			return exploreOrderRefused("unrated", "exploreorder.desk.Unrated"), nil
		}
		if ticket == nil {
			return CommandResult{}, fmt.Errorf("no ticket %v", input["ticket_id"])
		}
		if ticket.state != "Open" {
			return exploreOrderConflict(), nil
		}
		ticket.state = "Held"
		ticket.priority = input["priority"]
		return t.answered("held", "exploreorder.desk.Held", ticket.id), nil
	case "exploreorder.desk.Release":
		ticket := t.find(input["ticket_id"])
		if ticket == nil {
			return CommandResult{}, fmt.Errorf("no ticket %v", input["ticket_id"])
		}
		if t.mode == "state-first" && ticket.state == "Open" {
			return exploreOrderConflict(), nil
		}
		if exploreOrderNumber(input["note"]) == 7 {
			ticket.priority = input["note"]
			return t.answered("noted", "exploreorder.desk.Noted", ticket.id), nil
		}
		if ticket.state != "Held" {
			return exploreOrderConflict(), nil
		}
		ticket.state = "Open"
		return t.answered("released", "exploreorder.desk.Released", ticket.id), nil
	default:
		return CommandResult{}, exploreOrderUnsupported{request.Command + " is not a command of explore-guards-first.yaml"}
	}
}

func (t *exploreOrderTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "exploreorder.desk.Tickets" {
		return ViewResult{}, exploreOrderUnsupported{request.View + " is not a view of explore-guards-first.yaml"}
	}
	rows := []Row{}
	for _, ticket := range t.tickets {
		rows = append(rows, Row{"ticket_id": ticket.id, "priority": ticket.priority, "state": ticket.state})
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreOrderTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreOrderTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreOrderUnsupported{"no external outcomes"}
}

func (t *exploreOrderTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreOrderUnsupported{"no redelivery"}
}

func (t *exploreOrderTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreOrderUnsupported{"no bindings"}
}
