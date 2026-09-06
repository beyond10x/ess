---
format: aep.planning-md/1
id: review-result:normalization-binary64-adversary-pass1-public
kind: review-result
status: active
title: Binary64 model and normalization adversary pass 1
owner: aep-drive:adversary
relations:
- reviews: story:model-binary64-fields
revision: 1
---
unit: story:model-binary64-fields adversary pass 1 at bf16e504ccad68b2ee67607ba39606aadf07f627
verdict: nothing found
cases: executed 8→13, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 33 retained files; compiler temporary children under assigned scratch
needs-coordinator: record this report; final whole-workspace and site gates remain coordinator-owned
```console
git --no-pager diff --stat
```
Exit 0, empty stdout: no tracked change. The five added files are untracked tests, so the explicit no-index stat below includes them without staging. Each `git --no-pager diff --no-index --stat -- /dev/null <test>` exits 1 because a new file differs from /dev/null.

```text
 .../edge/ess-cli/tests/binary64_adversary.rs       | 55 ++++++++++++++++++++++
 1 file changed, 55 insertions(+)
 .../schema-contract/tests/binary64_adversary.rs    | 218 +++++++++++++++++++++
 1 file changed, 218 insertions(+)
 .../tests/fixtures/binary64_adversary.rs           | 121 +++++++++++++++++++++
 1 file changed, 121 insertions(+)
 .../tests/fixtures/binary64_adversary_go.go.txt    | 70 ++++++++++++++++++++++
 1 file changed, 70 insertions(+)
 .../tests/fixtures/binary64_adversary_rust.rs.txt  | 38 ++++++++++++++++++++++
 1 file changed, 38 insertions(+)
```

The complete new-test diff follows. Each corresponding no-index full-diff command exits 1; no implementation, existing test, fixture, generated baseline, documentation or planning file was edited.

