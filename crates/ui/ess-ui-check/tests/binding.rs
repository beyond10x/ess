//! The binding contract: an `ess-ui/1` document bound to the HTTP surface ESS synthesizes, through
//! a route table computed from the model by `ess_gen::http::routes` (beyond10x/ess#311, S1).
//!
//! Every path the binding hands a renderer is checked against the derivation the synthesized
//! servers answer, never against a literal alone, so a binding that computed its own paths would
//! fail here the first time a wire name moved.

use std::path::{Path, PathBuf};

use ess_gen::http::{self, Served};
use ess_ui::binding::Binding;
use ess_ui_check::{binding, compile_sources, BindingError, Refusal};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the crate is three levels below the repository root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// `examples/gatepass`, as `(label, text)` sources, with `extra_views` appended to its one domain
/// (whose last key is `views:`).
fn gatepass_with(extra_views: &str) -> Vec<(String, String)> {
    let root = repository().join("examples/gatepass");
    let mut visit = read(&root.join("domains/visit.yaml"));
    visit.push_str(extra_views);
    vec![
        ("system.yaml".to_owned(), read(&root.join("system.yaml"))),
        (
            "components.yaml".to_owned(),
            read(&root.join("components.yaml")),
        ),
        ("domains/visit.yaml".to_owned(), visit),
    ]
}

fn gatepass() -> Vec<(String, String)> {
    gatepass_with("")
}

/// The `shop` fixture model the model checks use, with each `(from, to)` replacement applied to
/// its components file.
fn shop_with_components(replacements: &[(&str, &str)]) -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/model");
    let mut components = read(&root.join("components.yaml"));
    for (from, to) in replacements {
        assert!(components.contains(from), "`{from}` is not in the fixture");
        components = components.replace(from, to);
    }
    vec![
        ("system.yaml".to_owned(), read(&root.join("system.yaml"))),
        ("components.yaml".to_owned(), components),
        (
            "domains/stock.yaml".to_owned(),
            read(&root.join("domains/stock.yaml")),
        ),
        (
            "domains/audit.yaml".to_owned(),
            read(&root.join("domains/audit.yaml")),
        ),
    ]
}

/// A document of system `model` whose one page is `page`, with `state` as the page's state.
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

fn refused_at<'a>(refusals: &'a [Refusal], path: &str) -> &'a Refusal {
    refusals
        .iter()
        .find(|refusal| refusal.path == path)
        .unwrap_or_else(|| panic!("no refusal at `{path}`; refusals: {refusals:#?}"))
}

/// The path `http::routes` gives the construct named `qualified` on component `component`.
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
        .unwrap_or_else(|| panic!("`{qualified}` has no route on the component"))
        .path
}

const DESK: &str = "{kind: detail_page, title: Desk, sections: [{name: expected, \
                    component: collection, reads: visit.ExpectedVisits, \
                    actions: [{name: register, does: visit.RegisterVisit}]}]}";

#[test]
fn a_view_and_a_command_bind_to_the_paths_the_served_surface_answers() {
    let sources = gatepass();
    let binding = bound(&document("gatepass", "fat", DESK), &sources);
    assert_eq!(binding.system, "gatepass");
    assert_eq!(
        binding.components.keys().collect::<Vec<_>>(),
        ["pass-service"]
    );
    let served = &binding.components["pass-service"];

    let view = &served.views["gatepass.visit.ExpectedVisits"];
    assert_eq!(view.path, "/visits/views/expected");
    assert_eq!(
        view.path,
        derived(&sources, "pass-service", "gatepass.visit.ExpectedVisits")
    );
    assert!(view.params.is_empty());

    let command = &served.commands["gatepass.visit.RegisterVisit"];
    assert_eq!(command.path, "/visits/commands/register-visit");
    assert_eq!(
        command.path,
        derived(&sources, "pass-service", "gatepass.visit.RegisterVisit")
    );
    assert!(command.body_required);
    let refused = &command.errors["gatepass.visit.InvalidVisitLength"];
    assert_eq!(refused.status, 422);

    // Only what the document names: gatepass serves two views and three commands.
    assert_eq!(served.views.len(), 1, "{served:#?}");
    assert_eq!(served.commands.len(), 1, "{served:#?}");
    assert_eq!(
        binding.names["visit.ExpectedVisits"],
        "gatepass.visit.ExpectedVisits"
    );
    assert_eq!(
        binding.names["visit.RegisterVisit"],
        "gatepass.visit.RegisterVisit"
    );
}

