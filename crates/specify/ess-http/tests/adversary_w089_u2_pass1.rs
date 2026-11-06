//! Adversary pass on unit U2 of the 0.89.0 wave: a bare request body
//! (`story:a-command-route-takes-one-input-as-its-bare-request-body`) under a refined binding, and
//! the refusals of `request.body` a refinement keeps.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{
    compile_refining, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation, Refined,
};

const MODEL: &str = include_str!("fixtures/board.yaml");

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("board.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("board.yaml", MODEL);
    compile_model(&spec, &sources).expect("compiles")
}

/// The routes ESS derives for the board, as `request_body.rs` writes them out.
fn derived(ir: &EssIr) -> Vec<DerivedRoute> {
    let mut table = Vec::new();
    for component in ir.components().values() {
        for handle in &component.accepts {
            table.push(DerivedRoute {
                component: component.name.to_string(),
                operation: Operation::Command(handle.name().to_string()),
                method: Method::Post,
                path: format!("/board/commands/{}", handle.name().local()),
            });
        }
        for domain in &component.owns {
            for handle in &ir.domain(domain).views {
                table.push(DerivedRoute {
                    component: component.name.to_string(),
                    operation: Operation::View(handle.name().to_string()),
                    method: Method::Get,
                    path: format!("/board/views/{}", handle.name().local()),
                });
            }
        }
    }
    table.sort_by(|left, right| {
        (&left.component, &left.path, left.method).cmp(&(
            &right.component,
            &right.path,
            right.method,
        ))
    });
    table
}

fn header(ir: &EssIr) -> String {
    format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n",
        ir.source_digest()
    )
}

fn refining(ir: &EssIr, protocol: &str, server: &str) -> Result<HttpIr, HttpDiagnostics> {
    let refined = Refined::from_yaml((header(ir) + protocol).as_bytes()).expect("parses");
    let text = header(ir) + &format!("refines: {}\n", refined.digest()) + server;
    let spec = HttpSpec::from_yaml(&text).expect("parses");
    compile_refining(&spec, Some(&refined), ir, &derived(ir))
}

fn located(result: &Result<HttpIr, HttpDiagnostics>) -> Vec<(&'static str, String)> {
    match result {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .0
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.path.clone()))
            .collect(),
    }
}

fn shown(result: &Result<HttpIr, HttpDiagnostics>) -> Vec<String> {
    match result {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics.0.iter().map(ToString::to_string).collect(),
    }
}

/// A protocol binding fixes the method and the request of a route and leaves its path open for the
/// server binding that refines it. A refinement restates the request it refines (`ESS-HTTP-012`),
/// so the protocol binding is the only place a bare body can be written. Its open route has no
/// path yet, so the board cannot be carried there, and `bare_body` refuses the body for it: the
/// adopter's reproduction cannot be bound under a refinement at all, though `encoding: form` can.
#[test]
fn adv_a_bare_body_can_be_bound_under_a_refinement() {
    let ir = model();
    let protocol = "routes:
  - command: catalog.board.SetCardPriorities
    method: put
    request: {encoding: json, body: input.priorities}
";
    let server = "routes:
  - command: catalog.board.SetCardPriorities
    method: put
    path: /boards/{board_id}/cards/priority
    parameters: {board_id: input.board_id}
    request: {encoding: json, body: input.priorities}
";
    let result = refining(&ir, protocol, server);
    assert_eq!(
        located(&result),
        [],
        "defect: a bare body cannot be bound under a refinement: the protocol binding's open \
         route is refused for `board_id`, which only the refinement's path will carry, and the \
         refinement may not add a body the protocol binding left out (ESS-HTTP-012): {:?}",
        shown(&result)
    );
}

/// The control: a refinement that adds a body the protocol binding left out is `ESS-HTTP-012`
/// at `routes[0].request.body`, as an added `encoding` is at `routes[0].request.encoding`.
#[test]
fn adv_a_refinement_that_adds_a_bare_body_is_ess_http_012() {
    let ir = model();
    let protocol = "routes:
  - command: catalog.board.SetCardPriorities
    method: put
";
    let server = "routes:
  - command: catalog.board.SetCardPriorities
    method: put
    path: /boards/{board_id}/cards/priority
    parameters: {board_id: input.board_id}
    request: {encoding: json, body: input.priorities}
";
    let result = refining(&ir, protocol, server);
    assert_eq!(
        located(&result),
        [("ESS-HTTP-012", "routes[0].request.body".to_owned())],
        "{:?}",
        shown(&result)
    );
}

/// A refinement of a fixed route that drops its bare body is `ESS-HTTP-012`.
#[test]
fn adv_a_refinement_that_drops_a_fixed_bare_body_is_ess_http_012() {
    let ir = model();
    let protocol = "routes:
  - command: catalog.board.SetCardPriorities
    method: put
    path: /boards/{board_id}/cards/priority
    parameters: {board_id: input.board_id}
    request: {body: input.priorities}
";
    let server = "routes:
  - command: catalog.board.SetCardPriorities
    method: put
    path: /boards/{board_id}/cards/priority
    parameters: {board_id: input.board_id}
";
    let result = refining(&ir, protocol, server);
    assert_eq!(
        located(&result),
        [("ESS-HTTP-012", "routes[0].request.body".to_owned())],
        "{:?}",
        shown(&result)
    );
}
