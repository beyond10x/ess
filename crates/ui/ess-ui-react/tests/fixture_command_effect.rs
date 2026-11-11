//! In fixture mode a confirmed command changes the fixture rows the way the model says it does
//! (story:fixture-mode-applies-a-confirmed-commands-effect).
//!
//! The runtime cases compile the generated `runtime/data.ts` and `runtime/actions.tsx` to
//! `CommonJS` with `tsc` and run them under `node` against the stand-ins in `support`.

mod support;

use std::path::{Path, PathBuf};

use ess_ui::binding::{CommandEffect, FixtureEffects, ModelEnums, RowFilter};
use support::{compile, node};

const COMMAND: &str = "desk.requests.ApproveRequest";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fixture-command-effect")
}

fn sources() -> Vec<(String, String)> {
    ["system.yaml", "components.yaml", "requests.yaml"]
        .iter()
        .map(|file| {
            let text = std::fs::read_to_string(fixtures_dir().join("spec").join(file))
                .expect("the model reads");
            ((*file).to_owned(), text)
        })
        .collect()
}

fn model() -> ess_ui_check::Model {
    ess_ui_check::model_from_sources(&sources(), Path::new("spec")).expect("the model compiles")
}

fn document(model: &ess_ui_check::Model) -> ess_ui::Document {
    ess_ui::load_path_with(&fixtures_dir().join("ui/desk.ui.yaml"), model)
        .unwrap_or_else(|error| panic!("{error}"))
}

fn scratch(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-fixture-command-effect")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    out
}

/// Generates the fixture project (with `effects`, or without) and compiles what the cases run.
fn project(name: &str, effects: Option<&FixtureEffects>) -> PathBuf {
    let out = scratch(name);
    let document = document(&model());
    let ui = fixtures_dir().join("ui");
    ess_ui_react::generate_with(&document, &ui, &out, None, effects)
        .unwrap_or_else(|error| panic!("{error}"));
    compile(&out, &["runtime/data.ts", "runtime/actions.tsx"]);
    out
}

const READS: &str = r#"
const view = async (name) => {
  const { fixtureAdapter } = out("runtime/data");
  const read = await fixtureAdapter.read({ view: `desk.requests.${name}`, params: {} });
  return read.rows.map((row) => `${row.id}:${row.state}`);
};
"#;

#[test]
fn the_model_says_what_approving_a_request_does() {
    let model = model();
    let effects = model
        .fixture_effects(&document(&model))
        .expect("a model gives effects");
    let Some(CommandEffect::Applies(change)) = effects.commands.get(COMMAND) else {
        panic!("the command applies: {:?}", effects.commands);
    };
    assert_eq!(change.entity, "desk.requests.Request");
    assert_eq!(change.instance, "id");
    assert_eq!(change.to.as_deref(), Some("Approved"));
    let pending = &effects.views["desk.requests.PendingRequests"];
    assert_eq!(pending.identity, "id");
    assert_eq!(pending.state, "state");
    assert_eq!(
        pending.filter,
        Some(RowFilter::Eq {
            field: "state".to_owned(),
            value: serde_json::json!("Pending"),
        })
    );
    assert_eq!(effects.views["desk.requests.AllRequests"].filter, None);
}

/// After the command the filtered view lists only `r2` and the unfiltered view of the same entity
/// shows `r1` approved; a fresh module instance (a reload) reads the rows as written.
#[test]
fn a_confirmed_command_moves_the_row_out_of_the_filtered_view() {
    let model = model();
    let effects = model.fixture_effects(&document(&model));
    let project = project("applies", effects.as_ref());
    node(
        &project,
        "applies",
        &format!(
            r#"{READS}
(async () => {{
  assert.deepStrictEqual(await view("PendingRequests"), ["r1:Pending", "r2:Pending"]);
  const {{ fixtureAdapter }} = out("runtime/data");
  const answer = await fixtureAdapter.send("{COMMAND}", {{ id: "r1" }});
  assert.strictEqual(answer.unapplied, undefined, "the effect was applied");
  assert.deepStrictEqual(await view("PendingRequests"), ["r2:Pending"]);
  assert.deepStrictEqual(await view("AllRequests"), ["r1:Approved", "r2:Pending", "r3:Approved"]);
  // A reload: the module is evaluated again, and the fixture rows are as written.
  for (const key of Object.keys(require.cache)) {{
    if (key.includes("test-out")) delete require.cache[key];
  }}
  assert.deepStrictEqual(await view("PendingRequests"), ["r1:Pending", "r2:Pending"]);
  assert.deepStrictEqual(await view("AllRequests"), ["r1:Pending", "r2:Pending", "r3:Approved"]);
}})().catch((error) => {{ console.error(error); process.exit(1); }});
"#
        ),
    );
}

