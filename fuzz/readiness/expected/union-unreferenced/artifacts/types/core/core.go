// generated from probe v1
// model digest 9e3586ab66a6d980c7c55e1355fe9d10489ca16add86c824b132d00a19a7cfed
// contract digest 89fdf37bf17bc57ca5366cf7422b23f1433c0b3236181f1ae9a4473adb683365
// do not edit: regenerate with `ess synthesize`

// Package core is core — `probe.core`.
//
// Everything this bounded context declares that the synthesis plan marks generated and this
// target can represent. What it cannot is in the TARGET.md beside this module, never absent.
package core

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
