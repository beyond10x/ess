//! Adversary cases for how selection by existence (ess/16, beyond10x/ess#164) is published:
//! `docs/design/outcome-shapes.md`, "Projections".

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/upsert-by-existence.yaml");

const COMPONENT: &str = "components:\n  - component: item-service\n    owns:\n      domains: [demo.items]\n    accepts:\n      commands: [demo.items.PutItem, demo.items.BookSlot]\n    publishes:\n      events: [demo.items.ItemStored, demo.items.SlotBooked]\n";

fn ir() -> EssIr {
    ir_of(MODEL)
}

/// The fixture with an input-guarded refusal on `PutItem`, beside its create-or-update pair.
fn with_rejection() -> EssIr {
    let text = MODEL
        .replace(
            "errors:\n",
            "errors:\n  - name: demo.items.BadLabel\n    summary: The label is not accepted.\n    fields: []\n",
        )
        .replacen(
            "      - {name: label, type: demo.items.Label}\n    outcomes:\n",
            "      - {name: label, type: demo.items.Label}\n    outcomes:\n      - {name: rejected, when: label == \"bad\", error: demo.items.BadLabel}\n",
            1,
        );
    assert!(text.contains("name: rejected"), "fixture rewritten");
    ir_of(&text)
}

fn ir_of(model: &str) -> EssIr {
    let raw = RawSpecFile::parse(&format!("{model}{COMPONENT}")).unwrap();
    let spec = Specification::assemble([(Source::new("items.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn artifacts(ir: &EssIr, keep: impl Fn(&str) -> bool) -> String {
    ess_gen::generate_all(ir)
        .unwrap()
        .into_iter()
        .filter(|(path, _)| keep(path))
        .map(|(path, artifact)| format!("== {path}\n{}", artifact.contents))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The design note: "the served surface answers it with `409`", and the creation beside it is an
/// ordinary acceptance.
#[test]
fn adversary_the_openapi_contract_answers_a_duplicate_with_conflict() {
    let ir = ir();
    let book = &ir.commands()[&"demo.items.BookSlot".parse().unwrap()];
    let taken = book
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "already-booked")
        .unwrap();
    assert_eq!(ess_gen::http::status(taken), "409");
    let put = &ir.commands()[&"demo.items.PutItem".parse().unwrap()];
    for outcome in &put.outcomes {
        assert_eq!(
            ess_gen::http::status(outcome),
            ess_gen::http::status(&book.outcomes[0]),
            "`{}` is an acceptance like `booked`",
            outcome.name
        );
    }
    let openapi = artifacts(&ir, |path| path.contains("openapi"));
    assert!(
        openapi.starts_with("== "),
        "an OpenAPI artifact is generated"
    );
    assert!(
        openapi.contains("'409'") || openapi.contains("\"409\""),
        "{openapi}"
    );
    assert!(
        openapi
            .contains("Taken when a record already carries the identity the request would create"),
        "{openapi}"
    );
}

/// The generated page must not tell a reader the creating half of create-or-update pre-empts every
/// other answer when the conformance suite requires an input-guarded refusal to answer an identity
/// no record carries (see `ess-conformance/tests/adversary_upsert_witness.rs`). The unit hands the
/// creating branch the ess/15 sentence unchanged: "before any other answer for it".
#[test]
fn adversary_the_page_does_not_promise_the_creation_answers_before_any_other_branch() {
    let docs = artifacts(&with_rejection(), |path| {
        std::path::Path::new(path)
            .extension()
            .is_some_and(|extension| extension == "md")
    });
    assert!(
        docs.contains("BadLabel"),
        "the refusal is on the page: {docs}"
    );
    assert!(
        !docs.contains(
            "Taken when the identity the command names is one no record carries, before any other answer for it."
        ),
        "the creating branch is documented as answered before any other branch:\n{docs}"
    );
}
