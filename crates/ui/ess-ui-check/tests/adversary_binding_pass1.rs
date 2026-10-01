//! Adversary pass 1 on story:ui-binding-contract: the route table held to the surfaces the
//! synthesized servers actually serve, and `read_params` held to every place a view is read.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use ess_gen::http::{self, Served};
use ess_ui::binding::Binding;
use ess_ui_check::{
    binding, check_source, compile_sources, model_from_sources, BindingError, Options, Refusal,
};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn fixture(file: &str) -> String {
    read(&crate_dir().join("tests/fixtures/model").join(file))
}

/// The `shop` fixture with `components` as its components file and `extra_stock` appended to the
/// stock domain (whose last key is `views:`).
fn shop(components: &str, extra_stock: &str) -> Vec<(String, String)> {
    let mut stock = fixture("domains/stock.yaml");
    stock.push_str(extra_stock);
    vec![
        ("system.yaml".to_owned(), fixture("system.yaml")),
        ("components.yaml".to_owned(), components.to_owned()),
        ("domains/stock.yaml".to_owned(), stock),
        (
            "domains/audit.yaml".to_owned(),
            fixture("domains/audit.yaml"),
        ),
    ]
}

fn document(model: &str, profile: &str, page: &str) -> String {
    format!(
        "format: ess-ui/1\napp: desk\nmodel: {model}\nplacement_profile: {profile}\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n\
         pages: {{p: {page}}}\n"
    )
}

fn bind(text: &str, sources: &[(String, String)]) -> Result<Binding, BindingError> {
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    binding(&document, sources)
}

fn bound(text: &str, sources: &[(String, String)]) -> Binding {
    bind(text, sources).unwrap_or_else(|error| panic!("the document does not bind: {error}"))
}

fn refusals_of(text: &str, sources: &[(String, String)]) -> Vec<Refusal> {
    match bind(text, sources) {
        Ok(binding) => panic!("the document binds: {binding:#?}"),
        Err(BindingError::Refused(refusals)) => refusals,
        Err(other) => panic!("not a refusal by node path: {other}"),
    }
}

fn derived(sources: &[(String, String)], component: &str, qualified: &str) -> String {
    let ir = compile_sources(sources, Path::new("test")).unwrap_or_else(|error| panic!("{error}"));
    let component = ir
        .components()
        .values()
        .find(|candidate| candidate.name.to_string() == component)
        .unwrap_or_else(|| panic!("no component `{component}`"));
    http::routes(&ir, component)
        .into_iter()
        .find(|route| match route.serves {
            Served::Command(handle) => handle.to_string() == qualified,
            Served::View(handle) => handle.to_string() == qualified,
        })
        .unwrap_or_else(|| panic!("`{qualified}` has no route on `{component:?}`"))
        .path
}

const TWO_COMPONENTS: &str = "components:\n  - component: stock-service\n    summary: Stock.\n    \
     owns:\n      domains: [shop.stock]\n    accepts:\n      commands: [shop.stock.AddItem]\n    \
     publishes:\n      events: [shop.stock.ItemAdded]\n    reached_by: network\n  - component: \
     audit-service\n    summary: Audit.\n    owns:\n      domains: [shop.audit]\n    accepts:\n      \
     commands: [shop.audit.RecordEntry]\n    publishes:\n      events: [shop.audit.EntryRecorded]\n\
     AUDIT_REACH";

const BOTH: &str = "{kind: detail_page, title: P, sections: [{name: items, component: collection, reads: stock.Items, \
     actions: [{name: add, does: shop.stock.AddItem}]}, {name: entries, component: collection, \
     reads: shop.audit.Entries, actions: [{name: record, does: audit.RecordEntry}]}]}";

#[test]
fn adv1_two_network_components_each_bind_their_own_routes() {
    let sources = shop(
        &TWO_COMPONENTS.replace("AUDIT_REACH", "    reached_by: network\n"),
        "",
    );
    let binding = bound(&document("shop", "fat", BOTH), &sources);
    assert_eq!(
        binding.components.keys().collect::<Vec<_>>(),
        ["audit-service", "stock-service"]
    );
    let stock = &binding.components["stock-service"];
    let audit = &binding.components["audit-service"];
    assert_eq!(stock.views["shop.stock.Items"].path, "/stock/views/items");
    assert_eq!(
        stock.commands["shop.stock.AddItem"].path,
        "/stock/commands/add-item"
    );
    assert_eq!(
        audit.views["shop.audit.Entries"].path,
        "/audit/views/entries"
    );
    assert_eq!(
        audit.commands["shop.audit.RecordEntry"].path,
        "/audit/commands/record-entry"
    );
    assert_eq!(
        audit.commands["shop.audit.RecordEntry"].path,
        derived(&sources, "audit-service", "shop.audit.RecordEntry")
    );
    assert!(!stock.views.contains_key("shop.audit.Entries"));
    assert_eq!(binding.names["shop.stock.AddItem"], "shop.stock.AddItem");
    assert_eq!(binding.names["stock.Items"], "shop.stock.Items");
}

