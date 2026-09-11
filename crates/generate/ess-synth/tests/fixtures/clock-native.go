import (
	p "example.invalid/chronology/types/primitives"
	"testing"
)

func clockFacts(formatter string) p.ClockReadingEvidence {
	return p.ClockReadingEvidence{Correlation: "scenario-1", Occurrence: "sample#0.value", ProcessInstance: "process-1", Epoch: "epoch-1", Origin: "producer_process", Formatter: formatter}
}
func clockEpoch(t *testing.T) p.ClockCoordinate {
	t.Helper()
	value, err := NewEpochSeconds(1789120800).ResolveReading(clockFacts("unix_seconds"), "scenario-1", "sample#0.value")
	if err != nil {
		t.Fatal(err)
	}
	return value
}
func TestClockOffsetEpochAgree(t *testing.T) {
	value, err := NewOffsetText("2026-09-11T12:00:00+02:00").ResolveReading(clockFacts("encoded_offset"), "scenario-1", "sample#0.value")
	if err != nil {
		t.Fatal(err)
	}
	order, err := p.CompareClockReadings(value, clockEpoch(t))
	if err != nil || order != 0 {
		t.Fatalf("order=%d err=%v", order, err)
	}
}
func TestClockLocalObservedOffset(t *testing.T) {
	value := NewLocalText("2026-09-11T12:00:00.000Z")
	facts := clockFacts("fixed_offset")
	minutes := int16(120)
	facts.OffsetMinutes = &minutes
	normalized, err := value.ResolveReading(facts, "scenario-1", "sample#0.value")
	if err != nil {
		t.Fatal(err)
	}
	order, err := p.CompareClockReadings(normalized, clockEpoch(t))
	if err != nil || order != 0 {
		t.Fatal(order, err)
	}
	minutes = 0
	normalized, err = value.ResolveReading(facts, "scenario-1", "sample#0.value")
	if err != nil {
		t.Fatal(err)
	}
	order, err = p.CompareClockReadings(normalized, clockEpoch(t))
	if err != nil || order != 1 {
		t.Fatal(order, err)
	}
}
func TestClockLiteralZHasNoAuthority(t *testing.T) {
	_, err := NewLocalText("2026-09-11T12:00:00.000Z").ResolveReading(clockFacts("unknown"), "scenario-1", "sample#0.value")
	if err == nil {
		t.Fatal("missing formatter accepted")
	}
}
func TestClockStaleEvidence(t *testing.T) {
	for _, scope := range [][2]string{{"old", "sample#0.value"}, {"scenario-1", "old"}} {
		if _, err := NewEpochSeconds(1).ResolveReading(clockFacts("unix_seconds"), scope[0], scope[1]); err == nil {
			t.Fatal("stale facts accepted")
		}
	}
}
func TestClockDifferentSourceAndEpoch(t *testing.T) {
	for _, change := range []string{"source", "epoch"} {
		facts := clockFacts("unix_seconds")
		if change == "source" {
			facts.ProcessInstance = "other"
		} else {
			facts.Epoch = "other"
		}
		value, err := NewEpochSeconds(1).ResolveReading(facts, "scenario-1", "sample#0.value")
		if err != nil {
			t.Fatal(err)
		}
		if _, err = p.CompareClockReadings(value, clockEpoch(t)); err == nil {
			t.Fatal("different clock accepted")
		}
	}
}
func TestClockInvalidScalarAndOrigin(t *testing.T) {
	for _, text := range []string{"2026-02-29T00:00:00Z", "2026-09-11T12:00:60Z", "2026-09-11T12:00:00+14:01", "1970-01-01T00:00:00+01:00"} {
		if _, err := NewOffsetText(text).ResolveReading(clockFacts("encoded_offset"), "scenario-1", "sample#0.value"); err == nil {
			t.Fatal(text)
		}
	}
	facts := clockFacts("unix_seconds")
	facts.Origin = "unknown"
	if _, err := NewEpochSeconds(1).ResolveReading(facts, "scenario-1", "sample#0.value"); err == nil {
		t.Fatal("erased origin accepted")
	}
	if _, err := NewEpochSeconds(9223372036854775807).ResolveReading(clockFacts("unix_seconds"), "scenario-1", "sample#0.value"); err == nil {
		t.Fatal("overflow accepted")
	}
}
