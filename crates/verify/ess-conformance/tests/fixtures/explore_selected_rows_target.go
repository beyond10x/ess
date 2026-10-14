// An in-memory implementation of `explore-selected-rows.yaml` (beyond10x/ess#538).
//
// `explore-selected-rows-target.mjs` is the same target in TypeScript, line for line; its header
// lists the modes.

package essconform

import "fmt"

type exploreRowsUnsupported struct{ reason string }

func (e exploreRowsUnsupported) Error() string { return e.reason }
func (e exploreRowsUnsupported) Unwrap() error { return ErrUnsupported }

type exploreRowsSeat struct {
	id         string
	state      string
	team       Node
	lastReason Node
	level      Node
}

type exploreRowsTarget struct {
	mode    string
	seats   []*exploreRowsSeat
	version int
}

func newExploreRowsTarget(mode string) *exploreRowsTarget {
	return &exploreRowsTarget{mode: mode}
}

func (t *exploreRowsTarget) reset() {
	t.seats = nil
	t.version = 0
}

func (t *exploreRowsTarget) find(id Node) *exploreRowsSeat {
	for _, seat := range t.seats {
		if seat.id == id {
			return seat
		}
	}
	return nil
}

func (t *exploreRowsTarget) answered(outcome string, events []ObservedEvent) CommandResult {
	t.version++
	return CommandResult{Outcome: outcome, Consistency: fmt.Sprintf("v%d", t.version), DirectEvents: events}
}

func (t *exploreRowsTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-selected-rows-target", Version: "1"}, nil
}

func (t *exploreRowsTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreRowsTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreRowsTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	seat := t.find(input["seat_id"])
	switch request.Command {
	case "explorerows.desk.Start":
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", len(t.seats)+1)
		t.seats = append(t.seats, &exploreRowsSeat{id: id, state: "Open", team: input["team"], lastReason: "none", level: input["level"]})
		return t.answered("started", []ObservedEvent{{Event: "explorerows.desk.Started", Payload: map[string]Node{"seat_id": id}}}), nil
	case "explorerows.desk.Dial":
		if seat == nil {
			return CommandResult{}, fmt.Errorf("no seat %v", input["seat_id"])
		}
		seat.level = input["dial_level"]
		if t.mode == "dial-adds-one" {
			seat.level = input["dial_level"].(float64) + 1
		}
		return t.answered("dialled", []ObservedEvent{{Event: "explorerows.desk.Dialled", Payload: map[string]Node{"seat_id": seat.id, "level": input["dial_level"]}}}), nil
	case "explorerows.desk.Note":
		if seat == nil {
			return CommandResult{}, fmt.Errorf("no seat %v", input["seat_id"])
		}
		seat.lastReason = input["reason"]
		return t.answered("noted", []ObservedEvent{{Event: "explorerows.desk.Noted", Payload: map[string]Node{"seat_id": seat.id}}}), nil
	case "explorerows.desk.Close":
		closed := 0
		for _, each := range t.seats {
			selected := each.team == input["team"] || t.mode == "close-every-team"
			if !selected || each.state != "Open" {
				continue
			}
			each.state = "Closed"
			if !(t.mode == "close-skips-one" && closed == 0) {
				each.lastReason = "hungup"
			}
			closed++
		}
		return t.answered("closed", []ObservedEvent{{Event: "explorerows.desk.TeamClosed", Payload: map[string]Node{"team": input["team"], "closed": float64(closed)}}}), nil
	case "explorerows.desk.Reopen":
		if seat == nil {
			return CommandResult{}, fmt.Errorf("no seat %v", input["seat_id"])
		}
		if seat.state != "Closed" {
			return CommandResult{Outcome: "not-closed", Error: "explorerows.desk.NotClosed", DirectEvents: []ObservedEvent{}}, nil
		}
		seat.state = "Open"
		return t.answered("reopened", []ObservedEvent{{Event: "explorerows.desk.Reopened", Payload: map[string]Node{"seat_id": seat.id}}}), nil
	default:
		return CommandResult{}, exploreRowsUnsupported{request.Command + " is not a command of explore-selected-rows.yaml"}
	}
}

func (t *exploreRowsTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "explorerows.desk.Seats" {
		return ViewResult{}, exploreRowsUnsupported{request.View + " is not a view of explore-selected-rows.yaml"}
	}
	rows := []Row{}
	for _, seat := range t.seats {
		rows = append(rows, Row{"seat_id": seat.id, "state": seat.state, "team": seat.team, "last_reason": seat.lastReason, "level": seat.level})
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreRowsTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreRowsTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreRowsUnsupported{"no external outcomes"}
}

func (t *exploreRowsTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreRowsUnsupported{"no redelivery"}
}

func (t *exploreRowsTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreRowsUnsupported{"no bindings"}
}
