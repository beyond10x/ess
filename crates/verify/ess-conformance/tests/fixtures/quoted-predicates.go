package essconform

import (
	"encoding/json"
	"strings"
	"testing"
)

func quotedSuite(t *testing.T, expression any) string {
	t.Helper()
	var document map[string]any
	if err := json.Unmarshal([]byte(suiteJSON), &document); err != nil {
		t.Fatal(err)
	}
	for _, raw := range document["scenarios"].(map[string]any) {
		step := raw.(map[string]any)["steps"].([]any)[0].(map[string]any)
		step["expectation"].(map[string]any)["predicate"] = expression
	}
	encoded, err := json.Marshal(document)
	if err != nil {
		t.Fatal(err)
	}
	return string(encoded)
}

func TestQuotedTrailingTokensRefused(t *testing.T) {
	for _, expression := range []string{
		`to == "" or text == ""`, `to == '' and text == ''`,
		`to == "done" trailing`, `to == "done""next"`,
		`not to == "" or text == ""`, `to == "unterminated`,
		`to == 'unterminated`, `to == "escaped\"`, `to == "slash\\" or text == ""`,
	} {
		t.Run(expression, func(t *testing.T) {
			_, err := admitRunInput(quotedSuite(t, expression))
			if err == nil || !strings.Contains(err.Error(), "structured") {
				t.Errorf("original suite admitted or lacks guidance: %v", err)
			}
			if _, err := parseLeaf(expression); err == nil {
				t.Error("evaluator admitted malformed leaf")
			}
		})
	}
}

func TestQuotedValuesRemainExact(t *testing.T) {
	for _, vector := range []struct{ expression, value string }{
		{`to == ""`, ""}, {`to == 'true'`, "true"}, {`to == "or and not"`, "or and not"},
		{" \tto == '  spaced  ' \n", "  spaced  "},
		{`to == "say \"or\""`, `say \"or\"`}, {`to == 'it\'s valid'`, `it\'s valid`},
		{`to == "path\\"`, `path\\`}, {`to == "雪 and ☃"`, "雪 and ☃"},
		{`to == "a == b or c != d"`, "a == b or c != d"},
	} {
		t.Run(vector.expression, func(t *testing.T) {
			if _, err := admitRunInput(quotedSuite(t, vector.expression)); err != nil {
				t.Fatal(err)
			}
			parsed, err := parseLeaf(vector.expression)
			if err != nil {
				t.Fatal(err)
			}
			if parsed.right.literal != vector.value {
				t.Fatalf("literal=%#v want %#v", parsed.right.literal, vector.value)
			}
			if parsed.evaluate(factSource{"to": vector.value}) != truthTrue {
				t.Fatal("exact value does not match")
			}
		})
	}
}

func TestQuotedStructuredAndPrefixNot(t *testing.T) {
	for _, expression := range []any{
		map[string]any{"any": []any{`to == ""`, map[string]any{"all": []any{`text == ''`, map[string]any{"not": "disabled"}}}}},
		`not to == ""`,
	} {
		if _, err := admitRunInput(quotedSuite(t, expression)); err != nil {
			t.Fatal(err)
		}
		parsed, err := fromNode(expression)
		if err != nil {
			t.Fatal(err)
		}
		if parsed.evaluate(factSource{"to": "filled", "text": "", "disabled": false}) != truthTrue {
			t.Fatalf("valid composition failed: %#v", expression)
		}
	}
}

func TestQuotedComparisonOperatorsInsideLiteralRemainData(t *testing.T) {
	for _, expression := range []string{`to < "a == b"`, `to < "a != b"`, `to < 'a >= b'`} {
		if _, err := admitRunInput(quotedSuite(t, expression)); err != nil {
			t.Fatal(err)
		}
		parsed, err := parseLeaf(expression)
		if err != nil {
			t.Fatal(err)
		}
		if parsed.left.path != "to" || parsed.op != "<" || parsed.evaluate(factSource{"to": "0"}) != truthTrue {
			t.Fatalf("comparison inside literal became an operator: %#v", parsed)
		}
	}
}
