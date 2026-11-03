//! A record's actions see the record's row in both renderers: a `row.<key>` bind sends the row's
//! key and a `visible: row.<field> == …` guard lets the action act. One document runs through the
//! terminal (`ess ui test`) and through the generated React page under `node`, and the commands
//! each sends are compared.
//!
//! React cases compile the generated runtime with `tsc` and run it under `node` against the
//! stand-ins in `ess-ui-react`'s test support.

#[path = "../../ess-ui-react/tests/support/mod.rs"]
mod support;

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = r#"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {things.ById: things.yaml}}
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: things.detail
  sections: [{name: all, pages: [things.detail]}]
pages:
  things.detail:
    kind: detail_page
    title: Thing
    sections:
      - name: thing
        component: record
        reads: {view: things.ById}
        fields: [name, state]
        actions:
          - {name: rename, does: things.Rename, bind: {thing: row.thing_key}, label: Rename}
          - {name: approve, does: things.Approve, bind: {thing: row.thing_key}, label: Approve, visible: "row.state == open"}
          - {name: reopen, does: things.Reopen, bind: {thing: row.thing_key}, label: Reopen, visible: "row.state == closed"}
"#;

const FIXTURE: &str =
    "views:\n  things.ById:\n    rows:\n      - {thing_key: k-1, name: Anvil, state: open}\n";

const ACTIONS: &str = "pages/things.detail/sections/thing/actions";

fn scratch(kind: &str, name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-record-action")
        .join(kind)
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is made");
    dir
}

fn document(name: &str) -> PathBuf {
    let dir = scratch("terminal", name);
    std::fs::write(dir.join("things.yaml"), FIXTURE).expect("the fixture is written");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    dir.join("ui.yaml")
}

