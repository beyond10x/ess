// The generated Go runner's target for a generated service: every request goes over the line
// protocol the harness under `ESS_SERVED_BINARY` speaks (`upsert-by-existence-go-harness/main.go`
// and its Rust twin), with the route table in `ESS_SERVED_ROUTES` as `name method path` triples.
// Nothing here decides a branch or an event: the generated service answers, and this file only
// carries the request there and the answer back, as `tests/emit_swap_served.rs`'s Rust adapter
// does for the native runner.
package essconform

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"
	"sync"
	"testing"
)

type served struct {
	in        io.WriteCloser
	out       *bufio.Reader
	published []ObservedEvent
	sequence  int
}

var (
	once     sync.Once
	shared   *served
	startErr error
)

func start() (*served, error) {
	command := exec.Command(os.Getenv("ESS_SERVED_BINARY"), strings.Fields(os.Getenv("ESS_SERVED_ROUTES"))...)
	command.Stderr = os.Stderr
	in, err := command.StdinPipe()
	if err != nil {
		return nil, err
	}
	out, err := command.StdoutPipe()
	if err != nil {
		return nil, err
	}
	if err := command.Start(); err != nil {
		return nil, err
	}
	return &served{in: in, out: bufio.NewReader(out)}, nil
}

func (s *served) ask(request map[string]any) (map[string]any, error) {
	line, err := json.Marshal(request)
	if err != nil {
		return nil, err
	}
	if _, err := s.in.Write(append(line, '\n')); err != nil {
		return nil, err
	}
	text, err := s.out.ReadString('\n')
	if err != nil {
		return nil, err
	}
	decoder := json.NewDecoder(strings.NewReader(text))
	decoder.UseNumber()
	var answer map[string]any
	if err := decoder.Decode(&answer); err != nil {
		return nil, fmt.Errorf("`%s`: %w", text, err)
	}
	return answer, nil
}

func (s *served) Identity() (Identity, error) {
	return Identity{Name: "emit-swap-served-generated", Version: "1"}, nil
}

func (s *served) BeginScenario(ScenarioContext) error {
	answer, err := s.ask(map[string]any{"op": "reset"})
	if err != nil {
		return err
	}
	if answer["ok"] != true {
		return fmt.Errorf("opening a scenario: %v", answer)
	}
	s.published = nil
	s.sequence = 0
	return nil
}

func (s *served) EndScenario(ScenarioContext) error {
	s.published = nil
	return nil
}

func (s *served) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	body, err := json.Marshal(request.Input)
	if err != nil {
		return CommandResult{}, err
	}
	var actor any
	if request.Actor != "" {
		actor = request.Actor
	}
	answer, err := s.ask(map[string]any{"op": "command", "command": request.Command, "actor": actor, "body": string(body)})
	if err != nil {
		return CommandResult{}, err
	}
	number, ok := answer["status"].(json.Number)
	if !ok {
		return CommandResult{}, fmt.Errorf("invoking `%s`: %v", request.Command, answer)
	}
	status, err := number.Int64()
	if err != nil {
		return CommandResult{}, err
	}
	reply, _ := answer["answer"].(map[string]any)
	if status == 403 && reply["refused"] == "not granted" {
		named, _ := reply["actor"].(string)
		return CommandResult{NotGranted: true, NotGrantedActor: named}, nil
	}
	if status == 501 {
		return CommandResult{}, nil
	}
	outcome, ok := reply["outcome"].(string)
	if !ok {
		return CommandResult{}, fmt.Errorf("invoking `%s`: %v", request.Command, answer)
	}
	result := CommandResult{Outcome: outcome}
	if declared, ok := reply["error"].(string); ok {
		result.Error = declared
		if payload, ok := reply["payload"].(map[string]any); ok {
			result.ErrorPayload = map[string]Node{}
			for field, value := range payload {
				if value != nil {
					result.ErrorPayload[field] = value
				}
			}
		}
	}
	published, _ := reply["published"].([]any)
	for _, each := range published {
		entry, _ := each.(map[string]any)
		name, ok := entry["event"].(string)
		if !ok {
			return CommandResult{}, fmt.Errorf("invoking `%s`: %v", request.Command, answer)
		}
		event := ObservedEvent{Event: name, Payload: map[string]Node{}}
		if payload, ok := entry["payload"].(map[string]any); ok {
			for field, value := range payload {
				event.Payload[field] = value
			}
		}
		result.DirectEvents = append(result.DirectEvents, event)
		s.published = append(s.published, event)
	}
	s.sequence++
	result.Consistency = fmt.Sprintf("seq:%d", s.sequence)
	return result, nil
}

func (s *served) QueryView(request ViewRequest) (ViewResult, error) {
	answer, err := s.ask(map[string]any{"op": "view", "view": request.View})
	if err != nil {
		return ViewResult{}, err
	}
	reply, _ := answer["answer"].(map[string]any)
	rows, ok := reply["rows"].([]any)
	if !ok {
		return ViewResult{}, fmt.Errorf("reading `%s`: %v", request.View, answer)
	}
	result := ViewResult{Rows: []Row{}}
	for _, each := range rows {
		row, _ := each.(map[string]any)
		result.Rows = append(result.Rows, Row(row))
	}
	return result, nil
}

func (s *served) ObserveEvents(request EventObservationRequest) ([]ObservedEvent, error) {
	seen := []ObservedEvent{}
	for _, event := range s.published {
		if event.Event == request.Event {
			seen = append(seen, event)
		}
	}
	return seen, nil
}

func (s *served) ConfigureExternalOutcome(ExternalOutcomeControl) error { return ErrUnsupported }

func (s *served) RedeliverEvent(RedeliveryRequest) error { return ErrUnsupported }

func (s *served) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, ErrUnsupported
}

func TestServed(t *testing.T) {
	once.Do(func() { shared, startErr = start() })
	if startErr != nil {
		t.Fatalf("the harness starts: %v", startErr)
	}
	Run(t, func() Target { return shared })
}
