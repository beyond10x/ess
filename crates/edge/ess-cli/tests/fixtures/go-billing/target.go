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
	// issuedAt is when it left Draft, and empty while it has not. The model declares
	// `issued_at: Optional<Timestamp>`, and `OutstandingInvoices` ranks by it.
	issuedAt string
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

	// now is this target's clock, in milliseconds since the scenario began.
	//
	// A logical clock, not a wall clock, and that is what makes the elapsed-time claims runnable
	// here at all: `go test` finishes in milliseconds, two runs produce the same report, and the
	// claim is still checked honestly. `essconform.Clock` deliberately does not say which kind a
	// target keeps — an end-to-end target waits, and this one adds a number.
	now int64
	// marked is the instant each `MarkInstant` named.
	marked map[string]int64
	// publishedAt is when each event was published, on this clock.
	publishedAt map[string][]int64
}

// New builds an empty billing system.
func New(broken string) *Target {
	return &Target{
		invoices:    map[string]*invoice{},
		forced:      map[string]string{},
		escalate:    map[string]bool{},
		pending:     map[string][]essconform.ObservedEvent{},
		broken:      broken,
		marked:      map[string]int64{},
		publishedAt: map[string][]int64{},
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
	t.now = 0
	t.marked = map[string]int64{}
	t.publishedAt = map[string][]int64{}
	return nil
}

// MarkInstant records where the clock stands, under the name a later claim measures from.
func (t *Target) MarkInstant(mark essconform.InstantMark) error {
	t.marked[mark.Instant] = t.now
	return nil
}

// ObserveElapsed lets a window close and says what the clock read, and what happened inside it.
//
// The whole of a logical-clock target's side of a duration claim: move the clock to the end of the
// window, then answer. `ESS_BREAK=never-holds` is the defect this claim exists to catch — a system
// that fires every timer the instant it is armed, which passes every other check in the suite.
func (t *Target) ObserveElapsed(request essconform.ElapsedRequest) (essconform.ElapsedObservation, error) {
	opened, ok := t.marked[request.Instant]
	if !ok {
		return essconform.ElapsedObservation{}, fmt.Errorf("`%s` was never marked", request.Instant)
	}
	if t.broken != "never-holds" {
		if closes := opened + int64(request.Hold)*1000; closes > t.now {
			t.now = closes
		}
	}
	published := 0
	if request.Watching != "" {
		for _, at := range t.publishedAt[request.Watching] {
			if at >= opened && at <= t.now {
				published++
			}
		}
	}
	return essconform.ElapsedObservation{ElapsedMillis: t.now - opened, Published: published}, nil
}

func (t *Target) EndScenario(essconform.ScenarioContext) error { return nil }

func (t *Target) ConfigureExternalOutcome(control essconform.ExternalOutcomeControl) error {
	t.forced[control.Command] = control.Outcome
	return nil
}

// record notes when an event was published, on this target's own clock.
//
// What a `quiet` claim reads. It is a record of *when*, not of *whether*: an event published before
// the window opened is outside it, and a target that answered "have you ever published this" would
// fail a claim that is true.
func (t *Target) record(result essconform.CommandResult) essconform.CommandResult {
	for _, event := range result.DirectEvents {
		t.publishedAt[event.Event] = append(t.publishedAt[event.Event], t.now)
	}
	for name, queued := range t.pending {
		for range queued {
			t.publishedAt[name] = append(t.publishedAt[name], t.now)
		}
	}
	return result
}

func (t *Target) ExecuteCommand(request essconform.CommandRequest) (essconform.CommandResult, error) {
	t.pending = map[string][]essconform.ObservedEvent{}
	// A command takes a moment. One millisecond rather than a real measurement, because the point
	// is only that two things that happened at different times have different times: an event
	// published before a window opened must fall outside it, and a clock that never moved between
	// them could not say so.
	t.now++
	switch request.Command {
	case "billing.invoice.CreateInvoice":
		return t.record(t.create(request)), nil
	case "billing.invoice.IssueInvoice":
		return t.record(t.move(request, "issue", "issued", "billing.invoice.InvoiceIssued")), nil
	case "billing.invoice.CancelInvoice":
		return t.record(t.move(request, "cancel", "cancelled", "billing.invoice.InvoiceCancelled")), nil
	case "billing.invoice.PayInvoice":
		return t.record(t.pay(request)), nil
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
	if transition == "issue" {
		t.minted++
		// Counted, not read off a clock: two invoices issued in one scenario have to be orderable,
		// and a wall clock would make that depend on how fast the test ran.
		held.issuedAt = fmt.Sprintf("2020-01-01T00:00:%02dZ", t.minted%60)
	}
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
	for _, id := range t.keys(request.View) {
		rows = append(rows, t.row(id))
	}
	if request.View == "billing.invoice.OutstandingInvoices" {
		if t.broken == "one-row" {
			// A second deliberate defect: a page that stops after the first row. Every row it does
			// return is right and in the right order, so only a count can see it.
			if len(rows) > 1 {
				rows = rows[:1]
			}
		}
	}
	return essconform.ViewResult{Rows: rows}, nil
}

// keys is the listing's index: the ids the view holds, in the order it declares.
//
// The index and the rows are separate on purpose, and that separation is what makes an ordered scan
// meaningful at all. Keeping an ordering is what an index does; building a row is the per-row work a
// producer does, and the only thing a reader's stop can save. A target whose ordered read is "give
// me every row" has no index in this sense, which is exactly the finding an early-stop claim exists
// to report.
//
// Sorted by id first, so two runs return the same page. `InvoiceById` declares no order, so any
// order satisfies it — that one is for the diagnostics.
func (t *Target) keys(view string) []string {
	var ids []string
	for _, id := range t.ids() {
		held := t.invoices[id]
		if view == "billing.invoice.OutstandingInvoices" && held.state != "Issued" {
			continue
		}
		ids = append(ids, id)
	}
	// `order_by: issued_at desc`, which the specification declares and this therefore obeys.
	if view == "billing.invoice.OutstandingInvoices" {
		byIssuedAtDescendingKey(ids, func(id string) string { return t.invoices[id].issuedAt })
		if t.broken == "reversed-order" {
			// The deliberate defect: the right rows, in the wrong order. Nothing else about the
			// answer changes, so the only assertion that can catch it is the declared order — and
			// it can only catch it where the view holds more than one row.
			for left, right := 0, len(ids)-1; left < right; left, right = left+1, right-1 {
				ids[left], ids[right] = ids[right], ids[left]
			}
		}
	}
	return ids
}

// row builds one row of a listing, which is the work an early stop saves.
func (t *Target) row(id string) essconform.Row {
	held := t.invoices[id]
	total := held.amount
	if t.broken == "negative-total" {
		total = map[string]any{"amount": -1.0, "currency": "EUR"}
	}
	// `reminder_count` is projected by `InvoiceById` and read by the entity's
	// `reminder_count >= 0`. Nothing in this fixture ever sends a reminder, so it is zero — which
	// the invariant is satisfied by, and which an invariant that compared the *name*
	// `reminder_count` against zero could not decide at all.
	row := essconform.Row{"invoice_id": held.id, "total": total, "reminder_count": 0.0}
	if held.issuedAt != "" {
		row["issued_at"] = held.issuedAt
	} else {
		row["issued_at"] = nil
	}
	return row
}

// ScanView reads a listing in its declared order, a row at a time, and stops when the reader does.
//
// The whole of a target's side of an early-stop claim, and it is eight lines because that is what
// the claim is: walk the index, build a row, offer it, and stop when the reader says so. What is
// reported is what this loop did — how many rows it *built*, and whether it left early — never
// whether that satisfies anything.
//
// `ESS_BREAK=never-stops` is the defect the claim exists to catch: a listing that is built in full
// before the reader sees the first row. Its rows are the right rows in the right order, it honours
// the reader's stop, and every other check in the suite passes against it.
func (t *Target) ScanView(request essconform.ScanRequest) (essconform.ScanObservation, error) {
	keys := t.keys(request.View)
	if t.broken == "never-stops" {
		for _, id := range keys {
			_ = t.row(id)
		}
		return essconform.ScanObservation{
			Produced: len(keys),
			Halted:   len(keys) >= request.StopAfter,
		}, nil
	}
	produced, halted := 0, false
	for _, id := range keys {
		_ = t.row(id)
		produced++
		if produced >= request.StopAfter {
			halted = true
			break
		}
	}
	return essconform.ScanObservation{Produced: produced, Halted: halted}, nil
}

// byIssuedAtDescendingKey puts the most recently issued id first.
func byIssuedAtDescendingKey(ids []string, at func(string) string) {
	for index := 1; index < len(ids); index++ {
		for back := index; back > 0 && at(ids[back]) > at(ids[back-1]); back-- {
			ids[back], ids[back-1] = ids[back-1], ids[back]
		}
	}
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
