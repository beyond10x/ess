//! Adversary pass 2 on story:ui-spec-schema: the pass order introduced by correction 1
//! (page kinds → `same_as` → widget instances → local shorthands → page context) and the
//! position table `ess_ui::positions()` driven against the schema.

use ess_ui::{Body, Composite, Document};

/// A one-page document; `widgets`, `page_kinds` and the page body are spliced in as flow YAML.
fn document(widgets: &str, page_kinds: &str, page: &str) -> String {
    format!(
        "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: p
  sections: [{{name: all, pages: [p]}}]
widgets: {widgets}
page_kinds: {page_kinds}
pages:
  p: {page}
"
    )
}

fn load(widgets: &str, page_kinds: &str, page: &str) -> Result<Document, ess_ui::LoadError> {
    ess_ui::load_str(&document(widgets, page_kinds, page))
}

fn loaded(widgets: &str, page_kinds: &str, page: &str) -> Document {
    load(widgets, page_kinds, page).unwrap_or_else(|error| panic!("refused: {error}"))
}

fn section_body<'a>(document: &'a Document, name: &str) -> &'a Body {
    &document.pages["p"]
        .sections
        .iter()
        .find(|section| section.name == name)
        .unwrap_or_else(|| panic!("no section `{name}`"))
        .body
}

/// A widget with one required string param.
const CARD: &str = "{card: {summary: c, params: {who: {type: string, required: true, note: n}}, \
                    body: [{name: t, primitive: text, text: args.who}]}}";

// ── the Composite shorthand now runs after widget expansion ──────────────────────────────────

/// `Composite` accepts `{ref: composite_kind}`; a bare string naming no member of the union is
/// not a composite kind. Before correction 1 a misspelt kind was refused by widget expansion
/// ("names neither a member of the composite union nor a widget"); the shorthand now runs after
/// that pass, so its output is never checked.
#[test]
fn a_misspelt_kind_in_the_composite_shorthand_is_refused() {
    let result = load(
        "{}",
        "{}",
        "{kind: detail_page, title: P, sections: [{name: summary, component: collection, \
           reads: t.Rows, expand: recrod}]}",
    );
    match result {
        Err(error) => assert_eq!(
            error.path().to_string(),
            "pages/p/sections/summary/expand",
            "{error}"
        ),
        Ok(document) => panic!(
            "accepted; expand is {:?}",
            match section_body(&document, "summary") {
                Body::Composite(Composite::Collection(collection)) => {
                    collection
                        .expand
                        .as_ref()
                        .map(|node| format!("{:?}", node.body))
                }
                other => Some(format!("{other:?}")),
            }
        ),
    }
}

/// The one map of nodes (board `widgets`) is where the bare-kind shorthand is most natural:
/// `{kpi: metric}`. A misspelt kind there must be refused as well.
#[test]
fn a_misspelt_kind_in_a_board_widget_is_refused() {
    let result = load(
        "{}",
        "{}",
        "{kind: dashboard_page, title: P, sections: [{name: board, reads: t.Board, \
           widgets: {kpi: metrc}}]}",
    );
    match result {
        Err(error) => assert_eq!(
            error.path().to_string(),
            "pages/p/sections/board/widgets/kpi",
            "{error}"
        ),
        Ok(_) => panic!("a board widget of kind `metrc` was accepted"),
    }
}

/// A widget written through the Composite shorthand either is refused (the shorthand accepts
/// composite kinds only) or is expanded like any widget use, which here must fail for the
/// missing required argument. It must not load as a widget use with no body.
#[test]
fn a_widget_named_by_the_composite_shorthand_is_not_left_unexpanded() {
    let result = load(
        CARD,
        "{}",
        "{kind: detail_page, title: P, sections: [{name: summary, component: collection, \
           reads: t.Rows, expand: card}]}",
    );
    if let Ok(document) = result {
        panic!(
            "accepted without the required argument `who`; expand is {:?}",
            match section_body(&document, "summary") {
                Body::Composite(Composite::Collection(collection)) => {
                    collection
                        .expand
                        .as_ref()
                        .map(|node| format!("{:?}", node.body))
                }
                other => Some(format!("{other:?}")),
            }
        );
    }
}

// ── export.reads is told apart by the key a map sits under ───────────────────────────────────

/// `export.reads` is the one `reads` that names a view rather than holding a `Reads`; the loader
/// tells it apart by the key the enclosing map sits under. An overlay *named* `export` sits
/// under that key too, and its own `reads` then misses the Reads shorthand.
#[test]
fn an_overlay_named_export_reads_through_the_shorthand() {
    let document = loaded(
        "{}",
        "{}",
        "{kind: detail_page, title: P, sections: [{name: summary, component: record, \
           reads: t.ById}], overlays: {export: {kind: dialog, component: record, reads: t.Csv}}}",
    );
    match &document.pages["p"].overlays["export"].body {
        Body::Composite(Composite::Record(record)) => assert_eq!(
            record
                .reads
                .as_ref()
                .and_then(|reads| reads.view.as_deref()),
            Some("t.Csv")
        ),
        other => panic!("not a record: {other:?}"),
    }
}

