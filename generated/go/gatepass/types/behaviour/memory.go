// generated from gatepass v1
// model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

// Ephemeral storage and context for generated demonstration servers.
package behaviour

import (
	"example.invalid/gatepass/types/visit"
	"reflect"
	"sort"
)

// InMemoryVisitStorage holds ephemeral rows of `gatepass.visit.Visit`.
type InMemoryVisitStorage struct{ rows []visit.VisitSnapshot }

func (s *InMemoryVisitStorage) Get(identity visit.VisitId) (visit.VisitSnapshot, bool) {
	for _, row := range s.rows {
		if reflect.DeepEqual(row.Data.VisitId, identity) {
			return row, true
		}
	}
	return visit.VisitSnapshot{}, false
}
func (s *InMemoryVisitStorage) Put(snapshot visit.VisitSnapshot) {
	for i, row := range s.rows {
		if reflect.DeepEqual(row.Data.VisitId, snapshot.Data.VisitId) {
			s.rows[i] = snapshot
			return
		}
	}
	s.rows = append(s.rows, snapshot)
}
func (s *InMemoryVisitStorage) Delete(identity visit.VisitId) {
	for i, row := range s.rows {
		if reflect.DeepEqual(row.Data.VisitId, identity) {
			s.rows = append(s.rows[:i], s.rows[i+1:]...)
			return
		}
	}
}
func (s *InMemoryVisitStorage) List() []visit.VisitSnapshot {
	rows := append([]visit.VisitSnapshot{}, s.rows...)
	sort.Slice(rows, func(i, j int) bool {
		return memoryLess(reflect.ValueOf(rows[i].Data.VisitId), reflect.ValueOf(rows[j].Data.VisitId))
	})
	return rows
}

// Compare identity representations, including nested newtypes and byte strings.
func memoryLess(a, b reflect.Value) bool {
	switch a.Kind() {
	case reflect.String:
		return a.String() < b.String()
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		return a.Int() < b.Int()
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return a.Uint() < b.Uint()
	case reflect.Bool:
		return !a.Bool() && b.Bool()
	case reflect.Struct:
		for i := 0; i < a.NumField(); i++ {
			if memoryLess(a.Field(i), b.Field(i)) {
				return true
			}
			if memoryLess(b.Field(i), a.Field(i)) {
				return false
			}
		}
	case reflect.Array, reflect.Slice:
		for i := 0; i < a.Len() && i < b.Len(); i++ {
			if memoryLess(a.Index(i), b.Index(i)) {
				return true
			}
			if memoryLess(b.Index(i), a.Index(i)) {
				return false
			}
		}
		return a.Len() < b.Len()
	default:
		panic("unsupported storage identity representation")
	}
	return false
}

// InMemoryContext supplies UUID v4 identities and system-clock timestamps only.
type InMemoryContext struct{}
