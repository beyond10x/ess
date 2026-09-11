//! Local release qualification, original-byte ownership and the actual Bash action boundary.
//! External fixtures simulate routing only; their output cannot authenticate an artifact.
use ess_conformance::{
    coverage::AdmittedInput, AdmittedSuite, CountReport, CountStatus, SuiteProvenance,
};
use ess_deployment::{Digest, ReleaseBundle};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Read, Write},
    os::unix::net::UnixListener,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::OnceLock,
};

#[path = "support/bundle_fixture.rs"]
mod bundle_fixture;

const QUALIFIERS: [&str; 4] = [
    "attachment binding: unverified",
    "producer origin: unverified",
    "artifact execution: unverified",
    "signature verification: unsupported",
];
const CONFORMANCE: &str = "application/vnd.beyond10x.ess.evidence.conformance.v1";
const BUNDLE: &str = "application/vnd.beyond10x.ess.release-bundle.v1";
fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}
fn model() -> ess_compiler::EssIr {
    use ess_domain::{
        spec::{RawSpecFile, Specification},
        system::Source,
    };
    let root = workspace().join("examples/oracle-fixture");
    let mut sources = ess_compiler::source::SourceMap::new();
    let files: Vec<_> = [
        "system.yaml",
        "components.yaml",
        "domains/order.yaml",
        "domains/dispatch.yaml",
    ]
    .into_iter()
    .map(|name| {
        let text = fs::read_to_string(root.join(name)).unwrap();
        let raw = RawSpecFile::parse(&text).unwrap();
        sources.insert(name, text);
        (Source::new(name), raw)
    })
    .collect();
    ess_compiler::resolve::compile(&Specification::assemble(files).unwrap(), &sources).unwrap()
}
fn suite_document() -> Value {
    let mut p = SuiteProvenance::of(&model());
    p.suite_version = ess_conformance::scenario::SuiteFormat::parse("ess-conformance/5").unwrap();
    json!({"provenance":p,"scenarios": {
        "oracle.order/authored/first":{"purpose":"Independent local policy fixture", "steps":[],"source":[]},
        "oracle.order/authored/second":{"purpose":"A second selected obligation", "steps":[],"source":[]}
    },"coverage":{
        "selection":{"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}},
        "knowledge":"complete_inventory","generated":[],"authored":["oracle.order/authored/first","oracle.order/authored/second"],"outside":[],"refused":[],
        "authored_sources": {
          "first.yaml":{"digest":Digest::of_bytes(b"first independent source"),"scenario":"oracle.order/authored/first","disposition":"accepted"},
          "second.yaml":{"digest":Digest::of_bytes(b"second independent source"),"scenario":"oracle.order/authored/second","disposition":"accepted"}
        },"counts":{"generated":0,"authored":2,"outside":0,"refused":0}
    }})
}
fn direct(document: &Value) -> AdmittedInput {
    AdmittedInput::from_suite(
        AdmittedSuite::from_json(&(serde_json::to_string_pretty(document).unwrap() + "\n"))
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn suite_seven_release_qualification_retains_exact_parent_lineage() {
    // Synthetic policy evidence tests qualification only, never target execution or adoption.
    let f = Fixture::new();
    let mut document = suite_document();
    document["provenance"]["suite_version"] = json!("ess-conformance/7");
    let original = direct(&document);
    let selected = original
        .select(&["oracle.order/authored/first".parse().unwrap()])
        .unwrap();
    assert_eq!(
        selected.selected().suite().provenance.suite_version.major(),
        7
    );
    assert_eq!(
        selected.parents()[0].original_json(),
        original.selected().original_json()
    );
    f.set_input(&selected, true);
    f.write("report.json", report(&selected));
    success(&f.qualify("check-conformance", true).output().unwrap());
    assert!(f.calls().is_empty());
    for missing in [true, false] {
        let mut carrier = selected.document();
        if missing {
            carrier.parent_suites.clear();
        } else {
            carrier.parent_suites[0].push(' ');
        }
        f.write("expected.json", serde_json::to_string(&carrier).unwrap());
        let output = f.qualify("check-conformance", true).output().unwrap();
        assert!(!output.status.success(), "{output:?}");
        assert!(
            f.calls().is_empty(),
            "local qualification must not invoke external tools"
        );
    }
    f.set_input(&selected, true);
    f.write("report.json", report(&original));
    let stale = f.qualify("check-conformance", true).output().unwrap();
    assert!(!stale.status.success(), "{stale:?}");
    assert!(f.calls().is_empty());
}

fn report_value(input: &AdmittedInput, category: &str, profile: &str) -> Value {
    let s = input.selected();
    let p = &s.suite().provenance;
    let c = s.coverage().unwrap();
    let ids: Vec<_> = s
        .suite()
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    let mut counts =
        json!({"total":ids.len(),"passed":0,"failed":0,"error":0,"unsupported":0,"skipped":0});
    counts[category] = json!(ids.len());
    let mut outcomes = json!({"passed":[],"failed":[],"error":[],"unsupported":[],"skipped":[]});
    outcomes[category] = json!(ids);
    let status = if matches!(category, "failed" | "unsupported") {
        "failed"
    } else if category == "passed" {
        "passed"
    } else {
        "inconclusive"
    };
    let qualified = if status == "passed" && (s.suite().is_empty() || !c.is_complete()) {
        "inconclusive"
    } else {
        status
    };
    json!({"format":"ess-conformance-report/2","specification":format!("{}/{}",p.system,p.specification_version),"spec_digest":p.spec_digest,"implementation":"finite independently supplied fixture 1","producer_profile":profile,
      "suite":{"version":p.suite_version,"digest_profile":"sha256-json-bytes/1","digest":s.digest()},
      "execution_status":status,"counts":counts,"outcomes":outcomes,
      "coverage":{"knowledge":c.knowledge,"selection":c.selection,"counts":c.counts,"refused":c.refused},
      "conformance_status":qualified,"policy":"complete-selection/1","completed_at":1_700_000_001_000_u64})
}
fn report(input: &AdmittedInput) -> Vec<u8> {
    let bytes =
        (serde_json::to_string_pretty(&report_value(input, "passed", "rust-scenario-status/1"))
            .unwrap()
            + "\n \n")
            .into_bytes();
    CountReport::from_json(std::str::from_utf8(&bytes).unwrap(), input.selected()).unwrap();
    bytes
}
struct Fixture {
    root: PathBuf,
    bundle: ReleaseBundle,
    input: AdmittedInput,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "ess-delivery-trust-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("calls")).unwrap();
        let input = direct(&suite_document());
        let bundle = bundle_fixture::persisted_bundle();
        let f = Self {
            root,
            bundle,
            input,
        };
        f.write("component.json", f.bundle.component.to_canonical_json());
        f.write("build.json", f.bundle.build.to_canonical_json());
        f.write("runtime.json", f.bundle.runtime.to_canonical_json());
        f.write("bundle.json", f.bundle.to_canonical_json());
        f.set_input(&f.input, false);
        f.write("report.json", report(&f.input));
        for name in [
            "system.yaml",
            "components.yaml",
            "domains/order.yaml",
            "domains/dispatch.yaml",
        ] {
            f.write(
                &format!("model/{name}"),
                fs::read(workspace().join("examples/oracle-fixture").join(name)).unwrap(),
            );
        }
        f
    }
    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) {
        let p = self.root.join(name);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, bytes).unwrap();
    }
    fn set_input(&self, input: &AdmittedInput, carrier: bool) {
        self.write(
            "expected.json",
            if carrier {
                input.document().to_canonical_json().unwrap()
            } else {
                input.selected().original_json().into()
            },
        );
    }
    fn command(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_ess"));
        c.current_dir(&self.root)
            .env("PATH", tool_path())
            .env("ESS_RELEASE_FIXTURE", &self.root);
        c
    }
    fn qualify(&self, route: &str, carrier: bool) -> Command {
        let mut c = self.command();
        c.args([
            "generate",
            "release",
            route,
            "--spec",
            "model",
            "--report",
            "report.json",
            if carrier {
                "--expected-suite-input"
            } else {
                "--expected-suite"
            },
            "expected.json",
        ]);
        if route == "publish" {
            c.args([
                "--path",
                "bundle.json",
                "--to",
                "registry.example/bundle:test",
            ]);
        } else {
            c.args([
                "--component-ir",
                "component.json",
                "--build-ir",
                "build.json",
                "--runtime-ir",
                "runtime.json",
            ]);
            if route == "publish-conformance" {
                c.args(["--to", "registry.example/evidence:test-conformance"]);
            }
        }
        c
    }
    fn calls(&self) -> Vec<PathBuf> {
        let mut paths: Vec<_> = fs::read_dir(self.root.join("calls"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        paths.sort();
        paths
    }
    fn refusal(&self, carrier: bool, reason: &str) {
        let before = self.calls().len();
        for route in ["check-conformance", "publish-conformance", "publish"] {
            let out = self.qualify(route, carrier).output().unwrap();
            assert_eq!(
                out.status.code(),
                Some(1),
                "{route}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            let err = String::from_utf8_lossy(&out.stderr);
            assert!(
                err.contains(reason),
                "{route} did not reach {reason}: {err}"
            );
            assert!(!err.contains("conformance: passed"));
            assert!(out.stdout.is_empty());
            assert_eq!(self.calls().len(), before, "effect before {reason}");
        }
    }
}
fn tool_path() -> std::ffi::OsString {
    static EXECUTORS: OnceLock<PathBuf> = OnceLock::new();
    let root = EXECUTORS.get_or_init(|| {
        let root = std::env::temp_dir().join(format!("ess-release-tools-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let out = Command::new(compiler)
            .args(["--edition=2021"])
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/support/fake_release_component.rs"),
            )
            .arg("-o")
            .arg(root.join("oras"))
            .output()
            .unwrap();
        fs::write(root.join("compiler.stdout"), &out.stdout).unwrap();
        fs::write(root.join("compiler.stderr"), &out.stderr).unwrap();
        fs::write(root.join("compiler.status"), out.status.to_string()).unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        for tool in ["docker", "helm", "cosign", "syft"] {
            fs::copy(root.join("oras"), root.join(tool)).unwrap();
        }
        root
    });
    let mut paths = vec![root.clone()];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
    std::env::join_paths(paths).unwrap()
}
fn success(out: &Output) -> String {
    assert!(
        out.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8(out.stderr.clone()).unwrap();
    for q in QUALIFIERS {
        assert!(err.contains(q), "missing {q}: {err}");
    }
    err
}

#[test]
fn t01_t02_placeholders_and_rehashed_evidence_remain_only_consistency_checked() {
    let f = Fixture::new();
    let mut b = f.bundle.clone();
    for release in b.releases.values_mut() {
        for (kind, evidence) in &mut release.evidence {
            evidence.reference = format!("opaque.example/declared/{kind:?}:arbitrary");
            evidence.digest = Digest::of_bytes(b"invented opaque bytes");
        }
    }
    assert_ne!(b.digest(), f.bundle.digest());
    for release in b.releases.values() {
        for (extension, raw) in [
            ("json", release.to_canonical_json()),
            ("yaml", serde_yaml::to_string(release).unwrap()),
        ] {
            f.write(&format!("release.{extension}"), raw);
            let out = f
                .command()
                .args([
                    "release",
                    "verify",
                    "--path",
                    &format!("release.{extension}"),
                    "--build-ir",
                    "build.json",
                    "--runtime-ir",
                    "runtime.json",
                ])
                .output()
                .unwrap();
            let err = success(&out);
            assert!(err.contains("release consistency: checked"));
            assert!(err.contains("conformance: not assessed"));
            assert!(String::from_utf8_lossy(&out.stdout).contains("consistency checked"));
        }
    }
    let yaml = serde_yaml::to_string(&b).unwrap();
    assert_eq!(ReleaseBundle::from_yaml(&yaml).unwrap(), b);
    f.write("bundle.json", b.to_canonical_json());
    success(
        &f.command()
            .args(["release", "verify-bundle", "--path", "bundle.json"])
            .output()
            .unwrap(),
    );
    assert!(f.calls().is_empty());
}
#[test]
fn t03_arbitrary_logs_and_wrong_envelopes_refuse_all_positive_routes() {
    let f = Fixture::new();
    for raw in ["all tests passed\n".into(),"exit 0\n".into(),"{}".into(),report_value(&f.input,"passed","rust-scenario-status/1").to_string().replace("ess-conformance-report/2","ess-conformance-report/99"),json!({"format":"ess-conformance-run/2","summary":report_value(&f.input,"passed","rust-scenario-status/1"),"started_at":0,"scenarios":[]}).to_string()] {
        f.write("report.json",raw);f.refusal(false,"report");
    }
}
#[test]
fn t04_legacy_readers_remain_readable_but_no_legacy_summary_qualifies() {
    let f = Fixture::new();
    for (status, entries, total) in [
        ("passed", vec![], 2),
        ("failed", vec!["failed oracle.order/authored/first"], 2),
        ("inconclusive", vec!["error oracle.order/authored/first"], 2),
        ("failed", vec!["unsupported oracle.order/authored/first"], 2),
        (
            "inconclusive",
            vec!["skipped oracle.order/authored/first"],
            2,
        ),
        ("passed", vec![], 0),
    ] {
        let value = json!({"format":"ess-conformance-report/1","specification":"oracle/v1","spec_digest":f.input.selected().suite().provenance.spec_digest,"implementation":"legacy fixture 1","status":status,"scenarios_total":total,"scenarios_failed":entries.len(),"suite_version":"ess-conformance/4","failed_scenarios":entries,"completed_at":1_700_000_001_000_i64});
        let raw = value.to_string();
        let old = ess_conformance::StandaloneConformanceReport::from_json(&raw)
            .expect("valid legacy fixture reaches the existing reader");
        assert_eq!(serde_json::to_value(old).unwrap(), value);
        f.write("report.json", raw);
        f.refusal(false, "report");
    }
}
#[test]
fn t06_complete_rust_go_and_filtered_reports_qualify_only_their_exact_selection() {
    let f = Fixture::new();
    let selected = f
        .input
        .select(&["oracle.order/authored/first".parse().unwrap()])
        .unwrap();
    for (input, carrier) in [(&f.input, false), (&selected, true)] {
        f.set_input(input, carrier);
        for profile in ["rust-scenario-status/1", "go-scenario-status/1"] {
            let raw = report_value(input, "passed", profile).to_string();
            assert_eq!(
                CountReport::from_json(&raw, input.selected())
                    .unwrap()
                    .conformance_status(),
                CountStatus::Passed
            );
            f.write("report.json", &raw);
            let out = f.qualify("check-conformance", carrier).output().unwrap();
            let err = success(&out);
            assert!(out.stdout.is_empty());
            for required in [
                "conformance: passed for the supplied exact declared selection",
                input.selected().digest(),
                &serde_json::to_string(&input.selected().coverage().unwrap().selection).unwrap(),
                f.input.selected().suite().provenance.spec_digest.as_str(),
                f.input
                    .selected()
                    .suite()
                    .provenance
                    .contract_digest
                    .as_str(),
                Digest::of_bytes(raw.as_bytes()).as_str(),
            ] {
                assert!(err.contains(required), "missing {required}: {err}");
            }
        }
    }
    assert!(f.calls().is_empty());
}
#[test]
fn t07_original_suite_pairing_and_full_model_identity_refuse_substitution() {
    let f = Fixture::new();
    f.write(
        "expected.json",
        format!("{} ", f.input.selected().original_json()),
    );
    f.refusal(false, "exact suite reference mismatch");
    f.set_input(&f.input, false);
    for (path, replacement) in [
        ("/suite/digest", json!(Digest::of_bytes(b"wrong"))),
        ("/suite/digest_profile", json!("other/1")),
        ("/suite/version", json!("ess-conformance/4")),
        ("/specification", json!("oracle/v2")),
        ("/spec_digest", json!("b".repeat(64))),
    ] {
        let mut value = report_value(&f.input, "passed", "rust-scenario-status/1");
        *value.pointer_mut(path).unwrap() = replacement;
        f.write("report.json", value.to_string());
        f.refusal(false, "report");
    }
    for (path, replacement) in [
        ("/provenance/system", json!("other")),
        ("/provenance/specification_version", json!("v2")),
        ("/provenance/spec_digest", json!("b".repeat(64))),
        ("/provenance/contract_digest", json!("b".repeat(64))),
        ("/provenance/spec_digest", json!("abcdef1234567890")),
        ("/provenance/contract_digest", json!("abcdef1234567890")),
    ] {
        let mut value = suite_document();
        *value.pointer_mut(path).unwrap() = replacement;
        let input = direct(&value);
        f.set_input(&input, false);
        f.write("report.json", report(&input));
        f.refusal(false, "model identity");
    }
    f.set_input(&f.input, false);
    f.write("report.json", report(&f.input));
    let system = fs::read_to_string(f.root.join("model/system.yaml")).unwrap();
    f.write(
        "model/system.yaml",
        system.replace("version: v1", "version: v2"),
    );
    f.refusal(false, "model identity");
}
#[test]
fn t08_scope_origin_and_subset_cannot_satisfy_a_different_expectation() {
    let f = Fixture::new();
    for component in ["order-service", "dispatch-service", "missing-component"] {
        let mut value = suite_document();
        value["provenance"]["component"] = json!(component);
        value["coverage"]["selection"]["scope"] = json!({"kind":"component","component":component});
        let input = direct(&value);
        f.set_input(&input, false);
        f.write("report.json", report(&input));
        if component == "missing-component" {
            f.refusal(false, "model component");
        } else {
            success(&f.qualify("check-conformance", false).output().unwrap());
            f.set_input(&f.input, false);
            f.refusal(false, "report");
        }
    }
    for origins in ["authored", "generated_and_authored"] {
        let mut value = suite_document();
        value["coverage"]["selection"]["origins"] = json!(origins);
        let input = direct(&value);
        f.set_input(&input, false);
        f.write("report.json", report(&input));
        success(&f.qualify("check-conformance", false).output().unwrap());
        let other = if origins == "authored" {
            "generated_and_authored"
        } else {
            "authored"
        };
        value["coverage"]["selection"]["origins"] = json!(other);
        f.set_input(&direct(&value), false);
        f.refusal(false, "report");
    }
    let subset = f
        .input
        .select(&["oracle.order/authored/first".parse().unwrap()])
        .unwrap();
    f.write("report.json", report(&subset));
    f.set_input(&f.input, false);
    f.refusal(false, "report");
    let empty = f.input.select(&[]).unwrap();
    f.set_input(&empty, true);
    f.write("report.json", report(&empty));
    f.refusal(true, "must be passed");
    // oracle is a delivery label, not a modeled component: the whole-system control already passed.
    assert!(!model()
        .components()
        .keys()
        .any(|c| c.to_string() == f.bundle.component.component().as_str()));
}
#[test]
fn t09_complete_original_parent_lineage_is_required() {
    let f = Fixture::new();
    let first = f
        .input
        .select(&["oracle.order/authored/first".parse().unwrap()])
        .unwrap();
    let second = first
        .select(&["oracle.order/authored/first".parse().unwrap()])
        .unwrap();
    f.write("report.json", report(&second));
    f.set_input(&second, true);
    success(&f.qualify("check-conformance", true).output().unwrap());
    let original = second.document();
    for variant in [
        "missing",
        "wrong",
        "reordered",
        "duplicate",
        "cycle",
        "extra",
        "payload",
        "dependency",
        "reserialized",
    ] {
        let mut d = original.clone();
        match variant {
            "missing" => {
                d.parent_suites.pop();
            }
            "wrong" => {
                d.parent_suites[0] = f.input.selected().original_json().into();
            }
            "reordered" => d.parent_suites.reverse(),
            "duplicate" => d.parent_suites.push(d.parent_suites[0].clone()),
            "cycle" => d.parent_suites.insert(0, d.suite_json.clone()),
            "extra" => d.parent_suites.push(suite_document().to_string()),
            "payload" => {
                d.suite_json = d.suite_json.replace(
                    "Independent local policy fixture",
                    "Altered retained payload",
                );
            }
            "dependency" => {
                let mut value: Value = serde_json::from_str(&d.suite_json).unwrap();
                value["scenarios"]["oracle.order/authored/first"]["source"] =
                    json!([{"kind":"command","name":"oracle.order.PlaceOrder"}]);
                d.suite_json = serde_json::to_string_pretty(&value).unwrap() + "\n";
            }
            "reserialized" => {
                d.parent_suites[0] = serde_json::from_str::<Value>(&d.parent_suites[0])
                    .unwrap()
                    .to_string();
            }
            _ => unreachable!(),
        }
        let raw = d.to_canonical_json().unwrap();
        let error = AdmittedInput::from_json(&raw).unwrap_err();
        if matches!(variant, "payload" | "dependency") {
            assert!(
                error
                    .to_string()
                    .contains("child changed full scenario semantics or dependencies"),
                "{variant} must reach lineage semantics: {error}"
            );
        }
        f.write("expected.json", raw);
        f.refusal(true, "expected selection");
    }
    f.write("expected.json", second.selected().original_json());
    f.refusal(false, "MissingParent");
}
#[test]
fn t10_negative_inconclusive_and_unknown_reports_stop_all_publication_routes() {
    let f = Fixture::new();
    for (category, profile) in [
        ("failed", "rust-scenario-status/1"),
        ("error", "rust-scenario-status/1"),
        ("unsupported", "rust-scenario-status/1"),
        ("skipped", "go-scenario-status/1"),
    ] {
        let raw = report_value(&f.input, category, profile).to_string();
        assert_ne!(
            CountReport::from_json(&raw, f.input.selected())
                .unwrap()
                .conformance_status(),
            CountStatus::Passed
        );
        f.write("report.json", raw);
        f.refusal(false, "must be passed");
    }
    let mut value = suite_document();
    value["coverage"]["knowledge"] = json!("unknown");
    let input = direct(&value);
    f.set_input(&input, false);
    f.write("report.json", report(&input));
    f.refusal(false, "must be passed");
    f.set_input(&f.input, false);
    for (path, value) in [
        ("/producer_profile", json!("unknown/1")),
        ("/conformance_status", json!("inconclusive")),
    ] {
        let mut r = report_value(&f.input, "passed", "rust-scenario-status/1");
        *r.pointer_mut(path).unwrap() = value;
        f.write("report.json", r.to_string());
        f.refusal(false, "report");
    }
}
#[test]
fn t11_exact_tokens_membership_and_duplicate_keys_reach_the_report_reader() {
    let f = Fixture::new();
    let original = report_value(&f.input, "passed", "rust-scenario-status/1").to_string();
    for token in ["18446744073709551616", "2.0", "2e0", "-2"] {
        let raw = original.replace("\"passed\":2", &format!("\"passed\":{token}"));
        assert_ne!(raw, original);
        f.write("report.json", raw);
        f.refusal(false, "report");
    }
    for (path, value) in [
        ("/counts/total", json!(3)),
        (
            "/outcomes/passed",
            json!(["oracle.order/authored/first", "oracle.order/authored/first"]),
        ),
        ("/outcomes/failed", json!(["oracle.order/authored/first"])),
        ("/coverage/counts/authored", json!(1)),
    ] {
        let mut r = report_value(&f.input, "passed", "rust-scenario-status/1");
        *r.pointer_mut(path).unwrap() = value;
        f.write("report.json", r.to_string());
        f.refusal(false, "report");
    }
    f.write(
        "report.json",
        original.replacen(
            "\"completed_at\":",
            "\"completed_at\":0,\"completed_at\":",
            1,
        ),
    );
    f.refusal(false, "report");
}
#[test]
fn t11_usage_groups_and_raw_pin_syntax_refuse_before_effects() {
    let f = Fixture::new();
    for args in [
        vec!["--spec", "model"],
        vec!["--report", "report.json"],
        vec!["--expected-suite", "expected.json"],
        vec!["--report-sha256", "sha256:bad"],
        vec!["--expected-input-sha256", "sha256:bad"],
    ] {
        let out = f
            .command()
            .args([
                "release",
                "publish",
                "--path",
                "bundle.json",
                "--to",
                "registry.example/bundle:test",
            ])
            .args(args)
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(2),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    for route in ["check-conformance", "publish-conformance", "publish"] {
        let out = f
            .qualify(route, false)
            .args(["--expected-suite-input", "expected.json"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        for pin in ["--report-sha256", "--expected-input-sha256"] {
            let out = f
                .qualify(route, false)
                .args([pin, "sha256:bad"])
                .output()
                .unwrap();
            assert_eq!(out.status.code(), Some(2));
        }
        let out = f
            .qualify(route, false)
            .args(["--report-sha256", Digest::of_bytes(b"wrong").as_str()])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stderr).contains("raw report byte pin mismatch"));
        let out = f
            .qualify(route, false)
            .args([
                "--expected-input-sha256",
                Digest::of_bytes(b"wrong").as_str(),
            ])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stderr).contains("expected input byte pin mismatch"));
    }
    assert!(f.calls().is_empty());
}
#[test]
fn t12_compiled_deployment_and_every_artifact_context_are_checked_and_named() {
    let f = Fixture::new();
    let out = f.qualify("publish", false).output().unwrap();
    let err = success(&out);
    for release in f.bundle.releases.values() {
        for a in release.artifacts.values() {
            for value in [
                release.release_unit.as_str(),
                a.build_output.as_str(),
                &a.reference,
                a.digest.as_str(),
                &release.source_commit,
            ] {
                assert!(err.contains(value), "{err} omits {value}");
            }
            for (p, d) in &a.platforms {
                assert!(err.contains(p));
                assert!(err.contains(d.as_str()));
            }
        }
    }
    for (file, pointer, replacement, reason) in [
        (
            "component.json",
            "/system",
            json!("other"),
            "model identity",
        ),
        (
            "component.json",
            "/semantic_version",
            json!("v2"),
            "model identity",
        ),
        (
            "runtime.json",
            "/semantic_digest",
            json!(Digest::of_bytes(b"other model")),
            "model identity",
        ),
        (
            "runtime.json",
            "/build_digest",
            json!(Digest::of_bytes(b"other build")),
            "build",
        ),
    ] {
        let original = fs::read(f.root.join(file)).unwrap();
        let mut v: Value = serde_json::from_slice(&original).unwrap();
        *v.pointer_mut(pointer).unwrap() = replacement;
        let canonical = if file == "component.json" {
            ess_deployment::ComponentIr::from_json(&v.to_string())
                .unwrap()
                .to_canonical_json()
        } else {
            ess_deployment::RuntimeIr::from_json(&v.to_string())
                .unwrap()
                .to_canonical_json()
        };
        f.write(file, canonical);
        // Typed canonical serialization is checked independently before model/relationship refusal.
        let out = f.qualify("check-conformance", false).output().unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(reason),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        f.write(file, original);
    }
}
#[test]
fn t13_report_upload_owns_original_bytes_and_distinguishes_attachment_digest() {
    let f = Fixture::new();
    let expected = fs::read(f.root.join("report.json")).unwrap();
    let raw_digest = Digest::of_bytes(&expected);
    let out = f
        .qualify("publish-conformance", false)
        .args(["--report-sha256", raw_digest.as_str()])
        .output()
        .unwrap();
    success(&out);
    let calls = f.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(fs::read(calls[0].join("payload.raw")).unwrap(), expected);
    assert_eq!(
        fs::read_to_string(calls[0].join("media-type")).unwrap(),
        "application/json"
    );
    assert!(fs::read_to_string(calls[0].join("argv"))
        .unwrap()
        .contains(CONFORMANCE));
    assert_eq!(
        fs::read_to_string(calls[0].join("cwd-mode")).unwrap(),
        0o700.to_string(),
        "publication staging must be private"
    );
    let manifest = format!("sha256:{}", "9".repeat(64));
    assert_ne!(raw_digest.as_str(), manifest);
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        format!("registry.example/evidence:test-conformance — published at {manifest}\n")
    );
}
#[test]
fn t13_mutating_caller_paths_after_admission_cannot_change_staged_report_or_bundle() {
    for route in ["publish-conformance", "publish"] {
        let f = Fixture::new();
        let expected = fs::read(f.root.join(if route == "publish" {
            "bundle.json"
        } else {
            "report.json"
        }))
        .unwrap();
        let listener = UnixListener::bind(f.root.join("admitted.sock")).unwrap();
        listener.set_nonblocking(true).unwrap();
        let mut command = f.qualify(route, false);
        command
            .env("ESS_RELEASE_SYNC", f.root.join("admitted.sock"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => panic!("sync accept: {e}"),
            }
            if child.try_wait().unwrap().is_some() {
                let out = child.wait_with_output().unwrap();
                panic!(
                    "publisher exited before admission synchronization: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
            if std::time::Instant::now() > deadline {
                child.kill().unwrap();
                let out = child.wait_with_output().unwrap();
                panic!(
                    "publisher never reached synchronization: {}",
                    String::from_utf8_lossy(&out.stderr)
                );
            }
            std::thread::yield_now();
        };
        let mut ready = [0];
        socket.read_exact(&mut ready).unwrap();
        assert_eq!(ready, [1]);
        for file in ["report.json", "expected.json", "bundle.json"] {
            f.write(file, b"mutated after admission");
        }
        socket.write_all(&[2]).unwrap();
        success(&child.wait_with_output().unwrap());
        assert_eq!(
            fs::read(f.calls()[0].join("payload.raw")).unwrap(),
            expected
        );
    }
}
#[test]
fn t13_oras_bad_digest_non_utf8_and_nonzero_status_fail_both_publishers() {
    for route in ["publish-conformance", "publish"] {
        for mode in ["bad-digest", "non-utf8", "fail-oras"] {
            let f = Fixture::new();
            let out = f
                .qualify(route, false)
                .env("ESS_RELEASE_MODE", mode)
                .output()
                .unwrap();
            assert_eq!(out.status.code(), Some(1));
            assert!(out.stdout.is_empty());
            assert!(String::from_utf8_lossy(&out.stderr).contains("ORAS"));
            assert_eq!(f.calls().len(), 1);
        }
    }
}
#[test]
fn t17_grouped_flat_streams_and_existing_canonical_outputs_are_identical() {
    let f = Fixture::new();
    for format in ["json", "yaml"] {
        let mut flat = f.command();
        flat.args([
            "release",
            "verify-bundle",
            "--path",
            "bundle.json",
            "--format",
            format,
        ]);
        let a = flat.output().unwrap();
        let b = f
            .command()
            .args([
                "generate",
                "release",
                "verify-bundle",
                "--path",
                "bundle.json",
                "--format",
                format,
            ])
            .output()
            .unwrap();
        success(&a);
        assert_eq!(a.stdout, b.stdout);
        assert_eq!(a.stderr, b.stderr);
        let expected = if format == "json" {
            serde_json::to_string_pretty(&f.bundle).unwrap() + "\n"
        } else {
            serde_yaml::to_string(&f.bundle).unwrap()
        };
        assert_eq!(a.stdout, expected.as_bytes());
    }
    for route in ["check-conformance", "publish-conformance", "publish"] {
        let mut grouped = f.qualify(route, false);
        let argv: Vec<_> = grouped
            .get_args()
            .skip(1)
            .map(std::ffi::OsStr::to_owned)
            .collect();
        let a = grouped.output().unwrap();
        let b = f.command().args(argv).output().unwrap();
        success(&a);
        assert_eq!(a.status.code(), b.status.code());
        assert_eq!(a.stdout, b.stdout);
        assert_eq!(a.stderr, b.stderr);
    }
    let plain = f
        .command()
        .args([
            "release",
            "publish",
            "--path",
            "bundle.json",
            "--to",
            "registry.example/bundle:test",
        ])
        .output()
        .unwrap();
    assert!(success(&plain).contains("conformance: not assessed"));
    assert!(String::from_utf8_lossy(&plain.stdout)
        .trim()
        .ends_with(&"9".repeat(64)));
}

const COMPONENT_SOURCE:&str = "format: ess-component/1\ncomponent: oracle\nsystem: oracle\nsemantic_version: v1\ninputs:\n  specification: model\n  realization: realization.yaml\n  build: build.yaml\n  runtime: runtime.yaml\nrelease_units:\n  runtime: oracle-runtime\n  chart: oracle-chart\n";
fn build_source() -> String {
    format!(
        r#"
format: ess-build/1
build: oracle-runtime
platforms: [{{os: linux, architecture: amd64}}]
secrets: [registry-token]
nodes:
  - {{id: base, kind: oci_base, reference: docker.io/library/alpine, digest: "sha256:{zeros}"}}
  - {{id: source, kind: source, path: ., destination: /src}}
  - id: compile
    kind: run
    base: base
    argv: [cp, /src/oracle, /usr/local/bin/oracle]
    mounts: [{{kind: input, from: source, target: /src}}]
  - id: runtime-image
    kind: image
    rootfs: compile
    config: {{entrypoint: [/usr/local/bin/oracle], user: "10001"}}
  - {{id: chart-file, kind: artifact, from: compile, path: /src/chart.tgz}}
outputs:
  - {{name: app, release_unit: oracle-runtime, node: runtime-image, kind: oci_image, repository: registry.example/oracle}}
  - {{name: chart, release_unit: oracle-chart, node: chart-file, kind: helm_chart}}
"#,
        zeros = "0".repeat(64)
    )
}
impl Fixture {
    fn action(&self) -> Command {
        self.write("component.yaml", COMPONENT_SOURCE);
        self.write("build.yaml", build_source());
        assert_eq!(
            ess_deployment::compile_build(
                &ess_deployment::BuildSpec::from_yaml(&build_source()).unwrap()
            )
            .unwrap(),
            self.bundle.build
        );
        let installed = self.root.join("target/ess/fixture-revision/bin/ess");
        fs::create_dir_all(installed.parent().unwrap()).unwrap();
        if !installed.exists() {
            std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_ess"), &installed).unwrap();
        }
        let mut c = Command::new("/usr/bin/bash");
        c.arg(workspace().join(".github/actions/release-component/release.sh"))
            .current_dir(&self.root)
            .env("PATH", tool_path())
            .env("ESS_RELEASE_FIXTURE", &self.root);
        for (name, value) in [
            ("COMPONENT_SERVICE", "oracle"),
            ("COMPONENT_VERSION", "1.2.3"),
            ("ESS_REVISION", "fixture-revision"),
            (
                "CHECK_COMMAND",
                "printf 'generic check only\\n'; printf checked > check-ran",
            ),
            ("COMPONENT_PATH", "component.yaml"),
            ("BUILD_PATH", "build.yaml"),
            ("BUILD_IR", "build.json"),
            ("RUNTIME_IR", "runtime.json"),
            ("IMAGE_REPOSITORY", "registry.example/oracle"),
            ("BUILD_IMAGE", "true"),
            ("CHART_NAMESPACE", "oci://registry.example/charts"),
            ("EVIDENCE_REPOSITORY", "registry.example/evidence"),
            ("BUNDLE_REPOSITORY", "registry.example/bundle"),
            ("SPEC_PATH", "model"),
            ("CONFORMANCE_REPORT", "report.json"),
            ("CONFORMANCE_SUITE", "expected.json"),
            ("CONFORMANCE_SUITE_INPUT", ""),
            ("GITHUB_SHA", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            ("GITHUB_REF_TYPE", "branch"),
            ("GITHUB_REPOSITORY", "local/fixture"),
            ("GITHUB_RUN_ID", "1"),
        ] {
            c.env(name, value);
        }
        c.env("GITHUB_STEP_SUMMARY", self.root.join("summary.md"));
        c
    }
    fn action_output(&self, c: &mut Command) -> Output {
        let out = c.output().unwrap();
        self.write("action.stdout.raw", &out.stdout);
        self.write("action.stderr.raw", &out.stderr);
        self.write("action.status", out.status.to_string());
        out
    }
    fn assert_no_publication(&self) {
        for call in self.calls() {
            assert!(
                !call
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with("-oras"),
                "late publication {}",
                call.display()
            );
        }
    }
}
#[test]
fn t15_missing_invalid_and_nonqualifying_action_inputs_stop_before_generic_check() {
    for variant in [
        "report-missing",
        "spec-missing",
        "no-expected",
        "both-expected",
        "report-file-missing",
        "expected-file-missing",
        "wrong-model",
        "log-report",
        "negative",
    ] {
        let f = Fixture::new();
        let mut c = f.action();
        match variant {
            "report-missing" => {
                c.env_remove("CONFORMANCE_REPORT");
            }
            "spec-missing" => {
                c.env_remove("SPEC_PATH");
            }
            "no-expected" => {
                c.env("CONFORMANCE_SUITE", "");
            }
            "both-expected" => {
                c.env("CONFORMANCE_SUITE_INPUT", "expected.json");
            }
            "report-file-missing" => {
                c.env("CONFORMANCE_REPORT", "missing.json");
            }
            "expected-file-missing" => {
                c.env("CONFORMANCE_SUITE", "missing.json");
            }
            "wrong-model" => {
                c.env("SPEC_PATH", workspace().join("examples/billing"));
            }
            "log-report" => f.write("report.json", "the generic log is not a report\n"),
            "negative" => f.write(
                "report.json",
                report_value(&f.input, "failed", "rust-scenario-status/1").to_string(),
            ),
            _ => unreachable!(),
        }
        let out = f.action_output(&mut c);
        assert!(!out.status.success(), "{variant}");
        assert!(
            !f.root.join("check-ran").exists(),
            "{variant} ran generic check: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            f.calls().is_empty(),
            "{variant} crossed release tool boundary"
        );
    }
}
#[test]
fn t15_snapshot_pins_block_mutation_before_evidence_and_final_bundle_publication() {
    for file in ["conformance-report.json", "expected-input.json"] {
        let f = Fixture::new();
        let mut c = f.action();
        c.env("CHECK_COMMAND",format!("for snapshot in target/release/conformance-inputs.*/{file}; do printf changed > \"$snapshot\"; done; printf checked > check-ran"));
        let out = f.action_output(&mut c);
        assert_eq!(
            out.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(String::from_utf8_lossy(&out.stderr).contains("byte pin mismatch"));
        assert!(f.root.join("check-ran").exists());
        assert!(f.calls().iter().any(|p| p
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with("-helm")));
        f.assert_no_publication();
    }
    let f = Fixture::new();
    let mut c = f.action();
    c.env("ESS_RELEASE_MODE", "mutate-after-conformance");
    let out = f.action_output(&mut c);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("raw report byte pin mismatch"));
    let oras: Vec<_> = f
        .calls()
        .into_iter()
        .filter(|p| p.file_name().unwrap().to_string_lossy().ends_with("-oras"))
        .collect();
    assert_eq!(oras.len(), 4);
    assert!(fs::read_to_string(oras[3].join("argv"))
        .unwrap()
        .contains(CONFORMANCE));
    assert!(!f.root.join("summary.md").exists());
}
#[test]
fn t15_publisher_failure_is_propagated_before_any_later_evidence_or_bundle_call() {
    let f = Fixture::new();
    let mut c = f.action();
    c.env("ESS_RELEASE_MODE", "fail-provenance");
    let out = f.action_output(&mut c);
    assert_eq!(
        out.status.code(),
        Some(23),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let calls = f.calls();
    let last = calls.last().unwrap();
    assert!(fs::read_to_string(last.join("argv"))
        .unwrap()
        .contains("evidence.provenance.v1"));
    assert_eq!(fs::read_to_string(last.join("status")).unwrap(), "23");
    assert!(!f.root.join("summary.md").exists());
}
#[test]
fn t16_action_input_environment_and_generic_check_failure_contract() {
    let raw = fs::read_to_string(workspace().join(".github/actions/release-component/action.yml"))
        .unwrap();
    let action: Value = serde_yaml::from_str(&raw).unwrap();
    for name in ["spec-path", "conformance-report"] {
        assert_eq!(action["inputs"][name]["required"], true);
    }
    for name in ["conformance-suite", "conformance-suite-input"] {
        assert_eq!(action["inputs"][name]["default"], "");
    }
    let step = action["runs"]["steps"].as_array().unwrap().last().unwrap();
    for (key, input) in [
        ("SPEC_PATH", "spec-path"),
        ("CONFORMANCE_REPORT", "conformance-report"),
        ("CONFORMANCE_SUITE", "conformance-suite"),
        ("CONFORMANCE_SUITE_INPUT", "conformance-suite-input"),
    ] {
        assert_eq!(step["env"][key], format!("${{{{ inputs.{input} }}}}"));
    }
    let f = Fixture::new();
    let mut c = f.action();
    c.env("CHECK_COMMAND", "printf 'generic failure\\n'; exit 19");
    let out = f.action_output(&mut c);
    assert_eq!(out.status.code(), Some(19));
    assert!(f.calls().is_empty());
    assert_eq!(
        fs::read_to_string(f.root.join("target/release/check.log")).unwrap(),
        "generic failure\n"
    );
}
#[test]
fn t16_actual_action_build_adopt_platform_chart_and_summary_paths_preserve_limits() {
    for (build, mode, carrier) in [
        ("true", "", false),
        ("false", "", false),
        ("false", "platform-fallback", true),
    ] {
        let f = Fixture::new();
        let mut c = f.action();
        c.env("BUILD_IMAGE", build).env("ESS_RELEASE_MODE", mode);
        let selected = if carrier {
            f.input
                .select(&["oracle.order/authored/first".parse().unwrap()])
                .unwrap()
        } else {
            f.input.clone()
        };
        f.set_input(&selected, carrier);
        let expected = report(&selected);
        f.write("report.json", &expected);
        if carrier {
            c.env("CONFORMANCE_SUITE", "")
                .env("CONFORMANCE_SUITE_INPUT", "expected.json");
        }
        let out = f.action_output(&mut c);
        success(&out);
        let bundle = ReleaseBundle::from_json(
            &fs::read_to_string(f.root.join("target/release/ess-release-bundle.json")).unwrap(),
        )
        .unwrap();
        let runtime = bundle
            .releases
            .values()
            .find(|r| r.release_unit.as_str() == "oracle-runtime")
            .unwrap();
        let chart = bundle
            .releases
            .values()
            .find(|r| r.release_unit.as_str() == "oracle-chart")
            .unwrap();
        assert_eq!(
            runtime.artifacts.values().next().unwrap().reference,
            "registry.example/oracle"
        );
        assert_eq!(
            chart.artifacts.values().next().unwrap().reference,
            "registry.example/charts/oracle-chart"
        );
        assert_eq!(
            chart.artifacts.values().next().unwrap().digest.as_str(),
            format!("sha256:{}", "3".repeat(64))
        );
        let image = runtime.artifacts.values().next().unwrap();
        assert_eq!(
            image.platforms["linux/amd64"].as_str(),
            format!(
                "sha256:{}",
                if mode == "platform-fallback" {
                    "1"
                } else {
                    "2"
                }
                .repeat(64)
            )
        );
        assert_action_uploads(&f, build, &expected, runtime, chart);
    }
}
fn assert_action_uploads(
    f: &Fixture,
    build: &str,
    expected: &[u8],
    runtime: &ess_deployment::ReleaseManifest,
    chart: &ess_deployment::ReleaseManifest,
) {
    let calls = f.calls();
    let argv: Vec<_> = calls
        .iter()
        .map(|p| fs::read_to_string(p.join("argv")).unwrap())
        .collect();
    assert_eq!(
        argv.iter()
            .filter(|a| a.starts_with("buildx\nbake\n"))
            .count(),
        if build == "true" { 2 } else { 1 }
    );
    let uploads: Vec<_> = calls
        .iter()
        .filter(|p| p.file_name().unwrap().to_string_lossy().ends_with("-oras"))
        .collect();
    assert_eq!(uploads.len(), 5);
    let report_upload = uploads
        .iter()
        .find(|p| {
            fs::read_to_string(p.join("argv"))
                .unwrap()
                .contains(CONFORMANCE)
        })
        .unwrap();
    assert_eq!(
        fs::read(report_upload.join("payload.raw")).unwrap(),
        expected
    );
    assert!(!argv
        .iter()
        .any(|a| a.contains("check.log") || a.contains("conformance.log")));
    let attachment = &runtime.evidence[&ess_deployment::EvidenceKind::Conformance];
    assert_eq!(
        attachment.reference,
        "registry.example/evidence:1.2.3-conformance"
    );
    assert_eq!(
        attachment.digest.as_str(),
        format!("sha256:{}", "9".repeat(64))
    );
    assert_ne!(attachment.digest, Digest::of_bytes(expected));
    assert_eq!(chart.evidence, runtime.evidence);
    assert_eq!(
        fs::read_to_string(f.root.join("target/release/check.log")).unwrap(),
        "generic check only\n"
    );
    let summary = fs::read_to_string(f.root.join("summary.md")).unwrap();
    for (repository, digit) in [
        ("registry.example/bundle", '9'),
        ("registry.example/oracle", '1'),
        ("registry.example/charts/oracle-chart", '3'),
    ] {
        assert!(summary.contains(&format!(
            "{repository}@sha256:{}",
            digit.to_string().repeat(64)
        )));
    }
    for q in QUALIFIERS {
        assert!(summary.contains(q));
    }
    assert!(summary.contains("supplied exact declared selection"));
    assert!(fs::read_to_string(uploads.last().unwrap().join("argv"))
        .unwrap()
        .contains(BUNDLE));
}
#[test]
fn t16_chart_digest_absence_preserves_earlier_effects_and_blocks_all_evidence() {
    let f = Fixture::new();
    let mut c = f.action();
    c.env("ESS_RELEASE_MODE", "chart-no-digest");
    let out = f.action_output(&mut c);
    assert!(!out.status.success());
    f.assert_no_publication();
    let calls = f.calls();
    assert!(fs::read_to_string(calls.last().unwrap().join("argv"))
        .unwrap()
        .starts_with("push\n"));
    assert!(!f.root.join("summary.md").exists());
}

#[test]
fn t08_generated_origin_qualifies_itself_and_refuses_other_origin_expectations() {
    let f = Fixture::new();
    let mut value = suite_document();
    value["scenarios"] = json!({"oracle.order.PlaceOrder/outcome/accepted":{"purpose":"Declared generated selection fixture", "steps":[],"source":[]}});
    value["coverage"]["generated"] = json!(["oracle.order.PlaceOrder/outcome/accepted"]);
    value["coverage"]["authored"] = json!([]);
    value["coverage"]["authored_sources"] = json!({});
    value["coverage"]["counts"] = json!({"generated":1,"authored":0,"outside":0,"refused":0});
    for origins in ["generated", "generated_and_authored"] {
        value["coverage"]["selection"]["origins"] = json!(origins);
        let input = direct(&value);
        f.set_input(&input, false);
        f.write("report.json", report(&input));
        success(&f.qualify("check-conformance", false).output().unwrap());
        f.set_input(&f.input, false);
        f.refusal(false, "report");
    }
}
#[test]
fn t10_in_scope_refusal_beside_nonempty_all_passes_blocks_positive_qualification() {
    use ess_conformance::{
        coverage::{Origins, Scope},
        coverage_build::{self, CoverageSource},
    };
    let f = Fixture::new();
    let rejected = coverage_build::build(
        &model(),
        &[CoverageSource::new("rejected.yaml", "not an authored scenario").unwrap()],
        Scope::System,
        Origins::Authored,
    )
    .unwrap();
    let r = rejected.selected().coverage().unwrap();
    assert!(!r.refused.is_empty());
    let mut value = suite_document();
    value["coverage"]["refused"] = serde_json::to_value(&r.refused).unwrap();
    value["coverage"]["counts"]["refused"] = json!(r.refused.len());
    for (name, source) in &r.authored_sources {
        value["coverage"]["authored_sources"][name.as_str()] =
            serde_json::to_value(source).unwrap();
    }
    let input = direct(&value);
    assert_eq!(input.selected().suite().len(), 2);
    f.set_input(&input, false);
    let raw = report(&input);
    assert_eq!(
        CountReport::from_json(std::str::from_utf8(&raw).unwrap(), input.selected())
            .unwrap()
            .conformance_status(),
        CountStatus::Inconclusive
    );
    f.write("report.json", raw);
    f.refusal(false, "must be passed");
}
#[test]
fn t02_invalid_nested_release_claims_and_duplicate_maps_refuse_before_qualified_upload() {
    let f = Fixture::new();
    for field in ["build_output", "kind"] {
        let mut value = serde_json::to_value(&f.bundle).unwrap();
        value["releases"]["oracle-runtime"]["artifacts"]["app"][field] =
            json!(if field == "kind" {
                "helm_chart"
            } else {
                "chart"
            });
        f.write("bundle.json", value.to_string());
        let out = f.qualify("publish", false).output().unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stderr).contains("bundle"));
        assert!(f.calls().is_empty());
    }
    let original = f.bundle.to_canonical_json();
    let value = serde_json::to_string(
        &f.bundle.releases.values().next().unwrap().evidence
            [&ess_deployment::EvidenceKind::Conformance],
    )
    .unwrap();
    let raw = original.replacen(
        "\"conformance\":",
        &format!("\"conformance\":{value},\"conformance\":"),
        1,
    );
    assert_ne!(raw, original);
    f.write("bundle.json", raw);
    let out = f.qualify("publish", false).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("duplicate"));
    assert!(f.calls().is_empty());
}
#[test]
fn t12_consistently_changed_artifact_platform_and_source_context_remains_execution_unverified() {
    let f = Fixture::new();
    let mut bundle = f.bundle.clone();
    for release in bundle.releases.values_mut() {
        release.source_commit = "b".repeat(40);
        for artifact in release.artifacts.values_mut() {
            artifact.digest = Digest::of_bytes(b"changed declared artifact");
            for digest in artifact.platforms.values_mut() {
                *digest = Digest::of_bytes(b"changed declared child");
            }
        }
    }
    bundle.validate().unwrap();
    f.write("bundle.json", bundle.to_canonical_json());
    let out = f.qualify("publish", false).output().unwrap();
    let err = success(&out);
    assert!(err.contains("artifact execution: unverified"));
    assert!(err.contains(Digest::of_bytes(b"changed declared artifact").as_str()));
    assert!(err.contains(Digest::of_bytes(b"changed declared child").as_str()));
    assert!(err.contains(&"b".repeat(40)));
}
#[test]
fn t11_neither_expected_variant_and_noncanonical_context_are_refused_without_tools() {
    let f = Fixture::new();
    for route in ["check-conformance", "publish-conformance", "publish"] {
        let original = f.qualify(route, false);
        let mut remove_next = false;
        let argv: Vec<_> = original
            .get_args()
            .filter(|arg| {
                if remove_next {
                    remove_next = false;
                    return false;
                }
                if *arg == "--expected-suite" {
                    remove_next = true;
                    return false;
                }
                true
            })
            .map(std::ffi::OsStr::to_owned)
            .collect();
        let out = f.command().args(argv).output().unwrap();
        assert_eq!(out.status.code(), Some(2));
    }
    for file in ["component.json", "build.json", "runtime.json"] {
        let raw = fs::read_to_string(f.root.join(file)).unwrap();
        f.write(file, format!("{raw} "));
        let out = f.qualify("check-conformance", false).output().unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stderr).contains("canonical"));
        f.write(file, raw);
    }
    assert!(f.calls().is_empty());
}

#[test]
fn t15_caller_files_beneath_release_output_get_distinct_action_snapshots() {
    let f = Fixture::new();
    let expected = report(&f.input);
    f.write("target/release/conformance-report.json", &expected);
    f.write(
        "target/release/expected-input.json",
        f.input.selected().original_json(),
    );
    let mut c = f.action();
    c.env(
        "CONFORMANCE_REPORT",
        "target/release/conformance-report.json",
    )
    .env("CONFORMANCE_SUITE", "target/release/expected-input.json");
    let out = f.action_output(&mut c);
    success(&out);
    assert_eq!(
        fs::read(f.root.join("target/release/conformance-report.json")).unwrap(),
        expected
    );
}

#[test]
fn t10_complete_but_empty_unfiltered_selection_is_inconclusive() {
    let f = Fixture::new();
    let mut value = suite_document();
    value["scenarios"] = json!({});
    value["coverage"]["authored"] = json!([]);
    value["coverage"]["authored_sources"] = json!({});
    value["coverage"]["counts"]["authored"] = json!(0);
    let input = direct(&value);
    assert!(input.selected().coverage().unwrap().is_complete());
    let raw = report(&input);
    assert_eq!(
        CountReport::from_json(std::str::from_utf8(&raw).unwrap(), input.selected())
            .unwrap()
            .conformance_status(),
        CountStatus::Inconclusive
    );
    f.set_input(&input, false);
    f.write("report.json", raw);
    f.refusal(false, "must be passed");
}

#[test]
fn adversary_component_release_unit_mismatch_stops_action_before_generic_check() {
    let f = Fixture::new();
    let mut command = f.action();
    let source = COMPONENT_SOURCE.replace("runtime: oracle-runtime", "runtime: unrelated-runtime");
    let component = ess_deployment::compile_component(
        &ess_deployment::ComponentSpec::from_yaml(&source).unwrap(),
    )
    .unwrap();
    assert_eq!(
        component.release_units().runtime.as_str(),
        "unrelated-runtime"
    );
    assert!(f
        .bundle
        .build
        .outputs()
        .values()
        .all(|output| { output.release_unit != component.release_units().runtime }));
    f.write("component.yaml", source);
    let out = f.action_output(&mut command);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        !f.root.join("check-ran").exists() && f.calls().is_empty(),
        "incompatible component/build release units must refuse at initial qualification; \
         generic_check_ran={}, external_calls={}, fixture={}, stderr={}",
        f.root.join("check-ran").exists(),
        f.calls().len(),
        f.root.display(),
        String::from_utf8_lossy(&out.stderr)
    );
}
#[test]
fn adversary_component_release_unit_mismatch_refuses_positive_cli_routes() {
    let f = Fixture::new();
    let source = COMPONENT_SOURCE.replace("runtime: oracle-runtime", "runtime: unrelated-runtime");
    let component = ess_deployment::compile_component(
        &ess_deployment::ComponentSpec::from_yaml(&source).unwrap(),
    )
    .unwrap();
    f.write("component.json", component.to_canonical_json());
    let mut results = Vec::new();
    for route in ["check-conformance", "publish-conformance"] {
        let out = f.qualify(route, false).output().unwrap();
        f.write(&format!("{route}.stdout.raw"), &out.stdout);
        f.write(&format!("{route}.stderr.raw"), &out.stderr);
        f.write(&format!("{route}.status"), out.status.to_string());
        results.push((route, out.status.code(), out.stdout.is_empty()));
    }
    assert!(
        results
            .iter()
            .all(|(_, status, empty)| *status == Some(1) && *empty)
            && f.calls().is_empty(),
        "component/build release-unit mismatch must refuse positive qualification before ORAS; \
         results={results:?}, external_calls={}, fixture={}",
        f.calls().len(),
        f.root.display()
    );
}

#[test]
fn adversary_action_keeps_original_snapshots_when_generic_check_replaces_caller_files() {
    let f = Fixture::new();
    let expected = fs::read(f.root.join("report.json")).unwrap();
    let mut command = f.action();
    command.env(
        "CHECK_COMMAND",
        "printf changed > report.json; printf changed > expected.json; printf checked > check-ran",
    );
    let out = f.action_output(&mut command);
    success(&out);
    assert_eq!(fs::read(f.root.join("report.json")).unwrap(), b"changed");
    assert_eq!(fs::read(f.root.join("expected.json")).unwrap(), b"changed");
    let calls = f.calls();
    let upload = calls
        .iter()
        .find(|call| {
            fs::read_to_string(call.join("argv"))
                .unwrap()
                .contains(CONFORMANCE)
        })
        .unwrap();
    assert_eq!(fs::read(upload.join("payload.raw")).unwrap(), expected);
}

fn fix1_context(f: &mut Fixture, component_source: &str, build_yaml: &str) {
    f.bundle.component = ess_deployment::compile_component(
        &ess_deployment::ComponentSpec::from_yaml(component_source).unwrap(),
    )
    .unwrap();
    f.bundle.build =
        ess_deployment::compile_build(&ess_deployment::BuildSpec::from_yaml(build_yaml).unwrap())
            .unwrap();
    let mut runtime = serde_json::to_value(&f.bundle.runtime).unwrap();
    runtime["build_digest"] = json!(f.bundle.build.digest());
    f.bundle.runtime = ess_deployment::RuntimeIr::from_json(&runtime.to_string()).unwrap();
    f.bundle
        .runtime
        .validate_against_build(&f.bundle.build)
        .unwrap();
    for release in f.bundle.releases.values_mut() {
        release.build_digest = f.bundle.build.digest();
        release.runtime_digest = f.bundle.runtime.digest();
    }
    f.write("component.yaml", component_source);
    f.write("build.yaml", build_yaml);
    f.write("component.json", f.bundle.component.to_canonical_json());
    f.write("build.json", f.bundle.build.to_canonical_json());
    f.write("runtime.json", f.bundle.runtime.to_canonical_json());
}

fn fix1_refuses_initial_context(component_source: &str, build_yaml: &str, reason: &str) {
    let mut f = Fixture::new();
    let mut action = f.action();
    fix1_context(&mut f, component_source, build_yaml);
    let mut results = Vec::new();
    for route in ["check-conformance", "publish-conformance"] {
        let out = f.qualify(route, false).output().unwrap();
        f.write(&format!("{route}.stdout.raw"), &out.stdout);
        f.write(&format!("{route}.stderr.raw"), &out.stderr);
        f.write(&format!("{route}.status"), out.status.to_string());
        results.push(out);
    }
    results.push(f.action_output(&mut action));
    assert!(
        results.iter().enumerate().all(|(index, out)| out.status.code() == Some(1)
            && (index == 2 || out.stdout.is_empty())
            && String::from_utf8_lossy(&out.stderr).contains(reason))
            && !f.root.join("check-ran").exists()
            && f.calls().is_empty(),
        "unbuildable component units must refuse before checks or tools: reason={reason}, results={results:?}, fixture={}",
        f.root.display()
    );
}

#[test]
fn fix1_missing_chart_release_unit_refuses_before_checks_or_tools() {
    fix1_refuses_initial_context(
        &COMPONENT_SOURCE.replace("chart: oracle-chart", "chart: unrelated-chart"),
        &build_source(),
        "component chart release unit unrelated-chart has no HelmChart build output",
    );
}

#[test]
fn fix1_runtime_release_unit_requires_an_owned_oci_output() {
    let build = build_source().replace("release_unit: oracle-runtime", "release_unit: another-runtime")
        + "  - {name: runtime-file, release_unit: oracle-runtime, node: chart-file, kind: binary}\n";
    fix1_refuses_initial_context(
        COMPONENT_SOURCE,
        &build,
        "component runtime release unit oracle-runtime has no OciImage build output",
    );
}

#[test]
fn fix1_chart_release_unit_requires_an_owned_chart_output() {
    fix1_refuses_initial_context(
        COMPONENT_SOURCE,
        &build_source().replace("kind: helm_chart", "kind: archive"),
        "component chart release unit oracle-chart has no HelmChart build output",
    );
}

#[test]
fn fix1_mixed_and_additional_build_outputs_preserve_qualification_and_publication() {
    let mut f = Fixture::new();
    let build = build_source()
        + "  - {name: runtime-file, release_unit: oracle-runtime, node: chart-file, kind: binary}\n"
        + "  - {name: chart-archive, release_unit: oracle-chart, node: chart-file, kind: archive}\n"
        + "  - {name: extra-file, release_unit: unrelated-unit, node: chart-file, kind: binary}\n";
    fix1_context(&mut f, COMPONENT_SOURCE, &build);
    for output in f.bundle.build.outputs().values() {
        if let Some(release) = f.bundle.releases.get_mut(&output.release_unit) {
            release
                .artifacts
                .entry(output.name.clone())
                .or_insert_with(|| ess_deployment::Artifact {
                    build_output: output.name.clone(),
                    kind: output.kind.into(),
                    reference: format!("registry.example/{}:fixture", output.name),
                    digest: Digest::of_bytes(b"declared additional artifact"),
                    platforms: std::collections::BTreeMap::new(),
                });
        }
    }
    f.bundle.validate().unwrap();
    f.write("bundle.json", f.bundle.to_canonical_json());
    for route in ["check-conformance", "publish-conformance", "publish"] {
        let out = f.qualify(route, false).output().unwrap();
        f.write(&format!("{route}.stdout.raw"), &out.stdout);
        f.write(&format!("{route}.stderr.raw"), &out.stderr);
        f.write(&format!("{route}.status"), out.status.to_string());
        let stderr = success(&out);
        assert!(stderr.contains("conformance: passed"));
    }
}

fn adversary2_record(f: &Fixture, label: &str, command: &mut Command) -> Output {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let started = std::time::SystemTime::now();
    let child = command.spawn().unwrap();
    let pid = child.id();
    f.write(
        &format!("{label}.started"),
        format!("pid {pid}\nstart {started:?}\n"),
    );
    let out = child.wait_with_output().unwrap();
    f.write(
        &format!("{label}.exit"),
        format!(
            "pid {pid}\nstart {started:?}\nend {:?}\nstatus {}\n",
            std::time::SystemTime::now(),
            out.status
        ),
    );
    f.write(&format!("{label}.stdout.raw"), &out.stdout);
    f.write(&format!("{label}.stderr.raw"), &out.stderr);
    out
}

#[test]
fn adversary2_coherent_wrong_model_bundle_refuses_qualification_after_plain_consistency() {
    // B01/B03: rehashing a valid /1 graph cannot make a different deployment model
    // satisfy the independently supplied model/report. Reach qualification after admission.
    for variant in ["system", "semantic-version", "semantic-digest"] {
        let f = Fixture::new();
        let mut bundle = f.bundle.clone();
        if variant == "semantic-digest" {
            let mut value = serde_json::to_value(&bundle.runtime).unwrap();
            value["semantic_digest"] = json!(Digest::of_bytes(b"another semantic model"));
            bundle.runtime = ess_deployment::RuntimeIr::from_json(&value.to_string()).unwrap();
            for release in bundle.releases.values_mut() {
                release.semantic_digest = bundle.runtime.semantic_digest().clone();
                release.runtime_digest = bundle.runtime.digest();
            }
        } else {
            let mut value = serde_json::to_value(&bundle.component).unwrap();
            if variant == "system" {
                value["system"] = json!("other");
                for release in bundle.releases.values_mut() {
                    release.system = "other".parse().unwrap();
                }
            } else {
                value["semantic_version"] = json!("v2");
            }
            bundle.component = ess_deployment::ComponentIr::from_json(&value.to_string()).unwrap();
        }
        bundle.validate().unwrap();
        let canonical = bundle.to_canonical_json();
        assert_ne!(bundle.digest(), f.bundle.digest());
        assert_eq!(
            ReleaseBundle::from_yaml(&serde_yaml::to_string(&bundle).unwrap()).unwrap(),
            bundle
        );
        f.write("bundle.json", &canonical);
        let plain = adversary2_record(
            &f,
            "plain-consistency",
            f.command().args([
                "release",
                "verify-bundle",
                "--path",
                "bundle.json",
                "--format",
                "json",
            ]),
        );
        assert!(success(&plain).contains("conformance: not assessed"));
        assert_eq!(plain.stdout, canonical.as_bytes());
        let out = adversary2_record(&f, "qualified-bundle", &mut f.qualify("publish", false));
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{variant}: {err}");
        assert!(
            err.contains("compiled deployment context differs from the explicit model identity"),
            "{variant}: {err}"
        );
        assert!(!err.contains("conformance: passed"));
        assert!(out.stdout.is_empty());
        assert!(
            f.calls().is_empty(),
            "{variant} crossed ORAS before qualification"
        );
    }
}

fn adversary2_write_changed_model(f: &Fixture) {
    use ess_domain::{
        spec::{RawSpecFile, Specification},
        system::Source,
    };
    let original = fs::read_to_string(f.root.join("model/domains/order.yaml")).unwrap();
    let changed = original.replacen("when: weight_grams >= 0", "when: weight_grams >= 1", 1);
    assert_ne!(changed, original);
    let mut sources = ess_compiler::source::SourceMap::new();
    let parsed: Vec<_> = [
        "system.yaml",
        "components.yaml",
        "domains/order.yaml",
        "domains/dispatch.yaml",
    ]
    .into_iter()
    .map(|name| {
        let raw = if name == "domains/order.yaml" {
            changed.clone()
        } else {
            fs::read_to_string(f.root.join("model").join(name)).unwrap()
        };
        let document = RawSpecFile::parse(&raw).unwrap();
        sources.insert(name, raw);
        (Source::new(name), document)
    })
    .collect();
    let changed_model =
        ess_compiler::resolve::compile(&Specification::assemble(parsed).unwrap(), &sources)
            .unwrap();
    let provenance = SuiteProvenance::of(&changed_model);
    let original_provenance = &f.input.selected().suite().provenance;
    assert_eq!(provenance.system, original_provenance.system);
    assert_eq!(
        provenance.specification_version,
        original_provenance.specification_version
    );
    assert_ne!(provenance.spec_digest, original_provenance.spec_digest);
    f.write("replacement.yaml", changed);
}

#[test]
fn adversary2_action_rechecks_current_model_and_deployment_before_evidence_upload() {
    // B06.4: report/input pins alone are insufficient when the generic check changes
    // another ordinary supplied input. The second gate must reload that current context.
    for variant in ["model", "component", "runtime"] {
        let f = Fixture::new();
        let mut command = f.action();
        let (check, reason) = match variant {
            "model" => {
                adversary2_write_changed_model(&f);
                (
                    "cp replacement.yaml model/domains/order.yaml; printf checked > check-ran",
                    "conformance model identity differs",
                )
            }
            "component" => {
                let source = COMPONENT_SOURCE.replace("chart: oracle-chart", "chart: other-chart");
                let component = ess_deployment::compile_component(
                    &ess_deployment::ComponentSpec::from_yaml(&source).unwrap(),
                )
                .unwrap();
                f.write("replacement.json", component.to_canonical_json());
                (
                    "cp replacement.json target/release/component.ir.json; printf checked > check-ran",
                    "component chart release unit other-chart has no HelmChart build output",
                )
            }
            "runtime" => {
                let mut value = serde_json::to_value(&f.bundle.runtime).unwrap();
                value["semantic_digest"] =
                    json!(Digest::of_bytes(b"changed runtime semantic context"));
                let runtime = ess_deployment::RuntimeIr::from_json(&value.to_string()).unwrap();
                runtime.validate_against_build(&f.bundle.build).unwrap();
                f.write("replacement.json", runtime.to_canonical_json());
                (
                    "cp replacement.json runtime.json; printf checked > check-ran",
                    "compiled deployment context differs from the explicit model identity",
                )
            }
            _ => unreachable!(),
        };
        command.env("CHECK_COMMAND", check);
        let out = adversary2_record(&f, "action-recheck", &mut command);
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{variant}: {err}");
        assert!(err.contains(reason), "{variant}: {err}");
        assert_eq!(
            err.matches("conformance: passed for the supplied exact declared selection")
                .count(),
            1
        );
        assert!(
            !err.contains("byte pin mismatch"),
            "must reach current context: {err}"
        );
        assert!(f.root.join("check-ran").exists());
        let calls = f.calls();
        assert_eq!(
            calls.len(),
            13,
            "{variant} must reach the pre-evidence gate"
        );
        for call in calls {
            assert_eq!(fs::read_to_string(call.join("status")).unwrap(), "0");
        }
        f.assert_no_publication();
        assert!(!f.root.join("summary.md").exists());
    }
}

#[test]
fn adversary2_post_admission_model_and_deployment_replacement_keeps_owned_context() {
    // B04: the child synchronization happens after admission and before consumption.
    // Replace caller files only; the private staging directory is never modified.
    for route in ["publish-conformance", "publish"] {
        let f = Fixture::new();
        let expected = if route == "publish" {
            f.bundle.to_canonical_json().into_bytes()
        } else {
            fs::read(f.root.join("report.json")).unwrap()
        };
        let listener = UnixListener::bind(f.root.join("context.sock")).unwrap();
        listener.set_nonblocking(true).unwrap();
        let mut command = f.qualify(route, false);
        command
            .env("ESS_RELEASE_SYNC", f.root.join("context.sock"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let started = std::time::SystemTime::now();
        let mut child = command.spawn().unwrap();
        let pid = child.id();
        f.write(
            "owned-context.started",
            format!("pid {pid}\nstart {started:?}\n"),
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let mut socket = loop {
            match listener.accept() {
                Ok((socket, _)) => break socket,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(error) => panic!("context synchronization: {error}"),
            }
            if child.try_wait().unwrap().is_some() || std::time::Instant::now() > deadline {
                let _ = child.kill();
                let out = child.wait_with_output().unwrap();
                f.write("owned-context.stdout.raw", &out.stdout);
                f.write("owned-context.stderr.raw", &out.stderr);
                f.write(
                    "owned-context.exit",
                    format!(
                        "pid {pid}\nstart {started:?}\nend {:?}\nstatus {}\n",
                        std::time::SystemTime::now(),
                        out.status
                    ),
                );
                panic!("publisher failed to synchronize: {out:?}");
            }
            std::thread::yield_now();
        };
        socket
            .set_read_timeout(Some(std::time::Duration::from_secs(30)))
            .unwrap();
        let mut ready = [0];
        socket.read_exact(&mut ready).unwrap();
        assert_eq!(ready, [1]);
        for file in [
            "model/system.yaml",
            "component.json",
            "build.json",
            "runtime.json",
            "bundle.json",
        ] {
            f.write(file, b"invalid replacement after admission");
        }
        socket.write_all(&[2]).unwrap();
        let out = child.wait_with_output().unwrap();
        f.write(
            "owned-context.exit",
            format!(
                "pid {pid}\nstart {started:?}\nend {:?}\nstatus {}\n",
                std::time::SystemTime::now(),
                out.status
            ),
        );
        f.write("owned-context.stdout.raw", &out.stdout);
        f.write("owned-context.stderr.raw", &out.stderr);
        let err = success(&out);
        for identity in [
            f.bundle.component.digest(),
            f.bundle.build.digest(),
            f.bundle.runtime.digest(),
        ] {
            assert!(
                err.contains(identity.as_str()),
                "admitted identity omitted: {err}"
            );
        }
        assert!(err.contains(f.input.selected().suite().provenance.spec_digest.as_str()));
        assert_eq!(f.calls().len(), 1);
        assert_eq!(
            fs::read(f.calls()[0].join("payload.raw")).unwrap(),
            expected
        );
        assert_eq!(
            fs::read(f.root.join("model/system.yaml")).unwrap(),
            b"invalid replacement after admission"
        );
    }
}

#[test]
fn discovery_manifest_qualifies_all_release_model_callers_and_stops_before_oras() {
    let f = Fixture::new();
    let paths = [
        "system.yaml",
        "components.yaml",
        "domains/order.yaml",
        "domains/dispatch.yaml",
    ];
    for path in paths {
        f.write(
            &format!("model/selected/{path}"),
            fs::read(f.root.join("model").join(path)).unwrap(),
        );
    }
    f.write("model/ess-inputs.yaml",serde_json::to_vec(&json!({"format":"ess-inputs/1","specification":paths.map(|p|format!("selected/{p}")),"scenarios":["unopened/scenario"]})).unwrap());
    // Poisoned unlisted copies would make a recursive walk fail or duplicate declarations.
    f.write("model/generated.yaml", "[invalid");
    for route in ["check-conformance", "publish-conformance", "publish"] {
        for flat in [false, true] {
            let original = f.qualify(route, false);
            let mut c = f.command();
            let args = original.get_args().collect::<Vec<_>>();
            c.args(if flat { &args[1..] } else { &args });
            success(&adversary2_record(
                &f,
                &format!("discovery-{route}-{flat}"),
                &mut c,
            ));
        }
    }
    let calls = f.calls().len();
    f.write(
        "model/ess-inputs.yaml",
        br#"{"format":"ess-inputs/1","specification":["absent"],"scenarios":[]}"#,
    );
    for route in ["check-conformance", "publish-conformance", "publish"] {
        let out = adversary2_record(
            &f,
            &format!("discovery-refused-{route}"),
            &mut f.qualify(route, false),
        );
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stderr).contains("ess-inputs.yaml"));
        assert!(out.stdout.is_empty());
        assert_eq!(f.calls().len(), calls);
    }
}

#[test]
fn discovery_runtime_system_acquires_manifest_before_realization_and_output() {
    let f = Fixture::new();
    let paths = [
        "system.yaml",
        "components.yaml",
        "domains/order.yaml",
        "domains/dispatch.yaml",
    ];
    for path in paths {
        f.write(
            &format!("selected/{path}"),
            fs::read(f.root.join("model").join(path)).unwrap(),
        );
    }
    f.write(
        "selected/ess-inputs.yaml",
        serde_json::to_vec(&json!({"format":"ess-inputs/1","specification":paths,"scenarios":[]}))
            .unwrap(),
    );
    f.write("runtime-input.json",serde_json::to_vec(&json!({"format":"ess-runtime/1","runtime":"discovery","semantic_digest":format!("sha256:{}",model().source_digest()),"realization_digest":format!("sha256:{}","0".repeat(64)),"build_digest":f.bundle.build.digest(),"processes":[],"containers":[],"workloads":[]})).unwrap());
    for flat in [false, true] {
        let mut c = f.command();
        if !flat {
            c.arg("specify");
        }
        c.args([
            "runtime",
            "compile",
            "--path",
            "runtime-input.json",
            "--system",
            "selected",
            "--realization",
            "intentionally-missing-realization.json",
            "--build-ir",
            "build.json",
            "--out",
            "must-not-exist.json",
        ]);
        let out = adversary2_record(&f, &format!("discovery-runtime-{flat}"), &mut c);
        assert_eq!(out.status.code(), Some(1));
        let text = String::from_utf8_lossy(&out.stderr);
        assert!(
            text.contains("reading intentionally-missing-realization.json"),
            "{out:?}"
        );
        assert!(!f.root.join("must-not-exist.json").exists());
    }
}
