---
format: aep.planning-md/1
id: review-result:normalization-binary64-count-integration-pass1
kind: review-result
status: active
title: Binary64 and count-writer integration adversary pass 1
owner: aep-drive:adversary
relations:
- reviews: story:model-binary64-fields
revision: 1
---
unit: Binary64/count integration adversary pass 1 at 12fb11a9225cea39975ad3303edc907046634238
verdict: nothing found
cases: executed 8→11, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 39 retained files; compiler temporary children under assigned scratch
needs-coordinator: record this report; full workspace/site gates and publication remain coordinator-owned
```console
git --no-pager diff --stat
```
Actual exit 0, empty stdout. No tracked source or existing test changed. The two new test files are untracked; these explicit no-index stat commands include their changes without staging. Each exits 1 because the test differs from /dev/null.

```text
 .../edge/ess-cli/tests/binary64_count_adversary.rs | 212 +++++++++++++++++++++
 1 file changed, 212 insertions(+)
 .../tests/binary64_count_adversary.rs              | 208 +++++++++++++++++++++
 1 file changed, 208 insertions(+)
```

Complete added-test diff; both `git --no-pager diff --no-index -- /dev/null <test>` commands exit 1:

```diff
diff --git a/crates/edge/ess-cli/tests/binary64_count_adversary.rs b/crates/edge/ess-cli/tests/binary64_count_adversary.rs
new file mode 100644
index 0000000..ef402fa
--- /dev/null
+++ b/crates/edge/ess-cli/tests/binary64_count_adversary.rs
@@ -0,0 +1,212 @@
+//! Combined CLI and generated Go count publication refuses Binary64 before effects.
+
+use ess_conformance::ConformanceSuite;
+use serde_json::{json, Value};
+use std::{
+    fs,
+    path::{Path, PathBuf},
+    process::{Command, Output},
+};
+
+const ID: &str = "probe.data/authored/guard";
+fn document(binary64: Option<bool>) -> Value {
+    let steps = binary64.map_or_else(|| json!([]), |optional| json!([
+        {"step":"expect_event","event":"probe.data.Created","shape":{"ratio/a~b":{"holds":"primitive","kind":"binary64","optional":optional}}}
+    ]));
+    json!({"provenance":{"suite_version":"ess-conformance/4","system":"probe","specification_version":"v1","spec_digest":"a".repeat(64),"contract_digest":"a".repeat(64)},
+        "scenarios":{ID:{"purpose":"Refuse Binary64 before report publication","steps":steps,"source":[]}}})
+}
+fn scratch(label: &str) -> PathBuf {
+    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
+        "binary64-count-adversary-{label}-{}",
+        std::process::id()
+    ));
+    fs::create_dir_all(&root).unwrap();
+    root
+}
+fn record(root: &Path, label: &str, command: &Command, result: &Output) -> String {
+    let text = format!(
+        "{}{}",
+        String::from_utf8_lossy(&result.stdout),
+        String::from_utf8_lossy(&result.stderr)
+    );
+    fs::write(
+        root.join(format!("{label}.log")),
+        format!("{command:?}\nexit {}\n{text}", result.status),
+    )
+    .unwrap();
+    text
+}
+
+#[test]
+fn cli_binary64_model_and_original_suites_preserve_report_destinations_in_both_formats() {
+    let root = scratch("cli");
+    fs::create_dir_all(root.join("model")).unwrap();
+    fs::write(root.join("model/system.yaml"),"format: ess/2\nsystem: probe\nversion: v1\ndomains: [probe.data]\ndomain: probe.data\ntypes:\n  - {name: probe.data.Ratio, kind: newtype, of: Binary64}\n").unwrap();
+    fs::write(root.join("suite.json"), document(Some(true)).to_string()).unwrap();
+    for format in ["1", "2"] {
+        for model in [false, true] {
+            for existing in [false, true] {
+                let label = format!("format-{format}-model-{model}-existing-{existing}");
+                let destination = root.join(format!("{label}.report.json"));
+                if existing {
+                    fs::write(&destination, b"owned report bytes\n").unwrap();
+                }
+                let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
+                command
+                    .current_dir(&root)
+                    .args([
+                        "verify",
+                        "conform",
+                        "run",
+                        "--target",
+                        "billing",
+                        "--report-format",
+                        format,
+                        "--allow-incomplete",
+                        "--format",
+                        "json",
+                    ])
+                    .args(if model {
+                        ["--path", "model"]
+                    } else {
+                        ["--suite", "suite.json"]
+                    })
+                    .arg("--report-out")
+                    .arg(&destination);
+                let output = command.output().unwrap();
+                let text = record(&root, &label, &command, &output);
+                assert!(!output.status.success(), "{label}: {text}");
+                assert!(text.contains("Binary64"), "{label}: {text}");
+                if model {
+                    assert!(text.contains("probe.data.Ratio"), "{text}");
+                }
+                if existing {
+                    assert_eq!(fs::read(&destination).unwrap(), b"owned report bytes\n");
+                } else {
+                    assert!(!destination.exists(), "{label} published a report");
+                }
+                eprintln!(
+                    "{label}: actual CLI exit {}; destination preserved",
+                    output.status
+                );
+            }
+        }
+    }
+}
+
+fn invoke_go(root: &Path, label: &str, format: &str, marker: &Path, destination: &Path) -> Output {
+    let compiler = PathBuf::from(std::env::var_os("ESS_GO_COMPILER").unwrap());
+    assert!(compiler.is_absolute() && compiler.is_file());
+    let mut command = Command::new(compiler);
+    command
+        .args([
+            "test",
+            "-v",
+            "-count=1",
+            "-race",
+            "-mod=readonly",
+            "./...",
+            "-run",
+            "^TestGuard$",
+        ])
+        .current_dir(root)
+        .env("GOTOOLCHAIN", "local")
+        .env("GOPROXY", "off")
+        .env("GOSUMDB", "off")
+        .env("GOFLAGS", "")
+        .env("GOMAXPROCS", "4")
+        .env("ESS_REPORT_FORMAT", format)
+        .env_remove("ESS_CONFORMANCE_STRICT")
+        .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE")
+        .env("ESS_REPORT_OUT", destination)
+        .env("GUARD_MARKER", marker)
+        .env(
+            "GOCACHE",
+            Path::new(env!("CARGO_TARGET_TMPDIR")).join("binary64-count-adversary-go-cache"),
+        );
+    let result = command.output().unwrap();
+    record(root, label, &command, &result);
+    result
+}
+
+#[test]
+fn generated_go_binary64_shapes_refuse_before_factory_and_report_publication() {
+    let root = scratch("go");
+    let control = document(None).to_string();
+    let suite = ConformanceSuite::from_json(&control).unwrap();
+    for artifact in ess_conformance::go::emit(&suite).unwrap() {
+        let path = root.join(artifact.path);
+        fs::create_dir_all(path.parent().unwrap()).unwrap();
+        fs::write(path, artifact.contents).unwrap();
+    }
+    fs::write(
+        root.join("go.mod"),
+        "module example.invalid/binary64-count-adversary\n\ngo 1.24\n",
+    )
+    .unwrap();
+    fs::write(root.join("essconform/binary64_count_adversary_test.go"),r#"package essconform
+import ("os"; "testing")
+type guardTarget struct { Target }
+func (guardTarget) Identity() (Identity,error) { return Identity{Name:"guard",Version:"1"},nil }
+func (guardTarget) BeginScenario(ScenarioContext) error { return nil }
+func (guardTarget) EndScenario(ScenarioContext) error { return nil }
+func TestGuard(t *testing.T) { Run(t,func() Target {
+    if err:=os.WriteFile(os.Getenv("GUARD_MARKER"),[]byte("factory called\n"),0600);err!=nil { panic(err) }
+    return guardTarget{}
+}) }
+"#).unwrap();
+    for format in ["1", "2"] {
+        fs::write(root.join("essconform/suite.json"), &control).unwrap();
+        let label = format!("control-{format}");
+        let marker = root.join(format!("{label}.marker"));
+        let destination = root.join(format!("{label}.report.json"));
+        let output = invoke_go(&root, &label, format, &marker, &destination);
+        assert!(
+            output.status.success(),
+            "{}{}",
+            String::from_utf8_lossy(&output.stdout),
+            String::from_utf8_lossy(&output.stderr)
+        );
+        assert_eq!(fs::read(&marker).unwrap(), b"factory called\n");
+        let report: Value = serde_json::from_slice(&fs::read(&destination).unwrap()).unwrap();
+        assert_eq!(report["format"], format!("ess-conformance-report/{format}"));
+        eprintln!("{label}: actual Go exit {}; control reached factory and published selected report format",output.status);
+        for optional in [false, true] {
+            fs::write(
+                root.join("essconform/suite.json"),
+                document(Some(optional)).to_string(),
+            )
+            .unwrap();
+            for existing in [false, true] {
+                let label = format!("format-{format}-optional-{optional}-existing-{existing}");
+                let marker = root.join(format!("{label}.marker"));
+                let destination = root.join(format!("{label}.report.json"));
+                if existing {
+                    fs::write(&destination, b"owned report bytes\n").unwrap();
+                }
+                let output = invoke_go(&root, &label, format, &marker, &destination);
+                let text = format!(
+                    "{}{}",
+                    String::from_utf8_lossy(&output.stdout),
+                    String::from_utf8_lossy(&output.stderr)
+                );
+                assert!(!output.status.success(), "{label}: {text}");
+                assert!(
+                    text.contains("suite admission") && text.contains("unknown primitive"),
+                    "{label}: {text}"
+                );
+                assert!(!marker.exists(), "{label} reached target factory");
+                if existing {
+                    assert_eq!(fs::read(&destination).unwrap(), b"owned report bytes\n");
+                } else {
+                    assert!(!destination.exists(), "{label} published report");
+                }
+                eprintln!(
+                    "{label}: actual Go exit {}; factory untouched and destination preserved",
+                    output.status
+                );
+            }
+        }
+    }
+}
diff --git a/crates/verify/ess-conformance/tests/binary64_count_adversary.rs b/crates/verify/ess-conformance/tests/binary64_count_adversary.rs
new file mode 100644
index 0000000..e086406
--- /dev/null
+++ b/crates/verify/ess-conformance/tests/binary64_count_adversary.rs
@@ -0,0 +1,208 @@
+//! Combined finite-primitive and original-byte count admission boundaries.
+
+use ess_conformance::target::{
+    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
+    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
+    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
+};
+use ess_conformance::{
+    AdmittedSuite, Clock, ConformanceSuite, Holds, Ids, LeafShape, Runner, RunnerConfig,
+    ScenarioStep,
+};
+use ess_domain::Primitive;
+use ess_primitives::time::Timestamp;
+use serde_json::{json, Value};
+use std::cell::Cell;
+
+const ID: &str = "probe.data/authored/guard";
+
+fn document() -> Value {
+    json!({"provenance":{"suite_version":"ess-conformance/4","system":"probe","specification_version":"v1","spec_digest":"a".repeat(64),"contract_digest":"a".repeat(64)},
+        "scenarios":{ID:{"purpose":"Refuse unsupported primitive before effects","steps":[
+            {"step":"expect_event","event":"probe.data.Created","shape":{}},
+            {"step":"eventually_event","event":"probe.data.Created","shape":{}}
+        ],"source":[]}}})
+}
+
+struct CountingClock<'a>(&'a Cell<usize>);
+impl Clock for CountingClock<'_> {
+    fn now(&mut self) -> Timestamp {
+        self.0.set(self.0.get() + 1);
+        Timestamp::from_epoch_millis(0)
+    }
+}
+struct UnreachableTarget(Cell<usize>);
+impl ConformanceTarget for UnreachableTarget {
+    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
+        self.0.set(self.0.get() + 1);
+        Ok(ImplementationIdentity::new("unreachable", "1"))
+    }
+    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
+        panic!("refused suite reached scenario callback")
+    }
+    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
+        panic!("refused suite reached scenario teardown")
+    }
+    fn execute_command(
+        &self,
+        _: SemanticCommandRequest,
+    ) -> Result<SemanticCommandResult, TargetError> {
+        panic!("refused suite reached command")
+    }
+    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
+        panic!("refused suite reached view")
+    }
+    fn observe_events(
+        &self,
+        _: EventObservationRequest,
+    ) -> Result<Vec<ObservedEvent>, TargetError> {
+        panic!("refused suite reached event observation")
+    }
+    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
+        panic!("refused suite reached outcome control")
+    }
+    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
+        panic!("refused suite reached redelivery")
+    }
+}
+
+#[test]
+fn both_fallible_runners_preserve_all_binary64_issues_before_effects() {
+    let mut suite = ConformanceSuite::from_json(&document().to_string()).unwrap();
+    for scenario in suite.scenarios.values_mut() {
+        for step in &mut scenario.steps {
+            let (ScenarioStep::ExpectEvent { shape, .. }
+            | ScenarioStep::EventuallyEvent { shape, .. }) = step
+            else {
+                panic!("fixture step")
+            };
+            shape.insert(
+                "ratio/a~b",
+                LeafShape::required(Holds::Primitive {
+                    kind: Primitive::Binary64,
+                })
+                .optional(),
+            );
+            shape.insert(
+                "z",
+                LeafShape::required(Holds::Primitive {
+                    kind: Primitive::Binary64,
+                }),
+            );
+        }
+    }
+    let expected = suite.to_canonical_json().unwrap_err();
+    assert_eq!(expected.issues.len(), 4);
+    let pointers = expected
+        .issues
+        .iter()
+        .map(|issue| issue.path.as_str())
+        .collect::<Vec<_>>();
+    assert_eq!(
+        pointers,
+        [
+            "/scenarios/probe.data~1authored~1guard/steps/0/shape/ratio~1a~0b",
+            "/scenarios/probe.data~1authored~1guard/steps/0/shape/z",
+            "/scenarios/probe.data~1authored~1guard/steps/1/shape/ratio~1a~0b",
+            "/scenarios/probe.data~1authored~1guard/steps/1/shape/z"
+        ]
+    );
+    assert!(expected
+        .issues
+        .iter()
+        .all(|issue| issue.reason == "UnsupportedPrimitive" && issue.detail.contains("Binary64")));
+    assert_eq!(AdmittedSuite::from_suite(&suite).unwrap_err(), expected);
+    assert_eq!(ess_conformance::go::emit(&suite).unwrap_err(), expected);
+    assert!(serde_json::to_string(&suite).is_err());
+    let clock = Cell::new(0);
+    let target = UnreachableTarget(Cell::new(0));
+    let runner = || {
+        Runner::new(
+            RunnerConfig::default(),
+            CountingClock(&clock),
+            Ids::for_suite(&suite),
+        )
+    };
+    assert_eq!(runner().run(&suite, &target).unwrap_err(), expected);
+    assert_eq!(runner().try_run(&suite, &target).unwrap_err(), expected);
+    assert_eq!(clock.get(), 0);
+    assert_eq!(target.0.get(), 0);
+    eprintln!("four located issues retained; run and try_run refused with zero clock/identity/callback effects");
+}
+
+#[test]
+fn original_byte_and_serde_admission_refuse_binary64_across_all_legacy_suite_majors() {
+    for major in 1..=4 {
+        for step in ["expect_event", "eventually_event"] {
+            for optional in [false, true] {
+                let mut value = document();
+                value["provenance"]["suite_version"] = json!(format!("ess-conformance/{major}"));
+                value["scenarios"][ID]["steps"] = json!([{"step":step,"event":"probe.data.Created","shape":{"ratio/a~b":{"holds":"primitive","kind":"decimal","optional":optional}}}]);
+                AdmittedSuite::from_json(&value.to_string()).expect("legacy control admits");
+                value["scenarios"][ID]["steps"][0]["shape"]["ratio/a~b"]["kind"] =
+                    json!("binary64");
+                let raw = value.to_string();
+                assert!(AdmittedSuite::from_json(&raw)
+                    .unwrap_err()
+                    .to_string()
+                    .contains("Binary64"));
+                assert!(ConformanceSuite::from_json(&raw).is_err());
+                assert!(serde_json::from_str::<ConformanceSuite>(&raw).is_err());
+            }
+        }
+    }
+    let mut value = document();
+    value["provenance"]["suite_version"] = json!("ess-conformance/5");
+    let dto = ConformanceSuite::from_json(&value.to_string())
+        .expect("historical unadmitted DTO still parses");
+    let clock = Cell::new(0);
+    let target = UnreachableTarget(Cell::new(0));
+    let runner = || {
+        Runner::new(
+            RunnerConfig::default(),
+            CountingClock(&clock),
+            Ids::for_suite(&dto),
+        )
+    };
+    assert!(runner().run(&dto, &target).is_err());
+    assert!(runner().try_run(&dto, &target).is_err());
+    assert_eq!((clock.get(), target.0.get()), (0, 0));
+    eprintln!("16 admitted decimal controls and 48 Binary64 reader refusals; both typed runners reject suite/5 before effects");
+}
+
+#[test]
+fn unified_model_issues_survive_authored_synthesis_and_web_boundaries() {
+    use ess_compiler::{resolve::compile, source::SourceMap};
+    use ess_domain::{
+        spec::{RawSpecFile, Specification},
+        system::Source,
+    };
+    let source = "format: ess/2\nsystem: probe\nversion: v1\ndomains: [probe.data]\ndomain: probe.data\ntypes:\n  - name: probe.data.Values\n    kind: struct\n    fields:\n      - {name: alpha, type: Binary64}\n      - {name: beta, type: 'List<Binary64>'}\n      - {name: gamma, type: 'Optional<Binary64>'}\n";
+    let spec = Specification::assemble([(Source::document(), RawSpecFile::parse(source).unwrap())])
+        .unwrap();
+    let ir = compile(&spec, &SourceMap::new()).unwrap();
+    let expected = ess_conformance::admission::model(&ir).unwrap_err();
+    assert_eq!(expected.issues.len(), 3);
+    let paths = expected
+        .issues
+        .iter()
+        .map(|issue| issue.path.clone())
+        .collect::<Vec<_>>();
+    let authoring = ess_conformance::authored::compile(&ir, &[]);
+    assert!(authoring.scenarios.is_empty());
+    assert_eq!(authoring.refusals.len(), 1);
+    let ess_conformance::authored::Cause::UnsupportedBinary64 { locations } =
+        &authoring.refusals[0].cause
+    else {
+        panic!("wrong refusal")
+    };
+    assert_eq!(locations, &paths);
+    let synthesis = ess_conformance::synthesize(&ir);
+    assert!(synthesis.suite.is_empty());
+    assert_eq!(synthesis.refusals.len(), 3);
+    assert_eq!(
+        ess_conformance::web::emit(&ir, &synthesis.suite).unwrap_err(),
+        expected
+    );
+    eprintln!("three model positions retained through unified issues, authored cause, synthesis and prepublication web refusal");
+}
```

