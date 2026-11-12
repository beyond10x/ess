// generated from billing v3
// contract digest c9ecfdf5bed1bcb88068dad060895f16ca73de361204ae488d6f0d39477f6f79
// do not edit: regenerate with `ess synthesize`

// Package behaviour is what the specification fully determines, generated: the behaviour of
// every command and the query of every view the plan lists as generated, written against
// ports the implementor supplies.
//
// Storage is a port: one interface per entity, get, put and delete of a snapshot by
// identity, and list where a generated query reads every row. ess generates the interface. Network entries supply ephemeral stores. Context carries the caller's attributes, every identity and
// value the model says the implementation assigns, and the answer to each `external:` branch.
// Owed is every behaviour and query the plan still owes, which [Generated] forwards to.
//
// A refusal from a generated method is the typed refusal naming the command: the model
// declares no outcome for the request (a guard is undecidable over it, or no declared
// branch answers it), or — as `entity invariant` — the declared outcome would leave an
// entity breaking an invariant.
//
// Every storage and owed port a generated method reads must be set. Context uses the
// fallible companion when supplied, otherwise the legacy port; absence is a typed refusal.
package behaviour

import (
	"example.invalid/billing/types/email"
	"example.invalid/billing/types/invoice"
	"example.invalid/billing/types/obligation"
	"sort"
	"strings"
)

// InvoiceStorage is where `billing.invoice.Invoice` is stored — a port the implementor provides.
//
// Keyed by the identity `invoice_id`. Network entries supply ephemeral stores; durable storage stays a port.
type InvoiceStorage interface {
	// Get is the instance with this identity and true, or false where none is stored.
	Get(identity invoice.InvoiceId) (invoice.InvoiceSnapshot, bool)

	// Put stores this instance under its identity, replacing what was held.
	Put(snapshot invoice.InvoiceSnapshot)

	// Delete removes the instance with this identity.
	Delete(identity invoice.InvoiceId)

	// List is every stored instance, in an order the store keeps stable: the order a
	// generated query answers an unordered view in.
	List() []invoice.InvoiceSnapshot
}

// ExternalCommand is the exact executing input supplied to an external decision.
// It supplies facts, not authority; the context must verify its request-bound proof.
// Nested pointers, maps and slices retain Go aliasing and must not be mutated.
type ExternalCommand interface {
	Name() string
	isExternalCommand()
}

// ExternalCommandBillingEmailSendEmail carries the executing `billing.email.SendEmail` input.
type ExternalCommandBillingEmailSendEmail struct {
	// Input is the actual command value.
	Input email.SendEmail
}

// Name is the canonical qualified command identity.
func (ExternalCommandBillingEmailSendEmail) Name() string { return "billing.email.SendEmail" }

func (ExternalCommandBillingEmailSendEmail) isExternalCommand() {}

// Context is what the specification leaves to the implementor's context — a port the
// implementor provides.
//
// The caller's attributes, the values the model says the implementation assigns, and the answer
// to each `external:` branch.
type Context interface {
	// GenerateBillingEmailMessageId is a new `billing.email.MessageId`, which the model says the implementation assigns — a
	// created identity, a `{generated: true}` value, or an event field the model leaves
	// undetermined.
	GenerateBillingEmailMessageId() email.MessageId

	// External reports whether the external branch outcome of command is taken on this
	// invocation.
	//
	// Asked in declaration order, before the branch's input guard is read; the first branch
	// answered true whose guard holds is taken. A test forces a branch by answering true for
	// it alone; a deployment asks whatever decides it.
	External(command ExternalCommand, outcome string) bool
}

// FallibleContext reports unavailable context answers explicitly.
type FallibleContext interface {
	TryGenerateBillingEmailMessageId() (email.MessageId, *obligation.UnmetObligation)
	TryExternal(command ExternalCommand, outcome string) (bool, *obligation.UnmetObligation)
}

// readGenerateBillingEmailMessageId prefers the fallible port, then adapts the legacy context.
func (contextPorts *Generated) readGenerateBillingEmailMessageId() (email.MessageId, *obligation.UnmetObligation) {
	if contextPorts.context != nil {
		return contextPorts.context.TryGenerateBillingEmailMessageId()
	}
	if contextPorts.ports.Context == nil {
		var zero email.MessageId
		return zero, UnmetContext("assigned value: billing.email.MessageId")
	}
	return contextPorts.ports.Context.GenerateBillingEmailMessageId(), nil
}

