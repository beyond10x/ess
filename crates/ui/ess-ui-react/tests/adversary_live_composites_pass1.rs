//! Adversary pass 1 for beyond10x/ess#354: the React half of two event sequences the terminal
//! half (`crates/ui/ess-ui-tui/tests/adversary_live_composites_pass1.rs`) plays. These cases
//! record what the generated React page does, so the two renderers can be compared by a program.

mod support;

use std::path::{Path, PathBuf};

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-live-composites-pass1")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

/// Shared by every case: the adapter counting reads per view and params, the event source and
/// helpers. `PAGE` and `MODULE` name the page component.
const HARNESS: &str = r"
const { setDataAdapter } = out('runtime/data');
const { useNetworkChannels } = out('runtime/live');
const reads = {};
const rows = {
  'objectives.All': [{ objective_id: 'o-1', goal: 'First goal' }, { objective_id: 'o-2', goal: 'Second goal' }],
  'objectives.Evidence': [
    { evidence_id: 'e-1', note: 'First benchmark' },
    { evidence_id: 'e-2', note: 'Second benchmark' },
    { evidence_id: 'e-3', note: 'Third benchmark' },
  ],
};
setDataAdapter({ read: async (request) => {
  const key = `${request.view}|${JSON.stringify(request.params || {})}`;
  reads[key] = (reads[key] || 0) + 1;
  const answer = rows[request.view] || [];
  return { rows: answer.map((row) => ({ ...row })), total: answer.length };
}});
const sources = [];
global.EventSource = class { constructor(url) { this.url = url; this.closed = false; sources.push(this); } close() { this.closed = true; } };
useNetworkChannels('http://app.test');
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
function textOf(node) {
  if (typeof node === 'string') return node;
  return (node.children || []).map(textOf).join('');
}
function play(event, payload) {
  const open = sources.filter((source) => !source.closed);
  assert.ok(open.length > 0, 'a channel is open');
  for (const source of open) source.onmessage({ data: JSON.stringify({ event, payload }) });
}
";

fn run(test: &str, document_text: &str, page: &str, body: &str) {
    let document = ess_ui::load_str(document_text)
        .unwrap_or_else(|error| panic!("the document loads: {error}"));
    let out = scratch(test);
    ess_ui_react::generate(&document, Path::new("."), &out)
        .unwrap_or_else(|error| panic!("the project generates: {error}"));
    let module = format!("pages/{page}");
    support::compile(&out, &[&format!("{module}.tsx")]);
    support::node(
        &out,
        test,
        &format!(
            "browser('/{route}');\n{HARNESS}\nconst {{ {page}Page }} = out('{module}');\n\
             const root = React.__root(jsx({page}Page, {{}}));\n\
             async function settle() {{ for (let i = 0; i < 4; i++) {{ root.settle(); await tick(); }} root.settle(); }}\n\
             (async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n",
            route = page.to_lowercase(),
        ),
    );
}

const EXPAND: &str = r"
format: ess-ui/1
app: console
model: console.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: objectives
  sections: [{name: all, pages: [objectives]}]
channels:
  progress:
    carries: {events: [objectives.EvidenceAdded]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  objectives:
    kind: list_page
    title: Objectives
    sections:
      - name: list
        component: collection
        reads: {view: objectives.All, key: objective_id}
        columns: [goal]
        expand:
          name: detail
          component: metric
          reads: {view: objectives.Evidence, params: {objective: row.objective_id}}
          aggregate: count
          label: Evidence
          live: {channel: progress, effect: insert_top}
";

/// React: o-1's expand is shown, o-2's replaces it, o-1's is shown again and reads again.
#[test]
fn react_an_expand_shown_again_for_its_row_reads_again() {
    run(
        "expand-again",
        EXPAND,
        "Objectives",
        r"
  await settle();
  const row = (goal) => {
    const found = root.find((node) => (node.tag === 'tr' || node.tag === 'li') && String(node.props.className || '').startsWith('ui-row') && textOf(node).includes(goal));
    assert.equal(found.length, 1, `one row ${goal}: ${root.text()}`);
    return found[0];
  };
  const first = 'objectives.Evidence|' + JSON.stringify({ objective: 'o-1' });
  const second = 'objectives.Evidence|' + JSON.stringify({ objective: 'o-2' });
  row('First goal').props.onClick();
  await settle();
  assert.equal(reads[first], 1, `o-1's expand reads: ${JSON.stringify(reads)}`);
  row('Second goal').props.onClick();
  await settle();
  assert.equal(reads[second], 1, `o-2's expand reads: ${JSON.stringify(reads)}`);
  row('First goal').props.onClick();
  await settle();
  assert.equal(reads[first], 2, `o-1's expand shown again reads again: ${JSON.stringify(reads)}`);
",
    );
}

const SHARED: &str = r"
format: ess-ui/1
app: console
model: console.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: board
  sections: [{name: all, pages: [board]}]
channels:
  progress:
    carries: {events: [objectives.EvidenceAdded]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  board:
    kind: list_page
    title: Board
    header:
      title: Board
      metrics:
        - name: evidence_count
          component: metric
          reads: {view: objectives.Evidence}
          aggregate: count
          label: Evidence count
          live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}
    sections:
      - name: evidence
        component: collection
        reads: {view: objectives.Evidence, key: evidence_id}
        columns: [note]
        live: {channel: progress, on: [objectives.EvidenceAdded], effect: patch_row}
";

/// React: the header metric applies its own `effect: insert_top` though a patching section reads
/// the same view.
#[test]
fn react_a_header_metric_sharing_a_patching_sections_read_applies_its_own_effect() {
    run(
        "shared-patch",
        SHARED,
        "Board",
        r"
  await settle();
  const metric = () => {
    const found = root.find((node) => node.tag === 'div' && node.props.className === 'ui-metric' && textOf(node).includes('Evidence count'));
    assert.equal(found.length, 1, root.text());
    return textOf(found[0].children.find((child) => child.props && child.props.className === 'ui-metric-value'));
  };
  assert.equal(metric(), '3');
  play('objectives.EvidenceAdded', { evidence_id: 'e-4', note: 'Fourth benchmark' });
  await settle();
  assert.equal(metric(), '4', 'the header metric inserts the new row');
",
    );
}
