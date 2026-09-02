// A Go implementation of `examples/billing`, for holding the emitted runner to a target that is
// actually right.
//
// Hand-written, and deliberately small: what it exists to prove is that the generated package runs
// a synthesized suite to green against a correct implementation and to red against a wrong one. It
// is not a reference implementation and nothing outside this test should read it.
package billing

import (
	"fmt"

	"essbilling/essconform"
)

// invoice is one instance, in whatever state it has reached.
type invoice struct {
	id     string
	state  string
	email  string
	amount map[string]any
}

// Target is the billing system, in memory.
type Target struct {
	invoices map[string]*invoice
	minted   int

	// forced is the branch an externally decided outcome must take, by command.
	forced map[string]string
	// escalate is set once the binding has given up, so redelivery does not un-escalate it.
	escalate map[string]bool
	// pending are the events a later observation will report, by event name.
	pending map[string][]essconform.ObservedEvent
	// broken makes one deliberate defect, for the test that checks the suite can fail.
	broken string
}

// New builds an empty billing system.
func New(broken string) *Target {
	return &Target{
		invoices: map[string]*invoice{},
		forced:   map[string]string{},
		escalate: map[string]bool{},
		pending:  map[string][]essconform.ObservedEvent{},
		broken:   broken,
	}
}

func (t *Target) Identity() (essconform.Identity, error) {
	return essconform.Identity{Name: "billing-go", Version: "v3"}, nil
}

func (t *Target) BeginScenario(essconform.ScenarioContext) error {
	t.invoices = map[string]*invoice{}
	t.forced = map[string]string{}
	t.escalate = map[string]bool{}
	t.pending = map[string][]essconform.ObservedEvent{}
	return nil
}

func (t *Target) EndScenario(essconform.ScenarioContext) error { return nil }

func (t *Target) ConfigureExternalOutcome(control essconform.ExternalOutcomeControl) error {
	t.forced[control.Command] = control.Outcome
	return nil
}

func (t *Target) ExecuteCommand(request essconform.CommandRequest) (essconform.CommandResult, error) {
	t.pending = map[string][]essconform.ObservedEvent{}
	switch request.Command {
	case "billing.invoice.CreateInvoice":
		return t.create(request), nil
	case "billing.invoice.IssueInvoice":
		return t.move(request, "issue", "issued", "billing.invoice.InvoiceIssued"), nil
	case "billing.invoice.CancelInvoice":
		return t.move(request, "cancel", "cancelled", "billing.invoice.InvoiceCancelled"), nil
	case "billing.invoice.PayInvoice":
		return t.pay(request), nil
	case "billing.email.SendEmail":
		return t.send(request), nil
	default:
		return essconform.CommandResult{}, fmt.Errorf("`%s` is not a command of this system", request.Command)
	}
}

func (t *Target) create(request essconform.CommandRequest) essconform.CommandResult {
	amount, _ := request.Input["amount"].(map[string]any)
	if !positive(amount) {
		return essconform.CommandResult{Outcome: "rejected", Error: "billing.invoice.InvalidAmount"}
	}
	t.minted++
	// Deterministic, because the runner owns every source of variation and a target that reached
	// for a random id would put one back.
	id := fmt.Sprintf("00000000-0000-4000-8000-%012d", t.minted)
	email, _ := request.Input["customer_email"].(string)
	// The account the invoice belongs to. Read from the input and announced unchanged: the
	// specification declares `billing.invoice.Account` owns invoices by this field, so a target
	// that minted one here would be claiming an owner nobody asked for.
	account, _ := request.Input["account_id"].(string)
	t.invoices[id] = &invoice{id: id, state: "Draft", email: email, amount: amount}

	created := essconform.ObservedEvent{
		Event: "billing.invoice.InvoiceCreated",
		Payload: map[string]essconform.Node{
			"invoice_id":     id,
			"account_id":     account,
			"customer_email": email,
			"amount":         amount,
		},
	}
	// The binding: `notify-on-invoice-created` invokes SendEmail, which either sends or, having
	// been given up on, escalates. Both are observed away from this command, so they are queued.
	t.deliver(email)
	return essconform.CommandResult{
		Outcome:      "accepted",
		Consistency:  fmt.Sprintf("token-%d", t.minted),
		DirectEvents: []essconform.ObservedEvent{created},
	}
}

// deliver runs the binding once, at least once being what the specification declares.
func (t *Target) deliver(email string) {
	if t.forced["billing.email.SendEmail"] == "failed" {
		t.escalate[email] = true
		t.queue(essconform.ObservedEvent{
			Event: "billing.email.DeliveryEscalated",
			Payload: map[string]essconform.Node{
				"recipient": email,
				"template":  "invoice-created",
			},
		})
		return
	}
	t.minted++
	t.queue(essconform.ObservedEvent{
		Event: "billing.email.EmailSent",
		Payload: map[string]essconform.Node{
			"message_id": fmt.Sprintf("00000000-0000-4000-8000-%012d", t.minted),
			"recipient":  email,
		},
	})
}

func (t *Target) queue(event essconform.ObservedEvent) {
	t.pending[event.Event] = append(t.pending[event.Event], event)
}

