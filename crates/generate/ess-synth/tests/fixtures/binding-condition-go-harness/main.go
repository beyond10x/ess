// Command harness is the context port for the Go behaviours synthesized from
// `ess-conformance/tests/fixtures/binding-condition.yaml` (with one component added) and its
// selecting variant, and the line protocol `tests/binding_condition_runtime.rs` drives the
// generated system through: the same protocol as `binding-condition-harness/main.rs`, answered in
// the same shapes.
//
// No command behaviour and no binding is written here: every command reaches the generated
// `behaviour.Generated`, and every binding is delivered by the generated `system.System`'s Pump,
// condition included. A command's input is read, and an event written, by the generated server's
// own wire codecs, which the test exports beside them in `server/harness.go`; this file supplies
// which external branch the order log takes and renders what the system recorded.
package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"reflect"
	"strings"
	"unicode"

	"example.invalid/demo/components/messagesservice"
	"example.invalid/demo/server"
	"example.invalid/demo/system"
	"example.invalid/demo/types/behaviour"
	"example.invalid/demo/types/messages"
	"example.invalid/demo/types/obligation"
)

// forced is the external branches forced and not yet taken.
type forced struct {
	pending [][2]string
}

func (f *forced) TryExternal(command behaviour.ExternalCommand, outcome string) (bool, *obligation.UnmetObligation) {
	for at, branch := range f.pending {
		if branch[0] == command.Name() && branch[1] == outcome {
			f.pending = append(f.pending[:at], f.pending[at+1:]...)
			return true, nil
		}
	}
	return false, nil
}

func assemble(context *forced) *system.System {
	behaviours := behaviour.NewWithContext(behaviour.Ports{}, context)
	return system.NewSystem(messagesservice.New(behaviours))
}

func text(value map[string]any, name string) string {
	held, _ := value[name].(string)
	return held
}

// fields writes a command input of text members as the wire does: each declared member by its
// name, an absent Optional left out. The inputs bindings fill here hold text only.
func fields(input any) map[string]any {
	out := map[string]any{}
	value := reflect.ValueOf(input)
	for at := 0; at < value.NumField(); at++ {
		member := value.Field(at)
		if member.Kind() == reflect.Pointer {
			if member.IsNil() {
				continue
			}
			member = member.Elem()
		}
		out[snake(value.Type().Field(at).Name)] = member.Interface()
	}
	return out
}

// snake spells a Go field name as the declared member it was generated from.
func snake(name string) string {
	var out strings.Builder
	for at, letter := range name {
		if unicode.IsUpper(letter) {
			if at > 0 {
				out.WriteByte('_')
			}
			out.WriteRune(unicode.ToLower(letter))
		} else {
			out.WriteRune(letter)
		}
	}
	return out.String()
}

func published(event system.SystemEvent) []any {
	return []any{server.HarnessEvent(event)}
}

func command(s *system.System, name string, input any) (map[string]any, error) {
	var outcome map[string]any
	switch name {
	case "demo.messages.ReceiveMessage":
		decoded, err := server.HarnessReceiveMessage(input)
		if err != nil {
			return nil, err
		}
		answer, refused := s.MessagesService.ReceiveMessage(decoded)
		if refused != nil {
			return nil, refused
		}
		received := answer.(messages.ReceiveMessageOutcomeReceived)
		outcome = map[string]any{"outcome": "received", "published": published(system.SystemEventMessageReceived{Event: received.MessageReceived})}
	case "demo.messages.MessageEvent":
		decoded, err := server.HarnessMessageEvent(input)
		if err != nil {
			return nil, err
		}
		answer, refused := s.MessagesService.MessageEvent(decoded)
		if refused != nil {
			return nil, refused
		}
		switch held := answer.(type) {
		case messages.MessageEventOutcomeMessaged:
			outcome = map[string]any{"outcome": "messaged", "published": published(system.SystemEventOrderMessaged{Event: held.OrderMessaged})}
		case messages.MessageEventOutcomeUnavailable:
			outcome = map[string]any{"outcome": "unavailable", "published": []any{}, "refusal": map[string]any{"error": "demo.messages.Unavailable", "payload": map[string]any{}}}
		}
	case "demo.messages.LogMessage":
		decoded, err := server.HarnessLogMessage(input)
		if err != nil {
			return nil, err
		}
		answer, refused := s.MessagesService.LogMessage(decoded)
		if refused != nil {
			return nil, refused
		}
		logged := answer.(messages.LogMessageOutcomeLogged)
		outcome = map[string]any{"outcome": "logged", "published": published(system.SystemEventMessageLogged{Event: logged.MessageLogged})}
	default:
		return nil, fmt.Errorf("no command `%s`", name)
	}
	return map[string]any{"outcome": outcome, "unmet": server.HarnessUnmet(s.Pump())}, nil
}

func observe(s *system.System) map[string]any {
	events := []any{}
	for _, logged := range s.Published() {
		events = append(events, server.HarnessEvent(logged))
	}
	invocations := []any{}
	for _, invocation := range s.Invocations() {
		switch held := invocation.(type) {
		case system.BindingInvocationReceived:
			invocations = append(invocations, map[string]any{"binding": "received", "command": "demo.messages.MessageEvent", "input": fields(held.Input)})
		case system.BindingInvocationLogged:
			invocations = append(invocations, map[string]any{"binding": "logged", "command": "demo.messages.LogMessage", "input": fields(held.Input)})
		}
	}
	return map[string]any{"published": events, "invocations": invocations}
}

func main() {
	context := &forced{}
	s := assemble(context)
	scanner := bufio.NewScanner(os.Stdin)
	scanner.Buffer(make([]byte, 1<<20), 1<<24)
	out := bufio.NewWriter(os.Stdout)
	for scanner.Scan() {
		var request map[string]any
		var answer any
		var failure error
		if err := json.Unmarshal(scanner.Bytes(), &request); err != nil {
			failure = err
		} else {
			switch text(request, "op") {
			case "reset":
				context.pending = nil
				s = assemble(context)
				answer = map[string]any{"ok": true}
			case "force":
				context.pending = append(context.pending, [2]string{text(request, "command"), text(request, "outcome")})
				answer = map[string]any{"ok": true}
			case "command":
				answer, failure = command(s, text(request, "command"), request["input"])
			case "pump":
				answer = map[string]any{"unmet": server.HarnessUnmet(s.Pump())}
			case "redeliver":
				name := text(request, "event")
				var latest system.SystemEvent
				for _, logged := range s.Published() {
					if server.HarnessEvent(logged)["event"] == name {
						latest = logged
					}
				}
				if latest == nil {
					failure = fmt.Errorf("no `%s` on the log to redeliver", name)
				} else {
					answer = map[string]any{"unmet": server.HarnessUnmet(s.Redeliver(latest))}
				}
			case "observe":
				answer = observe(s)
			default:
				failure = fmt.Errorf("no operation `%s`", text(request, "op"))
			}
		}
		if failure != nil {
			answer = map[string]any{"failure": failure.Error()}
		}
		encoded, err := json.Marshal(answer)
		if err != nil {
			panic(err)
		}
		out.Write(encoded)
		out.WriteString("\n")
		out.Flush()
	}
}
