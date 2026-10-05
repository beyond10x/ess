//! The served contract of a read-granted view (beyond10x/ess#286): the actors an actor's `may:`
//! grants the view are named on its operation (`x-ess-may-read`), and the operation declares the
//! standard refusal, `403` `{refused: "not granted", actor}`, the same body a command answers an
//! ungranted actor. A view no actor names stays open and declares neither, and a model that names
//! no view keeps its contract.
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};

const MODEL: &str = include_str!("../../../../docs/design/view-grants.example.yaml");

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("view-grants.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn contract(ir: &EssIr) -> Value {
    let component = ir.components().values().next().expect("one component");
    serde_json::from_str(&ess_gen::openapi::json(ir, component)).expect("the contract is JSON")
}

fn read<'a>(contract: &'a Value, view: &str) -> &'a Value {
    let found = contract["paths"]
        .as_object()
        .expect("paths")
        .values()
        .find(|item| item["get"]["operationId"] == view)
        .unwrap_or_else(|| panic!("`{view}` is served: {contract}"));
    &found["get"]
}

#[test]
fn a_read_granted_view_names_its_readers_and_declares_the_standard_refusal() {
    let contract = contract(&compiled(MODEL));
    let board = read(&contract, "desk.tickets.Board");
    assert_eq!(
        board["x-ess-may-read"],
        json!(["desk.tickets.Clerk"]),
        "{board}"
    );
    let refusal = &board["responses"]["403"]["content"]["application/json"]["schema"];
    assert_eq!(refusal["required"], json!(["refused", "actor"]), "{board}");
    assert_eq!(
        refusal["properties"]["refused"]["enum"],
        json!(["not granted"]),
        "{board}"
    );
    let titles = read(&contract, "desk.tickets.Titles");
    assert!(titles.get("x-ess-may-read").is_none(), "{titles}");
    assert!(titles["responses"].get("403").is_none(), "{titles}");
    let description = contract["info"]["description"].as_str().expect("described");
    assert!(
        description.contains("A view an actor's grant names is read-checked"),
        "{description}"
    );
    assert!(
        !description.contains("Views are not grant-checked"),
        "{description}"
    );
}

#[test]
fn a_model_that_names_no_view_keeps_its_contract() {
    let open = compiled(&MODEL.replace("      - desk.tickets.Board\n", ""));
    let contract = contract(&open);
    for view in ["desk.tickets.Board", "desk.tickets.Titles"] {
        let operation = read(&contract, view);
        assert!(operation.get("x-ess-may-read").is_none(), "{operation}");
        assert!(operation["responses"].get("403").is_none(), "{operation}");
    }
    let description = contract["info"]["description"].as_str().expect("described");
    assert!(
        description.contains("Views are not grant-checked"),
        "{description}"
    );
}
