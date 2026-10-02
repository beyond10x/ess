//! The generated predicate stays outside read identity, with the same fail-closed grammar as Rust.
mod support;
use std::path::{Path, PathBuf};

const DOCUMENT: &str = r"format: ess-ui/1
app: filters
model: t
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
pages:
  p:
    kind: detail_page
    title: Filters
    sections:
      - {name: list, component: collection, reads: {view: t.All, filter: row.group == one}, columns: [id]}
";
fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-read-filter")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).unwrap();
    }
    ess_ui_react::generate(&ess_ui::load_str(DOCUMENT).unwrap(), Path::new("."), &out).unwrap();
    out
}
#[test]
fn a_section_filters_after_its_raw_read() {
    let out = project("emission");
    let page = std::fs::read_to_string(out.join("src/pages/P.tsx")).unwrap();
    assert!(
        page.contains("row.group == one"),
        "the emitted section lost its predicate: {page}"
    );
    assert!(
        page.contains("filterRead(__read"),
        "the lifecycle must receive kept rows: {page}"
    );
}
#[test]
fn generated_filter_matches_the_rust_predicate_and_preserves_raw_rows() {
    let out = project("parity");
    support::compile(&out, &["runtime/expr.ts", "runtime/data.ts"]);
    let cases = [
        "row.group == one",
        "not (row.group != one)",
        "row.group in [one, two] and row.count != 0",
        "state.pick == null or row.group == state.pick",
        "not row.text",
        "row.count == 1",
        "row.count == '1'",
        "row.group ==",
        "row.group",
        "actor.id == row.id",
        "matches(params)",
        "row.group in state.groups",
        "false or true",
        "not false",
        "row.count == -1",
        "state == null",
        "row.group in {one, two}",
        "row.group == and",
        "row.group in [one,]",
        "row.count == 1e0",
    ];
    let scope: serde_yaml::Value = serde_yaml::from_str(
        "{row: {group: one, count: 1, text: 'false'}, state: {pick: null, groups: [one]}}",
    )
    .unwrap();
    let expected: Vec<_> = cases
        .iter()
        .map(|text| {
            ess_ui::filter::keeps(text, &|path| {
                let mut value = &scope;
                for segment in path {
                    value = &value[segment.as_str()];
                }
                value.clone()
            })
        })
        .collect();
    let script = format!(
        r"
const {{ keeps, evaluate }} = require('./test-out/src/runtime/expr');
assert.equal(evaluate('-1', {{}}), '-1', 'filter grammar must not change existing display expressions');
const {{ filterRead }} = require('./test-out/src/runtime/data');
assert.equal(typeof keeps, 'function', 'filters need a fail-closed evaluator');
const scope = {scope};
assert.deepEqual({cases}.map(text => keeps(text, scope)), {expected});
const raw = {{rows: [{{id: 'a', group: 'one'}}, {{id: 'b', group: 'two'}}], total: 99, params: {{}}, loading: false}};
assert.deepEqual(filterRead(raw, 'row.group == one', {{}}).rows.map(row => row.id), ['a']);
assert.equal(filterRead(raw, 'row.group == one', {{}}).total, 1);
assert.equal(raw.rows.length, 2);
assert.equal(filterRead(raw, undefined, {{}}), raw);
assert.deepEqual(filterRead(raw, 'row.group == state.pick', {{state: {{pick: 'two'}}}}).rows.map(row => row.id), ['b']);
",
        scope = serde_json::to_string(&scope).unwrap(),
        cases = serde_json::to_string(&cases).unwrap(),
        expected = serde_json::to_string(&expected).unwrap()
    );
    support::node(&out, "predicates", &script);
}

