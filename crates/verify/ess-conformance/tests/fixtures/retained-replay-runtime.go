package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strings"
	"testing"
)

const replayID = "00000000-0000-4000-8000-000000000037"

type retainedFixture struct {
	mode     string
	calls    int
	response map[string]Node
}

func (*retainedFixture) Identity() (Identity, error) {
	return Identity{Name: "retained-fixture", Version: "1"}, nil
}
func (f *retainedFixture) BeginScenario(ScenarioContext) error { f.calls = 0; return nil }
func (*retainedFixture) EndScenario(ScenarioContext) error     { return nil }
func retainedNativeInteger(value int64) Node                   { return value }
func retainedResponse() map[string]Node {
	return map[string]Node{"revision_id": replayID, "stamp": "2026-09-22T01:02:03Z", "number": retainedNativeInteger(9007199254740993), "values": []any{retainedNativeInteger(-9223372036854775808), retainedNativeInteger(9223372036854775807)}}
}
func (f *retainedFixture) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	f.calls++
	response := retainedResponse()
	if f.calls == 1 {
		f.response = response
		if f.mode == "mutate-input" {
			request.Input["document"] = "changed by target"
		}
		if f.mode == "mutate-nested-input" {
			request.Input["document"].(map[string]any)["value"] = "changed by target"
		}
		return CommandResult{Outcome: "seeded", Response: response, DirectEvents: []ObservedEvent{{Event: "retained.core.Seeded", Payload: map[string]Node{"record_id": replayID}}}}, nil
	}
	result := CommandResult{Outcome: "replayed", Response: response}
	switch f.mode {
	case "next-result":
		response["number"] = json.Number("9007199254740992")
	case "new-stamp":
		response["stamp"] = "2026-09-22T01:02:04Z"
	case "null-optional":
		response["optional"] = nil
	case "extra-field":
		response["extra"] = nil
	case "reordered-list":
		response["values"] = []any{json.Number("9223372036854775807"), json.Number("-9223372036854775808")}
	case "missing-response":
		result.Response = nil
	case "extra-event":
		result.DirectEvents = []ObservedEvent{{Event: "retained.core.Undeclared"}}
	case "error":
		result.Error = "retained.core.Failed"
	case "mutated-original":
		f.response["number"] = json.Number("9007199254740992")
		response["number"] = json.Number("9007199254740992")
	}
	return result, nil
}
func (f *retainedFixture) QueryView(ViewRequest) (ViewResult, error) {
	stamp := "2026-09-22T01:02:03Z"
	if f.calls > 1 && f.mode == "mutated-subject" {
		stamp = "2026-09-22T01:02:04Z"
	}
	row := map[string]Node{"record_id": replayID, "value": "document", "stamp": stamp, "state": "Committed"}
	if strings.HasPrefix(f.mode, "complete-") {
		parts := strings.SplitN(f.mode, "-", 3)
		if parts[1] == "both" || (parts[1] == "original" && f.calls == 1) || (parts[1] == "retry" && f.calls > 1) {
			switch parts[2] {
			case "identity-only":
				row = map[string]Node{"record_id": replayID}
			case "missing-value":
				delete(row, "value")
			case "missing-stamp":
				delete(row, "stamp")
			case "wrong-stamp":
				row["stamp"] = int64(42)
			case "missing-state":
				delete(row, "state")
			case "wrong-state":
				row["state"] = "NotDeclared"
			case "extra":
				row["extra"] = map[string]any{"exact": int64(9007199254740993)}
			case "null":
				row["optional"] = nil
			}
		}
	}
	return ViewResult{Rows: []map[string]Node{row}}, nil
}
func (*retainedFixture) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	panic("replay must not inject an external outcome")
}
func (*retainedFixture) RedeliverEvent(RedeliveryRequest) error { return ErrUnsupported }
func (*retainedFixture) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}
func (*retainedFixture) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, ErrUnsupported
}

func TestActualRetainedRunner(t *testing.T) {
	if mode := os.Getenv("RETAINED_FIXTURE_CHILD"); mode != "" {
		if os.Getenv("RETAINED_COVERAGE_CHILD") != "" {
			raw, err := os.ReadFile("../coverage.json")
			if err != nil {
				t.Fatal(err)
			}
			suiteJSON = string(raw)
		}
		if mode == "mutate-nested-input" {
			doc := retainedDocument(t)
			for _, raw := range retainedSteps(doc) {
				step := raw.(map[string]any)
				if step["step"] == "execute_command" {
					step["input"].(map[string]any)["document"].(map[string]any)["value"] = map[string]any{"value": "document"}
				}
			}
			raw, err := json.Marshal(doc)
			if err != nil {
				t.Fatal(err)
			}
			suiteJSON = string(raw)
		}
		Run(t, func() Target { return &retainedFixture{mode: mode} })
		return
	}
	for _, mode := range []string{"valid", "mutate-input", "mutate-nested-input", "next-result", "new-stamp", "null-optional", "extra-field", "reordered-list", "missing-response", "extra-event", "error", "mutated-subject", "mutated-original"} {
		t.Run(mode, func(t *testing.T) {
			binary, err := os.Executable()
			if err != nil {
				t.Fatal(err)
			}
			command := exec.Command(binary, "-test.run=^TestActualRetainedRunner$", "-test.v")
			command.Env = append(os.Environ(), "RETAINED_FIXTURE_CHILD="+mode, "ESS_REPORT_FORMAT=2")
			output, err := command.CombinedOutput()
			valid := mode == "valid" || mode == "mutate-input" || mode == "mutate-nested-input"
			if (err == nil) != valid || strings.Contains(string(output), "unsupported") || strings.Contains(string(output), "SKIP") {
				t.Fatalf("%s: %v\n%s", mode, err, output)
			}
		})
	}
}

