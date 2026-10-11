//! `ess-http/1` compiles against the specification it names, or says every reason it does not
//! (`docs/design/http-presentation-binding.md`, stage 1).

use std::fmt::Write as _;

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, DerivedRoute, HttpIr, HttpSpec, Method, Operation};

const MODEL: &str = include_str!("fixtures/catalog.yaml");

fn model_from(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("well formed");
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("catalog.yaml", text);
    compile_model(&spec, &sources).expect("compiles")
}

fn model() -> EssIr {
    model_from(MODEL)
}

/// The routes ESS derives for `ir`, written out the way `ess_gen::http::routes` derives them for
/// these fixtures: `post /{domain}/commands/{command}` for every accepted command, and
/// `get /{domain}/views/{view}` for every view of a `reached_by: network` component. `ess-http`
/// takes this table as an input and never depends on `ess-gen`; the `ess-cli` tests read the real
/// derivation.
fn derived(ir: &EssIr) -> Vec<DerivedRoute> {
    let mut table = Vec::new();
    for component in ir.components().values() {
        for handle in &component.accepts {
            table.push(DerivedRoute {
                component: component.name.to_string(),
                operation: Operation::Command(handle.name().to_string()),
                method: Method::Post,
                path: format!("/items/commands/{}", handle.name().local()),
            });
        }
        if component.reached_by == ess_domain::component::Reach::Network {
            for domain in &component.owns {
                for handle in &ir.domain(domain).views {
                    table.push(DerivedRoute {
                        component: component.name.to_string(),
                        operation: Operation::View(handle.name().to_string()),
                        method: Method::Get,
                        path: format!("/items/views/{}", handle.name().local()),
                    });
                }
            }
        }
    }
    table.sort_by(|left, right| {
        (&left.component, &left.path, left.method, &left.operation).cmp(&(
            &right.component,
            &right.path,
            right.method,
            &right.operation,
        ))
    });
    table
}

/// The fixture with one more accepted command, `catalog.items.TagItem`, whose input holds a field
/// of every type a path segment can and cannot carry.
fn typed_model() -> EssIr {
    let text = MODEL
        .replace(
            "  - {name: catalog.items.ItemId, kind: newtype, of: String}\n",
            "  - {name: catalog.items.ItemId, kind: newtype, of: String}
  - {name: catalog.items.Shade, kind: enum, variants: [red, blue]}
  - {name: catalog.items.Count, kind: newtype, of: Integer}
  - {name: catalog.items.ShadeName, kind: newtype, of: catalog.items.Shade}
  - {name: catalog.items.Code, kind: newtype, of: String, alphabet: abc}
  - {name: catalog.items.Amount, kind: newtype, of: Decimal}
  - name: catalog.items.Pair
    kind: struct
    fields:
      - {name: left, type: String}
",
        )
        .replace(
            "  - name: catalog.items.PurgeItems\n",
            "  - name: catalog.items.TagItem
    input:
      - {name: item_id, type: catalog.items.ItemId}
      - {name: label, type: String}
      - {name: count, type: Integer}
      - {name: shade, type: catalog.items.Shade}
      - {name: wrapped, type: catalog.items.Count}
      - {name: shade_name, type: catalog.items.ShadeName}
      - {name: note, type: Optional<String>}
      - {name: tags, type: List<String>}
      - {name: flag, type: Boolean}
      - {name: price, type: Decimal}
      - {name: amount, type: catalog.items.Amount}
      - {name: code, type: catalog.items.Code}
      - {name: pair, type: catalog.items.Pair}
      - {name: data, type: Json}
    outcomes:
      - name: tagged
        emits: [catalog.items.ItemTouched]
        payload:
          catalog.items.ItemTouched: {item_id: input.item_id}
  - name: catalog.items.PurgeItems
",
        )
        .replace(
            "accepts: {commands: [catalog.items.ListItem, catalog.items.TouchItem]}",
            "accepts: {commands: [catalog.items.ListItem, catalog.items.TagItem, catalog.items.TouchItem]}",
        );
    model_from(&text)
}

fn messages(ir: &EssIr, routes: &str) -> Vec<String> {
    let spec = HttpSpec::from_yaml(&(header(ir) + routes)).expect("parses");
    compile(&spec, ir, &derived(ir))
        .expect_err("refused")
        .0
        .into_iter()
        .map(|diagnostic| diagnostic.message)
        .collect()
}

fn header(ir: &EssIr) -> String {
    format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n",
        ir.source_digest()
    )
}

/// One route of each admitted shape: a fixed view path, an open command route, and a command on
/// `get` whose only input is mapped to the path.
const ROUTES: &str = "routes:
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters:
      item_id: param.item_id
  - command: catalog.items.ListItem
    method: post
  - command: catalog.items.TouchItem
    method: get
    path: /items/{item_id}/touch
    parameters:
      item_id: input.item_id
";

fn refusals(ir: &EssIr, text: &str) -> Vec<(&'static str, String)> {
    let spec = HttpSpec::from_yaml(text).expect("parses");
    match compile(&spec, ir, &derived(ir)) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .0
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.path.clone()))
            .collect(),
    }
}

