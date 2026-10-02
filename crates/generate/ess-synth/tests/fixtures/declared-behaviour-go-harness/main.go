// Command harness holds in-memory ports for the Go behaviours synthesized from
// `tests/fixtures/declared-behaviour/`, and speaks the line protocol `tests/declared_behaviour.rs`
// drives them through — the protocol the Rust harness beside it speaks.
//
// No command behaviour is written here. Every command reaches `behaviour.Generated`, which the
// generator wrote; this file supplies what the specification leaves to the implementor and says
// so: where a ticket is stored, who the caller is, which identities are minted, which external
// branch the provider takes. The one view query is generated too, over `List`. What is written
// here besides is representation: a request's JSON into the generated input types, and the
// generated outcome, events and rows back out as JSON.
//
// The protocol is one JSON object per line each way:
//
//   - `{"op":"reset"}` opens an empty scenario;
//   - `{"op":"command","command":…,"input":…,"caller":…,"actor":…,"body":…,"force":…}` runs one
//     command, `force` naming the external branch the provider takes on this invocation;
//   - `{"op":"view","view":…}` answers the view's rows.
//
// Its first argument chooses the path a command takes. `port` calls the component's port and
// answers `{"answer":…,"log":[…]}`: the outcome and every event the system then pumped and took
// off its log. `served`, followed by the route table as `name method path` triples, starts the
// generated HTTP surface on an ephemeral port in this process, sends the request to it over a real
// socket, and answers `{"status":…,"answer":<body>,"kept":…}`: the served body verbatim, and how
// many events the dispatch left on the system's log and in the component's outbox.
package main

import (
	"bufio"
	"bytes"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"reflect"
	"sort"
	"strings"

	"example.invalid/desk/components/deskservice"
	"example.invalid/desk/server"
	"example.invalid/desk/system"
	"example.invalid/desk/types/behaviour"
	"example.invalid/desk/types/primitives"
	"example.invalid/desk/types/ticket"
)

// shared is one scenario's state, behind every port.
type shared struct {
	tickets map[string]ticket.TicketSnapshot
	minted  int
	caller  *ticket.AgentId
	forced  []string
}

// fresh is an empty scenario.
func fresh() *shared {
	return &shared{tickets: map[string]ticket.TicketSnapshot{}}
}

// ports implements the storage and context ports over the current scenario.
type ports struct {
	state **shared
}

func (p ports) mint() primitives.Uuid {
	(*p.state).minted++
	return primitives.NewUuid(fmt.Sprintf("00000000-0000-4000-8000-%012d", (*p.state).minted))
}

// Get is the ticket stored under identity.
func (p ports) Get(identity ticket.TicketId) (ticket.TicketSnapshot, bool) {
	held, ok := (*p.state).tickets[identity.Value().Value()]
	return held, ok
}

// Put stores snapshot under its identity.
func (p ports) Put(snapshot ticket.TicketSnapshot) {
	(*p.state).tickets[snapshot.Data.TicketId.Value().Value()] = snapshot
}

// Delete removes the ticket stored under identity.
func (p ports) Delete(identity ticket.TicketId) {
	delete((*p.state).tickets, identity.Value().Value())
}

