// Runs Explore once per case in ESS_EXPLORE_CASES and writes what it found, so `tests/explore.rs`
// can compare this lane with the TypeScript lane (`explore-driver.mjs`).
//
// A case is `{name, mutant, grade, options, allowExcluded}`. Each is its own subtest of
// TestExplore, and each writes `<name>.json` (the result) and `<name>.assert` (`ok`, `failed:
// <first line>` or `refused: <message>`) into ESS_EXPLORE_OUT.

package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

type exploreCase struct {
	Name          string         `json:"name"`
	Mutant        string         `json:"mutant"`
	Grade         string         `json:"grade"`
	Options       ExploreOptions `json:"options"`
	AllowExcluded bool           `json:"allowExcluded"`
}

func TestExplore(t *testing.T) {
	out := os.Getenv("ESS_EXPLORE_OUT")
	random := NewMulberry32(1)
	outputs := []string{}
	for index := 0; index < 5; index++ {
		outputs = append(outputs, fmt.Sprint(random.NextUint32()))
	}
	if err := os.WriteFile(filepath.Join(out, "mulberry32"), []byte(strings.Join(outputs, " ")+"\n"), 0o644); err != nil {
		t.Fatal(err)
	}

	var cases []exploreCase
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
			grade := one.Grade
			if grade == "" {
				grade = "low"
			}
			result, err := Explore(func() Target { return newExploreFixtureTarget(one.Mutant, grade) }, one.Options)
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
