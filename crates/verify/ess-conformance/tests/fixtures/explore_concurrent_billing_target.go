// A small in-memory implementation of `examples/billing`, for the concurrent explorer lanes in
// `crates/edge/ess-cli/tests/explore_concurrent.rs`.
//
// `explore-concurrent-billing-target.mjs` is the same target in TypeScript, line for line: the two
// lanes' histories are compared byte for byte, so both mint the same identities in the same order.
// It is not an InterleavedTarget, so every call takes effect at its return instant.

package essconform

import "fmt"

type exploreBillingTarget struct {
	invoices map[string]string
	created  int
	sent     int
}

func newExploreBillingTarget() *exploreBillingTarget {
	target := &exploreBillingTarget{}
	target.reset()
	return target
}

func (t *exploreBillingTarget) reset() {
	t.invoices = map[string]string{}
	t.created = 0
	t.sent = 0
}

func exploreBillingID(n int) string {
	return fmt.Sprintf("00000000-0000-4000-8000-%012d", n)
}

func exploreBillingPositive(input map[string]Node) bool {
	amount, _ := input["amount"].(map[string]any)
	value, _ := amount["amount"].(float64)
	return value > 0
}

func (t *exploreBillingTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-billing-target", Version: "1"}, nil
}

func (t *exploreBillingTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreBillingTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreBillingTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	input := request.Input
	id, _ := input["invoice_id"].(string)
	state := t.invoices[id]
	conflict := CommandResult{Outcome: "wrong-state", Error: "billing.invoice.InvoiceStateConflict", DirectEvents: []ObservedEvent{}}
	moved := func(outcome, event, to string, payload map[string]Node) CommandResult {
		t.invoices[id] = to
		return CommandResult{Outcome: outcome, DirectEvents: []ObservedEvent{{Event: event, Payload: payload}}}
	}
	switch request.Command {
	case "billing.invoice.CreateInvoice":
		if !exploreBillingPositive(input) {
			return CommandResult{Outcome: "rejected", Error: "billing.invoice.InvalidAmount"}, nil
		}
		t.created++
		created := exploreBillingID(t.created)
		t.invoices[created] = "Draft"
		return CommandResult{Outcome: "accepted", DirectEvents: []ObservedEvent{{
			Event: "billing.invoice.InvoiceCreated",
			Payload: map[string]Node{
				"invoice_id": created, "account_id": input["account_id"],
				"customer_email": input["customer_email"], "amount": input["amount"],
			},
		}}}, nil
	case "billing.invoice.IssueInvoice":
		if state != "Draft" {
			return conflict, nil
		}
		return moved("issued", "billing.invoice.InvoiceIssued", "Issued", map[string]Node{"invoice_id": id}), nil
	case "billing.invoice.PayInvoice":
		if !exploreBillingPositive(input) {
			return CommandResult{Outcome: "rejected", Error: "billing.invoice.InvalidAmount"}, nil
		}
		if state != "Issued" {
			return conflict, nil
		}
		return moved("settled", "billing.invoice.InvoicePaid", "Paid", map[string]Node{"invoice_id": id, "amount": input["amount"]}), nil
	case "billing.invoice.CancelInvoice":
		if state != "Draft" && state != "Issued" {
			return conflict, nil
		}
		return moved("cancelled", "billing.invoice.InvoiceCancelled", "Cancelled", map[string]Node{"invoice_id": id}), nil
	case "billing.email.SendEmail":
		t.sent++
		return CommandResult{Outcome: "sent", DirectEvents: []ObservedEvent{{
			Event:   "billing.email.EmailSent",
			Payload: map[string]Node{"message_id": exploreBillingID(t.sent), "recipient": input["recipient"]},
		}}}, nil
	default:
		return CommandResult{}, exploreFixtureUnsupported{request.Command + " is not a command of examples/billing"}
	}
}

func (t *exploreBillingTarget) QueryView(request ViewRequest) (ViewResult, error) {
	return ViewResult{}, exploreFixtureUnsupported{"no views"}
}

func (t *exploreBillingTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreBillingTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreFixtureUnsupported{"no external outcome"}
}

func (t *exploreBillingTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreFixtureUnsupported{"no redelivery"}
}

func (t *exploreBillingTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreFixtureUnsupported{"no bindings"}
}
