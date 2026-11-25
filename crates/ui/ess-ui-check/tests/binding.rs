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
    assert_eq!(view.params.len(), 0);

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

/// The `shop` fixture model with an enum `shop.stock.Shelf` whose first variant is sent as `top`
/// and shown as `Top shelf`.
fn shelved_shop() -> Vec<(String, String)> {
    let mut sources = shop_with_components(&[]);
    // Variant naming is a format ess/5 construct.
    let system = sources
        .iter_mut()
        .find(|(label, _)| label == "system.yaml")
        .expect("the system");
    system.1 = system.1.replace("format: ess/1", "format: ess/5");
    let stock = sources
        .iter_mut()
        .find(|(label, _)| label == "domains/stock.yaml")
        .expect("the stock domain");
    stock.1 = stock
        .1
        .replace(
            "    of: Uuid\n",
            "    of: Uuid\n  - name: shop.stock.Shelf\n    kind: enum\n    variants:\n      \
             - name: Top\n        wire: top\n        display: Top shelf\n      - Bottom\n",
        )
        // ess/5 maps every payload field.
        .replace(
            "            label: input.label\n",
            "            label: input.label\n            item_id: {generated: true}\n",
        );
    let audit = sources
        .iter_mut()
        .find(|(label, _)| label == "domains/audit.yaml")
        .expect("the audit domain");
    audit.1 = audit.1.replace(
        "            note: input.note\n",
        "            note: input.note\n            entry_id: {generated: true}\n",
    );
    sources
}

/// beyond10x/ess#328, #330: a view a choice reads carries its rows' identity field, the default
/// value of the choice's options; a model enum a choice's options name is carried with its
/// variants, under the name the document writes, so a renderer holding only the binding lists
/// them. A view no choice reads carries no identity, so a binding without choices keeps its bytes.
#[test]
fn a_choice_carries_its_views_identity_and_its_model_enums_variants() {
    let sources = shelved_shop();
    let model = ess_ui_check::model_from_sources(&sources, Path::new("shop"))
        .unwrap_or_else(|error| panic!("{error}"));
    let text = document(
        "shop",
        "fat",
        "{kind: detail_page, title: Shop, sections: [{name: summary, reads: stock.Items}, \
          {name: pick, component: choice, reads: stock.Items}, \
          {name: shelf, component: choice, options: stock.Shelf}]}",
    );
    let loaded = ess_ui::load_str_with(&text, &model).unwrap_or_else(|error| panic!("{error}"));
    let bound = binding(&loaded, &sources).unwrap_or_else(|error| panic!("{error}"));
    let items = bound
        .view("stock.Items")
        .expect("the choice's view is bound");
    assert_eq!(items.identity.as_deref(), Some("item_id"));
    assert_eq!(bound.names["stock.Shelf"], "shop.stock.Shelf");
    let variants: Vec<(&str, &str)> = bound.enums["shop.stock.Shelf"]
        .iter()
        .map(|variant| (variant.value.as_str(), variant.label.as_str()))
        .collect();
    assert_eq!(variants, [("top", "Top shelf"), ("Bottom", "Bottom")]);
    let reloaded = ess_ui::load_str_with(&text, &bound).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        reloaded, loaded,
        "the binding resolves what the model resolved"
    );

    let plain = document(
        "shop",
        "fat",
        "{kind: detail_page, title: Shop, sections: [{name: summary, reads: stock.Items}]}",
    );
    let bound = bound_with_model(&plain, &sources);
    assert_eq!(
        bound
            .view("stock.Items")
            .and_then(|view| view.identity.clone()),
        None
    );
    assert!(bound.enums.is_empty());
}

