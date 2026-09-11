package essconform

import (
	"encoding/json"
	"os"
	"testing"
)

func selectionContract(t *testing.T) []*selectionObservation {
	t.Helper()
	raw, err := os.ReadFile("observation.json")
	if err != nil {
		t.Fatal(err)
	}
	var observations []any
	if err = json.Unmarshal(raw, &observations); err != nil {
		t.Fatal(err)
	}
	result := []*selectionObservation{}
	for _, raw := range observations {
		value, err := admitSelection(raw)
		if err != nil {
			t.Fatal(err)
		}
		result = append(result, value)
	}
	return result
}
func selectionLeg(id, domain, source string) Node {
	value := map[string]any{"id": id, "from": "remote"}
	if domain != "" {
		value["domain"] = domain
	}
	if source != "" {
		value["source"] = source
	}
	return value
}
func TestSelectionMixedOccurrences(t *testing.T) {
	observations := selectionContract(t)
	rows := []struct {
		items           []any
		agent, external string
		present         bool
	}{
		{[]any{}, "", "", false},
		{[]any{nil, selectionLeg("x", "", "")}, "x", "x", true},
		{[]any{selectionLeg("x", "External", ""), selectionLeg("y", "Internal", "")}, "y", "x", true},
		{[]any{selectionLeg("x", "External", "Webrtc")}, "x", "x", true},
		{[]any{selectionLeg("x", "External", "Webrtc"), selectionLeg("y", "External", "")}, "x", "y", true},
		{[]any{selectionLeg("x", "Internal", ""), selectionLeg("y", "External", "Webrtc")}, "x", "y", true},
		{[]any{selectionLeg("x", "External", "Webrtc"), selectionLeg("y", "External", "Webrtc")}, "x", "y", true},
		{[]any{selectionLeg("x", "External", ""), selectionLeg("y", "External", "")}, "x", "x", true},
		{[]any{nil, selectionLeg("", "", ""), selectionLeg("z", "", "")}, "z", "z", true},
		{[]any{selectionLeg("", "Internal", ""), selectionLeg("y", "External", "")}, "", "y", true},
		{[]any{selectionLeg("", "External", ""), selectionLeg("y", "", "")}, "y", "", true},
		{[]any{selectionLeg("x", "Internal", ""), selectionLeg("y", "Internal", ""), selectionLeg("z", "External", "")}, "x", "z", true},
		{[]any{selectionLeg("same", "External", "Webrtc"), selectionLeg("same", "External", "")}, "same", "same", true},
	}
	for index, row := range rows {
		for member, expected := range []string{row.agent, row.external} {
			value, present, err := observations[member].evaluate(map[string]Node{"data": row.items})
			if err != nil {
				t.Fatalf("row%d member%d: %v", index, member, err)
			}
			if present != row.present || (present && value != expected) {
				t.Fatalf("row%d member%d got %v/%v", index, member, value, present)
			}
		}
	}
}
func TestSelectionMalformedTail(t *testing.T) {
	observation := selectionContract(t)[0]
	for _, tail := range []any{"bad", selectionLeg("bad", "unknown", "")} {
		_, _, err := observation.evaluate(map[string]Node{"data": []any{selectionLeg("x", "Internal", ""), tail}})
		if err == nil {
			t.Fatal("malformed tail accepted")
		}
	}
}
func TestSelectionForgedReference(t *testing.T) {
	raw, err := os.ReadFile("observation.json")
	if err != nil {
		t.Fatal(err)
	}
	var observations []map[string]any
	if err = json.Unmarshal(raw, &observations); err != nil {
		t.Fatal(err)
	}
	observation := observations[0]
	selectors := observation["plan"].(map[string]any)["selectors"].([]any)
	selectors[1].(map[string]any)["operation"].(map[string]any)["excluding"] = []any{4}
	if _, err := admitSelection(observation); err == nil {
		t.Fatal("forward reference accepted")
	}
}