fn routes_refusals(ir: &EssIr, routes: &str) -> Vec<(&'static str, String)> {
    refusals(ir, &(header(ir) + routes))
}

fn refused(code: &'static str, path: &str) -> (&'static str, String) {
    (code, path.to_owned())
}

#[test]
fn a_binding_compiles_to_canonical_ir_ordered_by_path_then_method() {
    let ir = model();
    let spec = HttpSpec::from_yaml(&(header(&ir) + ROUTES)).expect("parses");
    let http = compile(&spec, &ir, &derived(&ir)).expect("compiles");
    let order: Vec<(Option<&str>, Method)> = http
        .routes()
        .iter()
        .map(|route| (route.path.as_deref(), route.method))
        .collect();
    assert_eq!(
        order,
        [
            (None, Method::Post),
            (Some("/items/{item_id}"), Method::Get),
            (Some("/items/{item_id}/touch"), Method::Get),
        ]
    );
    let json = http.to_canonical_json();
    let expected = format!(
        "{{\n  \"format\": \"ess-http-ir/1\",\n  \"specification\": {{\n    \"system\": \"catalog\",\n    \"version\": \"v1\",\n    \"source_digest\": \"sha256:{}\"\n  }},\n  \"routes\": [\n    {{\n      \"command\": \"catalog.items.ListItem\",\n      \"method\": \"post\"\n    }},\n    {{\n      \"view\": \"catalog.items.ItemById\",\n      \"method\": \"get\",\n      \"path\": \"/items/{{item_id}}\",\n      \"parameters\": {{\n        \"item_id\": \"param.item_id\"\n      }}\n    }},\n    {{\n      \"command\": \"catalog.items.TouchItem\",\n      \"method\": \"get\",\n      \"path\": \"/items/{{item_id}}/touch\",\n      \"parameters\": {{\n        \"item_id\": \"input.item_id\"\n      }}\n    }}\n  ]\n}}\n",
        ir.source_digest()
    );
    assert_eq!(json, expected, "ess-http-ir/1 bytes changed");
}

#[test]
fn equal_input_gives_equal_bytes_whatever_the_authored_order() {
    let ir = model();
    let forward = HttpSpec::from_yaml(&(header(&ir) + ROUTES)).expect("parses");
    let reversed_routes = "routes:
  - command: catalog.items.TouchItem
    method: get
    path: /items/{item_id}/touch
    parameters: {item_id: input.item_id}
  - command: catalog.items.ListItem
    method: post
  - {view: catalog.items.ItemById, method: get, path: '/items/{item_id}', parameters: {item_id: param.item_id}}
";
    let reversed = HttpSpec::from_yaml(&(header(&ir) + reversed_routes)).expect("parses");
    let first = compile(&forward, &ir, &derived(&ir))
        .expect("compiles")
        .to_canonical_json();
    assert_eq!(
        first,
        compile(&forward, &ir, &derived(&ir))
            .expect("again")
            .to_canonical_json()
    );
    assert_eq!(
        first,
        compile(&reversed, &ir, &derived(&ir))
            .expect("reordered")
            .to_canonical_json()
    );
    let json = serde_json::to_string(&serde_json::json!({
        "type": "ess-http/1",
        "specification": {
            "system": "catalog",
            "version": "v1",
            "source_digest": format!("sha256:{}", ir.source_digest()),
        },
        "routes": [
            {"command": "catalog.items.ListItem", "method": "post"},
            {"view": "catalog.items.ItemById", "method": "get", "path": "/items/{item_id}",
             "parameters": {"item_id": "param.item_id"}},
            {"command": "catalog.items.TouchItem", "method": "get", "path": "/items/{item_id}/touch",
             "parameters": {"item_id": "input.item_id"}},
        ],
    }))
    .unwrap();
    let from_json = HttpSpec::from_json(&json).expect("parses as JSON");
    assert_eq!(
        first,
        compile(&from_json, &ir, &derived(&ir))
            .expect("from JSON")
            .to_canonical_json()
    );
}

