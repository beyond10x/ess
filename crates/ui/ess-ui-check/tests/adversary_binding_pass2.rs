//! Adversary pass 2 on story:ui-binding-contract: the correction after pass 1 (`read_params` on
//! reads with no `params:` slot), error routes against the recorded server answers, and the
//! stability of the route table across spellings and source order.

use std::path::{Path, PathBuf};

use ess_ui::binding::Binding;
use ess_ui_check::{binding, check_source, model_from_sources, BindingError, Options};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repository() -> PathBuf {
    crate_dir()
        .ancestors()
        .nth(3)
        .expect("the crate is three levels below the repository root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn fixture(file: &str) -> String {
    read(&crate_dir().join("tests/fixtures/model").join(file))
}

/// The `shop` fixture with `extra_stock` appended to the stock domain (whose last key is `views:`).
fn shop(extra_stock: &str) -> Vec<(String, String)> {
    let mut stock = fixture("domains/stock.yaml");
    stock.push_str(extra_stock);
    vec![
        ("system.yaml".to_owned(), fixture("system.yaml")),
        ("components.yaml".to_owned(), fixture("components.yaml")),
        ("domains/stock.yaml".to_owned(), stock),
        (
            "domains/audit.yaml".to_owned(),
            fixture("domains/audit.yaml"),
        ),
    ]
}

fn gatepass() -> Vec<(String, String)> {
    let root = repository().join("examples/gatepass");
    vec![
        ("system.yaml".to_owned(), read(&root.join("system.yaml"))),
        (
            "components.yaml".to_owned(),
            read(&root.join("components.yaml")),
        ),
        (
            "domains/visit.yaml".to_owned(),
            read(&root.join("domains/visit.yaml")),
        ),
    ]
}

fn document(model: &str, page: &str) -> String {
    format!(
        "format: ess-ui/1\napp: desk\nmodel: {model}\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n\
         pages: {{p: {page}}}\n"
    )
}

fn bound(text: &str, sources: &[(String, String)]) -> Binding {
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    match binding(&document, sources) {
        Ok(binding) => binding,
        Err(BindingError::Refused(refusals)) => panic!("refused: {refusals:#?}"),
        Err(other) => panic!("{other}"),
    }
}

/// Every `read_params` finding `text` gets against `sources`, as `(path, message)`.
fn read_params_findings(text: &str, sources: &[(String, String)]) -> Vec<(String, String)> {
    let model = model_from_sources(sources, Path::new("shop")).unwrap_or_else(|e| panic!("{e}"));
    let report = check_source(
        text,
        "test.yaml",
        &crate_dir(),
        Some(&model),
        &Options::default(),
    );
    report
        .findings
        .iter()
        .filter(|finding| finding.check == "read_params")
        .map(|finding| (finding.path.clone(), finding.message.clone()))
        .collect()
}

const LABELLED: &str = "\n  - name: shop.stock.Labelled\n    source: shop.stock.Item\n    \
     consistency: read_your_writes\n    params:\n      - name: wanted\n        type: String\n    \
     filter: label == param.wanted\n    fields:\n      - name: item_id\n        type: \
     shop.stock.ItemId\n";

const HINTED: &str = "\n  - name: shop.stock.Hinted\n    source: shop.stock.Item\n    \
     consistency: read_your_writes\n    params:\n      - name: hint\n        type: \
     Optional<shop.stock.ItemId>\n    filter: item_id == param.hint\n    fields:\n      - name: \
     item_id\n        type: shop.stock.ItemId\n";

/// `export:` has a `params:` slot (`ess-ui.md`, Action: `params: optional expr`), and the
/// reference's own example binds it with `same_as(list)`. The correction treats an export as a
/// read that binds nothing, so the documented form is an error against any view with a required
/// parameter, although it carries exactly the parameters the section's read binds.
#[test]
fn adv2_an_export_that_binds_its_params_with_same_as_is_not_an_unbound_param_error() {
    let sources = shop(LABELLED);
    let text = document(
        "shop",
        "{kind: detail_page, title: P, params: {q: string}, sections: [{name: list, component: \
         collection, reads: {view: stock.Labelled, params: {wanted: params.q}}, actions: [{name: \
         export, export: {reads: stock.Labelled, as: csv, params: same_as(list)}}]}]}",
    );
    let found = read_params_findings(&text, &sources);
    assert_eq!(
        found,
        Vec::<(String, String)>::new(),
        "an export whose `params:` is `same_as(list)` binds what the list binds"
    );
}

/// The correction's rule must stay silent on a view whose only parameter is `Optional`, wherever
/// it is named without a `params:` slot.
#[test]
fn adv2_a_view_whose_only_param_is_optional_is_no_error_without_a_params_slot() {
    let sources = shop(HINTED);
    let text = "format: ess-ui/1\napp: desk\nmodel: shop\nplacement_profile: fat\n\
         shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
         navigation: {home: p, sections: [{name: all, pages: [p]}, {name: hinted, pages: \
         {from_view: stock.Hinted, page: p, param: q}}]}\n\
         pages: {p: {kind: detail_page, title: P, params: {q: string}, sections: [{name: s, \
         component: collection, reads: stock.Hinted, actions: [{name: export, export: {reads: \
         stock.Hinted, as: csv}}]}]}}\n";
    assert_eq!(
        read_params_findings(text, &sources),
        Vec::<(String, String)>::new()
    );
}

/// Every declared refusal the gatepass servers were recorded answering (`answers.json`) is in the
/// binding's error routes of the command that answered it, at the recorded status; an error two
/// commands share is in both.
#[test]
fn adv2_error_routes_carry_every_recorded_refusal_at_its_recorded_status() {
    let vectors: serde_json::Value = serde_json::from_str(&read(
        &repository().join("crates/ui/ess-ui/tests/vectors/answers.json"),
    ))
    .expect("the vectors are JSON");
    let text = document(
        "gatepass",
        "{kind: detail_page, title: Desk, sections: [{name: expected, component: collection, \
         reads: visit.ExpectedVisits, actions: [{name: register, does: visit.RegisterVisit}, \
         {name: admit, does: visit.AdmitVisitor}, {name: out, does: visit.SignOutVisitor}]}]}",
    );
    let binding = bound(&text, &gatepass());
    let served = &binding.components["pass-service"];
    let mut checked = 0;
    for vector in vectors["vectors"].as_array().expect("a vector list") {
        if vector["expected"]["answer"] != "refused" || vector["source"] == "constructed" {
            continue;
        }
        let command = vector["command"].as_str().expect("a command line");
        let path = command
            .strip_prefix("POST ")
            .and_then(|rest| rest.split(';').next())
            .expect("a POST");
        let (_, route) = served
            .commands
            .iter()
            .find(|(_, route)| route.path == path)
            .unwrap_or_else(|| panic!("no bound command at `{path}`"));
        let error = vector["expected"]["error"].as_str().expect("an error code");
        let declared = route
            .errors
            .get(error)
            .unwrap_or_else(|| panic!("`{path}` has no error route `{error}`: {route:#?}"));
        assert_eq!(
            u64::from(declared.status),
            vector["status"],
            "{path} {error}"
        );
        checked += 1;
    }
    assert!(checked >= 4, "{checked} recorded refusals checked");
    for command in [
        "gatepass.visit.AdmitVisitor",
        "gatepass.visit.SignOutVisitor",
    ] {
        let errors: Vec<(&str, u16)> = served.commands[command]
            .errors
            .iter()
            .map(|(code, route)| (code.as_str(), route.status))
            .collect();
        assert_eq!(
            errors,
            [("gatepass.visit.VisitStateConflict", 409)],
            "{command}"
        );
    }
}

/// A command with no declared error binds an empty error table, and serialises it.
#[test]
fn adv2_a_command_with_no_declared_error_binds_an_empty_error_table() {
    let text = document(
        "shop",
        "{kind: detail_page, title: P, sections: [{name: s, component: collection, reads: \
         stock.Items, actions: [{name: add, does: stock.AddItem}]}]}",
    );
    let binding = bound(&text, &shop(""));
    let route = &binding.components["shop-service"].commands["shop.stock.AddItem"];
    assert!(route.errors.is_empty(), "{route:#?}");
    let json = serde_json::to_value(&binding).expect("serialises");
    assert_eq!(
        json["components"]["shop-service"]["commands"]["shop.stock.AddItem"]["errors"],
        serde_json::json!({})
    );
}

/// One view written three ways binds one route and three names, and the JSON is the same bytes
/// whatever order the model's files are given in.
#[test]
fn adv2_spellings_and_source_order_do_not_move_the_json() {
    let text = document(
        "shop",
        "{kind: detail_page, title: P, sections: [{name: a, component: collection, reads: \
         stock.Items}, {name: b, component: collection, reads: shop.stock.Items}, {name: c, \
         component: collection, reads: stock.Items, actions: [{name: add, does: \
         shop.stock.AddItem}, {name: again, does: stock.AddItem}]}]}",
    );
    let sources = shop("");
    let mut reversed = sources.clone();
    reversed.reverse();
    let forward = bound(&text, &sources);
    let backward = bound(&text, &reversed);
    let served = &forward.components["shop-service"];
    assert_eq!(served.views.len(), 1, "{served:#?}");
    assert_eq!(served.commands.len(), 1, "{served:#?}");
    assert_eq!(
        forward.names.iter().collect::<Vec<_>>(),
        [
            (
                &"shop.stock.AddItem".to_owned(),
                &"shop.stock.AddItem".to_owned()
            ),
            (
                &"shop.stock.Items".to_owned(),
                &"shop.stock.Items".to_owned()
            ),
            (
                &"stock.AddItem".to_owned(),
                &"shop.stock.AddItem".to_owned()
            ),
            (&"stock.Items".to_owned(), &"shop.stock.Items".to_owned()),
        ]
    );
    assert_eq!(
        serde_json::to_string(&forward).expect("serialises"),
        serde_json::to_string(&backward).expect("serialises")
    );
}
