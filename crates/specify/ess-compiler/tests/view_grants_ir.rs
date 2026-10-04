//! A view an actor's `may:` names is resolved into the IR as that actor's read grant
//! (beyond10x/ess#286), and a model that names none keeps its IR bytes.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!("../../../../docs/design/view-grants.example.yaml");

fn ir(text: &str) -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("view-grants.yaml", text);
    let spec = Specification::assemble([(
        Source::new("view-grants.yaml"),
        RawSpecFile::parse(text).expect("parses"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn name(value: &str) -> ess_domain::QualifiedName {
    value.parse().expect("a qualified name")
}

#[test]
fn a_view_grant_resolves_to_the_actors_read_grants_and_not_its_command_grants() {
    let ir = ir(MODEL);
    let clerk = &ir.actors()[&name("desk.tickets.Clerk")];
    let read: Vec<String> = clerk
        .may_read
        .iter()
        .map(|view| view.name().to_string())
        .collect();
    assert_eq!(read, ["desk.tickets.Board"]);
    let invoked: Vec<String> = clerk
        .may
        .iter()
        .map(|command| command.name().to_string())
        .collect();
    assert_eq!(invoked, ["desk.tickets.OpenTicket"]);
    let watcher = &ir.actors()[&name("desk.tickets.Watcher")];
    assert_eq!(watcher.may_read.len(), 0, "{:?}", watcher.may_read);
}

#[test]
fn the_ir_answers_which_views_are_read_granted_and_to_whom() {
    let ir = ir(MODEL);
    let board = ir
        .views()
        .keys()
        .find(|view| view.to_string() == "desk.tickets.Board")
        .expect("declared");
    let titles = ir
        .views()
        .keys()
        .find(|view| view.to_string() == "desk.tickets.Titles")
        .expect("declared");
    assert!(ir.read_granted(board), "an actor names the Board");
    assert!(
        !ir.read_granted(titles),
        "no actor names Titles: it is open"
    );
    let readers: Vec<String> = ir
        .readers(board)
        .map(|actor| actor.name.to_string())
        .collect();
    assert_eq!(readers, ["desk.tickets.Clerk"]);
}

#[test]
fn the_canonical_ir_carries_the_read_grant_and_a_model_without_one_keeps_its_bytes() {
    let granted = ir(MODEL).to_canonical_json();
    assert!(granted.contains("\"may_read\""), "{granted}");
    let open = ir(&MODEL.replace("      - desk.tickets.Board\n", "")).to_canonical_json();
    assert!(!open.contains("may_read"), "{open}");
}
