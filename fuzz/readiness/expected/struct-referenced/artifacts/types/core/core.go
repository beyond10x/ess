// generated from probe v1
// model digest 0225cbad71bc25c994b43236bfead973b1cab195258528b14cf3b3cd6ad1f43c
// contract digest 89b8b411c78528af906761f8df7658afc2217b1f56d5f15fa7ff20c830b8247c
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

// Value is Value — `probe.core.Value`.
type Value struct {
	// Text is `text` — `String`.
	Text string
}
