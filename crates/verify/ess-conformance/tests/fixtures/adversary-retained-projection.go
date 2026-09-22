package essconform

import (
	"os"
	"os/exec"
	"strings"
	"testing"
)

type adversaryProjectionFixture struct {
	retainedFixture
	incomplete bool
}

func (f *adversaryProjectionFixture) QueryView(request ViewRequest) (ViewResult, error) {
	result, err := f.retainedFixture.QueryView(request)
	if f.incomplete {
		result.Rows = []map[string]Node{{"record_id": replayID}}
	}
	return result, err
}
func TestAdversarySnapshotProjection(t *testing.T) {
	if mode := os.Getenv("ADVERSARY_PROJECTION_CHILD"); mode != "" {
		Run(t, func() Target {
			return &adversaryProjectionFixture{retainedFixture: retainedFixture{mode: "valid"}, incomplete: mode == "incomplete"}
		})
		return
	}
	for _, mode := range []string{"complete", "incomplete"} {
		t.Run(mode, func(t *testing.T) {
			binary, err := os.Executable()
			if err != nil {
				t.Fatal(err)
			}
			command := exec.Command(binary, "-test.run=^TestAdversarySnapshotProjection$", "-test.v")
			command.Env = append(os.Environ(), "ADVERSARY_PROJECTION_CHILD="+mode, "ESS_REPORT_FORMAT=2")
			output, err := command.CombinedOutput()
			if strings.Contains(string(output), "SKIP") || strings.Contains(string(output), "unsupported") {
				t.Fatalf("projection case did not execute: %s", output)
			}
			if (err == nil) != (mode == "complete") {
				t.Fatalf("%s: identity-only rows must not certify complete original subject preservation: %v\n%s", mode, err, output)
			}
		})
	}
}
