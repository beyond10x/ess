//! Adversary pass 1 on the React generator: state placement against the schema's resolution and
//! refusals, query-key and identifier collisions, live-channel semantics the schema names, typed
//! drafts, and constructs the partner-portal example does not use.

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

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adv1")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

fn load(text: &str) -> Document {
    ess_ui::load_str(text).unwrap_or_else(|error| panic!("the document loads: {error}"))
}

fn generate(name: &str, text: &str) -> (PathBuf, Result<GeneratedFiles, String>) {
    let document = load(text);
    let out = scratch(name);
    let result = ess_ui_react::generate(&document, &root(), &out).map_err(|e| e.to_string());
    (out, result)
}

fn example(name: &str) -> GeneratedFiles {
    ess_ui_react::generate_path(&example_file(), &scratch(name))
        .unwrap_or_else(|error| panic!("{error}"))
}

fn file<'a>(files: &'a GeneratedFiles, path: &str) -> &'a str {
    files
        .files
        .get(path)
        .unwrap_or_else(|| panic!("`{path}` is generated; have {:?}", files.files.keys()))
}

/// The line declaring the hook of the state whose local is `local` (`const [<local>, …`).
fn hook_line<'a>(text: &'a str, local: &str) -> &'a str {
    let marker = format!("const [{local},");
    text.lines()
        .find(|line| line.contains(&marker))
        .unwrap_or_else(|| panic!("no hook for {local} in:\n{text}"))
}

