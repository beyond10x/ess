// An in-memory implementation of `explore-external.yaml`, a test double that arranges external
// outcomes the way the synthesized scenarios ask it to.
//
// `explore-external-target.mjs` is the same target in TypeScript, line for line; its header lists
// the modes.

package essconform

import "fmt"

type exploreExtUnsupported struct{ reason string }

func (e exploreExtUnsupported) Error() string { return e.reason }
func (e exploreExtUnsupported) Unwrap() error { return ErrUnsupported }

type exploreExtPayment struct {
	id     string
	amount Node
	state  string
}

type exploreExtTarget struct {
	mode     string
	payments map[string]*exploreExtPayment
	order    []string
	created  int
	version  int
	forced   map[string]string
}

func newExploreExtTarget(mode string) *exploreExtTarget {
	target := &exploreExtTarget{mode: mode}
	target.reset()
	return target
}

func (t *exploreExtTarget) reset() {
	t.payments = map[string]*exploreExtPayment{}
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

func (t *exploreExtTarget) arranged(command, outcome string) bool {
	return t.mode != "ignores-arrangement" && t.forced[command] == outcome
}

func exploreExtNumber(value Node) float64 {
	number, _ := value.(float64)
	return number
}

func (t *exploreExtTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-external-target", Version: "1"}, nil
}

func (t *exploreExtTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreExtTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreExtTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	switch request.Command {
	case "exploreext.pay.Authorize":
		amount := exploreExtNumber(input["amount"])
		if amount >= 1000 && t.arranged(request.Command, "declined") {
			return CommandResult{Outcome: "declined", Error: "exploreext.pay.Declined", DirectEvents: []ObservedEvent{}}, nil
		}
		if !(amount >= 1) {
			return CommandResult{Outcome: "invalid", Error: "exploreext.pay.InvalidAmount", DirectEvents: []ObservedEvent{}}, nil
		}
		t.created++
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", t.created)
		t.payments[id] = &exploreExtPayment{id: id, amount: input["amount"], state: "Pending"}
		t.order = append(t.order, id)
		return t.answered("authorized", "exploreext.pay.Authorized", map[string]Node{"payment_id": id, "amount": input["amount"]}), nil
	case "exploreext.pay.Capture":
		id, _ := input["payment_id"].(string)
		payment := t.payments[id]
		if payment.state != "Pending" {
			return CommandResult{Outcome: "wrong-state", Error: "exploreext.pay.PaymentStateConflict", DirectEvents: []ObservedEvent{}}, nil
		}
		if t.arranged(request.Command, "failed") {
			payment.state = "Failed"
			return t.answered("failed", "exploreext.pay.CaptureFailed", map[string]Node{"payment_id": payment.id}), nil
		}
		payment.state = "Settled"
		return t.answered("captured", "exploreext.pay.Captured", map[string]Node{"payment_id": payment.id}), nil
	default:
		return CommandResult{}, exploreExtUnsupported{request.Command + " is not a command of explore-external.yaml"}
	}
}

func (t *exploreExtTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "exploreext.pay.PendingPayments" {
		return ViewResult{}, exploreExtUnsupported{request.View + " is not a view of explore-external.yaml"}
	}
	rows := []Row{}
	for _, id := range t.order {
		payment := t.payments[id]
		if payment.state == "Pending" {
			rows = append(rows, Row{"payment_id": payment.id, "amount": payment.amount})
		}
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreExtTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreExtTarget) ConfigureExternalOutcome(control ExternalOutcomeControl) error {
	if t.mode == "cannot-arrange" {
		return exploreExtUnsupported{"this double arranges no external outcome"}
	}
	t.forced[control.Command] = control.Outcome
	return nil
}

func (t *exploreExtTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreExtUnsupported{"no redelivery"}
}

func (t *exploreExtTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreExtUnsupported{"no bindings"}
}
