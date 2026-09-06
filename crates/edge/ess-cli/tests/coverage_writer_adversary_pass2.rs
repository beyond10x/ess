//! Original-byte replay admission regressions; no claim about full player fidelity.
#[path = "support/browser.rs"]
mod browser;

use ess_conformance::{
    coverage::{AdmittedInput, SuiteReference},
    web_replay::AdmittedReplay,
    AdmittedSuite,
};
use serde_json::{json, Value};
use std::{fs, path::Path, process::Command};

#[test]
fn browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner() {
    let evidence = std::env::temp_dir().join(format!(
        "ess-coverage-review2-node-keys-{}",
        std::process::id()
    ));
    fs::create_dir_all(&evidence).unwrap();
    let generated = evidence.join("site");
    let original = emit_replay(&evidence, &generated);
    let mut replay: Value = serde_json::from_str(&original).unwrap();
    let mut suite: Value =
        serde_json::from_str(replay["input"]["suite_json"].as_str().unwrap()).unwrap();
    let id = suite["scenarios"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone();
    // These are loaded-suite wire inputs at declared Node owners. Model pairing checks declared
    // identity; it does not claim these test-authored steps were synthesized from the model.
    let payload = json!({"__proto__":{"sentinel":"retained"},"constructor":{"prototype":9},
        "toString":"ordinary data","hasOwnProperty":false,"\u{0000}":[null,2],
        "\u{e000}":"BMP key","\u{1f600}":"astral key"});
    let owners = payload_owners(&payload);
    suite["scenarios"][&id]["steps"] =
        json!(owners.iter().map(|(step, _)| step).collect::<Vec<_>>());
    let parent =
        AdmittedInput::from_suite(AdmittedSuite::from_json(&suite.to_string()).unwrap()).unwrap();
    let selected = parent.select(&[id.parse().unwrap()]).unwrap();
    replay["input"] =
        serde_json::from_str(&selected.document().to_canonical_json().unwrap()).unwrap();
    replay["suite"] = serde_json::to_value(SuiteReference::of(selected.selected())).unwrap();
    AdmittedReplay::from_json(&replay.to_string()).unwrap();
    let paths: Vec<_> = owners
        .iter()
        .enumerate()
        .map(|(index, (_, path))| format!("/{index}{path}"))
        .collect();
    let mut cases = vec![
        json!({"name":"all-node-owners-preserved","replay":replay.to_string(),"accepted":true}),
    ];
    for (index, (_, path)) in owners.iter().enumerate() {
        let mut changed = replay.clone();
        let mut child: Value =
            serde_json::from_str(changed["input"]["suite_json"].as_str().unwrap()).unwrap();
        child["scenarios"][&id]["steps"][index]
            .pointer_mut(path)
            .unwrap()["__proto__"]["sentinel"] = json!("changed child");
        let original_child = child.to_string();
        // Hash exact changed bytes so this exercises full survivor equality, not a stale digest.
        changed["suite"]["digest"] = hash_child(&evidence, index, &original_child);
        changed["input"]["suite_json"] = json!(original_child);
        let refusal = AdmittedReplay::from_json(&changed.to_string()).unwrap_err();
        cases.push(json!({"name":format!("changed-owner-{index}"),"replay":changed.to_string(),"accepted":false,"rust_refusal":refusal.to_string()}));
    }
    fs::write(
        generated.join("node-cases.json"),
        json!({"cases":cases,"id":id,"paths":paths,"payload":payload}).to_string(),
    )
    .unwrap();
    let server = browser::Server::new(&generated);
    let mut firefox = browser::Browser::new(&evidence);
    let context = firefox.open(&format!("{}/index.html", server.url));
    let observed = firefox.evaluate(&context, r"(async () => {
      const {admitReplay}=await import('./admission.js');
      const inputs=await fetch('node-cases.json').then(r=>r.json());
      const results=[];
      for(const test of inputs.cases) {
        try {
          const admitted=await admitReplay(test.replay);
          const steps=admitted.suite.scenarios[inputs.id].steps;
          const values=inputs.paths.map(path=>path.slice(1).split('/').reduce((value,key)=>value[key],steps));
          results.push({name:test.name,admitted:true,values,ownProto:values.map(value=>Object.hasOwn(value,'__proto__'))});
        } catch(error) {results.push({name:test.name,admitted:false,error:String(error)});}
      }
      return JSON.stringify(results);
    })()");
    fs::write(
        evidence.join("firefox-results.json"),
        serde_json::to_string_pretty(&observed).unwrap(),
    )
    .unwrap();
    assert_eq!(observed.as_array().unwrap().len(), cases.len());
    for (expected, actual) in cases.iter().zip(observed.as_array().unwrap()) {
        assert_eq!(actual["name"], expected["name"]);
        assert_eq!(actual["admitted"], expected["accepted"], "{actual}");
    }
    assert_payload_projection(&observed, &payload);
    assert!(observed[0]["ownProto"]
        .as_array()
        .unwrap()
        .iter()
        .all(|value| value == true));
    println!("actual Firefox: 12 Node owners preserve all 7 arbitrary keys; 12 changed survivor payloads refused");
}

