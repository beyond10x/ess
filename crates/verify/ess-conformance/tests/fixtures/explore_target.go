// An in-memory implementation of `explore.yaml`, and the engine mutants the explorer must catch.
//
// `explore-target.mjs` is the same target in TypeScript, line for line; its header lists the
// mutants `ESS_EXPLORE_MUTANT` selects and what `ESS_EXPLORE_GRADE` decides.

package essconform

import (
	"fmt"
	"os"
	"sort"
)

const (
	exploreFixtureOpen     = "Open"
	exploreFixtureHeld     = "Held"
	exploreFixtureClosed   = "Closed"
	exploreFixtureConflict = "explore.desk.TicketStateConflict"
)

// exploreFixtureUnsupported is ErrUnsupported with the reason as its whole text, so the reason the
// explorer records reads the same in both lanes.
type exploreFixtureUnsupported struct{ reason string }

func (e exploreFixtureUnsupported) Error() string { return e.reason }
func (e exploreFixtureUnsupported) Unwrap() error { return ErrUnsupported }

type exploreFixtureTicket struct {
	id       string
	items    Node
	priority Node
	state    string
	visited  map[string]bool
	returns  int
}

type exploreFixtureTarget struct {
	mutant    string
	grade     string
	tickets   map[string]*exploreFixtureTicket
	order     []string
	created   int
	first     string
	version   int
	projected []Row
}

func newExploreFixtureTarget(mutant, grade string) *exploreFixtureTarget {
	if mutant == "" {
		mutant = os.Getenv("ESS_EXPLORE_MUTANT")
	}
	if grade == "" {
		grade = os.Getenv("ESS_EXPLORE_GRADE")
	}
	if grade == "" {
		grade = "low"
	}
	target := &exploreFixtureTarget{mutant: mutant, grade: grade}
	target.reset()
	return target
}

func (t *exploreFixtureTarget) reset() {
	t.tickets = map[string]*exploreFixtureTicket{}
	t.order = nil
	t.created = 0
	t.first = ""
	t.version = 0
	t.projected = []Row{}
}

func (t *exploreFixtureTarget) row(ticket *exploreFixtureTicket) Row {
	return Row{"ticket_id": ticket.id, "items": ticket.items, "priority": ticket.priority}
}

func (t *exploreFixtureTarget) answered(outcome, event string, payload map[string]Node) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: payload}},
	}
}

func exploreFixtureRefused(error string) CommandResult {
	return CommandResult{Outcome: "wrong-state", Error: error, DirectEvents: []ObservedEvent{}}
}

// move is a move into `to`, unless the third-return mutant refuses it.
func (t *exploreFixtureTarget) move(ticket *exploreFixtureTicket, to string) bool {
	if ticket.visited[to] {
		ticket.returns++
		if t.mutant == "third-return-refused" && ticket.returns == 3 {
			return false
		}
	}
	ticket.visited[to] = true
	ticket.state = to
	return true
}

func exploreFixtureNumber(value Node) float64 {
	number, _ := value.(float64)
	return number
}

func (t *exploreFixtureTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-target", Version: "1"}, nil
}

