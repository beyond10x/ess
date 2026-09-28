package essconform

import (
	"fmt"
	"os"
	"sort"
	"testing"
)

// dialerBackend is the #179 dialer written in Go: `SetLead` copies four input leaves into a nested
// `lead` struct and generates the fifth. ESS_TEST_DIALER_MODE selects a mutation.
type dialerBackend struct {
	mode     string
	rows     map[string]Row
	sequence int
}

func (b *dialerBackend) Identity() (Identity, error) {
	return Identity{Name: "dialer-go-fixture", Version: "1"}, nil
}
func (b *dialerBackend) BeginScenario(ScenarioContext) error { b.rows = map[string]Row{}; return nil }
func (b *dialerBackend) EndScenario(ScenarioContext) error   { return nil }
func (b *dialerBackend) ExecuteCommand(r CommandRequest) (CommandResult, error) {
	b.sequence++
	token := fmt.Sprintf("seq:%d", b.sequence)
	switch r.Command {
	case "demo.dialer.Join":
		id := fmt.Sprintf("agent-%d", b.sequence)
		b.rows[id] = Row{"agent_id": id, "lead": nil}
		return CommandResult{Outcome: "joined", Consistency: token, DirectEvents: []ObservedEvent{
			{Event: "demo.dialer.Joined", Payload: map[string]Node{"agent_id": id}},
		}}, nil
	case "demo.dialer.SetLead":
		id, _ := r.Input["agent_id"].(string)
		row, ok := b.rows[id]
		if !ok {
			return CommandResult{Consistency: token}, nil
		}
		lead := func() map[string]Node {
			return map[string]Node{
				"id": r.Input["lead_id"], "uid": r.Input["lead_uid"], "number": r.Input["lead_number"],
				"rank": 0, "data": r.Input["lead_data"],
			}
		}
		stored, published := lead(), lead()
		if b.mode == "drops-number-from-the-row" {
			delete(stored, "number")
		}
		if b.mode == "wrong-number-on-the-event" {
			published["number"] = r.Input["lead_data"]
		}
		row["lead"] = stored
		return CommandResult{Outcome: "lead-set", Consistency: token, DirectEvents: []ObservedEvent{
			{Event: "demo.dialer.LeadSet", Payload: map[string]Node{"agent_id": id, "lead": published}},
		}}, nil
	}
	return CommandResult{}, fmt.Errorf("unexpected command %s", r.Command)
}
func (b *dialerBackend) QueryView(ViewRequest) (ViewResult, error) {
	ids := make([]string, 0, len(b.rows))
	for id := range b.rows {
		ids = append(ids, id)
	}
	sort.Strings(ids)
	rows := make([]Row, 0, len(ids))
	for _, id := range ids {
		rows = append(rows, b.rows[id])
	}
	return ViewResult{Rows: rows}, nil
}
func (b *dialerBackend) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}
func (b *dialerBackend) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return fmt.Errorf("nothing here is externally decided")
}
func (b *dialerBackend) RedeliverEvent(RedeliveryRequest) error { return fmt.Errorf("no bindings") }
func (b *dialerBackend) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}

func TestDialerRuntime(t *testing.T) {
	Run(t, func() Target { return &dialerBackend{mode: os.Getenv("ESS_TEST_DIALER_MODE")} })
}
