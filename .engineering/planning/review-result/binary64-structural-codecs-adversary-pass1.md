---
format: aep.planning-md/1
id: review-result:binary64-structural-codecs-adversary-pass1
kind: review-result
status: active
title: 'Independent review: structural Binary64 codecs, pass 1'
relations:
- reviews: story:binary64-structural-codecs
revision: 1
---
unit: standalone structural Binary64 codecs, pass 1, frozen 291f229256ce4fa78a17a01b085f56a9e5ab6990
verdict: nothing found
cases: executed 12→16, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 69 retained scratch paths; standard cache roots and tool-managed TMPDIR activity listed below
needs-coordinator: none
git --no-pager diff --stat
```text
(empty; exit 0)
```

The four additions are untracked. The following actual `git diff --no-index --stat -- /dev/null <path>` outputs include them; each exits 1 because it reports a new file:
```text
 .../tests/binary64_structural_adversary.rs         | 221 +++++++++++++++++++++
 1 file changed, 221 insertions(+)
 .../binary64_structural_adversary_rust.txt         | 88 ++++++++++++++++++++++
 1 file changed, 88 insertions(+)
 .../fixtures/binary64_structural_adversary_go.txt  | 135 +++++++++++++++++++++
 1 file changed, 135 insertions(+)
 .../ess-cli/tests/binary64_structural_adversary.rs | 123 +++++++++++++++++++++
 1 file changed, 123 insertions(+)
```

Tracked source audit: all 969 content hashes and file modes match the coordinator's frozen manifest; the only Git status entries are the four authorized new test paths. No existing test, implementation, document, planning artifact or Git state was edited.

This public body applies declared path normalization to quoted commands, outputs and inventories: `$WORKTREE` is the assigned frozen ESS worktree, `$SCRATCH` is this pass's assigned private scratch directory, `$CARGO` is the local Rust toolchain Cargo executable, and `$USER_HOME` is the invoking user's home directory. Normalized quotations are not verbatim machine output. The private companion and the named raw logs retain original paths and exact captured stdout/stderr. Source contents and assertions are unchanged by this presentation normalization.

