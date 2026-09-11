package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"testing"
)

// The controlled host owns a real dispatch queue, fresh reads and command calls.
// Advancing time runs that queue; no expected trace is returned by the fixture.
type periodicHostFixture struct {
	check                 PeriodicCheck
	scope                 PeriodicScope
	now, next             uint64
	active                bool
	busy, finish, pending uint64
	logs                  []PeriodicRecord
	reads, calls, closes  int
	fault                 string
	context               map[string]Node
}

func (h *periodicHostFixture) MarkInstant(InstantMark) error { return nil }
func (h *periodicHostFixture) ObserveElapsed(ElapsedRequest) (ElapsedObservation, error) {
	return ElapsedObservation{}, ErrUnsupported
}
func (h *periodicHostFixture) OpenPeriodic(request PeriodicOpen) (PeriodicOpened, error) {
	if h.fault == "unsupported" || request.Check.Periodic.Host.Authority != "authenticated-session-status" {
		return PeriodicOpened{}, ErrUnsupported
	}
	h.check = request.Check
	h.scope = PeriodicScope{Correlation: request.Mark.Correlation, Binding: h.check.Binding, Lifetime: "session-1"}
	p, _ := periodicSeconds(h.check.Periodic.Every)
	h.next = p * 1000
	h.active = true
	h.context = map[string]Node{"agent_id": "agent-1"}
	return PeriodicOpened{Scope: h.scope, Context: h.context}, nil
}
func (h *periodicHostFixture) emit(f PeriodicFact) {
	scope := h.scope
	if h.fault == "wrong_scope" {
		scope.Lifetime = "old-session"
	}
	h.logs = append(h.logs, PeriodicRecord{Scope: scope, Fact: f})
}
func (h *periodicHostFixture) receive(n uint64) {
	p, _ := periodicSeconds(h.check.Periodic.Every)
	due := n * p * 1000
	eligible := !(h.check.Fixture == "initially_inactive" && n == 1)
	at := h.now
	if h.fault == "early" {
		at = due - 1
	}
	if h.fault == "late" {
		at = due + 1
	}
	h.emit(PeriodicFact{Kind: "received", Ordinal: n, DueMillis: due, AtMillis: at, Eligible: eligible})
	if !eligible {
		h.emit(PeriodicFact{Kind: "completed", Ordinal: n, AtMillis: h.now})
		return
	}
	h.reads++
	h.busy = n
	h.finish = h.now
	if h.check.Fixture == "slow_first_read" && n == 1 {
		h.finish = due + p*2500
	}
}
func (h *periodicHostFixture) complete() {
	n := h.busy
	if h.check.Fixture == "first_read_fails" && n == 1 && h.fault != "ignored_failure" {
		h.emit(PeriodicFact{Kind: "read_failed", Ordinal: n, AtMillis: h.now})
	} else {
		read := map[string]Node{"status": fmt.Sprintf("fresh-%d", n)}
		input := map[string]Node{"agent_id": "agent-1", "status": read["status"]}
		if h.fault == "mutate_context" {
			h.context["agent_id"] = "agent-2"
			input["agent_id"] = "agent-2"
		}
		if h.fault == "stale_input" {
			input["status"] = "stale"
		}
		h.calls++
		if h.fault == "other_cause" {
			h.emit(PeriodicFact{Kind: "independent", AtMillis: h.now, Invocation: fmt.Sprintf("push-%d", n), Command: h.check.Command})
		} else {
			h.emit(PeriodicFact{Kind: "invoked", Ordinal: n, AtMillis: h.now, Invocation: fmt.Sprintf("poll-%d", n), Command: h.check.Command, Read: read, Input: input})
		}
	}
	h.emit(PeriodicFact{Kind: "completed", Ordinal: n, AtMillis: h.now})
	h.busy = 0
	if h.pending != 0 {
		n = h.pending
		h.pending = 0
		if h.fault != "lost_pending" {
			h.receive(n)
		}
	}
}
func (h *periodicHostFixture) advance(through uint64) {
	for {
		tick, finish := ^uint64(0), ^uint64(0)
		if h.active {
			tick = h.next
		}
		if h.busy != 0 {
			finish = h.finish
		}
		next := tick
		if finish < next {
			next = finish
		}
		if next > through {
			break
		}
		h.now = next
		if finish <= tick {
			h.complete()
			continue
		}
		p, _ := periodicSeconds(h.check.Periodic.Every)
		n := tick / (p * 1000)
		h.next += p * 1000
		if h.fault == "missing" {
			continue
		}
		if h.busy != 0 {
			if h.fault == "overlap" {
				h.receive(n)
			} else if h.pending == 0 {
				h.pending = n
			} else {
				h.emit(PeriodicFact{Kind: "dropped", First: n, Last: n, AtMillis: h.now})
			}
		} else {
			h.receive(n)
		}
	}
	h.now = through
}
func (h *periodicHostFixture) ObservePeriodic(request PeriodicObserve) (PeriodicObservation, error) {
	through := uint64(request.Elapsed.Hold) * 1000
	h.advance(through)
	p, _ := periodicSeconds(h.check.Periodic.Every)
	if h.fault == "overshoot_missing" && through == 4*p*1000 {
		through += p * 1000
		h.now = through
		h.next += p * 1000
	}
	if h.fault == "late" && len(h.logs) > 0 {
		through++
	}
	return PeriodicObservation{ElapsedMillis: through, CompleteThroughMillis: through, Cursor: uint64(len(h.logs)), Records: append([]PeriodicRecord{}, h.logs[request.After:]...)}, nil
}
func (h *periodicHostFixture) ClosePeriodic(scope PeriodicScope) (PeriodicClosed, error) {
	h.closes++
	if h.fault == "closure_missing" {
		p, _ := periodicSeconds(h.check.Periodic.Every)
		h.now += 2 * p * 1000
	}
	if h.fault != "post_stop" {
		h.active = false
	}
	return PeriodicClosed{Scope: scope, AtMillis: h.now}, nil
}
func TestPeriodicControlled(t *testing.T) {
	suite, err := admitRunInput(suiteJSON)
	if err != nil {
		t.Fatal(err)
	}
	for id, scenario := range suite.Scenarios {
		t.Run(id, func(t *testing.T) {
			h := &periodicHostFixture{}
			if err := CheckPeriodic(*scenario.Steps[0].Check, "controlled", h, h); err != nil {
				t.Fatal(err)
			}
			if h.calls < 2 || h.reads < h.calls || h.closes != 1 || h.active {
				t.Fatalf("host calls=%d reads=%d closes=%d active=%v", h.calls, h.reads, h.closes, h.active)
			}
		})
	}
}
func TestPeriodicFaults(t *testing.T) {
	suite, err := admitRunInput(suiteJSON)
	if err != nil {
		t.Fatal(err)
	}
	for _, fault := range []string{"early", "late", "missing", "other_cause", "wrong_scope", "stale_input", "post_stop", "overlap", "ignored_failure", "lost_pending", "mutate_context", "unsupported", "overshoot_missing", "closure_missing"} {
		t.Run(fault, func(t *testing.T) {
			aspect := "flow"
			if fault == "overlap" || fault == "lost_pending" {
				aspect = "delivery"
			}
			if fault == "ignored_failure" {
				aspect = "on_failure"
			}
			var check *PeriodicCheck
			for _, scenario := range suite.Scenarios {
				candidate := scenario.Steps[0].Check
				if candidate.Fixture == map[string]string{"flow": "ready", "delivery": "slow_first_read", "on_failure": "first_read_fails"}[aspect] {
					check = candidate
				}
			}
			if check == nil {
				t.Fatal("fixture missing")
			}
			h := &periodicHostFixture{fault: fault}
			if err := CheckPeriodic(*check, "fault", h, h); err == nil {
				t.Fatal("fault passed")
			}
			if fault != "unsupported" && h.closes != 1 {
				t.Fatal("host not closed")
			}
		})
	}
}

