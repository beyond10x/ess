package essconform

import (
	"encoding/json"
	"os"
	"os/exec"
	"strings"
	"testing"
)

type originChangingClock struct {
	clockFixture
	mutate bool
}

func (c *originChangingClock) ObserveClockReading(request ReadingObservationRequest) (ClockReadingEvidence, error) {
	// Validate the original model contract before touching the request DTO.
	evidence, err := c.clockFixture.ObserveClockReading(request)
	if err != nil {
		return evidence, err
	}
	if request.Reading.Field == "offset" || request.Reading.Field == "seconds" {
		// The actual source reports an undeclared origin in both controls.
		evidence.Origin = "consumer_process"
		if c.mutate {
			// A by-value observation request must not rewrite the runner's contract.
			request.Reading.Contract.Origins[0].Role = "consumer_process"
		}
	}
	return evidence, nil
}

func TestClockAdversaryRequestMutationCannotAuthorizeOrigin(t *testing.T) {
	if mode := os.Getenv("CLOCK_AUTHORITY_CHILD"); mode != "" {
		Run(t, func() Target { return &originChangingClock{mutate: mode == "mutate"} })
		return
	}
	for _, mode := range []string{"control", "mutate"} {
		t.Run(mode, func(t *testing.T) {
			binary, err := os.Executable()
			if err != nil {
				t.Fatal(err)
			}
			command := exec.Command(binary, "-test.run=^TestClockAdversaryRequestMutationCannotAuthorizeOrigin$", "-test.v")
			reportPath := t.TempDir() + "/report.json"
			command.Env = append(os.Environ(), "CLOCK_AUTHORITY_CHILD="+mode, "ESS_REPORT_FORMAT=2", "ESS_REPORT_OUT="+reportPath)
			output, exit := command.CombinedOutput()
			raw, err := os.ReadFile(reportPath)
			if err != nil {
				t.Fatalf("read report: %v; child exit=%v output=%s", err, exit, output)
			}
			var report map[string]any
			if err := json.Unmarshal(raw, &report); err != nil {
				t.Fatal(err)
			}
			t.Logf("mode=%s child exit=%v execution_status=%v\n%s", mode, exit, report["execution_status"], output)
			if mode == "control" && !strings.Contains(string(output), "IncompatibleOrigin") {
				t.Fatalf("control must reach the declared-origin check: %s", output)
			}
			if report["execution_status"] == "passed" {
				t.Fatalf("request mutation authorized an undeclared origin; exact report:\n%s", raw)
			}
		})
	}
}
