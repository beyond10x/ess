// generated from probe v1
// model digest 9f37f3209dbfbd3a8c9bc418447dc9e6b47896c47291caa29ab4aad5ec87a8e5
// contract digest a8ba924ad32ac1bf2d13666ea1a6c867907744384a5ac06982d20bd5ca16c03c
// do not edit: regenerate with `ess synthesize`

// Package core is core — `probe.core`.
//
// Everything this bounded context declares that the synthesis plan marks generated and this
// target can represent. What it cannot is in the TARGET.md beside this module, never absent.
package core

// Value is Value — `probe.core.Value`: a distinct wrapper around `String`.
//
// The field is unexported, so the only way to make one carrying a value is [NewValue] —
// a defined type over `string` would have let an untyped constant be assigned straight to
// Value, which is the distinctness this declaration exists for. Go's zero value still
// needs no constructor (see TARGET.md).
type Value struct {
	value string
}

// NewValue wraps a `String` as Value.
func NewValue(value string) Value {
	return Value{value: value}
}

// Value is the wrapped `String`.
func (v Value) Value() string {
	return v.value
}
