// generated from gatepass v1
// model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

// The `pass-service` component of `gatepass` v1, on the wire.
//
// The specification says this component's callers are not deployed with it, so its surface
// exists on a wire. Which wire is derived rather than chosen: the one contract this model
// projects for a command surface is the OpenAPI document, and an OpenAPI document is an
// HTTP contract. The document is beside this file, served verbatim at `/openapi.json`.
package server

import (
	_ "embed"
	"encoding/json"
	"example.invalid/gatepass/system"
	"example.invalid/gatepass/types/visit"
	"fmt"
	"net"
	"net/http"
)

// The contract this surface answers and the prose the same model produced, byte for byte as
// `generated/` commits them. Embedded rather than rebuilt at run time: a server that
// regenerated its own contract could publish one the repository never reviewed.
//
//go:embed pass-service.openapi.json
var openapiPassService string

//go:embed pass-service.docs.md
var docsPassService string

// RoutesPassService is every route this surface answers, in path order.
//
// The same set the OpenAPI document declares, plus the two documents about the surface itself,
// which no specification construct names and nothing can therefore derive. A path absent from
// this table is answered with 404, including one the document declares and this table forgot.
var RoutesPassService = [][2]string{
	{"GET", "/docs"},
	{"GET", "/openapi.json"},
	{"POST", "/visits/commands/admit-visitor"},
	{"POST", "/visits/commands/register-visit"},
	{"POST", "/visits/commands/sign-out-visitor"},
	{"GET", "/visits/views/by-id"},
	{"GET", "/visits/views/expected"},
}

// StartupPassService is what this process says about itself as it starts.
//
// Three lines of JSON on standard output, in this order, every member of them derived from the
// specification — except `runtime`, which is appended below and holds what is true of *this
// process*: the language it was synthesised into, and the address it bound. Everything outside
// `runtime` is the same in every language this plan is emitted into, and `cargo xtask synth
// --check` starts both and compares them.
var StartupPassService = []string{
	"{\"log\":\"ess/1\",\"event\":\"system.starting\",\"system\":\"gatepass\",\"version\":\"v1\",\"model_digest\":\"7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c\",\"contract_digest\":\"2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed\",\"components\":[\"pass-service\"],\"capabilities\":{\"generated\":28,\"obligations\":1,\"refused\":0}",
	"{\"log\":\"ess/1\",\"event\":\"surface.serving\",\"component\":\"pass-service\",\"reached_by\":\"network\",\"transport\":\"http/1.1\",\"routes\":7,\"paths\":[{\"method\":\"GET\",\"path\":\"/docs\",\"serves\":\"documentation\",\"name\":\"docs\"},{\"method\":\"GET\",\"path\":\"/openapi.json\",\"serves\":\"contract\",\"name\":\"openapi\"},{\"method\":\"POST\",\"path\":\"/visits/commands/admit-visitor\",\"serves\":\"command\",\"name\":\"gatepass.visit.AdmitVisitor\"},{\"method\":\"POST\",\"path\":\"/visits/commands/register-visit\",\"serves\":\"command\",\"name\":\"gatepass.visit.RegisterVisit\"},{\"method\":\"POST\",\"path\":\"/visits/commands/sign-out-visitor\",\"serves\":\"command\",\"name\":\"gatepass.visit.SignOutVisitor\"},{\"method\":\"GET\",\"path\":\"/visits/views/by-id\",\"serves\":\"view\",\"name\":\"gatepass.visit.VisitById\"},{\"method\":\"GET\",\"path\":\"/visits/views/expected\",\"serves\":\"view\",\"name\":\"gatepass.visit.ExpectedVisits\"}]",
	"{\"log\":\"ess/1\",\"event\":\"system.ready\",\"system\":\"gatepass\",\"surfaces\":1",
}

// announcePassService writes the startup record, with this process's own facts closing each line.
func announcePassService(address *net.TCPAddr) {
	for _, facts := range StartupPassService {
		runtime, err := json.Marshal(map[string]any{"address": address.String(), "language": "go", "port": address.Port})
		if err != nil {
			continue
		}
		fmt.Printf("%s,\"runtime\":%s}\n", facts, runtime)
	}
}