Publication normalization: `$WORKTREE` denotes the frozen coordinator checkout and `$SCRATCH` its assigned integration-adversary directory. Personal absolute paths in commands, quoted output and inventories have been replaced by those aliases. Public quotes are explicitly path-normalized, not verbatim; original output bytes remain in the private log files and private-report.md. The complete test-source diff and conclusions are unchanged. The public body has no terminal LF.

No product finding was reproduced in this integration pass.

The header counts the conformance lane: eight existing count-report and digest-binding tests, measured after all five new cases had their isolated execution while excluding the new test target, then those eight plus three new conformance cases, 11 total. The separate related CLI lane executed six tests: two new cases, three existing count CLI cases, and the existing Binary64 publication case. The focused after lanes therefore executed 17 distinct top-level tests. Nested expected refusal processes and repeated isolated runs are excluded from that count. No full workspace or site gate was run by this pass.

The review covered the exact integration resolutions and known shared admission, runner, Serde, authored, synthesis, web, generated Go and CLI boundaries between parents 242adef716f64473ed525e39cfa007556ed6ea2a and 87d9945051f5b2e6296cea8b4cbb82ad2e50aeb3. It did not repeat a generic whole-product review. The aep-drive adversary charter and repository AGENTS.md were read explicitly.

Five new tests were written before their respective first selected command. All are green now. The first command failed compilation with E0046 because this agent's new UnreachableTarget double omitted required end_scenario. No test executed in that attempt. Only the missing test-double method was added, with a panic if a refused suite reaches teardown; the first semantic execution then passed. The original compiler output and exit 101 are retained below. No production assertion was relaxed and no product failure is claimed from that harness error.

