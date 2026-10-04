//! beyond10x/ess#354 in the generated React project: a reading node inside a tab or a header
//! takes its own `live` block and applies the channel's events to its own read, while it is
//! shown; and `header.title_from` shows a field of the record a named section holds, with that
//! section's live changes applied.
//!
//! The generated page runs under `node` against the stand-in `react` of `support`, with a data
//! adapter that counts reads per view and a stand-in `EventSource` the test plays events into.

mod support;

use std::path::{Path, PathBuf};

use ess_ui::Document;
use ess_ui_react::GeneratedFiles;

const DOCUMENT: &str = r"
format: ess-ui/1
app: console
model: console.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: objective
  sections: [{name: all, pages: [objective]}]
channels:
  progress:
    carries: {events: [objectives.EvidenceAdded, objectives.GoalChanged]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  objective:
    kind: detail_page
    title: Objective
    header:
      title: Objective
      title_from: {section: objective, field: goal}
      metrics:
        - name: evidence_count
          component: metric
          reads: {view: objectives.Evidence}
          aggregate: count
          label: Evidence count
          live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}
    sections:
      - name: objective
        component: record
        reads: {view: objectives.ById, key: objective_id}
        live: {channel: progress, on: [objectives.GoalChanged], effect: patch_row}
        fields: [goal]
        tabs:
          - name: evidence
            label: Evidence
            form:
              name: evidence
              component: collection
              reads: {view: objectives.Evidence, key: evidence_id}
              columns: [note]
              live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}
          - name: cost
            label: Cost
            form:
              name: cost
              component: metric
              reads: {view: objectives.Cost}
              aggregate: sum
              field: amount
              label: Cost
              live: {channel: progress, effect: refetch}
";

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-live-composites")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

fn load(text: &str) -> Document {
    ess_ui::load_str(text).unwrap_or_else(|error| panic!("the document loads: {error}\n{text}"))
}

fn generate(name: &str, text: &str) -> (PathBuf, GeneratedFiles) {
    let out = scratch(name);
    let files = ess_ui_react::generate(&load(text), Path::new("."), &out)
        .unwrap_or_else(|error| panic!("the project generates: {error}"));
    (out, files)
}

