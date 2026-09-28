package essconform

import (
	"strings"
	"testing"
)

var suiteJSON = `{}`

// TestPresenceSuitesAreAdmitted: suite/24 and /25 (beyond10x/ess#139) are read, not refused by
// version, since the leaf's `presence` key is carried on Held (beyond10x/ess#188).
func TestPresenceSuitesAreAdmitted(t *testing.T) {
	for _, version := range []string{"ess-conformance/24", "ess-conformance/25"} {
		document := `{"provenance":{"suite_version":"` + version + `","system":"example",` +
			`"specification_version":"v1","spec_digest":"` + strings.Repeat("a", 64) +
			`","contract_digest":"` + strings.Repeat("b", 64) + `"},"scenarios":{}}`
		_, err := admitSuite(document)
		if version == "ess-conformance/25" {
			// A coverage major without its inventory is refused for that, never by version.
			if err == nil || strings.Contains(err.Error(), "unsupported suite version") {
				t.Fatalf("%s: %v", version, err)
			}
			continue
		}
		if err != nil {
			t.Fatalf("%s was refused: %v", version, err)
		}
	}
}

// TestPresencePolicyDecidesNullAndAbsence: the vectors ess_conformance's LeafShape::admits answers
// in tests/field_presence.rs, for a leaf reached directly and one under a struct.
func TestPresencePolicyDecidesNullAndAbsence(t *testing.T) {
	text := Held{Holds: "primitive", Kind: "string", Optional: true}
	nullWhenAbsent, omittedWhenAbsent := text, text
	nullWhenAbsent.Presence = "null_when_absent"
	omittedWhenAbsent.Presence = "omitted_when_absent"
	for _, vector := range []struct {
		name         string
		leaf         Held
		absent, null bool
	}{
		{"null_when_absent", nullWhenAbsent, false, true},
		{"omitted_when_absent", omittedWhenAbsent, true, false},
		{"no policy", text, true, true},
	} {
		for _, path := range []string{"field", "receipt.field"} {
			shape := map[string]Held{path: vector.leaf}
			absent := map[string]Node{"receipt": map[string]any{}}
			null := map[string]Node{"field": nil, "receipt": map[string]any{"field": nil}}
			value := map[string]Node{"field": "x", "receipt": map[string]any{"field": "x"}}
			wrong := map[string]Node{"field": true, "receipt": map[string]any{"field": true}}
			if got := holds(absent, shape) == ""; got != vector.absent {
				t.Errorf("%s at %s: an absent key admitted = %v", vector.name, path, got)
			}
			if got := holds(null, shape) == ""; got != vector.null {
				t.Errorf("%s at %s: an explicit null admitted = %v", vector.name, path, got)
			}
			if reason := holds(value, shape); reason != "" {
				t.Errorf("%s at %s: a value refused: %s", vector.name, path, reason)
			}
			if holds(wrong, shape) == "" {
				t.Errorf("%s at %s: a wrong value admitted", vector.name, path)
			}
		}
	}
}

// TestPresenceBelowSuite24IsRefused: a presence key under an older major is vocabulary that
// major does not have.
func TestPresenceBelowSuite24IsRefused(t *testing.T) {
	err := admitShape(map[string]any{"field": map[string]any{
		"holds": "primitive", "kind": "string", "optional": true, "presence": "null_when_absent",
	}}, 22)
	if err == nil || !strings.Contains(err.Error(), "suite/24") {
		t.Fatalf("admitted under suite/22: %v", err)
	}
	if err := admitShape(map[string]any{"field": map[string]any{
		"holds": "primitive", "kind": "string", "optional": true, "presence": "sometimes",
	}}, 24); err == nil {
		t.Fatalf("an unknown policy was admitted")
	}
}