The new conformance cases assert four exact escaped suite issue paths across required/optional leaves and both event step variants; equal refusal through canonical serialization, from_suite, Go emission, run and try_run; zero clock/identity/callback effects; 16 admitted Decimal controls and 48 Binary64 reader refusals across suite majors 1–4; preservation of the legacy unadmitted DTO distinction while both runner methods reject suite/5; and all three model locations surviving authored, synthesis and web refusal.

The new actual CLI case checks eight refusal executions: source model versus original suite, report formats 1/2, absent versus existing destination. The new generated Go case has two positive controls that reach its factory and publish the selected report format, plus eight injected Binary64 refusals covering required/optional leaves, formats 1/2 and both destination states. All native refusal cases assert no factory marker and no created/replaced report, with expected actual process exit 1. The parent Rust tests pass when these expected refusals hold.

Actual environment overrides for all Cargo commands, with cwd the frozen worktree:

```text
CARGO_TARGET_DIR=$WORKTREE/target
CARGO_BUILD_JOBS=4
CARGO_INCREMENTAL=0
TMPDIR=$SCRATCH
ESS_GO_COMPILER=/usr/bin/go
GOCACHE=$WORKTREE/target/binary64-count-go-cache
```

CARGO_PROFILE_DEV_DEBUG and CARGO_PROFILE_TEST_DEBUG were unset; the existing default debug cache was preserved. Native commands are recorded below and in the added source; Go runs use the explicit compiler, -race, offline module policy, -mod=readonly, GOMAXPROCS=4, and a unit-local cache. The observed compiler is go1.26.5-X:nodwarf5 linux/amd64.