/// Shared by every behavioural case: the adapter, the event source, the page and helpers.
const HARNESS: &str = r"
browser('/objective');
const { setDataAdapter } = out('runtime/data');
const { useNetworkChannels } = out('runtime/live');
const { ObjectivePage } = out('pages/Objective');
const reads = {};
const rows = {
  'objectives.ById': [{ objective_id: 'o-1', goal: 'Ship the importer' }],
  'objectives.Evidence': [{ evidence_id: 'e-1', note: 'First benchmark' }],
  'objectives.Cost': [{ id: 'c-1', amount: 10 }, { id: 'c-2', amount: 5 }],
};
setDataAdapter({ read: async (request) => {
  reads[request.view] = (reads[request.view] || 0) + 1;
  const answer = rows[request.view] || [];
  return { rows: answer.map((row) => ({ ...row })), total: answer.length };
}});
const sources = [];
global.EventSource = class { constructor(url) { this.url = url; this.closed = false; sources.push(this); } close() { this.closed = true; } };
useNetworkChannels('http://app.test');
const root = React.__root(jsx(ObjectivePage, {}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
function textOf(node) {
  if (typeof node === 'string') return node;
  return (node.children || []).map(textOf).join('');
}
function play(event, payload) {
  const open = sources.filter((source) => !source.closed);
  assert.ok(open.length > 0, 'a channel is open');
  for (const source of open) source.onmessage({ data: JSON.stringify({ event, payload }) });
}
const heading = () => root.find((node) => node.tag === 'h1').map(textOf).join('|');
const metric = (label) => {
  const found = root.find((node) => node.tag === 'div' && node.props.className === 'ui-metric' && textOf(node).includes(label));
  assert.equal(found.length, 1, `one metric labelled ${label}: ${root.text()}`);
  const value = found[0].children.find((child) => child.props && child.props.className === 'ui-metric-value');
  return textOf(value);
};
const tab = (label) => {
  const found = root.find((node) => node.tag === 'button' && node.props.role === 'tab' && textOf(node) === label);
  assert.equal(found.length, 1, `one tab labelled ${label}`);
  found[0].props.onClick();
};
";

fn run(test: &str, body: &str) {
    run_with(test, DOCUMENT, body);
}

fn run_with(test: &str, document_text: &str, body: &str) {
    let (out, _) = generate(test, document_text);
    support::compile(&out, &["pages/Objective.tsx"]);
    support::node(
        &out,
        test,
        &format!(
            "{HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

#[test]
fn a_live_collection_in_the_shown_tab_takes_the_channels_events() {
    run(
        "tab-collection",
        r"
  await settle();
  assert.ok(root.text().includes('First benchmark'), root.text());
  play('objectives.EvidenceAdded', { evidence_id: 'e-2', note: 'Second benchmark' });
  await settle();
  assert.ok(root.text().includes('Second benchmark'), `the tab's collection took the event: ${root.text()}`);
  assert.ok(root.text().includes('First benchmark'), root.text());
",
    );
}

#[test]
fn a_live_header_metric_takes_the_channels_events() {
    run(
        "header-metric",
        r"
  await settle();
  assert.equal(metric('Evidence count'), '1');
  play('objectives.EvidenceAdded', { evidence_id: 'e-2', note: 'Second benchmark' });
  await settle();
  assert.equal(metric('Evidence count'), '2', 'the header metric counts the inserted row');
  play('objectives.GoalChanged', { objective_id: 'o-1', goal: 'Ship it' });
  await settle();
  assert.equal(metric('Evidence count'), '2', 'an event outside `on` changes nothing');
",
    );
}

#[test]
fn the_header_title_reads_the_named_sections_record_and_its_live_changes() {
    run(
        "header-title",
        r"
  await settle();
  assert.equal(heading(), 'Ship the importer', 'the title is the record field');
  play('objectives.GoalChanged', { objective_id: 'o-1', goal: 'Ship the importer twice' });
  await settle();
  assert.equal(heading(), 'Ship the importer twice', 'the title follows the section patched live');
",
    );
}

#[test]
fn the_header_title_is_the_literal_title_until_the_record_holds_the_field() {
    run(
        "header-title-fallback",
        r"
  rows['objectives.ById'] = [{ objective_id: 'o-1' }];
  root.settle();
  assert.equal(heading(), 'Objective', 'before the read answers');
  await settle();
  assert.equal(heading(), 'Objective', 'a record without the field');
",
    );
}

#[test]
fn an_inactive_tab_holds_no_subscription_and_reads_again_when_shown() {
    run(
        "inactive-tab",
        r"
  await settle();
  assert.equal(reads['objectives.Cost'] || 0, 0, 'the hidden cost tab reads nothing');
  play('objectives.EvidenceAdded', { evidence_id: 'e-2', note: 'Second benchmark' });
  await settle();
  assert.equal(reads['objectives.Cost'] || 0, 0, 'a refetch event does not reach the hidden tab');
  tab('Cost');
  await settle();
  assert.equal(reads['objectives.Cost'], 1, 'shown, the tab reads');
  assert.equal(metric('Cost'), '15');
  rows['objectives.Cost'].push({ id: 'c-3', amount: 5 });
  play('objectives.GoalChanged', { objective_id: 'o-1', goal: 'Ship it' });
  await settle();
  assert.equal(reads['objectives.Cost'], 2, 'the shown tab refetches on an event');
  assert.equal(metric('Cost'), '20');
  const evidenceReads = reads['objectives.Evidence'];
  tab('Evidence');
  await settle();
  assert.ok(reads['objectives.Evidence'] > evidenceReads, 'the evidence tab reads again when shown');
",
    );
}

#[test]
fn a_document_without_nested_live_generates_what_it_did() {
    let plain = DOCUMENT
        .replace(
            "\n          live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}",
            "",
        )
        .replace(
            "\n              live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}",
            "",
        )
        .replace("\n              live: {channel: progress, effect: refetch}", "")
        .replace("\n      title_from: {section: objective, field: goal}", "");
    let (_, files) = generate("plain", &plain);
    let page = &files.files["src/pages/Objective.tsx"];
    assert_eq!(
        page.matches("useLive(").count(),
        1,
        "only the live section plays the channel: {page}"
    );
    assert!(!page.contains("titleFrom"), "{page}");
    assert!(!page.contains("DataScope"), "{page}");
}

/// A choice with its own `reads` takes `live` like any reading composite: its options are the
/// rows of its read with the channel's events applied.
#[test]
fn a_live_choice_takes_the_channels_events_into_its_options() {
    let text = DOCUMENT.replace(
        "        fields: [goal]\n",
        "        fields: [goal]\n        item:\n          - name: owner\n            component: choice\n            reads: {view: people.All, key: person_id}\n            value: person_id\n            label: name\n            live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}\n",
    );
    run_with(
        "live-choice",
        &text,
        r"
  rows['people.All'] = [{ person_id: 'p-1', name: 'Ada' }];
  await settle();
  assert.ok(root.text().includes('Ada'), root.text());
  assert.ok(!root.text().includes('Grace'), root.text());
  play('objectives.EvidenceAdded', { person_id: 'p-2', name: 'Grace' });
  await settle();
  assert.ok(root.text().includes('Grace'), `the choice offers the inserted row: ${root.text()}`);
  assert.equal(reads['people.All'], 1, 'insert_top changes the options without a read');
",
    );
}

// ── bound to a served surface ────────────────────────────────────────────────────────────────

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn gatepass() -> Vec<(String, String)> {
    let root = root().join("examples/gatepass");
    ["system.yaml", "components.yaml", "domains/visit.yaml"]
        .iter()
        .map(|file| {
            let text = std::fs::read_to_string(root.join(file))
                .unwrap_or_else(|error| panic!("{file}: {error}"));
            ((*file).to_owned(), text)
        })
        .collect()
}

/// A desk whose record section's item holds a live metric; `extra` is spliced into the metric.
fn bound_desk(extra: &str) -> String {
    format!(
        "format: ess-ui/1
app: desk
model: gatepass
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: desk, sections: [{{name: all, pages: [desk]}}]}}
channels:
  visits:
    carries: {{events: [visit.VisitRegistered]}}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  desk:
    kind: detail_page
    title: Desk
    sections:
      - name: expected
        component: collection
        reads: visit.ExpectedVisits
        item:
          - name: waiting
            component: metric
            reads: {{view: visit.ExpectedVisits{extra}}}
            aggregate: count
            live: {{channel: visits, effect: insert_top}}
"
    )
}

fn render_bound(text: &str) -> Result<GeneratedFiles, String> {
    let document = load(text);
    let binding = ess_ui_check::binding(&document, &gatepass())
        .unwrap_or_else(|error| panic!("the document binds: {error}"));
    ess_ui_react::render_bound(&document, &root(), Some(&binding)).map_err(|e| e.to_string())
}

#[test]
fn a_bound_nested_live_node_polls_its_read() {
    let files = render_bound(&bound_desk(", refresh: 30s")).expect("the desk renders");
    let polls: Vec<&str> = files
        .files
        .values()
        .flat_map(|text| text.lines())
        .filter(|line| line.contains("usePoll(") && line.contains("30000"))
        .collect();
    assert_eq!(
        polls.len(),
        1,
        "the nested live metric polls at its refresh"
    );
    assert!(
        !files.files.contains_key("src/runtime/live.ts"),
        "a bound project holds no channel open"
    );
    let out = scratch("bound-compiles");
    for (path, text) in &files.files {
        support::write(&out.join(path), text);
    }
    support::compile(&out, &["pages/Desk.tsx"]);
}

#[test]
fn a_bound_nested_live_node_refusing_to_poll_is_refused_at_its_path() {
    let text = bound_desk("").replace(
        "            aggregate: count\n",
        "            aggregate: count\n            degrades: {no_live: refuse}\n",
    );
    let error = render_bound(&text).expect_err("refused");
    assert!(
        error.contains("pages/desk/sections/expected/item/waiting/live"),
        "{error}"
    );
}
