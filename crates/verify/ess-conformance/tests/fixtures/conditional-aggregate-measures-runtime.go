package essconform

import (
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"strconv"
	"strings"
	"testing"
)

// The Go port of the store in tests/conditional_aggregate_measures.rs: it computes every view of
// fixtures/conditional-aggregate-measures.yaml itself. ESS_CASES_FAULT names the one defect
// switched in (`none` for none).

type caMeasure struct {
	field    string
	function string // count, distinct:<f>, sum:<f>, min:<f>, max:<f>, avg:<f>
	cond     string // all, completed, escalated, urgent, not-low, rush
}

type caView struct {
	name   string
	fields []caMeasure
}

var caViews = []caView{
	{"demo.cases.Scorecard", []caMeasure{
		{"total", "count", "all"},
		{"completed", "count", "completed"},
		{"escalated_cost", "sum:cents", "escalated"},
	}},
	{"demo.cases.Escalations", []caMeasure{
		{"cases", "count", "all"},
		{"escalated", "count", "escalated"},
		{"labels", "distinct:label", "escalated"},
		{"cost", "sum:cents", "escalated"},
		{"cheapest", "min:cents", "escalated"},
		{"dearest", "max:cents", "escalated"},
		{"mean", "avg:cents", "high"},
	}},
	{"demo.cases.Urgent", []caMeasure{
		{"cases", "count", "all"},
		{"urgent", "count", "urgent"},
		{"urgent_cost", "sum:cents", "not-low"},
	}},
	{"demo.cases.Tagged", []caMeasure{
		{"cases", "count", "all"},
		{"rushed", "count", "rush"},
	}},
}

type caStore struct {
	fault  string
	rows   []map[string]Node
	minted int
}

func (*caStore) Identity() (Identity, error) {
	return Identity{Name: "conditional-measures-fixture", Version: "1"}, nil
}
func (s *caStore) BeginScenario(ScenarioContext) error {
	s.rows = nil
	return nil
}
func (*caStore) EndScenario(ScenarioContext) error { return nil }
func (*caStore) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return ErrUnsupported
}

func (s *caStore) ExecuteCommand(r CommandRequest) (CommandResult, error) {
	s.minted++
	id := fmt.Sprintf("00000000-0000-4000-8000-%012d", s.minted)
	result := CommandResult{Consistency: fmt.Sprintf("seq:%d", s.minted)}
	switch r.Command {
	case "demo.cases.Open":
		row := map[string]Node{}
		for _, field := range []string{"team", "cents", "label", "escalated", "priority", "tags"} {
			row[field] = r.Input[field]
		}
		row["case_id"] = id
		row["state"] = "Open"
		s.rows = append(s.rows, row)
		result.Outcome = "opened"
		result.DirectEvents = []ObservedEvent{{Event: "demo.cases.Opened", Payload: map[string]Node{"case_id": id}}}
	case "demo.cases.Complete":
		var found map[string]Node
		for _, row := range s.rows {
			if equal(row["case_id"], r.Input["case_id"]) {
				found = row
				break
			}
		}
		if found == nil {
			return result, nil
		}
		if found["state"] != "Open" {
			result.Outcome = "unavailable"
			return result, nil
		}
		if s.fault != "stale-state" {
			found["state"] = "Completed"
		}
		result.Outcome = "completed"
		result.DirectEvents = []ObservedEvent{{Event: "demo.cases.Completed", Payload: map[string]Node{"case_id": r.Input["case_id"]}}}
	default:
		return result, ErrUnsupported
	}
	return result, nil
}

func (s *caStore) holds(cond string, row map[string]Node) bool {
	completed := row["state"] == "Completed"
	escalated := row["escalated"] == true
	high := row["priority"] == "High"
	switch cond {
	case "all":
		return true
	case "completed":
		return completed
	case "escalated":
		return escalated
	case "urgent":
		if s.fault == "first-branch" {
			return high
		}
		return high || (escalated && completed)
	case "high":
		return high
	case "not-low":
		return row["priority"] != "Low"
	case "rush":
		tags, _ := row["tags"].([]Node)
		all, any := true, false
		for _, tag := range tags {
			rush := tag == "rush"
			all = all && rush
			any = any || rush
		}
		if s.fault == "forall-for-exists" {
			return all
		}
		return any
	}
	panic("no condition " + cond)
}

