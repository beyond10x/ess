// generated from probe v1
// model digest 27242364f02f9693903e3e9667e3538f90f1b128a4b5f2e479b78c733c767ab3
// contract digest 6593e243bf4d7e9c281af5622aca66399d95e84990e6f5aa29adfbd094dcf58f
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

// Value is Value — `probe.core.Value`: one of a fixed set of shapes, tagged on the wire by `kind`.
//
// A closed set: the marker method below is unexported, so no type outside this package can
// join it. Go cannot check that a `switch` over it handles every case — that is a target-stage
// weakening of what the specification declares, recorded in TARGET.md, not a gap in the model.
type Value interface {
	isValue()
}

// ValueCount is the shape tagged `count` — `Integer`.
type ValueCount struct {
	// Value is what this shape carries.
	Value int64
}

func (ValueCount) isValue() {}

// ValueText is the shape tagged `text` — `String`.
type ValueText struct {
	// Value is what this shape carries.
	Value string
}

func (ValueText) isValue() {}