Initial isolated selection: test-double compile error, exit 101, zero tests executed.

Original merged output: `$SCRATCH/first-runner.log`.

```console
cargo test --offline --locked -p ess-conformance --test binary64_count_adversary both_fallible_runners_preserve_all_binary64_issues_before_effects -- --exact --nocapture
   Compiling serde_json v1.0.151
   Compiling serde v1.0.229
   Compiling serde_yaml v0.9.34+deprecated
   Compiling schemars v0.8.22
   Compiling ess-primitives v0.19.0 ($WORKTREE/crates/specify/ess-primitives)
   Compiling ess-domain v0.19.0 ($WORKTREE/crates/specify/ess-domain)
   Compiling ess-compiler v0.19.0 ($WORKTREE/crates/specify/ess-compiler)
   Compiling ess-gen v0.19.0 ($WORKTREE/crates/generate/ess-gen)
   Compiling ess-conformance v0.19.0 ($WORKTREE/crates/verify/ess-conformance)
error[E0046]: not all trait items implemented, missing: `end_scenario`
  --> crates/verify/ess-conformance/tests/binary64_count_adversary.rs:35:1
   |
35 | impl ConformanceTarget for UnreachableTarget {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `end_scenario` in implementation
   |
   = help: implement the missing item: `fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> { todo!() }`

For more information about this error, try `rustc --explain E0046`.
error: could not compile `ess-conformance` (test "binary64_count_adversary") due to 1 previous error
```

Actual command exit: 101.

Corrected isolated runner test: first semantic execution passes.

Original merged output: `$SCRATCH/corrected-runner.log`.

```console
cargo test --offline --locked -p ess-conformance --test binary64_count_adversary both_fallible_runners_preserve_all_binary64_issues_before_effects -- --exact --nocapture
   Compiling ess-conformance v0.19.0 ($WORKTREE/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.42s
     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-58105126dabf55c9)

running 1 test
four located issues retained; run and try_run refused with zero clock/identity/callback effects
test both_fallible_runners_preserve_all_binary64_issues_before_effects ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

```

Actual command exit: 0.

First isolated original-byte/Serde and typed-version case passes.

Original merged output: `$SCRATCH/first-original.log`.

```console
cargo test --offline --locked -p ess-conformance --test binary64_count_adversary original_byte_and_serde_admission_refuse_binary64_across_all_legacy_suite_majors -- --exact --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-58105126dabf55c9)

running 1 test
16 admitted decimal controls and 48 Binary64 reader refusals; both typed runners reject suite/5 before effects
test original_byte_and_serde_admission_refuse_binary64_across_all_legacy_suite_majors ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.01s

```

Actual command exit: 0.

First isolated unified model issue propagation case passes.

Original merged output: `$SCRATCH/first-model.log`.

```console
cargo test --offline --locked -p ess-conformance --test binary64_count_adversary unified_model_issues_survive_authored_synthesis_and_web_boundaries -- --exact --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-58105126dabf55c9)

running 1 test
three model positions retained through unified issues, authored cause, synthesis and prepublication web refusal
test unified_model_issues_survive_authored_synthesis_and_web_boundaries ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

```

Actual command exit: 0.

First isolated actual CLI case passes all eight expected refusal executions.

Original merged output: `$SCRATCH/first-cli.log`.

