package essconform

import (
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"strconv"
	"strings"
	"testing"
	"time"
)

// The Go port of the store in tests/aggregate_group_selection.rs: it computes every view of the
// group-selection fixtures itself. ESS_GROUPS_MODEL names the fixture and ESS_GROUPS_FAULT the one
// defect switched in (`none` for none).

type gsView struct {
	name     string
	groupBy  []string
	params   []string
	openOnly bool
	// Each field: its name, and `count`, `sum:<f>`, `max:<f>` or `min:<f>`.
	fields [][2]string
}

var gsModels = map[string][]gsView{
	"work": {
		{"demo.work.ByTeam", []string{"team"}, []string{"team"}, false, [][2]string{{"count", "count"}, {"total", "sum:cents"}, {"top", "max:cents"}}},
		{"demo.work.OpenByTeam", []string{"team"}, []string{"team"}, true, [][2]string{{"count", "count"}, {"total", "sum:cents"}}},
		{"demo.work.ByTeamState", []string{"team", "state"}, []string{"team"}, false, [][2]string{{"count", "count"}}},
		{"demo.work.ByTeamChannel", []string{"team", "channel"}, []string{"team"}, false, [][2]string{{"count", "count"}, {"total", "sum:cents"}}},
		{"demo.work.ByUrgency", []string{"urgent"}, []string{"urgent"}, false, [][2]string{{"count", "count"}, {"total", "sum:cents"}}},
		{"demo.work.ByBucket", []string{"bucket"}, []string{"bucket"}, false, [][2]string{{"count", "count"}}},
		{"demo.work.ByState", []string{"state"}, nil, false, [][2]string{{"count", "count"}, {"total", "sum:cents"}, {"low", "min:cents"}}},
		{"demo.work.TeamsInLane", []string{"team"}, []string{"lane", "team"}, false, [][2]string{{"count", "count"}}},
		{"demo.work.ByTeamUrgency", []string{"team", "urgent"}, []string{"team", "urgent"}, false, [][2]string{{"count", "count"}}},
		{"demo.work.ByKind", []string{"kind"}, nil, false, [][2]string{{"count", "count"}, {"mean", "avg:cents"}, {"lanes", "distinct:lane"}}},
		{"demo.work.ByUrgent", []string{"urgent"}, nil, false, [][2]string{{"count", "count"}}},
		{"demo.work.TopOpen", nil, nil, true, [][2]string{{"top", "max:cents"}, {"low", "min:cents"}}},
	},
	"depots": {
		{"demo.work.ByTeam", []string{"team"}, []string{"team"}, false, [][2]string{{"count", "count"}, {"total", "sum:cents"}}},
		{"demo.work.ItemsByState", []string{"state"}, nil, false, [][2]string{{"count", "count"}}},
	},
	"direct": {
		{"demo.work.ByTeam", []string{"team"}, []string{"team"}, false, [][2]string{{"count", "count"}}},
	},
	"copied": {
		{"demo.work.ByTeam", []string{"team"}, []string{"team"}, false, [][2]string{{"count", "count"}, {"total", "sum:cents"}}},
	},
	"states": {
		{"demo.work.ByState", []string{"state"}, nil, false, [][2]string{{"count", "count"}}},
	},
	"ledger": {
		{"demo.ledger.ByAccount", []string{"account_id"}, []string{"account_id"}, false, [][2]string{{"count", "count"}, {"total", "sum:cents"}}},
	},
	"goals": {
		{"demo.goals.EvidenceEvaluations", []string{"objective_id", "goal"}, []string{"objective_id"}, false, [][2]string{{"count", "count"}}},
		{"demo.goals.EvaluationsByGoal", []string{"goal"}, nil, false, [][2]string{{"count", "count"}, {"total", "sum:cents"}}},
	},
	"lists": {
		{"demo.calls.InQueues", []string{"queue_id"}, nil, false, [][2]string{{"calls", "count"}, {"talk_ms", "sum:duration_ms"}}},
		{"demo.calls.InQueuesOrAll", []string{"queue_id"}, nil, false, [][2]string{{"calls", "count"}}},
	},
	"window": {
		{"demo.window.CallsInRange", nil, nil, false, [][2]string{{"calls", "count"}, {"talk_ms", "sum:duration_ms"}}},
		{"demo.window.QueueCallsInRange", []string{"queue_id"}, nil, false, [][2]string{{"calls", "count"}}},
	},
}

// gsRanges names the field each window view holds to `[param.from, param.to)`.
var gsRanges = map[string]string{
	"demo.window.CallsInRange":      "started_at",
	"demo.window.QueueCallsInRange": "started_at",
}