/// A page of model `shop` with a section slot reading `view` and sending `command`, a page slot
/// sending `page_command`, and a field slot reading nothing and sending nothing.
fn slotted(view: &str, command: &str, page_command: &str) -> String {
    format!(
        "format: ess-ui/1\napp: desk\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p, editor]}}]}}\n\
         pages:\n  \
           p: {{kind: detail_page, title: P, sections: [\
             {{name: map, host: StockMap, reason: a vendor map, stands_in_for: collection, \
               reads: {view}, commands: [{command}]}}, \
             {{name: summary, reads: stock.Items, fields: [{{field: label, host: LabelPad, \
               reason: a pen input, stands_in_for: rich_text}}]}}]}}\n  \
           editor: {{kind: detail_page, title: E, host: AuditCanvas, reason: a vendor canvas, \
             stands_in_for: graph_editor, commands: [{page_command}]}}\n"
    )
}

/// story:a-host-slot-reads-and-commands-only-what-it-declares: a host component reads and sends
/// only through the binding of what its slot declares, so the binding routes exactly the slot's
/// reads and commands, and refuses them as it refuses a generated node's: an unknown name, a name
/// no network component serves, and an operation an `ess-http/1` binding does not route.
#[test]
fn a_host_slots_reads_and_commands_bind_as_a_generated_nodes_do() {
    let sources = shop_with_components(&[]);
    let text = slotted("stock.Items", "stock.AddItem", "audit.RecordEntry");
    let binding = bound(&text, &sources);
    let served = &binding.components["shop-service"];
    assert_eq!(
        served.views.keys().collect::<Vec<_>>(),
        ["shop.stock.Items"],
        "{served:#?}"
    );
    assert_eq!(
        served.commands.keys().collect::<Vec<_>>(),
        ["shop.audit.RecordEntry", "shop.stock.AddItem"],
        "{served:#?}"
    );
    assert_eq!(
        served.commands["shop.stock.AddItem"].path,
        derived(&sources, "shop-service", "shop.stock.AddItem")
    );
    assert_eq!(binding.names["audit.RecordEntry"], "shop.audit.RecordEntry");

    // Names the model does not declare.
    let unknown = slotted("stock.Missing", "stock.Nope", "audit.Gone");
    let refusals = refusals_of(&unknown, &sources);
    for (path, name) in [
        ("pages/p/sections/map/reads", "`stock.Missing`"),
        ("pages/p/sections/map/commands", "`stock.Nope`"),
        ("pages/editor/commands", "`audit.Gone`"),
    ] {
        let refusal = refused_at(&refusals, path);
        assert!(refusal.message.contains(name), "{refusal:?}");
    }
    assert_eq!(refusals.len(), 3, "{refusals:#?}");

    // No component reached over a network serves them.
    let unserved = shop_with_components(&[("    reached_by: network\n", "")]);
    let refusals = refusals_of(&text, &unserved);
    for path in [
        "pages/p/sections/map/reads",
        "pages/p/sections/map/commands",
        "pages/editor/commands",
    ] {
        let refusal = refused_at(&refusals, path);
        assert!(
            refusal.message.contains("reached_by: network"),
            "{refusal:?}"
        );
    }

    // An `ess-http/1` binding that routes the section slot's view and command, and not the page
    // slot's command.
    let ir = compile_sources(&sources, Path::new("shop")).unwrap_or_else(|error| panic!("{error}"));
    let declared = format!(
        "type: ess-http/1\nspecification:\n  system: shop\n  version: v1\n  source_digest: \
         sha256:{}\nroutes:\n  - {{view: shop.stock.Items, method: get, path: /items}}\n  \
         - {{command: shop.stock.AddItem, method: post, path: /items}}\n",
        ir.source_digest()
    );
    let spec = ess_gen::ess_http::HttpSpec::from_yaml(&declared).expect("parses");
    let compiled = ess_gen::ess_http::compile(&spec, &ir, &http::derived(&ir))
        .unwrap_or_else(|refused| panic!("compiles: {refused}"));
    let routed = http::Binding::new(compiled).expect("every route has a path");
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let over = |path: &'static str| ess_ui_check::HttpBinding {
        path: Path::new(path),
        binding: &routed,
    };
    let Err(BindingError::Refused(refusals)) =
        ess_ui_check::binding_over(&document, &sources, Some(over("shop-http.yaml")))
    else {
        panic!("the page slot's unrouted command is refused");
    };
    let refusal = refused_at(&refusals, "pages/editor/commands");
    assert!(
        refusal.message.contains("shop.audit.RecordEntry")
            && refusal.message.contains("shop-http.yaml"),
        "{refusal:?}"
    );
    assert_eq!(refusals.len(), 1, "{refusals:#?}");
    let routed_only = slotted("stock.Items", "stock.AddItem", "stock.AddItem");
    let document = ess_ui::load_str(&routed_only).unwrap_or_else(|error| panic!("{error}"));
    let binding = ess_ui_check::binding_over(&document, &sources, Some(over("shop-http.yaml")))
        .unwrap_or_else(|error| panic!("the routed slots bind: {error}"));
    assert_eq!(
        binding.components["shop-service"].views["shop.stock.Items"].path,
        "/items"
    );
}