#[test]
fn adv1_only_the_names_a_non_network_component_holds_are_refused() {
    let sources = shop(&TWO_COMPONENTS.replace("AUDIT_REACH", ""), "");
    let refusals = refusals_of(&document("shop", "fat", BOTH), &sources);
    let paths: Vec<&str> = refusals
        .iter()
        .map(|refusal| refusal.path.as_str())
        .collect();
    assert_eq!(
        paths,
        [
            "pages/p/sections/entries/actions/record",
            "pages/p/sections/entries/reads"
        ],
        "{refusals:#?}"
    );
}

#[test]
fn adv1_a_contested_command_moves_even_when_the_document_names_only_one_side() {
    let stock = fixture("domains/stock.yaml");
    let shelf = stock.replace("shop.stock", "shop.shelf");
    let sources = vec![
        (
            "system.yaml".to_owned(),
            "format: ess/1\nsystem: shop\nversion: v1\ndomains:\n  - shop.stock\n  - shop.shelf\n"
                .to_owned(),
        ),
        (
            "components.yaml".to_owned(),
            "components:\n  - component: shop-service\n    summary: Two shelves.\n    owns:\n      \
             domains: [shop.stock, shop.shelf]\n    accepts:\n      commands: \
             [shop.stock.AddItem, shop.shelf.AddItem]\n    publishes:\n      events: \
             [shop.stock.ItemAdded, shop.shelf.ItemAdded]\n    reached_by: network\n"
                .to_owned(),
        ),
        ("domains/stock.yaml".to_owned(), stock),
        ("domains/shelf.yaml".to_owned(), shelf),
    ];
    let text = document(
        "shop",
        "fat",
        "{kind: detail_page, title: P, sections: [{name: s, component: collection, reads: shop.stock.Items, \
         actions: [{name: add, does: stock.AddItem}]}]}",
    );
    let binding = bound(&text, &sources);
    let served = &binding.components["shop-service"];
    assert_eq!(
        served.commands["shop.stock.AddItem"].path,
        "/commands/shop.stock.AddItem"
    );
    assert_eq!(
        served.views["shop.stock.Items"].path,
        "/views/shop.stock.Items"
    );
}

/// A view two network components both own must not be bound, silently, to whichever one the
/// route table met last.
#[test]
fn adv1_a_view_two_network_components_serve_is_not_bound_silently_to_one() {
    let components = "components:\n  - component: a-service\n    summary: A.\n    owns:\n      \
         domains: [shop.stock, shop.audit]\n    accepts:\n      commands: [shop.stock.AddItem, \
         shop.audit.RecordEntry]\n    publishes:\n      events: [shop.stock.ItemAdded, \
         shop.audit.EntryRecorded]\n    reached_by: network\n  - component: b-service\n    \
         summary: B.\n    owns:\n      domains: [shop.stock]\n    reached_by: network\n";
    let sources = shop(components, "");
    let text = document(
        "shop",
        "fat",
        "{kind: detail_page, title: P, sections: [{name: s, component: collection, reads: stock.Items}]}",
    );
    match bind(&text, &sources) {
        Err(BindingError::Model(_) | BindingError::Refused(_)) => {}
        Ok(binding) => {
            let serving: Vec<&String> = binding
                .components
                .iter()
                .filter(|(_, served)| served.views.contains_key("shop.stock.Items"))
                .map(|(name, _)| name)
                .collect();
            assert_eq!(
                serving.len(),
                2,
                "two components own `shop.stock` and the binding chose {serving:?}"
            );
        }
    }
}

const PARAM_VIEW: &str = "\n  - name: shop.stock.Labelled\n    source: shop.stock.Item\n    \
     consistency: read_your_writes\n    params:\n      - name: wanted\n        type: String\n        \
     naming:\n          wire: q-wanted\n      - name: hint\n        type: Optional<shop.stock.ItemId>\n    \
     filter: {any: [label == param.wanted, item_id == param.hint]}\n    fields:\n      - name: \
     item_id\n        type: shop.stock.ItemId\n";