func TestCompleteActualSubjectRows(t *testing.T) {
	if mode := os.Getenv("COMPLETE_ROWS_CHILD"); mode != "" {
		var target *retainedFixture
		Run(t, func() Target {
			target = &retainedFixture{mode: mode}
			return target
		})
		if target != nil {
			t.Logf("complete row calls=%d", target.calls)
		}
		return
	}
	for _, phase := range []string{"both", "original", "retry"} {
		for _, fault := range []string{"identity-only", "missing-value", "missing-stamp", "wrong-stamp", "missing-state", "wrong-state", "extra", "null"} {
			mode := "complete-" + phase + "-" + fault
			t.Run(mode, func(t *testing.T) {
				binary, err := os.Executable()
				if err != nil {
					t.Fatal(err)
				}
				command := exec.Command(binary, "-test.run=^TestCompleteActualSubjectRows$", "-test.v")
				command.Env = append(os.Environ(), "COMPLETE_ROWS_CHILD="+mode, "ESS_REPORT_FORMAT=2")
				output, err := command.CombinedOutput()
				valid := phase == "both" && (fault == "extra" || fault == "null")
				if (err == nil) != valid || strings.Contains(string(output), "SKIP") {
					t.Fatalf("complete row outcome %s: %v\n%s", mode, err, output)
				}
				if fault != "extra" && fault != "null" {
					calls := 1
					if phase == "retry" {
						calls = 2
					}
					if !strings.Contains(string(output), "complete subject row:") || !strings.Contains(string(output), fmt.Sprintf("complete row calls=%d", calls)) {
						t.Fatalf("required actual-row type check did not decide at %s: %v\n%s", phase, err, output)
					}
				}
			})
		}
	}
}

func completeShapeStep(t *testing.T, doc map[string]any) map[string]any {
	t.Helper()
	for _, raw := range retainedSteps(doc) {
		step := raw.(map[string]any)
		if step["step"] == "snapshot_complete_subject" {
			return step
		}
	}
	t.Fatal("generated replay omitted its complete subject snapshot")
	return nil
}

func TestCompleteSubjectDescriptorAdmission(t *testing.T) {
	for _, mode := range []string{"missing", "null", "unknown", "empty-fields", "duplicate-field", "missing-identity", "optional-identity", "nested-optional-identity", "unknown-type", "unrelated-type", "recursive-type", "decimal", "binary64", "invariant", "reading", "wrong-selector", "literal-selector", "unbound-selector", "legacy-snapshot", "legacy-comparison", "legacy-pair"} {
		t.Run(mode, func(t *testing.T) {
			doc := retainedDocument(t)
			step := completeShapeStep(t, doc)
			shape := step["shape"].(map[string]any)
			fields := shape["fields"].([]any)
			declarations := shape["declarations"].(map[string]any)
			var identity map[string]any
			for _, raw := range fields {
				field := raw.(map[string]any)
				if field["name"] == shape["identity_field"] {
					identity = field
				}
			}
			switch mode {
			case "missing":
				delete(step, "shape")
			case "null":
				step["shape"] = nil
			case "unknown":
				shape["expected_row"] = map[string]any{}
			case "empty-fields":
				shape["fields"] = []any{}
			case "duplicate-field":
				shape["fields"] = append(fields, fields[0])
			case "missing-identity":
				shape["identity_field"] = "missing"
			case "optional-identity":
				identity["type"] = "Optional<Uuid>"
			case "nested-optional-identity":
				identity["type"] = "retained.core.Identity"
				declarations["retained.core.Identity"] = map[string]any{"kind": "newtype", "of": "Optional<Uuid>"}
			case "unknown-type":
				identity["type"] = "retained.core.Unknown"
			case "unrelated-type":
				declarations["retained.core.Unrelated"] = map[string]any{"kind": "newtype", "of": "String"}
			case "recursive-type":
				identity["type"] = "retained.core.Recursive"
				declarations["retained.core.Recursive"] = map[string]any{"kind": "newtype", "of": "retained.core.Recursive"}
			case "decimal", "binary64", "invariant", "reading":
				identity["type"] = map[string]string{"decimal": "Decimal", "binary64": "Binary64", "invariant": "Invariant<String>", "reading": "Reading"}[mode]
			case "wrong-selector":
				step["subject"] = map[string]any{"other": map[string]any{"kind": "instance", "instance": "record"}}
			case "literal-selector":
				step["subject"] = map[string]any{shape["identity_field"].(string): map[string]any{"kind": "literal", "value": replayID}}
			case "unbound-selector":
				step["subject"] = map[string]any{shape["identity_field"].(string): map[string]any{"kind": "instance", "instance": "unbound"}}
			case "legacy-snapshot", "legacy-pair":
				step["step"] = "snapshot_subject"
				delete(step, "shape")
			}
			if mode == "legacy-comparison" || mode == "legacy-pair" {
				for _, raw := range retainedSteps(doc) {
					step := raw.(map[string]any)
					if step["step"] == "expect_complete_subject_unchanged" {
						step["step"] = "expect_subject_unchanged"
					}
				}
			}
			raw, err := json.Marshal(doc)
			if err != nil {
				t.Fatal(err)
			}
			if _, err := admitRunInput(string(raw)); err == nil {
				t.Fatal("malformed complete authority admitted", mode)
			}
		})
	}
}

func TestCompleteSubjectNestedValues(t *testing.T) {
	doc := retainedDocument(t)
	rawShape := completeShapeStep(t, doc)["shape"].(map[string]any)
	rawShape["fields"] = append(rawShape["fields"].([]any), map[string]any{"name": "nested", "type": "Map<String, List<retained.core.Nested>>"}, map[string]any{"name": "optional", "type": "Optional<Integer>"})
	rawShape["declarations"].(map[string]any)["retained.core.Nested"] = map[string]any{"kind": "struct", "fields": []any{map[string]any{"name": "integer", "type": "Integer"}, map[string]any{"name": "optional", "type": "Optional<String>"}}}
	if err := admitSubjectShape(rawShape); err != nil {
		t.Fatal(err)
	}
	raw, _ := json.Marshal(rawShape)
	var shape subjectShape
	if err := json.Unmarshal(raw, &shape); err != nil {
		t.Fatal(err)
	}
	base := map[string]Node{"record_id": replayID, "value": "document", "stamp": "2026-09-22T01:02:03Z", "state": "Committed", "nested": map[string]any{"items": []any{map[string]any{"integer": json.Number("9007199254740993")}}}, "extra": "retained"}
	if err := shape.admitRow(base); err != nil {
		t.Fatal(err)
	}
	for _, mode := range []string{"missing-nested", "missing-member", "wrong-member", "integer-overflow", "unknown-state", "optional-null", "extra-member"} {
		t.Run(mode, func(t *testing.T) {
			row, err := replaySnapshot(base)
			if err != nil {
				t.Fatal(err)
			}
			member := row["nested"].(map[string]any)["items"].([]any)[0].(map[string]any)
			switch mode {
			case "missing-nested":
				delete(row, "nested")
			case "missing-member":
				delete(member, "integer")
			case "wrong-member":
				member["integer"] = "9007199254740993"
			case "integer-overflow":
				member["integer"] = json.Number("9223372036854775808")
			case "unknown-state":
				row["state"] = "NotDeclared"
			case "optional-null":
				row["optional"] = nil
			case "extra-member":
				row["another"] = nil
			}
			valid := mode == "optional-null" || mode == "extra-member"
			if err := shape.admitRow(row); (err == nil) != valid {
				t.Fatalf("nested actual row: %v", err)
			}
			if valid && responseEqual(base, row) {
				t.Fatal("optional presence or extra key was lost")
			}
		})
	}
}