fn bound_with_model(text: &str, sources: &[(String, String)]) -> Binding {
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    binding(&document, sources).unwrap_or_else(|error| panic!("{error}"))
}

/// The answers an `ess-http/1` binding declares reach the route table (beyond10x/ess#723): a view
/// answered as an array under `items`, a command's outcome as `204` with no body; and a command
/// whose outcome is a redirect is refused, since a generated UI follows none.
#[test]
fn a_bindings_declared_answers_reach_the_route_table() {
    use ess_ui::binding::DeclaredAnswer;
    let sources = shop_with_components(&[]);
    let ir = compile_sources(&sources, Path::new("shop")).unwrap_or_else(|error| panic!("{error}"));
    let routed = |added: &str| {
        let declared = format!(
            "type: ess-http/1\nspecification:\n  system: shop\n  version: v1\n  source_digest: \
             sha256:{}\nroutes:\n  - {{view: shop.stock.Items, method: get, path: /items, \
             body: {{items: array}}}}\n  - command: shop.stock.AddItem\n    method: post\n    \
             path: /items\n    outcomes:\n      added: {added}\n",
            ir.source_digest()
        );
        let spec = ess_gen::ess_http::HttpSpec::from_yaml(&declared).expect("parses");
        let compiled = ess_gen::ess_http::compile(&spec, &ir, &http::derived(&ir))
            .unwrap_or_else(|refused| panic!("compiles: {refused}"));
        http::Binding::new(compiled).expect("every route has a path")
    };
    let text = document(
        "shop",
        "fat",
        "{kind: detail_page, title: Shop, sections: [{name: summary, reads: stock.Items}, \
          {name: add, component: form, does: stock.AddItem, fields: [label]}]}",
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));

    let plain = routed("{status: 204, body: none}");
    let over = ess_ui_check::HttpBinding {
        path: Path::new("shop-http.yaml"),
        binding: &plain,
    };
    let bound = ess_ui_check::binding_over(&document, &sources, Some(over))
        .unwrap_or_else(|error| panic!("the declared answers bind: {error}"));
    let served = &bound.components["shop-service"];
    let items = &served.views["shop.stock.Items"];
    assert_eq!(
        (
            items.answer.as_deref(),
            items.member.as_deref(),
            items.empty
        ),
        (Some("array"), Some("items"), None)
    );
    assert_eq!(
        served.commands["shop.stock.AddItem"].answers,
        [DeclaredAnswer {
            status: 204,
            outcome: "added".to_owned(),
            body: "none".to_owned(),
            member: None,
            error: None,
            text: false,
        }]
    );
    assert!(bound.routed(), "declared answers make a routed binding");

    let redirected =
        routed("{status: 303, body: none, redirect: {location: input.label, mode: query}}");
    let over = ess_ui_check::HttpBinding {
        path: Path::new("shop-http.yaml"),
        binding: &redirected,
    };
    let Err(BindingError::Refused(refusals)) =
        ess_ui_check::binding_over(&document, &sources, Some(over))
    else {
        panic!("a redirect answer is refused");
    };
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.message.contains("redirect")
                && refusal.message.contains("`added`")
                && refusal.message.contains("shop-http.yaml")),
        "{refusals:#?}"
    );
}

