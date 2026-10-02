// generated from gatepass v1
// model digest 7d021b6ebe1c4715096f165d6564389be0f46311f67d791ed748f627314d611c
// contract digest 2668f3034afb388a33d7add462e15a830b6010fbfe83101f1dd2526fa18d52ed
// do not edit: regenerate with `ess synthesize`

package main

import (
	"flag"
	"fmt"
	"os"
)

func main() {
	listen := flag.String("listen", "127.0.0.1:8080", "listen address")
	callers := flag.String("callers", "none", "none or actor-header (demonstration only)")
	static := flag.String("static", "", "same-origin static directory")
	flag.Parse()
	if flag.NArg() != 0 || (*callers != "none" && *callers != "actor-header") {
		fmt.Fprintln(os.Stderr, "expected --callers none or actor-header and no positional arguments")
		os.Exit(2)
	}
	_ = listen
	_ = static
	fmt.Fprintln(os.Stderr, "generated entry point requires a realization: CommandBehavior `gatepass.visit.RegisterVisit`")
	os.Exit(1)
}