The complete added test diff follows. It also exists as `added-tests.patch`; its SHA-256 is `1e001360e4aaa457531281f9c046da24d207661bce3888c1d0792220f40af269`.
```diff
diff --git a/crates/generate/schema-contract/tests/binary64_structural_adversary.rs b/crates/generate/schema-contract/tests/binary64_structural_adversary.rs
new file mode 100644
index 0000000..888f59d
--- /dev/null
+++ b/crates/generate/schema-contract/tests/binary64_structural_adversary.rs
@@ -0,0 +1,221 @@
+//! Adversarial source-token paths through compiler-owned, mutually recursive model roots.
+
+#[allow(dead_code)]
+#[path = "fixtures/normalization_model.rs"]
+mod model;
+
+use schema_contract::realize::Plan;
+use serde_json::json;
+use std::path::{Path, PathBuf};
+use std::process::Command;
+
+const SOURCE: &str = r"format: ess/2
+system: probe
+version: v1
+domains: [probe.float]
+domain: probe.float
+types:
+  - {name: probe.float.Scalar, kind: newtype, of: Binary64}
+  - {name: probe.float.Maybe, kind: newtype, of: 'Optional<probe.float.Scalar>'}
+  - name: probe.float.A
+    kind: struct
+    fields:
+      - {name: number, wire: 'a/b~', type: probe.float.Scalar}
+      - {name: next, type: 'Optional<probe.float.B>'}
+  - name: probe.float.B
+    kind: struct
+    fields:
+      - {name: back, type: 'Optional<probe.float.A>'}
+      - {name: numbers, type: 'Map<String, List<Optional<probe.float.Scalar>>>'}
+  - name: probe.float.Choice
+    kind: union
+    tag: value
+    variants:
+      node: probe.float.A
+      numbers: 'Map<String, List<Optional<probe.float.Scalar>>>'
+      text: String
+  - name: probe.float.Envelope
+    kind: struct
+    fields:
+      - {name: choice, type: probe.float.Choice}
+      - {name: nullableBag, type: 'Map<String, Optional<probe.float.Choice>>'}
+      - {name: optionalMaybe, type: 'Optional<probe.float.Maybe>'}
+      - {name: optionalInts, type: 'Optional<Map<Integer, Binary64>>'}
+  - {name: probe.float.Plain, kind: newtype, of: Integer}
+";
+
+fn plan() -> Plan {
+    Plan::from_model(&model::selection(SOURCE, &["probe.float.Envelope"]))
+        .expect("public compiler-owned recursive selection")
+}
+
+fn native_root(language: &str) -> PathBuf {
+    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
+        "binary64-structural-adversary-{language}-{}",
+        std::process::id()
+    ))
+}
+
+fn run(command: &mut Command) {
+    println!("native command: {command:?}");
+    let result = command.output().expect("execute native case");
+    println!(
+        "native stdout:\n{}\nnative stderr:\n{}\nnative exit: {}",
+        String::from_utf8_lossy(&result.stdout),
+        String::from_utf8_lossy(&result.stderr),
+        result.status
+    );
+    assert!(result.status.success(), "native assertion failed");
+}
+
+#[test]
+fn rust_recursive_unions_and_nullable_maps_keep_original_tokens() {
+    let generated = plan().rust("adversary_types").unwrap();
+    let root = native_root("rust");
+    std::fs::create_dir_all(root.join("tests")).unwrap();
+    std::fs::write(root.join("Cargo.toml"), &generated.supporting["Cargo.toml"]).unwrap();
+    std::fs::write(root.join("types.rs"), generated.declarations).unwrap();
+    std::fs::write(
+        root.join("tests/wire.rs"),
+        include_str!("fixtures/binary64_structural_adversary_rust.txt"),
+    )
+    .unwrap();
+    run(Command::new(env!("CARGO"))
+        .args(["generate-lockfile", "--offline"])
+        .current_dir(&root));
+    for case in [
+        "recursive_ref_map_union_paths_keep_bits_and_float_markers",
+        "marker_objects_and_late_invalid_values_cannot_take_a_float_path",
+        "optional_nullable_alias_and_independent_text_alternative_remain_distinct",
+    ] {
+        run(Command::new(env!("CARGO"))
+            .args([
+                "test",
+                "--offline",
+                "--locked",
+                "--test",
+                "wire",
+                case,
+                "--",
+                "--exact",
+                "--nocapture",
+            ])
+            .current_dir(&root));
+    }
+}
+
+#[cfg(feature = "go-typecheck")]
+#[test]
+fn go_recursive_maps_and_failed_decode_preserve_complete_receiver_state() {
+    let generated = plan()
+        .go("adversary_types", "example.invalid/adversary-types")
+        .unwrap();
+    let root = native_root("go");
+    std::fs::create_dir_all(&root).unwrap();
+    std::fs::write(root.join("go.mod"), &generated.supporting["go.mod"]).unwrap();
+    std::fs::write(root.join("types.go"), generated.declarations).unwrap();
+    std::fs::write(
+        root.join("wire_test.go"),
+        include_str!("fixtures/binary64_structural_adversary_go.txt"),
+    )
+    .unwrap();
+    for case in [
+        "^TestRecursiveSourceBits$",
+        "^TestEveryLateFailureLeavesTheWholeReceiverUntouched$",
+        "^TestOptionalNullableAndRetryResetAreExplicit$",
+    ] {
+        run(Command::new(
+            std::env::var_os("ESS_GO_COMPILER")
+                .expect("explicit go-typecheck lane requires ESS_GO_COMPILER"),
+        )
+        .args([
+            "test", "-count=1", "-race", "-p=1", "-v", "-run", case, "./...",
+        ])
+        .current_dir(&root)
+        .env("GOTOOLCHAIN", "local")
+        .env("GOPROXY", "off")
+        .env("GOSUMDB", "off")
+        .env("GOWORK", "off")
+        .env("GOFLAGS", ""));
+    }
+}
+
+#[test]
+fn compiler_owned_helper_names_and_selected_root_reports_remain_conditional() {
+    let selected = model::selection(SOURCE, &["probe.float.Envelope"]);
+    let expected = selected.binary64_locations().clone();
+    assert!(!expected.is_empty());
+    let plan = Plan::from_model(&selected).unwrap();
+    let rust = json!(plan.rust("adversary_types").unwrap().report);
+    let go = json!(
+        plan.go("adversary_types", "example.invalid/adversary-types")
+            .unwrap()
+            .report
+    );
+    let ts = json!(plan.typescript().report);
+    for report in [&rust, &go, &ts] {
+        assert_eq!(report["roots"], json!(["probe.float.Envelope"]));
+        assert_eq!(report["input"]["kind"], "model");
+        assert!(report["obligations"]
+            .as_array()
+            .unwrap()
+            .iter()
+            .any(|e| e["rule"] == "model_map_keys"));
+    }
+    for at in &expected {
+        assert!(ts["obligations"]
+            .as_array()
+            .unwrap()
+            .iter()
+            .any(|e| e["rule"] == "model_binary64" && e["pointer"] == *at));
+        assert!(rust["obligations"]
+            .as_array()
+            .unwrap()
+            .iter()
+            .any(|e| e["rule"] == "rust_binary64_source" && e["pointer"] == *at));
+    }
+    for report in [&rust, &go] {
+        assert!(!report["obligations"]
+            .as_array()
+            .unwrap()
+            .iter()
+            .any(|e| e["rule"] == "model_binary64"));
+        assert!(report["obligations"]
+            .as_array()
+            .unwrap()
+            .iter()
+            .any(|e| e["rule"] == "oneOf"));
+    }
+    let non_binary64 = Plan::from_model(&model::selection(SOURCE, &["probe.float.Plain"])).unwrap();
+    assert!(
+        !non_binary64.rust("plain_types").unwrap().supporting["Cargo.toml"].contains("raw_value")
+    );
+    assert!(!non_binary64
+        .go("plain_types", "example.invalid/plain-types")
+        .unwrap()
+        .declarations
+        .contains("type EssBinary64 struct"));
+
+    for (name, rust_collision, go_collision) in [
+        ("ess.Binary64", true, true),
+        ("ess.Binary64Error", true, false),
+        ("new.ess.Binary64", false, true),
+    ] {
+        let domain = name.rsplit_once('.').unwrap().0;
+        let system = domain.split('.').next().unwrap();
+        let finite = format!("{domain}.Finite");
+        let source = format!("format: ess/2\nsystem: {system}\nversion: v1\ndomains: []\ntypes:\n  - {{name: {name}, kind: newtype, of: String}}\n  - {{name: {finite}, kind: newtype, of: Binary64}}\n");
+        // Public compiler names, never mutations of sealed target metadata.
+        let collision = Plan::from_model(&model::selection(&source, &[name, &finite])).unwrap();
+        let rust = collision.rust("adversary_types");
+        let go = collision.go("adversary_types", "example.invalid/adversary-types");
+        assert_eq!(rust.is_err(), rust_collision, "Rust helper {name}");
+        assert_eq!(go.is_err(), go_collision, "Go helper {name}");
+        if let Err(errors) = rust {
+            assert!(errors.0.iter().any(|e| e.rule == "rust_helper_collision"));
+        }
+        if let Err(errors) = go {
+            assert!(errors.0.iter().any(|e| e.rule == "go_helper_collision"));
+        }
+    }
+}
diff --git a/crates/generate/schema-contract/tests/fixtures/binary64_structural_adversary_rust.txt b/crates/generate/schema-contract/tests/fixtures/binary64_structural_adversary_rust.txt
new file mode 100644
index 0000000..2b1f258
--- /dev/null
+++ b/crates/generate/schema-contract/tests/fixtures/binary64_structural_adversary_rust.txt
@@ -0,0 +1,88 @@
+use adversary_types::*;
+use serde_json::Value;
+
+const INPUT: &str = r#"{"choice":{"value":"node","content":{"a/b~":TOKEN,"next":{"back":{"a/b~":TOKEN},"numbers":{"$serde_json::private::RawValue":[TOKEN,null]}}}},"nullableBag":{"none":null,"some":{"value":"numbers","content":{"__proto__":[TOKEN,null]}}},"optionalMaybe":TOKEN}"#;
+
+fn bits(value: &ProbeFloatEnvelope) -> [u64; 5] {
+    let ProbeFloatChoice::V0(branch) = value.choice.as_ref() else { panic!("node branch") };
+    let a = &branch.content;
+    let EssPresence::Present(b) = &a.next else { panic!("next") };
+    let EssPresence::Present(back) = &b.back else { panic!("back") };
+    let ProbeFloatChoice::V1(numbers) = value.nullable_bag.ess_extra["some"].as_ref().unwrap().as_ref() else { panic!("numbers branch") };
+    let EssPresence::Present(maybe) = &value.optional_maybe else { panic!("present optional alias") };
+    [
+        a.a_b.0.get().to_bits(),
+        back.a_b.0.get().to_bits(),
+        b.numbers.ess_extra["$serde_json::private::RawValue"][0].as_ref().unwrap().0.get().to_bits(),
+        numbers.content.ess_extra["__proto__"][0].as_ref().unwrap().0.get().to_bits(),
+        maybe.0.as_ref().unwrap().0.get().to_bits(),
+    ]
+}
+
+#[test]
+fn recursive_ref_map_union_paths_keep_bits_and_float_markers() {
+    for (token, expected) in [
+        ("-0", 0x8000000000000000),
+        ("-0e999999999999999999999999999999", 0x8000000000000000),
+        ("-1e-999999999999999999999999999999", 0x8000000000000000),
+        ("2.4703282292062328e-324", 1),
+        ("-2.4703282292062328e-324", 0x8000000000000001),
+        ("9007199254740995", 0x4340000000000002),
+    ] {
+        let input = INPUT.replace("TOKEN", token);
+        for read_mode in 0..3 {
+            let value: ProbeFloatEnvelope = match read_mode {
+                0 => serde_json::from_str(&input).unwrap(),
+                1 => serde_json::from_slice(input.as_bytes()).unwrap(),
+                _ => serde_json::from_reader(input.as_bytes()).unwrap(),
+            };
+            assert_eq!(bits(&value), [expected; 5], "{token}, source mode {read_mode}");
+            let output = serde_json::to_string(&value).unwrap();
+            // Observe the emitted typed values, avoiding private-marker interpretation by
+            // serde_json::Value when a perfectly ordinary map key resembles that marker.
+            let again: ProbeFloatEnvelope = serde_json::from_str(&output).unwrap();
+            assert_eq!(bits(&again), [expected; 5], "{token}, source mode {read_mode}: {output}");
+            if expected == 0x8000000000000000 {
+                assert_eq!(output.matches("-0.0").count(), 5, "{output}");
+            }
+        }
+    }
+}
+
+#[test]
+fn marker_objects_and_late_invalid_values_cannot_take_a_float_path() {
+    for token in [
+        r#"{"$serde_json::private::Number":"-0"}"#,
+        r#"{"$serde_json::private::RawValue":"-0"}"#,
+        r#"{"$serde_json::private::RawValue":"-0","extra":0}"#,
+        "1.7976931348623159e308", "-1e999999999999999999999999999999",
+        r#""-0""#, "[]", "true",
+    ] {
+        for selected in 0..5 {
+            let mut index = 0;
+            let input = INPUT.split("TOKEN").enumerate().fold(String::new(), |mut text, (i, part)| {
+                if i != 0 {
+                    text.push_str(if index == selected { token } else { "-0" });
+                    index += 1;
+                }
+                text.push_str(part);
+                text
+            });
+            assert!(serde_json::from_str::<ProbeFloatEnvelope>(&input).is_err(), "slot {selected}: {input}");
+        }
+    }
+}
+
+#[test]
+fn optional_nullable_alias_and_independent_text_alternative_remain_distinct() {
+    let missing = r#"{"choice":{"value":"text","content":"-0"},"nullableBag":{}}"#;
+    let value: ProbeFloatEnvelope = serde_json::from_str(missing).unwrap();
+    let output: Value = serde_json::to_value(value).unwrap();
+    assert!(output.get("optionalMaybe").is_none());
+    let null = r#"{"choice":{"value":"text","content":"-0"},"nullableBag":{"none":null},"optionalMaybe":null}"#;
+    let value: ProbeFloatEnvelope = serde_json::from_str(null).unwrap();
+    let output: Value = serde_json::to_value(value).unwrap();
+    assert!(output["optionalMaybe"].is_null());
+    assert_eq!(output["choice"]["content"], "-0");
+    assert!(serde_json::from_str::<ProbeFloatScalar>("null").is_err());
+}
diff --git a/crates/generate/schema-contract/tests/fixtures/binary64_structural_adversary_go.txt b/crates/generate/schema-contract/tests/fixtures/binary64_structural_adversary_go.txt
new file mode 100644
index 0000000..ee45836
--- /dev/null
+++ b/crates/generate/schema-contract/tests/fixtures/binary64_structural_adversary_go.txt
@@ -0,0 +1,135 @@
+package adversary_types
+
+import (
+	"encoding/json"
+	"math"
+	"reflect"
+	"strings"
+	"testing"
+)
+
+const adversaryInput = `{"choice":{"value":"node","content":{"a/b~":TOKEN,"next":{"back":{"a/b~":TOKEN},"numbers":{"$serde_json::private::RawValue":[TOKEN,null]}}}},"nullableBag":{"none":null,"some":{"value":"numbers","content":{"__proto__":[TOKEN,null]}}},"optionalMaybe":TOKEN}`
+
+func adversaryBits(value any, found *[]uint64) {
+	switch v := value.(type) {
+	case float64:
+		*found = append(*found, math.Float64bits(v))
+	case []any:
+		for _, child := range v {
+			adversaryBits(child, found)
+		}
+	case map[string]any:
+		for _, child := range v {
+			adversaryBits(child, found)
+		}
+	}
+}
+
+func TestRecursiveSourceBits(t *testing.T) {
+	for _, c := range []struct {
+		token string
+		bits  uint64
+	}{
+		{"-0", 0x8000000000000000},
+		{"-0e999999999999999999999999999999", 0x8000000000000000},
+		{"-1e-999999999999999999999999999999", 0x8000000000000000},
+		{"2.4703282292062328e-324", 1},
+		{"-2.4703282292062328e-324", 0x8000000000000001},
+		{"9007199254740995", 0x4340000000000002},
+	} {
+		var value ProbeFloatEnvelope
+		input := strings.ReplaceAll(adversaryInput, "TOKEN", c.token)
+		if err := json.Unmarshal([]byte(input), &value); err != nil {
+			t.Fatal(c.token, err)
+		}
+		output, err := json.Marshal(value)
+		if err != nil {
+			t.Fatal(err)
+		}
+		var decoded any
+		if err := json.Unmarshal(output, &decoded); err != nil {
+			t.Fatal(err)
+		}
+		var actual []uint64
+		adversaryBits(decoded, &actual)
+		if len(actual) != 5 {
+			t.Fatalf("%s: %v", output, actual)
+		}
+		for _, b := range actual {
+			if b != c.bits {
+				t.Fatalf("%s: %016x", c.token, b)
+			}
+		}
+		if c.bits == 0x8000000000000000 && strings.Count(string(output), "-0.0") != 5 {
+			t.Fatalf("%s", output)
+		}
+	}
+}
+
+func TestEveryLateFailureLeavesTheWholeReceiverUntouched(t *testing.T) {
+	for _, token := range []string{
+		`{"$serde_json::private::Number":"-0"}`,
+		`{"$serde_json::private::RawValue":"-0"}`,
+		`{"$serde_json::private::RawValue":"-0","extra":0}`,
+		"1.7976931348623159e308", "-1e999999999999999999999999999999",
+		`"-0"`, "[]", "true",
+	} {
+		for selected := 0; selected < 5; selected++ {
+			var receiver ProbeFloatEnvelope
+			if err := json.Unmarshal([]byte(strings.ReplaceAll(adversaryInput, "TOKEN", "-0")), &receiver); err != nil {
+				t.Fatal(err)
+			}
+			before, err := json.Marshal(receiver)
+			if err != nil {
+				t.Fatal(err)
+			}
+			var independent ProbeFloatEnvelope
+			if err := json.Unmarshal(before, &independent); err != nil {
+				t.Fatal(err)
+			}
+			parts := strings.Split(adversaryInput, "TOKEN")
+			input := parts[0]
+			for i, part := range parts[1:] {
+				replacement := "9007199254740995"
+				if i == selected {
+					replacement = token
+				}
+				input += replacement + part
+			}
+			if err := receiver.UnmarshalJSON([]byte(input)); err == nil {
+				t.Fatalf("slot %d: %s", selected, input)
+			}
+			after, err := json.Marshal(receiver)
+			if err != nil {
+				t.Fatal(err)
+			}
+			if string(after) != string(before) || !reflect.DeepEqual(receiver, independent) {
+				t.Fatalf("partial receiver mutation at slot %d", selected)
+			}
+		}
+	}
+}
+
+func TestOptionalNullableAndRetryResetAreExplicit(t *testing.T) {
+	var value ProbeFloatEnvelope
+	present := `{"choice":{"value":"text","content":"-0"},"nullableBag":{"none":null},"optionalMaybe":null}`
+	if err := json.Unmarshal([]byte(present), &value); err != nil {
+		t.Fatal(err)
+	}
+	raw, err := json.Marshal(value)
+	if err != nil || !strings.Contains(string(raw), `"optionalMaybe":null`) {
+		t.Fatalf("%s %v", raw, err)
+	}
+	missing := `{"choice":{"value":"text","content":"-0"},"nullableBag":{}}`
+	if err := json.Unmarshal([]byte(missing), &value); err != nil {
+		t.Fatal(err)
+	}
+	raw, err = json.Marshal(value)
+	if err != nil || strings.Contains(string(raw), "optionalMaybe") {
+		t.Fatalf("stale optional state: %s %v", raw, err)
+	}
+	var scalar ProbeFloatScalar
+	if json.Unmarshal([]byte("null"), &scalar) == nil {
+		t.Fatal("null became a scalar")
+	}
+}
diff --git a/crates/edge/ess-cli/tests/binary64_structural_adversary.rs b/crates/edge/ess-cli/tests/binary64_structural_adversary.rs
new file mode 100644
index 0000000..7ddefa4
--- /dev/null
+++ b/crates/edge/ess-cli/tests/binary64_structural_adversary.rs
@@ -0,0 +1,123 @@
+//! Native Binary64 publication must preserve every destination when a later path refuses.
+
+use std::collections::BTreeMap;
+use std::fs;
+use std::path::{Path, PathBuf};
+use std::process::Command;
+
+fn snapshot(path: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
+    let mut files = BTreeMap::new();
+    if path.exists() {
+        for entry in fs::read_dir(path).unwrap() {
+            let path = entry.unwrap().path();
+            if path.is_dir() {
+                files.extend(snapshot(&path));
+            } else {
+                files.insert(path.clone(), fs::read(path).unwrap());
+            }
+        }
+    }
+    files
+}
+
+#[test]
+fn binary64_publication_never_replaces_sources_or_partially_updates_a_library() {
+    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
+        "binary64-structural-adversary-cli-{}",
+        std::process::id()
+    ));
+    let model = root.join("model");
+    fs::create_dir_all(&model).unwrap();
+    fs::write(model.join("system.yaml"), "format: ess/2\nsystem: probe\nversion: v1\ndomains: [probe.float]\ndomain: probe.float\ntypes:\n  - {name: probe.float.Number, kind: newtype, of: Binary64}\n").unwrap();
+    for (target, extension, manifest) in [("rust", "rs", "Cargo.toml"), ("go", "go", "go.mod")] {
+        let invoke = |destination: &Path| {
+            let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
+            command
+                .args(["generate", "types", "--path"])
+                .arg(&model)
+                .args([
+                    "--all-types",
+                    "--target",
+                    target,
+                    "--package",
+                    "adversary_types",
+                    "--out",
+                ])
+                .arg(destination)
+                .env_remove("ESS_GO_COMPILER")
+                .env("PATH", "/nonexistent");
+            if target == "go" {
+                command.args(["--module", "example.invalid/adversary-types"]);
+            }
+            let output = command.output().unwrap();
+            println!(
+                "{command:?}: {}\nstdout:\n{}\nstderr:\n{}",
+                output.status,
+                String::from_utf8_lossy(&output.stdout),
+                String::from_utf8_lossy(&output.stderr)
+            );
+            output
+        };
+        let destination = root.join(target);
+        fs::create_dir_all(&destination).unwrap();
+        for file in [
+            format!("types.{extension}"),
+            manifest.to_owned(),
+            "source.schema.json".to_owned(),
+        ] {
+            fs::write(destination.join(file), "existing library sentinel\n").unwrap();
+        }
+        fs::create_dir(destination.join("types-report.json")).unwrap();
+        fs::write(
+            destination.join("types-report.json/keep"),
+            "nested sentinel",
+        )
+        .unwrap();
+        let before = snapshot(&destination);
+        let output = invoke(&destination);
+        assert!(!output.status.success());
+        assert_eq!(
+            snapshot(&destination),
+            before,
+            "{target}: partial publication"
+        );
+
+        let before = snapshot(&model);
+        let nested = model.join(format!("{target}-inside"));
+        let output = invoke(&nested);
+        assert!(!output.status.success());
+        assert_eq!(snapshot(&model), before);
+        assert!(
+            !nested.exists(),
+            "created a destination inside source input"
+        );
+
+        #[cfg(unix)]
+        {
+            let linked = root.join(format!("{target}-linked"));
+            fs::create_dir_all(&linked).unwrap();
+            std::os::unix::fs::symlink(
+                model.join("system.yaml"),
+                linked.join(format!("types.{extension}")),
+            )
+            .unwrap();
+            let output = invoke(&linked);
+            assert!(!output.status.success());
+            assert_eq!(snapshot(&model), before);
+            assert!(!linked.join(manifest).exists());
+            assert!(!linked.join("types-report.json").exists());
+        }
+
+        let success = root.join(format!("{target}-fresh"));
+        assert!(invoke(&success).status.success());
+        let report: serde_json::Value =
+            serde_json::from_slice(&fs::read(success.join("types-report.json")).unwrap()).unwrap();
+        assert_eq!(report["format"], "ess-types-report/3");
+        assert_eq!(report["roots"], serde_json::json!(["probe.float.Number"]));
+        assert!(
+            fs::read_to_string(success.join(format!("types.{extension}")))
+                .unwrap()
+                .contains("EssBinary64")
+        );
+    }
+}
```


