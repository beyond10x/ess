// An in-memory implementation of `explore-stored-rows.yaml` (beyond10x/ess#221, #223), deciding
// each command as the specification does.
//
// `explore-stored-rows-target.mjs` is the same target in TypeScript, line for line; its header
// lists the modes.

package essconform

import (
	"fmt"
	"unicode/utf8"
)

type exploreRowsUnsupported struct{ reason string }

func (e exploreRowsUnsupported) Error() string { return e.reason }
func (e exploreRowsUnsupported) Unwrap() error { return ErrUnsupported }

type exploreRowsKey struct {
	id     string
	state  string
	secret Node
	label  Node
	plan   string
	uses   Node
}

type exploreRowsTarget struct {
	mode    string
	keys    []*exploreRowsKey
	owners  []string
	version int
}

func newExploreRowsTarget(mode string) *exploreRowsTarget {
	return &exploreRowsTarget{mode: mode}
}

func (t *exploreRowsTarget) reset() {
	t.keys = nil
	t.owners = nil
	t.version = 0
}

func (t *exploreRowsTarget) find(id Node) *exploreRowsKey {
	for _, key := range t.keys {
		if key.id == id {
			return key
		}
	}
	return nil
}

func exploreRowsRefused(outcome, err string) CommandResult {
	return CommandResult{Outcome: outcome, Error: err, DirectEvents: []ObservedEvent{}}
}

func (t *exploreRowsTarget) answered(outcome, event, field string, id Node) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: map[string]Node{field: id}}},
	}
}

func exploreRowsNumber(value any) float64 {
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

func (t *exploreRowsTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-stored-rows-target", Version: "1"}, nil
}

func (t *exploreRowsTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreRowsTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreRowsTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	switch request.Command {
	case "exploredraw.keys.Bind":
		secret, _ := input["secret"].(string)
		length := utf8.RuneCountInString(secret)
		if t.mode == "label-plain" {
			if label, _ := input["label"].(string); label != "" && label != "a" && label != "b" {
				return CommandResult{}, fmt.Errorf("label %q is not one of the plain texts", label)
			}
		}
		if length < 12 || (t.mode == "count-inclusive" && length == 12) {
			return exploreRowsRefused("too-short", "exploredraw.keys.TooShort"), nil
		}
		if t.mode == "uses-seven" && exploreRowsNumber(input["uses"]) == 7 {
			return exploreRowsRefused("too-short", "exploredraw.keys.TooShort"), nil
		}
		id, _ := input["key_id"].(string)
		if existing := t.find(id); existing != nil {
			if t.mode != "existing-ignored" {
				return exploreRowsRefused("already-bound", "exploredraw.keys.AlreadyBound"), nil
			}
			existing.secret, existing.label, existing.plan, existing.uses = input["secret"], input["label"], "Basic", input["uses"]
			return t.answered("bound", "exploredraw.keys.Bound", "key_id", id), nil
		}
		t.keys = append(t.keys, &exploreRowsKey{id: id, state: "Active", secret: input["secret"], label: input["label"], plan: "Basic", uses: input["uses"]})
		return t.answered("bound", "exploredraw.keys.Bound", "key_id", id), nil
	case "exploredraw.keys.Rotate":
		key := t.find(input["key_id"])
		if key == nil {
			return exploreRowsRefused("unknown-key", "exploredraw.keys.UnknownKey"), nil
		}
		if key.secret == input["secret"] {
			return exploreRowsRefused("same-secret", "exploredraw.keys.SameSecret"), nil
		}
		key.secret = input["secret"]
		return t.answered("rotated", "exploredraw.keys.Rotated", "key_id", key.id), nil
	case "exploredraw.keys.Upgrade":
		key := t.find(input["key_id"])
		if key == nil {
			return CommandResult{}, fmt.Errorf("no key %v", input["key_id"])
		}
		if key.plan == "Basic" || t.mode == "field-ignored" {
			key.plan = "Pro"
			return t.answered("upgraded", "exploredraw.keys.Upgraded", "key_id", key.id), nil
		}
		return t.answered("kept", "exploredraw.keys.Kept", "key_id", key.id), nil
	case "exploredraw.keys.Revoke":
		key := t.find(input["key_id"])
		if key == nil {
			return CommandResult{}, fmt.Errorf("no key %v", input["key_id"])
		}
		if key.state == "Revoked" && t.mode != "state-ignored" {
			return exploreRowsRefused("already-revoked", "exploredraw.keys.AlreadyRevoked"), nil
		}
		key.state = "Revoked"
		return t.answered("revoked", "exploredraw.keys.Revoked", "key_id", key.id), nil
	case "exploredraw.keys.Resume":
		key := t.find(input["key_id"])
		if key == nil {
			return CommandResult{}, fmt.Errorf("no key %v", input["key_id"])
		}
		switch {
		case key.state == "Suspended" || (key.state == "Active" && t.mode == "changes-ignored"):
			key.state = "Active"
			return t.answered("resumed", "exploredraw.keys.Resumed", "key_id", key.id), nil
		case key.state == "Active":
			return t.answered("restated", "exploredraw.keys.Restated", "key_id", key.id), nil
		default:
			return exploreRowsRefused("stuck", "exploredraw.keys.Stuck"), nil
		}
	case "exploredraw.keys.Suspend":
		key := t.find(input["key_id"])
		if key == nil {
			return CommandResult{}, fmt.Errorf("no key %v", input["key_id"])
		}
		if key.state != "Active" {
			return exploreRowsRefused("not-active", "exploredraw.keys.NotActive"), nil
		}
		key.state = "Suspended"
		return t.answered("suspended", "exploredraw.keys.Suspended", "key_id", key.id), nil
	case "exploredraw.keys.Use":
		key := t.find(input["key_id"])
		if key == nil {
			return CommandResult{}, fmt.Errorf("no key %v", input["key_id"])
		}
		if exploreRowsNumber(key.uses) < exploreRowsNumber(input["amount"]) && t.mode != "predicate-ignored" {
			return exploreRowsRefused("exhausted", "exploredraw.keys.Exhausted"), nil
		}
		if t.mode == "bulk" && exploreRowsNumber(input["amount"]) > 100 {
			return t.answered("bulk", "exploredraw.keys.Used", "key_id", key.id), nil
		}
		return t.answered("used", "exploredraw.keys.Used", "key_id", key.id), nil
	case "exploredraw.keys.Enrol":
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", len(t.owners)+1)
		t.owners = append(t.owners, id)
		return t.answered("enrolled", "exploredraw.keys.Enrolled", "owner_id", id), nil
	default:
		return CommandResult{}, exploreRowsUnsupported{request.Command + " is not a command this target serves"}
	}
}

func (t *exploreRowsTarget) QueryView(request ViewRequest) (ViewResult, error) {
	rows := []Row{}
	switch request.View {
	case "exploredraw.keys.Keys":
		for _, key := range t.keys {
			rows = append(rows, Row{"key_id": key.id, "state": key.state, "secret": key.secret, "label": key.label, "plan": key.plan, "uses": key.uses})
		}
	case "exploredraw.keys.Owners":
		for _, owner := range t.owners {
			rows = append(rows, Row{"owner_id": owner, "state": "Enrolled"})
		}
	default:
		return ViewResult{}, exploreRowsUnsupported{request.View + " is not a view of explore-stored-rows.yaml"}
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