#[test]
fn compiled_ir_reads_back_and_renders_a_route_table() {
    let ir = model();
    let spec = HttpSpec::from_yaml(&(header(&ir) + ROUTES)).expect("parses");
    let json = compile(&spec, &ir, &derived(&ir))
        .expect("compiles")
        .to_canonical_json();
    let read = HttpIr::from_json(&json).expect("the compiled IR reads back");
    assert_eq!(read.to_canonical_json(), json, "round trip changed bytes");
    let mut table = String::from("| operation | method | path |\n|---|---|---|\n");
    for route in read.routes() {
        let operation = match &route.operation {
            Operation::Command(name) => format!("command `{name}`"),
            Operation::View(name) => format!("view `{name}`"),
        };
        let path = route.path.as_deref().unwrap_or("(open)");
        writeln!(
            table,
            "| {operation} | {} | `{path}` |",
            route.method.as_str()
        )
        .unwrap();
    }
    assert_eq!(
        table,
        "| operation | method | path |\n|---|---|---|\n\
         | command `catalog.items.ListItem` | post | `(open)` |\n\
         | view `catalog.items.ItemById` | get | `/items/{item_id}` |\n\
         | command `catalog.items.TouchItem` | get | `/items/{item_id}/touch` |\n"
    );
    let other = json.replace("ess-http-ir/1", "ess-http-ir/2");
    assert!(
        HttpIr::from_json(&other).is_err(),
        "another IR format reads"
    );
    let unknown_route_key = json.replacen(
        "\"method\": \"post\"",
        "\"method\": \"post\",\n      \"status\": 200",
        1,
    );
    assert_ne!(unknown_route_key, json);
    let error = HttpIr::from_json(&unknown_route_key)
        .expect_err("an unknown route key reads")
        .to_string();
    assert!(error.contains("status"), "{error}");
    let unknown_key = json.replacen("\"routes\"", "\"derived\": [],\n  \"routes\"", 1);
    let error = HttpIr::from_json(&unknown_key)
        .expect_err("an unknown key reads")
        .to_string();
    assert!(error.contains("unknown field `derived`"), "{error}");
}

#[test]
fn an_empty_binding_is_valid_and_routes_nothing() {
    let ir = model();
    let http = compile(
        &HttpSpec::from_yaml(&(header(&ir) + "routes: []\n")).expect("parses"),
        &ir,
        &derived(&ir),
    )
    .expect("compiles");
    assert_eq!(http.routes(), &[]);
    // Every operation keeps its derived route, and the IR records each one it was checked against.
    assert_eq!(http.derived_routes(), derived(&ir).as_slice());
}

#[test]
fn ess_http_001_another_type_is_refused() {
    let ir = model();
    let text = header(&ir).replace("ess-http/1", "ess-http/2") + ROUTES;
    assert_eq!(refusals(&ir, &text), [refused("ESS-HTTP-001", "type")]);
}

#[test]
fn ess_http_002_each_disagreeing_specification_key_is_refused() {
    let ir = model();
    let text = header(&ir)
        .replace("system: catalog", "system: other")
        .replace("version: v1", "version: v2")
        .replace("sha256:", "sha256:00")
        + ROUTES;
    assert_eq!(
        refusals(&ir, &text),
        [
            refused("ESS-HTTP-002", "specification.system"),
            refused("ESS-HTTP-002", "specification.version"),
            refused("ESS-HTTP-002", "specification.source_digest"),
        ]
    );
}

#[test]
fn ess_http_004_a_route_names_one_operation_that_exists() {
    let ir = model();
    assert_eq!(
        routes_refusals(&ir, "routes:\n  - {method: post}\n"),
        [refused("ESS-HTTP-004", "routes[0]")]
    );
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {command: catalog.items.ListItem, view: catalog.items.ItemById, method: post}\n"
        ),
        [refused("ESS-HTTP-004", "routes[0]")]
    );
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {command: catalog.items.ListItem, method: post}\n  - {command: catalog.items.Missing, method: post}\n"
        ),
        [refused("ESS-HTTP-004", "routes[1].command")]
    );
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {view: catalog.items.Missing, method: get}\n"
        ),
        [refused("ESS-HTTP-004", "routes[0].view")]
    );
    // A command's name in `view:` names no view.
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {view: catalog.items.ListItem, method: get}\n"
        ),
        [refused("ESS-HTTP-004", "routes[0].view")]
    );
}

#[test]
fn ess_http_005_one_operation_routed_twice_is_refused() {
    let ir = model();
    let found = routes_refusals(
        &ir,
        "routes:
  - {command: catalog.items.ListItem, method: post}
  - {view: catalog.items.ItemById, method: get}
  - {command: catalog.items.ListItem, method: put, path: /items}
",
    );
    assert_eq!(found, [refused("ESS-HTTP-005", "routes[2]")]);
    let spec = HttpSpec::from_yaml(
        &(header(&ir)
            + "routes:\n  - {command: catalog.items.ListItem, method: post}\n  - {command: catalog.items.ListItem, method: put}\n"),
    )
    .unwrap();
    let message = &compile(&spec, &ir, &derived(&ir)).expect_err("refused").0[0].message;
    assert!(
        message.contains("routes[0]") && message.contains("catalog.items.ListItem"),
        "{message}"
    );
}

#[test]
fn ess_http_006_a_method_outside_the_closed_set_is_refused() {
    let ir = model();
    for method in ["GET", "Post", "connect", "fetch", ""] {
        assert_eq!(
            routes_refusals(
                &ir,
                &format!("routes:\n  - {{command: catalog.items.ListItem, method: '{method}'}}\n")
            ),
            [refused("ESS-HTTP-006", "routes[0].method")],
            "{method:?}"
        );
    }
}