```diff
diff --git a/crates/edge/ess-cli/tests/binary64_adversary.rs b/crates/edge/ess-cli/tests/binary64_adversary.rs
new file mode 100644
index 0000000..3796f31
--- /dev/null
+++ b/crates/edge/ess-cli/tests/binary64_adversary.rs
@@ -0,0 +1,55 @@
+//! Actual CLI composition of explicit finite floats with retained text and exact siblings.
+
+#[path = "../../../generate/schema-contract/tests/fixtures/binary64_adversary.rs"]
+mod fixture;
+
+#[test]
+fn cli_composition_obeys_the_independently_authored_vectors() {
+    use std::{fs, path::Path, process::Command};
+    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
+        .join(format!("binary64-adversary-cli-{}", std::process::id()));
+    fs::create_dir_all(root.join("model")).unwrap();
+    fs::write(root.join("model/system.yaml"), fixture::SOURCE).unwrap();
+    fs::write(root.join("recipe.json"), fixture::fixture().1.to_string()).unwrap();
+    for (index, case) in fixture::cases().as_array().unwrap().iter().enumerate() {
+        fs::write(root.join("input.json"), case["input"].as_str().unwrap()).unwrap();
+        let output = Command::new(env!("CARGO_BIN_EXE_ess"))
+            .current_dir(&root)
+            .args([
+                "generate",
+                "schema",
+                "normalize-run",
+                "--recipe",
+                "recipe.json",
+                "--model",
+                "model",
+                "--input",
+                "input.json",
+                "--branch",
+                "primary",
+            ])
+            .output()
+            .unwrap();
+        let text = format!(
+            "{}{}",
+            String::from_utf8_lossy(&output.stdout),
+            String::from_utf8_lossy(&output.stderr)
+        );
+        fs::write(
+            root.join(format!("case-{index}.log")),
+            format!("exit {}\n{text}", output.status),
+        )
+        .unwrap();
+        if let Some(rule) = case["error"].as_str() {
+            assert!(!output.status.success(), "{case}: {text}");
+            assert!(text.contains(rule), "{case}: {text}");
+            if let Some(pointer) = case["pointer"].as_str() {
+                assert!(text.contains(pointer), "{case}: {text}");
+            }
+        } else {
+            assert!(output.status.success(), "{case}: {text}");
+            fixture::assert_case(case, Ok(serde_json::from_slice(&output.stdout).unwrap()));
+        }
+    }
+    eprintln!("8 actual CLI nested/raw/exact/default/error-order vectors");
+}
diff --git a/crates/generate/schema-contract/tests/binary64_adversary.rs b/crates/generate/schema-contract/tests/binary64_adversary.rs
new file mode 100644
index 0000000..40f6077
--- /dev/null
+++ b/crates/generate/schema-contract/tests/binary64_adversary.rs
@@ -0,0 +1,218 @@
+//! Adversarial composition at finite-number, raw-token and model-authority boundaries.
+
+#[path = "fixtures/binary64_adversary.rs"]
+mod fixture;
+
+use schema_contract::realize::normalize::Plan;
+use serde_json::json;
+
+#[test]
+fn nested_nullable_floats_raw_capture_and_exact_siblings_survive_two_stages() {
+    let plan = fixture::plan();
+    for case in fixture::cases().as_array().unwrap() {
+        fixture::assert_case(
+            case,
+            plan.run_json("primary", case["input"].as_str().unwrap()),
+        );
+    }
+    eprintln!("8 independently authored nested/raw/exact/default/error-order vectors");
+}
+
+#[test]
+fn optional_aliases_require_complete_policies_and_exact_model_pins() {
+    let (model, recipe) = fixture::fixture();
+    for index in 0..2 {
+        let mut value = recipe.clone();
+        value["binary64_inputs"]["primary"]
+            .as_array_mut()
+            .unwrap()
+            .remove(index);
+        let errors = Plan::check_with_models(
+            serde_json::from_value(value).unwrap(),
+            &[],
+            std::slice::from_ref(&model),
+        )
+        .unwrap_err();
+        assert!(
+            errors
+                .0
+                .iter()
+                .any(|e| e.rule == "model_binary64_policy"
+                    && e.pointer == "/binary64_inputs/primary"),
+            "{errors:?}"
+        );
+    }
+    for part in ["source_digest", "contract_digest", "projection_digest"] {
+        let mut value = recipe.clone();
+        value["branches"]["primary"][0]["input"]["model"][part] = json!("0".repeat(64));
+        let errors = Plan::check_with_models(
+            serde_json::from_value(value).unwrap(),
+            &[],
+            std::slice::from_ref(&model),
+        )
+        .unwrap_err();
+        assert!(
+            errors
+                .0
+                .iter()
+                .any(|e| e.rule == "unknown_model" && e.pointer == "/branches/primary/0/input"),
+            "{errors:?}"
+        );
+    }
+    eprintln!("2 omitted numeric policies and 3 separately corrupted model pins refuse");
+}
+
+#[test]
+fn lazy_defaults_and_later_stages_cannot_erase_float_identity() {
+    let (model, recipe) = fixture::fixture();
+    for expression in [
+        json!({"op":"integer","value":0}),
+        json!({"op":"string","value":"0.0"}),
+    ] {
+        let mut value = recipe.clone();
+        value["branches"]["primary"][0]["value"]["fields"]["ratio"]["fallback"] = expression;
+        let errors = Plan::check_with_models(
+            serde_json::from_value(value).unwrap(),
+            &[],
+            std::slice::from_ref(&model),
+        )
+        .unwrap_err();
+        assert!(
+            errors.0.iter().any(|e| e.rule == "output_type"),
+            "{errors:?}"
+        );
+    }
+    let mut value = recipe;
+    value["branches"]["primary"][1]["requires"][0]["right"] =
+        json!({"op":"read","scope":"input","path":["count"]});
+    let errors =
+        Plan::check_with_models(serde_json::from_value(value).unwrap(), &[], &[model]).unwrap_err();
+    assert!(
+        errors.0.iter().any(|e| e.rule == "equality_type"),
+        "{errors:?}"
+    );
+    eprintln!("2 wrong-kind lazy defaults and later-stage mixed numeric equality refuse");
+}
+
+fn write_target(
+    name: &str,
+    files: std::collections::BTreeMap<String, String>,
+) -> std::path::PathBuf {
+    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
+        .join(format!("binary64-adversary-{name}-{}", std::process::id()));
+    for (path, source) in files {
+        let path = root.join(path);
+        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
+        std::fs::write(path, source).unwrap();
+    }
+    root
+}
+
+#[test]
+fn native_rust_composition_preserves_identity_in_both_number_feature_modes() {
+    use std::{fs, path::Path, process::Command};
+    let root = write_target(
+        "rust",
+        fixture::plan().rust("normalization_adapter").unwrap().files,
+    );
+    fs::create_dir_all(root.join("tests")).unwrap();
+    fs::write(root.join("tests/cases.json"), fixture::cases().to_string()).unwrap();
+    fs::write(
+        root.join("tests/adversary.rs"),
+        include_str!("fixtures/binary64_adversary_rust.rs.txt"),
+    )
+    .unwrap();
+    for features in [None, Some("serde_json/arbitrary_precision")] {
+        let mut command = Command::new(env!("CARGO"));
+        command
+            .args(["test", "--offline", "--quiet", "--manifest-path"])
+            .arg(root.join("Cargo.toml"))
+            .env(
+                "CARGO_TARGET_DIR",
+                Path::new(env!("CARGO_TARGET_TMPDIR")).join("normalization-rust-target"),
+            );
+        if let Some(features) = features {
+            command.args(["--features", features]);
+        }
+        command.args(["--", "--nocapture"]);
+        let output = command.output().unwrap();
+        let text = format!(
+            "{}\n{}",
+            String::from_utf8_lossy(&output.stdout),
+            String::from_utf8_lossy(&output.stderr)
+        );
+        fs::write(
+            root.join(if features.is_some() {
+                "arbitrary.log"
+            } else {
+                "default.log"
+            }),
+            &text,
+        )
+        .unwrap();
+        eprintln!(
+            "native Rust {features:?}, actual exit {}:\n{text}",
+            output.status
+        );
+        assert!(output.status.success());
+    }
+}
+
+#[cfg(feature = "go-typecheck")]
+#[test]
+fn native_go_composition_preserves_identity_and_atomic_refusals() {
+    use std::{fs, path::Path, process::Command};
+    let compiler = std::path::PathBuf::from(std::env::var_os("ESS_GO_COMPILER").unwrap());
+    assert!(compiler.is_absolute() && compiler.is_file());
+    let version = Command::new(&compiler)
+        .arg("version")
+        .env("GOTOOLCHAIN", "local")
+        .output()
+        .unwrap();
+    assert!(version.status.success());
+    assert_eq!(
+        String::from_utf8_lossy(&version.stdout)
+            .split_whitespace()
+            .nth(2)
+            .and_then(|text| text.split('-').next()),
+        Some("go1.26.5")
+    );
+    let root = write_target(
+        "go",
+        fixture::plan()
+            .go(
+                "normalization_adapter",
+                "example.invalid/binary64-adversary",
+            )
+            .unwrap()
+            .files,
+    );
+    fs::write(root.join("cases.json"), fixture::cases().to_string()).unwrap();
+    fs::write(
+        root.join("adversary_test.go"),
+        include_str!("fixtures/binary64_adversary_go.go.txt"),
+    )
+    .unwrap();
+    let output = Command::new(compiler)
+        .args(["test", "-v", "-count=1", "-race", "-mod=readonly", "./..."])
+        .current_dir(&root)
+        .env("GOPROXY", "off")
+        .env("GOSUMDB", "off")
+        .env("GOTOOLCHAIN", "local")
+        .env("GOFLAGS", "")
+        .env(
+            "GOCACHE",
+            Path::new(env!("CARGO_TARGET_TMPDIR")).join("binary64-go-cache"),
+        )
+        .env("GOMAXPROCS", "4")
+        .output()
+        .unwrap();
+    let text = format!(
+        "{}\n{}",
+        String::from_utf8_lossy(&output.stdout),
+        String::from_utf8_lossy(&output.stderr)
+    );
+    fs::write(root.join("native-go.log"), &text).unwrap();
+    eprintln!("native Go actual exit {}:\n{text}", output.status);
+    assert!(output.status.success());
+}
diff --git a/crates/generate/schema-contract/tests/fixtures/binary64_adversary.rs b/crates/generate/schema-contract/tests/fixtures/binary64_adversary.rs
new file mode 100644
index 0000000..2d4dc77
--- /dev/null
+++ b/crates/generate/schema-contract/tests/fixtures/binary64_adversary.rs
@@ -0,0 +1,121 @@
+#![allow(dead_code)]
+
+use ess_compiler::{resolve::compile, source::SourceMap};
+use ess_domain::{
+    spec::{RawSpecFile, Specification},
+    system::Source,
+};
+use ess_gen::schema::ModelTypes;
+use schema_contract::realize::normalize::{Plan, Root};
+use serde_json::{json, Value};
+
+pub const SOURCE: &str = r"format: ess/2
+system: probe
+version: v1
+domains: [probe.data]
+domain: probe.data
+types:
+  - {name: probe.data.Maybe, kind: newtype, of: 'Optional<Binary64>'}
+  - name: probe.data.Input
+    kind: struct
+    fields:
+      - {name: values, wire: 'a/~', type: 'List<Optional<List<Optional<Binary64>>>>'}
+      - {name: count, type: Integer}
+      - {name: raw, type: Bytes}
+      - {name: ratio, type: 'Optional<probe.data.Maybe>'}
+  - name: probe.data.Output
+    kind: struct
+    fields:
+      - {name: values, wire: 'a/~', type: 'List<Optional<List<Optional<Binary64>>>>'}
+      - {name: count, type: Integer}
+      - {name: raw, type: Bytes}
+      - {name: ratio, type: Binary64}
+";
+
+pub fn fixture() -> (ModelTypes, Value) {
+    let mut sources = SourceMap::new();
+    sources.insert(Source::DOCUMENT, SOURCE.to_owned());
+    let spec = Specification::assemble([(Source::document(), RawSpecFile::parse(SOURCE).unwrap())])
+        .unwrap();
+    let ir = compile(&spec, &sources).unwrap();
+    let model = ModelTypes::select(
+        &ir,
+        &std::collections::BTreeSet::from([
+            "probe.data.Input".to_owned(),
+            "probe.data.Output".to_owned(),
+        ]),
+    )
+    .unwrap();
+    let root = |name| Root::pin_model(&model, name).unwrap();
+    let read = |name| json!({"op":"read","scope":"input","path":[name]});
+    let literal = |token| json!({"op":"binary64_literal","value":token});
+    let recipe = json!({"format":"ess-normalization/5",
+    "binary64_inputs":{"primary":[[{"kind":"field","name":"a/~"},{"kind":"items"},{"kind":"items"}],[{"kind":"field","name":"ratio"}]]},
+    "raw_json_inputs":{"primary":[[{"kind":"field","name":"raw"}]]},
+    "branches":{"primary":[
+      {"input":root("probe.data.Input"),"output":root("probe.data.Output"),"requires":[],"value":{"op":"record","fields":{
+        "a/~":read("a/~"),"count":read("count"),"raw":read("raw"),
+        "ratio":{"op":"fallback","value":read("ratio"),"fallback":literal("-0.0"),"on_null":true}
+      }}},
+      {"input":root("probe.data.Output"),"output":root("probe.data.Output"),"requires":[{"op":"equal","left":literal("-0"),"right":literal("0.0")}],"value":{"op":"read","scope":"input","path":[]}}
+    ]}});
+    (model, recipe)
+}
+
+pub fn plan() -> Plan {
+    let (model, recipe) = fixture();
+    Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model]).unwrap()
+}
+
+pub fn cases() -> Value {
+    json!([
+      {"input":"{\"a/~\":[null,[-0,null,5e-324,9007199254740995]],\"count\":9007199254740993,\"raw\": {\"x\":1e999, \"x\":-0}}", "expected":"{\"a/~\":[null,[-0.0,null,5e-324,9007199254740996.0]],\"count\":9007199254740993,\"raw\":\"eyJ4IjoxZTk5OSwgIngiOi0wfQ==\",\"ratio\":-0.0}"},
+      {"input":"{\"a/~\":[],\"count\":0,\"raw\":null,\"ratio\":null}","expected":"{\"a/~\":[],\"count\":0,\"raw\":\"bnVsbA==\",\"ratio\":-0.0}"},
+      {"input":"{\"a/~\":[[],[null,-1e-999]],\"count\":0,\"raw\":\"\\u0041\",\"ratio\":0}","expected":"{\"a/~\":[[],[null,-0.0]],\"count\":0,\"raw\":\"Ilx1MDA0MSI=\",\"ratio\":0.0}"},
+      {"input":"{\"a/~\":[[1e999]],\"count\":0,\"raw\":null}","error":"input_number","pointer":"/input/a~1~0/0/0"},
+      {"input":"{\"a/~\":[],\"count\":0.10000000000000001,\"raw\":null}","error":"input_number","pointer":"/input/count"},
+      {"input":"{\"a/~\":[],\"count\":0,\"raw\":null,\"ratio\":\"0\"}","error":"schema_validation"},
+      {"input":"{\"a/~\":[[\"0\"]],\"count\":0,\"raw\":null}","error":"schema_validation"},
+      {"input":"{\"a/~\":[],\"count\":0,\"raw\":null,\"ratio\":0,\"ratio\":1}","error":"input_syntax","pointer":"/input"}
+    ])
+}
+
+pub fn assert_case(case: &Value, result: Result<Value, schema_contract::realize::Refused>) {
+    if let Some(rule) = case["error"].as_str() {
+        let errors = result.unwrap_err();
+        assert_eq!(errors.0[0].rule, rule, "{case}: {errors:?}");
+        if let Some(pointer) = case["pointer"].as_str() {
+            assert_eq!(errors.0[0].pointer, pointer, "{case}");
+        }
+        return;
+    }
+    let actual = result.unwrap_or_else(|e| panic!("{case}: {e:?}"));
+    let expected: Value = serde_json::from_str(case["expected"].as_str().unwrap()).unwrap();
+    assert_value(&actual, &expected);
+}
+
+pub fn assert_value(actual: &Value, expected: &Value) {
+    match (actual, expected) {
+        (Value::Number(a), Value::Number(e)) if e.is_f64() => {
+            assert!(a.is_f64(), "floating identity: {actual}");
+            assert_eq!(
+                a.as_f64().unwrap().to_bits(),
+                e.as_f64().unwrap().to_bits(),
+                "{actual}: {expected}"
+            );
+        }
+        (Value::Array(a), Value::Array(e)) => {
+            assert_eq!(a.len(), e.len());
+            for (a, e) in a.iter().zip(e) {
+                assert_value(a, e);
+            }
+        }
+        (Value::Object(a), Value::Object(e)) => {
+            assert_eq!(a.keys().collect::<Vec<_>>(), e.keys().collect::<Vec<_>>());
+            for (k, e) in e {
+                assert_value(&a[k], e);
+            }
+        }
+        _ => assert_eq!(actual, expected),
+    }
+}
diff --git a/crates/generate/schema-contract/tests/fixtures/binary64_adversary_go.go.txt b/crates/generate/schema-contract/tests/fixtures/binary64_adversary_go.go.txt
new file mode 100644
index 0000000..31a38cb
--- /dev/null
+++ b/crates/generate/schema-contract/tests/fixtures/binary64_adversary_go.go.txt
@@ -0,0 +1,70 @@
+package normalization_adapter
+
+import (
+    "bytes"
+    "encoding/json"
+    "fmt"
+    "math"
+    "os"
+    "reflect"
+    "strconv"
+    "strings"
+    "testing"
+)
+
+func readExact(t *testing.T, data []byte) any {
+    t.Helper()
+    decoder := json.NewDecoder(bytes.NewReader(data))
+    decoder.UseNumber()
+    var value any
+    if err := decoder.Decode(&value); err != nil { t.Fatal(err) }
+    return value
+}
+
+func compare(t *testing.T, actual, expected any) {
+    t.Helper()
+    switch e := expected.(type) {
+    case json.Number:
+        a, ok := actual.(json.Number)
+        if !ok { t.Fatalf("number: %T", actual) }
+        if strings.ContainsAny(string(e), ".eE") {
+            if !strings.ContainsAny(string(a), ".eE") { t.Fatalf("floating identity: %s", a) }
+            av, ae := strconv.ParseFloat(string(a), 64)
+            ev, ee := strconv.ParseFloat(string(e), 64)
+            if ae != nil || ee != nil || math.Float64bits(av) != math.Float64bits(ev) { t.Fatalf("bits: %s versus %s", a, e) }
+        } else if a != e { t.Fatalf("exact integer: %s versus %s", a, e) }
+    case []any:
+        a, ok := actual.([]any)
+        if !ok || len(a) != len(e) { t.Fatal("list shape") }
+        for index, item := range e { compare(t, a[index], item) }
+    case map[string]any:
+        a, ok := actual.(map[string]any)
+        if !ok || len(a) != len(e) { t.Fatal("record shape") }
+        for key, item := range e { v, exists := a[key]; if !exists { t.Fatal(key) }; compare(t, v, item) }
+    default:
+        if !reflect.DeepEqual(actual, expected) { t.Fatalf("%v versus %v", actual, expected) }
+    }
+}
+
+func TestAdversarialComposition(t *testing.T) {
+    normalizer, err := New()
+    if err != nil { t.Fatal(err) }
+    data, err := os.ReadFile("cases.json")
+    if err != nil { t.Fatal(err) }
+    var cases []struct { Input, Expected, Error, Pointer string }
+    if err := json.Unmarshal(data, &cases); err != nil { t.Fatal(err) }
+    for index, item := range cases {
+        t.Run(fmt.Sprint(index), func(t *testing.T) {
+            actual, err := normalizer.Normalize("primary", []byte(item.Input))
+            if item.Error != "" {
+                errors, ok := err.(Refused)
+                if !ok || len(errors) == 0 || errors[0].Rule != item.Error { t.Fatalf("%s: %v", item.Input, err) }
+                if item.Pointer != "" && errors[0].Pointer != item.Pointer { t.Fatalf("%s: %v", item.Pointer, errors) }
+                if actual != nil { t.Fatal("refusal leaked output") }
+            } else {
+                if err != nil { t.Fatal(err) }
+                compare(t, readExact(t, actual), readExact(t, []byte(item.Expected)))
+            }
+        })
+    }
+}
diff --git a/crates/generate/schema-contract/tests/fixtures/binary64_adversary_rust.rs.txt b/crates/generate/schema-contract/tests/fixtures/binary64_adversary_rust.rs.txt
new file mode 100644
index 0000000..d9d1d7b
--- /dev/null
+++ b/crates/generate/schema-contract/tests/fixtures/binary64_adversary_rust.rs.txt
@@ -0,0 +1,38 @@
+use normalization_adapter::Normalizer;
+use serde_json::Value;
+
+fn compare(actual: &Value, expected: &Value) {
+    match (actual, expected) {
+        (Value::Number(a), Value::Number(e)) if e.is_f64() => {
+            assert!(a.is_f64(), "floating identity: {actual}");
+            assert_eq!(a.as_f64().unwrap().to_bits(), e.as_f64().unwrap().to_bits());
+        }
+        (Value::Array(a), Value::Array(e)) => {
+            assert_eq!(a.len(), e.len());
+            for (a, e) in a.iter().zip(e) { compare(a, e); }
+        }
+        (Value::Object(a), Value::Object(e)) => {
+            assert_eq!(a.keys().collect::<Vec<_>>(), e.keys().collect::<Vec<_>>());
+            for (key, e) in e { compare(&a[key], e); }
+        }
+        _ => assert_eq!(actual, expected),
+    }
+}
+
+#[test]
+fn adversarial_composition() {
+    let normalizer = Normalizer::new().unwrap();
+    let cases: Value = serde_json::from_str(include_str!("cases.json")).unwrap();
+    for case in cases.as_array().unwrap() {
+        let result = normalizer.normalize("primary", case["input"].as_str().unwrap());
+        if let Some(rule) = case["error"].as_str() {
+            let errors = result.unwrap_err();
+            assert_eq!(errors.0[0].rule, rule, "{case}: {errors:?}");
+            if let Some(pointer) = case["pointer"].as_str() { assert_eq!(errors.0[0].pointer, pointer); }
+        } else {
+            let expected: Value = serde_json::from_str(case["expected"].as_str().unwrap()).unwrap();
+            compare(&result.unwrap(), &expected);
+        }
+    }
+    eprintln!("8 independently authored nested/raw/exact/default/error-order vectors");
+}
```