// readExternal prefers the fallible port, then adapts the legacy context.
func (contextPorts *Generated) readExternal(command ExternalCommand, outcome string) (bool, *obligation.UnmetObligation) {
	if contextPorts.context != nil {
		return contextPorts.context.TryExternal(command, outcome)
	}
	if contextPorts.ports.Context == nil {
		return false, UnmetContext("external branch answer")
	}
	return contextPorts.ports.Context.External(command, outcome), nil
}

// UnmetContext names an unavailable runtime answer, not a new planned capability.
func UnmetContext(source string) *obligation.UnmetObligation {
	return &obligation.UnmetObligation{Capability: "context answer", Source: source}
}

// Owed is every behaviour and query the plan still owes, which [Generated] forwards to — a
// port the implementor provides.
type Owed interface {
	invoice.CreateInvoiceBehavior
	invoice.PayInvoiceBehavior
}

// Ports is everything the generated behaviours and queries read, and everything the plan still
// owes.
type Ports struct {
	// InvoiceStorage is where `billing.invoice.Invoice` is stored.
	InvoiceStorage InvoiceStorage
	// Context is the caller, the assigned values and the external answers.
	Context Context
	// Owed answers every behaviour and query the plan still owes.
	Owed Owed
}

// Generated is every generated behaviour and query of this module, over the ports.
//
// It has the method of every seam a component's behaviour bundle names, generated or owed, so
// a `*Generated` is a complete bundle for every component port. To replace one generated
// behaviour, write a bundle of your own with that method that delegates the rest to a
// `*Generated`.
type Generated struct {
	ports   Ports
	context FallibleContext
}

// New is the generated behaviours and queries, over ports.
func New(ports Ports) *Generated {
	return &Generated{ports: ports}
}

// NewWithContext explicitly supplies the fallible companion; it takes precedence over Ports.Context.
func NewWithContext(ports Ports, context FallibleContext) *Generated {
	return &Generated{ports: ports, context: context}
}

// SendEmail is `billing.email.SendEmail`, generated: every outcome is one the specification fully determines.
func (b *Generated) SendEmail(input email.SendEmail) (email.SendEmailOutcome, *obligation.UnmetObligation) {
	// `failed`: an external branch, where the context takes it.
	external0, contextErr1 := b.readExternal(ExternalCommandBillingEmailSendEmail{Input: input}, "failed")
	if contextErr1 != nil {
		return nil, contextErr1
	}
	if external0 {
		return email.SendEmailOutcomeFailed{Error: email.Undeliverable{}}, nil
	}
	// `sent`: the default.
	m2, contextErr3 := b.readGenerateBillingEmailMessageId()
	if contextErr3 != nil {
		return nil, contextErr3
	}
	return email.SendEmailOutcomeSent{EmailSent: email.EmailSent{MessageId: m2, Recipient: input.Recipient}}, nil
}

// CancelInvoice is `billing.invoice.CancelInvoice`, generated: every outcome is one the specification fully determines.
func (b *Generated) CancelInvoice(input invoice.CancelInvoice) (invoice.CancelInvoiceOutcome, *obligation.UnmetObligation) {
	// `cancelled`: the default.
	held, found := b.ports.InvoiceStorage.Get(input.InvoiceId)
	if !found {
		return invoice.CancelInvoiceOutcomeWrongStateUnknownInstance{}, nil
	}
	_ = held
	heldState := held.State
	refined, _ := held.Refine()
	var moved invoice.AnyInvoice
	switch instance := refined.(type) {
	case invoice.InvoiceInDraft:
		moved = instance.Cancel()
	case invoice.InvoiceInIssued:
		moved = instance.Cancel()
	default:
		return invoice.CancelInvoiceOutcomeWrongState{Error: invoice.InvoiceStateConflict{State: heldState}}, nil
	}
	next := moved.Snapshot()
	if broken, breaks := next.Data.BrokenInvariant(); breaks {
		return nil, &obligation.UnmetObligation{Capability: "entity invariant", Source: broken}
	}
	answer0 := invoice.CancelInvoiceOutcomeCancelled{InvoiceCancelled: invoice.InvoiceCancelled{InvoiceId: input.InvoiceId}}
	b.ports.InvoiceStorage.Put(next)
	return answer0, nil
}

// CreateInvoice forwards the owed behaviour `billing.invoice.CreateInvoice` to the ports.
func (b *Generated) CreateInvoice(input invoice.CreateInvoice) (invoice.CreateInvoiceOutcome, *obligation.UnmetObligation) {
	return b.ports.Owed.CreateInvoice(input)
}

