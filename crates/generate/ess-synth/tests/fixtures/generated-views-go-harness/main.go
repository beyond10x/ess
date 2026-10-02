// Command harness holds in-memory ports for the Go queries synthesized from
// `tests/fixtures/generated-views.yaml`, and speaks the line protocol
// `tests/generated_view_queries.rs` drives them through — the protocol the Rust harness beside it
// speaks.
//
// No view query and no command behaviour is written here. Every command and every view reaches
// `behaviour.Generated`, which the generator wrote, through the generated HTTP surface, started in
// this process on an ephemeral port and reached over a real socket; this file supplies only what
// the specification leaves to the implementor: where a task is stored (and in which order the
// store lists them) and which identities are minted.
//
// The protocol is one JSON object per line each way:
//
//   - `{"op":"reset"}` opens an empty scenario;
//   - `{"op":"command","command":…,"input":…}` runs one command and answers the served outcome,
//     its declared error under `refusal`, or `{"undeclared":…}` where the surface answered 501;
//   - `{"op":"view","view":…}` answers the served `{"rows":[…]}`.
//
// Its arguments are the route table, as `name method path` triples.
package main

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"

	"example.invalid/ledger/components/ledgerservice"
	"example.invalid/ledger/server"
	"example.invalid/ledger/system"
	"example.invalid/ledger/types/behaviour"
	"example.invalid/ledger/types/primitives"
	"example.invalid/ledger/types/work"
)

// shared is one scenario's state: the tasks in the order they were first stored.
type shared struct {
	tasks  []work.TaskSnapshot
	minted int
}

// ports implements the storage and context ports over the current scenario.
type ports struct {
	state **shared
}

// Get is the task stored under identity.
func (p ports) Get(identity work.TaskId) (work.TaskSnapshot, bool) {
	for _, held := range (*p.state).tasks {
		if held.Data.TaskId == identity {
			return held, true
		}
	}
	return work.TaskSnapshot{}, false
}

// Put stores snapshot under its identity, where it was, or last.
func (p ports) Put(snapshot work.TaskSnapshot) {
	for index, held := range (*p.state).tasks {
		if held.Data.TaskId == snapshot.Data.TaskId {
			(*p.state).tasks[index] = snapshot
			return
		}
	}
	(*p.state).tasks = append((*p.state).tasks, snapshot)
}

// Delete removes the task stored under identity.
func (p ports) Delete(identity work.TaskId) {
	kept := (*p.state).tasks[:0]
	for _, held := range (*p.state).tasks {
		if held.Data.TaskId != identity {
			kept = append(kept, held)
		}
	}
	(*p.state).tasks = kept
}

// List is every stored task, in the order each was first stored.
func (p ports) List() []work.TaskSnapshot {
	return append([]work.TaskSnapshot(nil), (*p.state).tasks...)
}

// GenerateLedgerWorkTaskId mints a task identity.
func (p ports) GenerateLedgerWorkTaskId() work.TaskId {
	(*p.state).minted++
	return work.NewTaskId(primitives.NewUuid(fmt.Sprintf("00000000-0000-4000-8000-%012d", (*p.state).minted)))
}

// started is the port the surface bound, read from its startup record.
func started(sys *system.System) (int, error) {
	reader, writer, err := os.Pipe()
	if err != nil {
		return 0, err
	}
	protocol := os.Stdout
	defer func() { os.Stdout = protocol }()
	os.Stdout = writer
	go func() {
		_ = server.ServeLedgerService(sys, "127.0.0.1:0")
	}()
	lines := bufio.NewReader(reader)
	for {
		line, err := lines.ReadString('\n')
		if err != nil {
			return 0, err
		}
		var record struct {
			Event   string `json:"event"`
			Runtime struct {
				Port int `json:"port"`
			} `json:"runtime"`
		}
		if err := json.Unmarshal([]byte(line), &record); err != nil {
			return 0, err
		}
		if record.Event == "system.ready" {
			return record.Runtime.Port, nil
		}
	}
}

