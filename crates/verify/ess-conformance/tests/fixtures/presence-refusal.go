package essconform

import (
	"strings"
	"testing"
)

var suiteJSON = `{}`

// TestPresenceSuitesAreRefusedByVersion: a leaf's `presence` key (beyond10x/ess#139) is dropped in
// silence by this runtime's unmarshal, so the only right answer to suite/24 and /25 is to refuse
// the envelope before reading it.
func TestPresenceSuitesAreRefusedByVersion(t *testing.T) {
	for _, version := range []string{"ess-conformance/24", "ess-conformance/25"} {
		document := `{"provenance":{"suite_version":"` + version + `","system":"example",` +
			`"specification_version":"v1","spec_digest":"` + strings.Repeat("a", 64) +
			`","contract_digest":"` + strings.Repeat("b", 64) + `"},"scenarios":{}}`
		_, err := admitSuite(document)
		if err == nil {
			t.Fatalf("%s was admitted", version)
		}
		if !strings.Contains(err.Error(), `unsupported suite version "`+version+`"`) {
			t.Fatalf("%s refused for another reason: %v", version, err)
		}
	}
}