#[test]
fn ess_http_006_a_view_is_bound_to_get_only() {
    let ir = model();
    for method in ["put", "post", "delete", "options", "head", "patch", "trace"] {
        assert_eq!(
            routes_refusals(
                &ir,
                &format!("routes:\n  - {{view: catalog.items.ItemById, method: {method}}}\n")
            ),
            [refused("ESS-HTTP-006", "routes[0].method")],
            "{method}"
        );
    }
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {view: catalog.items.ItemById, method: get}\n"
        ),
        []
    );
}

#[test]
fn ess_http_006_a_command_is_refused_on_head_options_and_trace() {
    let ir = model();
    for method in ["head", "options", "trace"] {
        assert_eq!(
            routes_refusals(
                &ir,
                &format!("routes:\n  - {{command: catalog.items.ListItem, method: {method}}}\n")
            ),
            [refused("ESS-HTTP-006", "routes[0].method")],
            "{method}"
        );
    }
    for method in ["post", "put", "patch", "delete"] {
        assert_eq!(
            routes_refusals(
                &ir,
                &format!("routes:\n  - {{command: catalog.items.ListItem, method: {method}}}\n")
            ),
            [],
            "{method}"
        );
    }
}

#[test]
fn ess_http_013_an_operation_without_an_http_surface_is_refused() {
    let ir = model();
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {command: catalog.items.PurgeItems, method: post}\n"
        ),
        [refused("ESS-HTTP-013", "routes[0].command")]
    );
    let in_process = model_from(&MODEL.replace("    reached_by: network\n", ""));
    assert_eq!(
        routes_refusals(
            &in_process,
            "routes:\n  - {view: catalog.items.ItemById, method: get}\n  - {command: catalog.items.ListItem, method: post}\n"
        ),
        [refused("ESS-HTTP-013", "routes[0].view")]
    );
}

#[test]
fn ess_http_014_a_command_on_get_maps_every_input_field_to_the_path() {
    let ir = model();
    let found = routes_refusals(
        &ir,
        "routes:
  - command: catalog.items.ListItem
    method: get
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
  - command: catalog.items.TouchItem
    method: get
    path: /touch
",
    );
    assert_eq!(
        found,
        [
            refused("ESS-HTTP-014", "routes[0]"),
            refused("ESS-HTTP-014", "routes[1]"),
        ]
    );
    let spec = HttpSpec::from_yaml(
        &(header(&ir)
            + "routes:\n  - {command: catalog.items.ListItem, method: get, path: '/items/{item_id}', parameters: {item_id: input.item_id}}\n"),
    )
    .unwrap();
    let message = &compile(&spec, &ir, &derived(&ir)).expect_err("refused").0[0].message;
    assert!(
        message.contains("catalog.items.ListItem") && message.contains("`title`"),
        "{message}"
    );
}

#[test]
fn ess_http_014_a_mapping_counts_only_when_it_maps_a_variable_of_the_path() {
    let ir = model();
    // The mapping names no variable of `/touch`, so the field it reads is not on the path: that is
    // both a mapping for no variable and an input field `get` cannot carry.
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {command: catalog.items.TouchItem, method: get, path: /touch, parameters: {item_id: input.item_id}}\n"
        ),
        [
            refused("ESS-HTTP-008", "routes[0].parameters.item_id"),
            refused("ESS-HTTP-014", "routes[0]"),
        ]
    );
    // An open route maps nothing to a path either.
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {command: catalog.items.TouchItem, method: get, parameters: {item_id: input.item_id}}\n"
        ),
        [
            refused("ESS-HTTP-008", "routes[0].parameters.item_id"),
            refused("ESS-HTTP-014", "routes[0]"),
        ]
    );
}

#[test]
fn ess_http_007_a_path_that_breaks_the_template_grammar_is_refused_naming_the_segment() {
    let ir = model();
    for (path, named) in [
        ("items", "items"),
        ("/items/", "segment 2"),
        ("//items", "segment 1"),
        ("/items//x", "segment 2"),
        ("/items/./x", "`.`"),
        ("/items/../x", "`..`"),
        ("/items?page=1", "`items?page=1`"),
        ("/items#top", "`items#top`"),
        ("/items/a%20b", "`a%20b`"),
        ("/items/{item_id}.json", "`{item_id}.json`"),
        ("/items/x{item_id}", "`x{item_id}`"),
        ("/items/{}", "`{}`"),
        ("/items/{1st}", "`{1st}`"),
        ("/items/{item-id}", "`{item-id}`"),
        ("/items/{item_id", "`{item_id`"),
        ("/items/{{item_id}}", "`{{item_id}}`"),
        ("/it ems", "`it ems`"),
        ("/caf\u{e9}", "`caf\u{e9}`"),
        ("/items/a\"b", "`a\"b`"),
        ("", "``"),
    ] {
        let routes = format!(
            "routes:\n  - {{command: catalog.items.ListItem, method: post, path: {}}}\n",
            serde_json::to_string(path).unwrap()
        );
        assert_eq!(
            routes_refusals(&ir, &routes),
            [refused("ESS-HTTP-007", "routes[0].path")],
            "{path:?}"
        );
        let found = messages(&ir, &routes);
        assert!(found[0].contains(named), "{path:?}: {}", found[0]);
    }
    // Every reason a path breaks the grammar is reported, one refusal each.
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {command: catalog.items.ListItem, method: post, path: '/a/../{x}.json/'}\n"
        ),
        [
            refused("ESS-HTTP-007", "routes[0].path"),
            refused("ESS-HTTP-007", "routes[0].path"),
            refused("ESS-HTTP-007", "routes[0].path"),
        ]
    );
    for path in [
        "/",
        "/items",
        "/.well-known/oauth-authorization-server",
        "/a-b_c.d~e/!$&'()*+,;=:@/...",
        "/items/{item_id}",
        "/{_x9}/{Item_ID}",
    ] {
        let parameters = match path {
            "/items/{item_id}" => "{item_id: input.item_id}",
            "/{_x9}/{Item_ID}" => "{_x9: input.item_id, Item_ID: input.title}",
            _ => "{}",
        };
        let routes = format!(
            "routes:\n  - {{command: catalog.items.ListItem, method: post, path: {}, parameters: {parameters}}}\n",
            serde_json::to_string(path).unwrap()
        );
        assert_eq!(routes_refusals(&ir, &routes), [], "{path:?}");
    }
}

