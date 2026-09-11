package essconform

import (
	"encoding/json"
	"os"
	"os/exec"
	"strings"
	"testing"
)

func TestAccessorSuiteAdmission(t *testing.T) {
	suite, err := admitRunInput(suiteJSON)
	if err != nil {
		t.Fatal(err)
	}
	if suite.Provenance.SuiteVersion != "ess-conformance/6" {
		t.Fatal(suite.Provenance.SuiteVersion)
	}
}

func findAccessor(t *testing.T, target string) *accessorObservation {
	t.Helper()
	suite, err := admitRunInput(suiteJSON)
	if err != nil {
		t.Fatal(err)
	}
	for _, scenario := range suite.Scenarios {
		for _, step := range scenario.Steps {
			if step.Step == "expect_invocation" {
				if v, ok := step.Input[target]; ok {
					return v.Accessor
				}
			}
		}
	}
	t.Fatal("accessor target missing", target)
	return nil
}
func TestAccessorWirePresence(t *testing.T) {
	cases := []struct {
		target  string
		payload map[string]Node
		present bool
		value   Node
	}{
		{"partial", map[string]Node{}, false, nil},
		{"deeper", map[string]Node{"data": map[string]Node{}}, true, nil},
		{"nested", map[string]Node{"data": map[string]Node{"nested": nil}}, false, nil},
		{"text", map[string]Node{"data": map[string]Node{"status": "yes"}}, true, "yes"},
		{"choice", map[string]Node{"choice": map[string]Node{"kind": "gone", "value": "gone"}}, false, nil},
		{"choice", map[string]Node{"choice": map[string]Node{"kind": "ready", "value": map[string]Node{"status": "yes"}}}, true, "yes"},
	}
	for _, c := range cases {
		value, present, err := findAccessor(t, c.target).evaluate(c.payload)
		if err != nil || present != c.present || !equal(value, c.value) {
			t.Fatalf("%s: got %v %v %v", c.target, value, present, err)
		}
	}
}
func TestAccessorMalformedInputIsNeverAbsence(t *testing.T) {
	cases := []struct {
		target  string
		payload map[string]Node
	}{
		{"choice", map[string]Node{"choice": map[string]Node{"kind": "gone", "value": float64(3)}}},
		{"choice", map[string]Node{"choice": map[string]Node{"kind": "gone"}}},
		{"choice", map[string]Node{"choice": map[string]Node{"kind": "unknown", "value": "gone"}}},
		{"partial", map[string]Node{"partial": float64(3)}},
		{"text", map[string]Node{"data": map[string]Node{"upstream_status": "yes"}}},
	}
	for _, c := range cases {
		if _, _, err := findAccessor(t, c.target).evaluate(c.payload); err == nil {
			t.Fatal("malformed input admitted", c.target, c.payload)
		}
	}
}

func accessorDocument(t *testing.T, target string) map[string]any {
	t.Helper()
	value, err := strictJSON(suiteJSON)
	if err != nil {
		t.Fatal(err)
	}
	for _, scenario := range value.(map[string]any)["scenarios"].(map[string]any) {
		for _, step := range scenario.(map[string]any)["steps"].([]any) {
			step := step.(map[string]any)
			if step["step"] == "expect_invocation" {
				if value, ok := step["input"].(map[string]any)[target]; ok {
					return value.(map[string]any)["accessor"].(map[string]any)
				}
			}
		}
	}
	t.Fatal("missing accessor", target)
	return nil
}

