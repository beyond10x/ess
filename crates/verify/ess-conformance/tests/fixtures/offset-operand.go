package essconform

import (
	"encoding/json"
	"fmt"
	"math"
	"math/big"
	"os"
	"strings"
	"testing"
)

// One constant offset on a right-hand fact (docs/design/expression-family-source22.md, A2): the
// shared vectors `crates/specify/ess-primitives/tests/vectors/offset-operand.json`, copied beside
// this file, answered by the Go reader as `tests/offset_operand.rs` answers them in Rust and
// `predicate.test.ts` in TypeScript. Each paired fault — wrapping, clamping, binary64, Unknown on
// overflow, an ignored sign, an ignored base, field minus field — must disagree with a row.

type offsetRow struct {
	Name      string          `json:"name"`
	Left      json.RawMessage `json:"left"`
	Base      json.RawMessage `json:"base"`
	Direction string          `json:"direction"`
	Magnitude json.RawMessage `json:"magnitude"`
	Op        string          `json:"op"`
	Truth     string          `json:"truth"`
}

func offsetVectors(t *testing.T) (rows []offsetRow, evaluate []struct {
	Name      string          `json:"name"`
	Predicate json.RawMessage `json:"predicate"`
	Row       json.RawMessage `json:"row"`
	Truth     string          `json:"truth"`
}, refused []struct {
	Name      string          `json:"name"`
	Predicate json.RawMessage `json:"predicate"`
}) {
	raw, err := os.ReadFile("offset-operand.json")
	if err != nil {
		t.Fatalf("vectors: %v", err)
	}
	var vectors struct {
		Integer   []offsetRow `json:"integer"`
		Timestamp []offsetRow `json:"timestamp"`
		Evaluate  []struct {
			Name      string          `json:"name"`
			Predicate json.RawMessage `json:"predicate"`
			Row       json.RawMessage `json:"row"`
			Truth     string          `json:"truth"`
		} `json:"evaluate"`
		Refused []struct {
			Name      string          `json:"name"`
			Predicate json.RawMessage `json:"predicate"`
		} `json:"refused"`
	}
	if err := json.Unmarshal(raw, &vectors); err != nil {
		t.Fatalf("vectors: %v", err)
	}
	return append(vectors.Integer, vectors.Timestamp...), vectors.Evaluate, vectors.Refused
}

func decodeRow(t *testing.T, name string, raw []byte) Row {
	var row Row
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.UseNumber()
	if err := decoder.Decode(&row); err != nil {
		t.Fatalf("%s: row: %v", name, err)
	}
	return row
}

func TestOffsetOperand(t *testing.T) {
	names := map[truth]string{truthTrue: "true", truthFalse: "false", truthUnknown: "unknown"}
	rows, evaluate, refused := offsetVectors(t)
	answered := 0
	for _, vector := range rows {
		predicate := fmt.Sprintf(`{"left": {%q: {"offset": {"fact": "base", %q: %s}}}}`, vector.Op, vector.Direction, vector.Magnitude)
		parsed, err := parsePredicate(json.RawMessage(predicate))
		if err != nil {
			t.Errorf("%s: %v", vector.Name, err)
			continue
		}
		row := decodeRow(t, vector.Name, []byte(fmt.Sprintf(`{"left": %s, "base": %s}`, vector.Left, vector.Base)))
		if got := names[parsed.evaluate(facts(row))]; got != vector.Truth {
			t.Errorf("%s: got %s, want %s", vector.Name, got, vector.Truth)
		}
		answered++
	}
	for _, vector := range evaluate {
		parsed, err := parsePredicate(vector.Predicate)
		if err != nil {
			t.Errorf("%s: %v", vector.Name, err)
			continue
		}
		if got := names[parsed.evaluate(facts(decodeRow(t, vector.Name, vector.Row)))]; got != vector.Truth {
			t.Errorf("%s: got %s, want %s", vector.Name, got, vector.Truth)
		}
		answered++
	}
	for _, vector := range refused {
		if _, err := parsePredicate(vector.Predicate); err == nil {
			t.Errorf("%s: read, want refused", vector.Name)
		}
		answered++
	}
	if answered < 60 {
		t.Fatalf("only %d vectors were answered", answered)
	}
}