#[test]
fn ess_http_008_each_variable_is_mapped_exactly_once() {
    let ir = model();
    // A variable not mapped.
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {view: catalog.items.ItemById, method: get, path: '/items/{item_id}'}\n"
        ),
        [refused("ESS-HTTP-008", "routes[0].parameters.item_id")]
    );
    // A mapping for no variable of the path.
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {view: catalog.items.ItemById, method: get, path: /items, parameters: {item_id: param.item_id}}\n"
        ),
        [refused("ESS-HTTP-008", "routes[0].parameters.item_id")]
    );
    // A variable mapped twice: named by two segments, or by two keys of a JSON object.
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {view: catalog.items.ItemById, method: get, path: '/items/{item_id}/{item_id}', parameters: {item_id: param.item_id}}\n"
        ),
        [refused("ESS-HTTP-008", "routes[0].parameters.item_id")]
    );
    let json = format!(
        "{{\"type\": \"ess-http/1\", \"specification\": {{\"system\": \"catalog\", \"version\": \"v1\", \"source_digest\": \"sha256:{}\"}}, \"routes\": [{{\"view\": \"catalog.items.ItemById\", \"method\": \"get\", \"path\": \"/items/{{item_id}}\", \"parameters\": {{\"item_id\": \"param.item_id\", \"item_id\": \"param.item_id\"}}}}]}}",
        ir.source_digest()
    );
    let spec = HttpSpec::from_json(&json).expect("parses");
    let found: Vec<(&str, String)> = compile(&spec, &ir, &derived(&ir))
        .expect_err("a variable mapped twice")
        .0
        .into_iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.path))
        .collect();
    assert_eq!(
        found,
        [refused("ESS-HTTP-008", "routes[0].parameters.item_id")]
    );
    let message = &messages(
        &ir,
        "routes:\n  - {view: catalog.items.ItemById, method: get, path: /items, parameters: {item_id: param.item_id}}\n",
    )[0];
    assert!(
        message.contains("`/items`") && message.contains("`item_id`"),
        "{message}"
    );
    // One source read by two variables, refused at the second variable of the path, whatever
    // order the mappings are written in.
    for parameters in [
        "{a: input.item_id, b: input.item_id}",
        "{b: input.item_id, a: input.item_id}",
    ] {
        let routes = format!(
            "routes:\n  - {{command: catalog.items.TouchItem, method: post, path: '/touch/{{a}}/{{b}}', parameters: {parameters}}}\n"
        );
        assert_eq!(
            routes_refusals(&ir, &routes),
            [refused("ESS-HTTP-008", "routes[0].parameters.b")],
            "{parameters}"
        );
    }
    let message = &messages(
        &ir,
        "routes:\n  - {command: catalog.items.TouchItem, method: post, path: '/touch/{a}/{b}', parameters: {a: input.item_id, b: input.item_id}}\n",
    )[0];
    assert!(
        message.contains("`input.item_id`") && message.contains("`a`"),
        "{message}"
    );
}

