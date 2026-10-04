package essconform

// Adversary pass 1 for beyond10x/ess#273: the desk of `captured-identities.yaml`, with faults the
// unit's own fixture does not apply. ESS_IDENTITY_FAULT selects one: a closed ticket naming the
// first queue the scenario opened instead of its own (two instances of one entity type), the
// Optional identity published as null, the ticket identity zeroed, or the ticket and queue
// identities swapped.

import (
	"fmt"
	"os"
	"testing"
)

type adversaryDesk struct {
	minted     int
	firstQueue string
	tickets    map[string]string
	closed     map[string]bool
	mode       string
}

func (b *adversaryDesk) mint() string {
	b.minted++
	return fmt.Sprintf("00000000-0000-4000-8000-%012d", b.minted)
}

func (*adversaryDesk) Identity() (Identity, error) { return Identity{Name: "desk", Version: "1"}, nil }
func (b *adversaryDesk) BeginScenario(ScenarioContext) error {
	b.tickets = map[string]string{}
	b.closed = map[string]bool{}
	b.firstQueue = ""
	return nil
}
func (*adversaryDesk) EndScenario(ScenarioContext) error { return nil }
func (*adversaryDesk) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return fmt.Errorf("no external outcomes")
}
func (b *adversaryDesk) ExecuteCommand(r CommandRequest) (CommandResult, error) {
	switch r.Command {
	case "desk.tickets.OpenQueue":
		id := b.mint()
		if b.firstQueue == "" {
			b.firstQueue = id
		}
		return CommandResult{Outcome: "opened", DirectEvents: []ObservedEvent{{Event: "desk.tickets.QueueOpened", Payload: map[string]Node{"queue_id": id, "label": r.Input["label"]}}}}, nil
	case "desk.tickets.FileTicket":
		id := b.mint()
		queue, _ := r.Input["queue_id"].(string)
		b.tickets[id] = queue
		return CommandResult{Outcome: "filed", DirectEvents: []ObservedEvent{{Event: "desk.tickets.TicketFiled", Payload: map[string]Node{"ticket_id": id, "queue_id": queue}}}}, nil
	case "desk.tickets.CloseTicket":
		ticket, _ := r.Input["ticket_id"].(string)
		queue, ok := b.tickets[ticket]
		if !ok || b.closed[ticket] {
			return CommandResult{Outcome: "wrong-state", Error: "desk.tickets.NotOpen"}, nil
		}
		b.closed[ticket] = true
		payload := map[string]Node{"ticket_id": ticket, "closed": ticket, "queue_id": queue}
		switch b.mode {
		case "swap-queue":
			payload["queue_id"] = b.firstQueue
		case "null-closed":
			payload["closed"] = nil
		case "zero-ticket":
			payload["ticket_id"] = "00000000-0000-0000-0000-000000000000"
		case "swap-pair":
			payload["ticket_id"], payload["queue_id"] = queue, ticket
		}
		return CommandResult{Outcome: "closed", DirectEvents: []ObservedEvent{{Event: "desk.tickets.TicketClosed", Payload: payload}}}, nil
	}
	return CommandResult{}, fmt.Errorf("undeclared command %s", r.Command)
}
func (*adversaryDesk) QueryView(ViewRequest) (ViewResult, error) {
	return ViewResult{}, fmt.Errorf("no views")
}
func (*adversaryDesk) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}
func (*adversaryDesk) RedeliverEvent(RedeliveryRequest) error { return fmt.Errorf("no bindings") }
func (*adversaryDesk) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}
func TestAdversary273(t *testing.T) {
	Run(t, func() Target { return &adversaryDesk{mode: os.Getenv("ESS_IDENTITY_FAULT")} })
}
