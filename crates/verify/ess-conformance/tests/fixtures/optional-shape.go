package essconform

import (
	"encoding/json"
	"testing"
)

var suiteJSON = `{}`

// A leaf's four answers, stated as data so no case can be added without saying which way it goes.
type optionalCase struct {
	name    string
	shape   map[string]Held
	payload map[string]Node
	refused bool
}

func assertHolds(t *testing.T, cases []optionalCase) {
	t.Helper()
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			reason := holds(tc.payload, tc.shape)
			if tc.refused && reason == "" {
				t.Fatalf("admitted: %#v against %#v", tc.payload, tc.shape)
			}
			if !tc.refused && reason != "" {
				t.Fatalf("refused (%s): %#v against %#v", reason, tc.payload, tc.shape)
			}
		})
	}
}

// TestShapeLeafKeepsItsOptionalFlag is the unmarshal the defect happened at.
//
// A struct field the type does not name is dropped in silence, so the flag the synthesizer writes
// on every leaf an `Optional` was walked through never reached the decision below.
func TestShapeLeafKeepsItsOptionalFlag(t *testing.T) {
	document := `{"step":"expect_event","event":"example.domain.Recorded","shape":{
		"note":{"holds":"primitive","kind":"string","optional":true},
		"serial":{"holds":"primitive","kind":"string"}}}`
	var step Step
	if err := json.Unmarshal([]byte(document), &step); err != nil {
		t.Fatalf("a step with a shape unmarshals: %v", err)
	}
	if !step.Shape["note"].Optional {
		t.Fatal("`optional: true` did not survive unmarshal")
	}
	if step.Shape["serial"].Optional {
		t.Fatal("a leaf the document does not mark optional became one")
	}
}

// TestOptionalLeafIsSatisfiedByAbsenceAndStillCheckedWhenPresent pins all four, plus the four a
// required leaf gives, because a check that admitted everything would pass as green as one that
// checked the right thing.
//
// The fourth case is the one a careless fix breaks: treating `optional` as "ignore this path" would
// make the suite pass against an implementation publishing a boolean where the specification
// declares a string, which is worse than the absence failure it replaces — that one is visible the
// moment a conforming implementation runs, and this one is visible to nobody.
func TestOptionalLeafIsSatisfiedByAbsenceAndStillCheckedWhenPresent(t *testing.T) {
	optional := map[string]Held{"note": {Holds: "primitive", Kind: "string", Optional: true}}
	required := map[string]Held{"note": {Holds: "primitive", Kind: "string"}}
	assertHolds(t, []optionalCase{
		{"absent and optional", optional, map[string]Node{}, false},
		{"explicit null and optional", optional, map[string]Node{"note": nil}, false},
		{"right value and optional", optional, map[string]Node{"note": "text"}, false},
		{"wrong value and optional", optional, map[string]Node{"note": true}, true},

		{"absent and required", required, map[string]Node{}, true},
		{"explicit null and required", required, map[string]Node{"note": nil}, true},
		{"right value and required", required, map[string]Node{"note": "text"}, false},
		{"wrong value and required", required, map[string]Node{"note": false}, true},
	})
}

// TestOptionalLeafUnderAStructThatIsNotThere is the same four one level down, where a specification
// that wraps a struct in `Optional` puts most of them.
//
// No leaf sits at `wrapup` itself — a struct contributes its fields and no path of its own — so the
// whole value being absent has to be read off every leaf under it.
func TestOptionalLeafUnderAStructThatIsNotThere(t *testing.T) {
	shape := map[string]Held{
		"wrapup.code":  {Holds: "primitive", Kind: "string", Optional: true},
		"wrapup.taken": {Holds: "primitive", Kind: "boolean", Optional: true},
	}
	mixed := map[string]Held{
		"wrapup.code":  {Holds: "primitive", Kind: "string", Optional: true},
		"wrapup.taken": {Holds: "primitive", Kind: "boolean"},
	}
	assertHolds(t, []optionalCase{
		{"the struct is absent", shape, map[string]Node{}, false},
		{"the struct is null", shape, map[string]Node{"wrapup": nil}, false},
		{"the struct is empty", shape, map[string]Node{"wrapup": map[string]any{}}, false},
		{
			"the struct is there and its leaves hold what is declared",
			shape,
			map[string]Node{"wrapup": map[string]any{"code": "resolved", "taken": true}},
			false,
		},
		{
			"the struct is there and one leaf holds the wrong kind",
			shape,
			map[string]Node{"wrapup": map[string]any{"code": 7.0, "taken": true}},
			true,
		},
		{
			"a scalar sits where the declaration says a struct",
			shape,
			map[string]Node{"wrapup": "resolved"},
			true,
		},
		{
			"a required leaf under an absent struct is still required",
			mixed,
			map[string]Node{},
			true,
		},
		{
			"and under a null one",
			mixed,
			map[string]Node{"wrapup": nil},
			true,
		},
	})
}
