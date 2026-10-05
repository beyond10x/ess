//! A page's `actor` binds the commands of the page to one actor's grants (beyond10x/ess#284).
//!
//! The grant semantics are the model's own (`ActorSpec::may_invoke`): an actor may invoke exactly
//! the commands its `may` lists. A command no actor is granted is granted to nobody — a served
//! surface refuses it to every caller — and a model that serves nothing leaves enforcing the grant
//! to its caller, which a UI is. A page without `actor`, and a check without a model, keep the
//! result they had.

use std::path::{Path, PathBuf};

use ess_ui_check::{
    check_source, exit_code, model_from_sources, CheckArgs, Finding, Model, Options, OutputFormat,
    Report,
};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture(file: &str) -> String {
    std::fs::read_to_string(crate_dir().join("tests/fixtures/model").join(file))
        .unwrap_or_else(|error| panic!("{file}: {error}"))
}

/// The fixture model's sources, with `actors` appended to the audit domain and every source
/// passed through `edit` (by label).
fn sources(actors: &str, edit: impl Fn(&str, String) -> String) -> Vec<(String, String)> {
    let mut audit = fixture("domains/audit.yaml");
    if !actors.is_empty() {
        audit.push_str("\nactors:\n");
        audit.push_str(actors);
    }
    [
        ("system.yaml", fixture("system.yaml")),
        ("components.yaml", fixture("components.yaml")),
        ("domains/stock.yaml", fixture("domains/stock.yaml")),
        ("domains/audit.yaml", audit),
    ]
    .into_iter()
    .map(|(label, text)| (label.to_owned(), edit(label, text)))
    .collect()
}

fn compile(sources: &[(String, String)]) -> Model {
    model_from_sources(sources, Path::new("shop")).unwrap_or_else(|error| panic!("{error}"))
}

/// The fixture model: `shop.stock.Clerk` may add items; nobody may record an audit entry.
fn fixture_model() -> Model {
    compile(&sources("", |_, text| text))
}

/// The fixture model with more actors declared in the audit domain.
fn model_with(actors: &str) -> Model {
    compile(&sources(actors, |_, text| text))
}

const AUDITOR: &str = "  - name: shop.audit.Auditor\n    may:\n      - shop.audit.RecordEntry\n";
const OBSERVER: &str = "  - name: shop.audit.Observer\n";

/// A document of one page `p`, whose body is `page`, plus `extra` top-level keys.
fn document(page: &str, extra: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n\
         pages: {{p: {page}}}\n{extra}"
    )
}

/// Page `p` with `actor` (when given) and one section whose actions send `commands`, by name.
fn binding(actor: Option<&str>, commands: &[&str]) -> String {
    let actions: Vec<String> = commands
        .iter()
        .enumerate()
        .map(|(index, command)| format!("{{name: a{index}, does: {command}}}"))
        .collect();
    let actor = actor.map_or(String::new(), |actor| format!("actor: {actor}, "));
    format!(
        "{{kind: detail_page, title: P, {actor}sections: [{{name: summary, reads: stock.Items, \
         actions: [{}]}}]}}",
        actions.join(", ")
    )
}

fn check(text: &str, model: Option<&Model>) -> Report {
    check_source(text, "ui.yaml", &crate_dir(), model, &Options::default())
}

fn tripped<'a>(report: &'a Report, id: &str) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == id)
        .collect()
}

/// The findings of the two page-actor checks.
fn actor_findings(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == "page_actor_grants" || finding.check == "actor_in_model")
        .collect()
}

/// The report's findings other than those of the two page-actor checks.
fn other_findings(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check != "page_actor_grants" && finding.check != "actor_in_model")
        .collect()
}

fn assert_refused(report: &Report, at: &str, actor: &str, command: &str) {
    let found = tripped(report, "page_actor_grants");
    let finding = found
        .iter()
        .find(|finding| finding.path == at)
        .unwrap_or_else(|| panic!("no `page_actor_grants` at `{at}`: {:#?}", report.findings));
    for named in [
        "page `p`".to_owned(),
        format!("`{actor}`"),
        format!("`{command}`"),
    ] {
        assert!(finding.message.contains(&named), "{named}: {finding:?}");
    }
    assert!(report.has_errors(), "{:#?}", report.findings);
}