/// Every answer form `ess specify http compile` accepts is either carried into the route table or
/// refused naming the route, the form and the binding's file (beyond10x/ess#723): a view's `rows`,
/// `array` and `single`, bare or under a member, and each outcome body (`none`, `response`,
/// `response.<field>`, each under a member, `{<member>: error}`, `encoding: text`) and a redirect.
/// The cases are listed by an exhaustive match on the compiled forms, so a form `ess-http/1` adds
/// fails to compile here until a case shows what a generated UI does with it.
#[test]
#[allow(clippy::too_many_lines)] // one table of every form
fn every_compiled_answer_form_is_carried_or_refused() {
    use ess_gen::ess_http::{Body, Operation, ResponseEncoding, ViewBody};
    // The case that witnesses each compiled form: no wildcard arm.
    fn view_case(form: ViewBody, member: bool) -> &'static str {
        match (form, member) {
            (ViewBody::Rows, false) => "rows",
            // `ess specify http compile` refuses `{<member>: rows}` (ESS-HTTP-024).
            (ViewBody::Rows, true) => "never compiled",
            (ViewBody::Array, false) => "array",
            (ViewBody::Array, true) => "{items: array}",
            (ViewBody::Single, false) => "single",
            (ViewBody::Single, true) => "{items: single}",
        }
    }
    // The view a case routes: `single` answers one row, so it routes the view by identity.
    fn viewed(view: &str) -> &'static str {
        if view.contains("single") {
            "ItemById"
        } else {
            "Items"
        }
    }
    fn outcome_case(body: &Body, encoding: ResponseEncoding) -> &'static str {
        match (body, encoding) {
            (Body::Envelope, _) => "{status: 200, body: envelope}",
            (Body::None, _) => "{status: 204, body: none}",
            (Body::Response, _) => "{status: 200, body: response}",
            (Body::ResponseField(_), ResponseEncoding::Json) => {
                "{status: 200, body: response.label}"
            }
            (Body::ResponseField(_), ResponseEncoding::Text) => {
                "{status: 200, body: response.label, encoding: text}"
            }
            (Body::Wrapped { field: None, .. }, _) => "{status: 200, body: {data: response}}",
            (Body::Wrapped { field: Some(_), .. }, _) => {
                "{status: 200, body: {data: response.label}}"
            }
            (Body::Error { .. }, _) => "{status: 409, body: {error: error}}",
        }
    }
    let mut sources = shop_with_components(&[]);
    let stock = &mut sources
        .iter_mut()
        .find(|(name, _)| name == "domains/stock.yaml")
        .expect("the stock domain")
        .1;
    stock.push_str(
        "  - name: shop.stock.ItemById\n    source: shop.stock.Item\n    consistency: read_your_writes\n    \
         params:\n      - name: id\n        type: shop.stock.ItemId\n    filter: item_id == param.id\n    \
         fields:\n      - name: item_id\n        type: shop.stock.ItemId\n      - name: label\n        type: String\n",
    );
    *stock = stock
        .replace(
            "actors:\n",
            "errors:\n  - name: shop.stock.Full\n    naming: {wire: full}\n\nactors:\n",
        )
        .replace(
            "        type: String\n    outcomes:\n      - name: added\n",
            "        type: String\n    response:\n      - {name: item_id, type: shop.stock.ItemId}\n      \
             - {name: label, type: String}\n    outcomes:\n      - name: full\n        \
             when: label == \"\"\n        error: shop.stock.Full\n      - name: added\n        \
             returns: true\n",
        );
    // A typed response and a direct return need `ess/17`.
    let system = &mut sources
        .iter_mut()
        .find(|(name, _)| name == "system.yaml")
        .expect("the system")
        .1;
    *system = system.replace("format: ess/1\n", "format: ess/17\n");
    for (file, field, mapped) in [
        (
            "domains/stock.yaml",
            "            label: input.label\n",
            "item_id",
        ),
        (
            "domains/audit.yaml",
            "            note: input.note\n",
            "entry_id",
        ),
    ] {
        let source = &mut sources
            .iter_mut()
            .find(|(name, _)| name == file)
            .expect("the domain")
            .1;
        *source = source.replace(
            field,
            &format!("{field}            {mapped}: {{generated: true}}\n"),
        );
    }
    let ir = compile_sources(&sources, Path::new("shop")).unwrap_or_else(|error| panic!("{error}"));
    let text = document(
        "shop",
        "fat",
        "{kind: detail_page, title: Shop, sections: [{name: summary, reads: stock.Items}, \
          {name: one, component: record, reads: {view: stock.ItemById, params: {id: params.id}}, fields: [label]}, \
          {name: add, component: form, does: stock.AddItem, fields: [label]}]}",
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    // What the route table holds for `view` and `answer` (the outcome `outcome` answers), or the
    // refusal's messages.
    let bind = |view: &str, outcome: &str, answer: &str| {
        // The case's form on the view it routes; the other view answers `rows`.
        let (items, one) = if view.contains("single") {
            ("rows".to_owned(), format!("{view}, empty: {{status: 404}}"))
        } else {
            (view.to_owned(), "rows".to_owned())
        };
        let declared = format!(
            "type: ess-http/1\nspecification:\n  system: shop\n  version: v1\n  source_digest: \
             sha256:{}\nroutes:\n  - {{view: shop.stock.Items, method: get, path: /items, body: {items}}}\n  \
             - {{view: shop.stock.ItemById, method: get, path: /item, body: {one}}}\n  \
             - command: shop.stock.AddItem\n    method: post\n    path: /items\n    outcomes:\n      \
             {outcome}: {answer}\n",
            ir.source_digest(),
        );
        let spec = ess_gen::ess_http::HttpSpec::from_yaml(&declared)
            .unwrap_or_else(|error| panic!("{error}\n{declared}"));
        let compiled =
            ess_gen::ess_http::compile(&spec, &ir, &http::derived(&ir)).unwrap_or_else(|refused| {
                panic!("ess specify http compile accepts it: {refused}\n{declared}")
            });
        let route = compiled
            .routes()
            .iter()
            .find(|route| {
                matches!(&route.operation, Operation::View(name) if *name == format!("shop.stock.{}", viewed(view)))
            })
            .expect("the view route");
        let listed = compiled
            .routes()
            .iter()
            .find_map(|route| route.answer(outcome, outcome == "full"))
            .expect("the outcome's answer");
        // The case this form is written as is the one the exhaustive match names.
        assert_eq!(
            view_case(route.body.form, route.body.member.is_some()),
            view
        );
        // A redirect is an outcome's `redirect`, beside its body: refused below, whatever the body.
        if listed.redirect.is_none() {
            assert_eq!(outcome_case(&listed.body, listed.encoding), answer);
        }
        let binding = http::Binding::new(compiled).expect("every route has a path");
        let over = ess_ui_check::HttpBinding {
            path: Path::new("shop-http.yaml"),
            binding: &binding,
        };
        ess_ui_check::binding_over(&document, &sources, Some(over)).map_err(|error| match error {
            BindingError::Refused(refusals) => refusals
                .iter()
                .map(|refusal| refusal.message.clone())
                .collect::<Vec<_>>(),
            other @ BindingError::Model(_) => panic!("{other}"),
        })
    };
    for (view, carried) in [
        ("rows", (None, None)),
        ("array", (Some("array"), None)),
        ("{items: array}", (Some("array"), Some("items"))),
        ("single", (Some("single"), None)),
        ("{items: single}", (Some("single"), Some("items"))),
    ] {
        let bound = bind(view, "added", "{status: 204, body: none}")
            .unwrap_or_else(|refusals| panic!("{view}: {refusals:#?}"));
        let items = &bound.components["shop-service"].views
            [format!("shop.stock.{}", viewed(view)).as_str()];
        assert_eq!(
            (items.answer.as_deref(), items.member.as_deref()),
            carried,
            "{view}"
        );
        assert_eq!(
            items.empty,
            view.contains("single").then_some(404),
            "{view}"
        );
    }
    for (outcome, answer, carried) in [
        ("added", "{status: 200, body: envelope}", None),
        (
            "added",
            "{status: 204, body: none}",
            Some(("none", None, false)),
        ),
        (
            "added",
            "{status: 200, body: response}",
            Some(("response", None, false)),
        ),
        (
            "added",
            "{status: 200, body: response.label}",
            Some(("response.label", None, false)),
        ),
        (
            "added",
            "{status: 200, body: response.label, encoding: text}",
            Some(("response.label", None, true)),
        ),
        (
            "added",
            "{status: 200, body: {data: response}}",
            Some(("response", Some("data"), false)),
        ),
        (
            "added",
            "{status: 200, body: {data: response.label}}",
            Some(("response.label", Some("data"), false)),
        ),
        (
            "full",
            "{status: 409, body: {error: error}}",
            Some(("error", Some("error"), false)),
        ),
    ] {
        let bound = bind("rows", outcome, answer)
            .unwrap_or_else(|refusals| panic!("{answer}: {refusals:#?}"));
        let answers = &bound.components["shop-service"].commands["shop.stock.AddItem"].answers;
        let read: Vec<(&str, Option<&str>, bool)> = answers
            .iter()
            .filter(|declared| declared.outcome == outcome)
            .map(|declared| {
                (
                    declared.body.as_str(),
                    declared.member.as_deref(),
                    declared.text,
                )
            })
            .collect();
        assert_eq!(read, carried.into_iter().collect::<Vec<_>>(), "{answer}");
    }
    let refusals = bind(
        "rows",
        "added",
        "{status: 303, body: none, redirect: {location: input.label, mode: query}}",
    )
    .expect_err("a redirect is refused");
    assert!(
        refusals
            .iter()
            .any(|message| message.contains("shop.stock.AddItem")
                && message.contains("redirect")
                && message.contains("`added`")
                && message.contains("shop-http.yaml")),
        "{refusals:#?}"
    );
}

