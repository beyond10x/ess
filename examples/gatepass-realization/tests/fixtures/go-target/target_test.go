package gatepass

import (
	"testing"

	"essgatepass/essconform"
)

// TestConformance is the whole adopter-facing surface: one target, handed over.
func TestConformance(t *testing.T) {
	essconform.Run(t, func() essconform.Target { return New() })
}