#[test]
fn a_name_no_network_component_serves_is_refused_by_node_path() {
    // The fixture's one component, no longer reached over a network: it serves nothing over HTTP.
    let sources = shop_with_components(&[("    reached_by: network\n", "")]);
    let text = document(
        "shop",
        "fat",
        "{kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items, \
         actions: [{name: add, does: stock.AddItem}]}]}",
    );
    let refusals = refusals_of(&text, &sources);
    let view = refused_at(&refusals, "pages/p/sections/summary/reads");
    assert!(view.message.contains("shop.stock.Items"), "{view:?}");
    assert!(view.message.contains("reached_by: network"), "{view:?}");
    let command = refused_at(&refusals, "pages/p/sections/summary/actions/add");
    assert!(
        command.message.contains("shop.stock.AddItem"),
        "{command:?}"
    );
    assert_eq!(refusals.len(), 2, "{refusals:#?}");

    // A name the model does not declare at all is refused at its node too.
    let sources = shop_with_components(&[]);
    let missing = document(
        "shop",
        "fat",
        "{kind: detail_page, title: P, sections: [{name: summary, reads: stock.Missing}]}",
    );
    let refusals = refusals_of(&missing, &sources);
    refused_at(&refusals, "pages/p/sections/summary/reads");
}

#[test]
fn contested_paths_bind_to_the_qualified_fallback() {
    // `shop.shelf` is `shop.stock` under another name and the same wire names: both domains are
    // `stock`, both commands `add-item`, both views `items`, so every one of those paths is
    // contested and moves to the qualified name.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/model");
    let stock = read(&root.join("domains/stock.yaml"));
    let shelf = stock.replace("shop.stock", "shop.shelf");
    let sources = vec![
        (
            "system.yaml".to_owned(),
            "format: ess/1\nsystem: shop\nversion: v1\ndomains:\n  - shop.stock\n  - shop.shelf\n"
                .to_owned(),
        ),
        (
            "components.yaml".to_owned(),
            "components:\n  - component: shop-service\n    summary: Holds two shelves.\n    \
             owns:\n      domains: [shop.stock, shop.shelf]\n    accepts:\n      commands: \
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
        "{kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items, \
         actions: [{name: add, does: shelf.AddItem}]}, \
         {name: other, component: collection, reads: shelf.Items}]}",
    );
    let binding = bound(&text, &sources);
    let served = &binding.components["shop-service"];
    for (qualified, path) in [
        ("shop.stock.Items", "/views/shop.stock.Items"),
        ("shop.shelf.Items", "/views/shop.shelf.Items"),
    ] {
        assert_eq!(served.views[qualified].path, path);
        assert_eq!(path, derived(&sources, "shop-service", qualified));
    }
    let command = &served.commands["shop.shelf.AddItem"];
    assert_eq!(command.path, "/commands/shop.shelf.AddItem");
    assert_eq!(
        command.path,
        derived(&sources, "shop-service", "shop.shelf.AddItem")
    );
}

#[test]
fn a_non_scalar_view_param_is_refused() {
    let sources = gatepass_with(
        "\n  - name: gatepass.visit.EscortedBy\n    source: gatepass.visit.Visit\n    \
         consistency: eventual\n    params:\n      - name: deposit\n        type: \
         gatepass.visit.Deposit\n      - name: building\n        type: \
         gatepass.visit.Building\n    filter: [deposit.currency == param.deposit.currency, \
         building == param.building]\n    fields:\n      - name: visit_id\n        type: \
         gatepass.visit.VisitId\n",
    );
    let text = document(
        "gatepass",
        "fat",
        "{kind: detail_page, title: P, params: {who: string, site: string}, sections: \
         [{name: escorted, component: collection, reads: {view: visit.EscortedBy, \
         params: {deposit: params.who, building: params.site}}}]}",
    );
    let refusals = refusals_of(&text, &sources);
    let refusal = refused_at(&refusals, "pages/p/sections/escorted/reads");
    assert!(refusal.message.contains("`deposit`"), "{refusal:?}");
    assert!(refusal.message.contains("scalar"), "{refusal:?}");
    // `building` is an enum: a scalar, so it is not refused.
    assert_eq!(refusals.len(), 1, "{refusals:#?}");
}

/// A `Binary64` is a scalar a query value could carry, and no code target serves one
/// (`ess-synth/src/failure.rs`), so a read binding a view parameter of that type is refused as the
/// servers would refuse the model.
#[test]
fn a_binary64_view_param_is_refused() {
    let mut sources = gatepass_with(
        "\n  - name: gatepass.visit.Weighed\n    source: gatepass.visit.Visit\n    \
         consistency: eventual\n    params:\n      - name: ratio\n        type: Binary64\n      \
         - name: building\n        type: gatepass.visit.Building\n    filter: [param.ratio > \
         0, building == param.building]\n    fields:\n      - name: visit_id\n        type: \
         gatepass.visit.VisitId\n",
    );
    // `Binary64` is admitted from `ess/2`; the example is written at `ess/1`.
    sources[0].1 = sources[0].1.replace("format: ess/1", "format: ess/2");
    let text = document(
        "gatepass",
        "fat",
        "{kind: detail_page, title: P, params: {ratio: number, site: string}, sections: \
         [{name: weighed, component: collection, reads: {view: visit.Weighed, params: \
         {ratio: params.ratio, building: params.site}}}]}",
    );
    let refusals = refusals_of(&text, &sources);
    let refusal = refused_at(&refusals, "pages/p/sections/weighed/reads");
    assert!(refusal.message.contains("`ratio`"), "{refusal:?}");
    assert!(refusal.message.contains("Binary64"), "{refusal:?}");
    assert_eq!(refusals.len(), 1, "{refusals:#?}");
}

#[test]
fn server_held_state_is_refused_for_a_live_binding() {
    let sources = gatepass();
    let declared = document(
        "gatepass",
        "fat",
        "{kind: detail_page, title: Desk, state: {picked: {type: string, class: selection, \
         store: server_session}, saved: {type: string, class: preference, store: server}, \
         open: {type: boolean, class: component_state, store: memory}}, \
         sections: [{name: expected, component: collection, reads: visit.ExpectedVisits}]}",
    );
    let refusals = refusals_of(&declared, &sources);
    let picked = refused_at(&refusals, "pages/p/state/picked");
    assert!(picked.message.contains("server_session"), "{picked:?}");
    let saved = refused_at(&refusals, "pages/p/state/saved");
    assert!(saved.message.contains("`server`"), "{saved:?}");
    assert_eq!(refusals.len(), 2, "{refusals:#?}");

    // Placed by the profile rather than by `store:`: thin keeps a selection in the server
    // session.
    let resolved = document(
        "gatepass",
        "thin",
        "{kind: detail_page, title: Desk, state: {picked: {type: string, class: selection}}, \
         sections: [{name: expected, component: collection, reads: visit.ExpectedVisits}]}",
    );
    let refusals = refusals_of(&resolved, &sources);
    refused_at(&refusals, "pages/p/state/picked");
}

#[test]
fn a_server_paged_read_is_refused_by_name() {
    let sources = gatepass();
    for paging in ["server", "cursor", "append"] {
        let text = document(
            "gatepass",
            "fat",
            &format!(
                "{{kind: detail_page, title: Desk, sections: [{{name: expected, \
                 component: collection, reads: {{view: visit.ExpectedVisits, paging: {paging}}}}}]}}"
            ),
        );
        let refusals = refusals_of(&text, &sources);
        let refusal = refused_at(&refusals, "pages/p/sections/expected/reads");
        assert!(
            refusal.message.contains("gatepass.visit.ExpectedVisits"),
            "{refusal:?}"
        );
        assert!(refusal.message.contains(paging), "{refusal:?}");
    }
    for paging in ["client", "none"] {
        let text = document(
            "gatepass",
            "fat",
            &format!(
                "{{kind: detail_page, title: Desk, sections: [{{name: expected, \
                 component: collection, reads: {{view: visit.ExpectedVisits, paging: {paging}}}}}]}}"
            ),
        );
        bound(&text, &sources);
    }
}

#[test]
fn the_binding_serialises_to_stable_json() {
    let sources = gatepass();
    let text = document("gatepass", "fat", DESK);
    let first = serde_json::to_string_pretty(&bound(&text, &sources)).expect("serialises");
    let second = serde_json::to_string_pretty(&bound(&text, &sources)).expect("serialises");
    assert_eq!(first, second);
    let expected = r#"{
  "system": "gatepass",
  "components": {
    "pass-service": {
      "views": {
        "gatepass.visit.ExpectedVisits": {
          "path": "/visits/views/expected",
          "params": []
        }
      },
      "commands": {
        "gatepass.visit.RegisterVisit": {
          "path": "/visits/commands/register-visit",
          "body_required": true,
          "errors": {
            "gatepass.visit.InvalidVisitLength": {
              "status": 422,
              "display": "InvalidVisitLength"
            }
          }
        }
      }
    }
  },
  "names": {
    "visit.ExpectedVisits": "gatepass.visit.ExpectedVisits",
    "visit.RegisterVisit": "gatepass.visit.RegisterVisit"
  }
}"#;
    assert_eq!(first, expected);
}
