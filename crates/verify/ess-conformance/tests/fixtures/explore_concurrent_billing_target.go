// A small in-memory implementation of `examples/billing`, for the concurrent explorer lanes in
// `crates/edge/ess-cli/tests/explore_concurrent.rs`.
//
// `explore-concurrent-billing-target.mjs` is the same target in TypeScript, line for line: the two
// lanes' histories are compared byte for byte, so both mint the same identities in the same order.
// It is not an InterleavedTarget, so every call takes effect at its return instant. Both views
// answer from the current invoices, which is what `read_your_writes` and `eventual` both allow.
//
// The mutant `double-apply` (the argument, or `ESS_EXPLORE_MUTANT`) applies the creation an
// `InvoiceCreated` announces again when the event is delivered a second time: an invoice nobody
// created, which only a read of a view shows.

package essconform

import (
	"fmt"
	"os"
)

type exploreBillingTarget struct {
	mutant   string
	invoices map[string]string
	// order is every invoice, in the order it was created.
	order   []string
	created int
	sent    int
	// creation is the input of the last `CreateInvoice` that created an invoice.
	creation map[string]Node
}

func newExploreBillingTarget(mutant string) *exploreBillingTarget {
	if mutant == "" {
		mutant = os.Getenv("ESS_EXPLORE_MUTANT")
	}
	target := &exploreBillingTarget{mutant: mutant}
	target.reset()
	return target
}

func (t *exploreBillingTarget) reset() {
	t.invoices = map[string]string{}
	t.order = nil
	t.created = 0
	t.sent = 0
	t.creation = nil
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

// create mints an invoice in `Draft` from input.
func (t *exploreBillingTarget) create(input map[string]Node) string {
	t.created++
	created := exploreBillingID(t.created)
	t.invoices[created] = "Draft"
	t.order = append(t.order, created)
	t.creation = input
	return created
}

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
		created := t.create(input)
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

// QueryView answers `InvoiceById` with every invoice and `OutstandingInvoices` with the issued
// ones, in the order they were created.
func (t *exploreBillingTarget) QueryView(request ViewRequest) (ViewResult, error) {
	outstanding := request.View == "billing.invoice.OutstandingInvoices"
	if !outstanding && request.View != "billing.invoice.InvoiceById" {
		return ViewResult{}, exploreFixtureUnsupported{request.View + " is not a view of examples/billing"}
	}
	rows := []Row{}
	for _, id := range t.order {
		if !outstanding || t.invoices[id] == "Issued" {
			rows = append(rows, Row{"invoice_id": id})
		}
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreBillingTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreBillingTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreFixtureUnsupported{"no external outcome"}
}

// RedeliverEvent delivers `InvoiceCreated` again to `notify-on-invoice-created`, whose mail this target
// does not model, so nothing changes; any other event it has no binding for. Under `double-apply`
// the creation it announces is applied again.
func (t *exploreBillingTarget) RedeliverEvent(request RedeliveryRequest) error {
	if request.Event != "billing.invoice.InvoiceCreated" {
		return exploreFixtureUnsupported{"no binding reacts to " + request.Event}
	}
	if t.mutant == "double-apply" && t.creation != nil {
		t.create(t.creation)
	}
	return nil
}

func (t *exploreBillingTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreFixtureUnsupported{"no bindings"}
}
