package essconform

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"
)

// The transcript target answers every question the Go runtime asks with the answer the Rust
// target gave the Rust reference runner to the same question (tests/support_go/mod.rs, Recorder).
// Questions are matched per scenario by method and key, in order; a request that differs from the
// recorded one is written to ESS_TRANSCRIPT_DIVERGENCE and fails the call.

type transcriptEntry struct {
	Method  string          `json:"method"`
	Key     string          `json:"key"`
	Request json.RawMessage `json:"request"`
	Error   *string         `json:"error"`
	Result  json.RawMessage `json:"result"`
}

var (
	transcriptOnce sync.Once
	transcriptData map[string][]transcriptEntry
	divergenceLock sync.Mutex
)

func loadTranscript(t *testing.T) {
	transcriptOnce.Do(func() {
		raw, err := os.ReadFile(os.Getenv("ESS_TRANSCRIPT"))
		if err != nil {
			t.Fatalf("transcript: %v", err)
		}
		if err := json.Unmarshal(raw, &transcriptData); err != nil {
			t.Fatalf("transcript: %v", err)
		}
	})
}

func diverged(format string, args ...any) error {
	divergenceLock.Lock()
	defer divergenceLock.Unlock()
	line := fmt.Sprintf(format, args...)
	file, err := os.OpenFile(os.Getenv("ESS_TRANSCRIPT_DIVERGENCE"), os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err == nil {
		fmt.Fprintln(file, strings.ReplaceAll(line, "\n", " "))
		file.Close()
	}
	return fmt.Errorf("transcript divergence: %s", line)
}

// canonical re-encodes a JSON value with exact numbers and sorted keys.
func canonical(raw []byte) string {
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	var value any
	if err := decoder.Decode(&value); err != nil {
		return "<" + err.Error() + ">"
	}
	encoded, _ := json.Marshal(integralTokens(value))
	return string(encoded)
}

// integralTokens spells every whole-number token the way Go does, so `1.0` and `1` compare equal: the
// Rust reference writes an Integer with a fractional part it does not have, and the value is one.
func integralTokens(value any) any {
	switch value := value.(type) {
	case json.Number:
		if whole, err := strconv.ParseFloat(value.String(), 64); err == nil && whole == float64(int64(whole)) && whole <= 9007199254740991 && whole >= -9007199254740991 {
			return json.Number(strconv.FormatInt(int64(whole), 10))
		}
		return value
	case []any:
		for index, element := range value {
			value[index] = integralTokens(element)
		}
		return value
	case map[string]any:
		for key, element := range value {
			value[key] = integralTokens(element)
		}
		return value
	default:
		return value
	}
}

type transcriptTarget struct {
	scenario string
	used     map[string]int
}

// repeatable methods answer with their last recorded answer once the recording runs out: a runner
// that polls a view one more time than the other has asked the same question again.
var repeatable = map[string]bool{"query_view": true, "observe_events": true, "observe_invocations": true}

func (b *transcriptTarget) next(method, key string, request any) (transcriptEntry, error) {
	var matching []transcriptEntry
	for _, entry := range transcriptData[b.scenario] {
		if entry.Method == method && entry.Key == key {
			matching = append(matching, entry)
		}
	}
	n := b.used[method+"\x00"+key]
	b.used[method+"\x00"+key] = n + 1
	var entry transcriptEntry
	switch {
	case n < len(matching):
		entry = matching[n]
	case len(matching) > 0 && repeatable[method]:
		entry = matching[len(matching)-1]
	default:
		return entry, diverged("%s: %s `%s` call %d was never made by the reference runner", b.scenario, method, key, n+1)
	}
	sent, _ := json.Marshal(request)
	if got, want := canonical(sent), canonical(entry.Request); got != want {
		return entry, diverged("%s: %s `%s` call %d sent %s, the reference runner sent %s", b.scenario, method, key, n+1, got, want)
	}
	if entry.Error != nil {
		if *entry.Error == "unsupported" {
			return entry, ErrUnsupported
		}
		return entry, fmt.Errorf("%s", *entry.Error)
	}
	return entry, nil
}

func decodeInto(raw json.RawMessage, target any) error {
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	return decoder.Decode(target)
}

type recordedEvent struct {
	Event   string          `json:"event"`
	Payload map[string]Node `json:"payload"`
}

type recordedResult struct {
	Outcome      *string         `json:"outcome"`
	Error        *string         `json:"error"`
	Consistency  *string         `json:"consistency"`
	DirectEvents []recordedEvent `json:"direct_events"`
	Response     map[string]Node `json:"response"`
	// ErrorPayload is the declared error's payload the reference target carried.
	ErrorPayload map[string]Node `json:"error_payload"`
	// NotGranted is the standard refusal for an actor no grant admits (beyond10x/ess#265).
	NotGranted      bool    `json:"not_granted"`
	NotGrantedActor *string `json:"not_granted_actor"`
}

func (r recordedResult) result() CommandResult {
	result := CommandResult{Response: r.Response}
	if r.Outcome != nil {
		result.Outcome = *r.Outcome
	}
	if r.Error != nil {
		result.Error = *r.Error
		result.ErrorPayload = r.ErrorPayload
	}
	if r.Consistency != nil {
		result.Consistency = *r.Consistency
	}
	for _, event := range r.DirectEvents {
		result.DirectEvents = append(result.DirectEvents, ObservedEvent{Event: event.Event, Payload: event.Payload})
	}
	result.NotGranted = r.NotGranted
	if r.NotGrantedActor != nil {
		result.NotGrantedActor = *r.NotGrantedActor
	}
	return result
}

func orNull(value string) any {
	if value == "" {
		return nil
	}
	return value
}

func orEmpty(value map[string]Node) map[string]Node {
	if value == nil {
		return map[string]Node{}
	}
	return value
}

func orNullMap(value map[string]Node) any {
	if value == nil {
		return nil
	}
	return value
}

func (b *transcriptTarget) Identity() (Identity, error) {
	return Identity{Name: "transcript", Version: "1"}, nil
}
func (b *transcriptTarget) BeginScenario(context ScenarioContext) error {
	b.scenario, b.used = context.Scenario, map[string]int{}
	return nil
}
func (b *transcriptTarget) EndScenario(ScenarioContext) error { return nil }
func (b *transcriptTarget) command(method string, command string, request any) (CommandResult, error) {
	entry, err := b.next(method, command, request)
	if err != nil {
		return CommandResult{}, err
	}
	var recorded recordedResult
	if err := decodeInto(entry.Result, &recorded); err != nil {
		return CommandResult{}, err
	}
	return recorded.result(), nil
}
func (b *transcriptTarget) ExecuteCommand(r CommandRequest) (CommandResult, error) {
	return b.command("execute_command", r.Command, map[string]any{
		"actor": orNull(r.Actor), "caller": orNullMap(r.Caller), "input": orEmpty(r.Input),
	})
}
func (b *transcriptTarget) ExecuteCommandWithoutInput(r AbsentInputRequest) (CommandResult, error) {
	return b.command("execute_command_without_input", r.Command, map[string]any{
		"actor": orNull(r.Actor), "caller": orNullMap(r.Caller),
	})
}
func (b *transcriptTarget) QueryView(r ViewRequest) (ViewResult, error) {
	request := map[string]any{"params": orEmpty(r.Params), "at_least": r.AtLeast}
	// A read sent as an actor (beyond10x/ess#286) was recorded with it, and only then.
	if r.Actor != "" {
		request["actor"] = r.Actor
	}
	if r.Anonymous {
		request["anonymous"] = true
	}
	entry, err := b.next("query_view", r.View, request)
	if err != nil {
		return ViewResult{}, err
	}
	var recorded struct {
		Rows            []Row   `json:"rows"`
		Total           *uint64 `json:"total"`
		NotGranted      bool    `json:"not_granted"`
		NotGrantedActor *string `json:"not_granted_actor"`
	}
	if err := decodeInto(entry.Result, &recorded); err != nil {
		return ViewResult{}, err
	}
	result := ViewResult{Rows: recorded.Rows, Total: recorded.Total, NotGranted: recorded.NotGranted}
	if recorded.NotGrantedActor != nil {
		result.NotGrantedActor = *recorded.NotGrantedActor
	}
	return result, nil
}
func (b *transcriptTarget) ObserveEvents(r EventObservationRequest) ([]ObservedEvent, error) {
	entry, err := b.next("observe_events", r.Event, map[string]any{})
	if err != nil {
		return nil, err
	}
	var recorded []recordedEvent
	if err := decodeInto(entry.Result, &recorded); err != nil {
		return nil, err
	}
	seen := make([]ObservedEvent, 0, len(recorded))
	for _, event := range recorded {
		seen = append(seen, ObservedEvent{Event: event.Event, Payload: event.Payload})
	}
	return seen, nil
}
func (b *transcriptTarget) ConfigureExternalOutcome(c ExternalOutcomeControl) error {
	_, err := b.next("configure_external_outcome", c.Command+"/"+c.Outcome, map[string]any{})
	return err
}
func (b *transcriptTarget) ConfigureExternalOutcomeRepeatedly(c ExternalOutcomeControl, times int) error {
	_, err := b.next("configure_external_outcome_repeatedly", c.Command+"/"+c.Outcome, map[string]any{"times": times})
	return err
}
func (b *transcriptTarget) RedeliverEvent(r RedeliveryRequest) error {
	_, err := b.next("redeliver_event", r.Event, map[string]any{})
	return err
}

// DeliverEvent replays an event an external channel delivered (ess/18).
func (b *transcriptTarget) DeliverEvent(r EventDeliveryRequest) error {
	_, err := b.next("deliver_event", r.Event+"|"+r.Authority, map[string]any{})
	return err
}
func (b *transcriptTarget) ObserveInvocations(r InvocationObservationRequest) ([]Invocation, error) {
	entry, err := b.next("observe_invocations", r.Binding+"|"+r.Command, map[string]any{})
	if err != nil {
		return nil, err
	}
	var recorded []Invocation
	if err := decodeInto(entry.Result, &recorded); err != nil {
		return nil, err
	}
	return recorded, nil
}
func (b *transcriptTarget) EstablishEntity(r EntitySetupRequest) error {
	_, err := b.next("establish_entity", r.Entity, map[string]any{
		"identity": r.Identity, "fields": orEmpty(r.Fields), "state": r.State,
	})
	return err
}

// bareTarget is the transcript target with only the methods of Target: every optional interface
// (AbsentInputTarget, RepeatedOutcomeTarget, EntitySetupTarget, Clock) is hidden, as it is for a
// target written before them.
type bareTarget struct{ Target }

func TestTranscriptReplay(t *testing.T) {
	loadTranscript(t)
	if millis := os.Getenv("ESS_TEST_WALL_MILLIS"); millis != "" {
		var at int64
		fmt.Sscan(millis, &at)
		wallClock = func() time.Time { return time.UnixMilli(at) }
	}
	if os.Getenv("ESS_TRANSCRIPT_BARE") == "1" {
		Run(t, func() Target { return bareTarget{&transcriptTarget{used: map[string]int{}}} })
		return
	}
	Run(t, func() Target { return &transcriptTarget{used: map[string]int{}} })
}
