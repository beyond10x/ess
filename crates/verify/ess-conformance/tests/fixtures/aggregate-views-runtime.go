package essconform

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"testing"
)

// sessions computes the three aggregate views of aggregate-views.yaml itself. ESS_SESSIONS_MUTANT
// =filter makes every view ignore its filter, the first mutant of docs/design/aggregate-views.md.
type sessions struct {
	rows   []map[string]Node
	minted int
}

func (*sessions) Identity() (Identity, error) { return Identity{Name: "sessions", Version: "1"}, nil }
func (s *sessions) BeginScenario(ScenarioContext) error {
	s.rows = nil
	return nil
}
func (*sessions) EndScenario(ScenarioContext) error { return nil }
func (*sessions) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return fmt.Errorf("not externally forced")
}

func (s *sessions) ExecuteCommand(r CommandRequest) (CommandResult, error) {
	s.minted++
	result := CommandResult{Consistency: fmt.Sprintf("%d", s.minted)}
	switch r.Command {
	case "metrics.session.Record":
		id := fmt.Sprintf("00000000-0000-4000-8000-%012d", s.minted)
		row := map[string]Node{}
		for k, v := range r.Input {
			row[k] = v
		}
		row["session_id"] = id
		row["state"] = "Open"
		s.rows = append(s.rows, row)
		result.Outcome = "recorded"
		result.DirectEvents = []ObservedEvent{{Event: "metrics.session.Recorded", Payload: map[string]Node{"session_id": id}}}
	case "metrics.session.Complete":
		id, _ := r.Input["session_id"].(string)
		for _, row := range s.rows {
			if row["session_id"] == id && row["state"] == "Open" {
				row["state"] = "Completed"
				result.Outcome = "completed"
				result.DirectEvents = []ObservedEvent{{Event: "metrics.session.Completed", Payload: map[string]Node{"session_id": id}}}
			}
		}
	default:
		return result, fmt.Errorf("unknown command %s", r.Command)
	}
	return result, nil
}

func integer(value Node) int64 {
	number, _ := asNumber(value)
	return int64(number)
}

func mean(values []int64) Node {
	if len(values) == 0 {
		return nil
	}
	var sum int64
	for _, v := range values {
		sum += v
	}
	n := int64(len(values))
	scaled := sum * 1000000
	q, r := scaled/n, scaled%n
	if 2*r > n || (2*r == n && q%2 == 1) {
		q++
	}
	text := strings.TrimRight(strings.TrimRight(fmt.Sprintf("%d.%06d", q/1000000, q%1000000), "0"), ".")
	parsed, _ := strconv.ParseFloat(text, 64)
	return parsed
}

func (s *sessions) QueryView(r ViewRequest) (ViewResult, error) {
	admitted := func(row map[string]Node) bool {
		if os.Getenv("ESS_SESSIONS_MUTANT") == "filter" {
			return true
		}
		completed := row["state"] == "Completed"
		if r.View == "metrics.session.QueueTotals" {
			return completed && equal(row["queue_id"], r.Params["queue_id"])
		}
		return completed
	}
	keys := map[string][]string{
		"metrics.session.TalkTimeByAgent":        {"agent_id"},
		"metrics.session.SessionsByAgentChannel": {"agent_id", "channel"},
	}[r.View]
	type group struct {
		first   map[string]Node
		members []map[string]Node
	}
	groups := []*group{}
	find := func(row map[string]Node) *group {
		for _, g := range groups {
			same := true
			for _, k := range keys {
				if !equal(g.first[k], row[k]) {
					same = false
				}
			}
			if same {
				return g
			}
		}
		g := &group{first: row}
		groups = append(groups, g)
		return g
	}
	for _, row := range s.rows {
		g := find(row)
		if admitted(row) {
			g.members = append(g.members, row)
		}
	}
	if len(keys) == 0 && len(groups) == 0 {
		groups = append(groups, &group{first: map[string]Node{}})
	}
	rows := []Row{}
	for _, g := range groups {
		if len(g.members) == 0 && len(keys) > 0 {
			continue
		}
		out := Row{"sessions": float64(len(g.members))}
		for _, k := range keys {
			out[k] = g.first[k]
		}
		var talk, waits []int64
		callers := map[string]bool{}
		for _, m := range g.members {
			talk = append(talk, integer(m["talk_seconds"]))
			waits = append(waits, integer(m["wait_seconds"]))
			caller, _ := m["caller"].(string)
			callers[caller] = true
		}
		var longest Node
		for i, w := range waits {
			if i == 0 || float64(w) > longest.(float64) {
				longest = float64(w)
			}
		}
		var total int64
		for _, t := range talk {
			total += t
		}
		switch r.View {
		case "metrics.session.TalkTimeByAgent":
			out["talk_seconds"] = float64(total)
			out["longest_wait"] = longest
			out["distinct_callers"] = float64(len(callers))
			out["mean_talk"] = mean(talk)
		case "metrics.session.SessionsByAgentChannel":
			out["talk_seconds"] = float64(total)
		default:
			out["longest_wait"] = longest
		}
		rows = append(rows, out)
	}
	return ViewResult{Rows: rows}, nil
}
func (*sessions) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) { return nil, nil }
func (*sessions) RedeliverEvent(RedeliveryRequest) error                         { return ErrUnsupported }
func (*sessions) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}
func TestSessions(t *testing.T) { Run(t, func() Target { return &sessions{} }) }

// TestAggregateAdmission reads documents the Rust test wrote: an aggregate scenario under an older
// ordinary suite, and an aggregate refusal under an older coverage suite, are both refused.
func TestAggregateAdmission(t *testing.T) {
	read := func(name string) string {
		raw, err := os.ReadFile(filepath.Join(os.Getenv("ESS_AGGREGATE_DOCS"), name))
		if err != nil {
			t.Fatal(err)
		}
		return string(raw)
	}
	for name, want := range map[string]string{
		"ordinary-14.json": "aggregate views require suite/16 or /17",
		"coverage-15.json": "aggregate view refusals require suite/17",
	} {
		if _, err := admitSuiteDocument(read(name), false); err == nil || !strings.Contains(err.Error(), want) {
			t.Fatalf("%s: want %q, got %v", name, want, err)
		}
	}
	if _, err := admitSuiteDocument(read("coverage-17.json"), false); err != nil {
		t.Fatalf("coverage-17.json: %v", err)
	}
}
