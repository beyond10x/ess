// Command harness holds in-memory ports for the Go system synthesized from the workbench report's
// examples 25 and 26 (`.engineering/repro/workbench-2026-11-09/examples/`), and speaks the line
// protocol `tests/generated_view_queries.rs` drives them through — the protocol the Rust harness
// beside it speaks.
//
// No view query and no command behaviour is written here. Every command and every view reaches
// `behaviour.Generated` through the generated HTTP surface, started in this process on an
// ephemeral port and reached over a real socket, so a view's parameters arrive in the query string
// and are decoded by the generated query decoder; this file supplies only what the specification
// leaves to the implementor: where an item and a parent are stored, and in which order the store
// lists them.
//
// The protocol is one JSON object per line each way:
//
//   - `{"op":"reset"}` opens an empty scenario and answers `{"status":0,"body":{}}`;
//   - `{"op":"command","command":…,"body":…}` runs one command with the request body `body`,
//     its input as JSON text, and answers the served status and body;
//   - `{"op":"view","view":…,"query":…}` reads one view with the query string `query` and answers
//     the served status and body.
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

	"example.invalid/gap/components/gapservice"
	"example.invalid/gap/server"
	"example.invalid/gap/system"
	"example.invalid/gap/types/behaviour"
	"example.invalid/gap/types/demo"
)

// shared is one scenario's state: the items and the parents in the order they were first stored.
type shared struct {
	items   []demo.ItemSnapshot
	parents []demo.ParentSnapshot
}

// ports implements the storage ports over the current scenario.
type ports struct {
	state **shared
}

// itemPorts is the item storage of a scenario.
type itemPorts struct{ ports }

// Get is the item stored under identity.
func (p itemPorts) Get(identity string) (demo.ItemSnapshot, bool) {
	for _, held := range (*p.state).items {
		if held.Data.Id == identity {
			return held, true
		}
	}
	return demo.ItemSnapshot{}, false
}

// Put stores snapshot under its identity, where it was, or last.
func (p itemPorts) Put(snapshot demo.ItemSnapshot) {
	for index, held := range (*p.state).items {
		if held.Data.Id == snapshot.Data.Id {
			(*p.state).items[index] = snapshot
			return
		}
	}
	(*p.state).items = append((*p.state).items, snapshot)
}

// Delete removes the item stored under identity.
func (p itemPorts) Delete(identity string) {
	kept := (*p.state).items[:0]
	for _, held := range (*p.state).items {
		if held.Data.Id != identity {
			kept = append(kept, held)
		}
	}
	(*p.state).items = kept
}

// List is every stored item, in the order each was first stored.
func (p itemPorts) List() []demo.ItemSnapshot {
	return append([]demo.ItemSnapshot(nil), (*p.state).items...)
}

// parentPorts is the parent storage of a scenario.
type parentPorts struct{ ports }

// Get is the parent stored under identity.
func (p parentPorts) Get(identity string) (demo.ParentSnapshot, bool) {
	for _, held := range (*p.state).parents {
		if held.Data.Id == identity {
			return held, true
		}
	}
	return demo.ParentSnapshot{}, false
}

// Put stores snapshot under its identity, where it was, or last.
func (p parentPorts) Put(snapshot demo.ParentSnapshot) {
	for index, held := range (*p.state).parents {
		if held.Data.Id == snapshot.Data.Id {
			(*p.state).parents[index] = snapshot
			return
		}
	}
	(*p.state).parents = append((*p.state).parents, snapshot)
}

// Delete removes the parent stored under identity.
func (p parentPorts) Delete(identity string) {
	kept := (*p.state).parents[:0]
	for _, held := range (*p.state).parents {
		if held.Data.Id != identity {
			kept = append(kept, held)
		}
	}
	(*p.state).parents = kept
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
		_ = server.ServeGapService(sys, "127.0.0.1:0", func(request *http.Request) *server.Caller {
			if request.Header.Get("X-Actor") != "gap.demo.Anyone" {
				return nil
			}
			return &server.Caller{Actor: server.ActorGapDemoAnyone}
		})
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

// send runs one request against the surface over a real socket, as the actor named in `actor`
// (none where it is empty), answering its status and body.
func send(port int, route [2]string, query string, body []byte, actor string) (int, []byte, error) {
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
	request.Header.Set("X-Actor", actor)
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
	generated := behaviour.New(behaviour.Ports{
		ItemStorage:   itemPorts{scenario},
		ParentStorage: parentPorts{scenario},
	})
	sys := system.NewSystem(gapservice.New(generated))
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
		status, answer, err := send(port, route, query, body, actor)
		if err != nil {
			return nil, err
		}
		return []byte(fmt.Sprintf(`{"status":%d,"body":%s}`, status, bytes.TrimSpace(answer))), nil
	}
	return nil, fmt.Errorf("no operation in `%s`", strings.TrimSpace(string(line)))
}
