//! The order a generated server tries its routes in, and the order the `OpenAPI` document lists
//! them in (`story:a-literal-path-segment-takes-precedence-over-a-variable-one`, beyond10x/ess#748
//! part 2; `docs/design/http-presentation-binding.md`, "Path templates").
//!
//! At the first position two templates of one method differ, a literal takes precedence over a
//! variable, so a server tries the literal one first. That order is computed, never read off the
//! template text: `~` is a path character that sorts after `{`. The `OpenAPI` document keeps its
//! path order.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::http::{derived, dispatch_order, Binding};
use ess_gen::openapi::BoundOpenApi;

const BOARD: &str = include_str!("../../../specify/ess-http/tests/fixtures/board.yaml");

#[test]
fn dispatch_order_puts_a_literal_before_a_variable() {
    // In path order the literal already comes first: kept.
    assert_eq!(
        dispatch_order(&[
            ("PUT", "/boards/{board_id}/cards/priority"),
            ("PUT", "/boards/{board_id}/cards/{card_id}"),
        ]),
        [0, 1]
    );
    // `~` sorts after `{`, so path order puts the variable first; the literal is tried first.
    assert_eq!(
        dispatch_order(&[
            ("PUT", "/boards/{board_id}/cards/{card_id}"),
            ("PUT", "/boards/{board_id}/cards/~priority"),
        ]),
        [1, 0]
    );
    // The first position the two differ decides, whatever comes after it.
    assert_eq!(
        dispatch_order(&[
            ("GET", "/{a}/{b}"),
            ("GET", "/{a}/~x"),
            ("GET", "/~y/{b}"),
            ("GET", "/~y/~x"),
        ]),
        [3, 2, 1, 0]
    );
    // Templates no request path matches both of keep their order: another method, another
    // length, two different literals at one position.
    for rows in [
        [("GET", "/a/{x}"), ("PUT", "/a/~b")],
        [("GET", "/a/{x}"), ("GET", "/a/~b/c")],
        [("GET", "/a/{x}/c"), ("GET", "/a/~b/d")],
        [("GET", "/docs"), ("GET", "/openapi.json")],
    ] {
        assert_eq!(dispatch_order(&rows), [0, 1], "{rows:?}");
    }
    // A table of exact paths, as every surface without a binding serves, keeps its order.
    let exact = [
        ("GET", "/docs"),
        ("GET", "/openapi.json"),
        ("POST", "/board/commands/rename-board"),
        ("POST", "/board/commands/set-card-priorities"),
        ("GET", "/board/views/board-by-id"),
    ];
    assert_eq!(dispatch_order(&exact), [0, 1, 2, 3, 4]);
    // An exact path is a literal too: `/docs` before a template that matches it.
    assert_eq!(
        dispatch_order(&[("GET", "/{page}"), ("GET", "/~docs")]),
        [1, 0]
    );
}

fn board() -> EssIr {
    let raw = RawSpecFile::parse(BOARD).expect("well formed");
    let spec = Specification::assemble([(Source::new("board.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("board.yaml", BOARD);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The board service's `OpenAPI` document under the binding `routes` declares.
fn document(ir: &EssIr, routes: &str) -> String {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n{routes}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    let compiled = ess_http::compile(&spec, ir, &derived(ir))
        .unwrap_or_else(|diagnostics| panic!("compiles: {diagnostics}"));
    let binding = Binding::new(compiled).expect("every route has a path");
    let artifacts = run(&BoundOpenApi(binding), ir).expect("generates");
    artifacts["openapi/board-service.yaml"].contents.clone()
}

/// Where each path key starts in `document`, in the order `paths` lists them.
fn listed<'a>(document: &str, paths: &[&'a str]) -> Vec<&'a str> {
    let mut found: Vec<(usize, &str)> = paths
        .iter()
        .map(|path| {
            let at = document
                .find(&format!("\n  {path}:\n"))
                .or_else(|| document.find(&format!("\n  '{path}':\n")))
                .unwrap_or_else(|| panic!("`{path}` is not a path of the document:\n{document}"));
            (at, *path)
        })
        .collect();
    found.sort_unstable();
    found.into_iter().map(|(_, path)| path).collect()
}

#[test]
fn openapi_path_order_is_unchanged_for_existing_bindings() {
    let ir = board();
    let card = "  - command: catalog.board.UpdateCard
    method: put
    path: /boards/{board_id}/cards/{card_id}
    parameters: {board_id: input.board_id, card_id: input.card_id}
";
    let priorities = "  - command: catalog.board.SetCardPriorities
    method: put
    path: /boards/{board_id}/cards/~priority
    parameters: {board_id: input.board_id}
    request: {body: input.priorities}
";
    // A binding whose templates do not overlap lists its paths in path order, as before.
    let apart = document(
        &ir,
        &format!(
            "routes:\n{card}{}",
            priorities.replace("/cards/~priority", "/priority")
        ),
    );
    assert_eq!(
        listed(
            &apart,
            &[
                "/boards/{board_id}/cards/{card_id}",
                "/boards/{board_id}/priority"
            ]
        ),
        [
            "/boards/{board_id}/cards/{card_id}",
            "/boards/{board_id}/priority"
        ]
    );
    // Two that overlap are still listed in path order, the variable first: precedence is the
    // servers' dispatch order, and `OpenAPI` states the same rule for its own readers.
    let overlapping = document(&ir, &format!("routes:\n{card}{priorities}"));
    assert_eq!(
        listed(
            &overlapping,
            &[
                "/boards/{board_id}/cards/~priority",
                "/boards/{board_id}/cards/{card_id}"
            ]
        ),
        [
            "/boards/{board_id}/cards/{card_id}",
            "/boards/{board_id}/cards/~priority"
        ]
    );
    // Declaring both routes in the other order writes the same bytes.
    assert_eq!(
        document(&ir, &format!("routes:\n{priorities}{card}")),
        overlapping
    );
}