#[test]
fn ess_http_009_a_source_reads_one_field_of_the_right_namespace_and_a_segment_type() {
    let ir = typed_model();
    for (operation, source) in [
        // Wrong namespace.
        ("view: catalog.items.ItemById, method: get", "input.item_id"),
        (
            "command: catalog.items.TouchItem, method: post",
            "param.item_id",
        ),
        ("command: catalog.items.TouchItem, method: post", "item_id"),
        (
            "command: catalog.items.TouchItem, method: post",
            "response.item_id",
        ),
        // Naming no field.
        (
            "command: catalog.items.TouchItem, method: post",
            "input.missing",
        ),
        ("command: catalog.items.TouchItem, method: post", "input."),
        (
            "command: catalog.items.TouchItem, method: post",
            "input.item_id.value",
        ),
        ("view: catalog.items.ItemById, method: get", "param.title"),
        // Of a type a segment cannot carry.
        ("command: catalog.items.TagItem, method: post", "input.note"),
        ("command: catalog.items.TagItem, method: post", "input.tags"),
        ("command: catalog.items.TagItem, method: post", "input.flag"),
        (
            "command: catalog.items.TagItem, method: post",
            "input.price",
        ),
        (
            "command: catalog.items.TagItem, method: post",
            "input.amount",
        ),
        ("command: catalog.items.TagItem, method: post", "input.code"),
        ("command: catalog.items.TagItem, method: post", "input.pair"),
        ("command: catalog.items.TagItem, method: post", "input.data"),
    ] {
        let routes = format!(
            "routes:\n  - {{{operation}, path: '/x/{{v}}', parameters: {{v: '{source}'}}}}\n"
        );
        assert_eq!(
            routes_refusals(&ir, &routes),
            [refused("ESS-HTTP-009", "routes[0].parameters.v")],
            "{source}"
        );
        let message = &messages(&ir, &routes)[0];
        assert!(message.contains(&format!("`{source}`")), "{message}");
    }
    let message = &messages(
        &ir,
        "routes:\n  - {command: catalog.items.TagItem, method: post, path: '/x/{v}', parameters: {v: input.note}}\n",
    )[0];
    assert!(message.contains("Optional<String>"), "{message}");
    for field in [
        "item_id",
        "label",
        "count",
        "shade",
        "wrapped",
        "shade_name",
    ] {
        let routes = format!(
            "routes:\n  - {{command: catalog.items.TagItem, method: post, path: '/x/{{v}}', parameters: {{v: input.{field}}}}}\n"
        );
        assert_eq!(routes_refusals(&ir, &routes), [], "{field}");
    }
}

#[test]
fn route_parameters_resolve_to_typed_sources() {
    let ir = model();
    let spec = HttpSpec::from_yaml(&(header(&ir) + ROUTES)).expect("parses");
    let http = compile(&spec, &ir, &derived(&ir)).expect("compiles");
    let view = http
        .route(&Operation::View("catalog.items.ItemById".into()))
        .expect("bound");
    assert_eq!(
        view.parameters.get("item_id"),
        Some(&ess_http::Source::Param("item_id".into()))
    );
    let command = http
        .route(&Operation::Command("catalog.items.TouchItem".into()))
        .expect("bound");
    assert_eq!(
        command.parameters.get("item_id"),
        Some(&ess_http::Source::Input("item_id".into()))
    );
    let json = http.to_canonical_json();
    let broken = json.replacen("\"param.item_id\"", "\"query.item_id\"", 1);
    assert_ne!(broken, json);
    let error = HttpIr::from_json(&broken)
        .expect_err("a source outside both namespaces reads back")
        .to_string();
    assert!(error.contains("query.item_id"), "{error}");
}

#[test]
fn compiled_ir_out_of_canonical_order_is_refused_naming_the_key() {
    let ir = model();
    let spec = HttpSpec::from_yaml(&(header(&ir) + ROUTES)).expect("parses");
    let json = compile(&spec, &ir, &derived(&ir))
        .expect("compiles")
        .to_canonical_json();
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["routes"].as_array_mut().expect("routes").reverse();
    let reversed = serde_json::to_string_pretty(&value).unwrap() + "\n";
    let error = HttpIr::from_json(&reversed)
        .expect_err("routes out of canonical order read back")
        .to_string();
    assert!(error.contains("`routes[1]`"), "{error}");
    let route = "\"item_id\": \"param.item_id\"";
    assert!(json.contains(route), "{json}");
    for (written, follows) in [
        (
            format!("\"zz\": \"param.item_id\",\n        {route}"),
            "`zz`",
        ),
        (format!("{route},\n        {route}"), "`item_id`"),
    ] {
        let shuffled = json.replacen(route, &written, 1);
        let error = HttpIr::from_json(&shuffled)
            .expect_err("parameters out of canonical order read back")
            .to_string();
        assert!(
            error.contains("`parameters`") && error.contains(follows),
            "{error}"
        );
    }
    let spec = HttpSpec::from_yaml(&(header(&ir) + "routes: []\n")).expect("parses");
    let json = compile(&spec, &ir, &derived(&ir))
        .expect("compiles")
        .to_canonical_json();
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["derived_routes"]
        .as_array_mut()
        .expect("derived routes")
        .swap(0, 1);
    let swapped = serde_json::to_string_pretty(&value).unwrap() + "\n";
    let error = HttpIr::from_json(&swapped)
        .expect_err("derived routes out of canonical order read back")
        .to_string();
    assert!(error.contains("`derived_routes[1]`"), "{error}");
}