// ---- paired faults ------------------------------------------------------------------------------

type arithmetic func(base, magnitude, left int64, add bool) (int, bool)

func exactOffset(base, magnitude, left int64, add bool) (int, bool) {
	target := big.NewInt(base)
	if add {
		target.Add(target, big.NewInt(magnitude))
	} else {
		target.Sub(target, big.NewInt(magnitude))
	}
	return big.NewInt(left).Cmp(target), true
}

func order64(left, target int64) int {
	switch {
	case left < target:
		return -1
	case left > target:
		return 1
	}
	return 0
}

func wrapsOffset(base, magnitude, left int64, add bool) (int, bool) {
	if add {
		return order64(left, base+magnitude), true
	}
	return order64(left, base-magnitude), true
}

func clampsOffset(base, magnitude, left int64, add bool) (int, bool) {
	sum := new(big.Int).Add(big.NewInt(base), big.NewInt(map[bool]int64{true: magnitude, false: -magnitude}[add]))
	if sum.Cmp(big.NewInt(math.MaxInt64)) > 0 {
		return order64(left, math.MaxInt64), true
	}
	if sum.Cmp(big.NewInt(math.MinInt64)) < 0 {
		return order64(left, math.MinInt64), true
	}
	return order64(left, sum.Int64()), true
}

func roundsOffset(base, magnitude, left int64, add bool) (int, bool) {
	target := float64(base) + float64(magnitude)
	if !add {
		target = float64(base) - float64(magnitude)
	}
	switch {
	case float64(left) < target:
		return -1, true
	case float64(left) > target:
		return 1, true
	}
	return 0, true
}

func unknownOnOverflow(base, magnitude, left int64, add bool) (int, bool) {
	sum := new(big.Int).Add(big.NewInt(base), big.NewInt(map[bool]int64{true: magnitude, false: -magnitude}[add]))
	if !sum.IsInt64() {
		return 0, false
	}
	return order64(left, sum.Int64()), true
}

func ignoresSign(base, magnitude, left int64, _ bool) (int, bool) {
	return exactOffset(base, magnitude, left, true)
}

func ignoresBase(_, magnitude, left int64, _ bool) (int, bool) {
	return order64(left, magnitude), true
}

func fieldMinusField(base, magnitude, left int64, add bool) (int, bool) {
	bound := magnitude
	if !add {
		bound = -magnitude
	}
	return order64(left-base, bound), true
}

func answerOffset(compute arithmetic, vector offsetRow) string {
	number := func(raw json.RawMessage) int64 {
		var value int64
		if err := json.Unmarshal(raw, &value); err != nil {
			panic(err)
		}
		return value
	}
	order, ok := compute(number(vector.Base), number(vector.Magnitude), number(vector.Left), vector.Direction == "add")
	if !ok {
		return "unknown"
	}
	if acceptsOrder(compareSpellings[vector.Op], order) {
		return "true"
	}
	return "false"
}

func TestOffsetOperandFaults(t *testing.T) {
	raw, err := os.ReadFile("offset-operand.json")
	if err != nil {
		t.Fatalf("vectors: %v", err)
	}
	var vectors struct {
		Integer []offsetRow `json:"integer"`
	}
	if err := json.Unmarshal(raw, &vectors); err != nil {
		t.Fatalf("vectors: %v", err)
	}
	for _, vector := range vectors.Integer {
		if got := answerOffset(exactOffset, vector); got != vector.Truth {
			t.Errorf("the exact reference answers %s with %s", vector.Name, got)
		}
	}
	for _, fault := range []struct {
		name    string
		compute arithmetic
	}{
		{"wrap", wrapsOffset},
		{"clamp", clampsOffset},
		{"binary64", roundsOffset},
		{"unknown on overflow", unknownOnOverflow},
		{"ignored sign", ignoresSign},
		{"ignored base", ignoresBase},
		{"field minus field", fieldMinusField},
	} {
		killed := false
		for _, vector := range vectors.Integer {
			if answerOffset(fault.compute, vector) != vector.Truth {
				killed = true
				break
			}
		}
		if !killed {
			t.Errorf("no vector tells the %s fault apart", fault.name)
		}
	}
}