fn terminal(name: &str, steps: &[String]) -> Outcome {
    let document = document(name);
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: record action\n  steps:\n",
        document.display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    let file = ess_ui_test::parse_str(&text, Path::new("record_action.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report = ess_ui_test::execute(&document, &[file]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

/// The commands the generated React page sends when `acts` (action names) are clicked in turn,
/// and the names of the record's actions it draws.
fn react(name: &str, acts: &[&str]) -> (Vec<(String, serde_json::Value)>, Vec<String>) {
    // The document's fixtures are read from the directory it is generated from.
    let root = scratch("react", name);
    std::fs::write(root.join("things.yaml"), FIXTURE).expect("the fixture is written");
    let out = root.join("app");
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::generate(&document, &root, &out).unwrap_or_else(|error| panic!("{error}"));
    support::compile(&out, &["pages/ThingsDetail.tsx"]);
    let acts = serde_json::to_string(acts).expect("the names serialize");
    let script = format!(
        r#"
const {{ ThingsDetailPage }} = out("pages/ThingsDetail");
const {{ setDataAdapter }} = out("runtime/data");
const sent = [];
setDataAdapter({{
  read: async (request) => ({{ rows: request.view === "things.ById" ? [{{ thing_key: "k-1", name: "Anvil", state: "open" }}] : [], total: 1 }}),
  send: async (command, input) => {{ sent.push([command, input]); return {{ accepted: true }}; }},
  loadState: async () => undefined,
  saveState: async () => undefined,
}});
const root = React.__root(jsx(ThingsDetailPage, {{}}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() {{ for (let i = 0; i < 4; i++) {{ root.settle(); await tick(); }} root.settle(); }}
const button = (name) => root.find((n) => n.tag === "button" && n.props["data-ui-path"] === "{ACTIONS}/" + name);
(async () => {{
  await settle();
  const drawn = ["rename", "approve", "reopen"].filter((name) => button(name).length > 0);
  for (const name of {acts}) {{
    const found = button(name);
    assert.strictEqual(found.length, 1, `one ${{name}} button: ${{root.text()}}`);
    click(found[0]);
    await settle();
  }}
  console.log(JSON.stringify({{ sent, drawn }}));
}})().catch((error) => {{ console.error(error); process.exit(1); }});
"#
    );
    let printed = support::node_stdout(&out, name, &script);
    let line = printed.lines().last().unwrap_or_default();
    let value: serde_json::Value =
        serde_json::from_str(line).unwrap_or_else(|error| panic!("{error}: {printed}"));
    let sent = value["sent"]
        .as_array()
        .expect("sent is a list")
        .iter()
        .map(|pair| {
            (
                pair[0].as_str().expect("a command name").to_owned(),
                pair[1].clone(),
            )
        })
        .collect();
    let drawn = value["drawn"]
        .as_array()
        .expect("drawn is a list")
        .iter()
        .map(|name| name.as_str().expect("a name").to_owned())
        .collect();
    (sent, drawn)
}

/// The command names the terminal sent, in order, read from the refusal of a command never sent.
fn terminal_sent(name: &str, acts: &[&str]) -> Vec<String> {
    let mut steps = vec!["open: things.detail".to_owned()];
    steps.extend(acts.iter().map(|act| format!("act: {ACTIONS}/{act}")));
    steps.push("expect_command: probe.NeverSent".to_owned());
    let outcome = terminal(name, &steps);
    assert_eq!(outcome.step, Some(steps.len()), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    let Some((list, _)) = message
        .rsplit_once("sent: [")
        .and_then(|(_, rest)| rest.split_once(']'))
    else {
        panic!("the refusal lists what was sent: {message}");
    };
    list.split(", ")
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_record_actions_row_bind_sends_the_rows_key_in_the_terminal() {
    let outcome = terminal(
        "bind",
        &[
            "open: things.detail".to_owned(),
            format!("act: {ACTIONS}/rename"),
            "expect_command: {command: things.Rename, input: {thing: k-1}}".to_owned(),
        ],
    );
    assert_eq!(
        outcome.status,
        Status::Passed,
        "story:a-record-action-in-the-terminal-renderer-sees-its-row: {outcome:?}"
    );
}

#[test]
fn a_record_action_visible_on_its_row_acts_in_the_terminal() {
    let outcome = terminal(
        "visible",
        &[
            "open: things.detail".to_owned(),
            format!("act: {ACTIONS}/approve"),
            "expect_command: {command: things.Approve, input: {thing: k-1}}".to_owned(),
        ],
    );
    assert_eq!(
        outcome.status,
        Status::Passed,
        "story:a-record-action-in-the-terminal-renderer-sees-its-row: {outcome:?}"
    );
}

#[test]
fn a_record_action_hidden_by_its_row_is_not_offered_in_the_terminal() {
    let outcome = terminal(
        "hidden",
        &[
            "open: things.detail".to_owned(),
            format!("act: {ACTIONS}/reopen"),
        ],
    );
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    assert!(message.contains("does not offer"), "{message}");
}

#[test]
fn the_terminal_and_react_send_the_same_commands_for_one_record_script() {
    let acts = ["rename", "approve"];
    let (react_sent, drawn) = react("parity", &acts);
    assert_eq!(
        drawn,
        ["rename", "approve"],
        "React draws the actions its row makes visible"
    );
    assert_eq!(
        react_sent,
        [
            (
                "things.Rename".to_owned(),
                serde_json::json!({"thing": "k-1"})
            ),
            (
                "things.Approve".to_owned(),
                serde_json::json!({"thing": "k-1"})
            ),
        ],
        "React sends the row's key"
    );
    let names: Vec<String> = react_sent.iter().map(|(name, _)| name.clone()).collect();
    assert_eq!(
        terminal_sent("parity-names", &acts),
        names,
        "story:a-record-action-in-the-terminal-renderer-sees-its-row: the terminal sends the \
         commands React sends, in order"
    );
    let mut steps = vec!["open: things.detail".to_owned()];
    steps.extend(acts.iter().map(|act| format!("act: {ACTIONS}/{act}")));
    steps.extend(
        react_sent.iter().map(|(command, input)| {
            format!("expect_command: {{command: {command}, input: {input}}}")
        }),
    );
    let outcome = terminal("parity-inputs", &steps);
    assert_eq!(
        outcome.status,
        Status::Passed,
        "story:a-record-action-in-the-terminal-renderer-sees-its-row: the terminal sends React's \
         inputs: {outcome:?}"
    );
}
