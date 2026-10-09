//! The same Rust display represents complete exact declarations before and after runtime Load.
mod support_scratch;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    web_execution::bundle::{self, Execution, Loaded, SourceDocument},
    AdmittedSuite, ConformanceSuite, SuiteProvenance,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
const SOURCE: &str = r"format: ess/4
system: demo
version: v1
domain: demo.api
events:
  - name: demo.api.Got
    fields: [{name: value, type: Integer}]
commands:
  - name: demo.api.Get
    response: [{name: value, type: Integer}]
    outcomes:
      - name: found
        emits: [demo.api.Got]
        payload:
          demo.api.Got:
            value: {response: value}
";
const AUTHORED: &str = r"type: ess-scenario/4
domain: demo.api
scenario: exact
summary: Exact response declaration.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.api.Get
    outcome: found
    response: {value: 9007199254740993}
    events:
      - event: demo.api.Got
        payload: {value: 9007199254740993}
";
fn emitted() -> (String, Vec<bundle::Blob>) {
    let spec =
        Specification::assemble([(Source::new("model"), RawSpecFile::parse(SOURCE).unwrap())])
            .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let authored = ess_conformance::authored::compile(
        &ir,
        &[ess_conformance::authored::Source::new(
            "exact.yaml",
            AUTHORED,
        )],
    );
    assert!(authored.is_complete(), "{:?}", authored.refusals);
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(&ir));
    for (id, scenario) in authored.scenarios {
        suite.insert(id, scenario).unwrap();
    }
    suite.select_fresh_format_for(&ir);
    bundle::create(
        &[SourceDocument {
            path: "sources/0000.yaml".into(),
            text: SOURCE.into(),
        }],
        &Execution::Ordinary(AdmittedSuite::from_suite(&suite).unwrap()),
    )
    .unwrap()
}
#[test]
fn numeric_payloads_are_text_and_original_suite_bytes_never_round_trip_through_display() {
    fn numeric(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::Array(v) => v.iter().any(numeric),
            serde_json::Value::Object(v) => v.values().any(numeric),
            serde_json::Value::Number(_) => true,
            _ => false,
        }
    }
    let (manifest, blobs) = emitted();
    let loaded = Loaded::admit(&manifest, blobs.clone()).unwrap();
    assert!(loaded
        .selected()
        .original_json()
        .contains("9007199254740993"));
    let display: serde_json::Value = serde_json::from_str(loaded.presentation()).unwrap();
    assert!(!numeric(&display["model"]));
    assert!(!numeric(&display["suite"]));
    assert!(!numeric(&display["scenarios"]));
    assert!(loaded.presentation().contains("\"9007199254740993\""));
    assert!(!loaded.presentation().contains("9007199254740992"));
    assert_eq!(
        loaded.presentation().as_bytes(),
        blobs.last().unwrap().bytes
    );
    assert_eq!(emitted().0, manifest);
}
#[test]
fn a_forged_display_with_a_matching_file_hash_still_cannot_issue_a_loaded_capability() {
    let (manifest, mut blobs) = emitted();
    let display = blobs.last_mut().unwrap();
    display.bytes = String::from_utf8(display.bytes.clone())
        .unwrap()
        .replace("9007199254740993", "9007199254740992")
        .into_bytes();
    let mut metadata: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    metadata["presentation"]["sha256"] = bundle::hash(&display.bytes).into();
    assert!(Loaded::admit(&metadata.to_string(), blobs).is_err());
}

