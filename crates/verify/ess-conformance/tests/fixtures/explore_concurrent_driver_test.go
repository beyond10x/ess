// Runs ExploreConcurrent once per case in ESS_CONCURRENT_CASES and writes what it found, so
// `crates/edge/ess-cli/tests/explore_concurrent.rs` can compare this lane with the TypeScript lane
// (`explore-concurrent-driver.mjs`).
//
// A case is `{name, target, mutant, pathEnv, options}`. Each is its own subtest of
// TestExploreConcurrent. Each writes its histories into `ESS_CONCURRENT_OUT/<name>/`, and
// `<name>.json` (the result) and `<name>.problem` (what CheckConcurrent says, or `none`), or
// `<name>.refused` (the error), beside them. `pathEnv`, where present,
// is the PATH the case runs under. `ESS_CONCURRENT_SPEC` is the specification `ess` checks against.

package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

type exploreConcurrentCase struct {
	Name    string            `json:"name"`
	Target  string            `json:"target"`
	Mutant  string            `json:"mutant"`
	PathEnv *string           `json:"pathEnv"`
	Options ConcurrentOptions `json:"options"`
}

func TestExploreConcurrent(t *testing.T) {
	out := os.Getenv("ESS_CONCURRENT_OUT")
	draws := NewSplitMix64(1)
	outputs := []string{}
	for index := 0; index < 5; index++ {
		outputs = append(outputs, fmt.Sprint(draws.Next()))
	}
	if err := os.WriteFile(filepath.Join(out, "splitmix64"), []byte(strings.Join(outputs, " ")+"\n"), 0o644); err != nil {
		t.Fatal(err)
	}

	var cases []exploreConcurrentCase
	if err := json.Unmarshal([]byte(os.Getenv("ESS_CONCURRENT_CASES")), &cases); err != nil {
		t.Fatal(err)
	}
	for _, one := range cases {
		one := one
		t.Run(one.Name, func(t *testing.T) {
			write := func(suffix, text string) {
				if err := os.WriteFile(filepath.Join(out, one.Name+suffix), []byte(text+"\n"), 0o644); err != nil {
					t.Fatal(err)
				}
			}
			if one.PathEnv != nil {
				t.Setenv("PATH", *one.PathEnv)
			}
			options := one.Options
			options.Path = os.Getenv("ESS_CONCURRENT_SPEC")
			options.Out = filepath.Join(out, one.Name)
			newTarget := func() Target { return newExploreFixtureTarget(one.Mutant, "low") }
			if one.Target == "billing" {
				newTarget = func() Target { return newExploreBillingTarget(one.Mutant) }
			}
			if one.Target == "retry" {
				newTarget = func() Target { return newExploreRetryTarget(one.Mutant) }
			}
			if one.Target == "pair" {
				newTarget = func() Target { return newExplorePairTarget(one.Mutant) }
			}
			result, err := ExploreConcurrent(newTarget, options)
			if err != nil {
				write(".refused", err.Error())
				return
			}
			bytes, err := json.Marshal(result)
			if err != nil {
				t.Fatal(err)
			}
			write(".json", string(bytes))
			if problem := CheckConcurrent(result); problem != nil {
				write(".problem", problem.Error())
			} else {
				write(".problem", "none")
			}
		})
	}
}
