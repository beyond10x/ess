// An in-memory implementation of `explore-held-state-external.yaml`, a test double that arranges
// external outcomes the way the synthesized scenarios ask it to. It is copied into the package as
// `explore_external_target_test.go`, which `explore_external_driver_test.go` runs.
//
// Each `Nudge` it answers is appended to `<ESS_EXPLORE_ANSWERS>/<mode>.jsonl` as the state the
// ticket rested in (empty for an identity no ticket carries), the branch arranged for the command
// and the branch it answered, so the test can replay every step through the interpreter.
//
// `explore-held-state-external-target.mjs` is the same target in TypeScript, line for line; its
// header lists the modes.

package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
)

type exploreExtUnsupported struct{ reason string }

func (e exploreExtUnsupported) Error() string { return e.reason }
func (e exploreExtUnsupported) Unwrap() error { return ErrUnsupported }

type exploreExtTicket struct {
	id    string
	label Node
	state string
}

type exploreExtTarget struct {
	mode    string
	tickets map[string]*exploreExtTicket
	order   []string
	created int
	version int
	forced  map[string]string
}

func newExploreExtTarget(mode string) *exploreExtTarget {
	target := &exploreExtTarget{mode: mode}
	target.reset()
	return target
}

func (t *exploreExtTarget) reset() {
	t.tickets = map[string]*exploreExtTicket{}
	t.order = nil
	t.created = 0
	t.version = 0
	t.forced = map[string]string{}
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

// record appends one answered `Nudge` to the mode's answer log.
func (t *exploreExtTarget) record(state, answered string) {
	dir := os.Getenv("ESS_EXPLORE_ANSWERS")
	if dir == "" {
		return
	}
	name := t.mode
	if name == "" {
		name = "held-first"
	}
	line, _ := json.Marshal(map[string]string{"state": state, "arranged": t.forced["explorehx.desk.Nudge"], "answered": answered})
	file, err := os.OpenFile(filepath.Join(dir, name+".jsonl"), os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		panic(err)
	}
	defer file.Close()
	if _, err := file.Write(append(line, '\n')); err != nil {
		panic(err)
	}
}

func (t *exploreExtTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-held-state-external-target", Version: "1"}, nil
}

func (t *exploreExtTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreExtTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreExtTarget) nudge(id string) CommandResult {
	ticket, held := t.tickets[id]
	if !held {
		return exploreExtRefused("no-ticket", "explorehx.desk.NoTicket")
	}
	bounced := t.forced["explorehx.desk.Nudge"] == "bounced"
	if t.mode == "external-first" && bounced {
		return exploreExtRefused("bounced", "explorehx.desk.Bounced")
	}
	if ticket.state == "Parked" {
		return exploreExtRefused("parked", "explorehx.desk.NotOpen")
	}
	if bounced {
		return exploreExtRefused("bounced", "explorehx.desk.Bounced")
	}
	return t.answered("nudged", "explorehx.desk.Nudged", map[string]Node{"ticket_id": ticket.id})
}

func (t *exploreExtTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	switch request.Command {
	case "explorehx.desk.OpenTicket":
		t.created++
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", t.created)
		t.tickets[id] = &exploreExtTicket{id: id, label: input["label"], state: "Open"}
		t.order = append(t.order, id)
		return t.answered("opened", "explorehx.desk.Opened", map[string]Node{"ticket_id": id}), nil
	case "explorehx.desk.Park":
		id, _ := input["ticket_id"].(string)
		ticket, held := t.tickets[id]
		if !held {
			return CommandResult{}, fmt.Errorf("no ticket %s", id)
		}
		if ticket.state != "Open" {
			return exploreExtRefused("not-open", "explorehx.desk.NotOpen"), nil
		}
		ticket.state = "Parked"
		return t.answered("parked-now", "explorehx.desk.Parked", map[string]Node{"ticket_id": ticket.id}), nil
	case "explorehx.desk.Unpark":
		id, _ := input["ticket_id"].(string)
		ticket, held := t.tickets[id]
		if !held {
			return CommandResult{}, fmt.Errorf("no ticket %s", id)
		}
		if ticket.state != "Parked" {
			return exploreExtRefused("not-parked", "explorehx.desk.NotParked"), nil
		}
		ticket.state = "Open"
		return t.answered("unparked", "explorehx.desk.Unparked", map[string]Node{"ticket_id": ticket.id}), nil
	case "explorehx.desk.Nudge":
		id, _ := input["ticket_id"].(string)
		state := ""
		if ticket, held := t.tickets[id]; held {
			state = ticket.state
		}
		result := t.nudge(id)
		t.record(state, result.Outcome)
		return result, nil
	default:
		return CommandResult{}, exploreExtUnsupported{request.Command + " is not a command of explore-held-state-external.yaml"}
	}
}

func (t *exploreExtTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "explorehx.desk.Tickets" {
		return ViewResult{}, exploreExtUnsupported{request.View + " is not a view of explore-held-state-external.yaml"}
	}
	rows := []Row{}
	for _, id := range t.order {
		ticket := t.tickets[id]
		rows = append(rows, Row{"ticket_id": ticket.id, "label": ticket.label, "state": ticket.state})
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreExtTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreExtTarget) ConfigureExternalOutcome(control ExternalOutcomeControl) error {
	t.forced[control.Command] = control.Outcome
	return nil
}

func (t *exploreExtTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreExtUnsupported{"no redelivery"}
}

func (t *exploreExtTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreExtUnsupported{"no bindings"}
}
