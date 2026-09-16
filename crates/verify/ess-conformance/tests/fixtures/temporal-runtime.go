package essconform

import (
	"encoding/json"
	"errors"
	"testing"
)

func temporalLedger(t *testing.T, records []Occurrence, through uint64) *ObservationLedger {
	t.Helper()
	ledger, err := NewObservationLedger("subscribed-before-stimulus")
	if err != nil {
		t.Fatal(err)
	}
	if err := ledger.Accept(ObservationBatch{Lifetime: "subscribed-before-stimulus", CompleteBeforeMS: through, Occurrences: records}); err != nil {
		t.Fatal(err)
	}
	return ledger
}

func temporalKind(t *testing.T, err error, expected string) *TemporalError {
	t.Helper()
	if expected == "" {
		if err != nil {
			t.Fatal(err)
		}
		return nil
	}
	var detail *TemporalError
	if !errors.As(err, &detail) || detail.Kind != expected {
		t.Fatalf("wanted %s; got %v", expected, err)
	}
	return detail
}

func TestNestedStabilityCannotFilterAwayItsCounterexample(t *testing.T) {
	name := "calls.live.Metrics"
	var records []Occurrence
	for index, waiting := range []string{"3", "0", "3"} {
		records = append(records, Occurrence{
			Sequence: uint64(index + 1), AtMS: uint64(index + 1), Event: name,
			Payload: map[string]Node{"snapshot": map[string]Node{"account": "owned", "waiting": waiting}},
		})
	}
	ledger := temporalLedger(t, records, 20)
	scope := OccurrenceMatcher{Event: name, Payload: map[string]Node{"snapshot.account": "owned"}}
	required := map[string]Node{"snapshot.waiting": "3"}
	err := temporalKind(t, ledger.Stable(scope, required, 1, 10), "counterexample")
	if err.Counterexample.Sequence != 2 {
		t.Fatal("transient middle frame was concealed")
	}
	scope.Payload = map[string]Node{"snapshot": map[string]Node{"account": "owned", "waiting": "3"}}
	temporalKind(t, ledger.Stable(scope, required, 1, 10), "filtered_claim")
	scope.Payload = map[string]Node{"snapshot.unknown": "3"}
	if found, err := ledger.Find(scope, 0); err != nil || found != nil {
		t.Fatalf("missing path matched: %v %v", found, err)
	}
}

func TestTemporalActualPayloadAndDistinctOccurrences(t *testing.T) {
	event := "calls.live.Bridged"
	records := []Occurrence{
		{Sequence: 1, AtMS: 1, Event: event, Payload: map[string]Node{"call": "other", "agent": "alice"}},
		{Sequence: 2, AtMS: 2, Event: event, Payload: map[string]Node{"call": "owned", "agent": "bob"}},
		{Sequence: 3, AtMS: 3, Event: event, Payload: map[string]Node{"call": "owned", "agent": "alice"}},
		{Sequence: 4, AtMS: 3, Event: event, Payload: map[string]Node{"call": "owned", "agent": "alice"}},
	}
	ledger := temporalLedger(t, records, 4)
	records[2].Payload["call"] = "mutated-after-observation"
	matcher := OccurrenceMatcher{Event: event, Payload: map[string]Node{"call": "owned", "agent": "alice"}}
	found, err := ledger.Find(matcher, 0)
	if err != nil || found == nil || found.Sequence != 3 {
		t.Fatalf("wrong occurrence: %+v %v", found, err)
	}
	found.Payload["call"] = "mutated-lookup"
	found, err = ledger.Find(matcher, 3)
	if err != nil || found == nil || found.Sequence != 4 {
		t.Fatalf("duplicate occurrence collapsed: %+v %v", found, err)
	}
	temporalKind(t, ledger.Ordered(3, 4), "")
	temporalKind(t, ledger.Ordered(4, 3), "order")
	temporalKind(t, ledger.Ordered(3, 3), "order")
	_, err = ledger.Find(matcher, 9)
	temporalKind(t, err, "unknown_anchor")
}

func TestTemporalBoundedAbsenceAndExactExpiry(t *testing.T) {
	matcher := OccurrenceMatcher{Event: "calls.live.Offered", Payload: map[string]Node{"agent": "alice"}}
	for _, at := range []uint64{100, 101, 5000, 20099, 20100} {
		ledger := temporalLedger(t, []Occurrence{
			{Sequence: 1, AtMS: 100, Event: "calls.live.WrapUp"},
			{Sequence: 2, AtMS: at, Event: matcher.Event, Payload: matcher.Payload},
		}, 20101)
		expected := "counterexample"
		if at == 20100 {
			expected = ""
		}
		temporalKind(t, ledger.Absent(matcher, 1, 20000), expected)
		temporalKind(t, ledger.Absent(matcher, 1, 0), "invalid_window")
		temporalKind(t, ledger.Absent(matcher, 1, ^uint64(0)), "invalid_window")
	}
	ledger := temporalLedger(t, []Occurrence{{Sequence: 1, AtMS: 100, Event: "calls.live.WrapUp"}}, 101)
	temporalKind(t, ledger.Absent(matcher, 1, 20000), "unfinished")
	temporalKind(t, ledger.Accept(ObservationBatch{Lifetime: "subscribed-before-stimulus", After: 1, CompleteBeforeMS: 20100}), "")
	temporalKind(t, ledger.Absent(matcher, 1, 20000), "")
}