// List is every stored ticket, in the order of its identity's text.
func (p ports) List() []ticket.TicketSnapshot {
	keys := make([]string, 0, len((*p.state).tickets))
	for key := range (*p.state).tickets {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	rows := make([]ticket.TicketSnapshot, 0, len(keys))
	for _, key := range keys {
		rows = append(rows, (*p.state).tickets[key])
	}
	return rows
}

// CallerAgentId is the caller's `agent_id`, where the step names one.
func (p ports) CallerAgentId() (ticket.AgentId, bool) {
	if (*p.state).caller == nil {
		return ticket.AgentId{}, false
	}
	return *(*p.state).caller, true
}

// GenerateDeskTicketEscalationId mints an escalation identity.
func (p ports) GenerateDeskTicketEscalationId() ticket.EscalationId {
	return ticket.NewEscalationId(p.mint())
}

// GenerateDeskTicketTicketId mints a ticket identity.
func (p ports) GenerateDeskTicketTicketId() ticket.TicketId {
	return ticket.NewTicketId(p.mint())
}

// GenerateDeskTicketTicketRef mints a ticket reference.
func (p ports) GenerateDeskTicketTicketRef() ticket.TicketRef {
	return ticket.NewTicketRef(p.mint())
}

// External is true for the one branch the step forced.
func (p ports) External(command string, outcome string) bool {
	forced := (*p.state).forced
	return len(forced) == 2 && forced[0] == command && forced[1] == outcome
}

// assemble is a fresh system over the ports.
func assemble(p ports) *system.System {
	generated := behaviour.New(behaviour.Ports{TicketStorage: p, Context: p})
	return system.NewSystem(deskservice.New(generated))
}

// ---- representation: JSON into the generated input types ----------------------------------------

func text(value any) string {
	held, _ := value.(string)
	return held
}

func integer(value any) int64 {
	number, _ := value.(json.Number)
	held, _ := number.Int64()
	return held
}

func ticketID(value any) ticket.TicketId {
	return ticket.NewTicketId(primitives.NewUuid(text(value)))
}

func priority(value any) ticket.Priority {
	switch text(value) {
	case "Low":
		return ticket.PriorityLow{}
	case "High":
		return ticket.PriorityHigh{}
	}
	return nil
}

// ---- representation: generated values back out as JSON ------------------------------------------

// snake is a Go field name in the specification's spelling: `ReopenCount` is `reopen_count`.
func snake(name string) string {
	var out strings.Builder
	for index, character := range name {
		if character >= 'A' && character <= 'Z' {
			if index > 0 {
				out.WriteByte('_')
			}
			character += 'a' - 'A'
		}
		out.WriteRune(character)
	}
	return out.String()
}

// kebab is an outcome variant's suffix in the specification's spelling: `AlreadyClosed` is
// `already-closed`.
func kebab(name string) string {
	return strings.ReplaceAll(snake(name), "_", "-")
}

// encode is a generated value as JSON: a newtype or primitive as what it wraps, an enum variant as
// its name, an absent optional as null, bytes as base64.
func encode(value reflect.Value) any {
	switch value.Kind() {
	case reflect.Pointer:
		if value.IsNil() {
			return nil
		}
		return encode(value.Elem())
	case reflect.Interface:
		if value.IsNil() {
			return nil
		}
		return strings.TrimPrefix(value.Elem().Type().Name(), value.Type().Name())
	case reflect.Struct:
		if method := value.MethodByName("Value"); method.IsValid() && method.Type().NumIn() == 0 {
			return encode(method.Call(nil)[0])
		}
		out := map[string]any{}
		for index := 0; index < value.NumField(); index++ {
			field := value.Type().Field(index)
			if field.IsExported() {
				out[snake(field.Name)] = encode(value.Field(index))
			}
		}
		return out
	case reflect.Slice:
		if value.Type().Elem().Kind() == reflect.Uint8 {
			return base64.StdEncoding.EncodeToString(value.Bytes())
		}
		out := []any{}
		for index := 0; index < value.Len(); index++ {
			out = append(out, encode(value.Index(index)))
		}
		return out
	case reflect.Map:
		out := map[string]any{}
		for _, key := range value.MapKeys() {
			out[key.String()] = encode(value.MapIndex(key))
		}
		return out
	case reflect.String:
		return value.String()
	case reflect.Int64, reflect.Int:
		return value.Int()
	case reflect.Bool:
		return value.Bool()
	}
	return fmt.Sprintf("unencodable %s", value.Kind())
}

// published is one event as the wire carries it.
func published(event reflect.Value) map[string]any {
	return map[string]any{"event": "desk.ticket." + event.Type().Name(), "payload": encode(event)}
}

// outcome is one generated outcome as the line protocol answers it.
func outcome(command string, taken any) map[string]any {
	value := reflect.ValueOf(taken)
	short := command[strings.LastIndex(command, ".")+1:]
	answer := map[string]any{
		"outcome":   kebab(strings.TrimPrefix(value.Type().Name(), short+"Outcome")),
		"published": []any{},
	}
	events := []any{}
	for index := 0; index < value.NumField(); index++ {
		field := value.Type().Field(index)
		if field.Name == "Error" {
			answer["refusal"] = map[string]any{
				"error":   "desk.ticket." + field.Type.Name(),
				"payload": encode(value.Field(index)),
			}
			continue
		}
		events = append(events, published(value.Field(index)))
	}
	answer["published"] = events
	return answer
}

// command runs one command through the component's port.
func command(sys *system.System, name string, input map[string]any) (any, error) {
	service := sys.DeskService
	var taken any
	var unmet error
	switch name {
	case "desk.ticket.OpenTicket":
		var note *string
		if held, ok := input["note"].(string); ok {
			note = &held
		}
		answer, refusal := service.OpenTicket(ticket.OpenTicket{
			Title:    ticket.NewTitle(text(input["title"])),
			Priority: priority(input["priority"]),
			Estimate: integer(input["estimate"]),
			Note:     note,
		})
		taken, unmet = answer, errorOf(refusal)
	case "desk.ticket.CloseTicket":
		answer, refusal := service.CloseTicket(ticket.CloseTicket{TicketId: ticketID(input["ticket_id"])})
		taken, unmet = answer, errorOf(refusal)
	case "desk.ticket.ReopenTicket":
		answer, refusal := service.ReopenTicket(ticket.ReopenTicket{TicketId: ticketID(input["ticket_id"])})
		taken, unmet = answer, errorOf(refusal)
	case "desk.ticket.Reprioritize":
		answer, refusal := service.Reprioritize(ticket.Reprioritize{
			TicketId: ticketID(input["ticket_id"]),
			Priority: priority(input["priority"]),
		})
		taken, unmet = answer, errorOf(refusal)
	case "desk.ticket.Comment":
		answer, refusal := service.Comment(ticket.Comment{
			TicketId: ticketID(input["ticket_id"]),
			Text:     text(input["text"]),
		})
		taken, unmet = answer, errorOf(refusal)
	case "desk.ticket.DeleteTicket":
		answer, refusal := service.DeleteTicket(ticket.DeleteTicket{TicketId: ticketID(input["ticket_id"])})
		taken, unmet = answer, errorOf(refusal)
	case "desk.ticket.Escalate":
		urgent, _ := input["urgent"].(bool)
		answer, refusal := service.Escalate(ticket.Escalate{Reason: text(input["reason"]), Urgent: urgent})
		taken, unmet = answer, errorOf(refusal)
	default:
		return nil, fmt.Errorf("no command `%s`", name)
	}
	if unmet != nil {
		return map[string]any{"undeclared": unmet.Error()}, nil
	}
	return outcome(name, taken), nil
}

// errorOf is a refusal as an error, nil where there is none.
func errorOf[T error](refusal T) error {
	if reflect.ValueOf(refusal).IsNil() {
		return nil
	}
	return refusal
}

// taken is every event the system's log holds once pumped, taken off it.
func taken(sys *system.System) ([]any, error) {
	if refusal := sys.Pump(); refusal != nil {
		return nil, refusal
	}
	out := []any{}
	for _, event := range sys.TakePublished() {
		held := reflect.ValueOf(event).FieldByName("Event")
		encoded := published(held)
		out = append(out, map[string]any{"name": encoded["event"], "encoded": encoded})
	}
	if len(sys.Published()) != 0 {
		return nil, fmt.Errorf("`TakePublished` left delivered events on the log")
	}
	return out, nil
}

// view reads the one view the fixture declares through the component's port.
func view(sys *system.System, name string) (any, error) {
	if name != "desk.ticket.Tickets" {
		return nil, fmt.Errorf("no view `%s`", name)
	}
	rows, refusal := sys.DeskService.Tickets()
	if refusal != nil {
		return nil, refusal
	}
	out := []any{}
	for _, row := range rows {
		out = append(out, encode(reflect.ValueOf(row)))
	}
	return map[string]any{"rows": out}, nil
}

// ---- the served path ----------------------------------------------------------------------------

// surface is the generated HTTP surface, started in this process.
type surface struct {
	port   int
	routes map[string][2]string
}

// serve starts the surface over sys and learns the port it bound from its startup record.
func serve(sys *system.System, routes []string) (*surface, error) {
	reader, writer, err := os.Pipe()
	if err != nil {
		return nil, err
	}
	protocol := os.Stdout
	os.Stdout = writer
	go func() {
		_ = server.ServeDeskService(sys, "127.0.0.1:0", authenticate)
	}()
	lines := bufio.NewReader(reader)
	port := 0
	for {
		line, err := lines.ReadString('\n')
		if err != nil {
			os.Stdout = protocol
			return nil, err
		}
		var record struct {
			Event   string `json:"event"`
			Runtime struct {
				Port int `json:"port"`
			} `json:"runtime"`
		}
		if err := json.Unmarshal([]byte(line), &record); err != nil {
			os.Stdout = protocol
			return nil, err
		}
		port = record.Runtime.Port
		if record.Event == "system.ready" {
			break
		}
	}
	os.Stdout = protocol
	table := map[string][2]string{}
	for index := 0; index+2 < len(routes); index += 3 {
		table[routes[index]] = [2]string{routes[index+1], routes[index+2]}
	}
	return &surface{port: port, routes: table}, nil
}

// authenticate trusts `Authorization: Actor <name>`, which only this harness sends.
func authenticate(request *http.Request) *server.Caller {
	name, ok := strings.CutPrefix(request.Header.Get("Authorization"), "Actor ")
	if !ok {
		return nil
	}
	return &server.Caller{Actor: server.Actor(name)}
}

// send runs one command or view through the surface over a real socket.
func (s *surface) send(sys *system.System, name string, actor string, body string) (any, error) {
	route, ok := s.routes[name]
	if !ok {
		return nil, fmt.Errorf("no route for `%s`", name)
	}
	var payload io.Reader
	if route[0] != "GET" {
		payload = strings.NewReader(body)
	}
	request, err := http.NewRequest(route[0], fmt.Sprintf("http://127.0.0.1:%d%s", s.port, route[1]), payload)
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
	kept := len(sys.Published()) + len(sys.DeskService.DrainOutbox())
	return map[string]any{"status": response.StatusCode, "answer": answer, "kept": kept}, nil
}

// ---- the loop -----------------------------------------------------------------------------------

func main() {
	arguments := os.Args[1:]
	served := len(arguments) > 0 && arguments[0] == "served"
	state := fresh()
	scenario := ports{state: &state}
	sys := assemble(scenario)
	var surfaceOf *surface
	if served {
		started, err := serve(sys, arguments[1:])
		if err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
		surfaceOf = started
	}
	out := bufio.NewWriter(os.Stdout)
	lines := bufio.NewScanner(os.Stdin)
	lines.Buffer(make([]byte, 1<<20), 1<<20)
	for lines.Scan() {
		decoder := json.NewDecoder(bytes.NewReader(lines.Bytes()))
		decoder.UseNumber()
		var request map[string]any
		var answer any
		err := decoder.Decode(&request)
		if err == nil {
			answer, err = handle(request, scenario, &sys, surfaceOf)
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
func handle(request map[string]any, scenario ports, sys **system.System, served *surface) (any, error) {
	switch text(request["op"]) {
	case "reset":
		*scenario.state = fresh()
		if served == nil {
			*sys = assemble(scenario)
		}
		return map[string]any{"ok": true}, nil
	case "command":
		name := text(request["command"])
		state := *scenario.state
		state.caller = nil
		if caller, ok := request["caller"].(map[string]any); ok {
			if agent, ok := caller["agent_id"].(string); ok {
				held := ticket.NewAgentId(primitives.NewUuid(agent))
				state.caller = &held
			}
		}
		state.forced = nil
		if force, ok := request["force"].(string); ok {
			state.forced = []string{name, force}
		}
		defer func() { state.forced = nil }()
		if served != nil {
			return served.send(*sys, name, text(request["actor"]), text(request["body"]))
		}
		input, _ := request["input"].(map[string]any)
		answer, err := command(*sys, name, input)
		if err != nil {
			return nil, err
		}
		log, err := taken(*sys)
		if err != nil {
			return nil, err
		}
		return map[string]any{"answer": answer, "log": log}, nil
	case "view":
		if served != nil {
			return served.send(*sys, text(request["view"]), "", "")
		}
		return view(*sys, text(request["view"]))
	}
	return nil, fmt.Errorf("no operation in %v", request)
}