// gsList is `exists: {in: param.<param>, as: q, that: <field> == q}`; with emptyAll, beside
// `param.<param>.count == 0`.
type gsList struct {
	param, field string
	emptyAll     bool
}

var gsLists = map[string]gsList{
	"demo.calls.InQueues":      {"queues", "queue_id", false},
	"demo.calls.InQueuesOrAll": {"queues", "queue_id", true},
}

type gsStore struct {
	model  string
	fault  string
	rows   []map[string]Node
	depots [][2]Node
	minted int
}

func (*gsStore) Identity() (Identity, error) {
	return Identity{Name: "group-selection-fixture", Version: "1"}, nil
}

func (s *gsStore) BeginScenario(ScenarioContext) error {
	if s.fault != "retains-rows" {
		s.rows = nil
		s.depots = nil
	}
	return nil
}
func (*gsStore) EndScenario(ScenarioContext) error { return nil }
func (*gsStore) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return ErrUnsupported
}

func (s *gsStore) kept(value Node) Node {
	if s.fault != "lossy-numbers" {
		return value
	}
	if number, ok := asNumber(value); ok {
		return number
	}
	return value
}

// family is the set of commands the model declares.
func (s *gsStore) family() string {
	switch s.model {
	case "depots", "copied":
		return "depots"
	case "ledger", "goals", "lists", "window":
		return s.model
	}
	return "work"
}

// related is the value copied from the related row named — or, under a copying fault, from the
// first or last one created.
func (s *gsStore) related(named Node) (Node, bool) {
	switch {
	case len(s.depots) == 0:
		return nil, false
	case s.fault == "copies-first-related":
		return s.depots[0][1], true
	case s.fault == "copies-last-related":
		return s.depots[len(s.depots)-1][1], true
	}
	for _, held := range s.depots {
		if equal(held[0], named) {
			return held[1], true
		}
	}
	return nil, false
}

