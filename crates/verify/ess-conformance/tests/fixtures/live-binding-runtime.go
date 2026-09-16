package essconform

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"testing"
	"time"
)

type bindingTarget struct {
	Target
	mode     string
	ended    int
	observed int
}

func (*bindingTarget) Identity() (Identity, error) {
	return Identity{Name: "fixture", Version: "1"}, nil
}
func (*bindingTarget) BeginScenario(ScenarioContext) error      { return nil }
func (target *bindingTarget) EndScenario(ScenarioContext) error { target.ended++; return nil }
func (target *bindingTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	result := CommandResult{Response: map[string]Node{"agent_id": "real-agent", "call_id": "real-call"}}
	if target.mode == "missing-response" {
		result.Response = nil
	}
	if target.mode == "malformed-response" {
		result.Response["agent_id"] = 7
	}
	if target.mode == "stale-direct-event" {
		result.DirectEvents = []ObservedEvent{bindingEvent("real-call", "real-agent")}
	}
	return result, nil
}
func bindingEvent(call, agent string) ObservedEvent {
	return ObservedEvent{Event: "fixture.routing.CallStateChanged", Payload: map[string]Node{
		"item": map[string]Node{"id": call, "agent": map[string]Node{"id": agent}, "state": "bridged"},
	}}
}
func (target *bindingTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	target.observed++
	if target.mode == "many-events" && target.observed <= 10 {
		return []ObservedEvent{bindingEvent("unrelated", "real-agent")}, nil
	}
	switch target.mode {
	case "wrong-call", "stale-direct-event":
		return []ObservedEvent{bindingEvent("unrelated", "real-agent")}, nil
	case "wrong-agent":
		return []ObservedEvent{bindingEvent("real-call", "unrelated")}, nil
	case "gap":
		return nil, fmt.Errorf("source disconnected")
	default:
		return []ObservedEvent{bindingEvent("unrelated", "real-agent"), bindingEvent("real-call", "unrelated"), bindingEvent("real-call", "real-agent")}, nil
	}
}

func TestLiveSessionUsesItsContextBudgetBeyondLegacyAttempts(t *testing.T) {
	original, err := os.ReadFile("live-suite.json")
	if err != nil { t.Fatal(err) }
	session, err := NewSessionFromInput(Identity{Name: "fixture", Version: "1"}, original)
	if err != nil { t.Fatal(err) }
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	target := &bindingTarget{mode: "many-events"}
	result, err := session.Execute(ctx, session.Scenarios()[0], func(context.Context) Target { return target })
	if err != nil || result.Status != "passed" || target.observed != 11 {
		t.Fatalf("live observation was truncated: result=%+v observations=%d err=%v", result, target.observed, err)
	}
}

func TestLiveBindingsUseActualResponsesAndNestedPayloads(t *testing.T) {
	original, err := os.ReadFile("live-suite.json")
	if err != nil {
		t.Fatal(err)
	}
	for _, mode := range []string{"pass", "wrong-call", "wrong-agent", "missing-response", "malformed-response", "stale-direct-event", "gap"} {
		t.Run(mode, func(t *testing.T) {
			session, err := NewSessionFromInput(Identity{Name: "fixture", Version: "1"}, original)
			if err != nil {
				t.Fatal(err)
			}
			target := &bindingTarget{mode: mode}
			result, err := session.Execute(context.Background(), session.Scenarios()[0], func(context.Context) Target { return target })
			if (err == nil) != (mode == "pass") {
				t.Fatalf("wrong execution error for %s: %v", mode, err)
			}
			if (result.Status == "passed") != (mode == "pass") {
				t.Fatalf("wrong status: %+v", result)
			}
			if target.ended != 1 {
				t.Fatal("scenario cleanup missing")
			}
			if strings.Contains(mode, "response") && target.observed != 0 {
				t.Fatal("observed after invalid setup response")
			}
			if _, err := session.Finish(); err != nil {
				t.Fatal(err)
			}
		})
	}
}

func TestLiveBindingsRefuseUnknownCapturesAndVocabularyBeforeFactory(t *testing.T) {
	original, err := os.ReadFile("live-suite.json")
	if err != nil {
		t.Fatal(err)
	}
	for _, changed := range []string{
		strings.ReplaceAll(string(original), "ess-conformance/10", "ess-conformance/9"),
		strings.Replace(string(original), `"instance": "call"}`, `"instance": "missing"}`, 1),
		strings.Replace(string(original), `"field": "call_id"`, `"field": "missing"`, 1),
		strings.Replace(string(original), `"instance": "call", "field"`, `"instance": "agent", "field"`, 1),
	} {
		if _, err := NewSessionFromInput(Identity{Name: "fixture", Version: "1"}, []byte(changed)); err == nil {
			t.Fatal("invalid binding admitted")
		}
	}
	var doc map[string]any
	if err := json.Unmarshal(original, &doc); err != nil {
		t.Fatal(err)
	}
	scenario := doc["scenarios"].(map[string]any)["fixture.routing/authored/real-identities"].(map[string]any)
	steps := scenario["steps"].([]any)
	steps[1].(map[string]any)["command"] = "fixture.routing.Other"
	changed, _ := json.Marshal(doc)
	if _, err := NewSessionFromInput(Identity{Name: "fixture", Version: "1"}, changed); err == nil {
		t.Fatal("capture from another invocation admitted")
	}
}