#[test]
fn two_filters_share_one_request_and_state_changes_do_not_read_again() {
    let out = project("sharing");
    support::compile(
        &out,
        &[
            "runtime/data.ts",
            "runtime/core.tsx",
            "runtime/composites/collection.tsx",
        ],
    );
    support::node(
        &out,
        "sharing",
        r"
(async () => {
  const { setDataAdapter, useRead, filterRead } = out('runtime/data');
  let calls = 0;
  setDataAdapter({ read: async request => {
    calls++;
    assert.equal(request.filter, undefined);
    return { rows: [{id: 'a', group: 'one'}, {id: 'b', group: 'two'}], total: 99 };
  }});
  let pick = 'one', paramPick = 'one';
  let first, second, third;
  function Reader({ other }) {
    const spec = { view: 't.All', filter: other ? 'row.group == two' : 'row.group == state.pick' };
    const scope = {state: {pick}};
    const value = filterRead(useRead(spec, scope), spec.filter, scope);
    if (other) second = value; else first = value;
    return null;
  }
  function ParamReader() {
    const scope = {params:{id:paramPick}};
    third = filterRead(useRead({view:'t.All'}, scope), 'row.group == params.id', scope);
    return null;
  }
  const root = React.__root([jsx(Reader, {other: false}), jsx(Reader, {other: true}), jsx(ParamReader, {})]);
  root.settle();
  await new Promise(resolve => setTimeout(resolve, 20));
  root.settle();
  assert.equal(calls, 1, 'two predicates must share the raw adapter request');
  assert.deepEqual(first.rows.map(row => row.id), ['a']);
  assert.deepEqual(second.rows.map(row => row.id), ['b']);
  pick = 'two'; root.settle();
  assert.deepEqual(first.rows.map(row => row.id), ['b']);
  assert.equal(calls, 1, 'state changes only refilter the held rows');
  assert.deepEqual(third.rows.map(row => row.id), ['a']);
  paramPick = 'two'; root.settle();
  assert.deepEqual(third.rows.map(row => row.id), ['b']);
  assert.equal(calls, 1, 'filter-only page params rerender held rows without reading again');
})().catch(error => { console.error(error); process.exitCode = 1; });
",
    );
}

#[test]
fn live_filter_judges_post_event_rows_retains_hidden_rows_and_counts_matching_inserts() {
    let out = project("live");
    let source = DOCUMENT.replace("pages:\n  p:", "channels: {updates: {carries: {events: [t.Changed]}, direction: server_to_client, delivery: every_event, resume: refetch}}\npages:\n  p:")
        .replace("columns: [id]}", "columns: [id], live: {channel: updates, effect: insert_or_patch, when_paged_away: count_new}}");
    ess_ui_react::generate(&ess_ui::load_str(&source).unwrap(), Path::new("."), &out).unwrap();
    support::compile(&out, &["runtime/live.ts", "runtime/data.ts"]);
    support::node(
        &out,
        "live",
        r"
const {useLive, useNetworkChannels} = out('runtime/live');
const {filterRead} = out('runtime/data');
const {keeps} = out('runtime/expr');
let source;
global.EventSource = class { constructor() { source = this; } close() {} };
useNetworkChannels('http://app.test');
const rows = [{id: 'a', group: 'one', label: 'Alpha'}, {id: 'b', group: 'two', label: 'Beta'}];
let page = 1, raw, shown;
function Reader() {
  raw = useLive({status: 'ready', rows, params: {page}, refetch() {}}, {channel: 'updates', effect: 'insert_or_patch', onlyIf: 'row.label != blocked', whenPagedAway: 'count_new'}, undefined, row => keeps('row.group == one', {row}));
  shown = filterRead(raw, 'row.group == one', {});
  return null;
}
const root = React.__root(jsx(Reader, {})).settle();
const event = payload => { source.onmessage({data: JSON.stringify({event: 't.Changed', payload})}); root.settle(); };
event({id:'a', group:'two'});
assert.equal(shown.rows.length, 0);
assert.equal(raw.rows.find(row => row.id === 'a').label, 'Alpha');
event({id:'a', group:'one'});
assert.deepEqual(shown.rows.map(row => row.id), ['a']);
page = 2; root.settle();
event({id:'c', group:'two'}); assert.equal(shown.newCount, 0);
event({id:'d', group:'one', label:'Delta'}); assert.equal(shown.newCount, 1);
event({id:'d', group:'two'}); assert.equal(shown.newCount, 0);
event({id:'d', group:'one'}); assert.equal(shown.newCount, 1);
event({id:'e', group:'one', label:'blocked'}); assert.equal(shown.newCount, 1);
",
    );
}