/// The reduction of beyond10x/ess#284: a page declaring `actor: <domain>.Operator` binds a command
/// the Operator is not granted. `ess ui check --model` exits 1 and names page, actor and command.
#[test]
fn the_284_reduction_fails_ess_ui_check_naming_page_actor_and_command() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("page_actor_284");
    let _ = std::fs::remove_dir_all(&root);
    let operator = "  - name: shop.audit.Operator\n    may:\n      - shop.stock.AddItem\n";
    for (label, text) in sources(operator, |_, text| text) {
        let path = root.join("model").join(label);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("the model dir");
        std::fs::write(path, text).expect("a model file");
    }
    let page = "{kind: detail_page, title: P, actor: audit.Operator, sections: [{name: summary, \
                reads: stock.Items, actions: [{name: add, does: stock.AddItem}]}], \
                overlays: {note: {kind: drawer, component: form, does: audit.RecordEntry, \
                fields: [note]}}}";
    let ui = root.join("ui.yaml");
    std::fs::write(&ui, document(page, "")).expect("the document");
    let args = CheckArgs {
        path: ui.clone(),
        model: Some(root.join("model")),
        format: OutputFormat::Text,
        lacks: Vec::new(),
    };
    assert_eq!(exit_code(&args).expect("the check runs"), 1);
    let report = ess_ui_check::check(&ui, Some(&root.join("model"))).expect("the check runs");
    assert_refused(
        &report,
        "pages/p/overlays/note",
        "shop.audit.Operator",
        "shop.audit.RecordEntry",
    );
    assert_eq!(
        tripped(&report, "page_actor_grants").len(),
        1,
        "{report:#?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The same page passes once the Operator holds the grant.
#[test]
fn a_page_actor_granted_every_bound_command_passes() {
    let model = model_with(
        "  - name: shop.audit.Operator\n    may:\n      - shop.stock.AddItem\n      - \
         shop.audit.RecordEntry\n",
    );
    let report = check(
        &document(
            &binding(
                Some("audit.Operator"),
                &["stock.AddItem", "audit.RecordEntry"],
            ),
            "",
        ),
        Some(&model),
    );
    assert!(actor_findings(&report).is_empty(), "{:#?}", report.findings);
    assert!(!report.has_errors(), "{:#?}", report.findings);
}

/// Mismatched: another actor holds the grant. The pooled grants of every actor do not admit it.
#[test]
fn mismatched_a_command_granted_to_another_actor_is_refused() {
    let model = model_with(AUDITOR);
    let refused = check(
        &document(&binding(Some("stock.Clerk"), &["audit.RecordEntry"]), ""),
        Some(&model),
    );
    assert_refused(
        &refused,
        "pages/p/sections/summary/actions/a0",
        "shop.stock.Clerk",
        "shop.audit.RecordEntry",
    );
    let granted = check(
        &document(
            &binding(Some("shop.audit.Auditor"), &["audit.RecordEntry"]),
            "",
        ),
        Some(&model),
    );
    assert!(
        actor_findings(&granted).is_empty(),
        "{:#?}",
        granted.findings
    );
}

/// Read-only: an actor that may invoke nothing binds a command.
#[test]
fn read_only_an_actor_granted_nothing_is_refused_every_command() {
    let model = model_with(OBSERVER);
    let refused = check(
        &document(&binding(Some("audit.Observer"), &["stock.AddItem"]), ""),
        Some(&model),
    );
    assert_refused(
        &refused,
        "pages/p/sections/summary/actions/a0",
        "shop.audit.Observer",
        "shop.stock.AddItem",
    );
    let reading = check(
        &document(&binding(Some("audit.Observer"), &[]), ""),
        Some(&model),
    );
    assert!(
        actor_findings(&reading).is_empty(),
        "{:#?}",
        reading.findings
    );
}

/// Open: a command no actor is granted is not open to everyone. The model grants it to nobody,
/// and its served surface refuses it to every caller, so the page actor may not bind it either.
#[test]
fn open_a_command_no_actor_is_granted_is_granted_to_nobody() {
    let report = check(
        &document(&binding(Some("stock.Clerk"), &["audit.RecordEntry"]), ""),
        Some(&fixture_model()),
    );
    assert_refused(
        &report,
        "pages/p/sections/summary/actions/a0",
        "shop.stock.Clerk",
        "shop.audit.RecordEntry",
    );
}

/// Caller: a model whose component serves nothing over the network leaves enforcing the grant to
/// its caller. The page is that caller, and is held to the same grants.
#[test]
fn caller_a_model_serving_nothing_still_holds_the_page_actor_to_its_grants() {
    let model = compile(&sources(AUDITOR, |label, text| {
        if label == "components.yaml" {
            text.replace("    reached_by: network\n", "")
        } else {
            text
        }
    }));
    let report = check(
        &document(
            &binding(Some("stock.Clerk"), &["stock.AddItem", "audit.RecordEntry"]),
            "",
        ),
        Some(&model),
    );
    assert_refused(
        &report,
        "pages/p/sections/summary/actions/a1",
        "shop.stock.Clerk",
        "shop.audit.RecordEntry",
    );
    assert_eq!(
        tripped(&report, "page_actor_grants").len(),
        1,
        "{report:#?}"
    );
}

/// A model that declares no actor says nothing about who may invoke what: a page without `actor`
/// keeps its findings, and a page that names an actor names nothing the model declares.
#[test]
fn a_model_declaring_no_actor_refuses_a_named_page_actor_and_grants_nothing_else() {
    let actorless = compile(&sources("", |label, text| {
        if label == "domains/stock.yaml" {
            text.replace(
                "actors:\n  - name: shop.stock.Clerk\n    may:\n      - shop.stock.AddItem\n    \
                 naming:\n      display: Clerk\n\n",
                "",
            )
        } else {
            text
        }
    }));
    let without = check(
        &document(&binding(None, &["stock.AddItem"]), ""),
        Some(&actorless),
    );
    assert!(
        actor_findings(&without).is_empty(),
        "{:#?}",
        without.findings
    );
    let named = check(
        &document(&binding(Some("stock.Clerk"), &["stock.AddItem"]), ""),
        Some(&actorless),
    );
    let finding = tripped(&named, "actor_in_model");
    assert_eq!(finding.len(), 1, "{:#?}", named.findings);
    assert_eq!(finding[0].path, "pages/p/actor");
    assert!(
        finding[0].message.contains("declares none"),
        "{:?}",
        finding[0]
    );
    assert_eq!(
        tripped(&named, "page_actor_grants").len(),
        0,
        "an unknown page actor decides no grant"
    );
}

/// A page without `actor` keeps exactly the findings it had: the page actor only adds its own two
/// checks, and nothing else in a report moves when one is written.
#[test]
fn a_page_without_actor_keeps_todays_result() {
    let model = model_with(AUDITOR);
    let commands = ["stock.AddItem", "audit.RecordEntry", "stock.Missing"];
    let without = check(&document(&binding(None, &commands), ""), Some(&model));
    assert!(
        actor_findings(&without).is_empty(),
        "{:#?}",
        without.findings
    );
    let with = check(
        &document(&binding(Some("stock.Clerk"), &commands), ""),
        Some(&model),
    );
    assert_eq!(other_findings(&with), other_findings(&without));
    assert_eq!(
        tripped(&with, "page_actor_grants").len(),
        1,
        "an unresolved command is `command_in_model`'s, not refused again: {:#?}",
        with.findings
    );
}

/// Without a model nothing resolves, and the page actor is not checked.
#[test]
fn without_a_model_a_page_actor_is_not_checked() {
    let text = document(&binding(Some("stock.Clerk"), &["audit.RecordEntry"]), "");
    let report = check(&text, None);
    assert!(actor_findings(&report).is_empty(), "{:#?}", report.findings);
    assert!(!report.has_errors(), "{:#?}", report.findings);
}

/// Every command the page binds is held — section actions, the page's overlays and header
/// actions, a form's `does` — and only those: a shell's account menu, and another page without an
/// actor, are not the page's.
#[test]
fn every_command_the_page_binds_is_held_and_no_other() {
    let model = fixture_model();
    let text = format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}, account: {{kind: account_menu, \
         props: {{actions: [{{name: note, does: audit.RecordEntry, label: Note}}]}}}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p, q]}}]}}\n\
         pages:\n  p: {}\n  q: {}\n",
        "{kind: detail_page, title: P, actor: stock.Clerk, \
         header: {actions: [{name: top, opens: note, label: Note}, \
         {name: bulk, upload: {accept: [text/csv], does: audit.RecordEntry}, label: Import}]}, \
         sections: [{name: summary, reads: stock.Items, actions: [{name: add, does: stock.AddItem}]}, \
         {name: entry, component: form, does: audit.RecordEntry, fields: [note]}], \
         overlays: {note: {kind: drawer, component: form, does: audit.RecordEntry, fields: [note]}}}",
        "{kind: detail_page, title: Q, sections: [{name: summary, reads: stock.Items, \
         actions: [{name: note, does: audit.RecordEntry}]}]}",
    );
    let report = check(&text, Some(&model));
    let mut paths: Vec<&str> = tripped(&report, "page_actor_grants")
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    paths.sort_unstable();
    assert_eq!(
        paths,
        [
            "pages/p/header/actions/bulk",
            "pages/p/overlays/note",
            "pages/p/sections/entry",
        ],
        "{:#?}",
        report.findings
    );
}

/// A retrofit's `UNMAPPED:` actor names nobody: `unmapped_reported` warns, and no grant decides.
#[test]
fn an_unmapped_page_actor_decides_nothing() {
    let text = document(
        &binding(
            Some("\"UNMAPPED: the role table is not read\""),
            &["audit.RecordEntry"],
        ),
        "",
    );
    let report = check(&text, Some(&fixture_model()));
    assert!(actor_findings(&report).is_empty(), "{:#?}", report.findings);
    assert!(
        !tripped(&report, "unmapped_reported").is_empty(),
        "{:#?}",
        report.findings
    );
}