```console
cargo test --offline --locked -p ess-cli --test binary64_count_adversary cli_binary64_model_and_original_suites_preserve_report_destinations_in_both_formats -- --exact --nocapture
   Compiling ess-conformance v0.19.0 ($WORKTREE/crates/verify/ess-conformance)
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
   Compiling ess-diff v0.19.0 ($WORKTREE/crates/verify/ess-diff)
   Compiling ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.97s
     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-19aa539d44ab3747)

running 1 test
format-1-model-false-existing-false: actual CLI exit exit status: 1; destination preserved
format-1-model-false-existing-true: actual CLI exit exit status: 1; destination preserved
format-1-model-true-existing-false: actual CLI exit exit status: 1; destination preserved
format-1-model-true-existing-true: actual CLI exit exit status: 1; destination preserved
format-2-model-false-existing-false: actual CLI exit exit status: 1; destination preserved
format-2-model-false-existing-true: actual CLI exit exit status: 1; destination preserved
format-2-model-true-existing-false: actual CLI exit exit status: 1; destination preserved
format-2-model-true-existing-true: actual CLI exit exit status: 1; destination preserved
test cli_binary64_model_and_original_suites_preserve_report_destinations_in_both_formats ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.04s

```

Actual command exit: 0.

The first execution's detailed child command/output records follow, copied from their original retained files.

`$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-1-model-false-existing-false.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270" && "$WORKTREE/target/debug/ess" "verify" "conform" "run" "--target" "billing" "--report-format" "1" "--allow-incomplete" "--format" "json" "--suite" "suite.json" "--report-out" "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-1-model-false-existing-false.report.json"
exit exit status: 1
error: $suite: InvalidSuite: Binary64 is not admitted by this conformance suite format at line 1 column 512; 

```

`$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-1-model-false-existing-true.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270" && "$WORKTREE/target/debug/ess" "verify" "conform" "run" "--target" "billing" "--report-format" "1" "--allow-incomplete" "--format" "json" "--suite" "suite.json" "--report-out" "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-1-model-false-existing-true.report.json"
exit exit status: 1
error: $suite: InvalidSuite: Binary64 is not admitted by this conformance suite format at line 1 column 512; 

```

`$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-1-model-true-existing-false.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270" && "$WORKTREE/target/debug/ess" "verify" "conform" "run" "--target" "billing" "--report-format" "1" "--allow-incomplete" "--format" "json" "--path" "model" "--report-out" "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-1-model-true-existing-false.report.json"
exit exit status: 1
error: types.probe.data.Ratio.of: UnsupportedPrimitive: finite Binary64 is not admitted by the current conformance suite and codecs; 

```

`$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-1-model-true-existing-true.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270" && "$WORKTREE/target/debug/ess" "verify" "conform" "run" "--target" "billing" "--report-format" "1" "--allow-incomplete" "--format" "json" "--path" "model" "--report-out" "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-1-model-true-existing-true.report.json"
exit exit status: 1
error: types.probe.data.Ratio.of: UnsupportedPrimitive: finite Binary64 is not admitted by the current conformance suite and codecs; 

```

`$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-2-model-false-existing-false.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270" && "$WORKTREE/target/debug/ess" "verify" "conform" "run" "--target" "billing" "--report-format" "2" "--allow-incomplete" "--format" "json" "--suite" "suite.json" "--report-out" "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-2-model-false-existing-false.report.json"
exit exit status: 1
error: $suite: InvalidSuite: Binary64 is not admitted by this conformance suite format at line 1 column 512; 

```

`$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-2-model-false-existing-true.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270" && "$WORKTREE/target/debug/ess" "verify" "conform" "run" "--target" "billing" "--report-format" "2" "--allow-incomplete" "--format" "json" "--suite" "suite.json" "--report-out" "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-2-model-false-existing-true.report.json"
exit exit status: 1
error: $suite: InvalidSuite: Binary64 is not admitted by this conformance suite format at line 1 column 512; 

```

`$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-2-model-true-existing-false.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270" && "$WORKTREE/target/debug/ess" "verify" "conform" "run" "--target" "billing" "--report-format" "2" "--allow-incomplete" "--format" "json" "--path" "model" "--report-out" "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-2-model-true-existing-false.report.json"
exit exit status: 1
error: types.probe.data.Ratio.of: UnsupportedPrimitive: finite Binary64 is not admitted by the current conformance suite and codecs; 

```

`$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-2-model-true-existing-true.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270" && "$WORKTREE/target/debug/ess" "verify" "conform" "run" "--target" "billing" "--report-format" "2" "--allow-incomplete" "--format" "json" "--path" "model" "--report-out" "$WORKTREE/target/tmp/binary64-count-adversary-cli-1476270/format-2-model-true-existing-true.report.json"
exit exit status: 1
error: types.probe.data.Ratio.of: UnsupportedPrimitive: finite Binary64 is not admitted by the current conformance suite and codecs; 

```

First isolated generated Go case passes both controls and all eight expected refusal executions.

Original merged output: `$SCRATCH/first-go.log`.

```console
cargo test --offline --locked -p ess-cli --test binary64_count_adversary generated_go_binary64_shapes_refuse_before_factory_and_report_publication -- --exact --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-19aa539d44ab3747)

running 1 test
control-1: actual Go exit exit status: 0; control reached factory and published selected report format
format-1-optional-false-existing-false: actual Go exit exit status: 1; factory untouched and destination preserved
format-1-optional-false-existing-true: actual Go exit exit status: 1; factory untouched and destination preserved
format-1-optional-true-existing-false: actual Go exit exit status: 1; factory untouched and destination preserved
format-1-optional-true-existing-true: actual Go exit exit status: 1; factory untouched and destination preserved
control-2: actual Go exit exit status: 0; control reached factory and published selected report format
format-2-optional-false-existing-false: actual Go exit exit status: 1; factory untouched and destination preserved
format-2-optional-false-existing-true: actual Go exit exit status: 1; factory untouched and destination preserved
format-2-optional-true-existing-false: actual Go exit exit status: 1; factory untouched and destination preserved
format-2-optional-true-existing-true: actual Go exit exit status: 1; factory untouched and destination preserved
test generated_go_binary64_shapes_refuse_before_factory_and_report_publication ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 25.41s

```

Actual command exit: 0.

The first execution's detailed child command/output records follow, copied from their original retained files.

`$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/control-1.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-go-1479870" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="1" ESS_REPORT_OUT="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/control-1.report.json" GOCACHE="$WORKTREE/target/tmp/binary64-count-adversary-go-cache" GOFLAGS="" GOMAXPROCS="4" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GUARD_MARKER="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/control-1.marker" "/usr/bin/go" "test" "-v" "-count=1" "-race" "-mod=readonly" "./..." "-run" "^TestGuard$"
exit exit status: 0
=== RUN   TestGuard
    binary64_count_adversary_test.go:7: probe v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestGuard/probe.data/authored/guard
    runtime.go:733: Refuse Binary64 before report publication
=== NAME  TestGuard
    binary64_count_adversary_test.go:7: report: passed, 1 scenario(s), 0 not passed, written to $WORKTREE/target/tmp/binary64-count-adversary-go-1479870/control-1.report.json
--- PASS: TestGuard (0.00s)
    --- PASS: TestGuard/probe.data/authored/guard (0.00s)
PASS
ok  	example.invalid/binary64-count-adversary/essconform	1.029s

```