1. Added cases and first isolated executions

| New top-level case | Reachable workflow and assertion | Current result |
|---|---|---|
| `rust_recursive_unions_and_nullable_maps_keep_original_tokens` | Public compiler selection of the Envelope root, emitted Rust library, three native cases: original numeric bits through a mutually recursive reference graph, union/map/optional paths; 40 wrong-kind or overflow placements; missing versus null aliases. | Green |
| `go_recursive_maps_and_failed_decode_preserve_complete_receiver_state` | Same public selection and emitted Go library, three native cases under the race detector: numeric bits; all 40 late failure placements leave the complete receiver unchanged; successful retry resets missing optional state. | Green |
| `compiler_owned_helper_names_and_selected_root_reports_remain_conditional` | Public compiler names and root selection: helper collisions, conditional runtime/dependency emission, exact compiler-owned report locations, and retention of map-key and union obligations. | Green |
| `binary64_publication_never_replaces_sources_or_partially_updates_a_library` | Actual CLI Rust/Go generation with no compiler on PATH: a blocked report path, destination inside input and symlinked output each refuse without partial writes, plus successful fresh destinations. Six expected refusals and two successful controls. | Green |

Every authored top-level case was selected alone before any related suite. Each native case is also invoked separately with an exact name. Three initial harness failures are retained below and are not product findings:

