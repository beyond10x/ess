//! The shared nested-response target and native reader execute in an actual WASM module.
use serde_json::{json, Value};
use std::{
    fmt::Write as _,
    path::{Path, PathBuf},
    process::Command,
};

const HOST: &str = r#"
use std::cell::RefCell;
use ess_conformance::Runner;
thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static OUTPUT: RefCell<String> = const { RefCell::new(String::new()) };
}
#[no_mangle] pub extern "C" fn ess_input_reserve(length:u32)->u32 {
    INPUT.with(|held|{let mut held=held.borrow_mut();*held=vec![0;length as usize];held.as_mut_ptr() as u32})
}
#[no_mangle] pub extern "C" fn ess_dispatch()->u32 {
    let answer=INPUT.with(|held|{
        let request:serde_json::Value=serde_json::from_slice(&held.borrow()).unwrap();
        let admitted=match AdmittedSuite::from_json(request["suite"].as_str().unwrap()) {
            Ok(value)=>value,
            Err(error)=>return serde_json::json!({"refused":error.to_string(),"callbacks":0}).to_string(),
        };
        let fault=match request["fault"].as_u64().unwrap(){0=>Fault::None,1=>Fault::Event,2=>Fault::Response,3=>Fault::GeneratedSibling,_=>unreachable!()};
        let target=Backend::new(fault);
        let run=Runner::for_suite(admitted.suite()).run_admitted(&admitted,&target);
        let report=ess_conformance::counts::CountReport::from_run(&run,&admitted).unwrap();
        serde_json::json!({"report":serde_json::from_str::<serde_json::Value>(&report.to_canonical_json().unwrap()).unwrap(),"calls":target.calls.get(),"callbacks":target.callbacks.get()}).to_string()
    });
    OUTPUT.with(|held|{*held.borrow_mut()=answer;held.borrow().as_ptr() as u32})
}
#[no_mangle] pub extern "C" fn ess_output_len()->u32 {OUTPUT.with(|held|held.borrow().len() as u32)}
"#;

fn run(command: &mut Command) -> std::process::Output {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn build_host() -> (PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("ess-nested-wasm-{}", std::process::id()));
    std::fs::create_dir_all(root.join("src")).unwrap();
    let conformance = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../verify/ess-conformance")
        .canonicalize()
        .unwrap();
    let mut manifest="[package]\nname=\"nested-response-host\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[workspace]\n[lib]\ncrate-type=[\"cdylib\"]\n[dependencies]\nserde_json={version=\"1\",features=[\"arbitrary_precision\"]}\n".to_owned();
    for (name, path) in [
        ("ess-conformance", conformance.clone()),
        ("ess-domain", conformance.join("../../specify/ess-domain")),
        (
            "ess-compiler",
            conformance.join("../../specify/ess-compiler"),
        ),
        (
            "ess-primitives",
            conformance.join("../../specify/ess-primitives"),
        ),
    ] {
        writeln!(
            manifest,
            "{name}={{path={}}}",
            serde_json::to_string(&path.canonicalize().unwrap()).unwrap()
        )
        .unwrap();
    }
    std::fs::write(root.join("Cargo.toml"), manifest).unwrap();
    let fixture =
        include_str!("../../../verify/ess-conformance/tests/support_nested_response/mod.rs");
    let source = fixture
        .lines()
        .filter(|line| !line.starts_with("pub mod ") && !line.starts_with("//!"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(root.join("src/lib.rs"), format!("{source}\n{HOST}")).unwrap();
    run(Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(&root));
    let cache =
        std::env::var_os("CARGO_TARGET_DIR").map_or_else(|| root.join("target"), PathBuf::from);
    run(Command::new("cargo")
        .args([
            "build",
            "--offline",
            "--locked",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .arg("--target-dir")
        .arg(&cache)
        .current_dir(&root));
    let bridge = include_str!("../src/web/page.rs")
        .split("const GLUE_BODY: &str = r#\"")
        .nth(1)
        .unwrap()
        .split("\"#;")
        .next()
        .unwrap();
    std::fs::write(root.join("bridge.mjs"),format!("{bridge}\nexport const EXPORTS=['ess_input_reserve','ess_dispatch','ess_output_len']; export const REALIZE='ess_realize';\n")).unwrap();
    // Reuse the existing actual-WASM bridge driver byte-for-byte.
    let driver = include_str!("../../../verify/ess-conformance/tests/support_initial_state/mod.rs")
        .split_once("root.join(\"driver.mjs\"),\n        r\"")
        .unwrap()
        .1
        .split_once("\",\n")
        .unwrap()
        .0;
    std::fs::write(root.join("driver.mjs"), driver).unwrap();
    std::fs::write(
        root.join("cases.json"),
        json!({"faults":[0,1,2,3],"item_case":0}).to_string(),
    )
    .unwrap();
    (root, cache)
}

#[test]
fn actual_wasm_observes_independent_response_event_and_generated_sibling() {
    let (root, cache) = build_host();
    let fixture =
        include_str!("../../../verify/ess-conformance/tests/support_nested_response/mod.rs");
    let model = fixture
        .split_once("pub const MODEL: &str = r\"")
        .unwrap()
        .1
        .split_once("\";")
        .unwrap()
        .0;
    let spec = ess_domain::Specification::assemble([(
        ess_domain::system::Source::new("nested.yaml"),
        ess_domain::spec::RawSpecFile::parse(model).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::resolve::compile(&spec, &ess_compiler::source::SourceMap::new()).unwrap();
    let suite = ess_conformance::synthesize(&ir).suite;
    let original = suite.to_compact_json().unwrap();
    std::fs::write(root.join("suite.json"), &original).unwrap();
    let execute = || {
        run(Command::new("node")
            .arg("driver.mjs")
            .arg(cache.join("wasm32-unknown-unknown/debug/nested_response_host.wasm"))
            .current_dir(&root))
    };
    let answers: Value = serde_json::from_slice(&execute().stdout).unwrap();
    for (index, answer) in answers.as_array().unwrap().iter().enumerate() {
        assert_eq!(answer["calls"], 1);
        assert_eq!(
            answer["report"]["counts"]["passed"],
            usize::from(index == 0)
        );
        assert_eq!(
            answer["report"]["counts"]["failed"],
            usize::from(index != 0)
        );
    }
    std::fs::write(root.join("answers.json"), answers.to_string()).unwrap();
    let mut malformed: Value = serde_json::from_str(&original).unwrap();
    for scenario in malformed["scenarios"].as_object_mut().unwrap().values_mut() {
        for step in scenario["steps"].as_array_mut().unwrap() {
            if step["step"] == "expect_response_payload" {
                step["response"]["nested"] = Value::Null;
            }
        }
    }
    std::fs::write(root.join("suite.json"), malformed.to_string()).unwrap();
    let answers: Value = serde_json::from_slice(&execute().stdout).unwrap();
    for answer in answers.as_array().unwrap() {
        assert!(answer["refused"].is_string());
        assert_eq!(answer["callbacks"], 0);
    }
    std::fs::write(root.join("refusals.json"), answers.to_string()).unwrap();
}