Publication normalization: `$WORKTREE` denotes the frozen unit checkout and `$SCRATCH` its assigned adversary-pass1 directory. Personal absolute paths in commands, quotes and inventories have been replaced with those declared aliases; these public quotes are path-normalized, not verbatim. The original merged output bytes remain in the private `$SCRATCH/*.log` files and private-report.md. Test-source diff bytes and conclusions are unchanged. The public body deliberately has no terminal LF.

No product finding was reproduced in this pass.

The header counts the focused schema-contract suite: the implementor's final deciding lane executed the eight existing `normalization_binary64` tests; this pass's related command executed those eight plus five new adversarial tests, 13 total. The separate new CLI case also executed, as did the one legacy-map case. Generated child test counts and repeated isolated runs are excluded from the header. No whole-workspace after count is claimed; the implementor's reported 1,036 tests were not rerun wholesale.

Read scope: the complete 82-path change against 6c78676c35193423fe326b9dde21b8fc21681b8a, the bound `docs/design/model-binary64.md`, changed contracts and tests, normalization checking/execution/source acquisition/native callers, primitive inventories, structural and whole-system refusals, and conformance reader/writer/runner/CLI callers. The eight frozen legacy templates were independently compared with their original base files and matched byte for byte. No base switch or base build occurred.