#[test]
fn source_labels_cannot_overwrite_the_product_manifest_or_fixed_assets() {
    let (_, blobs) = emitted();
    let original_suite = std::str::from_utf8(&blobs[1].bytes).unwrap();
    for label in [
        "sources/control.yaml",
        "browser.json",
        "index.html",
        "player.js",
        "worker.js",
        "README.md",
        "rust/browser_host.rs",
        "rust/Cargo.toml.example",
        "rust/lib.rs.example",
        "assets/vue.esm-browser.prod.js",
        "assets/vue.LICENSE",
    ] {
        let result = ess_conformance::web::emit_product(
            &[SourceDocument {
                path: label.into(),
                text: SOURCE.into(),
            }],
            &Execution::Ordinary(AdmittedSuite::from_json(original_suite).unwrap()),
        );
        if label == "sources/control.yaml" {
            let files = result.unwrap();
            assert_eq!(files[label].contents, SOURCE);
            assert!(files["browser.json"]
                .contents
                .contains("ess-conformance-browser/1"));
        } else {
            assert_eq!(
                result.err(),
                Some(ess_conformance::web_execution::Error::InvalidBundle),
                "{label}"
            );
        }
    }
}

const DEPTH_PROBE: &str = r#"
import {pathToFileURL} from 'node:url';
const {parseControl,validatePresentation}=await import(pathToFileURL(process.argv[1]).href);
const encode=new TextEncoder(), rows=[];
function status(call){try{call();return 'ok';}catch(error){return error.message;}}
for(const shape of ['object','list'])for(const position of ['model','scenario'])for(const depth of [0,128,340,341,1024,1025]){
    let value={kind:'null'};
    for(let index=0;index<depth;index++)value=shape==='object'?{kind:'object',value:[['entry',value]]}:{kind:'list',value:[value]};
    const doc={format:'ess-conformance-browser-presentation/1',selected_digest:'sha256:'+'0'.repeat(64),parent_digests:[],
        sources:[{path:'sources/source.yaml',sha256:'0'.repeat(64),byte_length:0}],model:{kind:'null'},suite:{kind:'null'},scenarios:[]};
    if(position==='model')doc.model=value;else doc.scenarios=[{id:'depth/scenario',declaration:value}];
    const bytes=encode.encode(JSON.stringify(doc));
    rows.push({shape,position,depth,bytes:bytes.length,typed:status(()=>validatePresentation(doc)),
        parsed:status(()=>validatePresentation(parseControl(bytes)))});
}
rows.push({control:'duplicate-key',result:status(()=>parseControl(encode.encode('{"format":"x","\\u0066ormat":"y"}')))});
rows.push({control:'physical-depth',result:status(()=>parseControl(encode.encode('['.repeat(3077)+'0'+']'.repeat(3077))))});
console.log(JSON.stringify(rows));
"#;

#[test]
fn display_wrapper_depth_does_not_reduce_the_logical_value_budget() {
    use std::{fs, process::Command};
    let root = support_scratch::Scratch::adopt(
        std::env::temp_dir().join(format!("ess-browser-display-depth-{}", std::process::id())),
    );
    fs::create_dir_all(&root).unwrap();
    let module = root.join("player.mjs");
    let asset = include_str!("../assets/browser-player.js");
    fs::write(&module, asset).unwrap();
    let output = Command::new("node")
        .args(["--input-type=module", "--eval", DEPTH_PROBE])
        .arg(&module)
        .output()
        .unwrap();
    fs::write(root.join("depth.stdout"), &output.stdout).unwrap();
    fs::write(root.join("depth.stderr"), &output.stderr).unwrap();
    fs::write(root.join("asset.sha256"), bundle::hash(asset.as_bytes())).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 26);
    for row in rows.as_array().unwrap() {
        if let Some(depth) = row["depth"].as_u64() {
            let expected = if depth <= 1024 {
                "ok"
            } else {
                "resource_limit"
            };
            assert_eq!(row["typed"], expected, "{row}");
            assert_eq!(
                row["parsed"], expected,
                "BROWSER_DISPLAY_DEPTH_WRAPPER_MISMATCH: {row}"
            );
        } else {
            let expected = if row["control"] == "duplicate-key" {
                "invalid_bundle"
            } else {
                "resource_limit"
            };
            assert_eq!(row["result"], expected, "{row}");
        }
    }
}
