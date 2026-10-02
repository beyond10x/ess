// generated from gatepass v1
// model digest f8ccea748a49e127ca2e18f725481394cc0eab1787fafd77d16c52485bf2abba
// contract digest a6fdd92f3a88ac0abbe59789406f3001df466e87f222e4aad1a8348c17f91d7c
// do not edit: regenerate with `ess synthesize`

// Package behaviour is what the specification fully determines, generated: the behaviour of
// every command and the query of every view the plan lists as generated, written against
// ports the implementor supplies.
//
// Storage is a port: one interface per entity, get, put and delete of a snapshot by
// identity, and list where a generated query reads every row. ess generates the interface
// and never a store. Context is the other port: the caller's attributes, every identity and
// value the model says the implementation assigns, and the answer to each `external:` branch.
// Owed is every behaviour and query the plan still owes, which [Generated] forwards to.
//
// A refusal from a generated method is the typed refusal naming the command: the model
// declares no outcome for the request (a guard is undecidable over it, or no declared
// branch answers it), or — as `entity invariant` — the declared outcome would leave an
// entity breaking an invariant.
package behaviour

import (
	"example.invalid/gatepass/types/obligation"
	"example.invalid/gatepass/types/visit"
)

// VisitStorage is where `gatepass.visit.Visit` is stored — a port the implementor provides.
//
// Keyed by the identity `visit_id`. ess generates this interface and never an implementation of it.
type VisitStorage interface {
	// Get is the instance with this identity and true, or false where none is stored.
	Get(identity visit.VisitId) (visit.VisitSnapshot, bool)

	// Put stores this instance under its identity, replacing what was held.
	Put(snapshot visit.VisitSnapshot)

	// Delete removes the instance with this identity.
	Delete(identity visit.VisitId)

	// List is every stored instance, in an order the store keeps stable: the order a
	// generated query answers an unordered view in.
	List() []visit.VisitSnapshot
}

// Owed is every behaviour and query the plan still owes, which [Generated] forwards to — a
// port the implementor provides.
type Owed interface {
	visit.RegisterVisitBehavior
}

// Ports is everything the generated behaviours and queries read, and everything the plan still
// owes.
type Ports struct {
	// VisitStorage is where `gatepass.visit.Visit` is stored.
	VisitStorage VisitStorage
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

// AdmitVisitor is `gatepass.visit.AdmitVisitor`, generated: every outcome is one the specification fully determines.
func (b *Generated) AdmitVisitor(input visit.AdmitVisitor) (visit.AdmitVisitorOutcome, *obligation.UnmetObligation) {
	// `admitted`: the default.
	held, found := b.ports.VisitStorage.Get(input.VisitId)
	if !found {
		return visit.AdmitVisitorOutcomeWrongStateUnknownInstance{}, nil
	}
	_ = held
	heldState := held.State
	refined, _ := held.Refine()
	var moved visit.AnyVisit
	switch instance := refined.(type) {
	case visit.VisitInExpected:
		moved = instance.Arrive()
	default:
		return visit.AdmitVisitorOutcomeWrongState{Error: visit.VisitStateConflict{State: heldState}}, nil
	}
	next := moved.Snapshot()
	if broken, breaks := next.Data.BrokenInvariant(); breaks {
		return nil, &obligation.UnmetObligation{Capability: "entity invariant", Source: broken}
	}
	b.ports.VisitStorage.Put(next)
	return visit.AdmitVisitorOutcomeAdmitted{VisitorAdmitted: visit.VisitorAdmitted{VisitId: input.VisitId, Badge: input.Badge}}, nil
}

// RegisterVisit forwards the owed behaviour `gatepass.visit.RegisterVisit` to the ports.
func (b *Generated) RegisterVisit(input visit.RegisterVisit) (visit.RegisterVisitOutcome, *obligation.UnmetObligation) {
	return b.ports.Owed.RegisterVisit(input)
}

// SignOutVisitor is `gatepass.visit.SignOutVisitor`, generated: every outcome is one the specification fully determines.
func (b *Generated) SignOutVisitor(input visit.SignOutVisitor) (visit.SignOutVisitorOutcome, *obligation.UnmetObligation) {
	// `signed-out`: the default.
	held, found := b.ports.VisitStorage.Get(input.VisitId)
	if !found {
		return visit.SignOutVisitorOutcomeWrongStateUnknownInstance{}, nil
	}
	_ = held
	heldState := held.State
	refined, _ := held.Refine()
	var moved visit.AnyVisit
	switch instance := refined.(type) {
	case visit.VisitInOnSite:
		moved = instance.Depart()
	default:
		return visit.SignOutVisitorOutcomeWrongState{Error: visit.VisitStateConflict{State: heldState}}, nil
	}
	next := moved.Snapshot()
	if broken, breaks := next.Data.BrokenInvariant(); breaks {
		return nil, &obligation.UnmetObligation{Capability: "entity invariant", Source: broken}
	}
	b.ports.VisitStorage.Put(next)
	return visit.SignOutVisitorOutcomeSignedOut{VisitorDeparted: visit.VisitorDeparted{VisitId: input.VisitId}}, nil
}

// ExpectedVisits is `gatepass.visit.ExpectedVisits`, generated: every row is one the specification fully determines from the
// stored `gatepass.visit.Visit`s.
func (b *Generated) ExpectedVisits() ([]visit.ExpectedVisits, *obligation.UnmetObligation) {
	listed := b.ports.VisitStorage.List()
	// `filter:` shows a row where it holds; false or unknown hides it.
	var kept []visit.VisitSnapshot
	for _, held := range listed {
		var r0 *string
		switch held.State.(type) {
		case visit.VisitStateDeparted:
			r0 = some("Departed")
		case visit.VisitStateExpected:
			r0 = some("Expected")
		case visit.VisitStateOnSite:
			r0 = some("OnSite")
		}
		if equal(r0, some("Expected")) == verity {
			kept = append(kept, held)
		}
	}
	rows := make([]visit.ExpectedVisits, 0, len(kept))
	for _, held := range kept {
		rows = append(rows, visit.ExpectedVisits{VisitId: held.Data.VisitId, Visitor: held.Data.Visitor, Building: held.Data.Building, Deposit: held.Data.Deposit})
	}
	return rows, nil
}

// VisitById is `gatepass.visit.VisitById`, generated: every row is one the specification fully determines from the
// stored `gatepass.visit.Visit`s.
func (b *Generated) VisitById() ([]visit.VisitById, *obligation.UnmetObligation) {
	listed := b.ports.VisitStorage.List()
	rows := make([]visit.VisitById, 0, len(listed))
	for _, held := range listed {
		rows = append(rows, visit.VisitById{VisitId: held.Data.VisitId, Visitor: held.Data.Visitor, Host: held.Data.Host, Escorts: held.Data.Escorts, Notes: held.Data.Notes, Badge: held.Data.Badge})
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
