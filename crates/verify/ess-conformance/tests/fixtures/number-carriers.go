package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"testing"
)

// The table `tests/primitive_corpus.rs` writes: every corpus number in every Go type that carries
// the value Rust's `Number` decides it is, and Rust's own ordering of every pair (beyond10x/ess#101).
type carrierTable struct {
	Numbers []struct {
		Name     string           `json:"name"`
		Display  string           `json:"display"`
		Integer  bool             `json:"integer"`
		Carriers []numberCarriage `json:"carriers"`
	} `json:"numbers"`
	Pairs []struct {
		Left     string `json:"left"`
		Right    string `json:"right"`
		Ordering int    `json:"ordering"`
	} `json:"pairs"`
}

type numberCarriage struct {
	Type string `json:"type"`
	Text string `json:"text"`
}

func (c numberCarriage) String() string { return c.Type + "(" + c.Text + ")" }

// carry builds the Go value an implementation would return.
func carry(t *testing.T, c numberCarriage) Node {
	t.Helper()
	signed := func(bits int) int64 {
		value, err := strconv.ParseInt(c.Text, 10, bits)
		if err != nil {
			t.Fatalf("%s: %v", c, err)
		}
		return value
	}
	unsigned := func(bits int) uint64 {
		value, err := strconv.ParseUint(c.Text, 10, bits)
		if err != nil {
			t.Fatalf("%s: %v", c, err)
		}
		return value
	}
	switch c.Type {
	case "json.Number":
		return json.Number(c.Text)
	case "float64", "float32":
		bits := 64
		if c.Type == "float32" {
			bits = 32
		}
		value, err := strconv.ParseFloat(c.Text, bits)
		if err != nil {
			t.Fatalf("%s: %v", c, err)
		}
		if bits == 32 {
			return float32(value)
		}
		return value
	case "int":
		return int(signed(0))
	case "int8":
		return int8(signed(8))
	case "int16":
		return int16(signed(16))
	case "int32":
		return int32(signed(32))
	case "int64":
		return signed(64)
	case "uint":
		return uint(unsigned(0))
	case "uint8":
		return uint8(unsigned(8))
	case "uint16":
		return uint16(unsigned(16))
	case "uint32":
		return uint32(unsigned(32))
	case "uint64":
		return unsigned(64)
	}
	t.Fatalf("%s is a carrier this test does not build", c)
	return nil
}

func readCarriers(t *testing.T) carrierTable {
	t.Helper()
	raw, err := os.ReadFile("number-carriers.json")
	if err != nil {
		t.Fatal(err)
	}
	var table carrierTable
	if err := json.Unmarshal(raw, &table); err != nil {
		t.Fatal(err)
	}
	return table
}

// TestEveryNumberCarrierComparesByTheValueRustDecides asks each comparison the runtime makes of an
// implementation's values — a payload or row field, a nested list or object, a response value, an
// ordering, a predicate — about every pair of corpus numbers in every carrier pair.
func TestEveryNumberCarrierComparesByTheValueRustDecides(t *testing.T) {
	table := readCarriers(t)
	carriers := map[string][]numberCarriage{}
	for _, number := range table.Numbers {
		carriers[number.Name] = number.Carriers
	}
	checked, crossTyped := 0, 0
	for _, pair := range table.Pairs {
		same := pair.Ordering == 0
		for _, lc := range carriers[pair.Left] {
			for _, rc := range carriers[pair.Right] {
				left, right := carry(t, lc), carry(t, rc)
				label := fmt.Sprintf("%s %s vs %s %s (Rust: %d)", pair.Left, lc, pair.Right, rc, pair.Ordering)
				if got := equal(left, right); got != same {
					t.Errorf("equal: %s: got %v", label, got)
				}
				if got := matches(Row{"v": left}, map[string]Node{"v": right}); got != same {
					t.Errorf("matches: %s: got %v", label, got)
				}
				if got := equal([]any{left}, []any{right}); got != same {
					t.Errorf("equal list: %s: got %v", label, got)
				}
				if got := equal(map[string]any{"v": left}, map[string]any{"v": right}); got != same {
					t.Errorf("equal object: %s: got %v", label, got)
				}
				if got := responseEqual(left, right); got != same {
					t.Errorf("responseEqual: %s: got %v", label, got)
				}
				if order, ok := compare(left, right); !ok || order != pair.Ordering {
					t.Errorf("compare: %s: got %d, %v", label, order, ok)
				}
				for op, holds := range map[string]bool{
					"==": same, "!=": !same,
					"<": pair.Ordering < 0, "<=": pair.Ordering <= 0,
					">": pair.Ordering > 0, ">=": pair.Ordering >= 0,
				} {
					leaf, err := parseLeaf("row.left " + op + " row.right")
					if err != nil {
						t.Fatal(err)
					}
					if got := leaf.evaluate(factSource{"row.left": left, "row.right": right}); got != truthOf(holds) {
						t.Errorf("predicate %s: %s: got %v", op, label, got)
					}
				}
				checked++
				if lc.Type != rc.Type {
					crossTyped++
				}
			}
		}
	}
	if checked < 1000 || crossTyped < 500 {
		t.Fatalf("the table selected %d carrier pairs, %d of them across two types", checked, crossTyped)
	}
	t.Logf("%d carrier pairs, %d across two types", checked, crossTyped)
}

