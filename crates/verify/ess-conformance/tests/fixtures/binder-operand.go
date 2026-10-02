package essconform

import (
	"encoding/json"
	"testing"
)

// beyond10x/ess#289: a bare word naming a quantifier binder in scope, on the right of a comparison,
// reads the binder, as `Predicate::from_node` does on the Rust side; a quoted one, the equality
// shorthand and a word naming no binder in scope stay text. `predicate.test.ts` answers the same
// vectors.
func TestBinderOperand(t *testing.T) {
	disjoint := `{"forall": {"in": "tags", "as": "tag", "that": {"forall": {"in": "banned", "as": "b", "that": "tag != b"}}}}`
	operator := `{"forall": {"in": "tags", "as": "tag", "that": {"forall": {"in": "banned", "as": "b", "that": {"tag": {"ne": "b"}}}}}}`
	quoted := `{"forall": {"in": "tags", "as": "tag", "that": {"forall": {"in": "banned", "as": "b", "that": "tag != \"b\""}}}}`
	shorthand := `{"forall": {"in": "tags", "as": "tag", "that": {"forall": {"in": "banned", "as": "b", "that": {"tag": "b"}}}}}`
	outOfScope := `{"all": [{"forall": {"in": "banned", "as": "b", "that": "b != x"}}, {"forall": {"in": "tags", "as": "tag", "that": "tag != b"}}]}`
	row := func(tags, banned []any) Row { return Row{"tags": tags, "banned": banned} }
	for _, vector := range []struct {
		name      string
		predicate string
		row       Row
		want      truth
	}{
		{"disjoint lists", disjoint, row([]any{"x", "y"}, []any{"z"}), truthTrue},
		{"a shared item", disjoint, row([]any{"x"}, []any{"x"}), truthFalse},
		{"a shared item named like the binder", disjoint, row([]any{"x", "b"}, []any{"b"}), truthFalse},
		{"the operator spelling, disjoint", operator, row([]any{"x"}, []any{"z"}), truthTrue},
		{"the operator spelling, shared", operator, row([]any{"x"}, []any{"x"}), truthFalse},
		{"a quoted word is text", quoted, row([]any{"x"}, []any{"x"}), truthTrue},
		{"a quoted word is text, matching", quoted, row([]any{"b"}, []any{"z"}), truthFalse},
		{"the shorthand is text", shorthand, row([]any{"b"}, []any{"z"}), truthTrue},
		{"the shorthand is text, not matching", shorthand, row([]any{"x"}, []any{"x"}), truthFalse},
		{"out of scope is text", outOfScope, row([]any{"b"}, []any{"z"}), truthFalse},
		{"out of scope is text, not matching", outOfScope, row([]any{"x"}, []any{"z"}), truthTrue},
	} {
		parsed, err := parsePredicate(json.RawMessage(vector.predicate))
		if err != nil {
			t.Fatalf("%s: %v", vector.name, err)
		}
		if got := parsed.evaluate(facts(vector.row)); got != vector.want {
			t.Errorf("%s: %s over %v: got %v, want %v", vector.name, vector.predicate, vector.row, got, vector.want)
		}
	}
}
