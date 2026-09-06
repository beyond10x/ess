package essconform

import (
	"errors"
	"os"
	"runtime"
	"testing"
)

type reviewTarget struct{ Target }

func (reviewTarget) Identity() (Identity, error) {
	return Identity{Name: "count-review", Version: "1"}, nil
}
func (reviewTarget) BeginScenario(ScenarioContext) error {
	if os.Getenv("REVIEW_MODE") == "begin-skip" {
		return ErrUnsupported
	}
	return nil
}
func (reviewTarget) EndScenario(ScenarioContext) error {
	switch os.Getenv("REVIEW_MODE") {
	case "skip-end-error":
		return errors.New("teardown did return an error")
	case "skip-end-goexit":
		runtime.Goexit()
	}
	return nil
}
func (reviewTarget) ExecuteCommand(CommandRequest) (CommandResult, error) {
	return CommandResult{}, ErrUnsupported
}
func (reviewTarget) QueryView(ViewRequest) (ViewResult, error) {
	return ViewResult{Rows: []Row{{"rows": []any{map[string]any{"ready": true}}}}}, nil
}
func TestReview(t *testing.T) {
	countReportNow = func() int64 { return 0 }
	Run(t, func() Target {
		if err := os.WriteFile(os.Getenv("REVIEW_MARKER"), []byte("constructed\n"), 0600); err != nil {
			panic(err)
		}
		return reviewTarget{}
	})
}
