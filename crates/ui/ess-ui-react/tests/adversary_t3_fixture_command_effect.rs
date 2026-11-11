//! Adversary cases for story:fixture-mode-applies-a-confirmed-commands-effect, generated React
//! runtime (`ess generate ui --target react --model <spec> --fixtures`). The model and document
//! are the terminal cases' own (`ess-ui-test/tests/fixtures/adversary-t3-fixture-command-effect`),
//! so both renderers are driven from one specification.

mod support;

use std::path::{Path, PathBuf};

use ess_ui::binding::ModelEnums;
use support::{compile, node};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ess-ui-test/tests/fixtures/adversary-t3-fixture-command-effect")
}

fn model() -> ess_ui_check::Model {
    let sources: Vec<(String, String)> = ["system.yaml", "components.yaml", "requests.yaml"]
        .iter()
        .map(|file| {
            let text = std::fs::read_to_string(fixtures_dir().join("spec").join(file))
                .expect("the model reads");
            ((*file).to_owned(), text)
        })
        .collect();
    ess_ui_check::model_from_sources(&sources, Path::new("spec")).expect("the model compiles")
}

fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-t3-fixture-command-effect")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let model = model();
    let document = ess_ui::load_path_with(&fixtures_dir().join("ui/desk.ui.yaml"), &model)
        .unwrap_or_else(|error| panic!("{error}"));
    let effects = model.fixture_effects(&document);
    let ui = fixtures_dir().join("ui");
    ess_ui_react::generate_with(&document, &ui, &out, None, effects.as_ref())
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

/// Rejecting `r3`, which is `Approved` (terminal; `reject` leaves only `Pending`), is what the
/// model answers with `wrong_state`. Fixture mode moves it to `Rejected` and says `done`.
#[test]
fn a_move_the_lifecycle_does_not_allow_is_not_applied_and_not_just_done() {
    let project = project("forbidden-move");
    node(
        &project,
        "forbidden-move",
        &format!(
            r#"{READS}
(async () => {{
  const {{ runCommand }} = out("runtime/actions");
  const notices = [];
  console.warn = () => {{}};
  const scope = {{ notify: (text, tone) => notices.push([text, tone]) }};
  await runCommand(scope, "desk.requests.RejectRequest", {{ id: "r3" }});
  assert.deepStrictEqual(
    await view("AllRequests"),
    ["r1:Pending", "r2:Pending", "r3:Approved", "r4:Pending"],
    "r3 stays Approved",
  );
  assert.notDeepStrictEqual(notices, [["desk.requests.RejectRequest done", "success"]]);
}})().catch((error) => {{ console.error(error); process.exit(1); }});
"#
        ),
    );
}

/// Approving `r1` makes `state == Approved` hold, so the approved view lists it.
#[test]
#[ignore = "backlog: a changed row does not enter a view it did not list; fixture rows are per view"]
fn an_approved_request_enters_the_approved_view() {
    let project = project("enters-view");
    node(
        &project,
        "enters-view",
        &format!(
            r#"{READS}
(async () => {{
  const {{ fixtureAdapter }} = out("runtime/data");
  const answer = await fixtureAdapter.send("desk.requests.ApproveRequest", {{ id: "r1" }});
  assert.strictEqual(answer.unapplied, undefined, "the effect was applied");
  assert.deepStrictEqual(await view("PendingRequests"), ["r2:Pending"]);
  assert.deepStrictEqual((await view("ApprovedRequests")).sort(), ["r1:Approved", "r3:Approved"]);
}})().catch((error) => {{ console.error(error); process.exit(1); }});
"#
        ),
    );
}

/// A detail view derived `by_id_from` the pending list (`desk-derived.ui.yaml`), read for `r1` after
/// approving `r1` moved it out of that list: it must not answer with another request's row. The
/// runtime's `by_id_from` falls back to the first row of the source when the id is not in it, so
/// the detail of `r1` shows `r2`; the terminal renderer answers no row.
#[test]
fn a_detail_derived_from_a_filtered_list_never_shows_another_row() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-t3-fixture-command-effect")
        .join("derived-detail");
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let model = model();
    let main = ess_ui::load_path_with(&fixtures_dir().join("ui/desk.ui.yaml"), &model)
        .unwrap_or_else(|error| panic!("{error}"));
    let effects = model.fixture_effects(&main);
    let derived = ess_ui::load_path(&fixtures_dir().join("ui/desk-derived.ui.yaml"))
        .unwrap_or_else(|error| panic!("{error}"));
    let ui = fixtures_dir().join("ui");
    ess_ui_react::generate_with(&derived, &ui, &out, None, effects.as_ref())
        .unwrap_or_else(|error| panic!("{error}"));
    compile(&out, &["runtime/data.ts", "runtime/actions.tsx"]);
    node(
        &out,
        "derived-detail",
        r#"
(async () => {
  const { fixtureAdapter } = out("runtime/data");
  const detail = async (id) => {
    const read = await fixtureAdapter.read({ view: "desk.requests.RequestById", params: { id } });
    return read.rows.map((row) => `${row.id}:${row.state}`);
  };
  assert.deepStrictEqual(await detail("r1"), ["r1:Pending"]);
  const answer = await fixtureAdapter.send("desk.requests.ApproveRequest", { id: "r1" });
  assert.strictEqual(answer.unapplied, undefined, "the effect was applied");
  const after = await detail("r1");
  assert.ok(!after.includes("r2:Pending"), `the detail of r1 shows ${JSON.stringify(after)}`);
})().catch((error) => { console.error(error); process.exit(1); });
"#,
    );
}

/// `priority == 1` still holds after approving `r1`: the row stays in `UrgentRequests`, patched.
/// The terminal renderer's case of the same name drops it.
#[test]
fn a_row_whose_numeric_filter_still_holds_stays_in_its_view() {
    let project = project("numeric-filter");
    node(
        &project,
        "numeric-filter",
        &format!(
            r#"{READS}
(async () => {{
  const {{ fixtureAdapter }} = out("runtime/data");
  const answer = await fixtureAdapter.send("desk.requests.ApproveRequest", {{ id: "r1" }});
  assert.strictEqual(answer.unapplied, undefined, "the effect was applied");
  assert.deepStrictEqual(await view("UrgentRequests"), ["r1:Approved", "r4:Pending"]);
}})().catch((error) => {{ console.error(error); process.exit(1); }});
"#
        ),
    );
}
