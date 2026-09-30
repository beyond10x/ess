//! The partner-portal example generates a React project that type-checks offline, renders every
//! node with its canonical `data-ui-path`, honours each state's placement and plays channel
//! fixtures; generation is deterministic, and a document without a construct gets no code for it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_ui::{Document, NodeRef};
use ess_ui_react::GeneratedFiles;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn example_file() -> PathBuf {
    root().join("examples/partner-portal/ui.yaml")
}

fn example() -> Document {
    ess_ui::load_path(&example_file()).unwrap_or_else(|error| panic!("{error}"))
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

fn generate_example(name: &str) -> (PathBuf, GeneratedFiles) {
    let out = scratch(name);
    let files = ess_ui_react::generate_path(&example_file(), &out)
        .unwrap_or_else(|error| panic!("{error}"));
    (out, files)
}

fn file<'a>(files: &'a GeneratedFiles, path: &str) -> &'a str {
    files
        .files
        .get(path)
        .unwrap_or_else(|| panic!("`{path}` is generated; have {:?}", files.files.keys()))
}

/// Every value the generated sources assign to `data-ui-path`: as a JSX attribute
/// (`data-ui-path="…"`) or as the property of a node spec a runtime component spreads onto its
/// element (`"data-ui-path": "…"`).
fn ui_paths(files: &GeneratedFiles) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for text in files.files.values() {
        for marker in ["data-ui-path=\"", "\"data-ui-path\": \""] {
            let mut rest = text.as_str();
            while let Some(start) = rest.find(marker) {
                rest = &rest[start + marker.len()..];
                let end = rest.find('"').expect("the value closes");
                found.insert(rest[..end].to_owned());
                rest = &rest[end..];
            }
        }
    }
    found
}

/// The nodes a user sees: everything but definitions (widget bodies, page kinds, states, guards
/// and channels), which render only through their instances or not at all.
fn rendered_paths(document: &Document) -> BTreeSet<String> {
    document
        .nodes()
        .into_iter()
        .filter(|located| !located.path.segments()[0].eq("widgets"))
        .filter(|located| {
            matches!(
                located.node,
                NodeRef::Shell(_)
                    | NodeRef::Page(_)
                    | NodeRef::Section(_)
                    | NodeRef::Header(_)
                    | NodeRef::Overlay(_)
                    | NodeRef::Node(_)
                    | NodeRef::Field(_)
                    | NodeRef::Action(_)
                    | NodeRef::Tab(_)
                    | NodeRef::FormGroup(_)
                    | NodeRef::LayoutColumn(_)
                    | NodeRef::Region(_)
                    | NodeRef::NavSection(_)
            )
        })
        .map(|located| located.path.to_string())
        .collect()
}

