// An implementation of `restart.yaml` that runs as a child process over a state directory, and the
// restart faults the explorer must catch (beyond10x/ess#297).
//
// BeginScenario makes a fresh state directory and starts the backend: this test binary again,
// running only TestRestartBackend. Restart closes that process's input, waits for it to exit and
// starts a new process over the same directory, so a restart here is a real process restart. The
// mode the factory is given selects what the backend keeps:
//
//	durable          every entry and the identity counter are written to the directory
//	counter-reset    every entry is written and the counter is not: each process counts from zero
//	volatile         nothing is written: a restarted process starts empty
//	no-restart       the target offers no Restart at all
//	restart-refused  Restart answers ErrUnsupported
//	token            durable, and every command answers a consistency token that every later read
//	                 must name: a read after a restart at any other token is refused
//
// `restart-target.mjs` and `restart-backend.mjs` are the same target in TypeScript.

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

// restartFixtureUnsupported is ErrUnsupported with the reason as its whole text, so the reason the
// explorer records reads the same in both lanes.
type restartFixtureUnsupported struct{ reason string }

func (e restartFixtureUnsupported) Error() string { return e.reason }
func (e restartFixtureUnsupported) Unwrap() error { return ErrUnsupported }

// restartFixtureProcesses is the process id of every backend any target started.
var restartFixtureProcesses = map[int]bool{}

type restartFixtureTarget struct {
	mode string
	dir  string
	cmd  *exec.Cmd
	in   io.WriteCloser
	out  *bufio.Reader
	// issued counts the commands answered; `token` mode answers the nth with token `tn`.
	issued int
}

// restartFixtureRestartable is the target with a Restart method; `no-restart` is the one without.
type restartFixtureRestartable struct{ *restartFixtureTarget }

func newRestartFixtureTarget(mode string) Target {
	base := &restartFixtureTarget{mode: mode}
	if mode == "no-restart" {
		return base
	}
	return restartFixtureRestartable{base}
}

func (t restartFixtureRestartable) Restart(ScenarioContext) error {
	if t.mode == "restart-refused" {
		return restartFixtureUnsupported{"this deployment cannot be restarted"}
	}
	if err := t.stop(); err != nil {
		return err
	}
	return t.start()
}

func (t *restartFixtureTarget) start() error {
	cmd := exec.Command(os.Args[0], "-test.run=^TestRestartBackend$")
	cmd.Env = append(os.Environ(), "ESS_RESTART_BACKEND="+t.mode, "ESS_RESTART_DIR="+t.dir)
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
	restartFixtureProcesses[cmd.Process.Pid] = true
	return nil
}

func (t *restartFixtureTarget) stop() error {
	if t.cmd == nil {
		return nil
	}
	_ = t.in.Close()
	err := t.cmd.Wait()
	t.cmd = nil
	return err
}

func (t *restartFixtureTarget) read(into any) error {
	line, err := t.out.ReadBytes('\n')
	if err != nil {
		return fmt.Errorf("the backend answered nothing: %w", err)
	}
	return json.Unmarshal(line, into)
}

func (t *restartFixtureTarget) call(request map[string]any, into any) error {
	line, err := json.Marshal(request)
	if err != nil {
		return err
	}
	if _, err := t.in.Write(append(line, '\n')); err != nil {
		return err
	}
	return t.read(into)
}

func (t *restartFixtureTarget) Identity() (Identity, error) {
	return Identity{Name: "restart-target", Version: "1"}, nil
}

func (t *restartFixtureTarget) BeginScenario(ScenarioContext) error {
	dir, err := os.MkdirTemp("", "ess-restart-")
	if err != nil {
		return err
	}
	t.dir = dir
	return t.start()
}

func (t *restartFixtureTarget) EndScenario(ScenarioContext) error {
	err := t.stop()
	_ = os.RemoveAll(t.dir)
	return err
}

func (t *restartFixtureTarget) ExecuteCommand(request CommandRequest) (CommandResult, error) {
	if request.Command != "restart.ledger.RecordEntry" {
		return CommandResult{}, restartFixtureUnsupported{request.Command + " is not a command of restart.yaml"}
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

func (t *restartFixtureTarget) QueryView(request ViewRequest) (ViewResult, error) {
	if request.View != "restart.ledger.Entries" {
		return ViewResult{}, restartFixtureUnsupported{request.View + " is not a view of restart.yaml"}
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

func (t *restartFixtureTarget) ObserveEvents(EventObservationRequest) ([]ObservedEvent, error) {
	return nil, nil
}

func (t *restartFixtureTarget) ConfigureExternalOutcome(ExternalOutcomeControl) error {
	return restartFixtureUnsupported{"no external outcome"}
}

func (t *restartFixtureTarget) RedeliverEvent(RedeliveryRequest) error {
	return restartFixtureUnsupported{"no redelivery"}
}

func (t *restartFixtureTarget) ObserveInvocations(InvocationObservationRequest) ([]Invocation, error) {
	return nil, restartFixtureUnsupported{"no bindings"}
}

type restartFixtureState struct {
	Next int   `json:"next"`
	Rows []Row `json:"rows"`
}

// TestRestartBackend is the backend process. It serves only when a target started it.
func TestRestartBackend(t *testing.T) {
	mode := os.Getenv("ESS_RESTART_BACKEND")
	if mode == "" {
		return
	}
	path := filepath.Join(os.Getenv("ESS_RESTART_DIR"), "state.json")
	state := restartFixtureState{Rows: []Row{}}
	if mode != "volatile" {
		if bytes, err := os.ReadFile(path); err == nil {
			if err := json.Unmarshal(bytes, &state); err != nil {
				t.Fatal(err)
			}
		}
	}
	if mode == "counter-reset" {
		state.Next = 0
	}
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
			if mode != "volatile" {
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