// IssueInvoice is `billing.invoice.IssueInvoice`, generated: every outcome is one the specification fully determines.
func (b *Generated) IssueInvoice(input invoice.IssueInvoice) (invoice.IssueInvoiceOutcome, *obligation.UnmetObligation) {
	// `issued`: the default.
	held, found := b.ports.InvoiceStorage.Get(input.InvoiceId)
	if !found {
		return invoice.IssueInvoiceOutcomeWrongStateUnknownInstance{}, nil
	}
	_ = held
	heldState := held.State
	refined, _ := held.Refine()
	var moved invoice.AnyInvoice
	switch instance := refined.(type) {
	case invoice.InvoiceInDraft:
		moved = instance.Issue()
	default:
		return invoice.IssueInvoiceOutcomeWrongState{Error: invoice.InvoiceStateConflict{State: heldState}}, nil
	}
	next := moved.Snapshot()
	next.Data.IssuedAt = some(input.IssuedAt)
	if broken, breaks := next.Data.BrokenInvariant(); breaks {
		return nil, &obligation.UnmetObligation{Capability: "entity invariant", Source: broken}
	}
	answer0 := invoice.IssueInvoiceOutcomeIssued{InvoiceIssued: invoice.InvoiceIssued{InvoiceId: input.InvoiceId}}
	b.ports.InvoiceStorage.Put(next)
	return answer0, nil
}

// PayInvoice forwards the owed behaviour `billing.invoice.PayInvoice` to the ports.
func (b *Generated) PayInvoice(input invoice.PayInvoice) (invoice.PayInvoiceOutcome, *obligation.UnmetObligation) {
	return b.ports.Owed.PayInvoice(input)
}

// InvoiceById is `billing.invoice.InvoiceById`, generated: every row is one the specification fully determines from the
// stored `billing.invoice.Invoice`s.
func (b *Generated) InvoiceById() ([]invoice.InvoiceById, *obligation.UnmetObligation) {
	listed := b.ports.InvoiceStorage.List()
	rows := make([]invoice.InvoiceById, 0, len(listed))
	for _, held := range listed {
		rows = append(rows, invoice.InvoiceById{InvoiceId: held.Data.InvoiceId, Total: held.Data.Total, ReminderCount: held.Data.ReminderCount})
	}
	return rows, nil
}

// OutstandingInvoices is `billing.invoice.OutstandingInvoices`, generated: every row is one the specification fully determines from the
// stored `billing.invoice.Invoice`s.
func (b *Generated) OutstandingInvoices() ([]invoice.OutstandingInvoices, *obligation.UnmetObligation) {
	listed := b.ports.InvoiceStorage.List()
	// `filter:` shows a row where it holds; false or unknown hides it.
	var kept []invoice.InvoiceSnapshot
	for _, held := range listed {
		var r0 *string
		switch held.State.(type) {
		case invoice.InvoiceStateCancelled:
			r0 = some("Cancelled")
		case invoice.InvoiceStateDraft:
			r0 = some("Draft")
		case invoice.InvoiceStateIssued:
			r0 = some("Issued")
		case invoice.InvoiceStatePaid:
			r0 = some("Paid")
		}
		if equal(r0, some("Issued")) == verity {
			kept = append(kept, held)
		}
	}
	// `order_by:`, stably: rows equal on every key keep the port's order.
	sort.SliceStable(kept, func(i int, j int) bool {
		left, right := kept[i], kept[j]
		var r2 *string
		if left.Data.IssuedAt != nil {
			v1 := *left.Data.IssuedAt
			_ = v1
			r2 = some(v1.Value())
		}
		var r4 *string
		if right.Data.IssuedAt != nil {
			v3 := *right.Data.IssuedAt
			_ = v3
			r4 = some(v3.Value())
		}
		if c := readOrderBy(r2, r4, instantOrder); c != 0 {
			return c > 0
		}
		return false
	})
	rows := make([]invoice.OutstandingInvoices, 0, len(kept))
	for _, held := range kept {
		rows = append(rows, invoice.OutstandingInvoices{InvoiceId: held.Data.InvoiceId, Total: held.Data.Total, IssuedAt: held.Data.IssuedAt})
	}
	return rows, nil
}

// truth is a guard's reading: true, false, or unknown where a value it reads is absent.
type truth int

// unknown is the reading of a guard a value it reads is absent from: it selects no branch.
const unknown truth = 0

// falsity is the reading of a guard that does not hold.
const falsity truth = 1

// verity is the reading of a guard that holds.
const verity truth = 2

// some is value, present.
func some[T any](value T) *T {
	return &value
}

// known is the reading of a decided guard.
func known(holds bool) truth {
	if holds {
		return verity
	}
	return falsity
}