func TestCompleteAndLegacyMixedSuite(t *testing.T) {
	t.Setenv("ESS_REPORT_FORMAT", "2")
	doc := retainedDocument(t)
	for id, scenario := range doc["scenarios"].(map[string]any) {
		legacy, err := replaySnapshot(scenario)
		if err != nil {
			t.Fatal(err)
		}
		var steps []any
		for _, raw := range legacy.(map[string]any)["steps"].([]any) {
			step := raw.(map[string]any)
			switch step["step"] {
			case "capture_command_result", "expect_replay_result":
				continue
			case "snapshot_complete_subject":
				step["step"] = "snapshot_subject"
				delete(step, "shape")
			case "expect_complete_subject_unchanged":
				step["step"] = "expect_subject_unchanged"
			}
			steps = append(steps, step)
		}
		legacy.(map[string]any)["steps"] = steps
		doc["scenarios"].(map[string]any)[id+"-legacy"] = legacy
		break
	}
	raw, err := json.Marshal(doc)
	if err != nil {
		t.Fatal(err)
	}
	before := suiteJSON
	suiteJSON = string(raw)
	defer func() { suiteJSON = before }()
	Run(t, func() Target { return &retainedFixture{mode: "valid"} })
}

func TestCompleteStepsRequireNewEnvelopeWithoutReplay(t *testing.T) {
	doc := retainedDocument(t)
	for _, raw := range doc["scenarios"].(map[string]any) {
		scenario := raw.(map[string]any)
		var steps []any
		for _, raw := range scenario["steps"].([]any) {
			switch raw.(map[string]any)["step"] {
			case "capture_command_result", "expect_replay_result":
				continue
			}
			steps = append(steps, raw)
		}
		scenario["steps"] = steps
	}
	raw, _ := json.Marshal(doc)
	if _, err := admitRunInput(string(raw)); err != nil {
		t.Fatal(err)
	}
	for major := 1; major < 12; major++ {
		t.Run(fmt.Sprint(major), func(t *testing.T) {
			old := strings.ReplaceAll(string(raw), "ess-conformance/12", fmt.Sprintf("ess-conformance/%d", major))
			if _, err := admitRunInput(old); err == nil {
				t.Fatal("old envelope admitted complete steps")
			}
		})
	}
}

func TestNoSubjectAuthorityFromErrorAndNoEvents(t *testing.T) {
	doc := retainedDocument(t)
	for _, raw := range doc["scenarios"].(map[string]any) {
		raw.(map[string]any)["steps"] = []any{map[string]any{"step": "execute_command", "command": "retained.core.Seed"}, map[string]any{"step": "expect_error", "error": "retained.core.Refused"}, map[string]any{"step": "expect_no_events"}}
	}
	raw, _ := json.Marshal(doc)
	if _, err := admitRunInput(string(raw)); err != nil {
		t.Fatal("unrelated error/no-events assertions acquired subject obligation", err)
	}
}

type integerSubjectFixture struct {
	retainedFixture
	neighbor bool
}

func (f *integerSubjectFixture) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	result, err := f.retainedFixture.ExecuteCommand(request)
	if f.calls == 1 {
		result.DirectEvents[0].Payload["record_id"] = int64(9007199254740993)
	}
	return result, err
}

func (f *integerSubjectFixture) QueryView(request ViewRequest) (ViewResult, error) {
	result, err := f.retainedFixture.QueryView(request)
	identity := int64(9007199254740993)
	if f.neighbor {
		identity--
	}
	result.Rows[0]["record_id"] = identity
	return result, err
}

func TestCompleteIntegerIdentity(t *testing.T) {
	if mode := os.Getenv("COMPLETE_INTEGER_ID_CHILD"); mode != "" {
		doc := retainedDocument(t)
		shape := completeShapeStep(t, doc)["shape"].(map[string]any)
		for _, raw := range shape["fields"].([]any) {
			field := raw.(map[string]any)
			if field["name"] == shape["identity_field"] {
				field["type"] = "Integer"
			}
		}
		for _, raw := range retainedSteps(doc) {
			step := raw.(map[string]any)
			if step["step"] == "expect_event" {
				step["shape"].(map[string]any)["record_id"].(map[string]any)["kind"] = "integer"
			}
		}
		raw, _ := json.Marshal(doc)
		suiteJSON = string(raw)
		Run(t, func() Target { return &integerSubjectFixture{neighbor: mode == "neighbor"} })
		return
	}
	for _, mode := range []string{"same", "neighbor"} {
		t.Run(mode, func(t *testing.T) {
			binary, err := os.Executable()
			if err != nil {
				t.Fatal(err)
			}
			command := exec.Command(binary, "-test.run=^TestCompleteIntegerIdentity$", "-test.v")
			command.Env = append(os.Environ(), "COMPLETE_INTEGER_ID_CHILD="+mode, "ESS_REPORT_FORMAT=2")
			output, err := command.CombinedOutput()
			if (err == nil) != (mode == "same") || strings.Contains(string(output), "SKIP") {
				t.Fatalf("exact integer identity: %v\n%s", err, output)
			}
		})
	}
}

func TestRetainedCoverageEnvelope(t *testing.T) {
	raw, err := os.ReadFile("../coverage.json")
	if err != nil {
		t.Fatal(err)
	}
	suite, err := admitRunInput(string(raw))
	if err != nil {
		t.Fatal(err)
	}
	if suite.Provenance.SuiteVersion != "ess-conformance/13" || suite.coverage == nil {
		t.Fatal("missing new coverage authority")
	}
	binary, err := os.Executable()
	if err != nil {
		t.Fatal(err)
	}
	command := exec.Command(binary, "-test.run=^TestActualRetainedRunner$", "-test.v")
	command.Env = append(os.Environ(), "RETAINED_FIXTURE_CHILD=valid", "RETAINED_COVERAGE_CHILD=1", "ESS_REPORT_FORMAT=2")
	output, err := command.CombinedOutput()
	if err != nil || strings.Contains(string(output), "SKIP") {
		t.Fatalf("coverage execution: %v\n%s", err, output)
	}
}

