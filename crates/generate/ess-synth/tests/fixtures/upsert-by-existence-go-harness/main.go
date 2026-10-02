// Command harness holds in-memory storage ports for the Go behaviours synthesized from the
// upsert-by-existence model, and speaks the line protocol `tests/upsert_by_existence.rs` drives
// them through — the protocol the Rust harness beside it speaks.
//
// No command behaviour is written here: every command reaches `behaviour.Generated`, which the
// generator wrote, so the lookup that selects a branch by whether the addressed record exists is
// the generated one. This file supplies only where an item and a slot are stored.
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

	"example.invalid/demo/components/itemsservice"
	"example.invalid/demo/server"
	"example.invalid/demo/system"
	"example.invalid/demo/types/behaviour"
	"example.invalid/demo/types/items"
)

// shared is one scenario's rows.
type shared struct {
	items map[string]items.ItemSnapshot
	slots map[string]items.SlotSnapshot
}

func fresh() *shared {
	return &shared{items: map[string]items.ItemSnapshot{}, slots: map[string]items.SlotSnapshot{}}
}

// itemStore is where an item is stored.
type itemStore struct{ state **shared }

func (s itemStore) Get(identity items.ItemId) (items.ItemSnapshot, bool) {
	held, ok := (*s.state).items[identity.Value()]
	return held, ok
}

func (s itemStore) Put(snapshot items.ItemSnapshot) {
	(*s.state).items[snapshot.Data.ItemId.Value()] = snapshot
}

func (s itemStore) Delete(identity items.ItemId) {
	delete((*s.state).items, identity.Value())
}

func (s itemStore) List() []items.ItemSnapshot {
	keys := make([]string, 0, len((*s.state).items))
	for key := range (*s.state).items {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	rows := make([]items.ItemSnapshot, 0, len(keys))
	for _, key := range keys {
		rows = append(rows, (*s.state).items[key])
	}
	return rows
}

// slotStore is where a slot is stored.
type slotStore struct{ state **shared }

func (s slotStore) Get(identity items.ItemId) (items.SlotSnapshot, bool) {
	held, ok := (*s.state).slots[identity.Value()]
	return held, ok
}

func (s slotStore) Put(snapshot items.SlotSnapshot) {
	(*s.state).slots[snapshot.Data.SlotId.Value()] = snapshot
}

func (s slotStore) Delete(identity items.ItemId) {
	delete((*s.state).slots, identity.Value())
}

func (s slotStore) List() []items.SlotSnapshot {
	keys := make([]string, 0, len((*s.state).slots))
	for key := range (*s.state).slots {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	rows := make([]items.SlotSnapshot, 0, len(keys))
	for _, key := range keys {
		rows = append(rows, (*s.state).slots[key])
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
		_ = server.ServeItemsService(sys, "127.0.0.1:0", authenticate)
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
		ItemStorage: itemStore{state: &state},
		SlotStorage: slotStore{state: &state},
	})
	port, err := started(system.NewSystem(itemsservice.New(generated)))
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
