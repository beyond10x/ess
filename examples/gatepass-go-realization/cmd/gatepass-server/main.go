// Command gatepass-server is the gatepass system, realized, on the wire.
//
// Thirty lines, and the arrow points the way it always points here: this command links the
// hand-written realization into the generated surface, and the generated surface knows nothing
// about it. server.ServePassService binds, writes the startup record and answers the routes the
// committed OpenAPI document declares; everything it answers *with* comes through the port from
// this module's linker.
//
// # The port comes from the environment, not from an argument
//
// PORT unset or 0 binds an ephemeral port, which is what makes the gate's demonstration
// deterministic: two of these run side by side without agreeing about a number in advance, and each
// says in its startup record which port it took. There is no flag parsing here at all — a
// synthesised surface takes no options, so there is nothing to parse.
package main

import (
	"fmt"
	"net/http"
	"os"
	"strings"

	realization "example.invalid/gatepass-realization"
	"example.invalid/gatepass/server"
)

// authenticate says who a request was sent by, from a demonstration credential:
// `Authorization: Actor <qualified actor name>`.
//
// How a request proves who sent it is the realization's, never the contract's; a deployment
// verifies a session, a token or a certificate here. This one trusts the header, which is exactly
// what a real deployment must not do, and it is here so the demonstration can be driven as any
// declared actor. A name no actor carries is refused by the served surface's grant check.
func authenticate(request *http.Request) *server.Caller {
	name, ok := strings.CutPrefix(request.Header.Get("Authorization"), "Actor ")
	if !ok {
		return nil
	}
	return &server.Caller{Actor: server.Actor(name)}
}

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "0"
	}
	assembled, err := realization.Link()
	if err != nil {
		fmt.Fprintf(os.Stderr, "{\"log\":\"ess/1\",\"event\":\"system.unlinked\",\"reason\":%q}\n", err.Error())
		os.Exit(1)
	}
	if err := server.ServePassService(assembled.System, "127.0.0.1:"+port, authenticate); err != nil {
		fmt.Fprintf(os.Stderr, "{\"log\":\"ess/1\",\"event\":\"system.stopped\",\"reason\":%q}\n", err.Error())
		os.Exit(1)
	}
}