#[test]
fn filter_removes_selection_before_a_bulk_action_can_run() {
    let out = project("selection");
    support::compile(
        &out,
        &["runtime/core.tsx", "runtime/composites/collection.tsx"],
    );
    support::node(
        &out,
        "selection",
        r"
(async () => {
  browser('/p');
  const {setDataAdapter} = out('runtime/data');
  const {ScopeLayer} = out('runtime/core');
  const {Collection} = out('runtime/composites/collection');
  let pick = 'one', selected;
  setDataAdapter({read: async () => ({rows: [{id:'a', group:'one'}, {id:'b', group:'two'}]})});
  function App() { return jsx(ScopeLayer, {values:{state:{pick}}, setters: {'state.selected': value => {selected = value;}}, children: jsx(Collection, {reads:{view:'t.All', filter:'row.group == state.pick'}, selection:'multiple', selectionState:'state.selected', columns:[{name:'id', field:'id'}], bulkActions:[{name:'apply', label:'Apply', does:'t.Bulk'}]})}); }
  const root = React.__root(jsx(App, {})).settle();
  await new Promise(resolve => setTimeout(resolve, 20)); root.settle();
  root.find(node => node.tag === 'input')[0].props.onChange({target:{checked:true}}); root.settle();
  assert.deepEqual(selected, ['a']);
  assert(root.text().includes('Apply'));
  pick = 'two'; root.settle();
  assert.deepEqual(selected, []);
  assert(!root.text().includes('Apply'));
})().catch(error => { console.error(error); process.exitCode = 1; });
",
    );
}

