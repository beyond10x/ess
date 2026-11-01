//! Adversary, wave D350 unit U1, pass 1: a `<command>.response` root exists exactly when the
//! command declares a response, as `--all-commands` lists it, with and without a binding
//! (story:a-response-root-requires-a-declared-response).

use std::collections::BTreeSet;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::http::{derived, Binding};
use ess_gen::schema::ModelTypes;

const SOURCE: &str = include_str!("fixtures/command-view-roots.yaml");

/// Both commands routed; `demo.notes.Pin` carries its only input field in the path.
const ROUTES: &str = "routes:
  - command: demo.notes.Create
    method: put
    path: /notes/{note_id}
    parameters: {note_id: input.note_id}
  - command: demo.notes.Pin
    method: post
    path: /notes/{note_id}/pin
    parameters: {note_id: input.note_id}
";

fn compiled() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, SOURCE.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(SOURCE).unwrap())])
            .unwrap();
    compile(&specification, &sources).unwrap()
}

fn bound(ir: &EssIr) -> Binding {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: demo\n  version: v1\n  source_digest: sha256:{}\n{ROUTES}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    let http = ess_http::compile(&spec, ir, &derived(ir))
        .unwrap_or_else(|diagnostics| panic!("compiles: {diagnostics}"));
    Binding::new(http).expect("every route has a path")
}

/// For every command, unbound and bound: naming `<command>.response` selects exactly when
/// `bound_command_roots` lists it, and is otherwise one `unknown_root` refusal for that name.
#[test]
fn every_response_root_is_selectable_exactly_when_all_commands_lists_it() {
    let ir = compiled();
    let binding = bound(&ir);
    let mut refused = 0;
    for binding in [None, Some(&binding)] {
        let listed = ModelTypes::bound_command_roots(&ir, binding);
        for command in ir.commands().values() {
            let root = format!("{}.response", command.name);
            let result = ModelTypes::select_bound(&ir, &BTreeSet::from([root.clone()]), binding);
            if listed.contains(&root) {
                let selection = result.unwrap_or_else(|errors| panic!("{root}: {errors:?}"));
                assert!(selection.definitions().contains_key(&root), "{root}");
            } else {
                refused += 1;
                let errors = result.err().unwrap_or_else(|| {
                    panic!("{root} is not listed by --all-commands and selects")
                });
                assert_eq!(errors.len(), 1, "{root}: {errors:?}");
                assert_eq!(errors[0].name, root);
                assert_eq!(errors[0].rule, "unknown_root", "{root}: {errors:?}");
            }
        }
    }
    assert_eq!(refused, 2, "`demo.notes.Pin.response`, unbound and bound");
}

/// A response-less response root beside roots that do select refuses the whole selection, naming
/// only itself: no partial library is produced.
#[test]
fn a_responseless_response_root_beside_valid_roots_refuses_the_selection() {
    let ir = compiled();
    let roots = BTreeSet::from([
        "demo.notes.Create.input".to_owned(),
        "demo.notes.Create.response".to_owned(),
        "demo.notes.Pin.input".to_owned(),
        "demo.notes.Pin.response".to_owned(),
    ]);
    let errors = ModelTypes::select(&ir, &roots).expect_err("one root names no body");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].name, "demo.notes.Pin.response");
    assert_eq!(errors[0].rule, "unknown_root");
}

/// `--all-commands`' roots always select together, unbound and bound.
#[test]
fn all_commands_roots_select_together() {
    let ir = compiled();
    let binding = bound(&ir);
    for binding in [None, Some(&binding)] {
        let listed = ModelTypes::bound_command_roots(&ir, binding);
        assert!(!listed.contains("demo.notes.Pin.response"));
        let selection = ModelTypes::select_bound(&ir, &listed, binding)
            .unwrap_or_else(|errors| panic!("{errors:?}"));
        assert_eq!(selection.roots(), &listed);
    }
}