- `isolated-config` asserted a map-key obligation while its initial fixture had only String keys. The authored fixture gained an actual `Optional<Map<Integer, Binary64>>` field; the assertion was retained.
- `isolated-config-corrected` used an invalid compiler domain equal to its system name. The authored helper-collision fixture was changed to valid system-level types with an empty domain list; collision assertions were retained.
- `isolated-rust` decoded and serialized the generated typed model, then its observer reparsed the emitted map through `serde_json::Value`. That observer treats `$serde_json::private::RawValue` as a private marker and failed on the array at that otherwise ordinary map key. The corrected observer reads generated typed fields and reparses output into the same generated type, preserving the marker-named key and bit assertions. The retained log records the original observer failure at `tests/wire.rs:34:49`; it is not a demonstrated generated-code defect.
Run `isolated-config`: 2026-09-06T12:24:13.145650+00:00 through 2026-09-06T12:24:15.311612+00:00.
```sh
cargo test --offline --locked -p schema-contract --features go-typecheck --test binary64_structural_adversary compiler_owned_helper_names_and_selected_root_reports_remain_conditional -- --exact --nocapture
```

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 2.14s
     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-072df0d06b054abe)

running 1 test

thread 'compiler_owned_helper_names_and_selected_root_reports_remain_conditional' (2028445) panicked at crates/generate/schema-contract/tests/binary64_structural_adversary.rs:158:9:
assertion failed: report["obligations"].as_array().unwrap().iter().any(|e|
        e["rule"] == "model_map_keys")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test compiler_owned_helper_names_and_selected_root_reports_remain_conditional ... FAILED

failures:

failures:
    compiler_owned_helper_names_and_selected_root_reports_remain_conditional

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p schema-contract --test binary64_structural_adversary`
```

Exit: `101`. Raw log: `$SCRATCH/isolated-config.log`.

Run `isolated-config-corrected`: 2026-09-06T12:25:01.211288+00:00 through 2026-09-06T12:25:01.568070+00:00.
```sh
cargo test --offline --locked -p schema-contract --features go-typecheck --test binary64_structural_adversary compiler_owned_helper_names_and_selected_root_reports_remain_conditional -- --exact --nocapture
```

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.33s
     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-072df0d06b054abe)

running 1 test

thread 'compiler_owned_helper_names_and_selected_root_reports_remain_conditional' (2031759) panicked at crates/generate/schema-contract/tests/fixtures/normalization_model.rs:63:14:
called `Result::unwrap()` on an `Err` value: ValidationErrors([ValidationError { code: SelfReference, location: "domain ess", message: "`ess` is the system itself, not a domain inside it", hint: Some("give it a namespace of its own, such as `ess.core`") }])
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test compiler_owned_helper_names_and_selected_root_reports_remain_conditional ... FAILED

failures:

failures:
    compiler_owned_helper_names_and_selected_root_reports_remain_conditional

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p schema-contract --test binary64_structural_adversary`
```

Exit: `101`. Raw log: `$SCRATCH/isolated-config-corrected.log`.

Run `isolated-config-system-types`: 2026-09-06T12:26:00.045127+00:00 through 2026-09-06T12:26:00.398206+00:00.
```sh
cargo test --offline --locked -p schema-contract --features go-typecheck --test binary64_structural_adversary compiler_owned_helper_names_and_selected_root_reports_remain_conditional -- --exact --nocapture
```

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.33s
     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-072df0d06b054abe)

running 1 test
test compiler_owned_helper_names_and_selected_root_reports_remain_conditional ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s

```

Exit: `0`. Raw log: `$SCRATCH/isolated-config-system-types.log`.

Run `isolated-rust`: 2026-09-06T12:26:07.528462+00:00 through 2026-09-06T12:26:10.373863+00:00.
```sh
cargo test --offline --locked -p schema-contract --features go-typecheck --test binary64_structural_adversary rust_recursive_unions_and_nullable_maps_keep_original_tokens -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.04s
     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-072df0d06b054abe)

running 1 test
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-rust-2036789" && "$CARGO" "generate-lockfile" "--offline"
native stdout:

native stderr:
     Locking 11 packages to latest compatible versions

native exit: exit status: 0
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-rust-2036789" && "$CARGO" "test" "--offline" "--locked" "--test" "wire" "recursive_ref_map_union_paths_keep_bits_and_float_markers" "--" "--exact" "--nocapture"
native stdout:

running 1 test
test recursive_ref_map_union_paths_keep_bits_and_float_markers ... FAILED

failures:

failures:
    recursive_ref_map_union_paths_keep_bits_and_float_markers

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s