`$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-1-optional-false-existing-false.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-go-1479870" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="1" ESS_REPORT_OUT="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-1-optional-false-existing-false.report.json" GOCACHE="$WORKTREE/target/tmp/binary64-count-adversary-go-cache" GOFLAGS="" GOMAXPROCS="4" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GUARD_MARKER="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-1-optional-false-existing-false.marker" "/usr/bin/go" "test" "-v" "-count=1" "-race" "-mod=readonly" "./..." "-run" "^TestGuard$"
exit exit status: 1
=== RUN   TestGuard
    binary64_count_adversary_test.go:7: suite admission: probe.data/authored/guard: unknown primitive
--- FAIL: TestGuard (0.00s)
FAIL
FAIL	example.invalid/binary64-count-adversary/essconform	0.024s
FAIL

```

`$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-1-optional-false-existing-true.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-go-1479870" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="1" ESS_REPORT_OUT="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-1-optional-false-existing-true.report.json" GOCACHE="$WORKTREE/target/tmp/binary64-count-adversary-go-cache" GOFLAGS="" GOMAXPROCS="4" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GUARD_MARKER="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-1-optional-false-existing-true.marker" "/usr/bin/go" "test" "-v" "-count=1" "-race" "-mod=readonly" "./..." "-run" "^TestGuard$"
exit exit status: 1
=== RUN   TestGuard
    binary64_count_adversary_test.go:7: suite admission: probe.data/authored/guard: unknown primitive
--- FAIL: TestGuard (0.00s)
FAIL
FAIL	example.invalid/binary64-count-adversary/essconform	0.020s
FAIL

```

`$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-1-optional-true-existing-false.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-go-1479870" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="1" ESS_REPORT_OUT="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-1-optional-true-existing-false.report.json" GOCACHE="$WORKTREE/target/tmp/binary64-count-adversary-go-cache" GOFLAGS="" GOMAXPROCS="4" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GUARD_MARKER="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-1-optional-true-existing-false.marker" "/usr/bin/go" "test" "-v" "-count=1" "-race" "-mod=readonly" "./..." "-run" "^TestGuard$"
exit exit status: 1
=== RUN   TestGuard
    binary64_count_adversary_test.go:7: suite admission: probe.data/authored/guard: unknown primitive
--- FAIL: TestGuard (0.00s)
FAIL
FAIL	example.invalid/binary64-count-adversary/essconform	0.033s
FAIL

```

`$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-1-optional-true-existing-true.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-go-1479870" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="1" ESS_REPORT_OUT="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-1-optional-true-existing-true.report.json" GOCACHE="$WORKTREE/target/tmp/binary64-count-adversary-go-cache" GOFLAGS="" GOMAXPROCS="4" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GUARD_MARKER="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-1-optional-true-existing-true.marker" "/usr/bin/go" "test" "-v" "-count=1" "-race" "-mod=readonly" "./..." "-run" "^TestGuard$"
exit exit status: 1
=== RUN   TestGuard
    binary64_count_adversary_test.go:7: suite admission: probe.data/authored/guard: unknown primitive
--- FAIL: TestGuard (0.00s)
FAIL
FAIL	example.invalid/binary64-count-adversary/essconform	0.025s
FAIL

```

`$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/control-2.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-go-1479870" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/control-2.report.json" GOCACHE="$WORKTREE/target/tmp/binary64-count-adversary-go-cache" GOFLAGS="" GOMAXPROCS="4" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GUARD_MARKER="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/control-2.marker" "/usr/bin/go" "test" "-v" "-count=1" "-race" "-mod=readonly" "./..." "-run" "^TestGuard$"
exit exit status: 0
=== RUN   TestGuard
    binary64_count_adversary_test.go:7: probe v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestGuard/probe.data/authored/guard
    runtime.go:733: Refuse Binary64 before report publication
--- PASS: TestGuard (0.00s)
    --- PASS: TestGuard/probe.data/authored/guard (0.00s)
PASS
ok  	example.invalid/binary64-count-adversary/essconform	1.034s

```

`$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-2-optional-false-existing-false.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-go-1479870" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-2-optional-false-existing-false.report.json" GOCACHE="$WORKTREE/target/tmp/binary64-count-adversary-go-cache" GOFLAGS="" GOMAXPROCS="4" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GUARD_MARKER="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-2-optional-false-existing-false.marker" "/usr/bin/go" "test" "-v" "-count=1" "-race" "-mod=readonly" "./..." "-run" "^TestGuard$"
exit exit status: 1
=== RUN   TestGuard
    binary64_count_adversary_test.go:7: suite admission: probe.data/authored/guard: unknown primitive
--- FAIL: TestGuard (0.00s)
FAIL
FAIL	example.invalid/binary64-count-adversary/essconform	0.018s
FAIL

```

`$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-2-optional-false-existing-true.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-go-1479870" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-2-optional-false-existing-true.report.json" GOCACHE="$WORKTREE/target/tmp/binary64-count-adversary-go-cache" GOFLAGS="" GOMAXPROCS="4" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GUARD_MARKER="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-2-optional-false-existing-true.marker" "/usr/bin/go" "test" "-v" "-count=1" "-race" "-mod=readonly" "./..." "-run" "^TestGuard$"
exit exit status: 1
=== RUN   TestGuard
    binary64_count_adversary_test.go:7: suite admission: probe.data/authored/guard: unknown primitive
--- FAIL: TestGuard (0.00s)
FAIL
FAIL	example.invalid/binary64-count-adversary/essconform	0.022s
FAIL

```

`$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-2-optional-true-existing-false.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-go-1479870" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-2-optional-true-existing-false.report.json" GOCACHE="$WORKTREE/target/tmp/binary64-count-adversary-go-cache" GOFLAGS="" GOMAXPROCS="4" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GUARD_MARKER="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-2-optional-true-existing-false.marker" "/usr/bin/go" "test" "-v" "-count=1" "-race" "-mod=readonly" "./..." "-run" "^TestGuard$"
exit exit status: 1
=== RUN   TestGuard
    binary64_count_adversary_test.go:7: suite admission: probe.data/authored/guard: unknown primitive
--- FAIL: TestGuard (0.00s)
FAIL
FAIL	example.invalid/binary64-count-adversary/essconform	0.010s
FAIL

```

`$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-2-optional-true-existing-true.log`