func (s *gsStore) ExecuteCommand(r CommandRequest) (CommandResult, error) {
	s.minted++
	id := fmt.Sprintf("00000000-0000-4000-8000-%012d", s.minted)
	result := CommandResult{Consistency: fmt.Sprintf("seq:%d", s.minted)}
	input := func(field string) Node {
		value, ok := r.Input[field]
		if !ok {
			return nil
		}
		return s.kept(value)
	}
	opened := func(event, field string) {
		result.Outcome = "opened"
		result.DirectEvents = []ObservedEvent{{Event: event, Payload: map[string]Node{field: id}}}
	}
	switch s.family() + " " + r.Command {
	case "work demo.work.Open":
		row := map[string]Node{}
		for _, field := range []string{"team", "lane", "cents", "urgent", "channel", "kind"} {
			row[field] = input(field)
		}
		row["id"] = id
		row["bucket"] = s.kept(json.Number("9007199254740993"))
		row["state"] = "Open"
		s.rows = append(s.rows, row)
		opened("demo.work.Opened", "id")
	case "work demo.work.Finish":
		var found map[string]Node
		for _, row := range s.rows {
			if equal(row["id"], r.Input["id"]) {
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
			found["state"] = "Done"
		}
		result.Outcome = "finished"
		result.DirectEvents = []ObservedEvent{{Event: "demo.work.Finished", Payload: map[string]Node{"id": r.Input["id"]}}}
	case "depots demo.work.OpenDepot":
		s.depots = append(s.depots, [2]Node{id, input("team")})
		opened("demo.work.DepotOpened", "depot_id")
	case "depots demo.work.OpenItem":
		team, ok := s.related(r.Input["depot_id"])
		if !ok {
			return result, nil
		}
		s.rows = append(s.rows, map[string]Node{
			"item_id": id, "depot_id": r.Input["depot_id"], "team": team,
			"cents": input("cents"), "state": "Open",
		})
		opened("demo.work.ItemOpened", "item_id")
	case "ledger demo.ledger.OpenAccount":
		s.depots = append(s.depots, [2]Node{id, input("holder")})
		opened("demo.ledger.AccountOpened", "account_id")
	case "ledger demo.ledger.PostEntry":
		known := false
		for _, held := range s.depots {
			known = known || equal(held[0], r.Input["account_id"])
		}
		if !known {
			return result, nil
		}
		s.rows = append(s.rows, map[string]Node{
			"entry_id": id, "account_id": r.Input["account_id"], "cents": input("cents"), "state": "Posted",
		})
		result.Outcome = "posted"
		result.DirectEvents = []ObservedEvent{{Event: "demo.ledger.EntryPosted", Payload: map[string]Node{"entry_id": id}}}
	case "goals demo.goals.OpenObjective":
		s.depots = append(s.depots, [2]Node{id, input("goal")})
		opened("demo.goals.ObjectiveOpened", "objective_id")
	case "goals demo.goals.IntegrateCandidate":
		// A missing objective is the `no-objective` refusal, which no aggregate scenario sends.
		goal, ok := s.related(r.Input["objective_id"])
		if !ok {
			return result, nil
		}
		s.rows = append(s.rows, map[string]Node{
			"evaluation_id": id, "objective_id": r.Input["objective_id"], "goal": goal,
			"cents": input("cents"), "state": "Recorded",
		})
		result.Outcome = "integrated"
		result.DirectEvents = []ObservedEvent{{Event: "demo.goals.Evaluated", Payload: map[string]Node{"evaluation_id": id}}}
	case "window demo.window.RecordCall":
		row := map[string]Node{"call_id": id, "state": "Recorded"}
		for _, field := range []string{"queue_id", "started_at", "duration_ms"} {
			row[field] = input(field)
		}
		s.rows = append(s.rows, row)
		result.Outcome = "recorded"
		result.DirectEvents = []ObservedEvent{{Event: "demo.window.CallRecorded", Payload: map[string]Node{"call_id": id}}}
	case "lists demo.calls.RecordCall":
		row := map[string]Node{"call_id": id, "state": "Recorded"}
		for _, field := range []string{"queue_id", "abandoned", "duration_ms"} {
			row[field] = input(field)
		}
		s.rows = append(s.rows, row)
		result.Outcome = "recorded"
		result.DirectEvents = []ObservedEvent{{Event: "demo.calls.CallRecorded", Payload: map[string]Node{"call_id": id}}}
	default:
		return result, ErrUnsupported
	}
	return result, nil
}

func (s *gsStore) selected(held, param, first Node, hasFirst bool) bool {
	switch s.fault {
	case "ignores-param":
		return true
	case "selects-first-group":
		return hasFirst && equal(first, held)
	case "prefix-param":
		h, hok := held.(string)
		p, pok := param.(string)
		if hok && pok {
			return strings.HasPrefix(h, p)
		}
		return equal(held, param)
	}
	return held != nil && equal(held, s.kept(param))
}

func (s *gsStore) admits(view gsView, row map[string]Node, params map[string]Node) bool {
	for _, param := range view.params {
		grouped := false
		for _, key := range view.groupBy {
			if key == param {
				grouped = true
			}
		}
		if grouped {
			var first Node
			hasFirst := len(s.rows) > 0
			if hasFirst {
				first = s.rows[0][param]
			}
			if !s.selected(row[param], params[param], first, hasFirst) {
				return false
			}
		} else if !equal(row[param], s.kept(params[param])) {
			return false
		}
	}
	if view.openOnly && row["state"] != "Open" {
		return false
	}
	if list, ok := gsLists[view.name]; ok && s.model == "lists" {
		return s.listed(list, row, params)
	}
	if field, ok := gsRanges[view.name]; ok && s.model == "window" {
		return s.within(field, row, params)
	}
	return true
}

// within is whether the row's instant is in `[param.from, param.to)`.
func (s *gsStore) within(field string, row map[string]Node, params map[string]Node) bool {
	held, _ := row[field].(string)
	from, _ := params["from"].(string)
	to, _ := params["to"].(string)
	if s.fault == "compares-text" {
		return from <= held && held < to
	}
	at := func(text string) time.Time {
		parsed, err := time.Parse(time.RFC3339, text)
		if err != nil {
			panic(fmt.Sprintf("not an instant: %q", text))
		}
		return parsed
	}
	h, f, t := at(held), at(from), at(to)
	lower := s.fault == "ignores-bound" || !h.Before(f)
	upper := h.Before(t) || (s.fault == "inclusive-to" && h.Equal(t))
	return lower && upper
}

// listed is whether the row's field is one the list parameter names.
func (s *gsStore) listed(list gsList, row map[string]Node, params map[string]Node) bool {
	sent, ok := params[list.param].([]any)
	if !ok {
		return false
	}
	switch {
	case s.fault == "ignores-list":
		return true
	case len(sent) == 0:
		return list.emptyAll && s.fault != "empty-matches-nothing"
	case s.fault == "first-element-only":
		return equal(row[list.field], sent[0])
	}
	for _, value := range sent {
		if equal(row[list.field], value) {
			return true
		}
	}
	return false
}

func gsInteger(value Node) *big.Rat {
	number, ok := numberValue(value)
	if !ok {
		panic(fmt.Sprintf("not a number: %#v", value))
	}
	return number
}

func gsNumber(value *big.Rat) Node { return json.Number(value.RatString()) }

func (s *gsStore) aggregate(function string, all, members []map[string]Node) Node {
	if function == "count" {
		counted := len(members)
		if s.fault == "counts-all-rows" {
			counted = len(all)
		}
		if s.fault == "wrong-count" {
			counted++
		}
		return json.Number(strconv.Itoa(counted))
	}
	kind, field, _ := strings.Cut(function, ":")
	switch kind {
	case "sum":
		total := new(big.Rat)
		for _, member := range members {
			total.Add(total, gsInteger(member[field]))
		}
		if s.fault == "wrong-sum" {
			total.Add(total, big.NewRat(1, 1))
		}
		return gsNumber(total)
	case "avg":
		if len(members) == 0 {
			return nil
		}
		total := new(big.Int)
		for _, member := range members {
			total.Add(total, gsInteger(member[field]).Num())
		}
		n := big.NewInt(int64(len(members)))
		q, r := new(big.Int).QuoRem(new(big.Int).Mul(total, big.NewInt(1000000)), n, new(big.Int))
		twice := new(big.Int).Mul(r, big.NewInt(2))
		if s.fault != "truncates-avg" && (twice.Cmp(n) > 0 || (twice.Cmp(n) == 0 && q.Bit(0) == 1)) {
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
		counted := len(seen)
		if s.fault == "distinct-counts-rows" {
			counted = len(members)
		}
		return json.Number(strconv.Itoa(counted))
	default:
		max := (kind == "max") != (s.fault == "wrong-extreme")
		var found *big.Rat
		for _, member := range members {
			value := gsInteger(member[field])
			if found == nil || (max && value.Cmp(found) > 0) || (!max && value.Cmp(found) < 0) {
				found = value
			}
		}
		if found == nil {
			return nil
		}
		return gsNumber(found)
	}
}

func (s *gsStore) QueryView(r ViewRequest) (ViewResult, error) {
	var view *gsView
	for index := range gsModels[s.model] {
		if gsModels[s.model][index].name == r.View {
			view = &gsModels[s.model][index]
		}
	}
	if view == nil {
		return ViewResult{}, ErrUnsupported
	}
	keys := view.groupBy
	if s.fault == "merges-groups" {
		if len(keys) > 1 {
			keys = keys[:1]
		} else {
			keys = nil
		}
	}
	type group struct {
		key          []Node
		all, members []map[string]Node
	}
	var groups []*group
	for _, row := range s.rows {
		key := make([]Node, len(keys))
		absent := false
		for index, name := range keys {
			key[index] = row[name]
			absent = absent || row[name] == nil
		}
		if s.fault == "drops-absent-group" && absent {
			continue
		}
		var found *group
		for _, g := range groups {
			same := true
			for index := range key {
				if !equal(g.key[index], key[index]) {
					same = false
				}
			}
			if same {
				found = g
				break
			}
		}
		if found == nil {
			found = &group{key: key}
			groups = append(groups, found)
		}
		found.all = append(found.all, row)
		if s.admits(*view, row, r.Params) {
			found.members = append(found.members, row)
		}
	}
	rows := []Row{}
	for _, g := range groups {
		if len(g.members) == 0 {
			continue
		}
		out := Row{}
		for _, key := range view.groupBy {
			out[key] = g.members[0][key]
		}
		for _, field := range view.fields {
			out[field[0]] = s.aggregate(field[1], g.all, g.members)
		}
		rows = append(rows, out)
	}
	// An ungrouped view is one row, over no admitted row too.
	if len(view.groupBy) == 0 && len(rows) == 0 {
		out := Row{}
		for _, field := range view.fields {
			out[field[0]] = s.aggregate(field[1], nil, nil)
		}
		rows = append(rows, out)
	}
	if s.fault == "spurious-group" && len(view.groupBy) > 0 && len(rows) > 0 {
		extra := Row{}
		for key, value := range rows[0] {
			extra[key] = value
		}
		for _, key := range view.groupBy {
			extra[key] = "spurious"
		}
		rows = append(rows, extra)
	}
	return ViewResult{Rows: rows}, nil
}

func (*gsStore) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) { return nil, nil }
func (*gsStore) RedeliverEvent(RedeliveryRequest) error                         { return ErrUnsupported }
func (*gsStore) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}

func TestGroups(t *testing.T) {
	model, fault := os.Getenv("ESS_GROUPS_MODEL"), os.Getenv("ESS_GROUPS_FAULT")
	if _, ok := gsModels[model]; !ok {
		t.Fatalf("ESS_GROUPS_MODEL %q names no model", model)
	}
	// One store behind every scenario's target, as one deployed implementation is: the runtime
	// builds a target per scenario, and isolation is BeginScenario's to give.
	store := &gsStore{model: model, fault: fault}
	Run(t, func() Target { return store })
}