func TestTemporalStabilityChecksEverySnapshotAndNumericValue(t *testing.T) {
	scope := OccurrenceMatcher{Event: "calls.live.Metrics", Payload: map[string]Node{"queue": "owned"}}
	for incorrect := -1; incorrect < 3; incorrect++ {
		records := []Occurrence{}
		for i, at := range []uint64{100, 500, 10099} {
			waiting := json.Number("3")
			if i == incorrect {
				waiting = json.Number("0")
			}
			records = append(records, Occurrence{Sequence: uint64(i + 1), AtMS: at, Event: scope.Event, Payload: map[string]Node{"queue": "owned", "waiting": waiting}})
		}
		ledger := temporalLedger(t, records, 10100)
		expected := "counterexample"
		if incorrect == -1 {
			expected = ""
		}
		detail := temporalKind(t, ledger.Stable(scope, map[string]Node{"waiting": 3}, 1, 10000), expected)
		if detail != nil {
			if detail.Counterexample.Sequence != uint64(incorrect+1) {
				t.Fatal("lost the actual transient counterexample")
			}
			detail.Counterexample.Payload["waiting"] = 3
			temporalKind(t, ledger.Stable(scope, map[string]Node{"waiting": 3}, 1, 10000), "counterexample")
		}
		temporalKind(t, ledger.Stable(scope, nil, 1, 10000), "empty_claim")
		temporalKind(t, ledger.Stable(OccurrenceMatcher{Event: scope.Event, Payload: map[string]Node{"waiting": 3}}, map[string]Node{"waiting": 3}, 1, 10000), "filtered_claim")
	}
	ledger := temporalLedger(t, []Occurrence{{Sequence: 1, AtMS: 1, Event: scope.Event, Payload: map[string]Node{"queue": "owned", "waiting": json.Number("9007199254740993")}}}, 20000)
	temporalKind(t, ledger.Stable(scope, map[string]Node{"waiting": json.Number("9007199254740992")}, 1, 10000), "counterexample")
}

func TestTemporalIncompleteLifetimesCannotRecover(t *testing.T) {
	bad := []ObservationBatch{
		{Lifetime: "reconnected", After: 1, CompleteBeforeMS: 4},
		{Lifetime: "subscribed-before-stimulus", After: 0, CompleteBeforeMS: 4},
		{Lifetime: "subscribed-before-stimulus", After: 1, CompleteBeforeMS: 1},
	}
	for _, item := range []Occurrence{
		{Sequence: 3, AtMS: 3, Event: "calls.live.Bridged"},
		{Sequence: 1, AtMS: 3, Event: "calls.live.Bridged"},
		{Sequence: 2, AtMS: 1, Event: "calls.live.Bridged"},
		{Sequence: 2, AtMS: 4, Event: "calls.live.Bridged"},
	} {
		bad = append(bad, ObservationBatch{Lifetime: "subscribed-before-stimulus", After: 1, CompleteBeforeMS: 4, Occurrences: []Occurrence{item}})
	}
	for _, batch := range bad {
		ledger := temporalLedger(t, []Occurrence{{Sequence: 1, AtMS: 1, Event: "calls.live.WrapUp"}}, 2)
		first := temporalKind(t, ledger.Accept(batch), "incomplete")
		if ledger.Cursor() != 1 {
			t.Fatal("partially admitted batch")
		}
		next := temporalKind(t, ledger.Accept(ObservationBatch{Lifetime: "subscribed-before-stimulus", After: 1, CompleteBeforeMS: 30000}), "incomplete")
		if first.Reason != next.Reason {
			t.Fatal("lost the original gap")
		}
		temporalKind(t, ledger.Absent(OccurrenceMatcher{Event: "calls.live.Offered"}, 1, 20000), "incomplete")
	}
	for _, reason := range []string{"disconnect", "decode", "overflow", "canceled"} {
		ledger := temporalLedger(t, []Occurrence{{Sequence: 1, AtMS: 1, Event: "calls.live.WrapUp"}}, 2)
		temporalKind(t, ledger.Invalidate(reason), "incomplete")
		temporalKind(t, ledger.Accept(ObservationBatch{Lifetime: "subscribed-before-stimulus", After: 1, CompleteBeforeMS: 30000}), "incomplete")
	}
}
