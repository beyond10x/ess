//! Implementation emitters must not silently omit a temporal disclosure policy.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, SynthesisPlan, Target, TargetFailure, TargetFailureCode};

const MODEL: &str = "format: ess/21\nsystem: credentials\nversion: v1\ndomain: credentials.api\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: String}]\n    outcomes: [{name: issued, returns: true, one_time_response: [secret]}]\n";

fn ir(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("policy.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn named(failure: &TargetFailure) {
    assert!(failure.causes().iter().any(|cause| cause.code()
        == TargetFailureCode::MissingRepresentation
        && cause
            .sources()
            .iter()
            .any(|source| source
                == "commands.credentials.api.Issue.outcomes.issued.one_time_response")));
    assert!(failure
        .to_canonical_json()
        .contains("atomic durable consumption"));
}

#[test]
fn all_implementation_entry_points_refuse_an_unimplemented_disclosure_policy() {
    let model = ir(MODEL);
    for target in [Target::Rust, Target::Go, Target::Web, Target::Clap] {
        named(
            &synthesize_for(&model, target)
                .err()
                .expect("no incomplete implementation"),
        );
    }
    let plan = SynthesisPlan::of(&model);
    for failure in [
        ess_synth::rust::workspace(&model, &plan).err(),
        ess_synth::rust::single_crate(&model, &plan).err(),
        ess_synth::go::workspace(&model, &plan).err(),
        ess_synth::web::workspace(&model, &plan).err(),
        ess_synth::clap::workspace(&model, &plan).err(),
    ] {
        named(&failure.expect("direct entry must enforce the same refusal"));
    }
}

#[test]
fn ordinary_direct_returns_keep_their_existing_implementation_path() {
    let source = MODEL
        .replace(", one_time_response: [secret]", "")
        .replace("ess/21", "ess/20");
    let model = ir(&source);
    for target in [Target::Rust, Target::Go] {
        assert!(synthesize_for(&model, target).is_ok(), "{target:?}");
    }
}

#[test]
fn direct_browser_catalog_refuses_dispatch_without_losing_policy_location() {
    let wired = format!("{MODEL}components:\n  - component: issuer\n    owns:\n      domains: [credentials.api]\n    accepts:\n      commands: [credentials.api.Issue]\n");
    for marked in [true, false] {
        let source = if marked {
            wired.clone()
        } else {
            wired.replace(", one_time_response: [secret]", "")
        };
        let model = ir(&source);
        let catalog: serde_json::Value = serde_json::from_str(
            ess_synth::web::browser_catalog(&model, &SynthesisPlan::of(&model)).as_json(),
        )
        .unwrap();
        assert_eq!(catalog["format"], "ess-browser-catalog/1");
        let command = &catalog["commands"][0];
        assert_eq!(command["component"], "issuer");
        assert_eq!(command["dispatchable"], !marked);
        if marked {
            assert_eq!(command["behavior"]["disposition"], "refused");
            let refusal = command["refusal"].as_str().unwrap();
            for required in [
                "one_time_response",
                "issued: secret",
                "atomic durable consumption",
            ] {
                assert!(refusal.contains(required), "{refusal}");
            }
        } else {
            assert!(command.get("refusal").is_none());
        }
    }
}