```console
cd "$WORKTREE/target/tmp/binary64-count-adversary-go-1479870" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-2-optional-true-existing-true.report.json" GOCACHE="$WORKTREE/target/tmp/binary64-count-adversary-go-cache" GOFLAGS="" GOMAXPROCS="4" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" GUARD_MARKER="$WORKTREE/target/tmp/binary64-count-adversary-go-1479870/format-2-optional-true-existing-true.marker" "/usr/bin/go" "test" "-v" "-count=1" "-race" "-mod=readonly" "./..." "-run" "^TestGuard$"
exit exit status: 1
=== RUN   TestGuard
    binary64_count_adversary_test.go:7: suite admission: probe.data/authored/guard: unknown primitive
--- FAIL: TestGuard (0.00s)
FAIL
FAIL	example.invalid/binary64-count-adversary/essconform	0.011s
FAIL

```

Baseline measurement after new-case execution, excluding binary64_count_adversary: eight tests pass.

Original merged output: `$SCRATCH/baseline-after-additions.log`.

```console
cargo test --offline --locked -p ess-conformance --test count_reports --test count_writer_pass1 --test count_writer_pass2 -- --test-threads=1 --nocapture
   Compiling ess-conformance v0.19.0 ($WORKTREE/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.44s
     Running tests/count_reports.rs (target/debug/deps/count_reports-084a0ab749d33e48)

running 6 tests
test actual_rust_producer_pairs_preserve_categories_precedence_empty_and_high_u64 ... ok
test detailed_admission_checks_fields_outcome_order_and_checked_time ... ok
test exact_bytes_profiles_partition_and_identity_cannot_be_guessed ... ok
test legacy_dto_is_not_original_byte_admission_and_typed_execution_still_checks_versions ... ok
test new_scalar_tokens_are_exact_unsigned_in_both_surfaces ... ok
test suite_admission_closes_structural_variants_before_target_identity ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-0ea168b971a96534)

running 1 test
test a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-29914f034431523e)

running 1 test
test cloned_execution_binding_survives_mutation_of_extracted_legacy_diagnostics ... original digest: sha256:272282e3693f614c1f5b8c3655f63cca06c4fe19230fdf299ba2c35dbc1a10ad; escaped digest: sha256:d0138164a373c3082ac3c159aef72d8df71774dfe9b1e74dfc799f99ce1e365b; immutable retained run remains passed
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Actual command exit: 0.

Related conformance lane: three new plus eight existing tests, 11 pass.

Original merged output: `$SCRATCH/related-conformance.log`.

```console
cargo test --offline --locked -p ess-conformance --test count_reports --test count_writer_pass1 --test count_writer_pass2 --test binary64_count_adversary -- --test-threads=1 --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-58105126dabf55c9)

running 3 tests
test both_fallible_runners_preserve_all_binary64_issues_before_effects ... four located issues retained; run and try_run refused with zero clock/identity/callback effects
ok
test original_byte_and_serde_admission_refuse_binary64_across_all_legacy_suite_majors ... 16 admitted decimal controls and 48 Binary64 reader refusals; both typed runners reject suite/5 before effects
ok
test unified_model_issues_survive_authored_synthesis_and_web_boundaries ... three model positions retained through unified issues, authored cause, synthesis and prepublication web refusal
ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/count_reports.rs (target/debug/deps/count_reports-084a0ab749d33e48)

running 6 tests
test actual_rust_producer_pairs_preserve_categories_precedence_empty_and_high_u64 ... ok
test detailed_admission_checks_fields_outcome_order_and_checked_time ... ok
test exact_bytes_profiles_partition_and_identity_cannot_be_guessed ... ok
test legacy_dto_is_not_original_byte_admission_and_typed_execution_still_checks_versions ... ok
test new_scalar_tokens_are_exact_unsigned_in_both_surfaces ... ok
test suite_admission_closes_structural_variants_before_target_identity ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-0ea168b971a96534)

running 1 test
test a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-29914f034431523e)

running 1 test
test cloned_execution_binding_survives_mutation_of_extracted_legacy_diagnostics ... original digest: sha256:272282e3693f614c1f5b8c3655f63cca06c4fe19230fdf299ba2c35dbc1a10ad; escaped digest: sha256:d0138164a373c3082ac3c159aef72d8df71774dfe9b1e74dfc799f99ce1e365b; immutable retained run remains passed
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Actual command exit: 0.

Related CLI lane: six tests pass, including all 18 existing Binary64 publication route checks.

Original merged output: `$SCRATCH/related-cli.log`.

```console
cargo test --offline --locked -p ess-cli --test binary64_count_adversary --test count_reports --test binary64_publication -- --test-threads=1 --nocapture
   Compiling ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.28s
     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-19aa539d44ab3747)

running 2 tests
test cli_binary64_model_and_original_suites_preserve_report_destinations_in_both_formats ... format-1-model-false-existing-false: actual CLI exit exit status: 1; destination preserved
format-1-model-false-existing-true: actual CLI exit exit status: 1; destination preserved
format-1-model-true-existing-false: actual CLI exit exit status: 1; destination preserved
format-1-model-true-existing-true: actual CLI exit exit status: 1; destination preserved
format-2-model-false-existing-false: actual CLI exit exit status: 1; destination preserved
format-2-model-false-existing-true: actual CLI exit exit status: 1; destination preserved
format-2-model-true-existing-false: actual CLI exit exit status: 1; destination preserved
format-2-model-true-existing-true: actual CLI exit exit status: 1; destination preserved
ok
test generated_go_binary64_shapes_refuse_before_factory_and_report_publication ... control-1: actual Go exit exit status: 0; control reached factory and published selected report format
format-1-optional-false-existing-false: actual Go exit exit status: 1; factory untouched and destination preserved
format-1-optional-false-existing-true: actual Go exit exit status: 1; factory untouched and destination preserved
format-1-optional-true-existing-false: actual Go exit exit status: 1; factory untouched and destination preserved
format-1-optional-true-existing-true: actual Go exit exit status: 1; factory untouched and destination preserved
control-2: actual Go exit exit status: 0; control reached factory and published selected report format
format-2-optional-false-existing-false: actual Go exit exit status: 1; factory untouched and destination preserved
format-2-optional-false-existing-true: actual Go exit exit status: 1; factory untouched and destination preserved
format-2-optional-true-existing-false: actual Go exit exit status: 1; factory untouched and destination preserved
format-2-optional-true-existing-true: actual Go exit exit status: 1; factory untouched and destination preserved
ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.80s

     Running tests/binary64_publication.rs (target/debug/deps/binary64_publication-c1cc38de9934ec2f)

running 1 test
test binary64_sparse_model_refuses_every_unsupported_publication_route ... 18 actual CLI refusal executions preserved absent/existing destinations
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/count_reports.rs (target/debug/deps/count_reports-7b4969127d037246)

running 3 tests
test count_cli_configuration_and_original_suite_refusals_preserve_destinations ... ok
test count_cli_preserves_default_bytes_and_standalone_detailed_pairing ... ok
test count_report_opt_in_has_a_distinct_detailed_surface_and_unknown_coverage ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s

```

