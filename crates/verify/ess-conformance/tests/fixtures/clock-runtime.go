package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"strings"
	"testing"
)

type clockFixture struct {
	correlation string
	wrong       string
}

func (c *clockFixture) Identity() (Identity, error) {
	return Identity{Name: "clock-fixture", Version: "1"}, nil
}
func (c *clockFixture) BeginScenario(s ScenarioContext) error {
	c.correlation = s.Correlation
	return nil
}
func (c *clockFixture) EndScenario(ScenarioContext) error { c.correlation = ""; return nil }
func (c *clockFixture) ExecuteCommand(CommandRequest) (CommandResult, error) {
	return CommandResult{}, ErrUnsupported
}
func (c *clockFixture) QueryView(ViewRequest) (ViewResult, error) {
	return ViewResult{}, ErrUnsupported
}
func (c *clockFixture) ConfigureExternalOutcome(ExternalOutcomeControl) error { return ErrUnsupported }
func (c *clockFixture) RedeliverEvent(RedeliveryRequest) error                { return ErrUnsupported }
func (c *clockFixture) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}
func (c *clockFixture) ObserveEvents(request EventObservationRequest) ([]ObservedEvent, error) {
	return []ObservedEvent{{Event: request.Event, Payload: map[string]Node{"offset": "2026-09-11T12:00:00+02:00", "seconds": json.Number("1789120800"), "local": "2026-09-11T12:00:00.000Z"}}}, nil
}
func (c *clockFixture) ObserveClockReading(request ReadingObservationRequest) (ClockReadingEvidence, error) {
	r := request.Reading
	if r.Event != "chronology.reading.Sampled" || r.Occurrence != 0 || request.Correlation != c.correlation {
		return ClockReadingEvidence{}, ErrUnsupported
	}
	// Independent declared model facts; standalone suite attachments cannot authenticate themselves.
	expected := map[string][3]string{"offset": {"OffsetText", "offset_date_time_text", "encoded_offset"}, "seconds": {"EpochSeconds", "unix_seconds", "encoding_defined_epoch"}, "local": {"LocalText", "local_date_time_millis_literal_z", "requires_observation"}}
	actual, ok := expected[r.Field]
	if !ok || r.DeclaredType != "chronology.reading."+actual[0] || r.Contract.Encoding != actual[1] {
		return ClockReadingEvidence{}, ErrUnsupported
	}
	origins := r.Contract.Origins
	if len(origins) < 1 || origins[0].Role != "producer_process" || origins[0].Offset != actual[2] || (r.Field != "local" && len(origins) != 1) || (r.Field == "local" && (len(origins) != 2 || origins[1].Role != "consumer_process" || origins[1].Offset != "requires_observation")) {
		return ClockReadingEvidence{}, ErrUnsupported
	}
	mode := map[string]string{"offset": "encoded_offset", "seconds": "unix_seconds", "local": "fixed_offset"}[r.Field]
	evidence := ClockReadingEvidence{Correlation: c.correlation, Occurrence: r.occurrenceKey(), ProcessInstance: "process-1", Epoch: "epoch-1", Origin: "producer_process", Formatter: mode}
	if r.Field == "local" {
		minutes := int16(120)
		evidence.OffsetMinutes = &minutes
	}
	switch c.wrong {
	case "unknown-source":
		evidence.ProcessInstance = ""
	case "unknown-origin":
		evidence.Origin = "unknown"
	case "stale-correlation":
		evidence.Correlation = "old"
	case "stale-occurrence":
		evidence.Occurrence = "old"
	case "source":
		if r.Field == "seconds" {
			evidence.ProcessInstance = "process-2"
		}
	case "epoch":
		if r.Field == "seconds" {
			evidence.Epoch = "epoch-2"
		}
	case "unknown-formatter":
		if r.Field == "local" {
			evidence.Formatter = "unknown"
			evidence.OffsetMinutes = nil
		}
	case "wrong-offset":
		if r.Field == "local" {
			minutes := int16(0)
			evidence.OffsetMinutes = &minutes
		}
	}
	return evidence, nil
}
func TestClockRunner(t *testing.T) {
	if mode := os.Getenv("CLOCK_FIXTURE_CHILD"); mode != "" {
		Run(t, func() Target { return &clockFixture{wrong: mode} })
		return
	}
	for _, test := range []struct {
		mode, message string
		success       bool
	}{
		{"valid", "PASS", true}, {"wrong-offset", "clock coordinate order differs", false},
		{"stale-correlation", "MismatchedOccurrence", false}, {"stale-occurrence", "MismatchedOccurrence", false},
		{"unknown-source", "UnknownEvidence", true}, {"unknown-origin", "UnknownEvidence", true},
		{"source", "DifferentClock", true}, {"epoch", "DifferentClock", true}, {"unknown-formatter", "UnknownEvidence", true},
	} {
		t.Run(test.mode, func(t *testing.T) {
			binary, err := os.Executable()
			if err != nil {
				t.Fatal(err)
			}
			command := exec.Command(binary, "-test.run=^TestClockRunner$", "-test.v")
			reportPath := t.TempDir() + "/report.json"
			command.Env = append(os.Environ(), "CLOCK_FIXTURE_CHILD="+test.mode, "ESS_REPORT_FORMAT=2", "ESS_REPORT_OUT="+reportPath)
			output, err := command.CombinedOutput()
			if (err == nil) != test.success || !strings.Contains(string(output), test.message) {
				t.Fatalf("mode %s exit %v:\n%s", test.mode, err, output)
			}
			raw, err := os.ReadFile(reportPath)
			if err != nil {
				t.Fatal(err)
			}
			var report map[string]any
			if err = json.Unmarshal(raw, &report); err != nil {
				t.Fatal(err)
			}
			expected := "inconclusive"
			if test.mode == "valid" {
				expected = "passed"
			} else if !test.success {
				expected = "failed"
			}
			if report["execution_status"] != expected {
				t.Fatalf("mode %s report: %s", test.mode, raw)
			}

		})
	}
}
func TestClockSuiteAdmission(t *testing.T) {
	if _, err := admitRunInput(suiteJSON); err != nil {
		t.Fatal(err)
	}
	for old := 1; old <= 5; old++ {
		if _, err := admitRunInput(strings.ReplaceAll(suiteJSON, "ess-conformance/6", fmt.Sprintf("ess-conformance/%d", old))); err == nil {
			t.Fatal("old version admitted reading", old)
		}
	}
	var document any
	if err := json.Unmarshal([]byte(suiteJSON), &document); err != nil {
		t.Fatal(err)
	}
	compact, err := json.Marshal(document)
	if err != nil {
		t.Fatal(err)
	}
	compactJSON := string(compact)
	for _, bad := range []string{strings.ReplaceAll(compactJSON, `"encoding":`, `"trusted":true,"encoding":`), strings.ReplaceAll(compactJSON, `"occurrence":0`, `"occurrence":65536`), strings.ReplaceAll(compactJSON, `"order":"equal"`, `"order":"elapsed"`)} {
		if _, err := admitRunInput(bad); err == nil {
			t.Fatal("malformed reading admitted")
		}
	}
}

func TestClockIntegerObservationTokens(t *testing.T) {
	for _, test := range []struct {
		raw, want string
		ok        bool
	}{
		{"1789120800", "1789120800", true}, {"1789120800.000", "1789120800", true}, {"1.7891208e9", "1789120800", true}, {"-0", "0", true},
		{"1789120800.000000001", "", false}, {"1e999999999", "", false}, {"01", "", false}, {"NaN", "", false}, {"-1", "", false},
	} {
		value, ok := clockIntegerText(test.raw)
		if ok != test.ok || value != test.want {
			t.Fatalf("%q => %q,%v", test.raw, value, ok)
		}
	}
}