native stderr:
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling serde_core v1.0.229
   Compiling zmij v1.0.23
   Compiling serde_json v1.0.151
   Compiling serde v1.0.229
   Compiling itoa v1.0.18
   Compiling memchr v2.8.3
   Compiling syn v3.0.5
   Compiling serde_derive v1.0.229
   Compiling adversary_types v0.0.0 ($WORKTREE/target/tmp/binary64-structural-adversary-rust-2036789)
    Finished `test` profile [unoptimized] target(s) in 2.73s
     Running tests/wire.rs (target/debug/deps/wire-27a7a461a504c1cc)

thread 'recursive_ref_map_union_paths_keep_bits_and_float_markers' (2037424) panicked at tests/wire.rs:34:49:
called `Result::unwrap()` on an `Err` value: Error("invalid type: sequence, expected raw value", line: 1, column: 107)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `--test wire`

native exit: exit status: 101

thread 'rust_recursive_unions_and_nullable_maps_keep_original_tokens' (2036790) panicked at crates/generate/schema-contract/tests/binary64_structural_adversary.rs:68:5:
native assertion failed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test rust_recursive_unions_and_nullable_maps_keep_original_tokens ... FAILED

failures:

failures:
    rust_recursive_unions_and_nullable_maps_keep_original_tokens

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 2.78s

error: test failed, to rerun pass `-p schema-contract --test binary64_structural_adversary`
```

Exit: `101`. Raw log: `$SCRATCH/isolated-rust.log`.

Run `isolated-rust-typed-observer`: 2026-09-06T12:28:26.938605+00:00 through 2026-09-06T12:28:30.255513+00:00.
```sh
cargo test --offline --locked -p schema-contract --features go-typecheck --test binary64_structural_adversary rust_recursive_unions_and_nullable_maps_keep_original_tokens -- --exact --nocapture
```

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.32s
     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-072df0d06b054abe)

running 1 test
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-rust-2053837" && "$CARGO" "generate-lockfile" "--offline"
native stdout:

native stderr:
     Locking 11 packages to latest compatible versions

native exit: exit status: 0
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-rust-2053837" && "$CARGO" "test" "--offline" "--locked" "--test" "wire" "recursive_ref_map_union_paths_keep_bits_and_float_markers" "--" "--exact" "--nocapture"
native stdout:

running 1 test
test recursive_ref_map_union_paths_keep_bits_and_float_markers ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s


native stderr:
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling serde_core v1.0.229
   Compiling zmij v1.0.23
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling itoa v1.0.18
   Compiling memchr v2.8.3
   Compiling syn v3.0.5
   Compiling serde_derive v1.0.229
   Compiling adversary_types v0.0.0 ($WORKTREE/target/tmp/binary64-structural-adversary-rust-2053837)
    Finished `test` profile [unoptimized] target(s) in 2.90s
     Running tests/wire.rs (target/debug/deps/wire-27a7a461a504c1cc)

native exit: exit status: 0
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-rust-2053837" && "$CARGO" "test" "--offline" "--locked" "--test" "wire" "marker_objects_and_late_invalid_values_cannot_take_a_float_path" "--" "--exact" "--nocapture"
native stdout:

running 1 test
test marker_objects_and_late_invalid_values_cannot_take_a_float_path ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s


native stderr:
    Finished `test` profile [unoptimized] target(s) in 0.01s
     Running tests/wire.rs (target/debug/deps/wire-27a7a461a504c1cc)

native exit: exit status: 0
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-rust-2053837" && "$CARGO" "test" "--offline" "--locked" "--test" "wire" "optional_nullable_alias_and_independent_text_alternative_remain_distinct" "--" "--exact" "--nocapture"
native stdout:

running 1 test
test optional_nullable_alias_and_independent_text_alternative_remain_distinct ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s


native stderr:
    Finished `test` profile [unoptimized] target(s) in 0.01s
     Running tests/wire.rs (target/debug/deps/wire-27a7a461a504c1cc)

native exit: exit status: 0
test rust_recursive_unions_and_nullable_maps_keep_original_tokens ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 2.98s

```

Exit: `0`. Raw log: `$SCRATCH/isolated-rust-typed-observer.log`.

Run `isolated-go`: 2026-09-06T12:29:05.310886+00:00 through 2026-09-06T12:29:09.298057+00:00.
```sh
cargo test --offline --locked -p schema-contract --features go-typecheck --test binary64_structural_adversary go_recursive_maps_and_failed_decode_preserve_complete_receiver_state -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-072df0d06b054abe)

running 1 test
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-go-2059126" && GOFLAGS="" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GOWORK="off" "/usr/bin/go" "test" "-count=1" "-race" "-p=1" "-v" "-run" "^TestRecursiveSourceBits$" "./..."
native stdout:
=== RUN   TestRecursiveSourceBits
--- PASS: TestRecursiveSourceBits (0.00s)
PASS
ok  	example.invalid/adversary-types	1.012s

native stderr:

native exit: exit status: 0
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-go-2059126" && GOFLAGS="" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GOWORK="off" "/usr/bin/go" "test" "-count=1" "-race" "-p=1" "-v" "-run" "^TestEveryLateFailureLeavesTheWholeReceiverUntouched$" "./..."
native stdout:
=== RUN   TestEveryLateFailureLeavesTheWholeReceiverUntouched
--- PASS: TestEveryLateFailureLeavesTheWholeReceiverUntouched (0.06s)
PASS
ok  	example.invalid/adversary-types	1.069s

native stderr:

native exit: exit status: 0
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-go-2059126" && GOFLAGS="" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GOWORK="off" "/usr/bin/go" "test" "-count=1" "-race" "-p=1" "-v" "-run" "^TestOptionalNullableAndRetryResetAreExplicit$" "./..."
native stdout:
=== RUN   TestOptionalNullableAndRetryResetAreExplicit
--- PASS: TestOptionalNullableAndRetryResetAreExplicit (0.00s)
PASS
ok  	example.invalid/adversary-types	1.007s

native stderr:

native exit: exit status: 0
test go_recursive_maps_and_failed_decode_preserve_complete_receiver_state ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 3.92s

```

Exit: `0`. Raw log: `$SCRATCH/isolated-go.log`.

Run `isolated-cli`: 2026-09-06T12:29:35.016756+00:00 through 2026-09-06T12:29:39.462071+00:00.
```sh
cargo test --offline --locked -p ess-cli --test binary64_structural_adversary binary64_publication_never_replaces_sources_or_partially_updates_a_library -- --exact --nocapture
```

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
   Compiling ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 4.37s
     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-e6799b61d16bc84b)

running 1 test
env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/model" "--all-types" "--target" "rust" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/rust": exit status: 1
stdout:

stderr:
error: output path has an incompatible file type or symlink: $WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/rust/types-report.json

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/model" "--all-types" "--target" "rust" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/model/rust-inside": exit status: 1
stdout:

stderr:
error: model type output must not replace or reside within its specification input

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/model" "--all-types" "--target" "rust" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/rust-linked": exit status: 1
stdout:

stderr:
error: output path has an incompatible file type or symlink: $WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/rust-linked/types.rs

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/model" "--all-types" "--target" "rust" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/rust-fresh": exit status: 0
stdout:
1 model type(s), written to $WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/rust-fresh; runtime obligations in types-report.json

stderr:

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/model" "--all-types" "--target" "go" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/go" "--module" "example.invalid/adversary-types": exit status: 1
stdout:

stderr:
error: output path has an incompatible file type or symlink: $WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/go/types-report.json

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/model" "--all-types" "--target" "go" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/model/go-inside" "--module" "example.invalid/adversary-types": exit status: 1
stdout:

stderr:
error: model type output must not replace or reside within its specification input

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/model" "--all-types" "--target" "go" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/go-linked" "--module" "example.invalid/adversary-types": exit status: 1
stdout:

stderr:
error: output path has an incompatible file type or symlink: $WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/go-linked/types.go

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/model" "--all-types" "--target" "go" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/go-fresh" "--module" "example.invalid/adversary-types": exit status: 0
stdout:
1 model type(s), written to $WORKTREE/target/tmp/binary64-structural-adversary-cli-2062820/go-fresh; runtime obligations in types-report.json

stderr:

test binary64_publication_never_replaces_sources_or_partially_updates_a_library ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