fn tsc(project: &Path) {
    let output = Command::new("tsc")
        .args(["-p", "tsconfig.offline.json", "--noEmit"])
        .current_dir(project)
        .output()
        .expect("tsc is on PATH");
    assert!(
        output.status.success(),
        "tsc --noEmit failed in {}:\n{}{}",
        project.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn every_page_and_section_renders_with_its_canonical_path() {
    let document = example();
    let (_, files) = generate_example("paths");
    let found = ui_paths(&files);
    for (name, page) in &document.pages {
        let at = format!("pages/{name}");
        assert!(found.contains(&at), "no data-ui-path for {at}");
        for section in &page.sections {
            let at = format!("pages/{name}/sections/{}", section.name);
            assert!(found.contains(&at), "no data-ui-path for {at}");
        }
    }
    let expected = rendered_paths(&document);
    let missing: Vec<_> = expected.difference(&found).collect();
    assert!(
        missing.is_empty(),
        "rendered nodes without data-ui-path: {missing:#?}"
    );
    let invented: Vec<_> = found.difference(&expected).collect();
    assert!(
        invented.is_empty(),
        "data-ui-path values that name no node: {invented:#?}"
    );
}

#[test]
fn the_generated_project_type_checks_offline() {
    let (out, files) = generate_example("typecheck");
    for path in [
        "package.json",
        "index.html",
        "vite.config.ts",
        "tsconfig.json",
        "tsconfig.offline.json",
        "types/react.d.ts",
        "types/react-dom-client.d.ts",
        "types/react-router.d.ts",
        "src/main.tsx",
        "src/App.tsx",
        "README.md",
    ] {
        file(&files, path);
    }
    assert!(file(&files, "README.md").contains("npm install"));
    tsc(&out);
}

#[test]
fn generation_is_deterministic() {
    let (first_dir, first) = generate_example("determinism-a");
    let (second_dir, second) = generate_example("determinism-b");
    assert_eq!(first, second);
    for (path, text) in &first.files {
        let a = std::fs::read(first_dir.join(path)).expect("the first file reads");
        let b = std::fs::read(second_dir.join(path)).expect("the second file reads");
        assert_eq!(a, b, "{path} differs between two runs");
        assert_eq!(
            a,
            text.as_bytes(),
            "{path} on disk is not what was rendered"
        );
    }
}

#[test]
fn routes_come_from_navigation_and_every_page_has_a_component() {
    let document = example();
    let (_, files) = generate_example("routes");
    let app = file(&files, "src/App.tsx");
    let routes = file(&files, "src/routes.ts");
    assert!(app.contains("BrowserRouter"), "{app}");
    for (name, page) in &document.pages {
        let pattern = ess_ui_react::route_pattern(name, page);
        assert!(
            routes.contains(&format!("\"{pattern}\"")),
            "no route {pattern}"
        );
    }
    assert!(routes.contains("\"/partners/detail/:id\""));
    assert!(routes.contains("home: \"overview\""), "{routes}");
    let pages: Vec<_> = files
        .files
        .keys()
        .filter(|path| path.starts_with("src/pages/"))
        .collect();
    assert_eq!(pages.len(), document.pages.len(), "{pages:?}");
    let overview = file(&files, "src/pages/Overview.tsx");
    for section in ["kpis", "pipeline", "feed", "board"] {
        let component = format!("function Overview{}Section(", pascal(section));
        assert!(overview.contains(&component), "no {component}");
    }
}

fn pascal(name: &str) -> String {
    let mut out = String::new();
    for part in name.split(['.', '_', '-']) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

#[test]
fn state_is_placed_where_its_store_says() {
    let (_, files) = generate_example("state");
    let overview = file(&files, "src/pages/Overview.tsx");
    assert!(overview.contains("useUrlState<"), "page_state in the url");
    assert!(overview.contains("useServerState<"), "a server preference");
    // activity.feed is `profile: thin`: its unstored component_state resolves through the page
    // profile to server_session before the document's `component_state: memory` default.
    let activity = file(&files, "src/pages/ActivityFeed.tsx");
    assert!(
        activity
            .contains("useServerSessionState<boolean>(\"portal:pages/activity.feed/state/paused\""),
        "a thin page's component_state on the server session"
    );
    // partners.list has no profile: its selection falls to the document default, memory.
    let partners = file(&files, "src/pages/PartnersList.tsx");
    assert!(
        partners.contains(
            "useMemoryState<Array<M.PartnerId>>(\"portal:pages/partners.list/state/selected\""
        ),
        "selection in memory by the document default"
    );
    let ticket = file(&files, "src/pages/TicketsDetail.tsx");
    assert!(
        ticket.contains("useSessionStorageState<"),
        "a session-storage draft"
    );
    let shell = file(&files, "src/shells/AppShell.tsx");
    assert!(
        shell.contains("useLocalStorageState<"),
        "a local-storage preference"
    );
    assert!(
        shell.contains("useServerSessionState<"),
        "a server-session credential"
    );
    let memory = file(&files, "src/runtime/state/memory.ts");
    assert!(memory.contains("useState"), "{memory}");
    let url = file(&files, "src/runtime/state/url.ts");
    assert!(url.contains("useSearchParams"), "{url}");
}

#[test]
fn reads_default_to_fixtures_behind_a_data_adapter() {
    let (_, files) = generate_example("fixtures");
    let fixtures = file(&files, "src/fixtures.ts");
    for view in ["partners.Page", "tickets.Messages", "forecast.Quarterly"] {
        assert!(
            fixtures.contains(&format!("\"{view}\"")),
            "no fixture for {view}"
        );
    }
    assert!(
        fixtures.contains("\"partners.ById\""),
        "derived views are listed"
    );
    let data = file(&files, "src/runtime/data.ts");
    assert!(data.contains("export interface DataAdapter"), "{data}");
    assert!(data.contains("fixtureAdapter"), "{data}");
    assert!(data.contains("httpAdapter"), "{data}");
}

#[test]
fn channels_get_transport_clients_and_a_fixture_player_by_default() {
    let (_, files) = generate_example("channels");
    let channels = file(&files, "src/channels.ts");
    assert!(
        channels.contains("\"sse\""),
        "server_to_client is server-sent events"
    );
    assert!(
        channels.contains("\"websocket\""),
        "both-way is a WebSocket"
    );
    let live = file(&files, "src/runtime/live.ts");
    assert!(live.contains("EventSource"), "an SSE client");
    assert!(live.contains("WebSocket"), "a WebSocket client");
    assert!(live.contains("fixturePlayer"), "a fixture player");
    assert!(file(&files, "src/fixtures.ts").contains("scripts"));
}

const MINIMAL: &str = r"
format: ess-ui/1
app: things
model: things.system
placement_profile: thin
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: things.detail
  sections: [{name: all, pages: [things.detail]}]
pages:
  things.detail:
    kind: detail_page
    title: Thing
    sections:
      - name: summary
        reads: {view: things.ById}
        fields: [title]
";

#[test]
fn a_document_without_a_construct_gets_no_code_for_it() {
    let document = ess_ui::load_str(MINIMAL).unwrap_or_else(|error| panic!("{error}"));
    let out = scratch("minimal");
    let files =
        ess_ui_react::generate(&document, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    for absent in [
        "src/channels.ts",
        "src/runtime/live.ts",
        "src/runtime/overlays.tsx",
        "src/runtime/composites/collection.tsx",
        "src/runtime/composites/chart.tsx",
        "src/runtime/primitives/button.tsx",
        "src/runtime/state/url.ts",
    ] {
        assert!(!files.files.contains_key(absent), "{absent} is generated");
    }
    file(&files, "src/runtime/composites/record.tsx");
    tsc(&out);
}

#[test]
fn the_command_entry_point_writes_the_project() {
    let out = scratch("command");
    let summary = ess_ui_react::run(&ess_ui_react::ReactArgs {
        path: example_file(),
        out: out.clone(),
    })
    .unwrap_or_else(|error| panic!("{error}"));
    assert!(summary.contains("files written"), "{summary}");
    assert!(out.join("src/App.tsx").is_file());
}

/// A path selects exactly one element: every `data-ui-path` value is assigned once in the
/// generated sources (a section or overlay frame carries its path; the composite body inside it
/// carries none).
#[test]
fn every_data_ui_path_is_assigned_exactly_once() {
    let (_, files) = generate_example("unique-paths");
    let mut counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for text in files.files.values() {
        for marker in ["data-ui-path=\"", "\"data-ui-path\": \""] {
            let mut rest = text.as_str();
            while let Some(start) = rest.find(marker) {
                rest = &rest[start + marker.len()..];
                let end = rest.find('"').expect("the value closes");
                *counts.entry(rest[..end].to_owned()).or_default() += 1;
                rest = &rest[end..];
            }
        }
    }
    let repeated: Vec<_> = counts.iter().filter(|(_, count)| **count != 1).collect();
    assert!(
        repeated.is_empty(),
        "paths assigned more than once: {repeated:#?}"
    );
    assert!(counts.len() > 200, "{} paths", counts.len());

    // The runtime row form: rows of a collection, board or graph editor render
    // `<container>/rows/<row key>`, and a node repeated in every row renders
    // `<container>/rows/<row key>/<rest of its path>` (innermost container first, as
    // `scopePath` in the runtime does). With two rows per container, no runtime path repeats.
    let document = example();
    let containers: BTreeSet<String> = document
        .nodes()
        .into_iter()
        .filter(|located| {
            let body = match located.node {
                NodeRef::Section(section) => &section.body,
                NodeRef::Node(node) => &node.body,
                NodeRef::Overlay(overlay) => &overlay.body,
                _ => return false,
            };
            matches!(
                body,
                ess_ui::Body::Composite(
                    ess_ui::Composite::Collection(_)
                        | ess_ui::Composite::Board(_)
                        | ess_ui::Composite::GraphEditor(_)
                )
            )
        })
        .map(|located| located.path.to_string())
        .collect();
    assert!(containers.len() > 10, "{containers:?}");
    let scope = |path: &str, key: &str| {
        let mut enclosing: Vec<&String> = containers
            .iter()
            .filter(|container| path.starts_with(&format!("{container}/")))
            .collect();
        enclosing.sort_by_key(|container| std::cmp::Reverse(container.len()));
        let mut scoped = path.to_owned();
        for container in enclosing {
            scoped = format!("{container}/rows/{key}/{}", &scoped[container.len() + 1..]);
        }
        scoped
    };
    let mut runtime: Vec<String> = Vec::new();
    for path in counts.keys() {
        let repeated = containers
            .iter()
            .any(|c| path.starts_with(&format!("{c}/")));
        if repeated {
            runtime.extend(["r-1", "r-2"].map(|key| scope(path, key)));
        } else {
            runtime.push(path.clone());
        }
    }
    for container in &containers {
        runtime.extend(["r-1", "r-2"].map(|key| format!("{}/rows/{key}", scope(container, "r-1"))));
    }
    let mut seen = BTreeSet::new();
    let twice: Vec<&String> = runtime.iter().filter(|path| !seen.insert(*path)).collect();
    assert!(twice.is_empty(), "runtime paths rendered twice: {twice:#?}");
}

fn refused(state: &str) -> String {
    let text = format!(
        r"
format: ess-ui/1
app: vault
model: vault.system
placement_profile: fat
placement_defaults: {{page_state: url}}
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: secrets.edit
  sections: [{{name: all, pages: [secrets.edit]}}]
pages:
  secrets.edit:
    kind: detail_page
    title: Secret
    state:
      held: {state}
    sections:
      - name: summary
        reads: {{view: secrets.ById}}
        fields: [title]
"
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::render(&document, &root())
        .map(|_| String::new())
        .expect_err("the placement is refused")
        .to_string()
}

/// `PlacementProfile.resolution.refusals`: a sensitive state never reaches the URL or browser
/// storage, a credential lives only in memory or the server session, and a fallback is held to
/// the same rules. Each refusal names the state's path.
#[test]
fn placement_refusals_name_the_state() {
    for (state, reason) in [
        ("{type: string, class: page_state, sensitive: true}", "sensitive state cannot be placed in url"),
        ("{type: string, class: draft, sensitive: true, store: session_storage}", "sensitive state cannot be placed in session_storage"),
        ("{type: string, class: credential, store: local_storage}", "a credential is held only in memory or server_session"),
        ("{type: string, class: credential, store: server}", "a credential is held only in memory or server_session"),
        (
            "{type: string, class: preference, sensitive: true, store: server, fallback: {when: 'true', store: local_storage}}",
            "fallback: sensitive state cannot be placed in local_storage",
        ),
    ] {
        let error = refused(state);
        assert!(error.starts_with("pages/secrets.edit/state/held: "), "{error}");
        assert!(error.contains(reason), "{state}: {error}");
    }
}

/// Resolution order: explicit store, pinned, section profile, page profile, document defaults,
/// document profile. A section profile wins over its page's; `pinned` skips both profiles.
#[test]
fn a_section_profile_wins_over_the_page_profile_and_pinned_skips_both() {
    let text = r"
format: ess-ui/1
app: order
model: order.system
placement_profile: hybrid
placement_defaults: {component_state: server_session, preference: local_storage}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: things.detail
  sections: [{name: all, pages: [things.detail]}]
pages:
  things.detail:
    kind: detail_page
    title: Thing
    profile: fat
    state:
      open:   {type: boolean, class: component_state}
      pinned: {type: boolean, class: component_state, pinned: true}
    sections:
      - name: summary
        profile: thin
        reads: {view: things.ById}
        fields: [title]
        state:
          wide: {type: boolean, class: preference}
";
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    let files = ess_ui_react::render(&document, &root()).unwrap_or_else(|error| panic!("{error}"));
    let page = file(&files, "src/pages/ThingsDetail.tsx");
    let hook = |local: &str| {
        page.lines()
            .find(|line| line.contains(&format!("const [{local},")))
            .unwrap_or_else(|| panic!("no {local}"))
            .to_owned()
    };
    assert!(
        hook("openValue").contains("useMemoryState<"),
        "fat page: component_state → memory"
    );
    assert!(
        hook("pinnedValue").contains("useServerSessionState<"),
        "pinned: document default"
    );
    assert!(
        hook("wideValue").contains("useServerState<"),
        "thin section: preference → server"
    );
}
