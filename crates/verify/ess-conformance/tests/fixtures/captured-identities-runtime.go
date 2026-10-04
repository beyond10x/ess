package essconform

// A desk that files tickets into queues (`captured-identities.yaml`). ESS_IDENTITY_FAULT selects a
// faulty implementation: one that drops the Optional identity a closed ticket names, or one that
// publishes another identity in place of the ticket's or its queue's.

import (
	"fmt"
	"os"
	"testing"
)

type deskBackend struct {
	minted  int
	tickets map[string]string
	closed  map[string]bool
	mode    string
}

func (b *deskBackend) mint() string {
	b.minted++
	return fmt.Sprintf("00000000-0000-4000-8000-%012d", b.minted)
}

func (*deskBackend) Identity() (Identity, error) { return Identity{Name: "desk", Version: "1"}, nil }
func (b *deskBackend) BeginScenario(ScenarioContext) error {
	b.tickets = map[string]string{}
	b.closed = map[string]bool{}
	return nil
}
func (*deskBackend) EndScenario(ScenarioContext) error { return nil }
func (*deskBackend) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return fmt.Errorf("no external outcomes")
}
func (b *deskBackend) ExecuteCommand(r CommandRequest) (CommandResult, error) {
	switch r.Command {
	case "desk.tickets.OpenQueue":
		id := b.mint()
		return CommandResult{Outcome: "opened", DirectEvents: []ObservedEvent{{Event: "desk.tickets.QueueOpened", Payload: map[string]Node{"queue_id": id, "label": r.Input["label"]}}}}, nil
	case "desk.tickets.FileTicket":
		id := b.mint()
		queue, _ := r.Input["queue_id"].(string)
		b.tickets[id] = queue
		if b.mode == "swap-filed" {
			queue = b.mint()
		}
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
		case "drop-subject":
			delete(payload, "closed")
		case "swap-input":
			payload["ticket_id"] = b.mint()
		case "swap-related":
			payload["queue_id"] = b.mint()
		}
		return CommandResult{Outcome: "closed", DirectEvents: []ObservedEvent{{Event: "desk.tickets.TicketClosed", Payload: payload}}}, nil
	}
	return CommandResult{}, fmt.Errorf("undeclared command %s", r.Command)
}
func (*deskBackend) QueryView(ViewRequest) (ViewResult, error) {
	return ViewResult{}, fmt.Errorf("no views")
}
func (*deskBackend) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) { return nil, nil }
func (*deskBackend) RedeliverEvent(RedeliveryRequest) error                         { return fmt.Errorf("no bindings") }
func (*deskBackend) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}
func TestCapturedIdentities(t *testing.T) {
	Run(t, func() Target { return &deskBackend{mode: os.Getenv("ESS_IDENTITY_FAULT")} })
}