fn tsc(project: &Path) -> Result<(), String> {
    let output = Command::new("tsc")
        .args(["-p", "tsconfig.offline.json", "--noEmit"])
        .current_dir(project)
        .output()
        .expect("tsc is on PATH");
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

const BROWSER_STORES: [&str; 3] = [
    "useUrlState<",
    "useSessionStorageState<",
    "useLocalStorageState<",
];

// ── sensitive and credential state (schema: PlacementProfile.resolution.refusals) ──────────

const VAULT: &str = r#"
format: ess-ui/1
app: vault
model: vault.system
placement_profile: fat
placement_defaults: {draft: session_storage, page_state: url, preference: local_storage}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: secrets.edit
  sections: [{name: all, pages: [secrets.edit]}]
pages:
  secrets.edit:
    kind: form_page
    title: Secret
    state:
      token_hint: {type: string, class: page_state, sensitive: true}
      pin:        {type: string, class: preference, sensitive: true, store: server, fallback: {when: "true", store: local_storage}}
      api_token:  {type: string, class: credential, store: local_storage}
    sections:
      - name: form
        component: form
        does: secrets.Save
        fields: [{field: secret, as: secret}]
        draft: {type: SecretDraft, class: draft, sensitive: true}
"#;

/// A sensitive state whose store comes from the defaults must not land in the URL or in
/// browser storage: the schema refuses `sensitive` with `url`, `session_storage` and `local_storage`.
/// Refusing to generate is an equally valid answer.
#[test]
fn sensitive_state_placed_by_defaults_never_reaches_url_or_browser_storage() {
    let (_, result) = generate("sensitive-defaults", VAULT);
    let Ok(files) = result else { return };
    let page = file(&files, "src/pages/SecretsEdit.tsx");
    for local in ["tokenHintValue", "draftValue"] {
        let line = hook_line(page, local);
        for hook in BROWSER_STORES {
            assert!(
                !line.contains(hook),
                "sensitive state `{local}` is placed with {hook}: {line}"
            );
        }
    }
}

/// `store: server` with a `local_storage` fallback on a sensitive state: the fallback must not
/// be taken (or generation refuses).
#[test]
fn a_sensitive_state_never_falls_back_to_local_storage() {
    let (_, result) = generate("sensitive-fallback", VAULT);
    let Ok(files) = result else { return };
    let page = file(&files, "src/pages/SecretsEdit.tsx");
    let line = hook_line(page, "pinValue");
    assert!(
        !line.contains("local_storage"),
        "sensitive state falls back to localStorage: {line}"
    );
}

/// A credential is "always sensitive" and only `memory` or `server_session` may hold it.
#[test]
fn a_credential_is_never_placed_in_local_storage() {
    let (_, result) = generate("credential", VAULT);
    let Ok(files) = result else { return };
    let page = file(&files, "src/pages/SecretsEdit.tsx");
    let line = hook_line(page, "apiTokenValue");
    for hook in BROWSER_STORES {
        assert!(
            !line.contains(hook),
            "a credential is placed with {hook}: {line}"
        );
    }
}

// ── placement resolution order (schema: PlacementProfile.resolution.order) ─────────────────

/// Resolution order: explicit store, pinned, section profile, page profile, document defaults.
/// `activity.feed` is `profile: thin`, so its unstored `component_state` resolves to thin's
/// `server_session`; `workflows.editor` is `profile: fat`, so its unstored `preference` resolves
/// to fat's `local_storage` — both before `placement_defaults` is consulted.
#[test]
fn a_page_profile_decides_placement_before_document_defaults() {
    let files = example("profiles");
    let activity = file(&files, "src/pages/ActivityFeed.tsx");
    let paused = hook_line(activity, "pausedValue");
    assert!(
        paused.contains("useServerSessionState<"),
        "thin page: component_state resolves to server_session: {paused}"
    );
    let editor = file(&files, "src/pages/WorkflowsEditor.tsx");
    let zoom = hook_line(editor, "zoomValue");
    assert!(
        zoom.contains("useLocalStorageState<"),
        "fat page: preference resolves to local_storage: {zoom}"
    );
}

// ── browser-storage keys (schema: Store.meaning.{session,local}_storage.constraints.key) ───

/// The schema keys session and local storage by `[origin, actor.user_id, actor.account_id]`;
/// a key without the actor hands one user's drafts and preferences to the next on the device.
#[test]
fn browser_storage_keys_carry_the_actor() {
    let files = example("storage-keys");
    let runtime = [
        "src/runtime/state/storage.ts",
        "src/runtime/state/local_storage.ts",
        "src/runtime/state/session_storage.ts",
    ]
    .map(|path| file(&files, path).to_owned())
    .join("\n");
    assert!(
        runtime.contains("actor"),
        "no storage key is derived from the actor:\n{runtime}"
    );
}

// ── url query keys ─────────────────────────────────────────────────────────────────────────

const TWIN_PICKERS: &str = r"
format: ess-ui/1
app: twins
model: twins.system
placement_profile: fat
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
      - name: left
        component: record
        reads: {view: things.Left}
        fields: [title]
        children:
          - {name: picker, primitive: text, text: state.mode, state: {mode: {type: string, class: page_state, store: url}}}
      - name: right
        component: record
        reads: {view: things.Right}
        fields: [title]
        children:
          - {name: picker, primitive: text, text: state.mode, state: {mode: {type: string, class: page_state, store: url}}}
";

/// Two url states with distinct canonical paths must not share one query parameter.
#[test]
fn url_states_of_two_same_named_nodes_get_distinct_query_keys() {
    let (_, result) = generate("twin-pickers", TWIN_PICKERS);
    let files = result.unwrap_or_else(|error| panic!("{error}"));
    let page = file(&files, "src/pages/ThingsDetail.tsx");
    let keys: Vec<&str> = page
        .lines()
        .filter_map(|line| line.split_once("useUrlState<"))
        .filter_map(|(_, rest)| rest.split_once(">(\""))
        .filter_map(|(_, rest)| rest.split_once('"'))
        .map(|(key, _)| key)
        .collect();
    let unique: BTreeSet<&str> = keys.iter().copied().collect();
    assert_eq!(
        unique.len(),
        keys.len(),
        "two url states share a query key: {keys:?}"
    );
}

// ── identifier collisions ──────────────────────────────────────────────────────────────────

const DRAFT_NAMED_STATE: &str = r"
format: ess-ui/1
app: notes
model: notes.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: notes.new
  sections: [{name: all, pages: [notes.new]}]
pages:
  notes.new:
    kind: form_page
    title: New note
    sections:
      - name: form
        component: form
        does: notes.Create
        fields: [title]
        state:
          draft: {type: boolean, class: component_state, default: false}
";

/// A form section with a state named `draft` is valid ess-ui; the project must still type-check.
#[test]
fn a_form_section_state_named_draft_type_checks() {
    let (out, result) = generate("draft-named-state", DRAFT_NAMED_STATE);
    result.unwrap_or_else(|error| panic!("{error}"));
    if let Err(errors) = tsc(&out) {
        panic!("tsc --noEmit failed:\n{errors}");
    }
}

// ── typed drafts ───────────────────────────────────────────────────────────────────────────

const TYPED_DRAFT: &str = r"
format: ess-ui/1
app: notes
model: notes.system
placement_profile: fat
types:
  NoteDraft: {record: {title: string, body: string}}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: notes.new
  sections: [{name: all, pages: [notes.new]}]
pages:
  notes.new:
    kind: form_page
    title: New note
    sections:
      - name: form
        component: form
        does: notes.Create
        fields: [title, body]
        draft: {type: NoteDraft, class: draft, store: session_storage}
";

/// A draft whose type the document defines is held as that type, not `Record<string, unknown>`.
#[test]
fn a_draft_of_a_document_type_is_typed_as_that_type() {
    let (_, result) = generate("typed-draft", TYPED_DRAFT);
    let files = result.unwrap_or_else(|error| panic!("{error}"));
    let page = file(&files, "src/pages/NotesNew.tsx");
    let line = hook_line(page, "draftValue");
    assert!(
        line.contains("<M.NoteDraft>"),
        "the draft's known type is dropped: {line}"
    );
}

// ── live channels (schema: Channel, Live) ──────────────────────────────────────────────────

fn live_runtime(name: &str) -> String {
    file(&example(name), "src/runtime/live.ts").to_owned()
}

/// `session: {per: ticket_id}` makes one channel instance per param value. The transport and
/// the fixture player must be told the value: today every ticket page shares one `ticket_chat`
/// stream and plays the `ticket_id: tk-01` script into whichever ticket is open.
#[test]
fn a_per_param_channel_session_reaches_transport_and_fixture_player() {
    let live = live_runtime("live-session");
    let uses = live.matches("sessionPer").count();
    assert!(
        uses >= 2,
        "`sessionPer` is declared and never read ({uses} occurrence)"
    );
    assert!(
        live.contains(".session"),
        "the fixture player ignores the script's `session`"
    );
}

/// `stale_after` is "down time after which fed sections are stale". Only the fixture script can
/// report `stale` today: the SSE and WebSocket clients never do, and `lastLive` is written but
/// never read, so over the network a section fed by a `stale_after` channel is never stale.
#[test]
fn stale_after_turns_sections_stale_without_a_transport_stale_signal() {
    let live = live_runtime("live-stale");
    let reads = live
        .lines()
        .filter(|line| line.contains("lastLive"))
        .filter(|line| !line.contains("lastLive:") && !line.contains("lastLive ="))
        .count();
    let signals = live.contains("onLifecycle(\"stale\")") || live.contains("lifecycle = \"stale\"");
    assert!(
        reads > 0 || signals,
        "nothing measures down time against staleAfter; `stale` is never produced locally"
    );
}

/// `resume: refetch` means "read again" after a reconnect. No code path re-reads on reconnect.
#[test]
fn resume_refetch_reads_again_after_a_reconnect() {
    let live = live_runtime("live-resume");
    let acts = live
        .lines()
        .any(|line| line.contains(".resume") && line.contains("\"refetch\""));
    assert!(acts, "`resume: refetch` has no implementation");
}

const EFFECTS: &str = r"
format: ess-ui/1
app: rows
model: rows.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: rows.board
  sections: [{name: all, pages: [rows.board]}]
channels:
  rows:
    carries: {events: [rows.Changed, rows.Removed]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
    stale_after: 5s
  totals:
    carries: {view: rows.Totals}
    direction: server_to_client
    delivery: latest_value
    resume: refetch
pages:
  rows.board:
    kind: detail_page
    title: Rows
    header: {live: [rows, totals]}
    sections:
      - name: summary
        reads: {view: rows.One}
        fields: [title]
        live: {channel: totals, effect: replace}
      - name: patched
        component: collection
        reads: {view: rows.All, paging: server}
        columns: [title]
        live: {channel: rows, on: [rows.Changed], effect: patch_row, match: key}
        states: {stale: {mark: dim}}
      - name: upserted
        component: collection
        reads: {view: rows.All, paging: server}
        columns: [title]
        live: {channel: rows, effect: insert_or_patch, only_if: matches(params), coalesce: 250ms, when_paged_away: insert}
      - name: removed
        component: collection
        reads: {view: rows.All}
        columns: [title]
        live: {channel: rows, on: [rows.Removed], effect: remove_row}
      - name: reread
        component: collection
        reads: {view: rows.All}
        columns: [title]
        live: {channel: rows, effect: refetch, when_paged_away: ignore}
";

/// Every live effect the schema names generates a typed code path.
#[test]
fn every_live_effect_generates_and_type_checks() {
    let (out, result) = generate("effects", EFFECTS);
    let files = result.unwrap_or_else(|error| panic!("{error}"));
    let page = file(&files, "src/pages/RowsBoard.tsx");
    for needle in [
        "effect: \"replace\"",
        "effect: \"patch_row\"",
        "match: \"key\"",
        "effect: \"insert_or_patch\"",
        "onlyIf: \"matches(params)\"",
        "coalesce: 250",
        "effect: \"remove_row\"",
        "effect: \"refetch\"",
        "whenPagedAway: \"ignore\"",
        "whenPagedAway: \"insert\"",
    ] {
        assert!(page.contains(needle), "no `{needle}` in:\n{page}");
    }
    let live = file(&files, "src/runtime/live.ts");
    for arm in [
        "case \"patch_row\"",
        "case \"insert_or_patch\"",
        "case \"remove_row\"",
        "case \"replace\"",
        "spec.effect === \"refetch\"",
    ] {
        assert!(live.contains(arm), "no `{arm}` in the live runtime");
    }
    if let Err(errors) = tsc(&out) {
        panic!("tsc --noEmit failed:\n{errors}");
    }
}

// ── constructs the partner-portal example does not use ─────────────────────────────────────

const SINK: &str = r"
format: ess-ui/1
app: sink
model: sink.system
placement_profile: fat
shells:
  app:
    regions:
      nav:     {kind: navigation}
      main:    {kind: page_outlet}
      overlay: {kind: overlay_outlet}
      notify:  {kind: notifications}
      helper:  {kind: assistant}
      account: {kind: account_menu}
widgets:
  pair:
    summary: A label and a flag.
    params:
      left: {type: string, required: true, note: the label}
    arrange: grid
    body:
      - {name: left, primitive: text, text: args.left, format: duration, style: mono}
      - {name: flag, primitive: toggle, label: Flag, action: {name: flip, does: things.Flip}}
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list, things.edit]}]
pages:
  things.list:
    kind: list_page
    title: Things
    header:
      metrics:
        - {name: count, component: metric, reads: {view: things.Count}, format: bytes, label: Count}
    state:
      min:  {type: {optional: number}, class: page_state, store: url}
      cols: {type: {list: string}, class: page_state, store: url}
      kind: {type: {enum: [bar, line]}, class: component_state, default: bar}
      text: {type: string, class: draft}
    sections:
      - name: filters
        inputs: [{field: min, as: number, binds: state.min}]
        actions: [{name: clear, sets: {state.search: ''}, label: Clear}]
      - name: list
        component: collection
        style: tree
        selection: {mode: single, enabled: row.editable}
        reads: {view: things.Page, paging: cursor}
        columns: {binds: state.cols, all: [title, {field: size, as: number}]}
        actions: [{name: refresh, does: things.Refresh, label: Refresh, confirm: sure}]
        expand:
          component: record
          reads: {view: things.ById, params: {id: row.id}}
          fields: [title]
          tabs:
            - {name: main, label: Main, fields: [title], form: {name: inner, primitive: text, text: row.title}}
            - {name: act, label: Act, form: {name: go, does: things.Go, label: Go}}
          actions: [{name: archive, does: things.Archive, label: Archive}]
        item:
          - {name: pair, component: pair, args: {left: row.title}}
        states: {loading: spinner, empty: {message: None yet, action: {name: add, opens: sure, label: Add}}, stale: {mark: dim}}
      - name: kinds
        component: chart
        chart: {binds: state.kind, options: [bar, line]}
        reads: {view: things.Kinds}
        x: kind
        series: [count]
      - name: refs
        component: references
        reads: {view: things.UsedBy}
        columns: [title]
    overlays:
      sure:
        kind: popover
        component: confirm
        title: Sure?
        does: things.Clear
        alternatives: [{name: later, does: things.Later, label: Later}]
      big:
        kind: fullscreen
        component: rich_text
        title: Big
        syntax: ssml
        binds: state.text
  things.edit:
    kind: form_page
    title: Edit
    sections:
      - name: form
        component: form
        does: things.Update
        save: on_blur
        fields: [title]
        tabs:
          - {name: basics, label: Basics, fields: [{field: size, as: number}]}
        groups:
          - {name: extra, label: Extra, fields: [note], actions: [{name: reset, does: things.Reset, label: Reset}]}
        parts:
          - {name: tags, component: choice, options: [a, b], style: checklist, multiple: true, binds: draft.tags}
        actions: [{name: preview, does: things.Preview, label: Preview}]
        result: {component: record, fields: [title]}
        record: {primitive: text, text: row.title}
        children:
          - {name: go, primitive: button, label: Go, tone: ghost, action: {name: go, does: things.Go}}
";

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

/// A document using constructs the example lacks (tree style, conditional selection, selectable
/// columns, chosen chart kind, popover and fullscreen overlays, record and form tabs with nested
/// nodes and actions, form groups, parts, result and record, grid widgets, toggles with actions)
/// renders every node at its canonical path, invents none, and type-checks.
#[test]
fn constructs_the_example_lacks_render_at_their_paths_and_type_check() {
    let document = load(SINK);
    let out = scratch("sink");
    let files =
        ess_ui_react::generate(&document, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    let found = ui_paths(&files);
    let expected = rendered_paths(&document);
    let missing: Vec<_> = expected.difference(&found).collect();
    let invented: Vec<_> = found.difference(&expected).collect();
    assert!(
        missing.is_empty() && invented.is_empty(),
        "missing: {missing:#?}\ninvented: {invented:#?}"
    );
    if let Err(errors) = tsc(&out) {
        panic!("tsc --noEmit failed:\n{errors}");
    }
}
