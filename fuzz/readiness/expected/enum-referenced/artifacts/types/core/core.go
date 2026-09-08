// generated from probe v1
// model digest 912d72c184b3a81de43f53374b05569f6dd5828d33749a060503d0337427347c
// contract digest b87b9f3c394c7cb86e46260d678860e21a1f86fd07126eecd174c00b12353675
// do not edit: regenerate with `ess synthesize`

// Package core is core — `probe.core`.
//
// Everything this bounded context declares that the synthesis plan marks generated and this
// target can represent. What it cannot is in the TARGET.md beside this module, never absent.
package core

// Holder is Holder — `probe.core.Holder`.
type Holder struct {
	// Value is `value` — `probe.core.Value`.
	Value Value
}

// Value is Value — `probe.core.Value`: one of a closed set of names.
//
// A closed set: the marker method below is unexported, so no type outside this package can
// join it. Go cannot check that a `switch` over it handles every case — that is a target-stage
// weakening of what the specification declares, recorded in TARGET.md, not a gap in the model.
type Value interface {
	isValue()
}

// ValueFirst is `First`.
type ValueFirst struct{}

func (ValueFirst) isValue() {}

// ValueSecond is `Second`.
type ValueSecond struct{}

func (ValueSecond) isValue() {}