Actual command exit: 0.

Clippy restricted to the two new integration-test targets, warnings denied; exit 0.

Original merged output: `$SCRATCH/added-test-clippy.log`.

```console
cargo clippy --offline --locked -p ess-conformance -p ess-cli --test binary64_count_adversary -- -D warnings
    Checking ess-conformance v0.19.0 ($WORKTREE/crates/verify/ess-conformance)
    Checking schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Checking ess-diff v0.19.0 ($WORKTREE/crates/verify/ess-diff)
    Checking ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.47s
```

Actual command exit: 0.

Rustfmt check of the two added test files; exit 0, empty stdout.

Original merged output: `$SCRATCH/fmt-check.log`.

```console
rustfmt --edition 2021 --check crates/edge/ess-cli/tests/binary64_count_adversary.rs crates/verify/ess-conformance/tests/binary64_count_adversary.rs
```

Actual command exit: 0.

Findings table: none. There is no introduced, pre-existing or undecided product finding, and no approval or independence claim.

What was attacked and did not break:

- Unified admission retains every Binary64 issue and escaped pointer through both fallible runners before any clock or target effect.
- Both original-byte admission and direct Serde refuse the new primitive under all current suite majors, including optional leaves; legacy DTO parsing remains distinct from admitted execution.
- Model refusals survive conversion into authored causes and prevent sparse synthesis/web success.
- CLI report formats 1 and 2 cannot create or replace destinations after either source-model or original-suite Binary64 refusal.
- Generated Go rejects injected Binary64 before factory construction; positive controls show that the same harness can reach the factory and publish both report formats.
- Existing exact-execution digest rebinding/clone tests, count categories and exact unsigned values, CLI default report bytes, strict/diagnostic behavior and all 18 prior Binary64 publication routes still pass in the combined tree.

The cases reach actual public ConformanceSuite/AdmittedSuite/Runner and emitter APIs, compiled authored model acquisition, the actual ess CLI, and the emitted Go Run entry point. Injected suites are retained original input documents supplied to the generated runner; the positive controls establish that the refusal assertions did not merely select no test.

Full source-content proof:

- `$SCRATCH/frozen-files.json`: SHA-256 `2981c3c61013f3245c4fcc5ecfb742ba10e7497dc6c9eb311de08dc1f31d3af3`.
- `$SCRATCH/after-source.json`: SHA-256 `2981c3c61013f3245c4fcc5ecfb742ba10e7497dc6c9eb311de08dc1f31d3af3`.
- `$SCRATCH/added-tests-manifest.json`: SHA-256 `d7d763f178b35e216c6da720bd58c7c446bffdce25a857012a35dd0728c78b89`.
- `$SCRATCH/added-tests.patch`: SHA-256 `8c5c02a48aaabd035327574dc68274467983d20a147e92c0764eea6a8b04cc16`.
- `$SCRATCH/source-content-proof.json`: SHA-256 `beb0d560265340a4afe52ed8ea90eafe3157ec34af74371809cbf329244c0d7f`.
- `$SCRATCH/native-cli-log-manifest.json`: SHA-256 `9cc0643523638c4431caf1dab19472218077dd453e5806f985ae0eb6e9c03e10`.

All 957 original tracked files were checked against the supplied frozen manifest before test edits and after all commands. Content hashes and permission modes are unchanged; the after manifest is byte identical to the frozen manifest. Only the two new test files differ from the frozen tree and are separately hashed above. No source, existing test, documentation, planning or Git mutation occurred. The 36 detailed child process logs from isolated and related executions are retained under the unit target and individually hashed in native-cli-log-manifest.json.

All invoked sessions and native children are terminal. Final observation found no active unit Cargo/rustc/Go/native-test process. Existing target usage is 30816636928 bytes; final available space is 30552416256 bytes, above the 8589934592-byte reserve. No build directory was removed. The exclusive root-team Cargo/native slot is released on this report handoff.

Every retained outside write follows. frozen-files.json was coordinator-supplied and only read, so it is excluded from the write count. Existing count CLI regressions wrote their two fixture directories beneath the assigned TMPDIR; those individual files are included. Compiler-created temporary children stayed under the assigned TMPDIR and were removed by their tools; their transient leaf names were not individually inventoried. All generated adapters and native caches stayed under this worktree target.

```text
$SCRATCH/added-test-clippy-command.json
$SCRATCH/added-test-clippy.log
$SCRATCH/added-tests-manifest.json
$SCRATCH/added-tests-stat.txt
$SCRATCH/added-tests.patch
$SCRATCH/after-source.json
$SCRATCH/baseline-after-additions-command.json
$SCRATCH/baseline-after-additions.log
$SCRATCH/before-proof.json
$SCRATCH/corrected-runner-command.json
$SCRATCH/corrected-runner.log
$SCRATCH/ess-count-cli-refusals-1495257/report.json
$SCRATCH/ess-count-cli-refusals-1495257/suite.json
$SCRATCH/ess-count-cli-surfaces-1495257/report-json.json
$SCRATCH/ess-count-cli-surfaces-1495257/report-yaml.json
$SCRATCH/ess-count-cli-surfaces-1495257/suite.json
$SCRATCH/final-environment.json
$SCRATCH/first-cli-command.json
$SCRATCH/first-cli.log
$SCRATCH/first-go-command.json
$SCRATCH/first-go.log
$SCRATCH/first-model-command.json
$SCRATCH/first-model.log
$SCRATCH/first-original-command.json
$SCRATCH/first-original.log
$SCRATCH/first-runner-command.json
$SCRATCH/first-runner.log
$SCRATCH/fmt-check-command.json
$SCRATCH/fmt-check.log
$SCRATCH/native-cli-log-manifest.json
$SCRATCH/outside-paths.txt
$SCRATCH/private-report.md
$SCRATCH/public-report.md
$SCRATCH/related-cli-command.json
$SCRATCH/related-cli.log
$SCRATCH/related-conformance-command.json
$SCRATCH/related-conformance.log
$SCRATCH/report-packet-manifest.json
$SCRATCH/source-content-proof.json
```

```findings
[]
```