package essconform
import ("encoding/json";"testing")
func TestStructuredSpecialTextPreservesRustLiteral(t *testing.T) {
    good,err:=parsePredicate(json.RawMessage(`{"to":{"eq":"Ready"}}`))
    if err!=nil || good.right.literal!="Ready" {t.Fatal("ordinary literal control",good,err)}
    // Rust FactValue::parse_literal refuses non-finite numbers and keeps this exact text.
    raw:=json.RawMessage(`{"to":{"eq":"NaN"}}`)
    parsed,err:=parsePredicate(raw)
    if err!=nil {t.Fatal(err)}
    if parsed.right.literal!="NaN" {t.Errorf("structured scalar changed: got %#v (%T), want text NaN",parsed.right.literal,parsed.right.literal)}
    var original any
    if err:=json.Unmarshal(raw,&original);err!=nil {t.Fatal(err)}
    if err:=admitPredicateVersion(original,7);err!=nil {t.Errorf("old-header disagreement: Rust preserves text NaN, Go refuses: %v",err)}
    if got:=parsed.evaluate(factSource{"to":"NaN"});got!=truthTrue {t.Errorf("matching text failed: %v",got)}
}