```

Exit: `0`. Raw log: `$SCRATCH/isolated-cli.log`.

2. Focused related suites, after all isolated cases

The comparable selected suite set contains five existing structural integration cases, four existing private codec-layout cases and three existing CLI model-types cases: 12 before additions, using the frozen implementor handoff and the unchanged test declarations as the before-count source. The executions below run those 12 plus four new top-level cases: 16 passed, zero failed, zero ignored. Nested native cases are reported separately and are not added twice to this count. The full package baseline in the implementor handoff is 134 schema-contract cases with `go-typecheck` and three CLI model-types cases; this pass does not claim to have rerun that entire package or a workspace/site gate.

Run `related-structural`: 2026-09-06T12:40:18.512424+00:00 through 2026-09-06T12:40:28.933587+00:00.
```sh
cargo test --offline --locked -p schema-contract --features go-typecheck --test binary64_structural --test binary64_structural_adversary -- --nocapture --test-threads=1
```

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.40s
     Running tests/binary64_structural.rs (target/debug/deps/binary64_structural-5ccdd3e2464d6eef)

running 5 tests
test all_selected_finite_model_types_emit_both_native_libraries ... ok
test complete_non_binary64_output_maps_remain_identical ... ok
test excluded_float_selection_preserves_plain_representation_and_reservations ... ok
test go_original_token_wire_corpus ... native Go stdout:
=== RUN   TestFiniteRoundingAndSerializedIdentity
--- PASS: TestFiniteRoundingAndSerializedIdentity (0.00s)
=== RUN   TestNumericKindConstructionAndAtomicScalar
--- PASS: TestNumericKindConstructionAndAtomicScalar (0.00s)
=== RUN   TestAllNestedStructuralPaths
--- PASS: TestAllNestedStructuralPaths (0.00s)
=== RUN   TestPresenceUnionAndAtomicRecords
--- PASS: TestPresenceUnionAndAtomicRecords (0.00s)
PASS
ok  	example.invalid/binary64types	0.003s

native Go stderr:

ok
test rust_original_token_wire_corpus ... native Rust stdout:

native Rust stderr:
     Locking 11 packages to latest compatible versions

native Rust stdout:

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 4 tests
test actual_numeric_source_kind_and_finite_construction ... ok
test presence_union_alternatives_and_prior_value_boundary ... ok
test finite_rounding_bits_and_serialized_float_identity ... ok
test original_tokens_reach_all_nested_structural_paths ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


native Rust stderr:
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling serde_core v1.0.229
   Compiling zmij v1.0.23
   Compiling serde_json v1.0.151
   Compiling serde v1.0.229
   Compiling memchr v2.8.3
   Compiling itoa v1.0.18
   Compiling syn v3.0.5
   Compiling serde_derive v1.0.229
   Compiling binary64_types v0.0.0 ($WORKTREE/target/tmp/binary64-rust-2126644)
    Finished `test` profile [unoptimized] target(s) in 2.78s
     Running unittests types.rs (target/debug/deps/binary64_types-dd762de61c823b0b)
     Running tests/wire.rs (target/debug/deps/wire-bf20571c69c598e5)
   Doc-tests binary64_types

ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.12s

     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-072df0d06b054abe)

running 3 tests
test compiler_owned_helper_names_and_selected_root_reports_remain_conditional ... ok
test go_recursive_maps_and_failed_decode_preserve_complete_receiver_state ... native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-go-2127529" && GOFLAGS="" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GOWORK="off" "/usr/bin/go" "test" "-count=1" "-race" "-p=1" "-v" "-run" "^TestRecursiveSourceBits$" "./..."
native stdout:
=== RUN   TestRecursiveSourceBits
--- PASS: TestRecursiveSourceBits (0.00s)
PASS
ok  	example.invalid/adversary-types	1.012s

native stderr:

native exit: exit status: 0
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-go-2127529" && GOFLAGS="" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GOWORK="off" "/usr/bin/go" "test" "-count=1" "-race" "-p=1" "-v" "-run" "^TestEveryLateFailureLeavesTheWholeReceiverUntouched$" "./..."
native stdout:
=== RUN   TestEveryLateFailureLeavesTheWholeReceiverUntouched
--- PASS: TestEveryLateFailureLeavesTheWholeReceiverUntouched (0.06s)
PASS
ok  	example.invalid/adversary-types	1.070s

native stderr:

native exit: exit status: 0
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-go-2127529" && GOFLAGS="" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GOWORK="off" "/usr/bin/go" "test" "-count=1" "-race" "-p=1" "-v" "-run" "^TestOptionalNullableAndRetryResetAreExplicit$" "./..."
native stdout:
=== RUN   TestOptionalNullableAndRetryResetAreExplicit
--- PASS: TestOptionalNullableAndRetryResetAreExplicit (0.00s)
PASS
ok  	example.invalid/adversary-types	1.007s

native stderr:

native exit: exit status: 0
ok
test rust_recursive_unions_and_nullable_maps_keep_original_tokens ... native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-rust-2127529" && "$CARGO" "generate-lockfile" "--offline"
native stdout:

native stderr:
     Locking 11 packages to latest compatible versions

native exit: exit status: 0
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-rust-2127529" && "$CARGO" "test" "--offline" "--locked" "--test" "wire" "recursive_ref_map_union_paths_keep_bits_and_float_markers" "--" "--exact" "--nocapture"
native stdout:

running 1 test
test recursive_ref_map_union_paths_keep_bits_and_float_markers ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s


native stderr:
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling serde_core v1.0.229
   Compiling zmij v1.0.23
   Compiling serde_json v1.0.151
   Compiling serde v1.0.229
   Compiling memchr v2.8.3
   Compiling itoa v1.0.18
   Compiling syn v3.0.5
   Compiling serde_derive v1.0.229
   Compiling adversary_types v0.0.0 ($WORKTREE/target/tmp/binary64-structural-adversary-rust-2127529)
    Finished `test` profile [unoptimized] target(s) in 2.93s
     Running tests/wire.rs (target/debug/deps/wire-27a7a461a504c1cc)

native exit: exit status: 0
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-rust-2127529" && "$CARGO" "test" "--offline" "--locked" "--test" "wire" "marker_objects_and_late_invalid_values_cannot_take_a_float_path" "--" "--exact" "--nocapture"
native stdout:

running 1 test
test marker_objects_and_late_invalid_values_cannot_take_a_float_path ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s


native stderr:
    Finished `test` profile [unoptimized] target(s) in 0.01s
     Running tests/wire.rs (target/debug/deps/wire-27a7a461a504c1cc)

native exit: exit status: 0
native command: cd "$WORKTREE/target/tmp/binary64-structural-adversary-rust-2127529" && "$CARGO" "test" "--offline" "--locked" "--test" "wire" "optional_nullable_alias_and_independent_text_alternative_remain_distinct" "--" "--exact" "--nocapture"
native stdout:

running 1 test
test optional_nullable_alias_and_independent_text_alternative_remain_distinct ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s


native stderr:
    Finished `test` profile [unoptimized] target(s) in 0.01s
     Running tests/wire.rs (target/debug/deps/wire-27a7a461a504c1cc)

native exit: exit status: 0
ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.88s

```

Exit: `0`. Raw log: `$SCRATCH/related-structural.log`.

Run `related-layout`: 2026-09-06T12:40:33.877099+00:00 through 2026-09-06T12:40:36.643320+00:00.
```sh
cargo test --offline --locked -p schema-contract --features go-typecheck --lib realize::binary64_codec_tests -- --nocapture --test-threads=1
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.04s
     Running unittests src/lib.rs (target/debug/deps/schema_contract-5e5a5202ac3ebed5)

running 4 tests
test realize::binary64_codec_tests::exact_tuples_and_conditional_helper_collisions_keep_their_boundaries ... ok
test realize::binary64_codec_tests::marked_non_numeric_or_missing_nodes_refuse_before_either_emission ... ok
test realize::binary64_codec_tests::native_tuple_union_order_and_known_open_fields_keep_original_tokens ... layout Rust:

     Locking 11 packages to latest compatible versions

layout Rust:

running 1 test
test raw_layouts ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling serde_core v1.0.229
   Compiling zmij v1.0.23
   Compiling serde_json v1.0.151
   Compiling serde v1.0.229
   Compiling itoa v1.0.18
   Compiling memchr v2.8.3
   Compiling syn v3.0.5
   Compiling serde_derive v1.0.229
   Compiling layout_types v0.0.0 ($WORKTREE/target/binary64-layout-rust-2129236)
    Finished `test` profile [unoptimized] target(s) in 2.46s
     Running unittests types.rs (target/debug/deps/layout_types-3da0fa0311a16556)
   Doc-tests layout_types

layout Go:
=== RUN   TestRawLayouts
--- PASS: TestRawLayouts (0.00s)
PASS
ok  	example.invalid/layouttypes	0.001s


ok
test realize::binary64_codec_tests::recursive_map_reachability_and_mixed_open_record_refusal_are_located ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 19 filtered out; finished in 2.70s

```

