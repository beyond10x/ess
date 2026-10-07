//! beyond10x/ess#284: a page names the ESS actor whose grants its commands are bound to, as
//! `actor: <actor>`. The loader keeps the name as written; `ess ui check --model` resolves it.

const DOCUMENT: &str = r"
format: ess-ui/1
app: console
model: console.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: desk
  sections: [{name: all, pages: [desk, open]}]
pages:
  desk:
    kind: detail_page
    title: Desk
    actor: console.desk.Operator
    sections:
      - {name: queue, component: collection, reads: {view: desk.Queue}, actions: [{name: take, does: desk.Take}]}
  open:
    kind: detail_page
    title: Open
    sections:
      - {name: queue, component: collection, reads: {view: desk.Queue}}
";

#[test]
fn a_page_actor_loads_as_written() {
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        document.pages["desk"].actor.as_deref(),
        Some("console.desk.Operator")
    );
    assert_eq!(document.pages["open"].actor, None);
}

#[test]
fn a_page_actor_is_a_name_not_a_structure() {
    let text = DOCUMENT.replace(
        "actor: console.desk.Operator",
        "actor: {name: console.desk.Operator}",
    );
    let error = ess_ui::load_str(&text).expect_err("a mapping is no actor name");
    let message = error.to_string();
    assert!(
        message.starts_with("pages/desk") && message.contains("expected a string"),
        "{message}"
    );
}