Cases were written before their first isolated execution. The eight new corpus vectors combine nested nullable list/alias paths, wire `a/~`, negative zero, signed underflow, smallest subnormal, nearest-even rounding, an exact integer above 2^53, raw duplicate keys and huge exponent capture, explicit missing/null default, a two-stage typed pipeline, strict outside-capture admission, wrong kinds and escaped error pointers. Five policy/pin mutations and three lazy-default/equality mutations have separate checker assertions. Source values and expected outputs were authored independently of observed implementation results.

All six added top-level tests are green now. The first Go selection stopped before adapter generation because this agent's new harness compared the full tool-version token with `go1.26.5`; the installed token is `go1.26.5-X:nodwarf5`. This was a harness error, not a product finding or a measured adapter failure. Only that new harness comparison was corrected to qualify the base version before its suffix, matching the existing native runner. The original exit-101 log is retained below; the next isolated run was the first execution of the new Go vectors. The first reference compilation also emitted a missing-crate-doc warning; adding only the new test crate doc removed it. No product assertion or expected vector was relaxed.

Commands share these actual environment overrides, with cwd the frozen worktree:

```text
CARGO_TARGET_DIR=$WORKTREE/target
CARGO_BUILD_JOBS=4
CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
TMPDIR=$SCRATCH
ESS_GO_COMPILER=/usr/bin/go
```

