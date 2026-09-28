//! `paging:` (ess/16, beyond10x/ess#174) lands on the resolved view, so every consumer reads which
//! declared parameters carry the page and its size rather than re-deriving it; a view without it
//! serializes exactly as before.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("../../../verify/ess-conformance/tests/fixtures/view-paging.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("view-paging.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

#[test]
fn the_resolved_view_carries_its_paging() {
    let ir = ir(MODEL);
    let view = ir
        .views()
        .get(&"demo.jobs.JobList".parse().unwrap())
        .expect("JobList");
    let paging = view.paging.as_ref().expect("paged");
    assert_eq!(
        (paging.page.as_str(), paging.size.as_str()),
        ("page", "size")
    );
    assert!(paging.total);
    assert!(view.params.iter().any(|param| param.name == paging.page));
    let json = serde_json::to_value(view).unwrap();
    assert_eq!(
        json["paging"],
        serde_json::json!({"page": "page", "size": "size", "total": true})
    );
}

#[test]
fn a_view_without_paging_has_no_paging_key() {
    let text = MODEL
        .replace("    paging: {page: page, size: size, total: true}\n", "")
        .replace(
            "      - {name: page, type: Integer}\n      - {name: size, type: Integer}\n",
            "",
        );
    let ir = ir(&text);
    let view = ir.views().values().next().expect("a view");
    assert!(view.paging.is_none());
    let json = serde_json::to_value(view).unwrap();
    assert!(json.get("paging").is_none(), "{json}");
}
