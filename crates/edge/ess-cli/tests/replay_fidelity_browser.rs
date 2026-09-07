//! F15's independent declaration vectors, through both real CLI emissions and Firefox/BiDi.
#[path = "support/browser.rs"]
mod browser;
use serde_json::{json, Value};
use std::{fmt::Write as _, fs, path::PathBuf, process::Command};

const ENTITY: &str = "replay.items.Item";
const LITERAL: &str = "Unknown: assignment literal is absent from this replay projection.";
const CONVERSION: &str =
    "Unknown: assignment types/conversion are absent from this replay projection.";
const SUBJECT: &str = "Unknown: subject identity source is absent from this replay projection.";
const PARAMETER: &str = "Unknown: this replay does not evaluate parameterized view results.";
const FILTER: &str = "Unknown: this replay does not evaluate this view filter.";
const SPEC: &str = r"format: ess/1
system: replay
version: v1
domain: replay.items
types:
  - {name: replay.items.Id, kind: newtype, of: String}
  - {name: replay.items.Source, kind: newtype, of: String}
  - {name: replay.items.Destination, kind: newtype, of: String}
  - name: replay.items.Detail
    kind: struct
    fields: [{name: score, type: Integer}]
conversions:
  - {from: replay.items.Source, to: replay.items.Destination, because: explicit crossing}
entities:
  - name: replay.items.Item
    identity: {name: id, type: replay.items.Id}
    fields:
      - {name: label, type: String}
      - {name: rank, type: Integer}
      - {name: copied, type: replay.items.Destination}
      - {name: bool_text, type: String}
      - {name: number_text, type: String}
      - {name: null_text, type: String}
      - {name: empty_text, type: String}
      - {name: ordinary_text, type: String}
      - {name: detail, type: replay.items.Detail}
    lifecycle:
      initial: Draft
      states: [Draft, Done]
      terminal: [Done]
      transitions: [{name: finish, from: [Draft], to: Done}]
events:
  - name: replay.items.Created
    fields: [{name: id, type: replay.items.Id}, {name: label, type: String}]
  - name: replay.items.Finished
    fields: [{name: id, type: replay.items.Id}]
errors:
  - {name: replay.items.Conflict}
commands:
  - name: replay.items.Create
    input:
      - {name: label, type: String}
      - {name: rank, type: Integer}
      - {name: copied, type: replay.items.Source}
      - {name: bool_text, type: String}
      - {name: number_text, type: String}
      - {name: null_text, type: String}
      - {name: empty_text, type: String}
      - {name: ordinary_text, type: String}
      - {name: null_value, type: Optional<String>}
      - {name: boolean_value, type: Boolean}
      - {name: zero_value, type: Integer}
      - {name: empty_value, type: String}
      - {name: false_value, type: String}
      - {name: numeric_value, type: String}
      - {name: list_value, type: 'List<Integer>'}
      - {name: mapping_value, type: 'Map<String, String>'}
    outcomes:
      - name: created
        creates: replay.items.Item
        instance: id
        emits: [replay.items.Created]
        payload:
          replay.items.Created: {label: input.label}
        sets:
          label: input.label
          rank: input.rank
          copied: input.copied
          bool_text: 'false'
          number_text: '0'
          null_text: 'null'
          empty_text: ''
          ordinary_text: 'stored'
  - name: replay.items.Finish
    input:
      - {name: a, type: replay.items.Id}
      - {name: z, type: replay.items.Id}
      - {name: literal_id, type: replay.items.Id}
      - {name: label, type: String}
      - {name: rank, type: Integer}
    outcomes:
      - name: finished
        moves: replay.items.Item.finish
        instance: z
        emits: [replay.items.Finished]
        sets: {label: input.label, rank: input.rank}
      - {name: wrong-state, wrong_state: true, error: replay.items.Conflict}
views:
  - name: replay.items.Ranked
    source: replay.items.Item
    consistency: read_your_writes
    order_by: [rank desc]
    fields: [{name: id, type: replay.items.Id}, {name: rank, type: Integer}]
  - name: replay.items.ByLabel
    source: replay.items.Item
    consistency: read_your_writes
    params: [{name: wanted, type: String}]
    filter: label == param.wanted
    fields: [{name: id, type: replay.items.Id}, {name: label, type: String}]
  - name: replay.items.Filtered
    source: replay.items.Item
    consistency: read_your_writes
    filter: rank >= 0
    fields: [{name: id, type: replay.items.Id}, {name: rank, type: Integer}]
";
fn scenario(swapped: bool) -> String {
    let mut text = String::from("type: ess-scenario/1\ndomain: replay.items\nscenario: declarations\nsummary: Independent replay declarations.\narrange:\n  - {instance: first, entity: replay.items.Item}\n  - {instance: second, entity: replay.items.Item}\ntimeline:\n");
    for (time, alias, rank, label) in [(0, "first", 1, "alpha"), (1, "second", 2, "beta")] {
        write!(
            text,
            r"  - at: 2026-01-05T09:00:0{time}Z
    command: replay.items.Create
    input:
      label: {label}
      rank: {rank}
      copied: source-value
      bool_text: decoy-boolean
      number_text: decoy-number
      null_text: decoy-null
      empty_text: decoy-empty
      ordinary_text: decoy-ordinary
      null_value: null
      boolean_value: false
      zero_value: 0
      empty_value: ''
      false_value: 'false'
      numeric_value: '0'
      list_value: [0, 1]
      mapping_value: {{instance: literal-alias}}
    outcome: created
    events: [{{event: replay.items.Created, payload: {{label: {label}}}}}]
    capture: {{instance: {alias}, event: replay.items.Created, field: id}}
"
        )
        .unwrap();
    }
    let (a, z) = if swapped {
        ("second", "first")
    } else {
        ("first", "second")
    };
    write!(
        text,
        r"  - at: 2026-01-05T09:00:02Z
    command: replay.items.Finish
    input:
      a: {{$instance: {a}}}
      z: {{$instance: {z}}}
      literal_id: real-looking-id
      label: changed
      rank: 7
    outcome: finished
assert:
  - view: replay.items.ByLabel
    params: {{wanted: alpha}}
    contains: {{id: {{$instance: first}}}}
  - view: replay.items.ByLabel
    params: {{wanted: absent}}
    counts: {{at_least: 0, at_most: 0}}
"
    )
    .unwrap();
    text
}