// send runs one request against the surface over a real socket.
func send(port int, route [2]string, body []byte) (int, map[string]any, error) {
	var payload io.Reader
	if route[0] != "GET" {
		payload = bytes.NewReader(body)
	}
	request, err := http.NewRequest(route[0], fmt.Sprintf("http://127.0.0.1:%d%s", port, route[1]), payload)
	if err != nil {
		return 0, nil, err
	}
	request.Header.Set("Content-Type", "application/json")
	response, err := http.DefaultClient.Do(request)
	if err != nil {
		return 0, nil, err
	}
	read, err := io.ReadAll(response.Body)
	_ = response.Body.Close()
	if err != nil {
		return 0, nil, err
	}
	decoder := json.NewDecoder(bytes.NewReader(read))
	decoder.UseNumber()
	var answer map[string]any
	if err := decoder.Decode(&answer); err != nil {
		return 0, nil, fmt.Errorf("`%s` is not a JSON object: %v", read, err)
	}
	return response.StatusCode, answer, nil
}

func main() {
	routes := map[string][2]string{}
	for index := 1; index+2 < len(os.Args); index += 3 {
		routes[os.Args[index]] = [2]string{os.Args[index+1], os.Args[index+2]}
	}
	state := &shared{}
	scenario := ports{state: &state}
	generated := behaviour.New(behaviour.Ports{TaskStorage: scenario, Context: scenario})
	sys := system.NewSystem(ledgerservice.New(generated))
	port, err := started(sys)
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	out := bufio.NewWriter(os.Stdout)
	lines := bufio.NewScanner(os.Stdin)
	lines.Buffer(make([]byte, 1<<20), 1<<20)
	for lines.Scan() {
		answer, err := handle(lines.Bytes(), scenario, routes, port)
		if err != nil {
			answer = map[string]any{"failure": err.Error()}
		}
		encoded, _ := json.Marshal(answer)
		_, _ = out.Write(append(encoded, '\n'))
		_ = out.Flush()
	}
}

// handle answers one request line.
func handle(line []byte, scenario ports, routes map[string][2]string, port int) (any, error) {
	decoder := json.NewDecoder(bytes.NewReader(line))
	decoder.UseNumber()
	var request map[string]any
	if err := decoder.Decode(&request); err != nil {
		return nil, err
	}
	op, _ := request["op"].(string)
	switch op {
	case "reset":
		*scenario.state = &shared{}
		return map[string]any{"ok": true}, nil
	case "command":
		name, _ := request["command"].(string)
		route, ok := routes[name]
		if !ok {
			return nil, fmt.Errorf("no route for `%s`", name)
		}
		body, err := json.Marshal(request["input"])
		if err != nil {
			return nil, err
		}
		status, answer, err := send(port, route, body)
		if err != nil {
			return nil, err
		}
		if status == 501 {
			return map[string]any{"undeclared": answer["refused"]}, nil
		}
		if _, ok := answer["outcome"]; !ok {
			return nil, fmt.Errorf("`%s` answered %d %v", name, status, answer)
		}
		read := map[string]any{"outcome": answer["outcome"], "published": answer["published"]}
		if refused, ok := answer["error"]; ok {
			read["refusal"] = map[string]any{"error": refused, "payload": answer["payload"]}
		}
		return read, nil
	case "view":
		name, _ := request["view"].(string)
		route, ok := routes[name]
		if !ok {
			return nil, fmt.Errorf("no route for `%s`", name)
		}
		status, answer, err := send(port, route, nil)
		if err != nil {
			return nil, err
		}
		if status != 200 {
			return nil, fmt.Errorf("`%s` answered %d %v", name, status, answer)
		}
		return answer, nil
	}
	return nil, fmt.Errorf("no operation in `%s`", strings.TrimSpace(string(line)))
}
