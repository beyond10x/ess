package essconform

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"testing"
)

// TestSuiteAdmission runs the runtime's own admission on every document in ESS_ADMISSION_DIR and
// prints one line per document: `ADMITTED <file>` or `REFUSED <file>: <reason>`.
func TestSuiteAdmission(t *testing.T) {
	directory := os.Getenv("ESS_ADMISSION_DIR")
	entries, err := os.ReadDir(directory)
	if err != nil {
		t.Fatalf("reading %s: %v", directory, err)
	}
	names := make([]string, 0, len(entries))
	for _, entry := range entries {
		names = append(names, entry.Name())
	}
	sort.Strings(names)
	for _, name := range names {
		raw, err := os.ReadFile(filepath.Join(directory, name))
		if err != nil {
			t.Fatalf("reading %s: %v", name, err)
		}
		if _, err := admitSuite(string(raw)); err != nil {
			fmt.Printf("REFUSED %s: %v\n", name, err)
		} else {
			fmt.Printf("ADMITTED %s\n", name)
		}
	}
}