/// A binding whose routes carry `request.members` or `select` is refused, naming the binding's
/// file, the route and the key, until a generated UI reads them: it would send a flat body, and
/// none of the members a shared route selects by.
#[test]
fn a_binding_with_members_or_select_is_refused_by_name() {
    let sources = shop_with_components(&[]);
    let ir = compile_sources(&sources, Path::new("shop")).unwrap_or_else(|error| panic!("{error}"));
    let text = document(
        "shop",
        "fat",
        "{kind: detail_page, title: Shop, sections: [{name: add, component: form, \
          does: stock.AddItem, fields: [label]}]}",
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    for (routes, key, route) in [
        (
            "  - {command: shop.stock.AddItem, method: post, path: /items, request: {members: \
             {label: data.label}}}\n",
            "`request.members`",
            "`post /items`",
        ),
        (
            "  - {command: shop.stock.AddItem, method: post, path: /entries, select: {kind: \
             item}}\n  - {command: shop.audit.RecordEntry, method: post, path: /entries, \
             select: {kind: entry}}\n",
            "`select`",
            "`post /entries`",
        ),
    ] {
        let declared = format!(
            "type: ess-http/1\nspecification:\n  system: shop\n  version: v1\n  source_digest: \
             sha256:{}\nroutes:\n{routes}",
            ir.source_digest()
        );
        let spec = ess_gen::ess_http::HttpSpec::from_yaml(&declared).expect("parses");
        let compiled = ess_gen::ess_http::compile(&spec, &ir, &http::derived(&ir))
            .unwrap_or_else(|refused| panic!("compiles: {refused}"));
        let routed = http::Binding::new(compiled).expect("every route has a path");
        let over = ess_ui_check::HttpBinding {
            path: Path::new("shop-http.yaml"),
            binding: &routed,
        };
        let Err(BindingError::Refused(refusals)) =
            ess_ui_check::binding_over(&document, &sources, Some(over))
        else {
            panic!("a UI was bound to a binding whose {key} it does not read");
        };
        assert!(
            refusals
                .iter()
                .any(|refusal| refusal.path == "shop-http.yaml"
                    && refusal.message.contains("command `shop.stock.AddItem`")
                    && refusal.message.contains(route)
                    && refusal.message.contains(key)
                    && refusal
                        .message
                        .contains("generated UI does not read it yet")),
            "{refusals:#?}"
        );
    }
}

/// A binding whose routes carry `lists` or `cors`, or which declares `cors` itself, is refused,
/// naming the binding's file, the route and the key, until a generated UI reads them: it would
/// repeat a list's key where the route joins it with commas, and a browser enforces what `cors`
/// states only against a server that answers it. A route's `cors: none` is refused too, though a
/// UI served from the API's own origin honours it by doing nothing.
#[test]
fn a_binding_with_lists_or_cors_is_refused_by_name() {
    let mut sources = shop_with_components(&[]);
    let stock = sources
        .iter_mut()
        .find(|(label, _)| label == "domains/stock.yaml")
        .expect("the stock domain");
    let view = "  - name: shop.stock.Items\n    source: shop.stock.Item\n";
    assert!(stock.1.contains(view), "the fixture's view");
    stock.1 = stock.1.replace(
        view,
        &format!(
            "{view}    params:\n      - {{name: labels, type: List<String>}}\n    filter: \
             {{exists: {{in: param.labels, as: l, that: label == l}}}}\n"
        ),
    );
    let ir = compile_sources(&sources, Path::new("shop")).unwrap_or_else(|error| panic!("{error}"));
    let text = document(
        "shop",
        "fat",
        "{kind: detail_page, title: Shop, sections: [{name: add, component: form, \
          does: stock.AddItem, fields: [label]}]}",
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    for (keys, operation, key, route) in [
        (
            "routes:\n  - {view: shop.stock.Items, method: get, path: /items, lists: {labels: \
             comma}}\n",
            "view `shop.stock.Items`",
            "`lists`",
            "`get /items`",
        ),
        (
            "routes:\n  - {command: shop.stock.AddItem, method: post, path: /items, cors: none}\n",
            "command `shop.stock.AddItem`",
            "`cors`",
            "`post /items`",
        ),
        (
            "cors: {origins: [https://app.example]}\nroutes:\n  - {command: shop.stock.AddItem, \
             method: post, path: /items}\n",
            "command `shop.stock.AddItem`",
            "the binding's `cors`",
            "`post /items`",
        ),
    ] {
        let declared = format!(
            "type: ess-http/1\nspecification:\n  system: shop\n  version: v1\n  source_digest: \
             sha256:{}\n{keys}",
            ir.source_digest()
        );
        let spec = ess_gen::ess_http::HttpSpec::from_yaml(&declared).expect("parses");
        let compiled = ess_gen::ess_http::compile(&spec, &ir, &http::derived(&ir))
            .unwrap_or_else(|refused| panic!("compiles: {refused}"));
        let routed = http::Binding::new(compiled).expect("every route has a path");
        let over = ess_ui_check::HttpBinding {
            path: Path::new("shop-http.yaml"),
            binding: &routed,
        };
        let Err(BindingError::Refused(refusals)) =
            ess_ui_check::binding_over(&document, &sources, Some(over))
        else {
            panic!("a UI was bound to a binding whose {key} it does not read");
        };
        assert!(
            refusals
                .iter()
                .any(|refusal| refusal.path == "shop-http.yaml"
                    && refusal.message.contains(operation)
                    && refusal.message.contains(route)
                    && refusal.message.contains(key)
                    && refusal
                        .message
                        .contains("generated UI does not read it yet")),
            "{refusals:#?}"
        );
    }
}
