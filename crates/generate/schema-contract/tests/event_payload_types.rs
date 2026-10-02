//! Existing event records share the structural plan and its actual target codecs.
use std::{collections::BTreeSet, fs, path::PathBuf, process::Command};

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::schema::{ModelRoot, ModelTypes};
use schema_contract::realize::Plan;
use serde_json::{json, Value};

fn plan() -> Plan {
    let text = include_str!("../../ess-gen/tests/fixtures/model-event-types.yaml");
    let spec =
        Specification::assemble([(Source::document(), RawSpecFile::parse(text).unwrap())]).unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let selected = ModelTypes::select_roots(
        &ir,
        &BTreeSet::from([ModelRoot::Event("telemetry.data.Recorded".parse().unwrap())]),
    )
    .unwrap();
    Plan::from_model(&selected).unwrap()
}

fn directory(target: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("event-types-{target}-{}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}

fn success(command: &mut Command) {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?}:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn event_reports_keep_typed_roots_and_runtime_obligations() {
    let plan = plan();
    let report: Value = serde_json::to_value(plan.typescript().report).unwrap();
    assert_eq!(report["format"], "ess-types-report/4");
    assert_eq!(
        report["model_roots"],
        json!([{"kind":"event", "name":"telemetry.data.Recorded"}])
    );
    assert_eq!(report["roots"], json!(["telemetry.data.Recorded"]));
    for rule in [
        "model_binary64",
        "json_number_precision",
        "typescript_nominal_identity",
    ] {
        assert!(
            report["obligations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["rule"] == rule),
            "{rule}: {report}"
        );
    }
    assert_eq!(plan.declarations().len(), 3);
}

#[test]
fn generated_event_rust_codecs_round_trip() {
    let emitted = plan().rust("event_data").unwrap();
    let root = directory("rust");
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("Cargo.toml"), &emitted.supporting["Cargo.toml"]).unwrap();
    fs::write(root.join("types.rs"), emitted.declarations).unwrap();
    fs::write(root.join("tests/wire.rs"), r##"
use event_data::*;
#[test]
fn actual_event_wire() {
    let raw = r#"{"record-id":"sensor","details":{"sequence":9007199254740993,"title":"reading","amount":-2147483648},"samples":[false,null,true],"weight":-0.0,"measurements":{"first":null,"second":1.25}}"#;
    let value: TelemetryDataRecorded = serde_json::from_str(raw).unwrap();
    assert!(matches!(value.label, EssPresence::Absent));
    let output = serde_json::to_string(&value).unwrap();
    assert!(output.contains("9007199254740993"));
    assert!(output.contains("\"weight\":-0.0"));
    let expected: serde_json::Value = serde_json::from_str(raw).unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(&output).unwrap(), expected);
    for suffix in [",\"label\":null", ",\"unknown\":true"] {
        let bad = format!("{}{}{}", &raw[..raw.len()-1], suffix, "}");
        assert!(serde_json::from_str::<TelemetryDataRecorded>(&bad).is_err());
    }
    assert!(serde_json::from_str::<TelemetryDataRecorded>(&raw.replace("\"record-id\":\"sensor\",", "")).is_err());
    let present = raw.replace("\"samples\"", "\"label\":\"present\",\"samples\"");
    let present: TelemetryDataRecorded = serde_json::from_str(&present).unwrap();
    assert!(matches!(present.label, EssPresence::Present(_)));
    for amount in ["-2147483648", "2147483647"] {
        let decoded: TelemetryDataRecorded = serde_json::from_str(&raw.replace("-2147483648", amount)).unwrap();
        assert!(serde_json::to_string(&decoded).unwrap().contains(amount));
    }
    for amount in ["-2147483649", "2147483648", "0.5", "null"] {
        assert!(serde_json::from_str::<TelemetryDataRecorded>(&raw.replace("-2147483648", amount)).is_err());
    }
    for sequence in ["-9223372036854775808", "9223372036854775807", "9007199254740993"] {
        let decoded: TelemetryDataRecorded = serde_json::from_str(&raw.replace("9007199254740993", sequence)).unwrap();
        assert!(serde_json::to_string(&decoded).unwrap().contains(sequence));
    }
    for sequence in ["-9223372036854775809", "9223372036854775808", "0.5", "null"] {
        assert!(serde_json::from_str::<TelemetryDataRecorded>(&raw.replace("9007199254740993", sequence)).is_err());
    }
}
"##).unwrap();
    success(
        Command::new(env!("CARGO"))
            .args(["test", "--offline", "--quiet", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .env(
                "CARGO_TARGET_DIR",
                PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("event-types-rust-target"),
            ),
    );
}

#[cfg(feature = "go-typecheck")]
#[test]
fn generated_event_go_codecs_round_trip() {
    let compiler =
        std::env::var_os("ESS_GO_COMPILER").expect("go-typecheck requires ESS_GO_COMPILER");
    let emitted = plan().go("eventdata", "example.invalid/eventdata").unwrap();
    let root = directory("go");
    fs::write(root.join("go.mod"), &emitted.supporting["go.mod"]).unwrap();
    fs::write(root.join("types.go"), emitted.declarations).unwrap();
    fs::write(root.join("wire_test.go"), r#"package eventdata
import ("encoding/json"; "math"; "strings"; "testing")
func TestActualEventWire(t *testing.T) {
    raw := `{"record-id":"sensor","details":{"sequence":9007199254740993,"title":"reading","amount":-2147483648},"samples":[false,null,true],"weight":-0.0,"measurements":{"first":null,"second":1.25}}`
    var value TelemetryDataRecorded
    if err := json.Unmarshal([]byte(raw), &value); err != nil { t.Fatal(err) }
    if value.Label.Present { t.Fatal("optional absence lost") }
    if !math.Signbit(value.Weight.Float64()) { t.Fatal("negative zero lost") }
    output, err := json.Marshal(value); if err != nil { t.Fatal(err) }
    if !strings.Contains(string(output), "9007199254740993") { t.Fatal("integer rounded", string(output)) }
    var again TelemetryDataRecorded
    if err := json.Unmarshal(output, &again); err != nil { t.Fatal(err) }
    for _, suffix := range []string{`,"label":null`, `,"unknown":true`} {
        bad := raw[:len(raw)-1]+suffix+"}"
        if json.Unmarshal([]byte(bad), &again) == nil { t.Fatal("invalid event accepted", bad) }
    }
    if json.Unmarshal([]byte(strings.Replace(raw, `"record-id":"sensor",`, "", 1)), &again) == nil { t.Fatal("missing wire key accepted") }
    present := strings.Replace(raw, `"samples"`, `"label":"present","samples"`, 1)
    if err := json.Unmarshal([]byte(present), &again); err != nil { t.Fatal(err) }
    if !again.Label.Present { t.Fatal("present optional lost") }
    for _, amount := range []string{"-2147483648", "2147483647"} {
        if err := json.Unmarshal([]byte(strings.Replace(raw, "-2147483648", amount, 1)), &again); err != nil { t.Fatal(err) }
        out, err := json.Marshal(again); if err != nil || !strings.Contains(string(out), amount) { t.Fatal("i32 boundary changed", string(out), err) }
    }
    for _, amount := range []string{"-2147483649", "2147483648", "0.5", "null"} {
        if json.Unmarshal([]byte(strings.Replace(raw, "-2147483648", amount, 1)), &again) == nil { t.Fatal("invalid i32 accepted", amount) }
    }
    for _, sequence := range []string{"-9223372036854775808", "9223372036854775807", "9007199254740993"} {
        if err := json.Unmarshal([]byte(strings.Replace(raw, "9007199254740993", sequence, 1)), &again); err != nil { t.Fatal(err) }
        out, err := json.Marshal(again); if err != nil || !strings.Contains(string(out), sequence) { t.Fatal("i64 boundary changed", string(out), err) }
    }
    for _, sequence := range []string{"-9223372036854775809", "9223372036854775808", "0.5", "null"} {
        if json.Unmarshal([]byte(strings.Replace(raw, "9007199254740993", sequence, 1)), &again) == nil { t.Fatal("invalid i64 accepted", sequence) }
    }
}
"#).unwrap();
    success(
        Command::new(compiler)
            .args(["test", "./..."])
            .current_dir(root)
            .env("GOTOOLCHAIN", "local")
            .env("GOPROXY", "off")
            .env("GOSUMDB", "off")
            .env("GOWORK", "off")
            .env("GOFLAGS", ""),
    );
}

#[cfg(feature = "typescript-typecheck")]
#[test]
fn generated_event_typescript_compiles_with_exact_presence() {
    let compiler = std::env::var_os("ESS_TYPESCRIPT_COMPILER")
        .expect("typescript-typecheck requires ESS_TYPESCRIPT_COMPILER");
    let root = directory("typescript");
    let mut source = plan().typescript().declarations;
    source.push_str(r#"
const record: TelemetryDataRecorded = {"record-id":"sensor", details:{sequence:1,title:"reading",amount:0},samples:[false,null,true],weight:-0,measurements:{first:null,second:1.25}};
const labelled: TelemetryDataRecorded = {...record,label:"present"};
// @ts-expect-error optional field does not admit null
const nullLabel: TelemetryDataRecorded = {...record,label:null};
// @ts-expect-error optional field does not admit explicit undefined
const undefinedLabel: TelemetryDataRecorded = {...record,label:undefined};
// @ts-expect-error wire name must be used
const wrongWire: TelemetryDataRecorded = {id:"sensor",details:record.details,samples:[],weight:0,measurements:{}};
// @ts-expect-error nested record remains typed
const wrongNested: TelemetryDataRecorded = {...record,details:{sequence:"1",title:"reading",amount:0}};
"#);
    fs::write(root.join("types.ts"), source).unwrap();
    fs::write(root.join("tsconfig.json"), json!({"compilerOptions":{"strict":true,"exactOptionalPropertyTypes":true,"noEmit":true,"types":[],"target":"ES2022","module":"ESNext"},"files":["types.ts"]}).to_string()).unwrap();
    success(
        Command::new("node")
            .arg(compiler)
            .args(["--pretty", "false", "--project"])
            .arg(root.join("tsconfig.json")),
    );
}
