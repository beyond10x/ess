// generated from probe v1
// model digest cfb42bb0c2ac70192da4218f5f8ffe6feeb57f987e1e94c1db8ea73772a52f9f
// contract digest 051ee7ce781460260a1ecff05dae61e225f09f1763ce92b27a2497940170b398
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