// ServePassService serves `pass-service` at address, and does not return while it can answer.
//
// address may name port 0, which binds an ephemeral port; the startup record says which one
// was taken, because a caller that cannot learn the port cannot make a request.
//
// It chooses no realization. Every command reaches the port, and a port over unimplemented
// obligations answers the typed refusal this surface reports as 501.
//
// authenticate is the realization's: it says who each request was sent by, or nil, and every
// command checks that caller's grant before it runs. Nothing here reads an actor from the
// request itself.
func ServePassService(system *system.System, address string, authenticate func(*http.Request) *Caller) error {
	listener, err := net.Listen("tcp", address)
	if err != nil {
		return err
	}
	bound, ok := listener.Addr().(*net.TCPAddr)
	if !ok {
		return fmt.Errorf("the listener bound something that is not a TCP address")
	}
	announcePassService(bound)
	return http.Serve(listener, http.HandlerFunc(func(writer http.ResponseWriter, request *http.Request) {
		answer := dispatchPassService(system, authenticate(request), request)
		answer.write(writer)
	}))
}

// dispatchPassService answers one request.
//
// A path this table does not hold is a 404 naming where the whole table is published; a path
// it holds under a different method is a 405 naming the one it answers. Neither is a status
// the contract declares, and neither should be: both are facts about a transport rather than
// about any command.
//
// caller is who the realization authenticated the request as, or nil. Every command checks
// its grant before it runs, and answers the standard refusal when the caller is nil or is
// an actor the specification does not grant the command.
func dispatchPassService(system *system.System, caller *Caller, request *http.Request) response {
	if refused := tooManyHeaders(request); refused != nil {
		return *refused
	}
	body, refused := readBody(request)
	if refused != nil {
		return *refused
	}
	// Held from the port call through Pump and TakePublished: the system is shared by
	// every connection.
	serving.Lock()
	defer serving.Unlock()
	switch request.URL.Path {
	case "/docs":
		if request.Method != "GET" {
			return methodNotAllowed("GET")
		}
		return response{status: 200, contentType: mediaMarkdown, body: docsPassService}
	case "/openapi.json":
		if request.Method != "GET" {
			return methodNotAllowed("GET")
		}
		return response{status: 200, contentType: mediaJSON, body: openapiPassService}
	case "/visits/commands/admit-visitor":
		if request.Method != "POST" {
			return methodNotAllowed("POST")
		}
		if refused := admit(caller, "gatepass.visit.AdmitVisitor"); refused != nil {
			return *refused
		}
		return serveGatepassVisitAdmitVisitor(system, body)
	case "/visits/commands/register-visit":
		if request.Method != "POST" {
			return methodNotAllowed("POST")
		}
		if refused := admit(caller, "gatepass.visit.RegisterVisit"); refused != nil {
			return *refused
		}
		return serveGatepassVisitRegisterVisit(system, body)
	case "/visits/commands/sign-out-visitor":
		if request.Method != "POST" {
			return methodNotAllowed("POST")
		}
		if refused := admit(caller, "gatepass.visit.SignOutVisitor"); refused != nil {
			return *refused
		}
		return serveGatepassVisitSignOutVisitor(system, body)
	case "/visits/views/by-id":
		if request.Method != "GET" {
			return methodNotAllowed("GET")
		}
		return serveGatepassVisitVisitById(system)
	case "/visits/views/expected":
		if request.Method != "GET" {
			return methodNotAllowed("GET")
		}
		return serveGatepassVisitExpectedVisits(system)
	}
	return refusal(404, fmt.Sprintf("`%s` is not a path this surface declares; `GET /openapi.json` publishes every one that is", request.URL.Path))
}

// serveGatepassVisitAdmitVisitor answers `POST` `gatepass.visit.AdmitVisitor`: reads the declared input, runs the port, answers the
// declared outcome.
func serveGatepassVisitAdmitVisitor(system *system.System, body []byte) response {
	value, refused := readJSON(body)
	if refused != nil {
		return *refused
	}
	input, err := decodeCommandGatepassVisitAdmitVisitor(value, "body")
	if err != nil {
		// 400 and not 422: this is a body the schema decides, which is the difference
		// between fixing a value and fixing a serialiser.
		return refusal(400, err.Error())
	}
	outcome, unmet := system.PassService.AdmitVisitor(input)
	if unmet != nil {
		return unfinished(unmet.Error(), false)
	}
	// Deliver what this command published to every binding that reacts to it, then take it
	// off the log: a long-running server keeps nothing from one request to the next. Pump answers
	// only for what this command published, and returns with all of it delivered.
	failure := system.Pump()
	system.TakePublished()
	if failure != nil {
		return unfinished("delivering what the command published: "+failure.Error(), true)
	}
	return answerGatepassVisitAdmitVisitor(outcome)
}