type heldFixture struct {
	retainedFixture
	state   string
	note    Node
	reached map[string]bool
}

func (f *heldFixture) BeginScenario(ScenarioContext) error { f.state = ""; f.note = nil; return nil }
func (f *heldFixture) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	result := CommandResult{Consistency: "observed-write"}
	event := ""
	switch request.Command {
	case "retained.core.Propose":
		result.Outcome = "proposed"
		f.state = "Proposed"
		f.note = request.Input["note"]
		event = "retained.core.Proposed"
	case "retained.core.Validate":
		result.Outcome = "validated"
		f.state = "Validated"
		event = "retained.core.Changed"
	case "retained.core.Reject":
		result.Outcome = "rejected"
		f.state = "Rejected"
		event = "retained.core.Changed"
	case "retained.core.Stale":
		result.Outcome = "stale"
		f.state = "Stale"
		event = "retained.core.Changed"
	case "retained.core.Commit":
		f.reached[f.state] = true
		switch f.state {
		case "Validated":
			result.Outcome = "committed"
			f.state = "Committed"
			event = "retained.core.Changed"
		case "Committed":
			result.Outcome = "replayed"
		default:
			result.Outcome = "refused"
			if f.mode != "missing-refusal" {
				result.Error = "retained.core.TransactionStateConflict"
			}
			if f.mode == "refusal-event" {
				event = "retained.core.Undeclared"
			}
			if f.mode == "refusal-mutation" {
				f.note = "changed by refusal"
			}
		}
	default:
		return result, fmt.Errorf("unexpected held-state command %s", request.Command)
	}
	if result.Outcome == "committed" || result.Outcome == "replayed" {
		result.Response = map[string]Node{"revision": json.Number("37")}
		if result.Outcome == "replayed" && f.mode == "move-result" {
			result.Response["revision"] = json.Number("38")
		}
	}
	if event != "" {
		payload := map[string]Node{}
		if result.Outcome == "proposed" {
			payload["transaction_id"] = replayID
		}
		result.DirectEvents = []ObservedEvent{{Event: event, Payload: payload}}
	}
	return result, nil
}
func (f *heldFixture) QueryView(ViewRequest) (ViewResult, error) {
	return ViewResult{Rows: []map[string]Node{{"transaction_id": replayID, "note": f.note, "state": f.state}}}, nil
}

func TestRetainedMoveAndHeldRefusals(t *testing.T) {
	if mode := os.Getenv("HELD_FIXTURE_CHILD"); mode != "" {
		raw, err := os.ReadFile("../commit.json")
		if err != nil {
			t.Fatal(err)
		}
		suiteJSON = string(raw)
		reached := map[string]bool{}
		Run(t, func() Target { return &heldFixture{retainedFixture: retainedFixture{mode: mode}, reached: reached} })
		for _, state := range []string{"Proposed", "Validated", "Rejected", "Stale", "Committed"} {
			if !reached[state] {
				t.Error("state not exercised", state)
			}
		}
		return
	}
	for _, mode := range []string{"valid", "refusal-event", "missing-refusal", "refusal-mutation", "move-result"} {
		t.Run(mode, func(t *testing.T) {
			binary, err := os.Executable()
			if err != nil {
				t.Fatal(err)
			}
			command := exec.Command(binary, "-test.run=^TestRetainedMoveAndHeldRefusals$", "-test.v")
			command.Env = append(os.Environ(), "HELD_FIXTURE_CHILD="+mode, "ESS_REPORT_FORMAT=2")
			output, err := command.CombinedOutput()
			if (err == nil) != (mode == "valid") || strings.Contains(string(output), "unsupported") || strings.Contains(string(output), "SKIP") || strings.Contains(string(output), "state not exercised") {
				t.Fatalf("%s: %v\n%s", mode, err, output)
			}
		})
	}
}
func retainedDocument(t *testing.T) map[string]any {
	t.Helper()
	value, err := strictJSON(suiteJSON)
	if err != nil {
		t.Fatal(err)
	}
	return value.(map[string]any)
}
func retainedSteps(doc map[string]any) []any {
	for _, scenario := range doc["scenarios"].(map[string]any) {
		return scenario.(map[string]any)["steps"].([]any)
	}
	panic("missing replay scenario")
}
func TestRetainedAdmission(t *testing.T) {
	if _, err := admitRunInput(suiteJSON); err != nil {
		t.Fatal(err)
	}
	for old := 1; old < 12; old++ {
		if _, err := admitRunInput(strings.ReplaceAll(suiteJSON, "ess-conformance/12", fmt.Sprintf("ess-conformance/%d", old))); err == nil {
			t.Fatal("old envelope admitted retained steps", old)
		}
	}
}

func replayContract(t *testing.T, ty string, declarations map[string]any) replayObservation {
	t.Helper()
	value := map[string]any{"snapshot": "original", "instance": "record", "identity": map[string]any{"kind": "event", "event": "retained.core.Seeded", "field": "record_id"}, "origin": map[string]any{"command": "retained.core.Seed", "outcome": "seeded"}, "replay": map[string]any{"command": "retained.core.Seed", "outcome": "replayed"}, "fields": []any{map[string]any{"name": "item", "type": ty}}, "declarations": declarations}
	if err := admitReplay(value); err != nil {
		t.Fatal(err)
	}
	raw, _ := json.Marshal(value)
	var result replayObservation
	if err := json.Unmarshal(raw, &result); err != nil {
		t.Fatal(err)
	}
	return result
}

