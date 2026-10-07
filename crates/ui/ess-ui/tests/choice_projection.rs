//! beyond10x/ess#328: a choice over a view names the row field each option sends (`value`) and
//! the one it shows (`label`); without them the value comes, in order, from the read's `key`, the
//! row field named like the form field the choice picks for, the view's identity in the model,
//! `id`, and last the row itself.
//!
//! [`ess_ui::Choice::row_option`] is that one contract; the terminal renderer and the test runner
//! call it, and the React runtime mirrors it (`ess-ui-react/tests/choice_value.rs`).

use ess_ui::{Body, Choice, Composite, Document};
use serde_yaml::Value;

const DOCUMENT: &str = r"
format: ess-ui/1
app: releases
model: release.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: releases.new
  sections: [{name: all, pages: [releases.new]}]
pages:
  releases.new:
    kind: form_page
    title: New release
    state:
      repository: {type: string, class: page_state}
    sections:
      - name: pick
        component: choice
        reads: {view: release.Repositories}
        binds: state.repository
        value: repository_id
        label: location
      - name: form
        component: form
        does: release.Cut
        fields:
          - field: selected_repository
            as: choice
            choice: {component: choice, reads: {view: release.Repositories}, value: repository_id, label: location}
";

fn document() -> Document {
    ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"))
}

fn choice_at<'a>(document: &'a Document, section: &str) -> &'a Choice {
    let page = &document.pages["releases.new"];
    let found = page
        .sections
        .iter()
        .find(|candidate| candidate.name == section)
        .expect("the section exists");
    match &found.body {
        Body::Composite(Composite::Choice(choice)) => choice,
        Body::Composite(Composite::Form(form)) => {
            match form.fields[0].choice.as_deref().map(|node| &node.body) {
                Some(Body::Composite(Composite::Choice(choice))) => choice,
                other => panic!("a choice field, not {other:?}"),
            }
        }
        other => panic!("a choice, not {other:?}"),
    }
}

fn row(yaml: &str) -> Value {
    serde_yaml::from_str(yaml).expect("the row parses")
}

