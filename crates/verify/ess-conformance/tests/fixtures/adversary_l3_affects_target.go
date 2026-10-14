// An in-memory implementation of `adversary-l3-affects.yaml`, as the interpreter runs it.
//
// `adversary-l3-affects-target.mjs` is the same target in TypeScript.

package essconform

import "fmt"

type advL3AffUnsupported struct{ reason string }

func (e advL3AffUnsupported) Error() string { return e.reason }
func (e advL3AffUnsupported) Unwrap() error { return ErrUnsupported }

type advL3AffSeat struct {
	id         string
	team       Node
	lastReason Node
}

type advL3AffTarget struct {
	seats   []*advL3AffSeat
	version int
}

func newAdvL3AffTarget(string) *advL3AffTarget { return &advL3AffTarget{} }

func (t *advL3AffTarget) answered(outcome string, events []ObservedEvent) CommandResult {
	t.version++
	return CommandResult{Outcome: outcome, Consistency: fmt.Sprintf("v%d", t.version), DirectEvents: events}
}

func (t *advL3AffTarget) Identity() (Identity, error) {
	return Identity{Name: "adversary-l3-affects-target", Version: "1"}, nil
}

func (t *advL3AffTarget) BeginScenario(ScenarioContext) error {
	t.seats, t.version = nil, 0
	return nil
}

func (t *advL3AffTarget) EndScenario(ScenarioContext) error {
	t.seats, t.version = nil, 0
	return nil
}

func (t *advL3AffTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	switch request.Command {
	case "advaff.desk.Start":
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", len(t.seats)+1)
		t.seats = append(t.seats, &advL3AffSeat{id: id, team: input["team"], lastReason: "none"})
		return t.answered("started", []ObservedEvent{{Event: "advaff.desk.Started", Payload: map[string]Node{"seat_id": id}}}), nil
	case "advaff.desk.Note":
		for _, seat := range t.seats {
			if seat.id != input["seat_id"] {
				continue
			}
			for _, other := range t.seats {
				if other.team == seat.team {
					other.lastReason = input["reason"]
				}
			}
			return t.answered("noted", []ObservedEvent{{Event: "advaff.desk.Noted", Payload: map[string]Node{"seat_id": seat.id}}}), nil
		}
		return CommandResult{}, fmt.Errorf("no seat %v", input["seat_id"])
	default:
		return CommandResult{}, advL3AffUnsupported{request.Command + " is not a command of adversary-l3-affects.yaml"}
	}
}

func (t *advL3AffTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "advaff.desk.Seats" {
		return ViewResult{}, advL3AffUnsupported{request.View + " is not a view"}
	}
	rows := []Row{}
	for _, seat := range t.seats {
		rows = append(rows, Row{"seat_id": seat.id, "last_reason": seat.lastReason})
	}
	return ViewResult{Rows: rows}, nil
}

func (t *advL3AffTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *advL3AffTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return advL3AffUnsupported{"no external outcomes"}
}

func (t *advL3AffTarget) RedeliverEvent(RedeliveryRequest) error {
	return advL3AffUnsupported{"no redelivery"}
}

func (t *advL3AffTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, advL3AffUnsupported{"no bindings"}
}