// answerGatepassVisitAdmitVisitor renders one declared outcome of `gatepass.visit.AdmitVisitor` as the contract publishes it: the
// branch that was taken, every event it published in publication order, the declared
// error where there is one, and that error's own payload.
func answerGatepassVisitAdmitVisitor(outcome visit.AdmitVisitorOutcome) response {
	body := map[string]any{}
	switch taken := outcome.(type) {
	case visit.AdmitVisitorOutcomeAdmitted:
		body["outcome"] = "admitted"
		body["published"] = []any{
			map[string]any{"event": "gatepass.visit.VisitorAdmitted", "payload": encodeEventGatepassVisitVisitorAdmitted(taken.VisitorAdmitted)},
		}
		_ = taken
		return rendered(202, body)
	case visit.AdmitVisitorOutcomeWrongState:
		body["outcome"] = "wrong-state"
		body["published"] = []any{}
		body["error"] = "gatepass.visit.VisitStateConflict"
		body["payload"] = encodeErrorGatepassVisitVisitStateConflict(taken.Error)
		return rendered(409, body)
	case visit.AdmitVisitorOutcomeWrongStateUnknownInstance:
		body["outcome"] = "wrong-state"
		body["published"] = []any{}
		body["error"] = "gatepass.visit.VisitStateConflict"
		return rendered(409, body)
	}
	// Go cannot check that a switch over a sealed interface is total, which is this target's
	// standing weakening (see TARGET.md). An outcome no branch above named is a value no
	// generated code can construct, and it is reported rather than dropped.
	return refusal(500, "the port answered an outcome this surface has no branch for")
}

// serveGatepassVisitRegisterVisit answers `POST` `gatepass.visit.RegisterVisit`: reads the declared input, runs the port, answers the
// declared outcome.
func serveGatepassVisitRegisterVisit(system *system.System, body []byte) response {
	value, refused := readJSON(body)
	if refused != nil {
		return *refused
	}
	input, err := decodeCommandGatepassVisitRegisterVisit(value, "body")
	if err != nil {
		// 400 and not 422: this is a body the schema decides, which is the difference
		// between fixing a value and fixing a serialiser.
		return refusal(400, err.Error())
	}
	outcome, unmet := system.PassService.RegisterVisit(input)
	if unmet != nil {
		return unfinished(unmet.Error(), false)
	}
	// Deliver what this command published to every binding that reacts to it, then take it
	// off the log: a long-running server keeps nothing from one request to the next. Pump answers
	// only for what this command published, and returns with all of it delivered.
	failure := system.Pump()
	system.TakePublished()
	if failure != nil {
		return unfinished("delivering what the command published: "+failure.Error(), true)
	}
	return answerGatepassVisitRegisterVisit(outcome)
}

// answerGatepassVisitRegisterVisit renders one declared outcome of `gatepass.visit.RegisterVisit` as the contract publishes it: the
// branch that was taken, every event it published in publication order, the declared
// error where there is one, and that error's own payload.
func answerGatepassVisitRegisterVisit(outcome visit.RegisterVisitOutcome) response {
	body := map[string]any{}
	switch taken := outcome.(type) {
	case visit.RegisterVisitOutcomeRegistered:
		body["outcome"] = "registered"
		body["published"] = []any{
			map[string]any{"event": "gatepass.visit.VisitRegistered", "payload": encodeEventGatepassVisitVisitRegistered(taken.VisitRegistered)},
		}
		_ = taken
		return rendered(202, body)
	case visit.RegisterVisitOutcomeRefused:
		body["outcome"] = "refused"
		body["published"] = []any{}
		body["error"] = "gatepass.visit.InvalidVisitLength"
		body["payload"] = encodeErrorGatepassVisitInvalidVisitLength(taken.Error)
		return rendered(422, body)
	}
	// Go cannot check that a switch over a sealed interface is total, which is this target's
	// standing weakening (see TARGET.md). An outcome no branch above named is a value no
	// generated code can construct, and it is reported rather than dropped.
	return refusal(500, "the port answered an outcome this surface has no branch for")
}

