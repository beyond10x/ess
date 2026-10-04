// Runs Explore once per case in ESS_EXPLORE_CASES against `restart_target.go`, so
// `tests/explore_restart.rs` can compare this lane with the TypeScript lane (`restart-driver.mjs`).
//
// A case is `{name, mode, options, allowExcluded}`. Each writes `<name>.json` (the result),
// `<name>.assert` (`ok`, `failed: <first line>` or `refused: <message>`) and `<name>.processes`
// (how many distinct backend processes its targets started) into ESS_EXPLORE_OUT.

package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

type restartCase struct {
	Name          string         `json:"name"`
	Mode          string         `json:"mode"`
	Options       ExploreOptions `json:"options"`
	AllowExcluded bool           `json:"allowExcluded"`
}

func TestExploreRestart(t *testing.T) {
	out := os.Getenv("ESS_EXPLORE_OUT")
	var cases []restartCase
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
			before := len(restartFixtureProcesses)
			result, err := Explore(func() Target { return newRestartFixtureTarget(one.Mode) }, one.Options)
			write(".processes", fmt.Sprint(len(restartFixtureProcesses)-before))
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
