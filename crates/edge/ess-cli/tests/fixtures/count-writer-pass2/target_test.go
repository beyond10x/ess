package essconform

import (
	"os"
	"runtime"
	"testing"
)

type secondTarget struct{ Target }
type exitError struct{}

func (exitError) Is(target error) bool { return target == ErrUnsupported }
func (exitError) Error() string {
	runtime.Goexit()
	return "unreachable"
}
func (secondTarget) Identity() (Identity, error) {
	return Identity{Name: "second-count-review", Version: "1"}, nil
}
func (secondTarget) BeginScenario(ScenarioContext) error {
	switch os.Getenv("REVIEW_MODE") {
	case "begin-skip":
		return ErrUnsupported
	case "begin-error-format-goexit":
		return exitError{}
	}
	return nil
}
func (secondTarget) EndScenario(ScenarioContext) error { return nil }
func (secondTarget) QueryView(ViewRequest) (ViewResult, error) {
	return ViewResult{Rows: []Row{{"ready": true, "rows": []any{map[string]any{"ready": true}}}}}, nil
}
func TestReviewSecond(t *testing.T) {
	countReportNow = func() int64 { return 0 }
	Run(t, func() Target {
		if err := os.WriteFile(os.Getenv("REVIEW_MARKER"), []byte("constructed\n"), 0600); err != nil {
			panic(err)
		}
		return secondTarget{}
	})
}
