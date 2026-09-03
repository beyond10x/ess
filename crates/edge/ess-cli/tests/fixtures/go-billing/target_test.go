package billing

import (
	"os"
	"testing"

	"essbilling/essconform"
)

// TestConformance is the whole adopter-facing surface: one target, handed over.
//
// `ESS_BREAK` makes the target wrong on purpose, so the test that asserts a defect is caught runs
// the same suite against the same code with one thing changed.
func TestConformance(t *testing.T) {
	essconform.Run(t, func() essconform.Target { return New(os.Getenv("ESS_BREAK")) })
}