#[test]
fn ess_http_010_a_declared_route_on_another_operations_derived_route_is_refused() {
    let ir = model();
    let routes = "routes:\n  - {command: catalog.items.TouchItem, method: post, path: /items/commands/ListItem}\n";
    assert_eq!(
        routes_refusals(&ir, routes),
        [refused("ESS-HTTP-010", "routes[0]")]
    );
    let message = &messages(&ir, routes)[0];
    for named in [
        "two operations claim the same method and path",
        "command `catalog.items.TouchItem`",
        "command `catalog.items.ListItem`",
        "items-service",
    ] {
        assert!(message.contains(named), "{named}: {message}");
    }
    // A route on its own operation's derived path replaces that route and claims nothing twice.
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {command: catalog.items.ListItem, method: post, path: /items/commands/ListItem}\n"
        ),
        []
    );
    // The same path on another method is another route.
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {command: catalog.items.TouchItem, method: put, path: /items/commands/ListItem}\n"
        ),
        []
    );
}

#[test]
fn ess_http_010_two_declared_routes_collide_whatever_their_variables_are_called() {
    let ir = model();
    let routes = "routes:
  - {view: catalog.items.ItemById, method: get, path: '/items/{item_id}', parameters: {item_id: param.item_id}}
  - {command: catalog.items.TouchItem, method: get, path: '/items/{id}', parameters: {id: input.item_id}}
";
    assert_eq!(
        routes_refusals(&ir, routes),
        [refused("ESS-HTTP-010", "routes[1]")]
    );
    let message = &messages(&ir, routes)[0];
    for named in [
        "two operations claim the same method and path",
        "view `catalog.items.ItemById`",
        "command `catalog.items.TouchItem`",
        "routes[0]",
    ] {
        assert!(message.contains(named), "{named}: {message}");
    }
}

#[test]
fn ess_http_010_collisions_are_checked_per_component() {
    let ir = model();
    // `ListItem` is served by another component, so its route there is another surface.
    let table = vec![
        DerivedRoute {
            component: "items-service".into(),
            operation: Operation::Command("catalog.items.TouchItem".into()),
            method: Method::Post,
            path: "/items/commands/TouchItem".into(),
        },
        DerivedRoute {
            component: "other-service".into(),
            operation: Operation::Command("catalog.items.ListItem".into()),
            method: Method::Post,
            path: "/x".into(),
        },
    ];
    let spec = HttpSpec::from_yaml(
        &(header(&ir)
            + "routes:\n  - {command: catalog.items.TouchItem, method: post, path: /x}\n"),
    )
    .unwrap();
    compile(&spec, &ir, &table).expect("no component serves both on `/x`");
    let mut shared = table.clone();
    shared.push(DerivedRoute {
        component: "other-service".into(),
        operation: Operation::Command("catalog.items.TouchItem".into()),
        method: Method::Post,
        path: "/other/TouchItem".into(),
    });
    let refused_here = compile(&spec, &ir, &shared).expect_err("other-service serves both");
    assert_eq!(refused_here.0.len(), 1, "{refused_here}");
    assert_eq!(refused_here.0[0].code, "ESS-HTTP-010");
    assert!(
        refused_here.0[0].message.contains("other-service"),
        "{refused_here}"
    );
}

#[test]
fn ess_http_011_a_literal_beside_a_variable_after_an_equal_prefix_is_refused() {
    let ir = model();
    let routes = "routes:
  - {command: catalog.items.ListItem, method: post, path: '/items/{item_id}', parameters: {item_id: input.item_id}}
  - {command: catalog.items.TouchItem, method: post, path: /items/current}
";
    assert_eq!(
        routes_refusals(&ir, routes),
        [refused("ESS-HTTP-011", "routes[1]")]
    );
    let message = &messages(&ir, routes)[0];
    for named in [
        "command `catalog.items.ListItem`",
        "command `catalog.items.TouchItem`",
        "`current`",
        "`{item_id}`",
    ] {
        assert!(message.contains(named), "{named}: {message}");
    }
    // Against a derived route: `{id}` sits where `ListItem`'s derived path has `commands`.
    let routes = "routes:\n  - {command: catalog.items.TouchItem, method: post, path: '/items/{id}/ListItem', parameters: {id: input.item_id}}\n";
    assert_eq!(
        routes_refusals(&ir, routes),
        [refused("ESS-HTTP-011", "routes[0]")]
    );
    let message = &messages(&ir, routes)[0];
    assert!(
        message.contains("`commands`") && message.contains("catalog.items.ListItem"),
        "{message}"
    );
    // No path matches both: they differ in length, or in a later literal.
    for (left, right) in [
        ("/items/{item_id}", "/items/current/x"),
        ("/items/{item_id}/a", "/items/current/b"),
    ] {
        let routes = format!(
            "routes:
  - {{command: catalog.items.ListItem, method: post, path: '{left}', parameters: {{item_id: input.item_id}}}}
  - {{command: catalog.items.TouchItem, method: post, path: '{right}'}}
"
        );
        assert_eq!(routes_refusals(&ir, &routes), [], "{left} {right}");
    }
    // Nor on another method.
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:
  - {view: catalog.items.ItemById, method: get, path: '/items/{item_id}', parameters: {item_id: param.item_id}}
  - {command: catalog.items.TouchItem, method: post, path: /items/current}
"
        ),
        []
    );
}

