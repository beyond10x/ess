// generated from probe v1
// model digest 7df1896edb53ff9b9cc3be6b5af4b11c3c6eeb2e81db899084e39a3c9bd68397
// contract digest c1d09d06bfa466b5c69362996c174a9621e3627064e1d9e602c3126135f11832
// do not edit: regenerate with `ess synthesize`

// Package core is core — `probe.core`.
//
// Everything this bounded context declares that the synthesis plan marks generated and this
// target can represent. What it cannot is in the TARGET.md beside this module, never absent.
package core

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