Native Rust's new generated workspace resolves its lockfile offline on first use and uses the existing unit-local native target cache; the root Cargo commands use --offline --locked. Native Go uses offline dependency configuration, -race, -mod=readonly, GOMAXPROCS=4 and the existing unit-local Go cache. Native exact commands are in the added test source above; their child output and exit appear in the containing isolated logs.

First isolated reference composition; one test, eight vectors, exit 0.

Original merged stdout/stderr retained at `$SCRATCH/first-composition.log`.

```console
cargo test --offline --locked -p schema-contract --test binary64_adversary nested_nullable_floats_raw_capture_and_exact_siblings_survive_two_stages -- --exact --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
warning: missing documentation for the crate
  --> crates/generate/schema-contract/tests/binary64_adversary.rs:1:1
   |
 1 | / #[path = "fixtures/binary64_adversary.rs"]
 2 | | mod fixture;
 3 | |
 4 | | use schema_contract::realize::normalize::Plan;
...  |
92 | |     eprintln!("2 wrong-kind lazy defaults and later-stage mixed numeric equality refuse");
93 | | }
   | |_^
   |
   = note: requested on the command line with `-W missing-docs`

warning: `schema-contract` (test "binary64_adversary") generated 1 warning
    Finished `test` profile [unoptimized] target(s) in 2.57s
     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-1aee8de19aefaba0)

running 1 test
8 independently authored nested/raw/exact/default/error-order vectors
test nested_nullable_floats_raw_capture_and_exact_siblings_survive_two_stages ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.03s

```

