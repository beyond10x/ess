// Adversary target for `restart.yaml` (beyond10x/ess#297): `restart_target.go` with more modes. The
// modes are the ones `adversary-restart-target.mjs` documents; the backend is this test binary again,
// running only TestAdversaryRestartBackend.

package essconform

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"testing"
)

type advRestartUnsupported struct{ reason string }

func (e advRestartUnsupported) Error() string { return e.reason }
func (e advRestartUnsupported) Unwrap() error { return ErrUnsupported }

var advRestartProcesses = map[int]bool{}

type advRestartTarget struct {
	mode    string
	backend string
	dir     string
	cmd     *exec.Cmd
	in      io.WriteCloser
	out     *bufio.Reader
	// issued counts the commands answered, the last of which names the token every read must name in `token` mode.
	issued int
}

func newAdvRestartTarget(mode string) Target {
	backend := mode
	if mode == "pretend" {
		backend = "counter-reset"
	}
	if mode == "token" {
		backend = "durable"
	}
	return &advRestartTarget{mode: mode, backend: backend}
}

func (t *advRestartTarget) Restart(ScenarioContext) error {
	if t.mode == "pretend" {
		return nil
	}
	if err := t.stop(); err != nil {
		return err
	}
	return t.start()
}

func (t *advRestartTarget) start() error {
	cmd := exec.Command(os.Args[0], "-test.run=^TestAdversaryRestartBackend$")
	cmd.Env = append(os.Environ(), "ESS_RESTART_BACKEND="+t.backend, "ESS_RESTART_DIR="+t.dir)
	cmd.Stderr = os.Stderr
	in, err := cmd.StdinPipe()
	if err != nil {
		return err
	}
	out, err := cmd.StdoutPipe()
	if err != nil {
		return err
	}
	if err := cmd.Start(); err != nil {
		return err
	}
	t.cmd, t.in, t.out = cmd, in, bufio.NewReader(out)
	var ready struct {
		Ready int `json:"ready"`
	}
	if err := t.read(&ready); err != nil {
		return err
	}
	if ready.Ready != cmd.Process.Pid {
		return fmt.Errorf("the backend answered as process %d, started as %d", ready.Ready, cmd.Process.Pid)
	}
	advRestartProcesses[cmd.Process.Pid] = true
	return nil
}

func (t *advRestartTarget) stop() error {
	if t.cmd == nil {
		return nil
	}
	_ = t.in.Close()
	err := t.cmd.Wait()
	t.cmd = nil
	return err
}

func (t *advRestartTarget) read(into any) error {
	line, err := t.out.ReadBytes('\n')
	if err != nil {
		return fmt.Errorf("the backend answered nothing: %w", err)
	}
	return json.Unmarshal(line, into)
}

func (t *advRestartTarget) call(request map[string]any, into any) error {
	line, err := json.Marshal(request)
	if err != nil {
		return err
	}
	if _, err := t.in.Write(append(line, '\n')); err != nil {
		return err
	}
	return t.read(into)
}

func (t *advRestartTarget) Identity() (Identity, error) {
	return Identity{Name: "adversary-restart-target", Version: "1"}, nil
}

func (t *advRestartTarget) BeginScenario(ScenarioContext) error {
	dir, err := os.MkdirTemp("", "ess-adv-restart-")
	if err != nil {
		return err
	}
	t.dir = dir
	return t.start()
}

func (t *advRestartTarget) EndScenario(ScenarioContext) error {
	err := t.stop()
	_ = os.RemoveAll(t.dir)
	return err
}

