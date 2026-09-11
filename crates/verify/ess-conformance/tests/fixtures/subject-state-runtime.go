package essconform

import (
	"fmt"
	"os"
	"testing"
)

const subjectID = "00000000-0000-4000-8000-000000000001"

type subjectBackend struct {
	row      Row
	ignore   bool
	sequence uint64
}

func (b *subjectBackend) Identity() (Identity, error) {
	return Identity{Name: "subject-state-fixture", Version: "1"}, nil
}
func (b *subjectBackend) BeginScenario(ScenarioContext) error { b.row = nil; return nil }
func (b *subjectBackend) EndScenario(ScenarioContext) error   { b.row = nil; return nil }
func (b *subjectBackend) ExecuteCommand(r CommandRequest) (CommandResult, error) {
	branch, event := "", "calls.core.Observed"
	payload := map[string]Node{}
	switch r.Command {
	case "calls.core.Open":
		b.row = Row{"call_id": subjectID, "state": "Init", "note": r.Input["note"]}
		branch, event, payload = "opened", "calls.core.Opened", map[string]Node{"call_id": subjectID}
	case "calls.core.Bridge", "calls.core.Report":
		if b.row == nil || r.Input["call_id"] != b.row["call_id"] {
			return CommandResult{}, nil
		}
		held := b.row["state"]
		if r.Command == "calls.core.Bridge" {
			if held == "Bridged" {
				branch, event = "already-bridged", ""
			} else {
				branch = "bridged"
				b.row["state"] = "Bridged"
			}
		} else {
			b.row["note"] = r.Input["note"]
			branch = "enriched"
			if r.Input["incoming"] == "Ringing" {
				switch held {
				case "Init":
					branch = "ringing"
					b.row["state"] = "Ringing"
				case "Ringing":
					branch = "refreshed"
				case "Bridged":
					branch = "preserved"
					// Keep the expected outcome but mutate the actual row: view observation must catch it.
					if b.ignore {
						b.row["state"] = "Ringing"
					}
				}
			}
		}
	default:
		return CommandResult{}, fmt.Errorf("unknown command %s", r.Command)
	}
	b.sequence++
	result := CommandResult{Outcome: branch, Consistency: fmt.Sprintf("seq:%d", b.sequence)}
	if event != "" {
		result.DirectEvents = []ObservedEvent{{Event: event, Payload: payload}}
	}
	return result, nil
}
func (b *subjectBackend) QueryView(ViewRequest) (ViewResult, error) {
	if b.row == nil {
		return ViewResult{}, nil
	}
	row := Row{}
	for k, v := range b.row {
		row[k] = v
	}
	return ViewResult{Rows: []Row{row}}, nil
}
func (b *subjectBackend) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}
func (b *subjectBackend) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return fmt.Errorf("held state is not an injected outcome")
}
func (b *subjectBackend) RedeliverEvent(RedeliveryRequest) error { return fmt.Errorf("no bindings") }
func (b *subjectBackend) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}

func TestSubjectStateRuntime(t *testing.T) {
	Run(t, func() Target { return &subjectBackend{ignore: os.Getenv("ESS_TEST_IGNORE_SUBJECT_GUARD") == "1"} })
}