Actual command exit: 0.

First isolated policy and pin case; one test, five refusal mutations, exit 0.

Original merged stdout/stderr retained at `$SCRATCH/first-authority.log`.

```console
cargo test --offline --locked -p schema-contract --test binary64_adversary optional_aliases_require_complete_policies_and_exact_model_pins -- --exact --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.39s
     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-1aee8de19aefaba0)

running 1 test
2 omitted numeric policies and 3 separately corrupted model pins refuse
test optional_aliases_require_complete_policies_and_exact_model_pins ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s

```

Actual command exit: 0.

First isolated type-identity case; one test, three refusal mutations, exit 0.

Original merged stdout/stderr retained at `$SCRATCH/first-identity.log`.

```console
cargo test --offline --locked -p schema-contract --test binary64_adversary lazy_defaults_and_later_stages_cannot_erase_float_identity -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-1aee8de19aefaba0)

running 1 test
2 wrong-kind lazy defaults and later-stage mixed numeric equality refuse
test lazy_defaults_and_later_stages_cannot_erase_float_identity ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s

```

Actual command exit: 0.

First isolated generated Rust case; eight vectors in each serde_json feature mode, both child exits 0.

Original merged stdout/stderr retained at `$SCRATCH/first-native-rust.log`.

```console
cargo test --offline --locked -p schema-contract --test binary64_adversary native_rust_composition_preserves_identity_in_both_number_feature_modes -- --exact --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.44s
     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-1aee8de19aefaba0)

running 1 test
native Rust None, actual exit exit status: 0:

running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


8 independently authored nested/raw/exact/default/error-order vectors

native Rust Some("serde_json/arbitrary_precision"), actual exit exit status: 0:

running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


8 independently authored nested/raw/exact/default/error-order vectors

test native_rust_composition_preserves_identity_in_both_number_feature_modes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 2.83s

```

Actual command exit: 0.

First Go harness selection; exit 101 before generation or vector execution, as explained above.

Original merged stdout/stderr retained at `$SCRATCH/first-native-go.log`.

```console
cargo test --offline --locked -p schema-contract --test binary64_adversary --features go-typecheck native_go_composition_preserves_identity_and_atomic_refusals -- --exact --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.42s
     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-bdbabb68c4af8905)

running 1 test

thread 'native_go_composition_preserves_identity_and_atomic_refusals' (1092633) panicked at crates/generate/schema-contract/tests/binary64_adversary.rs:173:5:
assertion `left == right` failed
  left: Some("go1.26.5-X:nodwarf5")
 right: Some("go1.26.5")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test native_go_composition_preserves_identity_and_atomic_refusals ... FAILED

failures:

failures:
    native_go_composition_preserves_identity_and_atomic_refusals

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p schema-contract --test binary64_adversary`
```

Actual command exit: 101.

First isolated actual CLI case; one test executes eight process vectors, exit 0.

Original merged stdout/stderr retained at `$SCRATCH/first-cli.log`.

```console
cargo test --offline --locked -p ess-cli --test binary64_adversary cli_composition_obeys_the_independently_authored_vectors -- --exact --nocapture
   Compiling ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.46s
     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-224d248ba4ff2879)

running 1 test
8 actual CLI nested/raw/exact/default/error-order vectors
test cli_composition_obeys_the_independently_authored_vectors ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