/// Without a model, and for a reason the model gives, the rows stay as they were and the notice
/// and the console line say so: never just `<command> done`.
#[test]
fn an_effect_fixture_mode_cannot_apply_is_said_where_the_user_acted() {
    let reason = "its success creates a row (`creates:`)";
    let mut effects = FixtureEffects::default();
    effects.commands.insert(
        COMMAND.to_owned(),
        CommandEffect::NotApplied {
            unapplied: reason.to_owned(),
        },
    );
    for (name, effects, expected) in [
        (
            "no-model",
            None,
            "no model was given (generate with `--model <specification> --fixtures`)",
        ),
        ("not-applied", Some(&effects), reason),
    ] {
        let project = project(name, effects);
        node(
            &project,
            name,
            &format!(
                r#"{READS}
(async () => {{
  const {{ runCommand }} = out("runtime/actions");
  const notices = [];
  const warned = [];
  console.warn = (line) => warned.push(line);
  const scope = {{ notify: (text, tone) => notices.push([text, tone]) }};
  assert.strictEqual(await runCommand(scope, "{COMMAND}", {{ id: "r1" }}), true);
  const line = "{COMMAND} done in fixture mode; its effect was not applied: {expected}";
  assert.deepStrictEqual(notices, [[line, undefined]]);
  assert.deepStrictEqual(warned, [line]);
  assert.deepStrictEqual(await view("PendingRequests"), ["r1:Pending", "r2:Pending"]);
}})().catch((error) => {{ console.error(error); process.exit(1); }});
"#
            ),
        );
    }
}

/// A command whose effect the model says nothing about is done, as before.
#[test]
fn a_command_that_changes_no_row_is_just_done() {
    let model = model();
    let effects = FixtureEffects::default();
    let _ = model;
    let project = project("nothing-to-apply", Some(&effects));
    node(
        &project,
        "nothing-to-apply",
        r#"
(async () => {
  const { runCommand } = out("runtime/actions");
  const notices = [];
  const scope = { notify: (text, tone) => notices.push([text, tone]) };
  assert.strictEqual(await runCommand(scope, "desk.requests.Other", { id: "r1" }), true);
  assert.deepStrictEqual(notices, [["desk.requests.Other done", "success"]]);
})().catch((error) => { console.error(error); process.exit(1); });
"#,
    );
}

/// A bound build (`--model` without `--fixtures`) gives the same files as before: no `effects` key
/// in `src/fixtures.ts`, no effect code in `runtime/data.ts`, and effects handed to it change
/// nothing.
#[test]
fn a_bound_build_carries_no_effect_code() {
    let model = model();
    let loaded = document(&model);
    let binding = ess_ui_check::binding(&loaded, &sources()).expect("the desk binds");
    let ui = fixtures_dir().join("ui");
    let bound = ess_ui_react::render_bound(&loaded, &ui, Some(&binding)).expect("it renders");
    let fixtures = &bound.files["src/fixtures.ts"];
    assert!(!fixtures.contains("effects"), "no effects key:\n{fixtures}");
    let data = &bound.files["src/runtime/data.ts"];
    for effect_code in [
        "applyEffect",
        "changedRows",
        "Unapplied",
        "fixtureSend",
        "FixtureEffects",
    ] {
        assert!(
            !data.contains(effect_code),
            "{effect_code} is in a bound build"
        );
    }
    let actions = &bound.files["src/runtime/actions.tsx"];
    assert!(
        !actions.contains("fixture mode"),
        "a bound build names no fixture mode"
    );
    let effects = model.fixture_effects(&loaded);
    let with_effects = ess_ui_react::render_with(&loaded, &ui, Some(&binding), effects.as_ref())
        .expect("it renders");
    assert_eq!(
        with_effects, bound,
        "effects change no byte of a bound build"
    );
}