// TestEveryNumberCarrierIsAdmittedAsTheKindItHolds keeps the type check in step with the
// comparison: a carrier the comparison accepts is a number to the shape check too.
func TestEveryNumberCarrierIsAdmittedAsTheKindItHolds(t *testing.T) {
	for _, number := range readCarriers(t).Numbers {
		for _, c := range number.Carriers {
			value := carry(t, c)
			if reason := primitive("decimal", value); reason != "" {
				t.Errorf("%s %s as decimal: %s", number.Name, c, reason)
			}
			if admitted := primitive("integer", value) == ""; admitted != number.Integer {
				t.Errorf("%s %s as integer: admitted=%v, Rust says %v", number.Name, c, admitted, number.Integer)
			}
		}
	}
}

// TestSuiteExpectationsMatchWhateverTypeCarriesTheNumber is the call site the defect was seen at:
// an `expect_event` payload and an `expect_view` row, with the expectation decoded as each suite
// major decodes it — `float64` or `json.Number` — and the implementation's value in every carrier.
func TestSuiteExpectationsMatchWhateverTypeCarriesTheNumber(t *testing.T) {
	checked := 0
	for _, number := range readCarriers(t).Numbers {
		binary := false
		for _, c := range number.Carriers {
			binary = binary || c.Type == "float64"
		}
		event := `{"step":"expect_event","event":"example.Recorded","payload":{"v":` + number.Display + `}}`
		view := `{"step":"expect_view","view":"example.Rows","expectation":{"expect":"contains","fields":{"v":{"kind":"literal","value":` + number.Display + `}}}}`
		for _, exact := range []bool{true, false} {
			// A suite decoded into float64 already holds only the binary64, so only a number
			// that survives it is asked of that decoding.
			if !exact && !binary {
				continue
			}
			// decodeStep is the runtime's own step reader: exact for suite/12 and later, the
			// legacy float64 decoding before it.
			decode := func(raw string) Step {
				step, err := decodeStep([]byte(raw), exact)
				if err != nil {
					t.Fatal(err)
				}
				return step
			}
			eventStep, viewStep := decode(event), decode(view)
			for _, c := range number.Carriers {
				value := carry(t, c)
				label := fmt.Sprintf("%s %s against a suite decoded exact=%v", number.Name, c, exact)
				r := &run{t: t, observed: map[string][]ObservedEvent{
					"example.Recorded": {{Event: "example.Recorded", Payload: map[string]Node{"v": value}}},
				}}
				if !r.expectEvent(0, eventStep) {
					t.Errorf("expect_event: %s", label)
				}
				r = &run{t: t, lastView: ViewResult{Rows: []Row{{"v": value}}}}
				if holds, reason, _ := r.decide(0, viewStep); !holds {
					t.Errorf("expect_view: %s: %s", label, reason)
				}
				checked++
			}
		}
	}
	if checked < 50 {
		t.Fatalf("the table selected %d expectations", checked)
	}
}