```

Actual command exit: 0.

Corrected isolated Go harness and first actual new Go-vector execution; eight subtests, -race, exit 0.

Original merged stdout/stderr retained at `$SCRATCH/corrected-native-go.log`.

```console
cargo test --offline --locked -p schema-contract --test binary64_adversary --features go-typecheck native_go_composition_preserves_identity_and_atomic_refusals -- --exact --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.41s
     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-bdbabb68c4af8905)

running 1 test
native Go actual exit exit status: 0:
=== RUN   TestAdversarialComposition
=== RUN   TestAdversarialComposition/0
=== RUN   TestAdversarialComposition/1
=== RUN   TestAdversarialComposition/2
=== RUN   TestAdversarialComposition/3
=== RUN   TestAdversarialComposition/4
=== RUN   TestAdversarialComposition/5
=== RUN   TestAdversarialComposition/6
=== RUN   TestAdversarialComposition/7
--- PASS: TestAdversarialComposition (0.01s)
    --- PASS: TestAdversarialComposition/0 (0.00s)
    --- PASS: TestAdversarialComposition/1 (0.00s)
    --- PASS: TestAdversarialComposition/2 (0.00s)
    --- PASS: TestAdversarialComposition/3 (0.00s)
    --- PASS: TestAdversarialComposition/4 (0.00s)
    --- PASS: TestAdversarialComposition/5 (0.00s)
    --- PASS: TestAdversarialComposition/6 (0.00s)
    --- PASS: TestAdversarialComposition/7 (0.00s)
PASS
ok  	example.invalid/binary64-adversary	1.024s


test native_go_composition_preserves_identity_and_atomic_refusals ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 1.57s

```

Actual command exit: 0.

Related suite, after all added cases had their isolated execution: five added plus eight existing tests, all pass.

Original merged stdout/stderr retained at `$SCRATCH/related-suite.log`.

```console
cargo test --offline --locked -p schema-contract --features go-typecheck --test normalization_binary64 --test binary64_adversary -- --test-threads=1 --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.56s
     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-bdbabb68c4af8905)

running 5 tests
test lazy_defaults_and_later_stages_cannot_erase_float_identity ... 2 wrong-kind lazy defaults and later-stage mixed numeric equality refuse
ok
test native_go_composition_preserves_identity_and_atomic_refusals ... native Go actual exit exit status: 0:
=== RUN   TestAdversarialComposition
=== RUN   TestAdversarialComposition/0
=== RUN   TestAdversarialComposition/1
=== RUN   TestAdversarialComposition/2
=== RUN   TestAdversarialComposition/3
=== RUN   TestAdversarialComposition/4
=== RUN   TestAdversarialComposition/5
=== RUN   TestAdversarialComposition/6
=== RUN   TestAdversarialComposition/7
--- PASS: TestAdversarialComposition (0.01s)
    --- PASS: TestAdversarialComposition/0 (0.00s)
    --- PASS: TestAdversarialComposition/1 (0.00s)
    --- PASS: TestAdversarialComposition/2 (0.00s)
    --- PASS: TestAdversarialComposition/3 (0.00s)
    --- PASS: TestAdversarialComposition/4 (0.00s)
    --- PASS: TestAdversarialComposition/5 (0.00s)
    --- PASS: TestAdversarialComposition/6 (0.00s)
    --- PASS: TestAdversarialComposition/7 (0.00s)
PASS
ok  	example.invalid/binary64-adversary	1.024s


ok
test native_rust_composition_preserves_identity_in_both_number_feature_modes ... native Rust None, actual exit exit status: 0:

running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


8 independently authored nested/raw/exact/default/error-order vectors

native Rust Some("serde_json/arbitrary_precision"), actual exit exit status: 0:

running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


8 independently authored nested/raw/exact/default/error-order vectors

ok
test nested_nullable_floats_raw_capture_and_exact_siblings_survive_two_stages ... 8 independently authored nested/raw/exact/default/error-order vectors
ok
test optional_aliases_require_complete_policies_and_exact_model_pins ... 2 omitted numeric policies and 3 separately corrupted model pins refuse
ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.21s

     Running tests/normalization_binary64.rs (target/debug/deps/normalization_binary64-88c1cee6f7600b2c)

running 8 tests
test compiler_metadata_and_structural_refusals_cannot_be_minted_by_schema_annotations ... ok
test floating_construction_requires_the_new_recipe_format ... ok
test floating_integral_results_do_not_become_integer_operands ... ok
test inaccessible_map_and_union_input_policies_refuse_before_generation ... ok
test literal_admission_and_numeric_assignment_are_checked_in_lazy_branches ... ok
test modeled_defaults_and_input_tokens_preserve_finite_bits ... reference modeled Binary64: 70 independently specified corpus cases
ok
test modeled_numeric_policy_is_required_even_for_unused_optional_fields ... ok
test new_expressions_refuse_old_formats_without_any_binary64_model_in_the_recipe ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s

```

Actual command exit: 0.

Direct compatibility check of all eight complete legacy generated file maps, one test passes.

Original merged stdout/stderr retained at `$SCRATCH/legacy-maps.log`.

```console
cargo test --offline --locked -p schema-contract --test normalization_legacy_bytes -- --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.48s
     Running tests/normalization_legacy_bytes.rs (target/debug/deps/normalization_legacy_bytes-ba4faeb96959453e)

