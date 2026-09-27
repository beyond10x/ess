package essconform

import "testing"

// beyond10x/ess#176: a present struct, list or map is present even when it is empty; null and a
// left-out member are absent. The Rust lanes and `predicate.test.ts` answer the same vectors.
func TestDefinedAggregatePresence(t *testing.T) {
	defined, err := parseLeaf("defined(metrics)")
	if err != nil {
		t.Fatal(err)
	}
	for _, vector := range []struct {
		name string
		row  Row
		want truth
	}{
		{"a struct", Row{"metrics": map[string]any{"waiting": float64(3)}}, truthTrue},
		{"an empty struct", Row{"metrics": map[string]any{}}, truthTrue},
		{"an empty list", Row{"metrics": []any{}}, truthTrue},
		{"a struct under a present parent", Row{"queue": map[string]any{"metrics": map[string]any{}}}, truthFalse},
		{"null", Row{"metrics": nil}, truthFalse},
		{"left out", Row{}, truthFalse},
		{"a scalar", Row{"metrics": "x"}, truthTrue},
	} {
		if got := defined.evaluate(facts(vector.row)); got != vector.want {
			t.Errorf("defined(metrics) over %s: got %v, want %v", vector.name, got, vector.want)
		}
	}
	nested, err := parseLeaf("defined(queue.metrics)")
	if err != nil {
		t.Fatal(err)
	}
	if got := nested.evaluate(facts(Row{"queue": map[string]any{"metrics": map[string]any{}}})); got != truthTrue {
		t.Errorf("defined(queue.metrics) over a nested empty struct: got %v", got)
	}
}

func TestDefinedAggregateNoValueReadSeesTheMark(t *testing.T) {
	row := facts(Row{"metrics": map[string]any{"waiting": float64(3)}, "tags": []any{}})
	for _, expression := range []string{"metrics", "metrics == 1", "tags"} {
		leaf, err := parseLeaf(expression)
		if err != nil {
			t.Fatal(err)
		}
		if got := leaf.evaluate(row); got != truthUnknown {
			t.Errorf("%s over a present aggregate: got %v, want unknown", expression, got)
		}
	}
	count, err := parseLeaf("tags.count == 0")
	if err != nil {
		t.Fatal(err)
	}
	if got := count.evaluate(row); got != truthTrue {
		t.Errorf("tags.count == 0: got %v", got)
	}
}

func TestDefinedAggregateInvariant(t *testing.T) {
	invariant, err := fromNode(map[string]any{"any": []any{"state == Paused", map[string]any{"not": "defined(metrics)"}}})
	if err != nil {
		t.Fatal(err)
	}
	for _, vector := range []struct {
		name string
		row  Row
		want truth
	}{
		{"paused with metrics", Row{"state": "Paused", "metrics": map[string]any{"waiting": float64(1)}}, truthTrue},
		{"running without metrics", Row{"state": "Running"}, truthTrue},
		{"running with null metrics", Row{"state": "Running", "metrics": nil}, truthTrue},
		{"running with metrics", Row{"state": "Running", "metrics": map[string]any{"waiting": float64(1)}}, truthFalse},
		{"running with empty metrics", Row{"state": "Running", "metrics": map[string]any{}}, truthFalse},
	} {
		if got := invariant.evaluate(facts(vector.row)); got != vector.want {
			t.Errorf("%s: got %v, want %v", vector.name, got, vector.want)
		}
	}
}

func TestDefinedAggregateThroughABinder(t *testing.T) {
	row := facts(Row{"queues": []any{map[string]any{"metrics": map[string]any{}}, map[string]any{"state": "Running"}}})
	some, err := fromNode(map[string]any{"exists": map[string]any{"in": "queues", "as": "q", "that": "defined(q.metrics)"}})
	if err != nil {
		t.Fatal(err)
	}
	every, err := fromNode(map[string]any{"forall": map[string]any{"in": "queues", "as": "q", "that": "defined(q.metrics)"}})
	if err != nil {
		t.Fatal(err)
	}
	element, err := fromNode(map[string]any{"exists": map[string]any{"in": "queues", "as": "q", "that": "defined(q)"}})
	if err != nil {
		t.Fatal(err)
	}
	if got := some.evaluate(row); got != truthTrue {
		t.Errorf("exists: got %v", got)
	}
	if got := every.evaluate(row); got != truthFalse {
		t.Errorf("forall: got %v", got)
	}
	if got := element.evaluate(row); got != truthTrue {
		t.Errorf("defined(q) over a struct element: got %v", got)
	}
}