func TestAccessorForgedCertificateAndPlanRefusals(t *testing.T) {
	mutations := []struct {
		name   string
		mutate func(map[string]any)
	}{
		{"hidden-optional", func(a map[string]any) {
			a["types"].(map[string]any)["nodes"].(map[string]any)["projection.core.Body"].(map[string]any)["members"] = []any{"Optional<Optional<String>>"}
		}},
		{"cycle", func(a map[string]any) {
			a["types"].(map[string]any)["nodes"].(map[string]any)["projection.core.Body"].(map[string]any)["members"] = []any{"projection.core.Body"}
		}},
		{"missing-facts", func(a map[string]any) { a["types"].(map[string]any)["nodes"] = map[string]any{} }},
		{"extra-field", func(a map[string]any) { a["plan"].(map[string]any)["root"].(map[string]any)["invented"] = true }},
		{"shape-mismatch", func(a map[string]any) {
			a["plan"].(map[string]any)["nodes"].([]any)[0].(map[string]any)["shape"] = map[string]any{"kind": "primitive", "name": "unknown"}
		}},
		{"forged-metadata", func(a map[string]any) {
			a["plan"].(map[string]any)["nodes"].([]any)[0].(map[string]any)["depth"] = json.Number("127")
		}},
	}
	for _, mutation := range mutations {
		t.Run(mutation.name, func(t *testing.T) {
			a := accessorDocument(t, "whole")
			mutation.mutate(a)
			if _, err := admitAccessor(a); err == nil {
				t.Fatal("forged accessor admitted")
			}
		})
	}
}

func TestAccessorCoverageSevenLineage(t *testing.T) {
	for _, filename := range []string{"coverage-input.json", "filtered-input.json"} {
		raw, err := os.ReadFile("../" + filename)
		if err != nil {
			t.Fatal(err)
		}
		suite, err := admitRunInput(string(raw))
		if err != nil || suite.Provenance.SuiteVersion != "ess-conformance/7" {
			t.Fatalf("%s: %v %v", filename, suite.Provenance, err)
		}
		if _, err := admitRunInput(strings.ReplaceAll(string(raw), "ess-conformance/7", "ess-conformance/5")); err == nil {
			t.Fatal("downgraded lineage admitted")
		}
	}
}

func TestAccessorRawBudgetRefusesBeforeNodeAdmission(t *testing.T) {
	raw := `{"scenarios":{"x":{"steps":[{"input":{"x":{"kind":"observed_accessor","accessor":{"padding":"` + strings.Repeat("x", 2*1048576) + `"}}}}]}}}`
	if err := accessorPreflight(raw); err == nil || !strings.Contains(err.Error(), "AccessorResource") {
		t.Fatal(err)
	}
}

type accessorInvocationTarget struct {
	Target
	input map[string]Node
}

func (target accessorInvocationTarget) ObserveInvocations(request InvocationObservationRequest) ([]Invocation, error) {
	return []Invocation{{Command: request.Command, Input: target.input}}, nil
}
func TestAccessorAbsenceChild(t *testing.T) {
	mode := os.Getenv("ESS_ACCESSOR_ABSENCE_CHILD")
	if mode == "" {
		t.Skip("subprocess witness")
	}
	input := map[string]Node{}
	if mode == "null" {
		input["partial"] = nil
	}
	r := run{t: t, target: accessorInvocationTarget{input: input}, harness: NewHarness("projection"), seen: []ObservedEvent{{Event: "projection.core.Arrived", Payload: map[string]Node{}}}}
	step := Step{Step: "expect_invocation", Binding: "project", Command: "projection.core.Receive", Input: map[string]Value{"partial": {Kind: "observed_accessor", Event: "projection.core.Arrived", Accessor: findAccessor(t, "partial")}}}
	r.expectInvocation(0, step)
}
func TestAccessorInvocationRequiresActualAbsence(t *testing.T) {
	for _, mode := range []string{"absent", "null"} {
		command := exec.Command(os.Args[0], "-test.run=^TestAccessorAbsenceChild$")
		command.Env = append(os.Environ(), "ESS_ACCESSOR_ABSENCE_CHILD="+mode)
		output, err := command.CombinedOutput()
		if (err == nil) != (mode == "absent") {
			t.Fatalf("%s: %v\n%s", mode, err, output)
		}
		if mode == "null" && !strings.Contains(string(output), "did not invoke") {
			t.Fatalf("wrong refusal: %s", output)
		}
	}
}
