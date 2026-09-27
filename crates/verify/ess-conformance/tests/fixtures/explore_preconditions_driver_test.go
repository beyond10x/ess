// Runs Explore once per case in ESS_EXPLORE_CASES against `explore_preconditions_target.go` and writes
// what it found, so `tests/explore_preconditions.rs` can compare this lane with the TypeScript lane
// (`explore-preconditions-driver.mjs`).
//
// A case is `{name, mode, options, allowExcluded}`. Each is its own subtest of
// TestExplorePreconditions, and each writes `<name>.json` and `<name>.assert` into ESS_EXPLORE_OUT.

package essconform

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

type explorePreCase struct {
	Name          string         `json:"name"`
	Mode          string         `json:"mode"`
	Options       ExploreOptions `json:"options"`
	AllowExcluded bool           `json:"allowExcluded"`
}

func TestExplorePreconditions(t *testing.T) {
	out := os.Getenv("ESS_EXPLORE_OUT")
	var cases []explorePreCase
	if err := json.Unmarshal([]byte(os.Getenv("ESS_EXPLORE_CASES")), &cases); err != nil {
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
			result, err := Explore(func() Target { return newExplorePreTarget(one.Mode) }, one.Options)
			if err != nil {
				write(".assert", "refused: "+err.Error())
				return
			}
			bytes, err := json.Marshal(result)
			if err != nil {
				t.Fatal(err)
			}
			write(".json", string(bytes))
			if problem := CheckExplored(result, AssertOptions{AllowExcluded: one.AllowExcluded}); problem != nil {
				write(".assert", "failed: "+strings.SplitN(problem.Error(), "\n", 2)[0])
			} else {
				write(".assert", "ok")
			}
		})
	}
}
