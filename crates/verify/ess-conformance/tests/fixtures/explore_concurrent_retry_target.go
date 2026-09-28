// A small in-memory implementation of `explore-retry/retry.yaml`, for the concurrent explorer lanes
// in `crates/edge/ess-cli/tests/explore_concurrent.rs`.
//
// `explore-concurrent-retry-target.mjs` is the same target in TypeScript, line for line: the two
// lanes' histories are compared byte for byte. A request is the command, its `document` and its
// correlation; one answered `seeded` before is answered `replayed`, with the retained response and
// no event. The mutant `unretained` is an InterleavedTarget that looks a request up when it is
// invoked and retains it only when it returns, so a retry sent while its original is in flight is
// applied as a new request and creates a second record.

package essconform

import "fmt"

type exploreRetryTarget struct {
	records  int
	retained map[string]CommandResult
}

func newExploreRetryTarget(mutant string) Target {
	target := &exploreRetryTarget{}
	target.reset()
	if mutant == "unretained" {
		return &exploreRetryUnretained{target}
	}
	return target
}

func (t *exploreRetryTarget) reset() {
	t.records = 0
	t.retained = map[string]CommandResult{}
}

func exploreRetryID(n int) string {
	return fmt.Sprintf("00000000-0000-4000-8000-%012d", n)
}

func exploreRetryKey(request CommandRequest) string {
	document, _ := request.Input["document"].(string)
	return request.Command + "\x00" + document + "\x00" + request.Correlation
}

func (t *exploreRetryTarget) Identity() (Identity, error) {
	return Identity{Name: "explore-retry-target", Version: "1"}, nil
}

func (t *exploreRetryTarget) BeginScenario(ScenarioContext) error { t.reset(); return nil }
func (t *exploreRetryTarget) EndScenario(ScenarioContext) error   { t.reset(); return nil }

// seed creates a record and retains its answer under key.
func (t *exploreRetryTarget) seed(key string) CommandResult {
	t.records++
	result := CommandResult{
		Outcome:      "seeded",
		Response:     map[string]Node{"revision_id": exploreRetryID(2 * t.records)},
		Consistency:  fmt.Sprintf("seq:%d", t.records),
		DirectEvents: []ObservedEvent{{Event: "retry.core.Seeded", Payload: map[string]Node{"record_id": exploreRetryID(2*t.records - 1)}}},
	}
	if _, known := t.retained[key]; !known {
		t.retained[key] = result
	}
	return result
}

func (t *exploreRetryTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	if request.Command != "retry.core.Seed" {
		return CommandResult{}, exploreFixtureUnsupported{request.Command + " is not a command of explore-retry"}
	}
	key := exploreRetryKey(request)
	if answered, known := t.retained[key]; known {
		return CommandResult{Outcome: "replayed", Response: answered.Response, Consistency: answered.Consistency, DirectEvents: []ObservedEvent{}}, nil
	}
	return t.seed(key), nil
}

// QueryView answers `Records` with every record, in the order it was created.
func (t *exploreRetryTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "retry.core.Records" {
		return ViewResult{}, exploreFixtureUnsupported{request.View + " is not a view of explore-retry"}
	}
	rows := []Row{}
	for n := 1; n <= t.records; n++ {
		rows = append(rows, Row{"record_id": exploreRetryID(2*n - 1)})
	}
	return ViewResult{Rows: rows}, nil
}

func (t *exploreRetryTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *exploreRetryTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return exploreFixtureUnsupported{"a replay is never forced"}
}

func (t *exploreRetryTarget) RedeliverEvent(RedeliveryRequest) error {
	return exploreFixtureUnsupported{"no bindings"}
}

func (t *exploreRetryTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, exploreFixtureUnsupported{"no bindings"}
}

type exploreRetryUnretained struct {
	*exploreRetryTarget
}

type exploreRetryCall struct {
	target     *exploreRetryTarget
	request    CommandRequest
	unretained bool
}

func (u *exploreRetryUnretained) InvokeCommand(request CommandRequest) PendingCommand {
	_, known := u.retained[exploreRetryKey(request)]
	return exploreRetryCall{target: u.exploreRetryTarget, request: request, unretained: !known}
}

func (c exploreRetryCall) Complete() (CommandResult, error) {
	key := exploreRetryKey(c.request)
	if _, known := c.target.retained[key]; known && c.unretained && c.request.Command == "retry.core.Seed" {
		return c.target.seed(key), nil
	}
	return c.target.ExecuteCommand(c.request)
}
