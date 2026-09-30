//! Adversary pass 1 on story:ui-spec-schema: the loader driven against the schema's own
//! shorthand, inheritance, naming and widget rules.

use ess_ui::{Body, Composite, Document, TabForm};

/// A one-page document; `widgets`, `page_kinds` and the page body are spliced in as flow YAML.
fn document(widgets: &str, page_kinds: &str, page: &str) -> String {
    format!(
        "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
types:
  Stage: {{enum: [lead, won]}}
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

// ── inheritance ──────────────────────────────────────────────────────────────────────────────

/// `names_unique` (error) holds for a page's own sections. Two page sections sharing the name of
/// an inherited section must be refused too, not merged silently with the second one dropped.
#[test]
fn two_page_sections_sharing_an_inherited_name_are_refused() {
    let result = load(
        "{}",
        "{}",
        "{kind: list_page, title: P, sections: [\
           {name: list, component: collection, reads: t.Page}, \
           {name: list, component: collection, reads: t.Other}]}",
    );
    match result {
        Err(error) => assert_eq!(error.path().to_string(), "pages/p/sections/list", "{error}"),
        Ok(document) => panic!(
            "accepted; sections kept: {:?}",
            document.pages["p"]
                .sections
                .iter()
                .map(|section| format!("{}: {:?}", section.name, section.body))
                .collect::<Vec<_>>()
        ),
    }
}

/// `shorthands.inheritance.named_lists` merges a page entry over the inherited one by name, and
/// `choice` `options[]` accepts bare strings. A page that overrides only the options of a choice
/// its kind declares writes no `component`, and the shorthand still applies.
#[test]
fn a_page_overrides_an_inherited_choices_options_with_the_shorthand() {
    let kinds = "{staged: {extends: list_page, sections: [{name: filters, choices: [\
                   {name: stage, component: choice, options: [a]}]}]}}";
    let document = loaded(
        "{}",
        kinds,
        "{kind: staged, title: P, sections: [\
           {name: filters, choices: [{name: stage, options: [x, y]}]}, \
           {name: list, component: collection, reads: t.Page}]}",
    );
    let Body::Composite(Composite::FilterBar(bar)) = section_body(&document, "filters") else {
        panic!("a filter bar")
    };
    let Body::Composite(Composite::Choice(choice)) = &bar.choices[0].body else {
        panic!("a choice")
    };
    let options: Vec<&str> = choice
        .options
        .iter()
        .map(|option| option.label.as_str())
        .collect();
    assert_eq!(options, ["x", "y"]);
}

// ── shorthands where the schema expects them ─────────────────────────────────────────────────

/// `Composite` accepts a bare kind name wherever a `Node` is expected; `form.record` is typed
/// `Node`.
#[test]
fn the_composite_shorthand_applies_to_a_forms_record() {
    let document = loaded(
        "{}",
        "{}",
        "{kind: form_page, title: P, sections: [{name: form, does: t.Save, record: record}]}",
    );
    let Body::Composite(Composite::Form(form)) = section_body(&document, "form") else {
        panic!("a form")
    };
    let record = form.record.as_ref().expect("the form has a record node");
    assert!(matches!(record.body, Body::Composite(Composite::Record(_))));
}

/// `Tab.form` is `one_of: [Node, Action]`; an action there derives its `name` like every other
/// action (`Action` at `name`: `first_present: [opens, …]`).
#[test]
fn an_action_in_a_tabs_form_derives_its_name() {
    let document = loaded(
        "{}",
        "{}",
        "{kind: detail_page, title: P, \
          overlays: {edit: {kind: drawer, component: form, does: t.Save}}, \
          sections: [{name: summary, reads: t.ById, tabs: [\
            {name: more, label: More, form: {opens: edit, label: Edit}}]}]}",
    );
    let Body::Composite(Composite::Record(record)) = section_body(&document, "summary") else {
        panic!("a record")
    };
    let Some(TabForm::Action(action)) = &record.tabs[0].form else {
        panic!("the tab holds an action: {:?}", record.tabs[0].form)
    };
    assert_eq!(action.name, "edit");
}

/// A widget param typed `Reads`, given a literal map (`WidgetInstance.args`: "an expression, or a
/// literal such as a map"), fills the body's `reads` with that map.
#[test]
fn a_reads_map_passed_as_a_widget_arg_becomes_the_bodys_read() {
    let widgets = "{feed: {summary: A feed., \
                     params: {source: {type: Reads, required: true, note: the rows}}, \
                     body: [{name: list, component: collection, reads: args.source}]}}";
    let document = loaded(
        widgets,
        "{}",
        "{kind: detail_page, title: P, sections: [\
           {name: summary, reads: t.ById}, \
           {name: feed, component: feed, args: {source: {view: t.Page, params: {q: state.q}}}}]}",
    );
    let Body::Widget(instance) = section_body(&document, "feed") else {
        panic!("a widget instance")
    };
    let Body::Composite(Composite::Collection(collection)) = &instance.body[0].body else {
        panic!("a collection")
    };
    let reads = collection.reads.as_ref().expect("the collection reads");
    assert_eq!(reads.view.as_deref(), Some("t.Page"));
    assert_eq!(reads.params["q"].0, "state.q");
}

// ── the unmapped marker ──────────────────────────────────────────────────────────────────────

/// `unmapped_marker.accepted_by: any field, whatever its declared type` — a retrofit marks an
/// enum it could not determine, and the validator reports it as a warning, never a refusal.
#[test]
fn an_unmapped_marker_is_accepted_by_an_enum_typed_field() {
    let result = load(
        "{}",
        "{}",
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
          load: 'UNMAPPED: the old page loads this on a timer nobody documented'}]}",
    );
    assert!(
        result.is_ok(),
        "refused: {}",
        result
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default()
    );
}