/// The standalone choice with `value` and `label` replaced by `yaml` (one line, or nothing).
fn plain(yaml: &str) -> Choice {
    let extra = if yaml.is_empty() {
        String::new()
    } else {
        format!("        {yaml}\n")
    };
    let text = DOCUMENT.replace(
        "        value: repository_id\n        label: location\n",
        &extra,
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    choice_at(&document, "pick").clone()
}

#[test]
fn a_standalone_and_a_field_choice_load_their_value_and_label_fields() {
    let document = document();
    for section in ["pick", "form"] {
        let choice = choice_at(&document, section);
        assert_eq!(choice.value.as_deref(), Some("repository_id"), "{section}");
        assert_eq!(choice.label.as_deref(), Some("location"), "{section}");
    }
}

/// The receiving field is named `selected_repository`, which no row carries: the option still
/// sends the row's `repository_id` and shows its `location`.
#[test]
fn an_explicit_value_and_label_project_a_differently_named_field() {
    let document = document();
    let choice = choice_at(&document, "form");
    let option = choice.row_option(
        &row("{repository_id: repo-1, location: example/one, id: wrong}"),
        Some("selected_repository"),
        None,
    );
    assert_eq!(
        option,
        Some((Value::from("repo-1"), Value::from("example/one")))
    );
}

/// An explicit `value` beats the same-named field, the model identity and `id`.
#[test]
fn an_explicit_value_is_authoritative() {
    let choice = choice_at(&document(), "pick").clone();
    let option = choice.row_option(
        &row("{repository_id: repo-1, location: here, selected: no, id: wrong, uid: wrong}"),
        Some("selected"),
        Some("uid"),
    );
    assert_eq!(option, Some((Value::from("repo-1"), Value::from("here"))));
}

/// A non-string identity keeps its type: the option's value is the number, not its text.
#[test]
fn a_typed_value_keeps_its_type() {
    let choice = plain("value: repository_no");
    let option = choice.row_option(&row("{repository_no: 7, name: Seven}"), None, None);
    assert_eq!(option, Some((Value::from(7), Value::from("Seven"))));
}

/// A row without the field `value` names offers no option, so no wrong value can be sent; a
/// row without the `label` field shows the default label instead.
#[test]
fn a_missing_value_field_drops_the_row_and_a_missing_label_falls_back() {
    let choice = choice_at(&document(), "pick").clone();
    assert_eq!(
        choice.row_option(&row("{location: nowhere, id: wrong}"), None, None),
        None
    );
    assert_eq!(
        choice.row_option(&row("{repository_id: repo-2, name: Two}"), None, None),
        Some((Value::from("repo-2"), Value::from("Two")))
    );
    assert_eq!(
        choice.row_option(&row("{repository_id: repo-3}"), None, None),
        Some((Value::from("repo-3"), Value::from("repo-3")))
    );
}

/// The read's `key` names the row's identity, so a choice without `value` sends it.
#[test]
fn the_reads_key_is_the_value_when_no_value_is_written() {
    let text = DOCUMENT.replace(
        "        reads: {view: release.Repositories}\n        binds: state.repository\n        value: repository_id\n        label: location\n",
        "        reads: {view: release.Repositories, key: repository_id}\n        binds: state.repository\n",
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let choice = choice_at(&document, "pick");
    assert_eq!(choice.value_field(), Some("repository_id"));
    assert_eq!(
        choice.row_option(
            &row("{repository_id: repo-1, selected: no, id: wrong}"),
            Some("selected"),
            Some("uid"),
        ),
        Some((Value::from("repo-1"), Value::from("repo-1")))
    );
    assert_eq!(
        choice.row_option(&row("{id: wrong}"), None, None),
        None,
        "a key the row lacks offers no option"
    );
}

/// Without `value` or `key`: the same-named field (beyond10x/ess#343, unchanged), then the
/// model's identity, then `id`, then the row itself.
#[test]
fn without_an_explicit_field_the_fallbacks_hold_in_order() {
    let choice = plain("");
    let full = row("{repository_id: same, uid: identity, id: id}");
    assert_eq!(
        choice.row_option(&full, Some("repository_id"), Some("uid")),
        Some((Value::from("same"), Value::from("same"))),
        "the same-named field first"
    );
    assert_eq!(
        choice.row_option(&full, Some("selected_repository"), Some("uid")),
        Some((Value::from("identity"), Value::from("identity"))),
        "then the model identity"
    );
    assert_eq!(
        choice.row_option(&full, Some("selected_repository"), None),
        Some((Value::from("id"), Value::from("id"))),
        "then `id`"
    );
    let bare = row("{location: there}");
    assert_eq!(
        choice.row_option(&bare, None, None),
        Some((bare.clone(), bare.clone())),
        "then the row itself"
    );
    assert_eq!(
        choice.row_option(&row("{id: 1, label: One, name: Uno}"), None, None),
        Some((Value::from(1), Value::from("One"))),
        "the default label is `label`, else `name`, else the value"
    );
}

/// A label field present as `null` is treated as absent: the label falls back to `label`, then
/// `name`, then the value. A value field present as `null` is present: the option sends `null`,
/// as it did before any of this (beyond10x/ess#328, adversary pass 1).
#[test]
fn a_null_label_falls_back_and_a_null_value_is_sent() {
    let choice = choice_at(&document(), "pick").clone();
    assert_eq!(
        choice.row_option(
            &row("{repository_id: repo-1, location: null, label: null, name: One}"),
            None,
            None
        ),
        Some((Value::from("repo-1"), Value::from("One")))
    );
    let plain = plain("");
    assert_eq!(
        plain.row_option(
            &row("{parent_id: null, item_id: item-1, label: One}"),
            Some("parent_id"),
            Some("item_id")
        ),
        Some((Value::Null, Value::from("One"))),
        "a same-named field present as null is sent, not the identity"
    );
}
