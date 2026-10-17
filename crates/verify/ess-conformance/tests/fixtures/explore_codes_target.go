// An in-memory implementation of `explore-codes.yaml` (beyond10x/ess#582, #583). It reads its own
// wall clock for every guard over `now`, as a deployed implementation does.
//
// `explore-codes-target.mjs` is the same target in TypeScript, line for line; its header lists the
// modes.

package essconform

import (
	"fmt"
	"time"
)

type exploreCodesUnsupported struct{ reason string }

func (e exploreCodesUnsupported) Error() string { return e.reason }
func (e exploreCodesUnsupported) Unwrap() error { return ErrUnsupported }

type exploreCodesCode struct {
	id      string
	expiry  Node
	deleted bool
}

type exploreCodesTarget struct {
	mode    string
	codes   []*exploreCodesCode
	version int
}

func newExploreCodesTarget(mode string) *exploreCodesTarget {
	return &exploreCodesTarget{mode: mode}
}

func (t *exploreCodesTarget) reset() {
	t.codes = nil
	t.version = 0
}

func (t *exploreCodesTarget) find(id Node) *exploreCodesCode {
	for _, code := range t.codes {
		if code.id == id {
			return code
		}
	}
	return nil
}

func exploreCodesInstant(value Node) time.Time {
	at, err := time.Parse(time.RFC3339, fmt.Sprint(value))
	if err != nil {
		panic(fmt.Sprintf("the explorer sent %v, which is no RFC 3339 instant", value))
	}
	return at
}

func exploreCodesRefused(outcome, err string) CommandResult {
	return CommandResult{Outcome: outcome, Error: err, DirectEvents: []ObservedEvent{}}
}

func (t *exploreCodesTarget) answered(outcome, event string, payload map[string]Node) CommandResult {
	t.version++
	return CommandResult{
		Outcome:      outcome,
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{{Event: event, Payload: payload}},
	}
}

func (t *exploreCodesTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-codes-target", Version: "1"}, nil
}

func (t *exploreCodesTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreCodesTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreCodesTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	now := time.Now()
	switch request.Command {
	case "explorecodes.auth.StoreCode":
		exploreCodesInstant(input["expiry"])
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", len(t.codes)+1)
		t.codes = append(t.codes, &exploreCodesCode{id: id, expiry: input["expiry"]})
		return t.answered("stored", "explorecodes.auth.Stored", map[string]Node{"code_id": id}), nil
	case "explorecodes.auth.RedeemCode":
		code := t.find(input["code"])
		if code == nil || (code.deleted && t.mode != "redeems-again") {
			return exploreCodesRefused("unknown-code", "explorecodes.auth.UnknownCode"), nil
		}
		if !code.deleted && exploreCodesInstant(code.expiry).Before(now) {
			return exploreCodesRefused("expired", "explorecodes.auth.Expired"), nil
		}
		code.deleted = true
		return t.answered("redeemed", "explorecodes.auth.Redeemed", map[string]Node{"code_id": code.id}), nil
	case "explorecodes.auth.PlanCode":
		if exploreCodesInstant(input["at"]).Before(now.Add(-time.Minute)) {
			return exploreCodesRefused("in-the-past", "explorecodes.auth.InThePast"), nil
		}
		return t.answered("planned", "explorecodes.auth.Planned", map[string]Node{"at": input["at"]}), nil
	default:
		return CommandResult{}, exploreCodesUnsupported{request.Command + " is not a command of explore-codes.yaml"}
	}
}

func (t *exploreCodesTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "explorecodes.auth.Codes" {
		return ViewResult{}, exploreCodesUnsupported{request.View + " is not a view of explore-codes.yaml"}
	}
	rows := []Row{}
	for _, code := range t.codes {
		if !code.deleted || t.mode == "keeps-row" {
			rows = append(rows, Row{"code_id": code.id, "expiry": code.expiry})
		}
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreCodesTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreCodesTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreCodesUnsupported{"no external outcomes"}
}

func (t *exploreCodesTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreCodesUnsupported{"no redelivery"}
}

func (t *exploreCodesTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreCodesUnsupported{"no bindings"}
}