running 1 test
test complete_legacy_file_maps_are_preserved ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

```

Actual command exit: 0.

Clippy restricted to the added integration-test targets, warnings denied; exit 0.

Original merged stdout/stderr retained at `$SCRATCH/added-test-clippy.log`.

```console
cargo clippy --offline --locked -p schema-contract -p ess-cli --features schema-contract/go-typecheck --test binary64_adversary -- -D warnings
    Checking schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Checking ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 1.06s
```

Actual command exit: 0.

Rustfmt check of the three added Rust test files; exit 0 with empty stdout.

Original merged stdout/stderr retained at `$SCRATCH/fmt-check.log`.

```console
rustfmt --edition 2021 --check crates/edge/ess-cli/tests/binary64_adversary.rs crates/generate/schema-contract/tests/binary64_adversary.rs crates/generate/schema-contract/tests/fixtures/binary64_adversary.rs
```

Actual command exit: 0.

Findings table: none. No introduced, pre-existing or undecided product finding is claimed, and there is no claimed approval or independence.

What the added cases attacked and did not break:

- Compiler-derived nullable/list metadata requires both explicit policies, including the optional alias, before execution.
- Three separately altered model identity digests refuse at the actual selected input root.
- Raw capture preserves duplicate keys, numeric spelling and string escape spelling while sibling exact inputs still reject precision loss.
- Negative zero, signed underflow, subnormals, tie rounding and exact integer siblings survive two stages in reference, actual CLI and generated Rust/Go.
- Integer/string lazy defaults and later-stage mixed Binary64/Integer equality refuse at checking.
- Go runtime refusals expose nil output, and escaped numeric error pointers match the reference.
- All old format 1–4 generated file maps still match their frozen complete snapshots.

These cases reach public `Plan::check_with_models`/`Plan::run_json`, actual `ess generate schema normalize-run`, and emitted Rust `Normalizer::normalize` / Go `Normalizer.Normalize`. They construct valid authored ess/2 model selections with compiler-minted pins. Unsupported structural native codecs, synthesized systems, conformance Binary64 codecs and TypeScript normalization remain the declared separate boundaries; this pass did not claim their implementation or rerun every existing publication route.

Full source-content proof:

- `$SCRATCH/before-source.json`: SHA-256 `be07735eb94b2ecec6d12aed532611620f3a9aa9ca5b7ca421c6c41f46440fed`.
- `$SCRATCH/after-source.json`: SHA-256 `be07735eb94b2ecec6d12aed532611620f3a9aa9ca5b7ca421c6c41f46440fed`.
- `$SCRATCH/added-tests-manifest.json`: SHA-256 `b1b96272e540895a9ab8d72b7cc92793655c39e7cf636302f5be08ed0bbc9c61`.
- `$SCRATCH/added-tests.patch`: SHA-256 `eeedf2248d54a8caa6f4538cabb4a4ce6f0cd5277876ba99eab277d053c98a43`.
- `$SCRATCH/source-content-proof.json`: SHA-256 `497dbe6aa1ad284d4cf7d18300333b37bf4917ebc226cc6020ffb53d76422b5d`.

All 926 original tracked files retain exactly their before content hashes and permission modes; before-source.json and after-source.json are byte identical. Five new test files are separately hashed, and their complete diff is above. Frozen HEAD remains bf16e504ccad68b2ee67607ba39606aadf07f627. Source was reread after the test runs. No source edit or Git mutation occurred while creating the report packet.

All invoked command sessions and native children are terminal. The final unit-process observation found no active Cargo/rustc/Go/native test process in the unit. Unit target uses 3229241344 bytes; available filesystem space is 31333683200 bytes, above the 8589934592-byte reserve. The exclusive root-team Cargo/native slot is released on this report's handoff. No target directory was removed.

Every retained outside write follows; unit-brief.md was read only and is excluded. Compiler-created temporary rustc/go-build children used the assigned scratch directory and were removed by their tools; their transient leaf names were not separately inventoried. Generated adapters, native logs and caches stayed under the worktree target, not outside it.

```text
$SCRATCH/added-test-clippy-command.json
$SCRATCH/added-test-clippy.log
$SCRATCH/added-tests-manifest.json
$SCRATCH/added-tests-stat.txt
$SCRATCH/added-tests.patch
$SCRATCH/after-source.json
$SCRATCH/before-source.json
$SCRATCH/corrected-native-go-command.json
$SCRATCH/corrected-native-go.log
$SCRATCH/final-environment.json
$SCRATCH/first-authority-command.json
$SCRATCH/first-authority.log
$SCRATCH/first-cli-command.json
$SCRATCH/first-cli.log
$SCRATCH/first-composition-command.json
$SCRATCH/first-composition.log
$SCRATCH/first-identity-command.json
$SCRATCH/first-identity.log
$SCRATCH/first-native-go-command.json
$SCRATCH/first-native-go.log
$SCRATCH/first-native-rust-command.json
$SCRATCH/first-native-rust.log
$SCRATCH/fmt-check-command.json
$SCRATCH/fmt-check.log
$SCRATCH/legacy-maps-command.json
$SCRATCH/legacy-maps.log
$SCRATCH/outside-paths.txt
$SCRATCH/private-report.md
$SCRATCH/public-report.md
$SCRATCH/related-suite-command.json
$SCRATCH/related-suite.log
$SCRATCH/report-packet-manifest.json
$SCRATCH/source-content-proof.json
```

```findings
[]
```