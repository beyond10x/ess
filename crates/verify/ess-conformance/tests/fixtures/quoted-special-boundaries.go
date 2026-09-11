package essconform

import (
	"encoding/json"
	"math"
	"testing"
)

// These are literal spellings, not JSON numeric tokens. Rust accepts finite decimal f64 syntax
// here; Go's NaN/infinity, hexadecimal and underscore extensions must not turn text into numbers.
func TestQuotedSpecialAndDecimalBoundaries(t *testing.T) {
	for _, text := range []string{"Ready", "NaN", "nan", "NAN", "Inf", "+Inf", "-Inf", "Infinity", "+Infinity", "-Infinity", "infinity", "0x1p2", "0x1.8p+1", "-0x1.8p+1", "1_000", "1e400", "-1e400"} {
		t.Run(text, func(t *testing.T) {
			original := map[string]any{"to": map[string]any{"eq": text}}
			raw, _ := json.Marshal(original)
			structured, err := parsePredicate(raw)
			if err != nil {
				t.Fatal(err)
			}
			compact, err := parseLeaf("to == " + text)
			if err != nil {
				t.Fatal(err)
			}
			for _, parsed := range []predicate{structured, compact} {
				if parsed.right.isFact || parsed.right.literal != text {
					t.Fatalf("got %#v, want text %q", parsed.right, text)
				}
				if parsed.evaluate(factSource{"to": text}) != truthTrue {
					t.Fatal("text equality failed")
				}
			}
			if err := admitPredicateVersion(original, 7); err != nil {
				t.Fatal("unchanged text refused under old header", err)
			}
		})
	}
	controls := []struct {
		text string
		want float64
	}{
		{"0", 0}, {"-0", math.Copysign(0, -1)}, {"+1", 1}, {"-1.25", -1.25},
		{".5", .5}, {"1.", 1}, {"1e3", 1000}, {"+1.25E-2", .0125}, {"-2e+2", -200}, {"1e-400", 0},
	}
	for _, control := range controls {
		t.Run(control.text, func(t *testing.T) {
			original := map[string]any{"to": map[string]any{"eq": control.text}}
			raw, _ := json.Marshal(original)
			structured, err := parsePredicate(raw)
			if err != nil {
				t.Fatal(err)
			}
			compact, err := parseLeaf("to == " + control.text)
			if err != nil {
				t.Fatal(err)
			}
			for _, parsed := range []predicate{structured, compact} {
				value, ok := parsed.right.literal.(float64)
				if !ok || value != control.want || math.Signbit(value) != math.Signbit(control.want) {
					t.Fatalf("got %#v, want %v", parsed.right, control.want)
				}
				if parsed.evaluate(factSource{"to": control.want}) != truthTrue {
					t.Fatal("number equality failed")
				}
			}
			if err := admitPredicateVersion(original, 7); err == nil {
				t.Fatal("changed structured scalar admitted under old header")
			}
			if err := admitPredicateVersion(original, 8); err != nil {
				t.Fatal(err)
			}
			if err := admitPredicateVersion("to == "+control.text, 7); err != nil {
				t.Fatal("legacy compact number refused", err)
			}
		})
	}
}