func TestRetainedExactRecursiveValues(t *testing.T) {
	declarations := map[string]any{"retained.core.Result": map[string]any{"kind": "struct", "fields": []any{map[string]any{"name": "count", "type": "Integer"}, map[string]any{"name": "optional", "type": "Optional<String>"}, map[string]any{"name": "values", "type": "Map<String, List<Integer>>"}}}}
	contract := replayContract(t, "retained.core.Result", declarations)
	base := map[string]Node{"item": map[string]any{"count": json.Number("9007199254740993"), "values": map[string]any{"sequence": []any{json.Number("-9223372036854775808"), json.Number("9223372036854775807")}}}}
	if err := contract.compare(base, base); err != nil {
		t.Fatal(err)
	}
	for _, mode := range []string{"adjacent", "overflow", "underflow", "fraction", "binary64", "absent-null", "extra-key", "wrong-kind", "list-order"} {
		t.Run(mode, func(t *testing.T) {
			other, err := replaySnapshot(base)
			if err != nil {
				t.Fatal(err)
			}
			item := other["item"].(map[string]any)
			switch mode {
			case "adjacent":
				item["count"] = json.Number("9007199254740992")
			case "overflow":
				item["count"] = json.Number("9223372036854775808")
			case "underflow":
				item["count"] = json.Number("-9223372036854775809")
			case "fraction":
				item["count"] = json.Number("1.5")
			case "binary64":
				item["count"] = float64(9007199254740992)
			case "absent-null":
				item["optional"] = nil
			case "extra-key":
				item["extra"] = nil
			case "wrong-kind":
				item["count"] = "9007199254740993"
			case "list-order":
				item["values"] = map[string]any{"sequence": []any{json.Number("9223372036854775807"), json.Number("-9223372036854775808")}}
			}
			if err := contract.compare(base, other); err == nil {
				t.Fatal("changed or malformed response passed")
			}
		})
	}
	integer := replayContract(t, "Integer", map[string]any{})
	for _, raw := range []string{"-9223372036854775808", "9223372036854775807", "9007199254740993"} {
		if err := integer.compare(map[string]Node{"item": json.Number(raw)}, map[string]Node{"item": json.Number(raw)}); err != nil {
			t.Fatal(raw, err)
		}
	}
	for _, ty := range []string{"String", "Timestamp", "Duration", "Uuid", "Bytes", "Boolean"} {
		contract := replayContract(t, ty, map[string]any{})
		var value any = "exact text"
		switch ty {
		case "Uuid":
			value = replayID
		case "Bytes":
			value = "AQI="
		case "Boolean":
			value = true
		}
		if err := contract.compare(map[string]Node{"item": value}, map[string]Node{"item": value}); err != nil {
			t.Fatal(ty, err)
		}
	}
}

func TestRetainedSchemaRefusesFloatingFamilies(t *testing.T) {
	for _, ty := range []string{"Decimal", "Binary64", "Optional<Decimal>", "List<Binary64>", "Map<String, Decimal>"} {
		for _, nested := range []bool{false, true} {
			t.Run(fmt.Sprintf("%s/nested=%v", ty, nested), func(t *testing.T) {
				doc := retainedDocument(t)
				for _, raw := range retainedSteps(doc) {
					step := raw.(map[string]any)
					capture, ok := step["capture"].(map[string]any)
					if !ok {
						continue
					}
					field := capture["fields"].([]any)[0].(map[string]any)
					field["type"] = ty
					if nested {
						field["type"] = "retained.core.Wrapper"
						capture["declarations"] = map[string]any{"retained.core.Wrapper": map[string]any{"kind": "union", "tag": "kind", "variants": map[string]any{"value": "retained.core.Inner"}}, "retained.core.Inner": map[string]any{"kind": "newtype", "of": ty}}
					}
				}
				raw, _ := json.Marshal(doc)
				if _, err := admitRunInput(string(raw)); err == nil {
					t.Fatal("floating response admitted")
				}
			})
		}
	}
}

func TestRetainedAdmissionRejectsSubstitutedAuthority(t *testing.T) {
	for _, mode := range []string{"unknown", "unknown-field", "missing", "overwrite", "wrong-command", "wrong-instance", "wrong-schema", "wrong-metadata", "late-snapshot", "post-retry-overwrite", "missing-snapshot", "wrong-subject", "missing-preservation", "wrong-view", "wrong-input", "wrong-actor", "aliased-capture", "unbound", "same-invocation", "pending-command", "injected-outcome", "missing-original-query", "missing-retry-query"} {
		t.Run(mode, func(t *testing.T) {
			doc := retainedDocument(t)
			steps := retainedSteps(doc)
			var origin, retry, snapshot, compare, preserve int
			for i, raw := range steps {
				step := raw.(map[string]any)
				switch step["step"] {
				case "capture_command_result":
					origin = i
				case "expect_replay_result":
					compare = i
				case "snapshot_complete_subject":
					snapshot = i
				case "expect_complete_subject_unchanged":
					preserve = i
				case "execute_command":
					if origin != 0 {
						retry = i
					}
				}
			}
			capture := steps[compare].(map[string]any)["capture"].(map[string]any)
			switch mode {
			case "unknown":
				capture["trusted"] = true
			case "unknown-field":
				capture["fields"].([]any)[0].(map[string]any)["trusted"] = true
			case "missing":
				steps = append(steps[:origin], steps[origin+1:]...)
			case "overwrite":
				steps = append(steps[:origin+1], append([]any{steps[origin]}, steps[origin+1:]...)...)
			case "wrong-command":
				capture["replay"].(map[string]any)["command"] = "retained.core.Other"
			case "wrong-instance":
				capture["instance"] = "other"
			case "wrong-schema":
				capture["fields"].([]any)[0].(map[string]any)["type"] = "String"
			case "wrong-metadata":
				capture["fields"].([]any)[0].(map[string]any)["summary"] = "substituted"
			case "late-snapshot":
				steps[snapshot], steps[retry] = steps[retry], steps[snapshot]
			case "post-retry-overwrite":
				steps = append(steps[:compare+1], append([]any{steps[snapshot]}, steps[compare+1:]...)...)
			case "missing-snapshot":
				steps = append(steps[:snapshot], steps[snapshot+1:]...)
			case "missing-original-query":
				steps = append(steps[:snapshot-1], steps[snapshot:]...)
			case "missing-retry-query":
				steps = append(steps[:preserve-1], steps[preserve:]...)
			case "wrong-subject":
				steps[snapshot].(map[string]any)["subject"] = map[string]any{"record_id": map[string]any{"kind": "literal", "value": replayID}}
			case "missing-preservation":
				steps = append(steps[:preserve], steps[preserve+1:]...)
			case "wrong-view":
				steps[preserve].(map[string]any)["view"] = "retained.core.Other"
			case "wrong-input":
				steps[retry].(map[string]any)["input"].(map[string]any)["document"].(map[string]any)["value"] = "different"
			case "wrong-actor":
				steps[retry].(map[string]any)["actor"] = "retained.core.Other"
			case "aliased-capture":
				extra, _ := replaySnapshot(steps[origin])
				extra.(map[string]any)["capture"].(map[string]any)["snapshot"] = "alias"
				steps = append(steps[:origin+1], append([]any{extra}, steps[origin+1:]...)...)
			case "unbound":
				steps = append(steps[:origin-1], steps[origin:]...)
			case "same-invocation":
				steps = append(steps[:retry], steps[retry+1:]...)
			case "pending-command":
				steps = append(steps[:compare+1], append([]any{steps[retry]}, steps[compare+1:]...)...)
			case "injected-outcome":
				steps = append(steps[:retry], append([]any{map[string]any{"step": "configure_external_outcome", "force": capture["replay"]}}, steps[retry:]...)...)
			}
			for _, scenario := range doc["scenarios"].(map[string]any) {
				scenario.(map[string]any)["steps"] = steps
			}
			raw, _ := json.Marshal(doc)
			if _, err := admitRunInput(string(raw)); err == nil {
				t.Fatal("invalid replay authority admitted", mode)
			}
		})
	}
}

