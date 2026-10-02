// generated from gatepass v1
// model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

// Ephemeral generated storage and context.
package server

import (
	"example.invalid/gatepass/types/behaviour"
	"example.invalid/gatepass/types/obligation"
	"example.invalid/gatepass/types/visit"
	"sort"
	"strings"
)

// MemoryContext supplies UUIDs and clock timestamps only.
type MemoryContext struct{}

// MemoryVisitStorage is ephemeral storage of `gatepass.visit.Visit`, in identity order.
type MemoryVisitStorage struct {
	rows []visit.VisitSnapshot
}

func (memoryStore *MemoryVisitStorage) Get(memoryIdentity visit.VisitId) (visit.VisitSnapshot, bool) {
	key := memoryDeclared7(memoryIdentity)
	for _, row := range memoryStore.rows {
		if memoryCompare(memoryDeclared7(row.Data.VisitId), key) == 0 {
			return row, true
		}
	}
	return visit.VisitSnapshot{}, false
}

func (memoryStore *MemoryVisitStorage) Put(memorySnapshot visit.VisitSnapshot) {
	memoryStore.Delete(memorySnapshot.Data.VisitId)
	memoryStore.rows = append(memoryStore.rows, memorySnapshot)
	sort.SliceStable(memoryStore.rows, func(i, j int) bool {
		return memoryCompare(memoryDeclared7(memoryStore.rows[i].Data.VisitId), memoryDeclared7(memoryStore.rows[j].Data.VisitId)) < 0
	})
}

func (memoryStore *MemoryVisitStorage) Delete(memoryIdentity visit.VisitId) {
	key := memoryDeclared7(memoryIdentity)
	for i, row := range memoryStore.rows {
		if memoryCompare(memoryDeclared7(row.Data.VisitId), key) == 0 {
			memoryStore.rows = append(memoryStore.rows[:i], memoryStore.rows[i+1:]...)
			return
		}
	}
}

func (memoryStore *MemoryVisitStorage) List() []visit.VisitSnapshot {
	return append([]visit.VisitSnapshot(nil), memoryStore.rows...)
}

func (*memoryOwed) RegisterVisit(input visit.RegisterVisit) (visit.RegisterVisitOutcome, *obligation.UnmetObligation) {
	return (visit.Unimplemented{}).RegisterVisit(input)
}

type memoryOwed struct{}

// NewMemoryPorts supplies ephemeral storage; clones of this value share its stores.
func NewMemoryPorts() behaviour.Ports {
	return behaviour.Ports{
		VisitStorage: &MemoryVisitStorage{},
		Owed:         &memoryOwed{},
	}
}

func memoryDeclared7(value visit.VisitId) memoryKey {
	return memoryKey{kind: 4, text: (value.Value()).Value()}
}

// memoryInvalidRaw is disjoint from every successfully decoded model value.
const memoryInvalidRaw = 9

type memoryKey struct {
	kind    int
	flag    bool
	integer int64
	text    string
	items   []memoryKey
}

func memoryCompare(left, right memoryKey) int {
	if left.kind < right.kind {
		return -1
	}
	if left.kind > right.kind {
		return 1
	}
	switch left.kind {
	case 1:
		if left.flag != right.flag {
			if left.flag {
				return 1
			}
			return -1
		}
	case 2:
		if left.integer < right.integer {
			return -1
		}
		if left.integer > right.integer {
			return 1
		}
	case 3, 4, 5, memoryInvalidRaw:
		return strings.Compare(left.text, right.text)
	case 6, 7, 8:
		for i := 0; i < len(left.items) && i < len(right.items); i++ {
			if order := memoryCompare(left.items[i], right.items[i]); order != 0 {
				return order
			}
		}
		if len(left.items) < len(right.items) {
			return -1
		}
		if len(left.items) > len(right.items) {
			return 1
		}
	}
	return 0
}
