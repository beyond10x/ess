//! Adversary pass 1 on story:ui-spec-schema: rules the schema states as refusals that the loader
//! accepts. They were routed to story:ui-spec-checks, so "refused" here means the checker reports
//! at least one error-severity finding for the document.

use std::path::Path;

fn document(widgets: &str, page: &str) -> String {
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
pages:
  p: {page}
"
    )
}

/// The control every case below depends on: without its one violation, the document passes, so
/// each refusal is that violation's and not the scaffold's.
#[test]
fn the_document_without_a_violation_is_accepted() {
    let page = "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}]}";
    assert!(accepted("{}", page));
    let widgets = "{leaf: {summary: A leaf., body: [{name: caption, primitive: text, text: T}]}}";
    assert!(accepted(widgets, page));
}

fn accepted(widgets: &str, page: &str) -> bool {
    !ess_ui_check::check_source(
        &document(widgets, page),
        "adversary.yaml",
        Path::new(env!("CARGO_MANIFEST_DIR")),
        None,
        &ess_ui_check::Options::default(),
    )
    .has_errors()
}

/// `PlacementProfile.resolution.refusals`: `{state.sensitive: true, store: [url, session_storage,
/// local_storage]} -> refuse`.
#[test]
fn sensitive_state_in_the_url_or_browser_storage_is_refused() {
    for store in ["url", "session_storage", "local_storage"] {
        let page = format!(
            "{{kind: detail_page, title: P, \
              state: {{token: {{type: string, class: page_state, store: {store}, sensitive: true}}}}, \
              sections: [{{name: summary, reads: t.ById}}]}}"
        );
        assert!(
            !accepted("{}", &page),
            "sensitive state in `{store}` was accepted"
        );
    }
}

/// `refusals: {state.class: credential, store_not_in: [memory, server_session]} -> refuse`.
#[test]
fn a_credential_in_local_storage_is_refused() {
    let page = "{kind: detail_page, title: P, \
                state: {token: {type: string, class: credential, store: local_storage}}, \
                sections: [{name: summary, reads: t.ById}]}";
    assert!(
        !accepted("{}", page),
        "a credential in local_storage was accepted"
    );
}

/// `type_rule`: a type is a lowercase primitive, a constructor map or a capitalized name;
/// `checks.types_structural` forbids `string_holding_a_type_expression`.
#[test]
fn a_type_written_as_a_string_expression_is_refused() {
    for ty in ["'list<PartnerId>'", "'{list: string}'", "'optional string'"] {
        let page = format!(
            "{{kind: detail_page, title: P, \
              state: {{picked: {{type: {ty}, class: selection}}}}, \
              sections: [{{name: summary, reads: t.ById}}]}}"
        );
        assert!(!accepted("{}", &page), "`type: {ty}` was accepted");
    }
}

/// `layers.widget`: "may not contain itself, directly or indirectly" — a property of the
/// declaration, whether or not a page uses it yet.
#[test]
fn a_widget_that_contains_itself_is_refused_even_before_it_is_used() {
    let widgets = "{loop: {summary: Recurses., body: [{name: again, component: loop}]}}";
    let page = "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById}]}";
    assert!(
        !accepted(widgets, page),
        "a self-containing widget was accepted"
    );
}

/// `Degrades.fields.(capability)` is an enum of twelve capabilities; a misspelt key is not a
/// fallback for anything.
#[test]
fn a_degrades_key_that_names_no_capability_is_refused() {
    let page = "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
                degrades: {no_chartz: table}}]}";
    assert!(
        !accepted("{}", page),
        "`no_chartz` was accepted as a capability"
    );
}

/// `link.exactly_one_of: [to, href]`.
#[test]
fn a_link_with_both_to_and_href_is_refused() {
    let page = "{kind: detail_page, title: P, sections: [{name: summary, reads: t.ById, \
                children: [{name: out, primitive: link, text: Out, to: {to: p}, href: row.url}]}]}";
    assert!(
        !accepted("{}", page),
        "a link with both `to` and `href` was accepted"
    );
}
