package essconform

import (
	"encoding/json"
	"os"
	"testing"
)

// A calendar window over an instant at UTC or a fixed offset (docs/design/calendar-window-guards.md):
// the shared vectors `crates/specify/ess-primitives/tests/vectors/calendar-window.json`, copied
// beside this file, answered by the Go reader as `tests/calendar_window.rs` answers them in Rust
// and `predicate.test.ts` in TypeScript. Each paired fault — host time, the offset ignored, `to`
// inclusive, the midnight crossing attributed to the instant's own day, the written clock read
// instead of the instant — must disagree with a row.

type windowVectors struct {
	Evaluate []struct {
		Name   string         `json:"name"`
		Window map[string]any `json:"window"`
		T      any            `json:"t"`
		Truth  string         `json:"truth"`
	} `json:"evaluate"`
	Refused []struct {
		Name   string         `json:"name"`
		Window map[string]any `json:"window"`
	} `json:"refused"`
	Canonical []struct {
		Name      string         `json:"name"`
		Written   map[string]any `json:"written"`
		Canonical map[string]any `json:"canonical"`
	} `json:"canonical"`
}

func readWindowVectors(t *testing.T) windowVectors {
	raw, err := os.ReadFile("calendar-window.json")
	if err != nil {
		t.Fatalf("vectors: %v", err)
	}
	var vectors windowVectors
	if err := json.Unmarshal(raw, &vectors); err != nil {
		t.Fatalf("vectors: %v", err)
	}
	return vectors
}

func windowRow(value any) factSource {
	if value == nil {
		return facts(Row{})
	}
	return facts(Row{"t": value})
}

func TestCalendarWindow(t *testing.T) {
	names := map[truth]string{truthTrue: "true", truthFalse: "false", truthUnknown: "unknown"}
	vectors := readWindowVectors(t)
	answered := 0
	for _, vector := range vectors.Evaluate {
		parsed, err := fromNode(map[string]any{"window": vector.Window})
		if err != nil {
			t.Errorf("%s: %v", vector.Name, err)
			continue
		}
		if got := names[parsed.evaluate(windowRow(vector.T))]; got != vector.Truth {
			t.Errorf("%s: %s, want %s", vector.Name, got, vector.Truth)
		}
		answered++
	}
	for _, vector := range vectors.Refused {
		if _, err := fromNode(map[string]any{"window": vector.Window}); err == nil {
			t.Errorf("%s: read, want refused", vector.Name)
		}
		answered++
	}
	for _, vector := range vectors.Canonical {
		written, err := fromNode(map[string]any{"window": vector.Written})
		if err != nil {
			t.Errorf("%s: %v", vector.Name, err)
			continue
		}
		canonical, err := fromNode(map[string]any{"window": vector.Canonical})
		if err != nil {
			t.Errorf("%s: canonical: %v", vector.Name, err)
			continue
		}
		if written.String() != canonical.String() {
			t.Errorf("%s: %s, want %s", vector.Name, written, canonical)
		}
		answered++
	}
	if answered < 60 {
		t.Errorf("%d vectors answered", answered)
	}
	// `now` reads no clock here: a runner evaluates no current time.
	now, err := fromNode(map[string]any{"window": map[string]any{"at": "now", "days": []any{"mon"}, "from": "08:00", "to": "16:00", "offset": "Z"}})
	if err != nil {
		t.Fatalf("now: %v", err)
	}
	if got := now.evaluate(facts(Row{"now": "2020-01-06T09:00:00Z"})); got != truthUnknown {
		t.Errorf("a window over now is Unknown without a clock, and a field named `now` is no clock: %s", names[got])
	}
}

// A suite carrying a window is read from suite/40, as the rest of the persisted family F vocabulary;
// a suite relabelled /39 is refused before anything is evaluated.
func TestCalendarWindowSuiteFormat(t *testing.T) {
	node := map[string]any{"window": map[string]any{"at": "ready_at", "days": []any{"mon"}, "from": "08:00", "to": "16:00", "offset": "Z"}}
	if err := admitPredicateVersion(node, 39); err == nil {
		t.Errorf("a window in a suite/39 predicate is admitted")
	}
	if err := admitPredicateVersion(node, 40); err != nil {
		t.Errorf("a window in a suite/40 predicate is refused: %v", err)
	}
	plain := map[string]any{"window": map[string]any{"gte": json.Number("3")}}
	if err := admitPredicateVersion(plain, 39); err != nil {
		t.Errorf("a fact named `window` is refused in suite/39: %v", err)
	}
}

// windowFault answers one vector the way a faulty evaluator would.
type windowFault func(window *windowPredicate, text string) (bool, bool)

func TestCalendarWindowFaults(t *testing.T) {
	vectors := readWindowVectors(t)
	healthy := func(window *windowPredicate, text string) (bool, bool) {
		at, ok := parseInstant(text)
		if !ok {
			return false, false
		}
		return window.contains(at.seconds), true
	}
	shifted := func(minutes int64) windowFault {
		return func(window *windowPredicate, text string) (bool, bool) {
			moved := *window
			moved.offset = minutes
			return healthy(&moved, text)
		}
	}
	faults := []struct {
		name  string
		fault windowFault
	}{
		{"host time at -07:00", shifted(-7 * 60)},
		{"the offset ignored", shifted(0)},
		{"to inclusive", func(window *windowPredicate, text string) (bool, bool) {
			at, ok := parseInstant(text)
			if !ok {
				return false, false
			}
			day, second := window.local(at.seconds)
			if window.from < window.to {
				return window.days[day] && second >= window.from*60 && second <= window.to*60, true
			}
			return healthy(window, text)
		}},
		{"the crossing on the instant's own day", func(window *windowPredicate, text string) (bool, bool) {
			at, ok := parseInstant(text)
			if !ok {
				return false, false
			}
			day, second := window.local(at.seconds)
			if window.from > window.to {
				return window.days[day] && (second >= window.from*60 || second < window.to*60), true
			}
			return healthy(window, text)
		}},
		{"the written clock read", func(window *windowPredicate, text string) (bool, bool) {
			if len(text) < 19 {
				return false, false
			}
			spelled := text[:19] + "Z"
			at, ok := parseInstant(spelled)
			if !ok {
				return false, false
			}
			moved := *window
			moved.offset = 0
			return moved.contains(at.seconds), true
		}},
	}
	answer := func(fault windowFault, window *windowPredicate, value any) string {
		text, isText := value.(string)
		if !isText {
			return "unknown"
		}
		inside, ok := fault(window, text)
		switch {
		case !ok:
			return "unknown"
		case inside:
			return "true"
		default:
			return "false"
		}
	}
	for _, vector := range vectors.Evaluate {
		parsed, err := fromNode(map[string]any{"window": vector.Window})
		if err != nil {
			t.Fatalf("%s: %v", vector.Name, err)
		}
		if got := answer(healthy, parsed.window, vector.T); got != vector.Truth {
			t.Errorf("the healthy reference answers %s with %s", vector.Name, got)
		}
	}
	for _, fault := range faults {
		killed := false
		for _, vector := range vectors.Evaluate {
			parsed, _ := fromNode(map[string]any{"window": vector.Window})
			if answer(fault.fault, parsed.window, vector.T) != vector.Truth {
				killed = true
				break
			}
		}
		if !killed {
			t.Errorf("no vector tells the %s fault apart", fault.name)
		}
	}
}