Exit: `0`. Raw log: `$SCRATCH/related-layout.log`.

Run `related-cli`: 2026-09-06T12:40:47.041353+00:00 through 2026-09-06T12:40:47.388780+00:00.
```sh
cargo test --offline --locked -p ess-cli --test model_types --test binary64_structural_adversary -- --nocapture --test-threads=1
```

```text
   Compiling ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.19s
     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-e6799b61d16bc84b)

running 1 test
test binary64_publication_never_replaces_sources_or_partially_updates_a_library ... env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/model" "--all-types" "--target" "rust" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/rust": exit status: 1
stdout:

stderr:
error: output path has an incompatible file type or symlink: $WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/rust/types-report.json

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/model" "--all-types" "--target" "rust" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/model/rust-inside": exit status: 1
stdout:

stderr:
error: model type output must not replace or reside within its specification input

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/model" "--all-types" "--target" "rust" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/rust-linked": exit status: 1
stdout:

stderr:
error: output path has an incompatible file type or symlink: $WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/rust-linked/types.rs

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/model" "--all-types" "--target" "rust" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/rust-fresh": exit status: 0
stdout:
1 model type(s), written to $WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/rust-fresh; runtime obligations in types-report.json

stderr:

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/model" "--all-types" "--target" "go" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/go" "--module" "example.invalid/adversary-types": exit status: 1
stdout:

stderr:
error: output path has an incompatible file type or symlink: $WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/go/types-report.json

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/model" "--all-types" "--target" "go" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/model/go-inside" "--module" "example.invalid/adversary-types": exit status: 1
stdout:

stderr:
error: model type output must not replace or reside within its specification input

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/model" "--all-types" "--target" "go" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/go-linked" "--module" "example.invalid/adversary-types": exit status: 1
stdout:

stderr:
error: output path has an incompatible file type or symlink: $WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/go-linked/types.go

env -u ESS_GO_COMPILER PATH="/nonexistent" "$WORKTREE/target/debug/ess" "generate" "types" "--path" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/model" "--all-types" "--target" "go" "--package" "adversary_types" "--out" "$WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/go-fresh" "--module" "example.invalid/adversary-types": exit status: 0
stdout:
1 model type(s), written to $WORKTREE/target/tmp/binary64-structural-adversary-cli-2130798/go-fresh; runtime obligations in types-report.json

stderr:

ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/model_types.rs (target/debug/deps/model_types-d1ddd7a24ad016b3)

running 3 tests
test all_type_binary64_libraries_publish_finite_codecs_with_atomic_preflight ... ok
test each_target_retains_the_same_model_selection_and_distinct_input_provenance ... ok
test root_selection_and_output_refusals_preserve_existing_files ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

```

Exit: `0`. Raw log: `$SCRATCH/related-cli.log`.

Scoped test lint and format checks: the first schema-test Clippy invocation rejected the similar local names `plan` and `plain` in the added test. Only that added-test local name changed to `non_binary64`. The initial lint failure is retained here, followed by green checks and an exact rerun of the affected test; it is not a product finding.

Run `clippy-schema-tests`: 2026-09-06T12:40:56.321383+00:00 through 2026-09-06T12:41:01.583966+00:00.
```sh
cargo clippy --offline --locked -p schema-contract --features go-typecheck --test binary64_structural_adversary -- -D warnings
```

```text
    Checking serde v1.0.229
    Checking thiserror v2.0.20
    Checking ref-cast v1.0.27
    Checking schemars v0.8.22
    Checking serde_yaml v0.9.34+deprecated
    Checking ahash v0.8.12
    Checking fluent-uri v0.4.1
    Checking jsonschema-value v0.52.1
    Checking referencing v0.52.1
    Checking email_address v0.2.9
    Checking ess-primitives v0.19.0 ($WORKTREE/crates/specify/ess-primitives)
    Checking jsonschema v0.52.1
    Checking ess-domain v0.19.0 ($WORKTREE/crates/specify/ess-domain)
    Checking ess-compiler v0.19.0 ($WORKTREE/crates/specify/ess-compiler)
    Checking ess-gen v0.19.0 ($WORKTREE/crates/generate/ess-gen)
    Checking schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
error: binding's name is too similar to existing binding
   --> crates/generate/schema-contract/tests/binary64_structural_adversary.rs:189:9
    |
189 |     let plain = Plan::from_model(&model::selection(SOURCE, &["probe.float.Plain"])).unwrap();
    |         ^^^^^
    |
note: existing binding defined here
   --> crates/generate/schema-contract/tests/binary64_structural_adversary.rs:148:9
    |
148 |     let plan = Plan::from_model(&selected).unwrap();
    |         ^^^^
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#similar_names
    = note: `-D clippy::similar-names` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::similar_names)]`

error: could not compile `schema-contract` (test "binary64_structural_adversary") due to 1 previous error
```

Exit: `101`. Raw log: `$SCRATCH/clippy-schema-tests.log`.

Run `clippy-schema-tests-corrected`: 2026-09-06T12:41:18.885772+00:00 through 2026-09-06T12:41:19.035331+00:00.
```sh
cargo clippy --offline --locked -p schema-contract --features go-typecheck --test binary64_structural_adversary -- -D warnings
```

```text
    Checking schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `dev` profile [unoptimized] target(s) in 0.12s
```

Exit: `0`. Raw log: `$SCRATCH/clippy-schema-tests-corrected.log`.

Run `clippy-cli-tests`: 2026-09-06T12:41:28.615437+00:00 through 2026-09-06T12:41:29.671525+00:00.
```sh
cargo clippy --offline --locked -p ess-cli --test binary64_structural_adversary -- -D warnings
```

```text
    Checking schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Checking ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 1.02s
```

Exit: `0`. Raw log: `$SCRATCH/clippy-cli-tests.log`.

Run `format-check`: 2026-09-06T12:41:48.366598+00:00 through 2026-09-06T12:41:48.380544+00:00.
```sh
rustfmt --edition 2021 --config skip_children=true --check crates/generate/schema-contract/tests/binary64_structural_adversary.rs crates/edge/ess-cli/tests/binary64_structural_adversary.rs
```

```text
(empty output)
```

Exit: `0`. Raw log: `$SCRATCH/format-check.log`.

Run `post-lint-config`: 2026-09-06T12:41:48.451008+00:00 through 2026-09-06T12:41:48.821503+00:00.
```sh
cargo test --offline --locked -p schema-contract --features go-typecheck --test binary64_structural_adversary compiler_owned_helper_names_and_selected_root_reports_remain_conditional -- --exact --nocapture
```

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.34s
     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-072df0d06b054abe)

running 1 test
test compiler_owned_helper_names_and_selected_root_reports_remain_conditional ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s

```

Exit: `0`. Raw log: `$SCRATCH/post-lint-config.log`.

Formatting history: the two initial formatting steps have empty retained logs and recorded exit 0 (`format-0` for the two added Rust integration files; `format-1` for the added Go fixture). Subsequent formatting only addressed those added Rust files and used `skip_children=true`; exact command/output/exit records follow, including formatting after the lint-only rename.

Run `format-final`: 2026-09-06T12:34:05.968135+00:00 through 2026-09-06T12:34:05.981968+00:00.
```sh
rustfmt --edition 2021 --config skip_children=true crates/generate/schema-contract/tests/binary64_structural_adversary.rs crates/edge/ess-cli/tests/binary64_structural_adversary.rs
```

```text
(empty output)
```

Exit: `0`. Raw log: `$SCRATCH/format-final.log`.

Run `format-postlint`: 2026-09-06T12:41:48.273500+00:00 through 2026-09-06T12:41:48.287142+00:00.
```sh
rustfmt --edition 2021 --config skip_children=true crates/generate/schema-contract/tests/binary64_structural_adversary.rs crates/edge/ess-cli/tests/binary64_structural_adversary.rs
```

