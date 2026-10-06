// Command harness holds an in-memory storage port for the Go behaviour synthesized from the
// subject-state-source model (ess/23, beyond10x/ess#458), and speaks the line protocol
// `tests/subject_state_source.rs` drives it through — the protocol the Rust harness beside it
// speaks.
//
// No command behaviour is written here: every command reaches `behaviour.Generated`, which the
// generator wrote, so the held state a refusal and an event read are the generated reads. This
// file supplies only where a document is stored.
//
// One JSON object per line each way: `{"op":"reset"}`; `{"op":"command","command":…,
// "actor":…,"body":…}`; `{"op":"view","view":…}`. Each command or view goes to the generated HTTP
// surface, started in this process on an ephemeral port and reached over a real socket, and answers
// `{"status":…,"answer":<served body>}`. The arguments are the route table as `name method path`
// triples.
package main

import (
	"bufio"
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"sort"
	"strings"

	"example.invalid/demo/components/docsservice"
	"example.invalid/demo/server"
	"example.invalid/demo/system"
	"example.invalid/demo/types/behaviour"
	"example.invalid/demo/types/docs"
)

// shared is one scenario's rows.
type shared struct {
	docs map[string]docs.DocSnapshot
}

func fresh() *shared {
	return &shared{docs: map[string]docs.DocSnapshot{}}
}

// docStore is where a document is stored.
type docStore struct{ state **shared }

func (s docStore) Get(identity docs.DocId) (docs.DocSnapshot, bool) {
	held, ok := (*s.state).docs[identity.Value().Value()]
	return held, ok
}

func (s docStore) Put(snapshot docs.DocSnapshot) {
	(*s.state).docs[snapshot.Data.DocId.Value().Value()] = snapshot
}

func (s docStore) Delete(identity docs.DocId) {
	delete((*s.state).docs, identity.Value().Value())
}

func (s docStore) List() []docs.DocSnapshot {
	keys := make([]string, 0, len((*s.state).docs))
	for key := range (*s.state).docs {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	rows := make([]docs.DocSnapshot, 0, len(keys))
	for _, key := range keys {
		rows = append(rows, (*s.state).docs[key])
	}
	return rows
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
		_ = server.ServeDocsService(sys, "127.0.0.1:0", authenticate)
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
func send(port int, route [2]string, actor string, body string) (any, error) {
	var payload io.Reader
	if route[0] != "GET" {
		payload = strings.NewReader(body)
	}
	request, err := http.NewRequest(route[0], fmt.Sprintf("http://127.0.0.1:%d%s", port, route[1]), payload)
	if err != nil {
		return nil, err
	}
	request.Header.Set("Content-Type", "application/json")
	if actor != "" {
		request.Header.Set("Authorization", "Actor "+actor)
	}
	response, err := http.DefaultClient.Do(request)
	if err != nil {
		return nil, err
	}
	read, err := io.ReadAll(response.Body)
	_ = response.Body.Close()
	if err != nil {
		return nil, err
	}
	decoder := json.NewDecoder(bytes.NewReader(read))
	decoder.UseNumber()
	var answer any
	if err := decoder.Decode(&answer); err != nil {
		return nil, fmt.Errorf("`%s` is not JSON: %v", read, err)
	}
	return map[string]any{"status": response.StatusCode, "answer": answer}, nil
}

func main() {
	routes := map[string][2]string{}
	for index := 1; index+2 < len(os.Args); index += 3 {
		routes[os.Args[index]] = [2]string{os.Args[index+1], os.Args[index+2]}
	}
	state := fresh()
	generated := behaviour.New(behaviour.Ports{
		DocStorage: docStore{state: &state},
	})
	port, err := started(system.NewSystem(docsservice.New(generated)))
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	out := bufio.NewWriter(os.Stdout)
	lines := bufio.NewScanner(os.Stdin)
	lines.Buffer(make([]byte, 1<<20), 1<<20)
	for lines.Scan() {
		var request map[string]any
		var answer any
		err := json.Unmarshal(lines.Bytes(), &request)
		if err == nil {
			answer, err = handle(request, &state, routes, port)
		}
		if err != nil {
			answer = map[string]any{"failure": err.Error()}
		}
		encoded, _ := json.Marshal(answer)
		_, _ = out.Write(append(encoded, '\n'))
		_ = out.Flush()
	}
}

// handle answers one request line.
func handle(request map[string]any, state **shared, routes map[string][2]string, port int) (any, error) {
	op, _ := request["op"].(string)
	switch op {
	case "reset":
		*state = fresh()
		return map[string]any{"ok": true}, nil
	case "command", "view":
		name, _ := request["command"].(string)
		if op == "view" {
			name, _ = request["view"].(string)
		}
		route, ok := routes[name]
		if !ok {
			return nil, fmt.Errorf("no route for `%s`", name)
		}
		actor, _ := request["actor"].(string)
		body, _ := request["body"].(string)
		return send(port, route, actor, body)
	}
	return nil, fmt.Errorf("no operation `%s`", op)
}
