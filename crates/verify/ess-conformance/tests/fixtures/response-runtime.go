package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strings"
	"testing"
)

type responseFixture struct{ mode string }

func (*responseFixture) Identity() (Identity, error) {
	return Identity{Name: "response-fixture", Version: "1"}, nil
}
func (*responseFixture) BeginScenario(ScenarioContext) error { return nil }
func (*responseFixture) EndScenario(ScenarioContext) error   { return nil }
func (f *responseFixture) ExecuteCommand(CommandRequest) (CommandResult, error) {
	response := map[string]Node{"item": map[string]any{"remaining": json.Number("37"), "created": "2026-09-11T10:00:00Z", "ended": "2026-09-11T10:01:00Z", "state": "Ready", "call_type": "incoming", "features": map[string]any{"enabled": true}}}
	raw, _ := json.Marshal(response)
	decoder := json.NewDecoder(strings.NewReader(string(raw)))
	decoder.UseNumber()
	var payload map[string]Node
	if err := decoder.Decode(&payload); err != nil {
		return CommandResult{}, err
	}
	payload["receipt"] = "generated-37"
	switch f.mode {
	case "wrong-value":
		response["item"].(map[string]any)["remaining"] = json.Number("38")
	case "wrong-map":
		response["item"].(map[string]any)["features"] = map[string]any{"enabled": "true"}
		payload["item"] = response["item"]
	case "missing-field":
		delete(response, "item")
	case "missing-response":
		response = nil
	case "extra":
		response["undeclared"] = nil
	}
	return CommandResult{Outcome: "cancelled", Response: response, DirectEvents: []ObservedEvent{{Event: "demo.api.Returned", Payload: payload}}}, nil
}
func (*responseFixture) QueryView(ViewRequest) (ViewResult, error) {
	return ViewResult{}, ErrUnsupported
}
func (*responseFixture) ConfigureExternalOutcome(ExternalOutcomeControl) error { return ErrUnsupported }
func (*responseFixture) RedeliverEvent(RedeliveryRequest) error                { return ErrUnsupported }
func (*responseFixture) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}
func (*responseFixture) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, ErrUnsupported
}
func TestActualResponseRunner(t *testing.T) {
	if mode := os.Getenv("RESPONSE_FIXTURE_CHILD"); mode != "" {
		Run(t, func() Target { return &responseFixture{mode: mode} })
		return
	}
	for _, mode := range []string{"valid", "wrong-value", "wrong-map", "missing-field", "missing-response", "extra"} {
		t.Run(mode, func(t *testing.T) {
			binary, err := os.Executable()
			if err != nil {
				t.Fatal(err)
			}
			command := exec.Command(binary, "-test.run=^TestActualResponseRunner$", "-test.v")
			command.Env = append(os.Environ(), "RESPONSE_FIXTURE_CHILD="+mode, "ESS_REPORT_FORMAT=2")
			output, err := command.CombinedOutput()
			if (err == nil) != (mode == "valid") || strings.Contains(string(output), "unsupported") || strings.Contains(string(output), "SKIP") {
				t.Fatalf("%s: %v\n%s", mode, err, output)
			}
		})
	}
}
func TestResponseContractAdmission(t *testing.T) {
	if _, err := admitRunInput(suiteJSON); err != nil {
		t.Fatal(err)
	}
	for old := 1; old < 8; old++ {
		if _, err := admitRunInput(strings.ReplaceAll(suiteJSON, "ess-conformance/8", fmt.Sprintf("ess-conformance/%d", old))); err == nil {
			t.Fatal("old version admitted response", old)
		}
	}
	var doc map[string]any
	if err := json.Unmarshal([]byte(suiteJSON), &doc); err != nil {
		t.Fatal(err)
	}
	for _, scenario := range doc["scenarios"].(map[string]any) {
		for _, raw := range scenario.(map[string]any)["steps"].([]any) {
			step := raw.(map[string]any)
			if step["step"] == "expect_response_payload" {
				step["response"].(map[string]any)["trusted"] = true
			}
		}
	}
	bad, err := json.Marshal(doc)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := admitRunInput(string(bad)); err == nil {
		t.Fatal("unknown response authority admitted")
	}
}