#[test]
fn the_ir_records_the_derived_routes_it_was_checked_against() {
    let ir = model();
    let spec = HttpSpec::from_yaml(
        &(header(&ir)
            + "routes:\n  - {command: catalog.items.ListItem, method: post, path: /orders}\n"),
    )
    .unwrap();
    let http = compile(&spec, &ir, &derived(&ir)).expect("compiles");
    // `ListItem` is bound, so its derived route is not part of the surface.
    let recorded: Vec<(&str, &str)> = http
        .derived_routes()
        .iter()
        .map(|route| (route.operation.name(), route.path.as_str()))
        .collect();
    assert_eq!(
        recorded,
        [
            ("catalog.items.TouchItem", "/items/commands/TouchItem"),
            ("catalog.items.ItemById", "/items/views/ItemById"),
        ]
    );
    let json = http.to_canonical_json();
    assert!(
        json.ends_with(
            "  \"derived_routes\": [\n    {\n      \"component\": \"items-service\",\n      \"command\": \"catalog.items.TouchItem\",\n      \"method\": \"post\",\n      \"path\": \"/items/commands/TouchItem\"\n    },\n    {\n      \"component\": \"items-service\",\n      \"view\": \"catalog.items.ItemById\",\n      \"method\": \"get\",\n      \"path\": \"/items/views/ItemById\"\n    }\n  ]\n}\n"
        ),
        "{json}"
    );
    assert_eq!(
        HttpIr::from_json(&json)
            .expect("reads back")
            .to_canonical_json(),
        json
    );
    // The order the table arrives in does not reach the bytes.
    let mut reversed = derived(&ir);
    reversed.reverse();
    assert_eq!(
        compile(&spec, &ir, &reversed)
            .expect("compiles")
            .to_canonical_json(),
        json
    );
}

#[test]
fn every_refusal_is_returned_at_once() {
    let ir = model();
    let text = header(&ir).replace("version: v1", "version: v2")
        + "routes:
  - {method: post}
  - {view: catalog.items.ItemById, method: post}
  - {command: catalog.items.PurgeItems, method: post}
  - {command: catalog.items.TouchItem, method: get, path: /touch}
  - {view: catalog.items.ItemById, method: get}
";
    assert_eq!(
        refusals(&ir, &text),
        [
            refused("ESS-HTTP-002", "specification.version"),
            refused("ESS-HTTP-004", "routes[0]"),
            refused("ESS-HTTP-006", "routes[1].method"),
            refused("ESS-HTTP-013", "routes[2].command"),
            refused("ESS-HTTP-014", "routes[3]"),
            refused("ESS-HTTP-005", "routes[4]"),
        ]
    );
}

#[test]
fn a_stage_2_key_is_refused_as_an_unknown_field_naming_it() {
    let ir = model();
    for key in [
        "request: {encoding: form}",
        "response: {}",
        "status: 200",
        "headers: {Cache-Control: {literal: no-store}}",
        "redirect: {location: input.title}",
        "outcomes: {}",
        "body: single",
        "empty: {status: 404}",
    ] {
        let name = key.split(':').next().unwrap();
        let text = header(&ir)
            + &format!(
                "routes:\n  - command: catalog.items.ListItem\n    method: post\n    {key}\n"
            );
        let error = HttpSpec::from_yaml(&text).expect_err(name).to_string();
        assert!(
            error.contains(&format!("unknown field `{name}`")),
            "{name}: {error}"
        );
    }
    let text = header(&ir) + "refusals: {}\n" + ROUTES;
    let error = HttpSpec::from_yaml(&text)
        .expect_err("refusals")
        .to_string();
    assert!(error.contains("unknown field `refusals`"), "{error}");
    let text = header(&ir).replace("version: v1", "version: v1\n  extra: 1") + ROUTES;
    let error = HttpSpec::from_yaml(&text)
        .expect_err("specification")
        .to_string();
    assert!(error.contains("unknown field `extra`"), "{error}");
}

#[test]
fn every_code_this_crate_can_emit_is_one_of_the_stage_1_codes() {
    for code in ess_http::CODES {
        assert!(
            matches!(
                *code,
                "ESS-HTTP-001"
                    | "ESS-HTTP-002"
                    | "ESS-HTTP-004"
                    | "ESS-HTTP-005"
                    | "ESS-HTTP-006"
                    | "ESS-HTTP-007"
                    | "ESS-HTTP-008"
                    | "ESS-HTTP-009"
                    | "ESS-HTTP-010"
                    | "ESS-HTTP-011"
                    | "ESS-HTTP-013"
                    | "ESS-HTTP-014"
            ),
            "{code}"
        );
    }
    let source = include_str!("../src/lib.rs");
    let mut emitted: Vec<&str> = source
        .match_indices("\"ESS-HTTP-")
        .map(|(at, _)| &source[at + 1..at + 13])
        .collect();
    emitted.sort_unstable();
    emitted.dedup();
    assert_eq!(
        emitted,
        ess_http::CODES,
        "a code in the source is not in CODES"
    );
}