/// The same ambiguity in the one map of nodes: a board widget kind named `export`.
#[test]
fn a_board_widget_named_export_reads_through_the_shorthand() {
    let document = loaded(
        "{}",
        "{}",
        "{kind: dashboard_page, title: P, sections: [{name: board, reads: t.Board, \
           widgets: {export: {component: metric, reads: t.Exports}}}]}",
    );
    let Body::Composite(Composite::Board(board)) = section_body(&document, "board") else {
        panic!("not a board");
    };
    match &board.widgets["export"].body {
        Body::Composite(Composite::Metric(metric)) => assert_eq!(
            metric
                .reads
                .as_ref()
                .and_then(|reads| reads.view.as_deref()),
            Some("t.Exports")
        ),
        other => panic!("not a metric: {other:?}"),
    }
}

// ── widget definitions are typed before substitution ─────────────────────────────────────────

/// `WidgetInstance.args`: "one entry per param: an expression, or a literal such as a map".
/// A param that carries a list of fields into a list-typed position is substituted at the use
/// site, where the result is a valid record. The declaration itself, with `args.cols` still in
/// place, must not be what refuses the document.
#[test]
fn a_list_arg_fills_a_list_typed_position_of_a_widget_body() {
    let widgets = "{panel: {summary: p, params: {src: {type: string, required: true, note: n}, \
                     cols: {type: {list: string}, required: true, note: n}}, \
                   body: [{name: rec, component: record, reads: args.src, fields: args.cols}]}}";
    let document = loaded(
        widgets,
        "{}",
        "{kind: detail_page, title: P, sections: [{name: summary, component: record, \
           reads: t.ById, item: [{name: pn, component: panel, args: {src: t.X, cols: [a, b]}}]}]}",
    );
    let paths: Vec<String> = document
        .nodes()
        .iter()
        .map(|located| located.path.to_string())
        .collect();
    assert!(
        paths.contains(&"pages/p/sections/summary/item/pn/body/rec/fields/b".to_owned()),
        "{paths:?}"
    );
}

// ── the unmapped marker ──────────────────────────────────────────────────────────────────────

/// Decision (correction 2, F5): `unmapped_marker.accepted_by` lists string, enum and reference
/// fields only. A boolean field refuses the marker, naming its path and saying why.
#[test]
fn an_unmapped_marker_is_refused_by_a_boolean_field() {
    let error = load(
        "{}",
        "{}",
        "{kind: detail_page, title: P, sections: [{name: summary, component: collection, \
           reads: t.Rows, columns: [{field: x, sortable: 'UNMAPPED: the grid config is dynamic'}]}]}",
    )
    .map(|_| ())
    .expect_err("a boolean field refuses the marker");
    assert_eq!(
        error.path().to_string(),
        "pages/p/sections/summary/columns/x",
        "{error}"
    );
    assert!(
        error
            .message()
            .contains("only in a string, enum or reference field"),
        "{error}"
    );
}

/// The same for a record-typed field (`collection.sort`).
#[test]
fn an_unmapped_marker_is_refused_by_a_record_field() {
    let error = load(
        "{}",
        "{}",
        "{kind: detail_page, title: P, sections: [{name: summary, component: collection, \
           reads: t.Rows, sort: 'UNMAPPED: sorting is done by a plugin'}]}",
    )
    .map(|_| ())
    .expect_err("a record field refuses the marker");
    assert_eq!(
        error.path().to_string(),
        "pages/p/sections/summary",
        "{error}"
    );
    assert!(
        error
            .message()
            .contains("only in a string, enum or reference field"),
        "{error}"
    );
}

/// `unmapped_marker.renderer: treat_as_absent`. At a `Reads` position the shorthand wraps the
/// marker as `{view: "UNMAPPED: …"}`, so the document now names a view called `UNMAPPED: …`
/// and a renderer reads it.
#[test]
fn an_unmapped_marker_at_reads_is_not_read_as_a_view_name() {
    if let Ok(document) = load(
        "{}",
        "{}",
        "{kind: detail_page, title: P, sections: [{name: summary, component: record, \
           reads: 'UNMAPPED: the handler builds the query at runtime'}]}",
    ) {
        if let Body::Composite(Composite::Record(record)) = section_body(&document, "summary") {
            let view = record.reads.as_ref().and_then(|reads| reads.view.clone());
            assert!(
                !view.as_deref().unwrap_or("").starts_with("UNMAPPED: "),
                "the marker became view {view:?}"
            );
        }
    }
}
