//! A structural projection preserves temporal obligations without claiming enforcement.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = "format: ess/21\nsystem: credentials\nversion: v1\ndomain: credentials.api\ncommands:\n  - name: credentials.api.Issue\n    response: [{name: secret, type: String}]\n    outcomes: [{name: issued, returns: true, one_time_response: [secret]}]\ncomponents:\n  - component: issuer\n    owns: {domains: [credentials.api]}\n    accepts: {commands: [credentials.api.Issue]}\n    reached_by: network\n";

fn ir(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("policy.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn assert_policy(value: &serde_json::Value) {
    let policies = value["x-ess-one-time-response"]
        .as_array()
        .expect("policy retained");
    assert_eq!(policies.len(), 1);
    assert_eq!(policies[0]["command"], "credentials.api.Issue");
    assert_eq!(policies[0]["outcome"], "issued");
    assert_eq!(policies[0]["fields"], serde_json::json!(["secret"]));
    let obligation = policies[0]["obligation"].as_str().unwrap();
    for required in [
        "do not enforce",
        "atomic durable consumption",
        "restart",
        "concurrency",
    ] {
        assert!(obligation.contains(required), "{required}: {obligation}");
    }
}

#[test]
fn every_document_projection_preserves_policy_and_implementation_obligations() {
    let model = ir(MODEL);
    let outputs = ess_gen::generate_all(&model).unwrap();
    for (generator, path) in [
        ("openapi", "issuer.yaml"),
        ("asyncapi", "issuer.yaml"),
        ("schema", "responses/credentials.api.Issue.schema.json"),
        ("schema", "commands/credentials.api.Issue.schema.json"),
    ] {
        let output = &outputs[&format!("{generator}/{path}")].contents;
        let value = if generator == "schema" {
            serde_json::from_str(output).unwrap()
        } else {
            serde_yaml::from_str(output).unwrap()
        };
        assert_policy(&value);
    }
    let component = model.components().values().next().unwrap();
    assert_policy(&serde_json::from_str(&ess_gen::openapi::json(&model, component)).unwrap());
    for generator in ["docs", "site"] {
        let artifacts =
            ess_gen::artifact::run(ess_gen::generator(generator).unwrap().as_ref(), &model)
                .unwrap();
        assert!(
            artifacts.values().any(|artifact| artifact
                .contents
                .contains("One-time response fields:")
                && artifact.contents.contains("atomic durable consumption")),
            "{generator}"
        );
    }
    assert_eq!(outputs, ess_gen::generate_all(&model).unwrap());
}

#[test]
fn unmarked_models_have_no_new_projection_annotations() {
    let source = MODEL
        .replace(", one_time_response: [secret]", "")
        .replace("ess/21", "ess/20");
    let outputs = ess_gen::generate_all(&ir(&source)).unwrap();
    assert!(outputs.values().all(|artifact| !artifact
        .contents
        .contains("x-ess-one-time-response")
        && !artifact.contents.contains("One-time response fields:")));
}
