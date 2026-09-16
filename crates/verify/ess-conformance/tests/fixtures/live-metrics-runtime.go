package essconform

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"testing"
)

type metricsOccurrence struct {
	Sequence uint64 `json:"sequence"`
	AtMS uint64 `json:"at_ms"`
	Event string `json:"event"`
	Payload map[string]Node `json:"payload"`
}
type metricsBatch struct {
	After uint64 `json:"after"`
	Through uint64 `json:"complete_before_ms"`
	Occurrences []metricsOccurrence `json:"occurrences"`
}
type metricsCase struct {
	Name string `json:"name"`
	Pass bool `json:"pass"`
	ErrorAt *int `json:"error_at"`
	CleanupFailure bool `json:"cleanup_failure"`
	Batches []metricsBatch `json:"batches"`
}
type metricsTarget struct {
	bindingTarget
	data metricsCase
	next int
}
func (t *metricsTarget) ObserveOccurrences(r LiveObservationRequest) (ObservationBatch,error) {
	index := t.next
	t.next++
	if t.data.ErrorAt != nil && *t.data.ErrorAt == index {
		return ObservationBatch{},context.Canceled
	}
	if index >= len(t.data.Batches) {
		return ObservationBatch{Lifetime:r.Lifetime,After:r.After,
			CompleteBeforeMS:t.data.Batches[len(t.data.Batches)-1].Through},nil
	}
	b := t.data.Batches[index]
	rows := make([]Occurrence,0,len(b.Occurrences))
	for _, row := range b.Occurrences {
		rows = append(rows,Occurrence{Sequence:row.Sequence,AtMS:row.AtMS,Event:row.Event,Payload:row.Payload})
	}
	return ObservationBatch{Lifetime:r.Lifetime,After:b.After,CompleteBeforeMS:b.Through,Occurrences:rows},nil
}
func (t *metricsTarget) EndScenario(ScenarioContext) error {
	t.ended++
	if t.data.CleanupFailure { return fmt.Errorf("cleanup failed") }
	return nil
}

func TestCapturedMetricsShareRustInputsAndCounterexamples(t *testing.T) {
	raw,err := os.ReadFile("live-metrics-suite.json")
	if err != nil { t.Fatal(err) }
	manifest,err := os.ReadFile("live-metrics-manifest.json")
	if err != nil { t.Fatal(err) }
	data,err := os.ReadFile("live-metrics-cases.json")
	if err != nil { t.Fatal(err) }
	var cases []metricsCase
	decoder := json.NewDecoder(strings.NewReader(string(data)))
	decoder.UseNumber()
	if err := decoder.Decode(&cases); err != nil { t.Fatal(err) }
	for _, c := range cases {
		t.Run(c.Name,func(t *testing.T) {
			s,err := NewSessionFromInput(Identity{Name:"fixture",Version:"1"},raw)
			if err != nil { t.Fatal(err) }
			if err := s.CheckLiveInputs(manifest); err != nil { t.Fatal(err) }
			target := &metricsTarget{data:c}
			result,err := s.Execute(context.Background(),s.Scenarios()[0],func(context.Context) Target { return target })
			if (err == nil) != c.Pass || (result.Status == "passed") != c.Pass {
				t.Fatalf("%+v %v",result,err)
			}
			if target.ended != 1 { t.Fatal("cleanup missing") }
			if _,err := s.Finish(); err != nil { t.Fatal(err) }
		})
	}
	var original map[string]json.RawMessage
	if err := json.Unmarshal(raw,&original); err != nil { t.Fatal(err) }
	var provenance map[string]json.RawMessage
	if err := json.Unmarshal(original["provenance"],&provenance); err != nil { t.Fatal(err) }
	provenance["suite_version"] = json.RawMessage(`"ess-conformance/11"`)
	original["provenance"],_ = json.Marshal(provenance)
	old,_ := json.Marshal(original)
	if _,err := NewSessionFromInput(Identity{Name:"fixture",Version:"1"},old); err == nil {
		t.Fatal("old envelope admitted new observation semantics")
	}
	provenance["suite_version"] = json.RawMessage(`"ess-conformance/12"`)
	original["provenance"],_ = json.Marshal(provenance)
	delete(original,"coverage")
	ordinary,_ := json.Marshal(original)
	if _,err := NewSessionFromInput(Identity{Name:"fixture",Version:"1"},ordinary); err != nil {
		t.Fatal(err)
	}
	s,err := NewSessionFromInput(Identity{Name:"fixture",Version:"1"},raw)
	if err != nil { t.Fatal(err) }
	selected,err := SelectInput(raw,s.Scenarios())
	if err != nil { t.Fatal(err) }
	if err := os.WriteFile("live-metrics-selected.json",selected,0600); err != nil { t.Fatal(err) }
}