func TestResponseSnapshotPreventsLaterMutation(t *testing.T) {
	item := map[string]any{"remaining": json.Number("37")}
	response := map[string]Node{"item": item}
	original := CommandResult{Response: response, DirectEvents: []ObservedEvent{{Event: "demo.api.Returned", Payload: map[string]Node{"item": item}}}}
	held, err := snapshotResponseResult(original)
	if err != nil {
		t.Fatal(err)
	}
	item["remaining"] = json.Number("99")
	heldItem := held.Response["item"].(map[string]any)
	if heldItem["remaining"] != json.Number("37") {
		t.Fatal("response authority aliases later callback state")
	}
	heldItem["remaining"] = json.Number("42")
	if held.DirectEvents[0].Payload["item"].(map[string]any)["remaining"] != json.Number("37") {
		t.Fatal("response and event observation share mutable authority")
	}
}
func TestResponseExactIntegerAuthority(t *testing.T) {
	for _, test := range []struct {
		raw   string
		valid bool
	}{{"9007199254740993", true}, {"-9223372036854775808", true}, {"9223372036854775807", true}, {"9223372036854775808", false}, {"9007199254740992.5", false}, {"1e0", true}} {
		if responsePrimitiveAdmits("Integer", json.Number(test.raw)) != test.valid {
			t.Fatal("integer admission", test.raw)
		}
	}
	if responseEqual(json.Number("9007199254740992"), json.Number("9007199254740993")) {
		t.Fatal("different exact integers compared equal")
	}
	if !responseEqual(json.Number("1"), json.Number("1e0")) {
		t.Fatal("equivalent integers differ")
	}
	if responsePrimitiveAdmits("Boolean", "true") || responsePrimitiveAdmits("String", true) {
		t.Fatal("wrong primitive admitted")
	}
}
func TestNullableResponseIsObservedAsAbsent(t *testing.T) {
	var document struct {
		Scenarios map[string]struct{ Steps []Step }
	}
	if err := json.Unmarshal([]byte(suiteJSON), &document); err != nil {
		t.Fatal(err)
	}
	var contract *responseObservation
	for _, scenario := range document.Scenarios {
		for _, step := range scenario.Steps {
			if step.Response != nil {
				contract = step.Response
			}
		}
	}
	if contract == nil {
		t.Fatal("response contract absent")
	}
	contract.Fields[0].Type = "Optional<demo.api.Item>"
	contract.Targets[0].Type = "Optional<demo.api.Item>"
	for _, response := range []map[string]Node{{"item": nil}, {}} {
		if err := contract.compare(response, map[string]Node{"receipt": "generated-37"}); err != nil {
			t.Fatal(err)
		}
	}
}

func TestResponseUnionClosedMembers(t *testing.T) {
	raw, err := os.ReadFile("union-contracts.json")
	if err != nil {
		t.Fatal(err)
	}
	var contracts []responseObservation
	if err = json.Unmarshal(raw, &contracts); err != nil {
		t.Fatal(err)
	}
	for _, contract := range contracts {
		declaration := contract.Declarations["example.api.Result"]
		tag := declaration.Tag
		content := "value"
		if tag == "value" {
			content = "content"
		}
		for _, inner := range []map[string]Node{{tag: "text", content: "ok"}, {tag: "text", content: nil}, {tag: "text"}} {
			// Decode each returned observation separately; equality cannot bless malformed shape.
			encoded, _ := json.Marshal(map[string]Node{"item": inner})
			var response, payload map[string]Node
			if err = json.Unmarshal(encoded, &response); err != nil {
				t.Fatal(err)
			}
			if err = json.Unmarshal(encoded, &payload); err != nil {
				t.Fatal(err)
			}
			if err = contract.compare(response, payload); err != nil {
				t.Fatal(err)
			}
			response["item"].(map[string]any)["unexpected"] = true
			payload["item"].(map[string]any)["unexpected"] = true
			if err = contract.compare(response, payload); err == nil {
				t.Fatalf("accepted unknown member under tag %s", tag)
			}
		}
		for _, inner := range []map[string]Node{{tag: "unknown", content: "ok"}, {tag: "count"}, {tag: "text", content: true}} {
			body := map[string]Node{"item": inner}
			if err = contract.compare(body, body); err == nil {
				t.Fatalf("accepted malformed union %#v", inner)
			}
		}
	}
}