// applied is the condition this store reads for one measure, and whether it inverts it.
func (s *caStore) applied(view caView, m caMeasure) (string, bool) {
	kind, _, _ := strings.Cut(m.function, ":")
	name := map[string]string{"count": "count", "distinct": "count-distinct", "sum": "sum", "min": "min", "max": "max", "avg": "avg"}[kind]
	if view.name == "demo.cases.Escalations" && m.cond != "all" {
		if s.fault == "drop-"+name {
			return "all", false
		}
		if s.fault == "invert-"+name {
			return m.cond, true
		}
	}
	if s.fault == "swap" && view.name == "demo.cases.Scorecard" {
		switch m.field {
		case "completed":
			return "escalated", false
		case "escalated_cost":
			return "completed", false
		}
	}
	return m.cond, false
}

func caInteger(value Node) *big.Int {
	number, ok := numberValue(value)
	if !ok || !number.IsInt() {
		panic(fmt.Sprintf("not an integer: %#v", value))
	}
	return number.Num()
}

func caAggregate(function string, members []map[string]Node) Node {
	if function == "count" {
		return json.Number(strconv.Itoa(len(members)))
	}
	kind, field, _ := strings.Cut(function, ":")
	switch kind {
	case "sum":
		total := new(big.Int)
		for _, member := range members {
			total.Add(total, caInteger(member[field]))
		}
		return json.Number(total.String())
	case "avg":
		if len(members) == 0 {
			return nil
		}
		total := new(big.Int)
		for _, member := range members {
			total.Add(total, caInteger(member[field]))
		}
		n := big.NewInt(int64(len(members)))
		q, r := new(big.Int).QuoRem(new(big.Int).Mul(total, big.NewInt(1000000)), n, new(big.Int))
		twice := new(big.Int).Mul(r, big.NewInt(2))
		if twice.Cmp(n) > 0 || (twice.Cmp(n) == 0 && q.Bit(0) == 1) {
			q.Add(q, big.NewInt(1))
		}
		whole, fraction := new(big.Int).QuoRem(q, big.NewInt(1000000), new(big.Int))
		text := strings.TrimRight(strings.TrimRight(fmt.Sprintf("%s.%06d", whole.String(), fraction.Int64()), "0"), ".")
		return json.Number(text)
	case "distinct":
		var seen []Node
		for _, member := range members {
			known := false
			for _, held := range seen {
				known = known || equal(held, member[field])
			}
			if !known {
				seen = append(seen, member[field])
			}
		}
		return json.Number(strconv.Itoa(len(seen)))
	default:
		var found *big.Int
		for _, member := range members {
			value := caInteger(member[field])
			if found == nil || (kind == "max" && value.Cmp(found) > 0) || (kind == "min" && value.Cmp(found) < 0) {
				found = value
			}
		}
		if found == nil {
			return nil
		}
		return json.Number(found.String())
	}
}

func (s *caStore) QueryView(r ViewRequest) (ViewResult, error) {
	var view *caView
	for index := range caViews {
		if caViews[index].name == r.View {
			view = &caViews[index]
		}
	}
	if view == nil {
		return ViewResult{}, ErrUnsupported
	}
	type group struct {
		key     Node
		members []map[string]Node
	}
	var groups []*group
	for _, row := range s.rows {
		var found *group
		for _, g := range groups {
			if equal(g.key, row["team"]) {
				found = g
				break
			}
		}
		if found == nil {
			found = &group{key: row["team"]}
			groups = append(groups, found)
		}
		found.members = append(found.members, row)
	}
	first := ""
	for _, m := range view.fields {
		if m.cond != "all" {
			first = m.cond
			break
		}
	}
	rows := []Row{}
	for _, g := range groups {
		members := g.members
		if s.fault == "whole-view" && first != "" {
			var kept []map[string]Node
			for _, row := range members {
				if s.holds(first, row) {
					kept = append(kept, row)
				}
			}
			if len(kept) == 0 {
				continue
			}
			members = kept
		}
		out := Row{"team": g.key}
		selectedAny := false
		for _, m := range view.fields {
			cond, inverted := s.applied(*view, m)
			var selected []map[string]Node
			for _, row := range members {
				if s.holds(cond, row) != inverted {
					selected = append(selected, row)
				}
			}
			if m.cond != "all" && len(selected) > 0 {
				selectedAny = true
			}
			out[m.field] = caAggregate(m.function, selected)
		}
		if s.fault == "drops-zero-selected" && !selectedAny {
			continue
		}
		rows = append(rows, out)
	}
	return ViewResult{Rows: rows}, nil
}

func (*caStore) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) { return nil, nil }
func (*caStore) RedeliverEvent(RedeliveryRequest) error                         { return ErrUnsupported }
func (*caStore) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}

func TestCases(t *testing.T) {
	store := &caStore{fault: os.Getenv("ESS_CASES_FAULT")}
	Run(t, func() Target { return store })
}
