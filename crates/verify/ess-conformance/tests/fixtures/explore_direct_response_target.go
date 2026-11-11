// An in-memory implementation of `explore-direct-response.yaml`. It is copied into the package as
// `explore_external_target_test.go`, which `explore_external_driver_test.go` runs.
//
// `explore-direct-response-target.mjs` is the same target in TypeScript, line for line; its
// header lists the modes.

package essconform

import "fmt"

type exploreExtUnsupported struct{ reason string }

func (e exploreExtUnsupported) Error() string { return e.reason }
func (e exploreExtUnsupported) Unwrap() error { return ErrUnsupported }

type exploreExtTarget struct {
	mode    string
	minted  int
	version int
}

func newExploreExtTarget(mode string) *exploreExtTarget {
	return &exploreExtTarget{mode: mode}
}

func (t *exploreExtTarget) reset() {
	t.minted = 0
	t.version = 0
}

func (t *exploreExtTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-direct-response-target", Version: "1"}, nil
}

func (t *exploreExtTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreExtTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

func (t *exploreExtTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	if request.Command != "explorert.keys.Mint" {
		return CommandResult{}, exploreExtUnsupported{request.Command + " is not a command of explore-direct-response.yaml"}
	}
	label, _ := request.Input["label"].(string)
	if len([]rune(label)) > 8 {
		return CommandResult{Outcome: "too-long", Error: "explorert.keys.TooLong", DirectEvents: []ObservedEvent{}}, nil
	}
	t.minted++
	t.version++
	response := map[string]Node{"token": fmt.Sprintf("tok-%d", t.minted), "label": label}
	switch t.mode {
	case "drops-token":
		delete(response, "token")
	case "null-token":
		response["token"] = nil
	case "null-note":
		response["note"] = nil
	case "other-label":
		response["label"] = fmt.Sprintf("label-%d", t.minted)
	case "with-note":
		response["note"] = "a note"
	}
	return CommandResult{
		Outcome:      "minted",
		Consistency:  fmt.Sprintf("v%d", t.version),
		DirectEvents: []ObservedEvent{},
		Response:     response,
	}, nil
}

func (t *exploreExtTarget) QueryView(request ViewRequest) (ViewResult, error) {
	return ViewResult{}, exploreExtUnsupported{request.View + " is not a view of explore-direct-response.yaml"}
}

func (t *exploreExtTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreExtTarget) ConfigureExternalOutcome(control ExternalOutcomeControl) error {
	return exploreExtUnsupported{"no external branches"}
}

func (t *exploreExtTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreExtUnsupported{"no redelivery"}
}

func (t *exploreExtTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreExtUnsupported{"no bindings"}
}
