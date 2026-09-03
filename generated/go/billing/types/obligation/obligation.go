// generated from billing v3
// model digest 62706dc8de60f859f9fa11d363bae20825e7c74e71435e2fd28691488d787af1
// contract digest d0791c480f462a0bd205e4eda077f60c22bedf0f83756f7ff35687682ce8e3dd
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
