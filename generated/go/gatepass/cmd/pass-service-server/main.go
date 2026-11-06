// generated from gatepass v1
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

// Generated ephemeral server; no production authentication or durable storage.
package main

import (
	"flag"
	"fmt"
	"os"
)

func main() {
	listen := flag.String("listen", "127.0.0.1:8080", "HTTP listen address")
	callers := flag.String("callers", "none", "none or demonstration actor-header")
	staticRoot := flag.String("static", "", "same-origin static directory")
	flag.Parse()
	if *callers != "none" && *callers != "actor-header" {
		fmt.Fprintln(os.Stderr, "unknown caller mode")
		os.Exit(2)
	}
	_ = listen
	_ = staticRoot
	fmt.Fprintln(os.Stderr, "unmet startup obligations:\ncommand behaviour: gatepass.visit.RegisterVisit")
	os.Exit(1)
}