```text
(empty output)
```

Exit: `0`. Raw log: `$SCRATCH/format-postlint.log`.

3. Findings

Nothing found against frozen source `291f229256ce4fa78a17a01b085f56a9e5ab6990`. There are no judgement findings or finding rows. The three harness failures above are retained as execution history and do not populate the findings block.

4. Attacked boundaries and limits

- Public compiler-owned, transitively selected recursive model roots preserve original float tokens through generated Rust `from_str`, `from_slice` and `from_reader` paths and generated Go raw-token decoding.
- Six fixed bit expectations cover lexical negative zero, enormous exponents on zero, signed underflow, positive and negative minimum-subnormal rounding, and the integer tie `9007199254740995`; all five scalar positions survive typed/native round trips.
- Forty invalid placements per language reject private Number/RawValue marker objects, malformed scalar kinds and overflow. Go additionally retains the complete prior receiver on every refusal.
- Missing optional values, explicit nullable aliases, nullable map entries, reserved-looking ordinary map keys and an independent string union alternative retain their distinct meanings.
- Public helper-name collisions and selected-root report accounting remain conditional; the existing complete non-Binary64 emitted-file-map checksum case passes with frozen digest `02312f45fadac5e16b54aa1fce13d8f68cd8aaf14336a2180b85cd923811c79f`.
- Existing private layout cases cover sealed metadata guards, tuple/union order, mixed-open-record refusal and original tokens in declared open fields. Those private states are not presented as public model construction workflows.
- Actual CLI destination preflight preserves source and existing destination contents, including late report-path and symlink refusals, and succeeds without invoking a native compiler.
- This pass attacks the standalone structural-codec unit and its stated report/publication boundaries. Rust `from_value` prior sign loss and the documented mixed declared/Binary64-extra refusal remain explicit limits; whole-system/conformance support, TypeScript runtime support and unrestricted schema validation are not claimed.

5. Source content and test proof

Frozen manifest SHA-256: `98b22d030fc5502ba2a55aa0a39874ceb900f5268b759a61fcbb4f7a67150bc7`. Per-file original and observed hashes/modes for all 969 entries are retained in `source-proof.json` (SHA-256 `61e74a1ebd921943b98c23b4f942498e10ebadf084b9a9f6756e609867f6cb08`). The audit observed exact HEAD `291f229256ce4fa78a17a01b085f56a9e5ab6990` and no changed frozen entry.

| Added test path | Bytes | SHA-256 |
|---|---:|---|
| `crates/generate/schema-contract/tests/binary64_structural_adversary.rs` | 8048 | `704a291a2b07e624a9b949b17c5179c0a85e780e0f59cdf6f1adad86f1137f7c` |
| `crates/generate/schema-contract/tests/fixtures/binary64_structural_adversary_rust.txt` | 4366 | `44d86177a9ac3d65313d3c79ade499cb6ac1b0db9ed4a2afb16a14019e4c8bb9` |
| `crates/generate/schema-contract/tests/fixtures/binary64_structural_adversary_go.txt` | 3996 | `008af47c4ea35393775cf8f31324ab297a87536580fce705b9c2e31e9c9858eb` |
| `crates/edge/ess-cli/tests/binary64_structural_adversary.rs` | 4606 | `00d662dff031a0b51fc5aa8f37fa273a8822b64e378bcea7281cdefece9af0d7` |

6. Outside-worktree writes and retained execution inventory

All intentionally created non-build scratch files are listed below, including this report, its companion, scripts and manifests. The coordinator supplied `unit-brief.md` and `root-frozen-files.json` before the pass; those two files were only read. Native generated library roots, their target directories and CLI fixture trees stay inside the assigned worktree `target/`; no alternate worktree or external Cargo target was selected.

Every logged execution used four Cargo jobs, no incremental compilation, debug level zero for dev/test, the supplied sccache wrapper, offline/locked Cargo, `/usr/bin/go`, and the assigned scratch directory as TMPDIR. New Go native tests set local toolchain, proxy off, sumdb off and run serially with the race detector. Every allocation preflight checked an 8 GiB free-space floor. Temporary files below the assigned TMPDIR are tool-managed and may be removed by the tools themselves. Standard tool cache activity was not redirected: `$USER_HOME/.cache/sccache`, `$USER_HOME/.cache/go-build` and `$USER_HOME/.cargo` are the cache roots in use; per-entry cache mutations were not instrumented, so the retained-file inventory is not a claim that the standard caches received no writes.

- `$SCRATCH/added-test-hashes.json`
- `$SCRATCH/added-tests.patch`
- `$SCRATCH/assemble-report.py`
- `$SCRATCH/audit-source.py`
- `$SCRATCH/clippy-cli-tests.command.json`
- `$SCRATCH/clippy-cli-tests.exit`
- `$SCRATCH/clippy-cli-tests.log`
- `$SCRATCH/clippy-schema-tests-corrected.command.json`
- `$SCRATCH/clippy-schema-tests-corrected.exit`
- `$SCRATCH/clippy-schema-tests-corrected.log`
- `$SCRATCH/clippy-schema-tests.command.json`
- `$SCRATCH/clippy-schema-tests.exit`
- `$SCRATCH/clippy-schema-tests.log`
- `$SCRATCH/command-exits.json`
- `$SCRATCH/diff-stat.json`
- `$SCRATCH/format-0.exit`
- `$SCRATCH/format-0.log`
- `$SCRATCH/format-1.exit`
- `$SCRATCH/format-1.log`
- `$SCRATCH/format-check.command.json`
- `$SCRATCH/format-check.exit`
- `$SCRATCH/format-check.log`
- `$SCRATCH/format-final.command.json`
- `$SCRATCH/format-final.exit`
- `$SCRATCH/format-final.log`
- `$SCRATCH/format-postlint.command.json`
- `$SCRATCH/format-postlint.exit`
- `$SCRATCH/format-postlint.log`
- `$SCRATCH/isolated-cli.command.json`
- `$SCRATCH/isolated-cli.exit`
- `$SCRATCH/isolated-cli.log`
- `$SCRATCH/isolated-config-corrected.command.json`
- `$SCRATCH/isolated-config-corrected.exit`
- `$SCRATCH/isolated-config-corrected.log`
- `$SCRATCH/isolated-config-system-types.command.json`
- `$SCRATCH/isolated-config-system-types.exit`
- `$SCRATCH/isolated-config-system-types.log`
- `$SCRATCH/isolated-config.command.json`
- `$SCRATCH/isolated-config.exit`
- `$SCRATCH/isolated-config.log`
- `$SCRATCH/isolated-go.command.json`
- `$SCRATCH/isolated-go.exit`
- `$SCRATCH/isolated-go.log`
- `$SCRATCH/isolated-rust-typed-observer.command.json`
- `$SCRATCH/isolated-rust-typed-observer.exit`
- `$SCRATCH/isolated-rust-typed-observer.log`
- `$SCRATCH/isolated-rust.command.json`
- `$SCRATCH/isolated-rust.exit`
- `$SCRATCH/isolated-rust.log`
- `$SCRATCH/outside-writes.json`
- `$SCRATCH/post-lint-config.command.json`
- `$SCRATCH/post-lint-config.exit`
- `$SCRATCH/post-lint-config.log`
- `$SCRATCH/preflight.json`
- `$SCRATCH/raw-log-hashes.json`
- `$SCRATCH/related-cli.command.json`
- `$SCRATCH/related-cli.exit`
- `$SCRATCH/related-cli.log`
- `$SCRATCH/related-layout.command.json`
- `$SCRATCH/related-layout.exit`
- `$SCRATCH/related-layout.log`
- `$SCRATCH/related-structural.command.json`
- `$SCRATCH/related-structural.exit`
- `$SCRATCH/related-structural.log`
- `$SCRATCH/report-private.md`
- `$SCRATCH/report-public.md`
- `$SCRATCH/report-receipt.json`
- `$SCRATCH/run-command.py`
- `$SCRATCH/source-proof.json`

All Cargo/native processes were terminal when the compiler lane was released to the coordinator; report assembly and source auditing perform no builds.

```findings
[]
```