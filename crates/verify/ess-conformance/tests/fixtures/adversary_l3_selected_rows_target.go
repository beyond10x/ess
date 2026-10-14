// An in-memory implementation of `adversary-l3-selected-rows.yaml`, as the interpreter runs it.
//
// `adversary-l3-selected-rows-target.mjs` is the same target in TypeScript; its header says what
// each command does.

package essconform

import "fmt"

type advL3Unsupported struct{ reason string }

func (e advL3Unsupported) Error() string { return e.reason }
func (e advL3Unsupported) Unwrap() error { return ErrUnsupported }

type advL3Seat struct {
	id         string
	state      string
	tag        Node
	lastReason Node
}

type advL3Desk struct {
	id    string
	state string
	zone  Node
}

type advL3Target struct {
	mode    string
	seats   []*advL3Seat
	desks   []*advL3Desk
	created int
	version int
}

func newAdvL3Target(mode string) *advL3Target { return &advL3Target{mode: mode} }

func (t *advL3Target) reset() {
	t.seats, t.desks, t.created, t.version = nil, nil, 0, 0
}

func (t *advL3Target) mint() string {
	t.created++
	return fmt.Sprintf("00000000-0000-4000-8000-%012d", t.created)
}

func (t *advL3Target) answered(outcome string, events []ObservedEvent) CommandResult {
	t.version++
	return CommandResult{Outcome: outcome, Consistency: fmt.Sprintf("v%d", t.version), DirectEvents: events}
}

func (t *advL3Target) Identity() (Identity, error) {
	return Identity{Name: "adversary-l3-selected-rows-target", Version: "1"}, nil
}

func (t *advL3Target) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *advL3Target) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *advL3Target) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	switch request.Command {
	case "advrows.desk.Start":
		id := t.mint()
		t.seats = append(t.seats, &advL3Seat{id: id, state: "Open", lastReason: "none"})
		return t.answered("started", []ObservedEvent{{Event: "advrows.desk.Started", Payload: map[string]Node{"seat_id": id}}}), nil
	case "advrows.desk.Tag":
		for _, seat := range t.seats {
			if seat.id == input["seat_id"] {
				seat.tag = input["tag"]
				return t.answered("tagged", []ObservedEvent{{Event: "advrows.desk.Tagged", Payload: map[string]Node{"seat_id": seat.id}}}), nil
			}
		}
		return CommandResult{}, fmt.Errorf("no seat %v", input["seat_id"])
	case "advrows.desk.CloseTagged":
		for _, seat := range t.seats {
			selected := seat.tag != nil && seat.tag == input["team"]
			if t.mode == "closes-untagged" && seat.tag == nil {
				selected = true
			}
			if !selected || seat.state != "Open" {
				continue
			}
			seat.state = "Closed"
			seat.lastReason = "hungup"
		}
		return t.answered("closed", []ObservedEvent{{Event: "advrows.desk.TagClosed", Payload: map[string]Node{"team": input["team"]}}}), nil
	case "advrows.desk.OpenDesk":
		id := t.mint()
		t.desks = append(t.desks, &advL3Desk{id: id, state: "Idle"})
		return t.answered("opened", []ObservedEvent{{Event: "advrows.desk.DeskOpened", Payload: map[string]Node{"desk_id": id}}}), nil
	case "advrows.desk.Activate":
		for _, desk := range t.desks {
			if desk.id == input["desk_id"] {
				if desk.state != "Idle" {
					return CommandResult{}, fmt.Errorf("only an idle desk activates")
				}
				desk.state = "Active"
				desk.zone = input["zone"]
				return t.answered("activated", []ObservedEvent{{Event: "advrows.desk.Activated", Payload: map[string]Node{"desk_id": desk.id}}}), nil
			}
		}
		return CommandResult{}, fmt.Errorf("no desk %v", input["desk_id"])
	case "advrows.desk.Finish":
		for _, desk := range t.desks {
			if desk.state != "Active" || desk.zone != input["zone"] {
				continue
			}
			desk.state = "Done"
		}
		return t.answered("finished", []ObservedEvent{{Event: "advrows.desk.Finished", Payload: map[string]Node{"zone": input["zone"]}}}), nil
	default:
		return CommandResult{}, advL3Unsupported{request.Command + " is not a command of adversary-l3-selected-rows.yaml"}
	}
}

func (t *advL3Target) QueryView(request ViewRequest) (ViewResult, error) {
	rows := []Row{}
	switch request.View {
	case "advrows.desk.Seats":
		for _, seat := range t.seats {
			rows = append(rows, Row{"seat_id": seat.id, "state": seat.state, "last_reason": seat.lastReason})
		}
	case "advrows.desk.Desks":
		for _, desk := range t.desks {
			rows = append(rows, Row{"desk_id": desk.id, "state": desk.state})
		}
	default:
		return ViewResult{}, advL3Unsupported{request.View + " is not a view of adversary-l3-selected-rows.yaml"}
	}
	return ViewResult{Rows: rows}, nil
}

func (t *advL3Target) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *advL3Target) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return advL3Unsupported{"no external outcomes"}
}

func (t *advL3Target) RedeliverEvent(RedeliveryRequest) error {
	return advL3Unsupported{"no redelivery"}
}

func (t *advL3Target) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, advL3Unsupported{"no bindings"}
}