fn assert_payload_projection(observed: &Value, payload: &Value) {
    let values = observed[0]["values"].as_array().unwrap();
    assert_eq!(values.len(), 12);
    // Full lineage admission normalizes every owner. The reduced player does not consume
    // nested view expectations, whose returned representation still preserves numeric tokens.
    let mut unused_view_payload = payload.clone();
    unused_view_payload["constructor"]["prototype"] = json!({"raw":"9.0"});
    unused_view_payload["\u{0000}"][1] = json!({"raw":"2.0"});
    for (index, value) in values.iter().enumerate() {
        let expected = if matches!(index, 6 | 7 | 9) {
            &unused_view_payload
        } else {
            payload
        };
        assert_eq!(
            value, expected,
            "owner {index}: story:review-browser-replay-fidelity must update the exact current \
             view projection assertion when supported view execution consumes these fields; \
             retain every Node admission and arbitrary-key check"
        );
    }
}

fn emit_replay(evidence: &Path, generated: &Path) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut emit = Command::new(env!("CARGO_BIN_EXE_ess"));
    emit.args(["conform", "web", "--path"])
        .arg(root.join("examples/billing"))
        .arg("--scenarios")
        .arg(root.join("examples/billing-scenarios"))
        .args(["--suite-format", "5", "--out"])
        .arg(generated);
    fs::write(evidence.join("emit.command"), format!("{emit:?}\n")).unwrap();
    let output = emit.output().unwrap();
    fs::write(evidence.join("emit.stdout"), &output.stdout).unwrap();
    fs::write(evidence.join("emit.stderr"), &output.stderr).unwrap();
    fs::write(
        evidence.join("emit.exit"),
        format!("{:?}\n", output.status.code()),
    )
    .unwrap();
    assert!(output.status.success(), "{output:?}");
    let original = fs::read_to_string(generated.join("replay.json")).unwrap();
    fs::write(evidence.join("emitted-replay.json"), &original).unwrap();
    AdmittedReplay::from_json(&original).unwrap();
    original
}

fn payload_owners(payload: &Value) -> [(Value, &'static str); 12] {
    let literal = json!({"kind":"literal","value":payload});
    [
        (
            json!({"step":"execute_command","command":"billing.invoice.CreateInvoice","input":{"metadata":literal}}),
            "/input/metadata/value",
        ),
        (
            json!({"step":"expect_error","error":"billing.invoice.Refused","fields":{"metadata":payload}}),
            "/fields/metadata",
        ),
        (
            json!({"step":"expect_event","event":"billing.invoice.InvoiceCreated","payload":{"metadata":payload}}),
            "/payload/metadata",
        ),
        (
            json!({"step":"eventually_event","event":"billing.invoice.InvoiceCreated","payload":{"metadata":payload}}),
            "/payload/metadata",
        ),
        (
            json!({"step":"expect_invocation","binding":"forward","command":"billing.invoice.CreateInvoice","input":{"metadata":literal}}),
            "/input/metadata/value",
        ),
        (
            json!({"step":"query_view","view":"billing.invoice.AllInvoices","params":{"metadata":literal}}),
            "/params/metadata/value",
        ),
        (
            json!({"step":"expect_view","view":"billing.invoice.AllInvoices","expectation":{"expect":"contains","fields":{"metadata":literal}}}),
            "/expectation/fields/metadata/value",
        ),
        (
            json!({"step":"expect_view","view":"billing.invoice.AllInvoices","expectation":{"expect":"excludes","fields":{"metadata":literal}}}),
            "/expectation/fields/metadata/value",
        ),
        (
            json!({"step":"eventually_view","view":"billing.invoice.AllInvoices","params":{"metadata":literal},"expectation":{"expect":"counts"}}),
            "/params/metadata/value",
        ),
        (
            json!({"step":"expect_view","view":"billing.invoice.AllInvoices","expectation":{"expect":"at","order_by":["id"],"position":{"row":"first"},"fields":{"metadata":literal}}}),
            "/expectation/fields/metadata/value",
        ),
        (
            json!({"step":"expect_halt","view":"billing.invoice.AllInvoices","after":1,"params":{"metadata":literal}}),
            "/params/metadata/value",
        ),
        (
            json!({"step":"eventually_halt","view":"billing.invoice.AllInvoices","after":1,"params":{"metadata":literal}}),
            "/params/metadata/value",
        ),
    ]
}

fn hash_child(evidence: &Path, index: usize, original_child: &str) -> Value {
    let child_path = evidence.join(format!("changed-child-{index}.json"));
    fs::write(&child_path, original_child).unwrap();
    let mut hash_command = Command::new("sha256sum");
    hash_command.arg(&child_path);
    fs::write(
        evidence.join(format!("hash-{index}.command")),
        format!("{hash_command:?}\n"),
    )
    .unwrap();
    let hash_output = hash_command.output().unwrap();
    fs::write(
        evidence.join(format!("hash-{index}.stdout")),
        &hash_output.stdout,
    )
    .unwrap();
    fs::write(
        evidence.join(format!("hash-{index}.stderr")),
        &hash_output.stderr,
    )
    .unwrap();
    assert!(hash_output.status.success());
    let digest = String::from_utf8(hash_output.stdout).unwrap();
    json!(format!(
        "sha256:{}",
        digest.split_whitespace().next().unwrap()
    ))
}