#[test]
fn adv1_query_params_carry_their_wire_names_requiredness_and_scalars() {
    let sources = shop(&fixture("components.yaml"), PARAM_VIEW);
    let text = document(
        "shop",
        "fat",
        "{kind: detail_page, title: P, params: {q: string}, sections: [{name: s, component: \
         collection, reads: {view: stock.Labelled, params: {wanted: params.q}}}]}",
    );
    let binding = bound(&text, &sources);
    let params = &binding.components["shop-service"].views["shop.stock.Labelled"].params;
    let seen: Vec<(&str, &str, bool, &str)> = params
        .iter()
        .map(|param| {
            (
                param.name.as_str(),
                param.wire.as_str(),
                param.required,
                param.scalar.as_str(),
            )
        })
        .collect();
    assert_eq!(
        seen,
        [
            ("wanted", "q-wanted", true, "String"),
            ("hint", "hint", false, "Uuid")
        ]
    );
}

/// Every code target refuses a model whose view declares `paging:` (`ess-synth/src/paging.rs`),
/// so no served surface answers that view at all; a binding that hands out its route points a
/// renderer at a server that cannot be synthesized.
#[test]
fn adv1_a_view_the_model_pages_is_refused_since_no_code_target_serves_it() {
    let root = crate_dir().join("../../verify/ess-conformance/tests/fixtures/view-paging.yaml");
    let sources = vec![
        ("view-paging.yaml".to_owned(), read(&root)),
        (
            "components.yaml".to_owned(),
            "components:\n  - component: job-service\n    summary: Jobs.\n    owns:\n      \
             domains: [demo.jobs]\n    accepts:\n      commands: [demo.jobs.CreateJob]\n    \
             publishes:\n      events: [demo.jobs.JobCreated]\n    reached_by: network\n"
                .to_owned(),
        ),
    ];
    for reads in [
        "jobs.JobList",
        "{view: jobs.JobList, paging: client}",
        "{view: jobs.JobList, params: {page: '1', size: '10'}}",
    ] {
        let text = document(
            "demo",
            "fat",
            &format!(
                "{{kind: detail_page, title: P, sections: [{{name: s, component: collection, \
                 reads: {reads}}}]}}"
            ),
        );
        let refusals = refusals_of(&text, &sources);
        let refusal = refusals
            .iter()
            .find(|refusal| refusal.path == "pages/p/sections/s/reads")
            .unwrap_or_else(|| panic!("{reads}: no refusal at the read: {refusals:#?}"));
        assert!(refusal.message.contains("demo.jobs.JobList"), "{refusal:?}");
    }
}

/// `store: server` is an explicit placement; an UNMAPPED class (a warning, so the document still
/// loads and checks) must not hide it from the server-state refusal.
#[test]
fn adv1_explicit_server_store_is_refused_even_when_the_class_is_unmapped() {
    let sources = shop(&fixture("components.yaml"), "");
    for store in ["server", "server_session"] {
        let text = document(
            "shop",
            "fat",
            &format!(
                "{{kind: detail_page, title: P, state: {{saved: {{type: string, class: \
                 'UNMAPPED: retrofit could not tell', store: {store}}}}}, sections: [{{name: s, component: collection, \
                 reads: stock.Items}}]}}"
            ),
        );
        let refusals = refusals_of(&text, &sources);
        assert!(
            refusals
                .iter()
                .any(|refusal| refusal.path == "pages/p/state/saved"),
            "{store}: {refusals:#?}"
        );
    }
}

/// A dynamic menu reads its view with no `params:` at all, so a view with a required parameter
/// can never be answered there: `read_params` must say so as it does for a section's read.
#[test]
fn adv1_a_menu_built_from_a_view_with_a_required_param_is_a_read_params_error() {
    let sources = shop(&fixture("components.yaml"), PARAM_VIEW);
    let model = model_from_sources(&sources, Path::new("shop")).unwrap_or_else(|e| panic!("{e}"));
    let mut text = String::new();
    let _ = write!(
        text,
        "format: ess-ui/1\napp: desk\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}, {{name: labelled, pages: \
         {{from_view: stock.Labelled, page: p, param: q}}}}]}}\n\
         pages: {{p: {{kind: detail_page, title: P, params: {{q: string}}, sections: [{{name: s, component: collection, \
         reads: stock.Items}}]}}}}\n"
    );
    let report = check_source(
        &text,
        "test.yaml",
        &crate_dir(),
        Some(&model),
        &Options::default(),
    );
    let found: Vec<_> = report
        .findings
        .iter()
        .filter(|finding| finding.check == "read_params")
        .collect();
    assert!(
        found
            .iter()
            .any(|finding| finding.message.contains("`wanted`")),
        "no read_params finding for the menu's read of `shop.stock.Labelled`: {:#?}",
        report.findings
    );
}