type exactIntegerInputFixture struct {
	retainedFixture
	seen int
}

func (f *exactIntegerInputFixture) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	value, exact := request.Input["document"].(json.Number)
	if !exact || value.String() != "9007199254740993" {
		return CommandResult{}, fmt.Errorf("large Integer input lost before target callback: %T %v", request.Input["document"], request.Input["document"])
	}
	f.seen++
	return f.retainedFixture.ExecuteCommand(request)
}

func TestRetainedInputLiteralRemainsExact(t *testing.T) {
	doc := retainedDocument(t)
	for _, raw := range retainedSteps(doc) {
		step := raw.(map[string]any)
		if step["step"] == "execute_command" {
			step["input"].(map[string]any)["document"].(map[string]any)["value"] = json.Number("9007199254740993")
		}
	}
	raw, _ := json.Marshal(doc)
	suite, err := admitRunInput(string(raw))
	if err != nil {
		t.Fatal(err)
	}
	suite, err = executionSuite(suite)
	if err != nil {
		t.Fatal(err)
	}
	for _, scenario := range suite.Scenarios {
		for _, step := range scenario.Steps {
			if step.Step == "execute_command" && step.Input["document"].Value != json.Number("9007199254740993") {
				t.Fatal("input integer rounded", step.Input)
			}
		}
	}
	t.Setenv("ESS_REPORT_FORMAT", "2")
	before := suiteJSON
	suiteJSON = string(raw)
	defer func() { suiteJSON = before }()
	var target *exactIntegerInputFixture
	Run(t, func() Target {
		target = &exactIntegerInputFixture{}
		return target
	})
	if target == nil || target.seen != 2 {
		t.Fatal("actual original and retry callbacks did not receive the exact large Integer")
	}
}

func TestRetainedNominalSnapshotPreservesSchema(t *testing.T) {
	contract := replayContract(t, "retained.core.Result", map[string]any{"retained.core.Result": map[string]any{"kind": "union", "tag": "kind", "variants": map[string]any{"choice": "retained.core.Choice", "count": "Integer"}}, "retained.core.Choice": map[string]any{"kind": "enum", "variants": []any{"First", "Second"}}})
	copy, err := replaySnapshot(contract)
	if err != nil {
		t.Fatal(err)
	}
	left, _ := json.Marshal(contract)
	right, _ := json.Marshal(copy)
	if string(left) != string(right) {
		t.Fatal("schema snapshot changed authority")
	}
	value := map[string]Node{"item": map[string]any{"kind": "choice", "value": "First"}}
	if err = copy.compare(value, value); err != nil {
		t.Fatal(err)
	}
	value["item"].(map[string]any)["extra"] = nil
	if err = copy.compare(value, value); err == nil {
		t.Fatal("union extra member admitted")
	}
}

func TestCompleteNoEvents(t *testing.T) {
	if mode := os.Getenv("NO_EVENTS_CHILD"); mode != "" {
		r := run{t: t, lastCommand: "retained.core.Commit", status: statusPassed}
		if mode == "unknown" {
			r.last.DirectEvents = []ObservedEvent{{Event: "retained.core.Undeclared"}}
		}
		if mode == "unbound" {
			r.lastCommand = ""
		}
		r.step(0, Step{Step: "expect_no_events"})
		return
	}
	for _, mode := range []string{"empty", "unknown", "unbound"} {
		t.Run(mode, func(t *testing.T) {
			binary, err := os.Executable()
			if err != nil {
				t.Fatal(err)
			}
			command := exec.Command(binary, "-test.run=^TestCompleteNoEvents$", "-test.v")
			command.Env = append(os.Environ(), "NO_EVENTS_CHILD="+mode)
			output, err := command.CombinedOutput()
			if (err == nil) != (mode == "empty") {
				t.Fatalf("%v: %s", err, output)
			}
		})
	}
	for version := 1; version <= 13; version++ {
		err := admitStep(map[string]any{"step": "expect_no_events"}, version)
		if (err == nil) != (version >= 12) {
			t.Fatal("no-events envelope", version, err)
		}
	}
	if err := admitStep(map[string]any{"step": "expect_no_events", "event": "retained.core.Known"}, 12); err == nil {
		t.Fatal("complete event assertion admitted narrowed authority")
	}
}

func TestReplayIntegerNodeProfile(t *testing.T) {
	contract := replayContract(t, "Integer", map[string]any{})
	for _, vector := range []struct {
		raw      string
		admitted bool
	}{{"1", true}, {"1.0", true}, {"1e0", true}, {"-0", true}, {"-0.0", true}, {"9007199254740993", true}} {
		t.Run(vector.raw, func(t *testing.T) {
			value := map[string]Node{"item": json.Number(vector.raw)}
			if err := contract.compare(value, value); (err == nil) != vector.admitted {
				t.Fatalf("integer node %s: admitted=%v error=%v", vector.raw, vector.admitted, err)
			}
		})
	}
	if !responsePrimitiveAdmits("Integer", json.Number("1.0")) || !responsePrimitiveAdmits("Integer", json.Number("1e0")) {
		t.Fatal("legacy response-to-event numeric authority changed")
	}
}

func TestReplayEquivalentSmallIntegralInputSpellings(t *testing.T) {
	doc := retainedDocument(t)
	invocation := 0
	for _, raw := range retainedSteps(doc) {
		step := raw.(map[string]any)
		if step["step"] == "execute_command" {
			token := "1"
			if invocation != 0 {
				token = "1e0"
			}
			step["input"].(map[string]any)["document"].(map[string]any)["value"] = json.Number(token)
			invocation++
		}
	}
	raw, _ := json.Marshal(doc)
	if _, err := admitRunInput(string(raw)); err != nil {
		t.Fatal("equivalent admitted Integer inputs refused", err)
	}
}