struct Fixture {
    evidence: PathBuf,
    site: PathBuf,
    model: Value,
    suite: Value,
    replay: Option<Value>,
}
impl Fixture {
    fn emit(name: &str, route: u8, spec: &str, authored: &str) -> Self {
        let evidence =
            std::env::temp_dir().join(format!("ess-replay-{name}-{route}-{}", std::process::id()));
        fs::create_dir(&evidence).unwrap();
        let source = evidence.join("system.yaml");
        let scenarios = evidence.join("scenario.yaml");
        fs::write(&source, spec).unwrap();
        fs::write(&scenarios, authored).unwrap();
        let generated = evidence.join("site");
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_ess"));
        cmd.args(["conform", "web", "--path"])
            .arg(&source)
            .arg("--scenarios")
            .arg(&scenarios)
            .args(["--suite-format", &route.to_string(), "--out"])
            .arg(&generated);
        fs::write(evidence.join("emit.command"), format!("{cmd:?}\n")).unwrap();
        let output = cmd.output().unwrap();
        fs::write(
            evidence.join("emit.exit"),
            format!("{:?}\n", output.status.code()),
        )
        .unwrap();
        fs::write(evidence.join("emit.stdout"), &output.stdout).unwrap();
        fs::write(evidence.join("emit.stderr"), &output.stderr).unwrap();
        assert!(
            output.status.success(),
            "{name}/{route}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let (model, suite, replay): (Value, Value, Option<Value>) = if route == 5 {
            let replay: Value =
                serde_json::from_slice(&fs::read(generated.join("replay.json")).unwrap()).unwrap();
            ess_conformance::web_replay::AdmittedReplay::from_json(&replay.to_string()).unwrap();
            (
                replay["model"].clone(),
                serde_json::from_str(replay["input"]["suite_json"].as_str().unwrap()).unwrap(),
                Some(replay),
            )
        } else {
            (
                serde_json::from_slice(&fs::read(generated.join("model.json")).unwrap()).unwrap(),
                serde_json::from_slice(&fs::read(generated.join("suite.json")).unwrap()).unwrap(),
                None,
            )
        };
        fs::write(evidence.join("emitted-model.json"), model.to_string()).unwrap();
        fs::write(evidence.join("emitted-suite.json"), suite.to_string()).unwrap();
        Self {
            evidence,
            site: generated,
            model,
            suite,
            replay,
        }
    }
    fn steps(&mut self) -> &mut Vec<Value> {
        self.suite["scenarios"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .next()
            .unwrap()["steps"]
            .as_array_mut()
            .unwrap()
    }
    // Only persisted control vectors use this route. Keep IDs, source inventory and coverage;
    // recompute the exact suite-byte reference and ask the real closed Rust readers to admit it.
    fn persist(&self) {
        let original = self.suite.to_string();
        let admitted = ess_conformance::admission::AdmittedSuite::from_json(&original).unwrap();
        if let Some(mut replay) = self.replay.clone() {
            let input = ess_conformance::coverage::AdmittedInput::from_suite(admitted).unwrap();
            replay["input"] = serde_json::to_value(input.document()).unwrap();
            replay["suite"] = serde_json::to_value(ess_conformance::coverage::SuiteReference::of(
                input.selected(),
            ))
            .unwrap();
            let original = replay.to_string();
            ess_conformance::web_replay::AdmittedReplay::from_json(&original).unwrap();
            fs::write(self.site.join("replay.json"), &original).unwrap();
            fs::write(self.evidence.join("persisted-replay.json"), original).unwrap();
        } else {
            fs::write(self.site.join("suite.json"), &original).unwrap();
        }
        fs::write(self.evidence.join("persisted-suite.json"), original).unwrap();
    }
    fn browse(&self, script: &str) -> Value {
        let server = browser::Server::new(&self.site);
        let mut browser = browser::Browser::new(&self.evidence);
        let context = browser.open(&format!("{}/index.html", server.url));
        let expression = format!(
            r"(async()=>{{
          const {{default:p}}=await import('./player.js');
          const {{nextTick}}=await import('./assets/vue.esm-browser.prod.js');
          const click=async(label)=>{{const b=[...document.querySelectorAll('button')].find(b=>b.textContent.trim().startsWith(label));if(!b)throw Error('missing '+label);b.click();await nextTick();}};
          const snapshot=()=>JSON.parse(JSON.stringify({{world:p.state.world,cursor:p.state.cursor,changes:p.state.lastChanges,acts:p.scenarios[0].acts,views:p.liveViews.value,dom:document.getElementById('app').textContent,boom:document.getElementById('boom').textContent}}));
          const all=async()=>{{for(let i=0;i<p.scenarios[0].acts.length;i++)await click('Step');}};
          {script}
        }})()"
        );
        fs::write(self.evidence.join("probe.js"), &expression).unwrap();
        let result = browser.evaluate(&context, &expression);
        fs::write(
            self.evidence.join("result.json"),
            serde_json::to_string_pretty(&result).unwrap(),
        )
        .unwrap();
        result
    }
}

// The model emits admitted Node numbers as e.g. 1.0; JSON.stringify emits the same JS Number as 1.
// Normalize only inside Literal.value in these small finite vectors, never metadata or strings.
fn normalize_literal_numbers(value: &mut Value) {
    fn node(value: &mut Value) {
        match value {
            Value::Number(n) if n.is_f64() => {
                let v = n.as_f64().unwrap();
                assert!(v.is_finite() && v.abs() < 100.0 && v.fract() == 0.0);
                *value = serde_json::from_str(&v.to_string()).unwrap();
            }
            Value::Array(items) => items.iter_mut().for_each(node),
            Value::Object(fields) => fields.values_mut().for_each(node),
            _ => {}
        }
    }
    if value["kind"] == "literal" {
        node(&mut value["value"]);
        return;
    }
    match value {
        Value::Array(items) => items.iter_mut().for_each(normalize_literal_numbers),
        Value::Object(fields) => fields.values_mut().for_each(normalize_literal_numbers),
        _ => {}
    }
}