func (t *exploreFixtureTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreFixtureTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreFixtureTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	id, _ := input["ticket_id"].(string)
	ticket := t.tickets[id]
	switch request.Command {
	case "explore.desk.OpenTicket":
		if !(exploreFixtureNumber(input["items"]) >= 0) {
			return CommandResult{Outcome: "rejected", Error: "explore.desk.InvalidItems"}, nil
		}
		t.created++
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", t.created)
		if t.created == 1 {
			t.first = id
		}
		if t.mutant == "fourth-create-reuses-first" && t.created == 4 {
			id = t.first
		}
		if _, known := t.tickets[id]; !known {
			t.order = append(t.order, id)
		}
		t.tickets[id] = &exploreFixtureTicket{
			id: id, items: input["items"], priority: float64(0), state: exploreFixtureOpen,
			visited: map[string]bool{exploreFixtureOpen: true},
		}
		return t.answered("opened", "explore.desk.TicketOpened", map[string]Node{"ticket_id": id, "items": input["items"]}), nil
	case "explore.desk.HoldTicket":
		if ticket.state != exploreFixtureOpen {
			if t.mutant == "wrong-state-applies-sets" {
				ticket.priority = input["priority"]
			}
			return exploreFixtureRefused(exploreFixtureConflict), nil
		}
		if !t.move(ticket, exploreFixtureHeld) {
			return exploreFixtureRefused(exploreFixtureConflict), nil
		}
		ticket.priority = input["priority"]
		return t.answered("held", "explore.desk.TicketHeld", map[string]Node{"ticket_id": ticket.id}), nil
	case "explore.desk.ReleaseTicket":
		if ticket.state != exploreFixtureHeld || !t.move(ticket, exploreFixtureOpen) {
			return exploreFixtureRefused(exploreFixtureConflict), nil
		}
		return t.answered("released", "explore.desk.TicketReleased", map[string]Node{"ticket_id": ticket.id}), nil
	case "explore.desk.CloseTicket":
		if t.mutant == "close-unsupported" {
			return CommandResult{}, exploreFixtureUnsupported{"CloseTicket is not exposed"}
		}
		if ticket.state == exploreFixtureClosed || !t.move(ticket, exploreFixtureClosed) {
			return exploreFixtureRefused(exploreFixtureConflict), nil
		}
		return t.answered("closed", "explore.desk.TicketClosed", map[string]Node{"ticket_id": ticket.id}), nil
	case "explore.desk.Restock":
		if !(exploreFixtureNumber(input["items"]) >= 0) {
			return CommandResult{Outcome: "refused", Error: "explore.desk.InvalidItems"}, nil
		}
		ticket.items = input["items"]
		return t.answered("restocked", "explore.desk.TicketRestocked", map[string]Node{"ticket_id": ticket.id, "items": input["items"]}), nil
	case "explore.desk.Grade":
		score := exploreFixtureNumber(input["score"])
		if score > 60 || (score >= 50 && t.grade == "high") {
			return t.answered("high", "explore.desk.GradedHigh", map[string]Node{"score": input["score"]}), nil
		}
		if score >= 10 {
			return t.answered("low", "explore.desk.GradedLow", map[string]Node{"score": input["score"]}), nil
		}
		return CommandResult{Outcome: "ungraded", Error: "explore.desk.Ungradable"}, nil
	default:
		return CommandResult{}, exploreFixtureUnsupported{request.Command + " is not a command of explore.yaml"}
	}
}

func (t *exploreFixtureTarget) QueryView(request ViewRequest) (ViewResult, error) {
	all := []*exploreFixtureTicket{}
	for _, id := range t.order {
		all = append(all, t.tickets[id])
	}
	switch request.View {
	case "explore.desk.OpenTickets":
		rows := []Row{}
		for _, ticket := range all {
			if ticket.state == exploreFixtureOpen {
				rows = append(rows, t.row(ticket))
			}
		}
		return ViewResult{Rows: rows}, nil
	case "explore.desk.TicketsByItems":
		shown := t.projected
		fresh := []Row{}
		for _, ticket := range all {
			fresh = append(fresh, t.row(ticket))
		}
		sort.SliceStable(fresh, func(i, j int) bool {
			return exploreFixtureNumber(fresh[i]["items"]) > exploreFixtureNumber(fresh[j]["items"])
		})
		t.projected = fresh
		if t.mutant == "view-cap-5" && len(shown) > 5 {
			shown = shown[:5]
		}
		return ViewResult{Rows: shown}, nil
	default:
		return ViewResult{}, exploreFixtureUnsupported{request.View + " is not a view of explore.yaml"}
	}
}

func (t *exploreFixtureTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreFixtureTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreFixtureUnsupported{"no external outcome"}
}

func (t *exploreFixtureTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreFixtureUnsupported{"no redelivery"}
}

func (t *exploreFixtureTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreFixtureUnsupported{"no bindings"}
}