// ── node names ───────────────────────────────────────────────────────────────────────────────

fn segment_ok(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_.-".contains(character))
}

/// `NodePath.syntax.segment_pattern: ^[A-Za-z0-9_.-]+$`, separator `/`. A name holding `/`
/// forges a path of a different node.
#[test]
fn a_node_name_holding_the_path_separator_is_refused() {
    let result = load(
        "{}",
        "{}",
        "{kind: detail_page, title: P, sections: [\
           {name: summary, reads: t.ById}, \
           {name: 'summary/fields/tier', component: record, reads: t.ById}]}",
    );
    if let Ok(document) = result {
        let bad: Vec<String> = document
            .nodes()
            .iter()
            .map(|located| located.path.to_string())
            .filter(|path| path.contains("summary/fields/tier"))
            .collect();
        panic!("accepted; a section now sits at {bad:?}");
    }
}

/// A derived action name is a node name, so it matches the segment pattern or the action is
/// refused; `copy` holds an expression, which the pattern does not admit.
#[test]
fn a_derived_action_name_is_a_valid_path_segment() {
    let result = load(
        "{}",
        "{}",
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, actions: [\
           {copy: 'row.first + row.last', label: Copy name}]}]}",
    );
    if let Ok(document) = result {
        for located in document.nodes() {
            for segment in located.path.segments() {
                assert!(
                    segment_ok(segment),
                    "`{segment}` in `{}` is not a node name",
                    located.path
                );
            }
        }
    }
}

// ── the composite union and widgets ──────────────────────────────────────────────────────────

/// `Composite.union.other_tag_values: {ref: widget}`: a `component` names a widget only when it
/// names no member. A widget declared under a member's name can never be used; declaring it is
/// an error the loader has to say, not a definition it keeps and never reaches.
#[test]
fn a_widget_named_like_a_member_of_the_union_is_refused() {
    let widgets = "{metric: {summary: Shadowed., body: [{name: t, primitive: text, text: x}]}}";
    let result = load(
        widgets,
        "{}",
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
          item: [{name: m, component: metric, from: channel.c.x}]}]}",
    );
    assert!(
        result.is_err(),
        "a widget named `metric` was accepted and is unreachable"
    );
}

/// An optional param without a default that the instance omits is still a param of the widget;
/// whatever the expansion does with it, it does not call it unknown.
#[test]
fn an_omitted_optional_arg_is_not_reported_as_an_unknown_param() {
    let widgets = "{pill: {summary: A pill., \
                     params: {hidden: {type: boolean, note: hides the pill}}, \
                     body: [{name: t, primitive: text, text: x, visible: 'not args.hidden'}]}}";
    let result = load(
        widgets,
        "{}",
        "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
          item: [{name: pill, component: pill}]}]}",
    );
    if let Err(error) = result {
        assert!(
            !error.message().contains("names no param"),
            "`hidden` is a param of `pill`, yet: {error}"
        );
    }
}

// ── exactly_one_of on reads ──────────────────────────────────────────────────────────────────

/// The loader refuses a placeholder read without its `fixture` in a section; an overlay holds a
/// composite with the same `Reads`, and the same rule applies there.
#[test]
fn a_placeholder_read_without_a_fixture_in_an_overlay_is_refused() {
    let result = load(
        "{}",
        "{}",
        "{kind: detail_page, title: P, \
          overlays: {peek: {kind: drawer, component: record, reads: {placeholder: t.Draft}}}, \
          sections: [{name: summary, reads: t.ById}]}",
    );
    assert!(
        result.is_err(),
        "an overlay's placeholder read without a fixture was accepted"
    );
}
