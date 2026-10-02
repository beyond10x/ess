// Package realization is the hand-written half of the synthesised gatepass Go module.
//
// `generated/go/gatepass/` holds the types, the typestate lifecycle, the component port, the system,
// the HTTP surface, and the behaviour of every command and view the specification fully determines
// — `AdmitVisitor`, `SignOutVisitor`, `ExpectedVisits` and `VisitById`, in its `behaviour` package,
// written against storage it does not provide. What is left is what the plan owes: the
// `RegisterVisit` behaviour, whose visit the specification does not fully describe, and the store
// every behaviour reads.
//
// This module is that half, and it is deliberately *not* a translation of
// `examples/gatepass-realization/`. Both were written from the same specification, in the language
// of the tree each links into, and the demonstration is that the two answer the same requests the
// same way.
//
// # No clock, no randomness
//
// Identifiers come from a per-store counter in the Uuid wire shape, for the reason the Rust half
// gives: two processes synthesised from one specification are started side by side and their
// answers compared, so an identifier from a random source would make the two disagree about a
// value neither of them chose.
package realization

import (
	"fmt"
	"sort"

	"example.invalid/gatepass/types/obligation"
	"example.invalid/gatepass/types/primitives"
	"example.invalid/gatepass/types/visit"
)

// Store is what one run's visits amount to: every snapshot, and the identifier mint. It is the
// generated behaviours' `VisitStorage`.
//
// Keyed by the identifier's wire rendering and listed in sorted order, so the generated queries
// answer in a stable order — the same order the Rust realization's BTreeMap gives, which is what
// lets two processes be compared row by row.
type Store struct {
	visits   map[string]visit.VisitSnapshot
	sequence int64
}

// NewStore is an empty store.
func NewStore() *Store {
	return &Store{visits: map[string]visit.VisitSnapshot{}}
}

// identifier is a fresh identity in the Uuid wire shape, from the counter rather than randomness.
func (s *Store) identifier() visit.VisitId {
	s.sequence++
	return visit.NewVisitId(primitives.NewUuid(fmt.Sprintf("00000000-0000-4000-8000-%012d", s.sequence)))
}

// Get is the visit stored under identity.
func (s *Store) Get(identity visit.VisitId) (visit.VisitSnapshot, bool) {
	held, ok := s.visits[identity.Value().Value()]
	return held, ok
}

// Put stores snapshot under its identity, replacing what was held.
func (s *Store) Put(snapshot visit.VisitSnapshot) {
	s.visits[snapshot.Data.VisitId.Value().Value()] = snapshot
}

// Delete removes the visit stored under identity.
func (s *Store) Delete(identity visit.VisitId) {
	delete(s.visits, identity.Value().Value())
}

// List is every stored visit, in the order of its identifier's wire rendering.
func (s *Store) List() []visit.VisitSnapshot {
	keys := make([]string, 0, len(s.visits))
	for key := range s.visits {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	rows := make([]visit.VisitSnapshot, 0, len(keys))
	for _, key := range keys {
		rows = append(rows, s.visits[key])
	}
	return rows
}

// Realization is the honest implementation of the one behaviour the plan owes, over the store the
// generated behaviours read.
type Realization struct {
	store *Store
}

// Over is the realization, answering over store.
func Over(store *Store) *Realization {
	return &Realization{store: store}
}

// RegisterVisit decides and enacts exactly one declared outcome of `gatepass.visit.RegisterVisit`.
func (r *Realization) RegisterVisit(input visit.RegisterVisit) (visit.RegisterVisitOutcome, *obligation.UnmetObligation) {
	// The declared guard, first and alone: `registered` when `expected_minutes > 0`, `refused`
	// otherwise. Nothing else about the input can refuse a registration.
	if input.ExpectedMinutes <= 0 {
		return visit.RegisterVisitOutcomeRefused{
			Error: visit.InvalidVisitLength{Submitted: input.ExpectedMinutes},
		}, nil
	}
	visitID := r.store.identifier()
	// What the command does not determine, the realization decides and says so: there is no badge
	// until one is printed at the desk, which is what AdmitVisitor carries.
	registered := visit.NewVisit(visit.VisitData{
		VisitId:         visitID,
		Visitor:         input.Visitor,
		Building:        input.Building,
		Host:            input.Host,
		ExpectedMinutes: input.ExpectedMinutes,
		ExpectedStay:    input.ExpectedStay,
		Deposit:         input.Deposit,
		Escorts:         input.Escorts,
		Notes:           input.Notes,
		Badge:           nil,
		OnWatchlist:     input.OnWatchlist,
	})
	r.store.Put(registered.Snapshot())
	return visit.RegisterVisitOutcomeRegistered{
		VisitRegistered: visit.VisitRegistered{
			VisitId:  visitID,
			Visitor:  input.Visitor,
			Building: input.Building,
		},
	}, nil
}
