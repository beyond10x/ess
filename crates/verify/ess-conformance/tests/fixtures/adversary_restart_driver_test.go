// Runs Explore once per case in ESS_EXPLORE_CASES against `adversary_restart_target.go`, writing
// `<name>.json`, `<name>.assert` and `<name>.processes` into ESS_EXPLORE_OUT, as
// `restart_driver_test.go` does.

package essconform

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

type advRestartCase struct {
	Name          string         `json:"name"`
	Mode          string         `json:"mode"`
	Options       ExploreOptions `json:"options"`
	AllowExcluded bool           `json:"allowExcluded"`
}

func TestAdversaryExploreRestart(t *testing.T) {
	out := os.Getenv("ESS_EXPLORE_OUT")
	var cases []advRestartCase
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
			before := len(advRestartProcesses)
			result, err := Explore(func() Target { return newAdvRestartTarget(one.Mode) }, one.Options)
			write(".processes", fmt.Sprint(len(advRestartProcesses)-before))
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