fn contains(result: &Value, text: &str) {
    assert!(
        result["dom"].as_str().unwrap().contains(text),
        "missing {text}: {result}"
    );
}
fn unknown_fields(result: &Value, alias: &str, fields: &[&str]) {
    let inst = &result["world"]["instances"][alias];
    assert!(inst.is_object(), "{result}");
    for field in fields {
        assert!(
            inst["fields"].get(*field).is_none(),
            "invented {alias}.{field}: {result}"
        );
    }
    assert_eq!(result["boom"], "", "render failed: {result}");
}

#[test]
fn b01_capture_establishes_only_local_alias_and_initial_state() {
    for route in [4, 5] {
        let f = Fixture::emit("b01", route, SPEC, &scenario(false));
        let r = f.browse("await click('Step');return JSON.stringify(snapshot());");
        assert_eq!(r["world"]["instances"]["first"]["state"], "Draft");
        unknown_fields(&r, "first", &["id", "label", "rank", "detail"]);
        contains(&r, "first");
        contains(&r, "Draft");
        contains(&r, "Unknown");
        contains(&r, "detail");
    }
}
#[test]
fn b02_move_processes_every_set_without_guessing_subject() {
    for route in [4, 5] {
        let f = Fixture::emit("b02", route, SPEC, &scenario(false));
        let r = f.browse("await all();return JSON.stringify(snapshot());");
        for alias in ["first", "second"] {
            unknown_fields(&r, alias, &["label", "rank"]);
            assert!(r["world"]["instances"][alias].get("state").is_none(), "{r}");
        }
        contains(&r, SUBJECT);
        contains(&r, CONVERSION);
        contains(&r, "finish");
        let effects = r["world"]["unknownEffects"].as_array().unwrap();
        assert!(effects
            .iter()
            .any(|e| e["field"] == "label" && e["at"] == 2));
        assert!(effects.iter().any(|e| e["field"] == "rank" && e["at"] == 2));
    }
}
#[test]
fn b03_missing_literals_never_copy_same_named_decoys() {
    for route in [4, 5] {
        let f = Fixture::emit("b03", route, SPEC, &scenario(false));
        let r = f.browse("await click('Step');return JSON.stringify(snapshot());");
        unknown_fields(
            &r,
            "first",
            &[
                "bool_text",
                "number_text",
                "null_text",
                "empty_text",
                "ordinary_text",
            ],
        );
        contains(&r, LITERAL);
        let effects = r["world"]["unknownEffects"].as_array().unwrap();
        for field in [
            "bool_text",
            "number_text",
            "null_text",
            "empty_text",
            "ordinary_text",
        ] {
            assert!(
                effects
                    .iter()
                    .any(|e| e["field"] == field && e["reason"] == LITERAL),
                "{r}"
            );
        }
    }
}
#[test]
fn b04_typed_literals_are_retained_and_literal_mapping_is_no_reference() {
    for route in [4, 5] {
        let f = Fixture::emit("b04", route, SPEC, &scenario(false));
        let r = f.browse("await click('Step');return JSON.stringify(snapshot());");
        for (field, value) in [
            ("null_value", json!(null)),
            ("boolean_value", json!(false)),
            ("zero_value", json!(0)),
            ("empty_value", json!("")),
            ("false_value", json!("false")),
            ("numeric_value", json!("0")),
            ("list_value", json!([0, 1])),
            ("mapping_value", json!({"instance":"literal-alias"})),
        ] {
            assert_eq!(
                r["acts"][0]["input"][field],
                json!({"kind":"literal","value":value})
            );
            contains(&r, field);
            contains(&r, &format!("literal {value}"));
        }
        assert!(r["world"]["instances"].get("literal-alias").is_none());
        contains(&r, "literal");
    }
}
#[test]
fn b05_different_conversion_models_have_same_unknown_reduced_assignment() {
    for route in [4, 5] {
        let mut shapes = Vec::new();
        for (name, spec) in [
            ("conversion", SPEC.to_string()),
            (
                "identity",
                SPEC.replace(
                    "{name: copied, type: replay.items.Source}",
                    "{name: copied, type: replay.items.Destination}",
                ),
            ),
        ] {
            let f = Fixture::emit(&format!("b05-{name}"), route, &spec, &scenario(false));
            shapes.push(f.model["commands"][0]["outcomes"][0]["sets"].clone());
            let r = f.browse("await click('Step');return JSON.stringify(snapshot());");
            unknown_fields(&r, "first", &["copied"]);
            contains(&r, CONVERSION);
        }
        assert_eq!(shapes[0], shapes[1]);
    }
}
#[test]
fn b06_swapped_ordered_alias_vectors_never_select_a_subject() {
    for route in [4, 5] {
        let mut vectors = Vec::new();
        for swapped in [false, true] {
            let mut f = Fixture::emit(&format!("b06-{swapped}"), route, SPEC, &scenario(swapped));
            let input = &f
                .steps()
                .iter()
                .find(|s| s["command"] == "replay.items.Finish")
                .unwrap()["input"];
            let vector: Vec<_> = input
                .as_object()
                .unwrap()
                .iter()
                .filter(|(_, v)| v["kind"] == "instance")
                .map(|(k, v)| json!([k, v["kind"], v["instance"]]))
                .collect();
            assert_eq!(
                vector,
                if swapped {
                    vec![
                        json!(["a", "instance", "second"]),
                        json!(["z", "instance", "first"]),
                    ]
                } else {
                    vec![
                        json!(["a", "instance", "first"]),
                        json!(["z", "instance", "second"]),
                    ]
                }
            );
            assert_eq!(
                input["literal_id"],
                json!({"kind":"literal","value":"real-looking-id"})
            );
            vectors.push(vector);
            let r = f.browse("await all();return JSON.stringify(snapshot());");
            contains(&r, SUBJECT);
            assert_eq!(r["world"]["instances"].as_object().unwrap().len(), 2);
            for alias in ["first", "second"] {
                assert!(r["world"]["instances"][alias].get("state").is_none(), "{r}");
            }
        }
        assert_ne!(vectors[0], vectors[1]);
    }
}
#[test]
fn b07_both_model_rank_directions_have_no_computed_rows() {
    for route in [4, 5] {
        for order in ["asc", "desc"] {
            let f = Fixture::emit(
                &format!("b07-{order}"),
                route,
                &SPEC.replace("rank desc", &format!("rank {order}")),
                &if order == "asc" {
                    scenario(false).replace("rank: 1", "rank: 9")
                } else {
                    scenario(false)
                },
            );
            let r = f.browse("await all();await click('Views');return JSON.stringify(snapshot());");
            assert_eq!(r["views"].as_array().unwrap().len(), 3);
            for view in r["views"].as_array().unwrap() {
                assert!(view.get("rows").is_none(), "{r}");
            }
            contains(&r, "ordering was not projected");
            assert!(!r["dom"].as_str().unwrap().contains("no rows match"));
        }
    }
}
#[test]
fn b08_supplied_and_unresolved_query_parameters_remain_unknown() {
    for route in [4, 5] {
        let mut f = Fixture::emit("b08", route, SPEC, &scenario(false));
        f.steps().push(json!({"step":"query_view","view":"replay.items.ByLabel","params":{"wanted":{"kind":"observed","event":"replay.items.Created","field":"label"}}}));
        f.persist();
        let r = f.browse("await all();await click('Views');return JSON.stringify(snapshot());");
        contains(&r, PARAMETER);
        contains(&r, "wanted");
        contains(&r, "alpha");
        contains(&r, "absent");
        contains(&r, "observed");
        assert_eq!(r["world"]["events"], json!([]));
        for view in r["views"].as_array().unwrap() {
            assert!(view.get("rows").is_none());
        }
    }
}
#[test]
fn b09_filters_are_visible_without_partial_evaluation() {
    for route in [4, 5] {
        for (i, filter) in [
            "{all: ['rank >= 0', 'rank < 5']}",
            "{any: ['rank >= 0', 'rank < 5']}",
            "rank > 0",
            "detail.score >= 1",
        ]
        .iter()
        .enumerate()
        {
            let f = Fixture::emit(
                &format!("b09-{i}"),
                route,
                &SPEC.replace("filter: rank >= 0", &format!("filter: {filter}")),
                &scenario(false),
            );
            let r = f.browse("await all();await click('Views');return JSON.stringify(snapshot());");
            let expected = [
                "(rank >= 0 and rank < 5)",
                "(rank >= 0 or rank < 5)",
                "rank > 0",
                "detail.score >= 1",
            ][i];
            assert_eq!(
                f.model["views"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|v| v["name"] == "replay.items.Filtered")
                    .unwrap()["filter"],
                expected
            );
            contains(&r, expected);
            contains(&r, FILTER);
            assert!(r["views"]
                .as_array()
                .unwrap()
                .iter()
                .all(|v| v.get("rows").is_none()));
        }
    }
}
#[test]
fn b10_unbound_effects_and_zero_candidates_are_unknown() {
    for route in [4, 5] {
        let mut f = Fixture::emit("b10", route, SPEC, &scenario(false));
        f.steps().retain(|s| s["step"] != "capture_instance");
        f.persist();
        let r=f.browse("await click('Views');const empty=snapshot();await all();const after=snapshot();await click('State');return JSON.stringify({...snapshot(),empty,after});");
        for key in ["empty", "after"] {
            contains(&r[key], PARAMETER);
            contains(&r[key], "ordering was not projected");
            assert_eq!(r[key]["views"].as_array().unwrap().len(), 3);
        }
        assert_eq!(r["world"]["instances"], json!({}));
        contains(&r, "Unknown");
        assert!(!r["dom"].as_str().unwrap().contains("Nothing exists"));
        assert!(
            r["views"]
                .as_array()
                .unwrap()
                .iter()
                .all(|v| v.get("rows").is_none()),
            "{r}"
        );
    }
}
#[test]
fn b11_original_controls_and_expectations_are_unexecuted() {
    for route in [4, 5] {
        let mut f = Fixture::emit("b11", route, SPEC, &scenario(false));
        f.steps()
            .insert(0, json!({"step":"mark_instant","instant":"start"}));
        for s in [
            json!({"step":"redeliver_event","event":"replay.items.Created"}),
            json!({"step":"eventually_event","event":"replay.items.Created"}),
            json!({"step":"expect_not_before","instant":"start","elapsed":1}),
            json!({"step":"expect_within","instant":"start","elapsed":2}),
            json!({"step":"expect_quiet","instant":"start","elapsed":1,"event":"replay.items.Created"}),
            json!({"step":"expect_halt","view":"replay.items.Ranked","after":1}),
            json!({"step":"eventually_halt","view":"replay.items.Ranked","after":1}),
            json!({"step":"eventually_view","view":"replay.items.Ranked","expectation":{"expect":"counts","at_least":0,"at_most":0}}),
        ] {
            f.steps().push(s);
        }
        let mut expected = f.steps().clone();
        for step in &mut expected {
            normalize_literal_numbers(step);
            if route == 5 {
                for path in [
                    "/elapsed",
                    "/after",
                    "/expectation/at_least",
                    "/expectation/at_most",
                    "/expectation/position/index",
                ] {
                    if let Some(value) = step.pointer_mut(path) {
                        *value = json!({"raw":value.as_u64().unwrap().to_string()});
                    }
                }
            }
        }
        f.persist();
        let r = f.browse("await all();return JSON.stringify(snapshot());");
        let actual: Vec<_> = r["acts"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|a| {
                a["steps"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|s| s["declaration"].clone())
            })
            .collect();
        assert_eq!(actual, expected);
        assert_eq!(r["world"]["events"], json!([]));
        contains(&r, "Unexecuted");
        contains(&r, "\"after\":1");
        contains(&r, "\"elapsed\":2");
        for kind in [
            "mark_instant",
            "redeliver_event",
            "eventually_event",
            "expect_not_before",
            "expect_within",
            "expect_quiet",
            "expect_halt",
            "eventually_halt",
            "eventually_view",
        ] {
            contains(&r, kind);
        }
        assert!(!r["dom"].as_str().unwrap().contains('✓'));
    }
}
#[test]
fn b12_controls_reconstruct_prefixes_and_cancel_stale_playback() {
    for route in [4, 5] {
        let f = Fixture::emit("b12", route, SPEC, &scenario(false));
        let r=f.browse(r"
          await click('Step');const first=snapshot();await click('Step');const second=snapshot();
          await click('Step');const moved=snapshot();await click('◂ Back');const back=snapshot();await click('Step');const replayed=snapshot();
          await click('Reset');const reset=snapshot();p.state.speed=30;await click('▶ Play');await click('Reset');
          await new Promise(r=>setTimeout(r,100));const afterReset=snapshot();
          await click('▶ Play');document.querySelector('.scn').click();await nextTick();await new Promise(r=>setTimeout(r,100));const afterSelect=snapshot();
          return JSON.stringify({first,second,moved,back,replayed,reset,afterReset,afterSelect});
        ");
        assert_eq!(r["second"], r["back"]);
        assert_eq!(r["moved"], r["replayed"]);
        for key in ["reset", "afterReset", "afterSelect"] {
            assert_eq!(r[key]["cursor"], -1);
            assert_eq!(r[key]["world"]["instances"], json!({}));
            assert_eq!(r[key]["world"]["events"], json!([]));
            assert_eq!(r[key]["world"]["notes"], json!([]));
            assert_eq!(r[key]["world"]["unknownEffects"], json!([]));
        }
        assert_eq!(r["first"]["world"]["instances"]["first"]["entity"], ENTITY);
    }
}

#[test]
fn b02_unknown_writes_invalidate_existing_values_and_preserve_unrelated_facts() {
    for route in [4, 5] {
        let f = Fixture::emit("b02-invalidation", route, SPEC, &scenario(false));
        // A skin can retain knowledge through the public reactive API. An unresolved write must
        // remove affected values even in that case, while preserving a field the effect never sets.
        let r=f.browse(r"await click('Step');await click('Step');
          for(const i of Object.values(p.state.world.instances)){i.fields.label='prior';i.fields.rank=99;i.fields.detail={score:42};}
          await nextTick();const before=snapshot();await click('Step');return JSON.stringify({before,after:snapshot()});");
        for alias in ["first", "second"] {
            assert_eq!(
                r["before"]["world"]["instances"][alias]["fields"]["label"],
                "prior"
            );
            unknown_fields(&r["after"], alias, &["label", "rank"]);
            assert_eq!(
                r["after"]["world"]["instances"][alias]["fields"]["detail"],
                json!({"score":42})
            );
        }
        contains(&r["after"], SUBJECT);
        contains(&r["after"], CONVERSION);
    }
}
#[test]
fn b01_conflicting_captures_outcomes_and_missing_assignment_input_are_diagnostic() {
    for route in [4, 5] {
        for mode in [
            "duplicate-capture",
            "wrong-entity",
            "duplicate-outcome",
            "missing-input",
        ] {
            let mut f = Fixture::emit(&format!("b01-{mode}"), route, SPEC, &scenario(false));
            let steps = f.steps();
            match mode {
                "duplicate-capture" => {
                    let i = steps
                        .iter()
                        .position(|s| s["step"] == "capture_instance")
                        .unwrap();
                    steps.insert(i, steps[i].clone());
                }
                "wrong-entity" => {
                    steps
                        .iter_mut()
                        .find(|s| s["step"] == "capture_instance")
                        .unwrap()["entity"] = json!("replay.items.Other");
                }
                "duplicate-outcome" => {
                    let i = steps
                        .iter()
                        .position(|s| s["step"] == "expect_outcome")
                        .unwrap();
                    steps.insert(i, steps[i].clone());
                }
                "missing-input" => {
                    steps[0]["input"].as_object_mut().unwrap().remove("copied");
                }
                _ => unreachable!(),
            }
            f.persist();
            let r = f.browse("await click('Step');return JSON.stringify(snapshot());");
            if mode == "missing-input" {
                unknown_fields(&r, "first", &["copied"]);
                contains(&r, "assignment input \"copied\" is missing");
            } else {
                assert_eq!(r["world"]["instances"], json!({}));
                assert!(!r["world"]["notes"].as_array().unwrap().is_empty());
                contains(&r, "Unknown");
            }
        }
    }
}
#[test]
fn b08_missing_required_authored_parameter_still_refuses() {
    for route in [4, 5] {
        let evidence = std::env::temp_dir().join(format!(
            "ess-replay-b08-refused-{route}-{}",
            std::process::id()
        ));
        fs::create_dir(&evidence).unwrap();
        fs::write(evidence.join("system.yaml"), SPEC).unwrap();
        fs::write(
            evidence.join("scenario.yaml"),
            scenario(false).replace("    params: {wanted: alpha}\n", ""),
        )
        .unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command
            .args(["conform", "web", "--path"])
            .arg(evidence.join("system.yaml"))
            .arg("--scenarios")
            .arg(evidence.join("scenario.yaml"))
            .args(["--suite-format", &route.to_string(), "--out"])
            .arg(evidence.join("site"));
        fs::write(evidence.join("emit.command"), format!("{command:?}")).unwrap();
        let output = command.output().unwrap();
        fs::write(
            evidence.join("emit.exit"),
            format!("{:?}", output.status.code()),
        )
        .unwrap();
        fs::write(evidence.join("emit.stdout"), &output.stdout).unwrap();
        fs::write(evidence.join("emit.stderr"), &output.stderr).unwrap();
        assert!(!output.status.success());
        let message = if route == 4 {
            String::from_utf8(output.stdout).unwrap()
        } else {
            let replay = fs::read_to_string(evidence.join("site/replay.json")).unwrap();
            ess_conformance::web_replay::AdmittedReplay::from_json(&replay).unwrap();
            replay
        };
        assert!(message.contains("wanted"), "{message}");
        assert!(message.contains("refus"), "{message}");
    }
}
#[test]
fn b11_refusal_preserves_established_declarations_and_unknown_facts() {
    for route in [4, 5] {
        // Wrong-state is a legitimate authored declared refusal; replay does not decide its guard.
        let authored = scenario(false).replace("outcome: finished", "outcome: wrong-state");
        let f = Fixture::emit("b11-refusal", route, SPEC, &authored);
        let command = f.model["commands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "replay.items.Create")
            .unwrap();
        assert_eq!(command["outcomes"][0]["refuses"], true);
        assert_eq!(command["outcomes"][0]["subject"]["kind"], "creates");
        let r=f.browse("await click('Step');await click('Step');const before=snapshot();await click('Step');return JSON.stringify({before,after:snapshot()});");
        for key in ["instances", "unknownEffects", "events", "notes"] {
            assert_eq!(
                r["before"]["world"][key], r["after"]["world"][key],
                "{key}: {r}"
            );
        }
        assert_eq!(r["after"]["changes"], json!([]));
        contains(&r["after"], "wrong-state");
        assert_eq!(r["after"]["world"]["instances"]["first"]["state"], "Draft");
    }
}
#[test]
fn b12_reset_select_pause_and_restart_cancel_callbacks_from_previous_play() {
    for route in [4, 5] {
        let f = Fixture::emit("b12-timers", route, SPEC, &scenario(false));
        let r=f.browse(r"
          const wait=()=>new Promise(r=>setTimeout(r,200));const saved=[];
          p.state.speed=30;await click('▶ Play');await click('Reset');p.state.speed=1000;await click('▶ Play');await wait();saved.push(snapshot());await click('❙❙ Pause');await click('Reset');
          p.state.speed=30;await click('▶ Play');document.querySelector('.scn').click();await nextTick();p.state.speed=1000;await click('▶ Play');await wait();saved.push(snapshot());await click('❙❙ Pause');await click('Reset');
          p.state.speed=30;await click('▶ Play');await click('❙❙ Pause');p.state.speed=1000;await click('▶ Play');await wait();saved.push(snapshot());await click('❙❙ Pause');
          await all();p.state.speed=1000;await click('▶ Play');await wait();saved.push(snapshot());await click('❙❙ Pause');
          return JSON.stringify(saved);
        ");
        for (i, expected) in [0, 0, 1, 0].iter().enumerate() {
            assert_eq!(r[i]["cursor"], *expected, "timer case {i}: {r}");
            assert_eq!(r[i]["boom"], "");
        }
    }
}

#[test]
fn b01_capture_event_field_need_not_equal_entity_identity_field() {
    for route in [4, 5] {
        let spec=SPEC.replacen("fields: [{name: id, type: replay.items.Id}, {name: label, type: String}]", "fields: [{name: generated_id, type: replay.items.Id}, {name: label, type: String}]", 1)
            .replace("        instance: id\n", "        instance: generated_id\n");
        let authored = scenario(false).replace("field: id}", "field: generated_id}");
        let f = Fixture::emit("b01-capture-field", route, &spec, &authored);
        let r = f.browse("await click('Step');return JSON.stringify(snapshot());");
        assert_eq!(r["world"]["instances"]["first"]["state"], "Draft");
        unknown_fields(&r, "first", &["id", "generated_id"]);
        contains(&r, "generated_id");
    }
}
#[test]
fn b06_update_with_single_reference_keeps_state_but_cannot_choose_subject() {
    for route in [4, 5] {
        let update = r"  - name: replay.items.Update
    input:
      - {name: a, type: replay.items.Id}
      - {name: z, type: replay.items.Id}
      - {name: literal_id, type: replay.items.Id}
      - {name: label, type: String}
      - {name: rank, type: Integer}
    outcomes:
      - name: finished
        updates: replay.items.Item
        instance: z
        emits: [replay.items.Finished]
        sets: {label: input.label, rank: input.rank}
";
        let spec = SPEC.replace("views:\n", &format!("{update}views:\n"));
        let authored = scenario(false)
            .replace(
                "command: replay.items.Finish",
                "command: replay.items.Update",
            )
            .replace("z: {$instance: second}", "z: literal-subject");
        let f = Fixture::emit("b06-update", route, &spec, &authored);
        let r = f.browse("await all();return JSON.stringify(snapshot());");
        assert_eq!(r["world"]["instances"].as_object().unwrap().len(), 2);
        contains(&r, SUBJECT);
        for alias in ["first", "second"] {
            assert_eq!(r["world"]["instances"][alias]["state"], "Draft");
            unknown_fields(&r, alias, &["label", "rank"]);
        }
    }
}
#[test]
fn b11_binding_and_external_controls_never_create_observations_or_instances() {
    for route in [4, 5] {
        let entity = r"  - name: replay.items.Receipt
    identity: {name: id, type: replay.items.Id}
    lifecycle: {initial: Recorded, states: [Recorded], terminal: [Recorded]}
";
        let event =
            "  - name: replay.items.Recorded\n    fields: [{name: id, type: replay.items.Id}]\n";
        let command = r"  - name: replay.items.Record
    input: [{name: label, type: String}]
    outcomes:
      - name: recorded
        creates: replay.items.Receipt
        instance: id
        emits: [replay.items.Recorded]
      - name: unavailable
        external: recorder unavailable
        error: replay.items.Conflict
";
        let binding = r"
bindings:
  - id: record-created
    when: {event: replay.items.Created}
    invoke: {command: replay.items.Record}
    mapping: {label: event.label}
    delivery: at_least_once
    on_failure: drop
";
        let spec = SPEC
            .replace("events:\n", &format!("{entity}events:\n"))
            .replace("errors:\n", &format!("{event}errors:\n"))
            .replace("views:\n", &format!("{command}views:\n"))
            + binding;
        let mut f = Fixture::emit("b11-binding", route, &spec, &scenario(false));
        f.steps().insert(0,json!({"step":"configure_external_outcome","force":{"command":"replay.items.Record","outcome":"unavailable"}}));
        f.steps().push(json!({"step":"expect_invocation","binding":"record-created","command":"replay.items.Record","input":{"label":{"kind":"observed","event":"replay.items.Created","field":"label"}}}));
        f.persist();
        let r = f.browse("await all();return JSON.stringify(snapshot());");
        assert_eq!(r["world"]["instances"].as_object().unwrap().len(), 2);
        assert_eq!(r["world"]["events"], json!([]));
        contains(&r, "record-created");
        contains(&r, "Unexecuted binding declaration");
        contains(&r, "configure_external_outcome");
        contains(&r, "expect_invocation");
        assert!(r["world"]["instances"]
            .as_object()
            .unwrap()
            .values()
            .all(|i| i["entity"] == ENTITY));
    }
}

// Independent source attack pass 1: actual authored count bounds through emitted assets.
fn adversary_single_count_bound(route: u8, bound: &str) {
    let authored = scenario(false).replace(
        "counts: {at_least: 0, at_most: 0}",
        &format!("counts: {{{bound}: 0}}"),
    );
    let f = Fixture::emit(&format!("adversary-count-{bound}"), route, SPEC, &authored);
    let expectation = f.suite["scenarios"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|step| step["expectation"]["expect"] == "counts")
        .unwrap()["expectation"]
        .clone();
    assert_eq!(expectation[bound], 0);
    let absent = if bound == "at_least" {
        "at_most"
    } else {
        "at_least"
    };
    assert!(expectation.get(absent).is_none());
    let server = browser::Server::new(&f.site);
    let mut browser = browser::Browser::new(&f.evidence);
    let context = browser.open(&format!("{}/index.html", server.url));
    let script = r"(async()=>{
      try {
        const {default:p}=await import('./player.js');
        const {nextTick}=await import('./assets/vue.esm-browser.prod.js');
        for(let i=0;i<p.scenarios[0].acts.length;i++)p.step();
        p.state.tab='views';await nextTick();
        return JSON.stringify({mounted:true,cursor:p.state.cursor,views:p.liveViews.value,dom:document.getElementById('app').textContent,boom:document.getElementById('boom').textContent});
      } catch(error) {
        return JSON.stringify({mounted:false,error:String(error),dom:document.getElementById('app').textContent,boom:document.getElementById('boom').textContent});
      }
    })()";
    fs::write(f.evidence.join("adversary-probe.js"), script).unwrap();
    let result = browser.evaluate(&context, script);
    fs::write(
        f.evidence.join("adversary-result.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();
    assert_eq!(
        result["mounted"], true,
        "valid authored {bound}-only count must mount on suite/{route}: {result}"
    );
    assert_eq!(result["boom"], "");
    contains(
        &result,
        "Unknown: this replay does not evaluate view results.",
    );
    assert!(result["views"]
        .as_array()
        .unwrap()
        .iter()
        .all(|view| view.get("rows").is_none()));
}

#[test]
fn adversary_lower_only_count_mounts_suite4() {
    adversary_single_count_bound(4, "at_least");
}
#[test]
fn adversary_lower_only_count_mounts_suite5() {
    adversary_single_count_bound(5, "at_least");
}
#[test]
fn adversary_upper_only_count_mounts_suite4() {
    adversary_single_count_bound(4, "at_most");
}
#[test]
fn adversary_upper_only_count_mounts_suite5() {
    adversary_single_count_bound(5, "at_most");
}

#[test]
fn adversary_query_only_prefix_remains_visible_and_reconstructible() {
    for route in [4, 5] {
        let mut f = Fixture::emit("adversary-query-only", route, SPEC, &scenario(false));
        f.steps()
            .retain(|step| matches!(step["step"].as_str(), Some("query_view" | "expect_view")));
        assert!(!f.steps().is_empty());
        f.persist();
        let result = f.browse(r"await click('Views');const initial=snapshot();await click('Step');const reached=snapshot();await click('◂ Back');const back=snapshot();await click('Step');return JSON.stringify({initial,reached,back,replayed:snapshot()});");
        assert_eq!(result["initial"], result["back"]);
        assert_eq!(result["reached"], result["replayed"]);
        assert_eq!(result["reached"]["world"]["instances"], json!({}));
        assert_eq!(result["reached"]["world"]["events"], json!([]));
        contains(&result["reached"], "query_view");
        contains(&result["reached"], PARAMETER);
        assert_eq!(result["reached"]["boom"], "");
    }
}

fn adversary_explicit_null_count_bound(route: u8) {
    let bound = "at_least";
    let authored = scenario(false).replace(
        "counts: {at_least: 0, at_most: 0}",
        &format!("counts: {{{bound}: 0}}"),
    );
    let mut f = Fixture::emit("adversary-explicit-null-count", route, SPEC, &authored);
    f.steps()
        .iter_mut()
        .find(|step| step["expectation"]["expect"] == "counts")
        .unwrap()["expectation"]["at_most"] = Value::Null;
    // An optional bound may be explicitly null in admitted persisted suite bytes.
    // The actual Rust suite and paired replay readers must admit this exact carrier first.
    f.persist();
    let expectation = f.suite["scenarios"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|step| step["expectation"]["expect"] == "counts")
        .unwrap()["expectation"]
        .clone();
    assert_eq!(expectation[bound], 0);
    let absent = if bound == "at_least" {
        "at_most"
    } else {
        "at_least"
    };
    assert!(expectation[absent].is_null());
    let server = browser::Server::new(&f.site);
    let mut browser = browser::Browser::new(&f.evidence);
    let context = browser.open(&format!("{}/index.html", server.url));
    let script = r"(async()=>{
      try {
        const {default:p}=await import('./player.js');
        const {nextTick}=await import('./assets/vue.esm-browser.prod.js');
        for(let i=0;i<p.scenarios[0].acts.length;i++)p.step();
        p.state.tab='views';await nextTick();
        return JSON.stringify({mounted:true,cursor:p.state.cursor,views:p.liveViews.value,dom:document.getElementById('app').textContent,boom:document.getElementById('boom').textContent});
      } catch(error) {
        return JSON.stringify({mounted:false,error:String(error),dom:document.getElementById('app').textContent,boom:document.getElementById('boom').textContent});
      }
    })()";
    fs::write(f.evidence.join("adversary-probe.js"), script).unwrap();
    let result = browser.evaluate(&context, script);
    fs::write(
        f.evidence.join("adversary-result.json"),
        serde_json::to_string_pretty(&result).unwrap(),
    )
    .unwrap();
    assert_eq!(
        result["mounted"], true,
        "admitted explicit-null {bound}-only count must mount on suite/{route}: {result}"
    );
    assert_eq!(result["boom"], "");
    contains(
        &result,
        "Unknown: this replay does not evaluate view results.",
    );
    assert!(result["views"]
        .as_array()
        .unwrap()
        .iter()
        .all(|view| view.get("rows").is_none()));
}

#[test]
fn adversary_explicit_null_count_mounts_suite4() {
    adversary_explicit_null_count_bound(4);
}
#[test]
fn adversary_explicit_null_count_mounts_suite5() {
    adversary_explicit_null_count_bound(5);
}

// Independent ordinary browser-source review pass 2; inherited assertions remain unchanged.
#[test]
fn adversary2_exact_metadata_and_literal_lookalikes_keep_distinct_kinds() {
    let mut f = Fixture::emit("adversary2-metadata", 5, SPEC, &scenario(false));
    let large = u64::MAX;
    let literal = json!({"elapsed":null,"after":false,"expectation":{"at_least":0,"at_most":"18446744073709551615","position":{"index":{"raw":"literal text"}}}});
    f.steps()[0]["input"]["mapping_value"] = json!({"kind":"literal","value":literal});
    f.steps()
        .insert(0, json!({"step":"mark_instant","instant":"start"}));
    f.steps().extend([
        json!({"step":"expect_within","instant":"start","elapsed":u32::MAX}),
        json!({"step":"expect_halt","view":"replay.items.Ranked","after":large}),
        json!({"step":"expect_view","view":"replay.items.Ranked","expectation":{"expect":"counts","at_least":null,"at_most":large}}),
        json!({"step":"expect_view","view":"replay.items.Ranked","expectation":{"expect":"at","order_by":["rank desc"],"position":{"row":"nth","index":large},"fields":{"rank":{"kind":"literal","value":0}}}}),
    ]);
    f.persist();
    let r = f.browse("await all();await click('Views');return JSON.stringify(snapshot());");
    assert_eq!(r["boom"], "");
    assert_eq!(r["acts"][1]["input"]["mapping_value"]["value"], literal);
    let declarations: Vec<_> = r["acts"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|act| act["steps"].as_array().unwrap())
        .collect();
    for (step, path) in [
        ("expect_within", "/elapsed"),
        ("expect_halt", "/after"),
        ("counts", "/expectation/at_most"),
        ("at", "/expectation/position/index"),
    ] {
        let d = declarations
            .iter()
            .rev()
            .find(|d| {
                d["declaration"]["step"] == step
                    || d["declaration"]["expectation"]["expect"] == step
            })
            .unwrap();
        let token = if step == "expect_within" {
            u64::from(u32::MAX)
        } else {
            large
        }
        .to_string();
        assert_eq!(
            d["declaration"].pointer(path).unwrap(),
            &json!({"raw":token})
        );
        assert!(d["text"].as_str().unwrap().contains(&token), "{d}");
    }
    contains(&r, "\"at_least\":null");
    contains(&r, "\"index\":18446744073709551615");
    contains(&r, "\"elapsed\":4294967295");
    contains(&r, "\"raw\":\"literal text\"");
    assert!(r["views"]
        .as_array()
        .unwrap()
        .iter()
        .all(|view| view.get("rows").is_none()));
    assert_eq!(r["world"]["events"], json!([]));
}

#[test]
fn adversary2_reused_creation_alias_is_diagnostic_and_preserves_prior_prefix() {
    for route in [4, 5] {
        let mut f = Fixture::emit("adversary2-reused-alias", route, SPEC, &scenario(false));
        f.steps()
            .iter_mut()
            .filter(|step| step["step"] == "capture_instance")
            .nth(1)
            .unwrap()["instance"] = json!("first");
        f.persist();
        let r = f.browse(r"await click('Step');const first=snapshot();await click('Step');const duplicate=snapshot();await click('◂ Back');const back=snapshot();await click('Step');return JSON.stringify({first,duplicate,back,replayed:snapshot()});");
        assert_eq!(r["first"], r["back"]);
        assert_eq!(r["duplicate"], r["replayed"]);
        assert_eq!(
            r["duplicate"]["world"]["instances"],
            r["first"]["world"]["instances"]
        );
        assert_eq!(
            r["duplicate"]["world"]["instances"]
                .as_object()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(r["duplicate"]["world"]["events"], json!([]));
        assert_eq!(r["duplicate"]["boom"], "");
        contains(&r["duplicate"], "unused scenario-local alias");
        assert!(r["duplicate"]["world"]["unknownEffects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|effect| effect["at"] == 1 && effect.get("instance").is_none()));
    }
}

#[test]
fn adversary2_switching_to_a_distinct_authored_scenario_cancels_pending_play() {
    for route in [4, 5] {
        let mut f = Fixture::emit("adversary2-scenario-select", route, SPEC, &scenario(false));
        let sources = f.evidence.join("two-scenarios");
        fs::create_dir(&sources).unwrap();
        fs::write(sources.join("first.yaml"), scenario(false)).unwrap();
        fs::write(
            sources.join("alternate.yaml"),
            scenario(false)
                .replace("scenario: declarations", "scenario: z-alternate")
                .replace("first", "third")
                .replace("second", "fourth"),
        )
        .unwrap();
        f.site = f.evidence.join("two-scenario-site");
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command
            .args(["conform", "web", "--path"])
            .arg(f.evidence.join("system.yaml"))
            .arg("--scenarios")
            .arg(&sources)
            .args(["--suite-format", &route.to_string(), "--out"])
            .arg(&f.site);
        fs::write(
            f.evidence.join("two-scenario.command"),
            format!("{command:?}\n"),
        )
        .unwrap();
        let output = command.output().unwrap();
        fs::write(f.evidence.join("two-scenario.stdout"), &output.stdout).unwrap();
        fs::write(f.evidence.join("two-scenario.stderr"), &output.stderr).unwrap();
        fs::write(
            f.evidence.join("two-scenario.exit"),
            format!("{:?}\n", output.status.code()),
        )
        .unwrap();
        assert!(output.status.success(), "{output:?}");
        if route == 5 {
            let original = fs::read_to_string(f.site.join("replay.json")).unwrap();
            ess_conformance::web_replay::AdmittedReplay::from_json(&original).unwrap();
        }
        let r = f.browse(r"const names=p.scenarios.map(s=>s.name);if(names.length!==2)throw Error('expected two scenarios');
          const selectAlternate=async()=>{document.querySelectorAll('.scn')[1].click();await nextTick();};
          await selectAlternate();await click('Step');const expected=snapshot();
          document.querySelectorAll('.scn')[0].click();await nextTick();p.state.speed=30;await click('▶ Play');await selectAlternate();
          const selected=snapshot();p.state.speed=1000;await click('▶ Play');await new Promise(r=>setTimeout(r,200));await click('❙❙ Pause');
          return JSON.stringify({names,expected,selected,after:snapshot()});");
        assert_eq!(r["names"].as_array().unwrap().len(), 2);
        assert_eq!(r["selected"]["cursor"], -1);
        assert_eq!(r["selected"]["world"]["instances"], json!({}));
        assert_eq!(r["after"], r["expected"]);
        assert_eq!(r["after"]["cursor"], 0);
        assert_eq!(r["after"]["world"]["instances"]["third"]["state"], "Draft");
        assert!(r["after"]["world"]["instances"].get("first").is_none());
        assert_eq!(r["after"]["boom"], "");
    }
}