func (t *Target) move(request essconform.CommandRequest, transition, outcome, event string) essconform.CommandResult {
	id, _ := request.Input["invoice_id"].(string)
	held, ok := t.invoices[id]
	if !ok || !legal(held.state, transition) {
		return essconform.CommandResult{
			Outcome: "wrong-state",
			Error:   "billing.invoice.InvoiceStateConflict",
		}
	}
	held.state = reached(transition)
	return essconform.CommandResult{
		Outcome:     outcome,
		Consistency: fmt.Sprintf("token-%s-%s", id, transition),
		DirectEvents: []essconform.ObservedEvent{{
			Event:   event,
			Payload: map[string]essconform.Node{"invoice_id": id},
		}},
	}
}

func (t *Target) pay(request essconform.CommandRequest) essconform.CommandResult {
	amount, _ := request.Input["amount"].(map[string]any)
	// The amount is decided before the state, which is what the scenarios read: `rejected` is
	// reached with an invoice id nothing created, so a state check first would answer wrong-state.
	if !positive(amount) {
		return essconform.CommandResult{Outcome: "rejected", Error: "billing.invoice.InvalidAmount"}
	}
	id, _ := request.Input["invoice_id"].(string)
	held, ok := t.invoices[id]
	if !ok || !legal(held.state, "settle") {
		return essconform.CommandResult{
			Outcome: "wrong-state",
			Error:   "billing.invoice.InvoiceStateConflict",
		}
	}
	held.state = "Paid"
	return essconform.CommandResult{
		Outcome:     "settled",
		Consistency: fmt.Sprintf("token-%s-settle", id),
		DirectEvents: []essconform.ObservedEvent{{
			Event: "billing.invoice.InvoicePaid",
			Payload: map[string]essconform.Node{
				"invoice_id": id,
				"amount":     amount,
			},
		}},
	}
}

func (t *Target) send(request essconform.CommandRequest) essconform.CommandResult {
	if t.forced["billing.email.SendEmail"] == "failed" {
		return essconform.CommandResult{Outcome: "failed", Error: "billing.email.Undeliverable"}
	}
	recipient, _ := request.Input["recipient"].(string)
	t.minted++
	return essconform.CommandResult{
		Outcome: "sent",
		DirectEvents: []essconform.ObservedEvent{{
			Event: "billing.email.EmailSent",
			Payload: map[string]essconform.Node{
				"message_id": fmt.Sprintf("00000000-0000-4000-8000-%012d", t.minted),
				"recipient":  recipient,
			},
		}},
	}
}

func (t *Target) QueryView(request essconform.ViewRequest) (essconform.ViewResult, error) {
	var rows []essconform.Row
	// Sorted by id, so two runs return the same page. The model declares no order for either view,
	// so any order satisfies it — this one is for the diagnostics.
	for _, id := range t.ids() {
		held := t.invoices[id]
		if request.View == "billing.invoice.OutstandingInvoices" && held.state != "Issued" {
			continue
		}
		total := held.amount
		if t.broken == "negative-total" {
			total = map[string]any{"amount": -1.0, "currency": "EUR"}
		}
		rows = append(rows, essconform.Row{"invoice_id": held.id, "total": total})
	}
	return essconform.ViewResult{Rows: rows}, nil
}

func (t *Target) ids() []string {
	ids := make([]string, 0, len(t.invoices))
	for id := range t.invoices {
		ids = append(ids, id)
	}
	sortStrings(ids)
	return ids
}

func (t *Target) ObserveEvents(request essconform.EventObservationRequest) ([]essconform.ObservedEvent, error) {
	seen := t.pending[request.Event]
	delete(t.pending, request.Event)
	return seen, nil
}

// RedeliverEvent delivers the triggering event again, which is the only way to perform the claim
// `delivery: at_least_once` makes.
func (t *Target) RedeliverEvent(request essconform.RedeliveryRequest) error {
	if request.Event != "billing.invoice.InvoiceCreated" {
		return essconform.ErrUnsupported
	}
	for _, held := range t.invoices {
		t.deliver(held.email)
	}
	return nil
}

func (t *Target) ObserveInvocations(request essconform.InvocationObservationRequest) ([]essconform.Invocation, error) {
	if request.Binding != "notify-on-invoice-created" {
		return nil, essconform.ErrUnsupported
	}
	var made []essconform.Invocation
	for _, id := range t.ids() {
		made = append(made, essconform.Invocation{
			Command: "billing.email.SendEmail",
			Input: map[string]essconform.Node{
				"recipient": t.invoices[id].email,
				"template":  "invoice-created",
			},
		})
	}
	return made, nil
}

// ---- the lifecycle, as the specification declares it -------------------------------------------

func legal(state, transition string) bool {
	switch transition {
	case "issue":
		return state == "Draft"
	case "settle":
		return state == "Issued"
	case "cancel":
		return state == "Draft" || state == "Issued"
	default:
		return false
	}
}

func reached(transition string) string {
	switch transition {
	case "issue":
		return "Issued"
	case "settle":
		return "Paid"
	default:
		return "Cancelled"
	}
}

func positive(amount map[string]any) bool {
	value, ok := amount["amount"].(float64)
	return ok && value > 0
}

func sortStrings(values []string) {
	for index := 1; index < len(values); index++ {
		for back := index; back > 0 && values[back] < values[back-1]; back-- {
			values[back], values[back-1] = values[back-1], values[back]
		}
	}
}
