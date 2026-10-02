// generated from billing v3
// model digest 1e7906786567af32118eb2d0a8c3fcafa16c32c9649a80b60487fd2eeebc4c9c
// contract digest a21fd36f0055057629f4c235962163cdd34a3d178aa068925bcb53be623af301
// do not edit: regenerate with `ess synthesize`

// Package behaviour is what the specification fully determines, generated: the behaviour of
// every command and the query of every view the plan lists as generated, written against
// ports the implementor supplies.
//
// Storage is a port: one interface per entity, get, put and delete of a snapshot by
// identity, and list where a generated query reads every row. ess generates the interface
// and ephemeral storage for network-reached systems. Context is the other port: the caller's attributes, every identity and
// value the model says the implementation assigns, and the answer to each `external:` branch.
// Owed is every behaviour and query the plan still owes, which [Generated] forwards to.
//
// A refusal from a generated method is the typed refusal naming the command: the model
// declares no outcome for the request (a guard is undecidable over it, or no declared
// branch answers it), or — as `entity invariant` — the declared outcome would leave an
// entity breaking an invariant.
//
// Every port a generated method reads must be set: a nil field of [Ports] is a nil-pointer
// panic at the first call that reads it.
package behaviour

import (
	"example.invalid/billing/types/email"
	"example.invalid/billing/types/invoice"
	"example.invalid/billing/types/obligation"
)

// InvoiceStorage is where `billing.invoice.Invoice` is stored — a port the implementor provides.
//
// Keyed by the identity `invoice_id`. Network-reached systems also get an optional in-memory implementation.
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
	External(command string, outcome string) bool
}

// Owed is every behaviour and query the plan still owes, which [Generated] forwards to — a
// port the implementor provides.
type Owed interface {
	invoice.CreateInvoiceBehavior
	invoice.PayInvoiceBehavior
	invoice.OutstandingInvoicesQuery
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
	ports Ports
}

// New is the generated behaviours and queries, over ports.
func New(ports Ports) *Generated {
	return &Generated{ports: ports}
}

// SendEmail is `billing.email.SendEmail`, generated: every outcome is one the specification fully determines.
func (b *Generated) SendEmail(input email.SendEmail) (email.SendEmailOutcome, *obligation.UnmetObligation) {
	// `failed`: an external branch, where the context takes it.
	if b.ports.Context.External("billing.email.SendEmail", "failed") {
		return email.SendEmailOutcomeFailed{Error: email.Undeliverable{}}, nil
	}
	// `sent`: the default.
	m0 := b.ports.Context.GenerateBillingEmailMessageId()
	return email.SendEmailOutcomeSent{EmailSent: email.EmailSent{MessageId: m0, Recipient: input.Recipient}}, nil
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
	b.ports.InvoiceStorage.Put(next)
	return invoice.CancelInvoiceOutcomeCancelled{InvoiceCancelled: invoice.InvoiceCancelled{InvoiceId: input.InvoiceId}}, nil
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
	if broken, breaks := next.Data.BrokenInvariant(); breaks {
		return nil, &obligation.UnmetObligation{Capability: "entity invariant", Source: broken}
	}
	b.ports.InvoiceStorage.Put(next)
	return invoice.IssueInvoiceOutcomeIssued{InvoiceIssued: invoice.InvoiceIssued{InvoiceId: input.InvoiceId}}, nil
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

// OutstandingInvoices forwards the owed query `billing.invoice.OutstandingInvoices` to the ports.
func (b *Generated) OutstandingInvoices() ([]invoice.OutstandingInvoices, *obligation.UnmetObligation) {
	return b.ports.Owed.OutstandingInvoices()
}