func TestReplayIdentitySourceContract(t *testing.T) {
	for _, kind := range []string{"input", "event"} {
		t.Run(kind, func(t *testing.T) {
			identity := map[string]any{"kind": kind, "field": "record_id"}
			if kind == "event" {
				identity["event"] = "retained.core.Seeded"
			}
			value := map[string]any{"snapshot": "original", "instance": "record", "origin": map[string]any{"command": "retained.core.Seed", "outcome": "seeded"}, "replay": map[string]any{"command": "retained.core.Seed", "outcome": "replayed"}, "identity": identity, "fields": []any{map[string]any{"name": "revision", "type": "Integer"}}, "declarations": map[string]any{}}
			if err := admitReplay(value); err != nil {
				t.Fatal("declared original identity authority refused", err)
			}
			identity["ignored"] = true
			if err := admitReplay(value); err == nil {
				t.Fatal("unknown identity authority admitted")
			}
		})
	}
}

func TestReplayOriginalIdentityBinding(t *testing.T) {
	if mode := os.Getenv("IDENTITY_FIXTURE_CHILD"); mode != "" {
		contract := replayContract(t, "Integer", map[string]any{})
		input := strings.HasPrefix(mode, "input-")
		if input {
			contract.Identity = replayIdentity{Kind: "input", Field: "record_id"}
		}
		identity := replayID
		if strings.HasSuffix(mode, "neighbor") {
			identity = "00000000-0000-4000-8000-000000000099"
		}
		r := run{t: t, status: statusPassed, lastCommand: "retained.core.Seed", lastInput: map[string]Node{"record_id": replayID}, instances: map[string]Node{"record": identity}, last: CommandResult{Outcome: "seeded", Response: map[string]Node{"item": json.Number("37")}, DirectEvents: []ObservedEvent{{Event: "retained.core.Seeded", Payload: map[string]Node{"record_id": replayID}}}}}
		switch mode {
		case "input-missing":
			r.lastInput = map[string]Node{}
		case "event-missing-field":
			r.last.DirectEvents[0].Payload = map[string]Node{}
		case "event-duplicate":
			r.last.DirectEvents = append(r.last.DirectEvents, r.last.DirectEvents[0])
		case "event-wrong-name":
			r.last.DirectEvents[0].Event = "retained.core.Neighbor"
		}
		r.retainedResult(0, Step{Step: "capture_command_result", Capture: &contract})
		return
	}
	for _, mode := range []string{"input-valid", "input-neighbor", "input-missing", "event-valid", "event-neighbor", "event-missing-field", "event-duplicate", "event-wrong-name"} {
		t.Run(mode, func(t *testing.T) {
			binary, err := os.Executable()
			if err != nil {
				t.Fatal(err)
			}
			command := exec.Command(binary, "-test.run=^TestReplayOriginalIdentityBinding$", "-test.v")
			command.Env = append(os.Environ(), "IDENTITY_FIXTURE_CHILD="+mode)
			output, err := command.CombinedOutput()
			if (err == nil) != (mode == "input-valid" || mode == "event-valid") {
				t.Fatalf("%s: %v\n%s", mode, err, output)
			}
		})
	}
}

func TestReplayIdentityAdmissionRejectsNeighborAndStaleBinding(t *testing.T) {
	for _, mode := range []string{"event-field", "event-name", "event-stale", "input-neighbor", "input-field"} {
		t.Run(mode, func(t *testing.T) {
			doc := retainedDocument(t)
			if strings.HasPrefix(mode, "input-") {
				raw, err := os.ReadFile("../commit.json")
				if err != nil {
					t.Fatal(err)
				}
				value, err := strictJSON(string(raw))
				if err != nil {
					t.Fatal(err)
				}
				doc = value.(map[string]any)
			}
			for id, raw := range doc["scenarios"].(map[string]any) {
				if !strings.HasSuffix(id, "/outcome/replayed") {
					continue
				}
				scenario := raw.(map[string]any)
				steps := scenario["steps"].([]any)
				binding := -1
				for i, raw := range steps {
					step := raw.(map[string]any)
					if step["step"] == "capture_instance" {
						binding = i
					}
					if capture, ok := step["capture"].(map[string]any); ok {
						source := capture["identity"].(map[string]any)
						switch mode {
						case "event-field", "input-field":
							source["field"] = "neighbor_id"
						case "event-name":
							source["event"] = "retained.core.Neighbor"
						case "input-neighbor":
							capture["instance"] = "neighbor"
						}
					}
					if mode == "input-neighbor" && step["step"] == "snapshot_complete_subject" {
						for _, value := range step["subject"].(map[string]any) {
							value.(map[string]any)["instance"] = "neighbor"
						}
					}
				}
				if binding < 0 {
					t.Fatal("missing generated identity binding")
				}
				if mode == "event-stale" {
					held := steps[binding]
					steps = append(steps[:binding], steps[binding+1:]...)
					steps = append([]any{held}, steps...)
				}
				if mode == "input-neighbor" {
					neighbor, err := replaySnapshot(steps[binding])
					if err != nil {
						t.Fatal(err)
					}
					neighbor.(map[string]any)["instance"] = "neighbor"
					steps = append(steps[:binding+1], append([]any{neighbor}, steps[binding+1:]...)...)
				}
				scenario["steps"] = steps
			}
			raw, _ := json.Marshal(doc)
			if _, err := admitRunInput(string(raw)); err == nil {
				t.Fatal("substituted original identity admitted", mode)
			}
		})
	}
}

func TestReplayRequestDoesNotMutateResolvedAuthority(t *testing.T) {
	value := map[string]any{"value": "document"}
	r := run{t: t, target: &retainedFixture{mode: "mutate-nested-input"}, replayMode: true, observed: map[string][]ObservedEvent{}, status: statusPassed}
	step := Step{Command: "retained.core.Seed", Input: map[string]Value{"document": {Kind: "literal", Value: value}}}
	if !r.executeCommand(0, step) {
		t.Fatal("command failed")
	}
	if value["value"] != "document" || r.lastInput["document"].(map[string]any)["value"] != "document" {
		t.Fatal("target mutated resolved input authority")
	}
}