func (t *advRestartTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	if request.Command != "restart.ledger.RecordEntry" {
		return CommandResult{}, advRestartUnsupported{request.Command + " is not a command of restart.yaml"}
	}
	var answer struct {
		EntryID string `json:"entry_id"`
	}
	if err := t.call(map[string]any{"op": "record", "amount": request.Input["amount"]}, &answer); err != nil {
		return CommandResult{}, err
	}
	t.issued++
	token := ""
	if t.mode == "token" {
		token = fmt.Sprintf("t%d", t.issued)
	}
	return CommandResult{
		Outcome:     "recorded",
		Consistency: token,
		DirectEvents: []ObservedEvent{{
			Event:   "restart.ledger.EntryRecorded",
			Payload: map[string]Node{"entry_id": answer.EntryID, "amount": request.Input["amount"]},
		}},
	}, nil
}

func (t *advRestartTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "restart.ledger.Entries" {
		return ViewResult{}, advRestartUnsupported{request.View + " is not a view of restart.yaml"}
	}
	if t.mode == "token" && request.AtLeast != fmt.Sprintf("t%d", t.issued) {
		return ViewResult{}, fmt.Errorf("read at token %q, the last command answered t%d", request.AtLeast, t.issued)
	}
	var answer struct {
		Rows []Row `json:"rows"`
	}
	if err := t.call(map[string]any{"op": "list"}, &answer); err != nil {
		return ViewResult{}, err
	}
	return ViewResult{Rows: answer.Rows}, nil
}

func (t *advRestartTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *advRestartTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return advRestartUnsupported{"no external outcome"}
}

func (t *advRestartTarget) RedeliverEvent(RedeliveryRequest) error {
	return advRestartUnsupported{"no redelivery"}
}

func (t *advRestartTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, advRestartUnsupported{"no bindings"}
}

type advRestartState struct {
	Next   int   `json:"next"`
	Rows   []Row `json:"rows"`
	Starts int   `json:"starts"`
}

// TestAdversaryRestartBackend is the backend process. It serves only when a target started it.
func TestAdversaryRestartBackend(t *testing.T) {
	mode := os.Getenv("ESS_RESTART_BACKEND")
	if mode == "" {
		return
	}
	path := filepath.Join(os.Getenv("ESS_RESTART_DIR"), "state.json")
	state := advRestartState{Rows: []Row{}}
	if bytes, err := os.ReadFile(path); err == nil {
		if err := json.Unmarshal(bytes, &state); err != nil {
			t.Fatal(err)
		}
	}
	state.Starts++
	if mode == "counter-reset" {
		state.Next = 0
	}
	if mode == "counter-reset-second" && state.Starts == 3 {
		state.Next = 0
	}
	persist := func() {
		bytes, err := json.Marshal(state)
		if err != nil {
			t.Fatal(err)
		}
		if err := os.WriteFile(path+".next", bytes, 0o644); err != nil {
			t.Fatal(err)
		}
		if err := os.Rename(path+".next", path); err != nil {
			t.Fatal(err)
		}
	}
	persist()
	out := json.NewEncoder(os.Stdout)
	if err := out.Encode(map[string]int{"ready": os.Getpid()}); err != nil {
		t.Fatal(err)
	}
	lines := bufio.NewScanner(os.Stdin)
	for lines.Scan() {
		var request struct {
			Op     string `json:"op"`
			Amount Node   `json:"amount"`
		}
		if err := json.Unmarshal(lines.Bytes(), &request); err != nil {
			t.Fatal(err)
		}
		var answer any
		switch request.Op {
		case "record":
			if mode == "constant" {
				state.Next = 0
			}
			state.Next++
			id := fmt.Sprintf("00000000-0000-4000-8000-%012d", state.Next)
			row := Row{"entry_id": id, "amount": request.Amount}
			stored := false
			for index, existing := range state.Rows {
				if existing["entry_id"] == id {
					state.Rows[index] = row
					stored = true
				}
			}
			if !stored {
				state.Rows = append(state.Rows, row)
			}
			persist()
			answer = map[string]string{"entry_id": id}
		case "list":
			answer = map[string]any{"rows": state.Rows}
		default:
			t.Fatalf("unknown request %q", request.Op)
		}
		if err := out.Encode(answer); err != nil {
			t.Fatal(err)
		}
	}
}