// Embedding the unused event target interface does not fabricate event behavior: only the
// periodic step is admitted here and all calls below exercise the actual runner dispatch.
type periodicRunnerFixture struct {
	Target
	periodicHostFixture
}

func (h *periodicRunnerFixture) Identity() (Identity, error) {
	return Identity{Name: "controlled-periodic", Version: "test"}, nil
}
func (h *periodicRunnerFixture) BeginScenario(ScenarioContext) error { return nil }
func (h *periodicRunnerFixture) EndScenario(ScenarioContext) error   { return nil }
func TestPeriodicCountReports(t *testing.T) {
	for _, fault := range []string{"", "unsupported"} {
		t.Run("report-"+fault, func(t *testing.T) {
			path := "report-ready.json"
			if fault != "" {
				path = "report-unsupported.json"
			}
			t.Setenv("ESS_REPORT_FORMAT", "2")
			t.Setenv("ESS_REPORT_OUT", path)
			Run(t, func() Target { return &periodicRunnerFixture{periodicHostFixture: periodicHostFixture{fault: fault}} })
			data, err := os.ReadFile(path)
			if err != nil {
				t.Fatal(err)
			}
			var report map[string]any
			if err = json.Unmarshal(data, &report); err != nil {
				t.Fatal(err)
			}
			counts := report["counts"].(map[string]any)
			if report["format"] != "ess-conformance-report/2" || counts["total"] != float64(4) {
				t.Fatalf("%s", data)
			}
			if fault == "" {
				if counts["passed"] != float64(4) || report["execution_status"] != "passed" {
					t.Fatalf("%s", data)
				}
			} else {
				if counts["skipped"] != float64(4) || counts["passed"] != float64(0) || report["conformance_status"] == "passed" {
					t.Fatalf("%s", data)
				}
			}
		})
	}
}
