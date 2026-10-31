// Command harness holds in-memory ports for the Go system `tests/integer_spellings.rs`
// synthesizes from its `demo` model, and speaks the line protocol that test drives it through —
// the protocol the Rust harness beside it speaks.
//
// No view query and no command behaviour is written here. Every command and every view reaches
// `behaviour.Generated` through the generated HTTP surface, started in this process on an ephemeral
// port and reached over a real socket, so a request body is read by the generated JSON decoder;
// this file supplies only where a board is stored and which identities are minted.
//
// The protocol is one JSON object per line each way:
//
//   - `{"op":"reset"}` opens an empty scenario and answers `{"status":0,"body":{}}`;
//   - `{"op":"command","command":…,"actor":…,"body":…}` runs one command as the declared actor
//     `actor` (or as no one) with the request body `body`, its input as JSON text, and answers
//     the served status and body;
//   - `{"op":"view","view":…,"query":…}` reads one view and answers the served status and body.
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

	"example.invalid/demo/components/boardservice"
	"example.invalid/demo/server"
	"example.invalid/demo/system"
	"example.invalid/demo/types/behaviour"
	"example.invalid/demo/types/board"
	"example.invalid/demo/types/primitives"
)

// shared is one scenario's state: the boards in the order they were first stored.
type shared struct {
	boards []board.BoardSnapshot
	minted int
}

// ports implements the storage and context ports over the current scenario.
type ports struct {
	state **shared
}

// Get is the board stored under identity.
func (p ports) Get(identity board.BoardId) (board.BoardSnapshot, bool) {
	for _, held := range (*p.state).boards {
		if held.Data.BoardId == identity {
			return held, true
		}
	}
	return board.BoardSnapshot{}, false
}

// Put stores snapshot under its identity, where it was, or last.
func (p ports) Put(snapshot board.BoardSnapshot) {
	for index, held := range (*p.state).boards {
		if held.Data.BoardId == snapshot.Data.BoardId {
			(*p.state).boards[index] = snapshot
			return
		}
	}
	(*p.state).boards = append((*p.state).boards, snapshot)
}

// Delete removes the board stored under identity.
func (p ports) Delete(identity board.BoardId) {
	kept := (*p.state).boards[:0]
	for _, held := range (*p.state).boards {
		if held.Data.BoardId != identity {
			kept = append(kept, held)
		}
	}
	(*p.state).boards = kept
}

// List is every stored board, in the order each was first stored.
func (p ports) List() []board.BoardSnapshot {
	return append([]board.BoardSnapshot(nil), (*p.state).boards...)
}

// GenerateDemoBoardBoardId mints a board identity.
func (p ports) GenerateDemoBoardBoardId() board.BoardId {
	(*p.state).minted++
	return board.NewBoardId(primitives.NewUuid(fmt.Sprintf("00000000-0000-4000-8000-%012d", (*p.state).minted)))
}

// authenticate trusts `Authorization: Actor <name>`, which only this harness sends.
func authenticate(request *http.Request) *server.Caller {
	name, ok := strings.CutPrefix(request.Header.Get("Authorization"), "Actor ")
	if !ok {
		return nil
	}
	return &server.Caller{Actor: server.Actor(name)}
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
		_ = server.ServeBoardService(sys, "127.0.0.1:0", authenticate)
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

// send runs one request against the surface over a real socket, answering its status and body.
func send(port int, route [2]string, actor string, query string, body []byte) (int, []byte, error) {
	var payload io.Reader
	if route[0] != "GET" {
		payload = bytes.NewReader(body)
	}
	target := fmt.Sprintf("http://127.0.0.1:%d%s", port, route[1])
	if query != "" {
		target += "?" + query
	}
	request, err := http.NewRequest(route[0], target, payload)
	if err != nil {
		return 0, nil, err
	}
	request.Header.Set("Content-Type", "application/json")
	if actor != "" {
		request.Header.Set("Authorization", "Actor "+actor)
	}
	response, err := http.DefaultClient.Do(request)
	if err != nil {
		return 0, nil, err
	}
	read, err := io.ReadAll(response.Body)
	_ = response.Body.Close()
	if err != nil {
		return 0, nil, err
	}
	return response.StatusCode, read, nil
}

func main() {
	routes := map[string][2]string{}
	for index := 1; index+2 < len(os.Args); index += 3 {
		routes[os.Args[index]] = [2]string{os.Args[index+1], os.Args[index+2]}
	}
	state := &shared{}
	scenario := ports{state: &state}
	generated := behaviour.New(behaviour.Ports{BoardStorage: scenario, Context: scenario})
	sys := system.NewSystem(boardservice.New(generated))
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
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
		_, _ = out.Write(append(answer, '\n'))
		_ = out.Flush()
	}
}

// handle answers one request line.
func handle(line []byte, scenario ports, routes map[string][2]string, port int) ([]byte, error) {
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
		return []byte(`{"status":0,"body":{}}`), nil
	case "command", "view":
		key := "command"
		if op == "view" {
			key = "view"
		}
		name, _ := request[key].(string)
		route, ok := routes[name]
		if !ok {
			return nil, fmt.Errorf("no route for `%s`", name)
		}
		text, _ := request["body"].(string)
		body := []byte(text)
		query, _ := request["query"].(string)
		actor, _ := request["actor"].(string)
		status, answer, err := send(port, route, actor, query, body)
		if err != nil {
			return nil, err
		}
		return []byte(fmt.Sprintf(`{"status":%d,"body":%s}`, status, bytes.TrimSpace(answer))), nil
	}
	return nil, fmt.Errorf("no operation in `%s`", strings.TrimSpace(string(line)))
}
