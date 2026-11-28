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

/// What Go, web and the command-line target say when they refuse a marked outcome, word for word.
const REFUSAL: &str = "one_time_response requires implementation-owned atomic durable consumption \
                       and fresh issuance; this code target cannot implement that policy";

/// The refusal names the marked outcome with the exact text. The refusal document also embeds
/// the plan, whose contract carries the one-time clause for every target, so the document's text
/// alone does not show the refusal is this one.
fn named(failure: &TargetFailure) {
    assert!(
        failure.causes().iter().any(|cause| cause.code()
            == TargetFailureCode::MissingRepresentation
            && cause.detail() == REFUSAL
            && cause.sources().iter().any(|source| source
                == "commands.credentials.api.Issue.outcomes.issued.one_time_response")),
        "{}",
        failure.to_canonical_json()
    );
}

#[test]
fn all_implementation_entry_points_refuse_an_unimplemented_disclosure_policy() {
    let model = ir(MODEL);
    for target in [Target::Go, Target::Web, Target::Clap] {
        named(
            &synthesize_for(&model, target)
                .err()
                .expect("no incomplete implementation"),
        );
    }
    let plan = SynthesisPlan::of(&model);
    for failure in [
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

/// `MODEL`, beside an outcome of `Issue` without the key and a second command whose response has a
/// field of the same name and no outcome with the key.
const SIBLINGS: &str = "format: ess/22\nsystem: credentials\nversion: v1\ndomain: credentials.api\nerrors:\n  - name: credentials.api.Denied\ncommands:\n  - name: credentials.api.Issue\n    input: [{name: holder, type: String}]\n    response: [{name: secret, type: String}]\n    outcomes:\n      - {name: issued, returns: true, one_time_response: [secret]}\n      - {name: denied, when: holder.utf8_bytes == 0, error: credentials.api.Denied}\n  - name: credentials.api.Rotate\n    response: [{name: secret, type: String}]\n    outcomes: [{name: rotated, returns: true}]\n";

/// What the one-time clause of a behaviour's contract states, each phrase on its own.
const CLAUSE: [&str; 3] = ["atomic durable consumption", "fresh issuance", "no replay"];

/// The behaviour obligation of `command`, from the plan.
fn behaviour(plan: &SynthesisPlan, command: &str) -> ess_synth::ImplementationObligation {
    plan.obligation_of(ess_synth::CapabilityKind::CommandBehavior, command)
        .unwrap_or_else(|| panic!("the behaviour of `{command}` is no obligation"))
        .clone()
}

/// The declared outcome branches of a behaviour's contract, keyed by the outcome they open with.
fn branches(contract: &str) -> Vec<(String, String)> {
    let (_, declared) = contract
        .split_once("Declared outcomes (declaration order, not selection precedence): ")
        .expect("a behaviour contract lists its declared outcomes");
    declared
        .split("; ")
        .map(|branch| {
            let (name, _) = branch
                .strip_prefix('`')
                .and_then(|rest| rest.split_once('`'))
                .unwrap_or_else(|| panic!("a branch opens with its outcome: {branch}"));
            (name.to_owned(), branch.to_owned())
        })
        .collect()
}

/// The trait documentation and the `Unimplemented` stub body of `trait_name` in the generated Rust
/// source of `artifacts`.
fn stub<'a>(
    artifacts: impl IntoIterator<Item = &'a ess_gen::Artifact>,
    trait_name: &str,
) -> (String, String) {
    let opening = format!("pub trait {trait_name} {{");
    let implementation = format!("impl {trait_name} for Unimplemented {{");
    let source = artifacts
        .into_iter()
        .find(|artifact| artifact.contents.contains(&implementation))
        .unwrap_or_else(|| panic!("no generated stub `{implementation}`"));
    let contents = &source.contents;
    let before = &contents[..contents
        .find(&opening)
        .expect("the stub's trait is generated")];
    let doc = before[before.rfind("\n\n").map_or(0, |at| at + 2)..].to_owned();
    let start = contents.find(&implementation).unwrap();
    let body = &contents[start..];
    let body = body[..body.find("\n    }").expect("the stub ends") + 6].to_owned();
    (doc, body)
}

#[test]
fn rust_target_carries_a_one_time_response_outcome_as_an_obligation() {
    let model = ir(MODEL);
    let plan = SynthesisPlan::of(&model);
    let counts = plan.counts();
    assert_eq!(
        (counts.generated, counts.obligations, counts.refused),
        (1, 1, 0),
        "{}",
        plan.to_canonical_json()
    );
    assert!(plan.is_generated(
        ess_synth::CapabilityKind::CommandContract,
        "credentials.api.Issue"
    ));
    let owed = behaviour(&plan, "credentials.api.Issue");
    assert!(
        matches!(
            owed.reason,
            ess_synth::ObligationReason::Undetermined { .. }
        ),
        "{:?}",
        owed.reason
    );
    let contract = &owed.contract;
    for required in ["`issued`", "`secret`"].iter().chain(CLAUSE.iter()) {
        assert!(
            contract.contains(required),
            "{required} missing: {contract}"
        );
    }

    let synthesized = synthesize_for(&model, Target::Rust)
        .unwrap_or_else(|failure| panic!("{}", failure.to_canonical_json()));
    assert!(!synthesized
        .artifacts
        .values()
        .any(|artifact| artifact.contents.contains("cannot implement that policy")));
    let workspace = ess_synth::rust::workspace(&model, &plan)
        .unwrap_or_else(|failure| panic!("{}", failure.to_canonical_json()));
    let single = ess_synth::rust::single_crate(&model, &plan)
        .unwrap_or_else(|failure| panic!("{}", failure.to_canonical_json()));
    for (doc, body) in [
        stub(synthesized.artifacts.values(), "IssueBehavior"),
        stub(&workspace, "IssueBehavior"),
        stub(&single, "IssueBehavior"),
    ] {
        assert!(doc.contains(&format!("Contract: {contract}.")), "{doc}");
        assert!(
            body.contains("Err(crate::obligation::UnmetObligation {"),
            "{body}"
        );
        for forbidden in ["secret", "Ok(", "Issued"] {
            assert!(!body.contains(forbidden), "{forbidden} in the stub: {body}");
        }
    }
}

#[test]
fn the_one_time_clause_names_only_its_outcome() {
    let model = ir(SIBLINGS);
    let plan = SynthesisPlan::of(&model);
    let counts = plan.counts();
    assert_eq!(
        (counts.obligations, counts.refused),
        (2, 0),
        "{}",
        plan.to_canonical_json()
    );

    let issue = behaviour(&plan, "credentials.api.Issue").contract;
    let outcomes = branches(&issue);
    assert_eq!(
        outcomes
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["issued", "denied"],
        "{issue}"
    );
    for (name, branch) in &outcomes {
        for phrase in CLAUSE {
            assert_eq!(
                branch.contains(phrase),
                name == "issued",
                "{phrase} on `{name}`: {issue}"
            );
        }
    }
    let rotate = behaviour(&plan, "credentials.api.Rotate").contract;
    for phrase in CLAUSE {
        assert!(!rotate.contains(phrase), "{phrase}: {rotate}");
    }

    let synthesized = synthesize_for(&model, Target::Rust)
        .unwrap_or_else(|failure| panic!("{}", failure.to_canonical_json()));
    let (doc, _) = stub(synthesized.artifacts.values(), "IssueBehavior");
    assert!(doc.contains(&format!("Contract: {issue}.")), "{doc}");
    let (doc, body) = stub(synthesized.artifacts.values(), "RotateBehavior");
    assert!(doc.contains(&format!("Contract: {rotate}.")), "{doc}");
    for phrase in CLAUSE {
        assert!(
            !doc.contains(phrase) && !body.contains(phrase),
            "{phrase}: {doc}{body}"
        );
    }
}
