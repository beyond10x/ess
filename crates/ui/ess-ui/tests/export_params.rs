//! An export's `params` is written out as a map `{<param>: <expr>}` or as one expression
//! (`same_as(<section>)`), as before (story:a-refused-node-is-reported-at-its-own-path,
//! beyond10x/ess#723).

use std::collections::BTreeMap;

use ess_ui::{load_str, Body, Composite, Document, Export, ExportParams, Expr};

fn document(export: &str) -> String {
    format!(
        "format: ess-ui/1
app: desk
model: desk.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: files
  sections: [{{name: all, pages: [files]}}]
pages:
  files:
    kind: list_page
    title: Files
    sections:
      - name: list
        component: collection
        selection: multiple
        reads: {{view: desk.Files}}
        columns: [name]
        bulk_actions:
          - {{name: download, label: Download, export: {export}}}
"
    )
}

fn export(document: &Document) -> &Export {
    let section = document.pages["files"]
        .sections
        .iter()
        .find(|section| section.name == "list")
        .expect("the page has the section `list`");
    let Body::Composite(Composite::Collection(collection)) = &section.body else {
        panic!("the section is a collection: {:?}", section.body);
    };
    collection.bulk_actions[0]
        .export
        .as_ref()
        .expect("the bulk action exports")
}

#[test]
fn an_export_takes_params_written_out() {
    let text = document("{reads: desk.Bundle, as: zip, params: {ids: selection.id}}");
    let document = load_str(&text).unwrap_or_else(|error| panic!("the document loads: {error}"));
    let export = export(&document);
    assert_eq!(export.reads, "desk.Bundle");
    assert_eq!(
        export.params,
        Some(ExportParams::Map(BTreeMap::from([(
            "ids".to_owned(),
            Expr("selection.id".to_owned())
        )])))
    );
}

#[test]
fn an_export_keeps_the_expression_form() {
    let text = document("{reads: desk.Bundle, as: csv, params: same_as(list)}");
    let document = load_str(&text).unwrap_or_else(|error| panic!("the document loads: {error}"));
    assert_eq!(
        export(&document).params,
        Some(ExportParams::Expression(Expr("same_as(list)".to_owned())))
    );
}

#[test]
fn a_wrongly_shaped_params_names_the_two_shapes() {
    for (params, found) in [
        ("[ids, names]", "not a list"),
        (
            "{ids: {from: selection.id}}",
            "param `ids` is a map, not an expression",
        ),
    ] {
        let text = document(&format!(
            "{{reads: desk.Bundle, as: zip, params: {params}}}"
        ));
        let Err(error) = load_str(&text) else {
            panic!("`params: {params}` loads");
        };
        let message = error.message();
        assert!(
            message.contains("a map of params `{<param>: <expr>}`")
                && message.contains("one expression such as `same_as(<section>)`")
                && message.contains(found)
                && !message.contains("untagged"),
            "story:a-refused-node-is-reported-at-its-own-path: `params: {params}` is refused \
             naming the two shapes and `{found}`: {error}"
        );
        assert!(
            error
                .path()
                .to_string()
                .starts_with("pages/files/sections/list/bulk_actions/download"),
            "the refusal is at the bulk action: {error}"
        );
    }
}