// equal is the equality of two read values; an unread one is unknown.
func equal[T comparable](left *T, right *T) truth {
	if left == nil || right == nil {
		return unknown
	}
	return known(*left == *right)
}

// readOrderBy orders two read texts as an `Option` orders: an unread one first, then by order.
func readOrderBy(left *string, right *string, order func(string, string) int) int {
	switch {
	case left == nil && right == nil:
		return 0
	case left == nil:
		return -1
	case right == nil:
		return 1
	}
	return order(*left, *right)
}

// instant is the instant an RFC 3339 `date-time` names, as seconds from the epoch and
// nanoseconds; ok is false where it names none.
func instant(value string) (int64, int64, bool) {
	digits := func(from int, to int) (int64, bool) {
		if from >= to || to > len(value) {
			return 0, false
		}
		var total int64
		for index := from; index < to; index++ {
			if value[index] < '0' || value[index] > '9' {
				return 0, false
			}
			total = total*10 + int64(value[index]-'0')
		}
		return total, true
	}
	at := func(index int, expected string) bool {
		return index < len(value) && strings.IndexByte(expected, value[index]) >= 0
	}
	if !at(4, "-") || !at(7, "-") || !at(10, "Tt") || !at(13, ":") || !at(16, ":") {
		return 0, 0, false
	}
	year, yearOk := digits(0, 4)
	month, monthOk := digits(5, 7)
	day, dayOk := digits(8, 10)
	if !yearOk || !monthOk || !dayOk {
		return 0, 0, false
	}
	leap := (year%4 == 0 && year%100 != 0) || year%400 == 0
	var length int64
	switch month {
	case 1, 3, 5, 7, 8, 10, 12:
		length = 31
	case 4, 6, 9, 11:
		length = 30
	case 2:
		length = 28
		if leap {
			length = 29
		}
	default:
		return 0, 0, false
	}
	if day < 1 || day > length {
		return 0, 0, false
	}
	hour, hourOk := digits(11, 13)
	minute, minuteOk := digits(14, 16)
	second, secondOk := digits(17, 19)
	if !hourOk || !minuteOk || !secondOk || hour > 23 || minute > 59 || second > 59 {
		return 0, 0, false
	}
	position := 19
	var nanos int64
	if at(position, ".") {
		start := position + 1
		end := start
		for end < len(value) && value[end] >= '0' && value[end] <= '9' {
			end++
		}
		width := end - start
		if width == 0 || width > 9 {
			return 0, 0, false
		}
		fraction, ok := digits(start, end)
		if !ok {
			return 0, 0, false
		}
		for padding := width; padding < 9; padding++ {
			fraction *= 10
		}
		nanos = fraction
		position = end
	}
	var offset int64
	rest := value[position:]
	switch {
	case rest == "Z" || rest == "z":
		offset = 0
	case len(rest) == 6 && (rest[0] == '+' || rest[0] == '-') && rest[3] == ':':
		hours, hoursOk := digits(position+1, position+3)
		minutes, minutesOk := digits(position+4, position+6)
		if !hoursOk || !minutesOk || hours > 23 || minutes > 59 {
			return 0, 0, false
		}
		offset = hours*3600 + minutes*60
		if rest[0] == '-' {
			offset = -offset
		}
	default:
		return 0, 0, false
	}
	shifted := year
	if month <= 2 {
		shifted--
	}
	era := shifted
	if era < 0 {
		era -= 399
	}
	era /= 400
	yearOfEra := shifted - era*400
	shiftedMonth := month - 3
	if month <= 2 {
		shiftedMonth = month + 9
	}
	dayOfYear := (153*shiftedMonth+2)/5 + day - 1
	dayOfEra := yearOfEra*365 + yearOfEra/4 - yearOfEra/100 + dayOfYear
	days := era*146097 + dayOfEra - 719468
	return days*86400 + hour*3600 + minute*60 + second - offset, nanos, true
}

// instantOrder orders two RFC 3339 renderings by the instant each names, and text that names
// none by its bytes: -1, 0 or 1.
func instantOrder(left string, right string) int {
	leftSeconds, leftNanos, leftOk := instant(left)
	rightSeconds, rightNanos, rightOk := instant(right)
	if !leftOk || !rightOk {
		return strings.Compare(left, right)
	}
	switch {
	case leftSeconds < rightSeconds:
		return -1
	case leftSeconds > rightSeconds:
		return 1
	case leftNanos < rightNanos:
		return -1
	case leftNanos > rightNanos:
		return 1
	}
	return 0
}
