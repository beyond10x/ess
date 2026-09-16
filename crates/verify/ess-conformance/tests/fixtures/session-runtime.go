package essconform

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"testing"
)

func TestNativeSelectionRetainsCoverageAndParents(t *testing.T) {
	original, err := os.ReadFile("selection-parent.json")
	if err != nil {
		t.Fatal(err)
	}
	id := "example.domain/authored/second"
	child, err := SelectInput(original, []string{id})
	if err != nil {
		t.Fatal(err)
	}
	session, err := NewSessionFromInput(Identity{Name: "fixture", Version: "1"}, child)
	if err != nil {
		t.Fatal(err)
	}
	if ids := session.Scenarios(); len(ids) != 1 || ids[0] != id {
		t.Fatal(ids)
	}
	for _, ids := range [][]string{{id, id}, {"missing"}} {
		if _, err := SelectInput(original, ids); err == nil {
			t.Fatal("invalid selection admitted")
		}
	}
	if _, err := SelectInput([]byte(suiteJSON), []string{id}); err == nil {
		t.Fatal("unknown coverage narrowed")
	}
	if err := os.WriteFile("selection-child.json", child, 0600); err != nil {
		t.Fatal(err)
	}
	empty, err := SelectInput(child, nil)
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile("selection-empty.json", empty, 0600); err != nil {
		t.Fatal(err)
	}
}

var suiteJSON = `{"provenance":{"suite_version":"ess-conformance/4","system":"example","specification_version":"v1","spec_digest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","contract_digest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},"scenarios":{"example.domain/authored/live":{"purpose":"Observe the owned call","steps":[{"step":"eventually_event","event":"example.domain.Bridged","payload":{"call":"owned"},"shape":{"agent":{"holds":"primitive","kind":"string"}}}],"source":[]}}}`

type sessionTarget struct {
	Target
	mode  string
	ended int
	ctx   context.Context
}

func (*sessionTarget) Identity() (Identity, error) {
	return Identity{Name: "fixture", Version: "1"}, nil
}
func (f *sessionTarget) BeginScenario(ScenarioContext) error {
	switch f.mode {
	case "partial":
		return fmt.Errorf("partially acquired fixture")
	case "unsupported":
		return ErrUnsupported
	}
	return nil
}
func (f *sessionTarget) EndScenario(ScenarioContext) error {
	f.ended++
	if f.mode == "cleanup" {
		return fmt.Errorf("cleanup failed")
	}
	if f.mode == "panic" {
		panic("teardown interrupted")
	}
	return nil
}
func (f *sessionTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	if f.ctx.Err() != nil {
		return nil, f.ctx.Err()
	}
	payload := map[string]Node{"call": "owned", "agent": "actual-agent"}
	if f.mode == "other-call" {
		payload["call"] = "someone-else"
	}
	if f.mode == "wrong-shape" {
		payload["agent"] = true
	}
	return []ObservedEvent{{Event: "example.domain.Bridged", Payload: payload}}, nil
}

func TestSessionNativeReceipts(t *testing.T) {
	for _, mode := range []string{"pass", "other-call", "wrong-shape", "partial", "unsupported", "cleanup", "cancel"} {
		t.Run(mode, func(t *testing.T) {
			s, err := NewSession(Identity{Name: "fixture", Version: "1"})
			if err != nil {
				t.Fatal(err)
			}
			if _, err := s.Finish(); err == nil {
				t.Fatal("unexecuted suite produced report")
			}
			ctx, cancel := context.WithCancel(context.Background())
			defer cancel()
			if mode == "cancel" {
				cancel()
			}
			f := &sessionTarget{mode: mode}
			id := s.Scenarios()[0]
			result, err := s.Execute(ctx, id, func(ctx context.Context) Target { f.ctx = ctx; return f })
			if (err == nil) != (mode == "pass") {
				t.Fatalf("result=%+v err=%v", result, err)
			}
			if f.ended != 1 {
				t.Fatalf("cleanup calls=%d", f.ended)
			}
			if _, err := s.Execute(ctx, id, func(context.Context) Target { return f }); err == nil {
				t.Fatal("duplicate execution admitted")
			}
			result.Status = "passed" // Caller cannot mint or overwrite a receipt.
			raw, err := s.Finish()
			if err != nil {
				t.Fatal(err)
			}
			var report map[string]any
			if err := json.Unmarshal(raw, &report); err != nil {
				t.Fatal(err)
			}
			want := "failed"
			if mode == "pass" {
				want = "passed"
			}
			if mode == "unsupported" {
				want = "inconclusive"
			}
			if report["execution_status"] != want {
				t.Fatalf("%s", raw)
			}
			if report["conformance_status"] == "passed" {
				t.Fatal("unknown coverage qualified")
			}
			raw[0] = 'x'
			again, err := s.Finish()
			if err != nil || again[0] != '{' {
				t.Fatal("report aliases caller storage")
			}
		})
	}
}

func TestSessionPanicCannotProduceReceipt(t *testing.T) {
	s, err := NewSession(Identity{Name: "fixture", Version: "1"})
	if err != nil {
		t.Fatal(err)
	}
	f := &sessionTarget{mode: "panic", ctx: context.Background()}
	func() {
		defer func() {
			if recover() == nil {
				t.Error("target panic swallowed")
			}
		}()
		_, _ = s.Execute(f.ctx, s.Scenarios()[0], func(context.Context) Target { return f })
	}()
	if _, err := s.Finish(); err == nil {
		t.Fatal("interrupted callback manufactured receipt")
	}
}

func TestSessionRejectsInvalidInventory(t *testing.T) {
	before := suiteJSON
	defer func() { suiteJSON = before }()
	suiteJSON = "{}"
	if _, err := NewSession(Identity{Name: "fixture", Version: "1"}); err == nil {
		t.Fatal("invalid suite admitted")
	}
}

func TestSessionRetainsOriginalInputBytes(t *testing.T) {
	original := []byte(" \n" + suiteJSON + "\n")
	s, err := NewSessionFromInput(Identity{Name: "fixture", Version: "1"}, original)
	if err != nil {
		t.Fatal(err)
	}
	original[0] = 'x'
	if s.OriginalInput()[0] != ' ' {
		t.Fatal("input aliases host memory")
	}
	copy := s.OriginalInput()
	copy[0] = 'x'
	if s.OriginalInput()[0] != ' ' {
		t.Fatal("input aliases returned evidence")
	}
}
