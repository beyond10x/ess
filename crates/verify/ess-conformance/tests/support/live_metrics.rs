use ess_compiler::{compile, source::SourceMap};
use ess_composition::{CompositionSpec, ServiceImportSpec, ServiceKey};
use ess_conformance::{authored::Source, compact, models::Models, recipes::Library};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source as ModelSource,
};
use serde_json::{json, Value};

pub const SOURCE: &str = r"type: ess-scenario/4
domain: fixture.metrics
scenario: observed-counts
summary: Exact owned counts after a fresh quiet baseline
when:
  - do: observe
    service: metrics
    event: fixture.metrics.Status
    matches: {status: live}
    anchor: first-live
    integers:
      catchup_ms: {min: 0, max: 10000}
  - do: quiet
    service: metrics
    event: fixture.metrics.Snapshot
    matches: {account: owned}
    quiet_for_ms: 2000
    anchor: settled
    capture:
      baseline: {path: waiting}
    integers:
      waiting: {min: 0, max: 9223372036854775807}
  - do: observe
    service: metrics
    event: fixture.metrics.Snapshot
    matches:
      account: owned
      waiting: {$offset: {capture: baseline, plus: 3}}
    anchor: raised
    after: settled
  - do: stable
    service: metrics
    event: fixture.metrics.Snapshot
    matches: {account: owned}
    since: raised
    stable_for_ms: 10000
    required:
      waiting: {$offset: {capture: baseline, plus: 3}}
then:
  - service: metrics
    event: fixture.metrics.Snapshot
    matches: {account: owned}
    since: raised
    stable_for_ms: 10000
    required:
      waiting: {$offset: {capture: baseline, plus: 3}}
";

pub fn compilation(source: &str) -> Result<compact::Compilation, String> {
    let files = [
        (
            "system.yaml",
            "format: ess/4\nsystem: fixture\nversion: v1\ndomains: [fixture.metrics]\n",
        ),
        (
            "metrics.yaml",
            "domain: fixture.metrics\nevents:\n  - name: fixture.metrics.Status\n    fields: [{name: status, type: String}, {name: catchup_ms, type: Optional<Integer>}]\n  - name: fixture.metrics.Snapshot\n    fields: [{name: account, type: String}, {name: waiting, type: Integer}]\n",
        ),
        (
            "components.yaml",
            "components:\n  - component: metrics\n    owns: {domains: [fixture.metrics]}\n    accepts: {commands: []}\n    publishes: {events: [fixture.metrics.Status, fixture.metrics.Snapshot]}\n",
        ),
    ];
    let spec = Specification::assemble(
        files.map(|(path, text)| (ModelSource::new(path), RawSpecFile::parse(text).unwrap())),
    )
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let key = ServiceKey::new("metrics").unwrap();
    let composition = CompositionSpec::new(
        "live".parse().unwrap(),
        vec![ServiceImportSpec::of(
            key.clone(),
            "metrics".parse().unwrap(),
            &ir,
        )],
        vec![],
    );
    let models = Models::compile(&composition, &[(key, &ir)]).unwrap();
    compact::compile(
        &models,
        &Library::parse(&[]).unwrap(),
        &[Source::new("metrics.yaml", source)],
    )
}

pub fn cases() -> Value {
    let modes = [
        "nonzero-baseline",
        "large-integer",
        "slow-first",
        "missing-catchup",
        "null-catchup",
        "negative-catchup",
        "fractional-catchup",
        "overflow-catchup",
        "overflow-offset",
        "transient-zero",
        "wrong-account",
        "gap",
        "cancelled",
        "truncated",
        "cleanup",
    ];
    Value::Array(modes.into_iter().map(|mode| {
        let (initial, baseline, raised) = match mode {
            "large-integer" => (9_007_199_254_740_993_i64, 9_007_199_254_740_997_i64, 9_007_199_254_741_000_i64),
            "overflow-offset" => (1, 9_223_372_036_854_775_806, 0),
            _ => (7, 12, 15),
        };
        let mut status = json!({"status": "live", "catchup_ms": 1});
        match mode {
            "slow-first" => status["catchup_ms"] = json!(10001),
            "missing-catchup" => { status.as_object_mut().unwrap().remove("catchup_ms"); }
            "null-catchup" => status["catchup_ms"] = Value::Null,
            "negative-catchup" => status["catchup_ms"] = json!(-1),
            "fractional-catchup" => status["catchup_ms"] = json!(0.5),
            "overflow-catchup" => status["catchup_ms"] = json!(9_223_372_036_854_775_808_u64),
            _ => {}
        }
        let snapshot = |account: &str, count: i64| json!({"account": account, "waiting": count});
        let mut cursor = 0_u64;
        let mut batch = |through: u64, rows: Vec<(u64, &str, Value)>| {
            let after = cursor;
            let rows = rows.into_iter().map(|(at, event, payload)| {
                cursor += 1;
                json!({"sequence": cursor, "at_ms": at, "event": format!("fixture.metrics.{event}"), "payload": payload})
            }).collect::<Vec<_>>();
            json!({"after":after,"complete_before_ms":through,"occurrences":rows})
        };
        let account = if mode == "wrong-account" { "unrelated" } else { "owned" };
        let mut batches = vec![
            batch(3000, vec![(1,"Status",status), (2,"Status",json!({"status":"live","catchup_ms":1})),
                (3,"Snapshot",snapshot("owned",initial))]),
            // Activation must request this fresh fence, despite a prior quiet history.
            batch(5000, vec![]),
            // A change just before expiry resets the quiet candidate to 8999.
            batch(7000, vec![(6999,"Snapshot",snapshot("owned",baseline))]),
            batch(9000, vec![]),
            batch(9002, vec![(9000,"Snapshot",snapshot("unrelated",99)),
                (9001,"Snapshot",snapshot(account,raised))]),
            batch(if mode == "truncated" {19000} else {19001}, vec![
                (10000,"Snapshot",snapshot(account,if mode == "transient-zero" {0} else {raised})),
                (12000,"Snapshot",snapshot("unrelated",99)),
                (18000,"Snapshot",snapshot(account,raised)),
            ]),
        ];
        if mode == "gap" { batches[2]["after"] = json!(0); }
        json!({
            "name":mode, "pass":matches!(mode,"nonzero-baseline"|"large-integer"),
            "error_at":if mode == "cancelled" {Some(3)} else {None::<usize>},
            "cleanup_failure":mode == "cleanup", "batches":batches,
        })
    }).collect())
}