#[test]
fn bound_shared_read_aborts_only_when_its_last_subscriber_leaves() {
    let out = project("bound-sharing");
    let binding = serde_json::from_str(r#"{"system":"t","names":{"t.All":"t.All"},"components":{"api":{"views":{"t.All":{"path":"/all","params":[]}},"commands":{}}}}"#).unwrap();
    ess_ui_react::generate_bound(
        &ess_ui::load_str(DOCUMENT).unwrap(),
        Path::new("."),
        &out,
        Some(&binding),
    )
    .unwrap();
    support::compile(&out, &["runtime/data.ts"]);
    support::node(
        &out,
        "bound-sharing",
        r"
(async () => {
  const {setDataAdapter, useRead} = out('runtime/data');
  const requests = [];
  setDataAdapter({read: request => { requests.push(request); return new Promise(() => {}); }});
  let first = true, second = true;
  function Reader({other}) { useRead({view:'t.All', filter: other ? 'row.group == two' : 'row.group == one'}, {}); return null; }
  function App() { return [first ? jsx(Reader, {other:false}, 'first') : null, second ? jsx(Reader, {other:true}, 'second') : null]; }
  const root = React.__root(jsx(App, {})).settle();
  await new Promise(resolve => setTimeout(resolve, 20)); root.settle();
  assert.equal(requests.length, 1);
  first = false; root.settle();
  assert.equal(requests[0].signal.aborted, false, 'one subscriber still needs the response');
  second = false; root.settle();
  assert.equal(requests[0].signal.aborted, true);
})().catch(error => { console.error(error); process.exitCode = 1; });
",
    );
}

#[test]
fn staggered_refetches_cannot_reuse_another_consumers_completed_refresh() {
    let out = project("staggered-refresh");
    support::compile(&out, &["runtime/data.ts"]);
    support::node(
        &out,
        "staggered-refresh",
        r"
(async () => {
  const {setDataAdapter, useRead} = out('runtime/data');
  let revision = 0, calls = 0, first, second;
  setDataAdapter({read: async () => { calls++; return {rows:[{id:revision}]}; }});
  function Reader({other}) { const read = useRead({view:'t.All'}, {}); if(other) second=read; else first=read; return null; }
  const root = React.__root([jsx(Reader, {other:false}), jsx(Reader, {other:true})]);
  async function settle() { root.settle(); await new Promise(resolve => setTimeout(resolve, 20)); root.settle(); }
  await settle(); assert.equal(calls, 1);
  revision=1; first.refetch(); await settle(); assert.equal(first.rows[0].id, 1);
  revision=2; second.refetch(); await settle();
  assert.equal(second.rows[0].id, 2, 'a later explicit refresh must fetch the current backend revision');
  assert.equal(calls, 3);
})().catch(error => { console.error(error); process.exitCode = 1; });
",
    );
}

#[test]
fn authorization_change_never_shares_the_previous_actors_rows() {
    let out = project("authorization");
    let binding = serde_json::from_str(r#"{"system":"t","names":{"t.All":"t.All"},"components":{"api":{"views":{"t.All":{"path":"/all","params":[]}},"commands":{}}}}"#).unwrap();
    ess_ui_react::generate_bound(
        &ess_ui::load_str(DOCUMENT).unwrap(),
        Path::new("."),
        &out,
        Some(&binding),
    )
    .unwrap();
    support::compile(&out, &["runtime/data.ts"]);
    support::node(
        &out,
        "authorization",
        r"
(async () => {
  const {setDataAdapter, setAuthorization, httpAdapter, useRead} = out('runtime/data');
  const binding = {system:'t', names:{}, components:{api:{views:{'t.All':{path:'/all',params:[]}},commands:{}}}};
  const requests=[];
  global.fetch = async (_url, request) => { requests.push(request.headers.authorization); return {status:200, text: async () => JSON.stringify({rows:[{id:request.headers.authorization}]})}; };
  setDataAdapter(httpAdapter(binding, () => 'http://app.test'));
  setAuthorization('old actor');
  let mountSecond=false, first, second;
  function Reader({other}) { const read=useRead({view:'t.All'}, {}); if(other) second=read; else first=read; return null; }
  function App() { return [jsx(Reader, {other:false}, 'first'), mountSecond ? jsx(Reader, {other:true}, 'second') : null]; }
  const root = React.__root(jsx(App, {}));
  async function settle() { root.settle(); await new Promise(resolve => setTimeout(resolve, 20)); root.settle(); }
  await settle(); assert.equal(first.rows[0].id, 'old actor');
  setAuthorization('new actor'); mountSecond=true; root.settle();
  assert.deepEqual(first.rows, [], 'already-mounted readers immediately hide previous-authority rows');
  assert.deepEqual(second.rows, []);
  await settle();
  assert.deepEqual(second.rows, [{id:'new actor'}], 'a newly mounted reader must never receive a previous actors cached rows');
  assert.deepEqual(first.rows, [{id:'new actor'}]);
  assert.deepEqual(requests, ['old actor','new actor']);
  setDataAdapter({read: async () => ({rows:[{id:'new adapter'}]})}); await settle();
  assert.deepEqual(first.rows, [{id:'new adapter'}]); assert.deepEqual(second.rows, [{id:'new adapter'}]);
})().catch(error => { console.error(error); process.exitCode = 1; });
",
    );
}

#[test]
fn stalled_poller_restarts_without_aborting_another_readers_pending_request() {
    let out = project("stalled-sharing");
    let binding = serde_json::from_str(r#"{"system":"t","names":{"t.All":"t.All"},"components":{"api":{"views":{"t.All":{"path":"/all","params":[]}},"commands":{}}}}"#).unwrap();
    ess_ui_react::generate_bound(
        &ess_ui::load_str(DOCUMENT).unwrap(),
        Path::new("."),
        &out,
        Some(&binding),
    )
    .unwrap();
    support::compile(&out, &["runtime/data.ts"]);
    support::node(
        &out,
        "stalled-sharing",
        r"
(async () => {
  const {setDataAdapter, useRead, usePoll} = out('runtime/data');
  const requests=[];
  let poll, held, finishHeld;
  setDataAdapter({read: request => { requests.push(request); return requests.length===1 ? new Promise(resolve => {finishHeld=resolve;}) : Promise.resolve({rows:[{id:'fresh'}]}); }});
  let tick, now=0;
  global.setInterval = callback => {tick=callback; return {unref(){}};};
  global.clearInterval = () => {};
  Date.now = () => now;
  function Poller() { poll=usePoll(useRead({view:'t.All'}, {}), 10); return null; }
  function Holder() { held=useRead({view:'t.All'}, {}); return null; }
  const root=React.__root([jsx(Poller, {}), jsx(Holder, {})]);
  async function settle() {root.settle(); await new Promise(resolve => setTimeout(resolve, 20)); root.settle();}
  await settle(); assert.equal(requests.length, 1);
  now=49; tick(); await settle(); assert.equal(requests.length, 1);
  now=50; tick(); await settle();
  assert.equal(requests.length, 2, 'a stalled polling subscriber must start a new request even while another subscriber holds the old one');
  assert.equal(requests[0].signal.aborted, false, 'the other subscriber still owns the original request');
  assert.deepEqual(poll.rows, [{id:'fresh'}]);
  finishHeld({rows:[{id:'original'}]}); await settle();
  assert.deepEqual(held.rows, [{id:'original'}]);
  assert.deepEqual(poll.rows, [{id:'fresh'}]);
})().catch(error => { console.error(error); process.exitCode=1; });
",
    );
}

#[test]
fn dynamic_menu_and_choice_filter_held_rows_without_changing_option_identity() {
    let out = project("menu-choice");
    let source = DOCUMENT.replace("main: {kind: page_outlet}", "main: {kind: page_outlet}, nav: {kind: navigation}")
        .replace("navigation: {home: p, sections: [{name: all, pages: [p]}]}", "navigation: {home: p, sections: [{name: all, pages: {from_view: t.All, page: p, param: id, filter: row.group == one}}]}")
        .replace("columns: [id]}", "columns: [id]}\n      - {name: pick, component: choice, reads: {view: t.All, filter: row.group == one}}");
    ess_ui_react::generate(&ess_ui::load_str(&source).unwrap(), Path::new("."), &out).unwrap();
    support::compile(
        &out,
        &["runtime/navigation.tsx", "runtime/composites/choice.tsx"],
    );
    support::node(
        &out,
        "menu-choice",
        r"
(async () => {
  browser('/p');
  const {setDataAdapter} = out('runtime/data');
  const {ScopeLayer} = out('runtime/core');
  const {NavSection} = out('runtime/navigation');
  const {ChoiceView} = out('runtime/composites/choice');
  let calls=0, filter='row.group == state.pick', picked;
  setDataAdapter({read: async () => { calls++; return {rows:[{id:'a',option:'chosen',label:'Alpha',group:'one'},{id:'b',option:'hidden',label:'Beta',group:'two'}]}; }});
  function App() { return jsx(ScopeLayer, {values:{state:{pick:'one'}}, setters:{'state.chosen': value => {picked=value;}}, children:[jsx(NavSection, {name:'items', dynamic:{fromView:'t.All',page:'p',param:'id',label:'row.label',synonyms:[],filter}}), jsx(ChoiceView, {reads:{view:'t.All',filter}, valueKey:'option', binds:'state.chosen'})]}); }
  const root=React.__root(jsx(App, {})).settle();
  await new Promise(resolve => setTimeout(resolve, 20)); root.settle();
  assert.equal(calls, 1); assert(root.text().includes('Alpha')); assert(!root.text().includes('Beta'));
  assert.equal(root.find(node => node.tag==='a').length, 1);
  root.find(node => node.tag==='select')[0].props.onChange({target:{value:'chosen'}});
  assert.equal(picked, 'chosen', 'filtering preserves the existing valueKey option value');
  filter='row.group'; root.settle();
  assert(!root.text().includes('Alpha')); assert.equal(root.find(node => node.tag==='a').length, 0);
  assert.equal(calls, 1);
})().catch(error => {console.error(error); process.exitCode=1;});
",
    );
}

#[test]
fn filtered_live_refetch_respects_event_admission() {
    let out = project("refetch-admission");
    let source = DOCUMENT.replace("pages:\n  p:", "channels: {updates: {carries: {events: [t.Changed]}, direction: server_to_client, delivery: every_event, resume: refetch}}\npages:\n  p:")
        .replace("columns: [id]}", "columns: [id], live: {channel: updates, effect: refetch, only_if: row.group == one}}");
    ess_ui_react::generate(&ess_ui::load_str(&source).unwrap(), Path::new("."), &out).unwrap();
    support::compile(&out, &["runtime/live.ts", "runtime/data.ts"]);
    support::node(
        &out,
        "refetch-admission",
        r"
const {useLive, useNetworkChannels} = out('runtime/live');
let source, calls=0;
global.EventSource = class {constructor(){source=this;} close(){}};
useNetworkChannels('http://app.test');
function Reader() {useLive({status:'ready',rows:[],params:{},refetch(){calls++;}}, {channel:'updates',effect:'refetch',onlyIf:'row.group == one'}, undefined, () => true); return null;}
const root=React.__root(jsx(Reader, {})).settle();
source.onmessage({data:JSON.stringify({event:'t.Changed',payload:{id:'a',group:'two'}})}); root.settle();
assert.equal(calls, 0, 'a filtered refetch still rejects an event that fails only_if');
source.onmessage({data:JSON.stringify({event:'t.Changed',payload:{id:'a',group:'one'}})}); root.settle();
assert.equal(calls, 1);
",
    );
}

#[test]
fn numeric_filter_equality_matches_ecmascript_text_and_canonical_values() {
    let out = project("numeric-parity");
    support::compile(&out, &["runtime/expr.ts"]);
    let cases: serde_json::Value = serde_json::from_str(
        r#"[
      [0.0000001,"1e-7",true], [0.0000001,"0.0000001",false],
      [0.000001,"0.000001",true], [1e20,"100000000000000000000",true],
      [1e21,"1e+21",true], [1e21,"1000000000000000000000",false],
      [-0.0,"0",true], [[0.0000001],"1e-7",true], [[1e21],"1e+21",true],
      [[-0.0],[0],true], [{"b":[-0.0,0.0000001],"a":1e21},{"a":1e21,"b":[0,0.0000001]},true]
    ]"#,
    )
    .unwrap();
    let actual: Vec<_> = cases
        .as_array()
        .unwrap()
        .iter()
        .map(|case| {
            // Decode JSON-shaped fixture bytes. Direct Value serialization exposes serde_json's
            // private number envelope when arbitrary_precision is unified on.
            let scope = serde_yaml::from_str(&format!(
                r#"{{"row":{{"number":{}}},"state":{{"value":{}}}}}"#,
                case[0], case[1]
            ))
            .unwrap();
            ess_ui::filter::keeps("row.number == state.value", &|path| {
                let segments: Vec<_> = path.iter().map(String::as_str).collect();
                ess_ui::filter::path(&scope, &segments)
            })
        })
        .collect();
    support::node(
        &out,
        "numeric-parity",
        &format!(
            r"
const {{keeps}} = out('runtime/expr');
const cases={cases};
const reactResults=cases.map(([number,value]) => keeps('row.number == state.value', {{row:{{number}},state:{{value}}}}));
assert.deepEqual(reactResults, cases.map(entry=>entry[2]), 'the expected values follow actual generated React equality');
assert.deepEqual({actual}, reactResults, 'Rust filters must use the same numeric text, including exponent thresholds and nested negative zero');
",
            actual = serde_json::to_string(&actual).unwrap()
        ),
    );
}

#[test]
fn adapter_change_drops_live_rows_even_when_both_raw_reads_are_empty() {
    let out = project("authority-live");
    let source = DOCUMENT.replace("pages:\n  p:", "channels: {updates: {carries: {events: [t.Changed]}, direction: server_to_client, delivery: every_event, resume: refetch}}\npages:\n  p:")
        .replace("columns: [id]}", "columns: [id], live: {channel: updates, effect: insert_or_patch}}");
    ess_ui_react::generate(&ess_ui::load_str(&source).unwrap(), Path::new("."), &out).unwrap();
    support::compile(&out, &["runtime/live.ts", "runtime/data.ts"]);
    support::node(
        &out,
        "authority-live",
        r"
(async () => {
  const {setDataAdapter,useRead} = out('runtime/data');
  const {useLive,useNetworkChannels} = out('runtime/live');
  let source, visible;
  global.EventSource = class {constructor(){source=this;} close(){}};
  useNetworkChannels('http://app.test');
  setDataAdapter({read:async () => ({rows:[]})});
  function Reader() {visible=useLive(useRead({view:'t.All'}, {}), {channel:'updates',effect:'insert_or_patch'}, undefined, () => true); return null;}
  const root=React.__root(jsx(Reader, {}));
  async function settle(){root.settle(); await new Promise(resolve=>setTimeout(resolve,20)); root.settle();}
  await settle();
  source.onmessage({data:JSON.stringify({event:'t.Changed',payload:{id:'old_actor_row'}})}); root.settle();
  assert.deepEqual(visible.rows, [{id:'old_actor_row'}]);
  setDataAdapter({read:async () => ({rows:[]})}); root.settle();
  assert.deepEqual(visible.rows, [], 'live rows from the previous source must clear even when raw rows stay empty');
  await settle(); assert.deepEqual(visible.rows, []);
})().catch(error => {console.error(error); process.exitCode=1;});
",
    );
}

#[test]
fn coalesced_event_survives_an_ordinary_raw_read_completion() {
    let out = project("coalesced-read");
    let source = DOCUMENT.replace("pages:\n  p:", "channels: {updates: {carries: {events: [t.Changed]}, direction: server_to_client, delivery: every_event, resume: refetch}}\npages:\n  p:")
        .replace("columns: [id]}", "columns: [id], live: {channel: updates, effect: insert_or_patch}}");
    ess_ui_react::generate(&ess_ui::load_str(&source).unwrap(), Path::new("."), &out).unwrap();
    support::compile(&out, &["runtime/live.ts", "runtime/data.ts"]);
    support::node(
        &out,
        "coalesced-read",
        r"
const {useLive,useNetworkChannels} = out('runtime/live');
let source, flush, raw=[], visible;
global.EventSource = class {constructor(){source=this;} close(){}};
useNetworkChannels('http://app.test');
const originalTimeout=global.setTimeout;
global.setTimeout=(callback,ms,...args) => ms===100 ? (flush=callback, 1) : originalTimeout(callback,ms,...args);
function Reader(){visible=useLive({status:'ready',rows:raw,params:{},source:1,refetch(){}}, {channel:'updates',effect:'insert_or_patch',coalesce:100}, undefined, () => true); return null;}
const root=React.__root(jsx(Reader, {})).settle();
source.onmessage({data:JSON.stringify({event:'t.Changed',payload:{id:'a',label:'new'}})});
raw=[{id:'a',label:'old'}]; root.settle();
flush(); root.settle();
assert.deepEqual(visible.rows, [{id:'a',label:'new'}], 'ordinary read completion must not discard a pending coalesced event');
",
    );
}
