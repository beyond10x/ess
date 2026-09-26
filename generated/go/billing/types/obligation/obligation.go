// generated from billing v3
// model digest 1e7906786567af32118eb2d0a8c3fcafa16c32c9649a80b60487fd2eeebc4c9c
// contract digest a21fd36f0055057629f4c235962163cdd34a3d178aa068925bcb53be623af301
// do not edit: regenerate with `ess synthesize`

// Package obligation carries the typed refusal of an unmet obligation.
//
// An obligation is a capability the synthesis plan owes the implementor — the contract is
// declared, the behaviour is not. Until an implementation satisfies one, its stub returns
// [UnmetObligation]: a value naming the plan entry, never a panic and never a guess, so a
// module built on stubs compiles and reports its own gaps.
//
// Its own package, and one that imports nothing from this module: Go refuses an import
// cycle where Rust allows a module cycle, and every bounded context's package has to name
// this type.
package obligation

import (
	"fmt"
)

// UnmetObligation is a capability the synthesis plan owes and nothing has satisfied yet.
//
// The two fields spell the plan entry: look the pair up in PLAN.md for the contract being
// refused. A satisfying implementation never constructs one.
type UnmetObligation struct {
	// Capability is the capability kind, as the plan spells it.
	Capability string
	// Source is the construct that requires it, in the specification's own spelling.
	Source string
}

// Error names the plan entry being refused.
func (u *UnmetObligation) Error() string {
	return fmt.Sprintf("unmet obligation: %s `%s` — see PLAN.md", u.Capability, u.Source)
}