func TestRetainedNativeIntegerConversion(t *testing.T) {
	for _, vector := range []struct {
		value int64
		exact string
	}{{9007199254740992, "9007199254740992"}, {9007199254740993, "9007199254740993"}, {-9223372036854775808, "-9223372036854775808"}, {9223372036854775807, "9223372036854775807"}} {
		t.Run(vector.exact, func(t *testing.T) {
			result, err := snapshotResponseResult(CommandResult{Response: map[string]Node{"integer": retainedNativeInteger(vector.value)}})
			if err != nil {
				t.Fatal(err)
			}
			if result.Response["integer"] != json.Number(vector.exact) {
				t.Fatalf("native Integer adapter changed %s into %v", vector.exact, result.Response["integer"])
			}
		})
	}
}

func TestExternalStaleRefusalObligations(t *testing.T) {
	raw, err := os.ReadFile("../external-stale.json")
	if err != nil {
		t.Fatal(err)
	}
	suite, err := admitRunInput(string(raw))
	if err != nil {
		t.Fatal(err)
	}
	for _, state := range []string{"Proposed", "Rejected", "Stale"} {
		id := "retained.core.Transaction/state/" + state + "/refuses/retained.core.Commit"
		if _, exists := suite.Scenarios[id]; !exists {
			t.Error("missing Commit refusal obligation", state)
		}
	}
	if _, exists := suite.Scenarios["retained.core.Transaction/state/Stale/refuses/retained.core.Validate"]; !exists {
		t.Error("missing Validate/Stale refusal obligation")
	}
	for id, scenario := range suite.Scenarios {
		for _, step := range scenario.Steps {
			if step.Command == "retained.core.Stale" {
				t.Error("standalone Stale command substituted for actual adopter", id)
			}
			if strings.HasSuffix(id, "/outcome/replayed") && step.Step == "configure_external_outcome" {
				t.Error("retained replay was externally forced")
			}
		}
	}
}

// The adopter has no Stale command: a real external result of Commit makes the transition.
type externalStaleFixture struct {
	heldFixture
	forceStale bool
	observed   map[string]int
}

func (f *externalStaleFixture) BeginScenario(context ScenarioContext) error {
	f.forceStale = false
	return f.heldFixture.BeginScenario(context)
}
func (f *externalStaleFixture) ConfigureExternalOutcome(control ExternalOutcomeControl) error {
	if control.Command != "retained.core.Commit" || control.Outcome != "stale" {
		return fmt.Errorf("fixture only controls the actual Commit/stale external outcome")
	}
	f.forceStale = true
	return nil
}
func (f *externalStaleFixture) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	if request.Command == "retained.core.Stale" {
		return CommandResult{}, fmt.Errorf("the adopter has no standalone Stale command")
	}
	if request.Command != "retained.core.Propose" && request.Input["transaction_id"] != replayID {
		return CommandResult{}, fmt.Errorf("command does not select the established transaction")
	}
	if request.Command == "retained.core.Commit" {
		f.reached[f.state] = true
		stale := f.forceStale
		f.forceStale = false
		if f.state == "Validated" && stale {
			f.state = "Stale"
			f.observed["external-stale"]++
			return CommandResult{Outcome: "stale", Consistency: "observed-write", DirectEvents: []ObservedEvent{{Event: "retained.core.Changed", Payload: map[string]Node{}}}}, nil
		}
		if f.state != "Validated" && f.state != "Committed" {
			f.observed["commit-refused-"+f.state]++
		}
	}
	if request.Command == "retained.core.Validate" && f.state != "Proposed" {
		if f.state == "Stale" {
			f.observed["validate-refused-stale"]++
		}
		result := CommandResult{Outcome: "refused", Consistency: "observed-write"}
		if f.mode != "missing-refusal" {
			result.Error = "retained.core.TransactionStateConflict"
		}
		if f.mode == "refusal-event" {
			result.DirectEvents = []ObservedEvent{{Event: "retained.core.Undeclared"}}
		}
		if f.mode == "refusal-mutation" {
			f.note = "changed by refusal"
		}
		return result, nil
	}
	return f.heldFixture.ExecuteCommand(request)
}

func TestExternalCommitStaleRefusals(t *testing.T) {
	if mode := os.Getenv("EXTERNAL_STALE_CHILD"); mode != "" {
		raw, err := os.ReadFile("../external-stale.json")
		if err != nil {
			t.Fatal(err)
		}
		suiteJSON = string(raw)
		reached := map[string]bool{}
		observed := map[string]int{}
		Run(t, func() Target {
			return &externalStaleFixture{heldFixture: heldFixture{retainedFixture: retainedFixture{mode: mode}, reached: reached}, observed: observed}
		})
		for _, state := range []string{"Proposed", "Validated", "Rejected", "Stale", "Committed"} {
			if !reached[state] {
				t.Error("state not exercised", state)
			}
		}
		for _, obligation := range []string{"external-stale", "commit-refused-Proposed", "commit-refused-Rejected", "commit-refused-Stale", "validate-refused-stale"} {
			if observed[obligation] == 0 {
				t.Error("obligation not exercised", obligation)
			}
		}
		return
	}
	for _, mode := range []string{"valid", "refusal-event", "missing-refusal", "refusal-mutation"} {
		t.Run(mode, func(t *testing.T) {
			binary, err := os.Executable()
			if err != nil {
				t.Fatal(err)
			}
			command := exec.Command(binary, "-test.run=^TestExternalCommitStaleRefusals$", "-test.v")
			command.Env = append(os.Environ(), "EXTERNAL_STALE_CHILD="+mode, "ESS_REPORT_FORMAT=2")
			output, err := command.CombinedOutput()
			result := string(output)
			if (err == nil) != (mode == "valid") || strings.Contains(result, "unsupported") || strings.Contains(result, "SKIP") || strings.Contains(result, "panic:") || strings.Contains(result, "state not exercised") || strings.Contains(result, "obligation not exercised") {
				t.Fatalf("%s: %v\n%s", mode, err, output)
			}
			if mode != "valid" {
				for _, state := range []string{"Proposed", "Rejected", "Stale"} {
					witness := "--- FAIL: TestExternalCommitStaleRefusals/retained.core.Transaction/state/" + state + "/refuses/retained.core.Commit"
					if !strings.Contains(result, witness) {
						t.Fatalf("%s did not detect %s refusal violation\n%s", mode, state, output)
					}
				}
				if !strings.Contains(result, "--- FAIL: TestExternalCommitStaleRefusals/retained.core.Transaction/state/Stale/refuses/retained.core.Validate") {
					t.Fatalf("%s did not detect Validate/Stale refusal violation\n%s", mode, output)
				}
			}
		})
	}
}