// serveGatepassVisitSignOutVisitor answers `POST` `gatepass.visit.SignOutVisitor`: reads the declared input, runs the port, answers the
// declared outcome.
func serveGatepassVisitSignOutVisitor(system *system.System, body []byte) response {
	value, refused := readJSON(body)
	if refused != nil {
		return *refused
	}
	input, err := decodeCommandGatepassVisitSignOutVisitor(value, "body")
	if err != nil {
		// 400 and not 422: this is a body the schema decides, which is the difference
		// between fixing a value and fixing a serialiser.
		return refusal(400, err.Error())
	}
	outcome, unmet := system.PassService.SignOutVisitor(input)
	if unmet != nil {
		return unfinished(unmet.Error(), false)
	}
	// Deliver what this command published to every binding that reacts to it, then take it
	// off the log: a long-running server keeps nothing from one request to the next. Pump answers
	// only for what this command published, and returns with all of it delivered.
	failure := system.Pump()
	system.TakePublished()
	if failure != nil {
		return unfinished("delivering what the command published: "+failure.Error(), true)
	}
	return answerGatepassVisitSignOutVisitor(outcome)
}

// answerGatepassVisitSignOutVisitor renders one declared outcome of `gatepass.visit.SignOutVisitor` as the contract publishes it: the
// branch that was taken, every event it published in publication order, the declared
// error where there is one, and that error's own payload.
func answerGatepassVisitSignOutVisitor(outcome visit.SignOutVisitorOutcome) response {
	body := map[string]any{}
	switch taken := outcome.(type) {
	case visit.SignOutVisitorOutcomeSignedOut:
		body["outcome"] = "signed-out"
		body["published"] = []any{
			map[string]any{"event": "gatepass.visit.VisitorDeparted", "payload": encodeEventGatepassVisitVisitorDeparted(taken.VisitorDeparted)},
		}
		_ = taken
		return rendered(202, body)
	case visit.SignOutVisitorOutcomeWrongState:
		body["outcome"] = "wrong-state"
		body["published"] = []any{}
		body["error"] = "gatepass.visit.VisitStateConflict"
		body["payload"] = encodeErrorGatepassVisitVisitStateConflict(taken.Error)
		return rendered(409, body)
	case visit.SignOutVisitorOutcomeWrongStateUnknownInstance:
		body["outcome"] = "wrong-state"
		body["published"] = []any{}
		body["error"] = "gatepass.visit.VisitStateConflict"
		return rendered(409, body)
	}
	// Go cannot check that a switch over a sealed interface is total, which is this target's
	// standing weakening (see TARGET.md). An outcome no branch above named is a value no
	// generated code can construct, and it is reported rather than dropped.
	return refusal(500, "the port answered an outcome this surface has no branch for")
}

// serveGatepassVisitVisitById answers `GET` `gatepass.visit.VisitById` at `eventual` consistency: every row the owed projection
// holds.
func serveGatepassVisitVisitById(system *system.System) response {
	rows, unmet := system.PassService.VisitById()
	if unmet != nil {
		return unfinished(unmet.Error(), false)
	}
	encoded := make([]any, 0, len(rows))
	for _, row := range rows {
		encoded = append(encoded, encodeViewGatepassVisitVisitById(row))
	}
	return rendered(200, map[string]any{"rows": encoded})
}

// serveGatepassVisitExpectedVisits answers `GET` `gatepass.visit.ExpectedVisits` at `read_your_writes` consistency: every row the owed projection
// holds.
func serveGatepassVisitExpectedVisits(system *system.System) response {
	rows, unmet := system.PassService.ExpectedVisits()
	if unmet != nil {
		return unfinished(unmet.Error(), false)
	}
	encoded := make([]any, 0, len(rows))
	for _, row := range rows {
		encoded = append(encoded, encodeViewGatepassVisitExpectedVisits(row))
	}
	return rendered(200, map[string]any{"rows": encoded})
}
