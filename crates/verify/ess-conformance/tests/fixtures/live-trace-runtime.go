package essconform

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"testing"
)

func TestLiveTraceRefusesOldVocabulary(t *testing.T) {
	original, err := os.ReadFile("live-trace-suite.json")
	if err != nil {
		t.Fatal(err)
	}
	var document map[string]any
	if err := json.Unmarshal(original, &document); err != nil {
		t.Fatal(err)
	}
	document["provenance"].(map[string]any)["suite_version"] = "ess-conformance/8"
	old, err := json.Marshal(document)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := NewSessionFromInput(Identity{Name: "fixture", Version: "1"}, old); err == nil {
		t.Fatal("old vocabulary accepted live trace")
	}
}

func (t *bindingTarget) OpenObservations([]string) (string, error) { return "subscribed", nil }
func (t *bindingTarget) ObserveOccurrences(r LiveObservationRequest) (ObservationBatch, error) {
	if t.mode == "gap" {
		return ObservationBatch{}, fmt.Errorf("disconnected")
	}
	rows := []Occurrence{
		{AtMS: 100, Event: "fixture.routing.Agent", Payload: map[string]Node{"id": "owned", "state": "wrap-up"}},
		{AtMS: 120, Event: "fixture.routing.Agent", Payload: map[string]Node{"id": "owned", "state": "available"}},
		{AtMS: 121, Event: "fixture.routing.Agent", Payload: map[string]Node{"id": "owned", "state": "selected"}},
		{AtMS: 200, Event: "fixture.routing.Metrics", Payload: map[string]Node{"account": "owned", "waiting": 3}},
		{AtMS: 205, Event: "fixture.routing.Metrics", Payload: map[string]Node{"account": "owned", "waiting": 3}},
		{AtMS: 209, Event: "fixture.routing.Metrics", Payload: map[string]Node{"account": "owned", "waiting": 3}},
	}
	if t.mode == "early-offer" {
		rows[2].AtMS = 119
		rows[1], rows[2] = rows[2], rows[1]
	}
	if t.mode == "inverted" {
		rows[1].AtMS = 99
		rows[0], rows[1] = rows[1], rows[0]
	}
	if t.mode == "transient-zero" {
		rows[4].Payload["waiting"] = 0
	}
	for i := range rows {
		rows[i].Sequence = uint64(i + 1)
	}
	if r.After != 0 {
		rows = nil
	}
	through := uint64(211)
	if t.mode == "truncated" {
		through = 209
	}
	return ObservationBatch{Lifetime: r.Lifetime, After: r.After, CompleteBeforeMS: through, Occurrences: rows}, nil
}

func TestNativeExecutionOfLiveWindows(t *testing.T) {
	raw, err := os.ReadFile("live-trace-suite.json")
	if err != nil {
		t.Fatal(err)
	}
	for _, mode := range []string{"pass", "early-offer", "inverted", "transient-zero", "truncated", "gap"} {
		t.Run(mode, func(t *testing.T) {
			s, err := NewSessionFromInput(Identity{Name: "fixture", Version: "1"}, raw)
			if err != nil {
				t.Fatal(err)
			}
			target := &bindingTarget{mode: mode}
			result, err := s.Execute(context.Background(), s.Scenarios()[0], func(context.Context) Target { return target })
			if (err == nil) != (mode == "pass") || (result.Status == "passed") != (mode == "pass") {
				t.Fatalf("%+v %v", result, err)
			}
			if target.ended != 1 {
				t.Fatal("cleanup missing")
			}
			if _, err := s.Finish(); err != nil {
				t.Fatal(err)
			}
		})
	}
}
