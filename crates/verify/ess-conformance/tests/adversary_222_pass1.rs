//! Adversary pass 1 against beyond10x/ess#222 (`ESS-AUTHOR-041`).
//!
//! A guard over the current-time operand (`now`, beyond10x/ess#171) is decided by the target
//! against its own clock (`docs/design/current-time-guards.md`, "Where `now` comes from"). The
//! authored check decides it against `InputFacts::now`, synthesis's fixed reference instant
//! `2019-12-30T23:59:59Z`, so a literal instant between that reference and the run is decided
//! the opposite way a correct target decides it. A valid authored act is then refused.

use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Source};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source as SpecSource;
use ess_domain::Specification;

/// `start-in-past: starts_at < now - 60s` (an input-guarded refusal) and the default `scheduled`.
const JOBS: &str = include_str!("fixtures/current-time-guard.yaml");

/// The same command with a refusal over a start too far ahead: `starts_at > now + 1h`.
const AHEAD: &str = r"
format: ess/16
system: demo
version: v1
domain: demo.jobs
types:
  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}
entities:
  - name: demo.jobs.Job
    identity: {name: job_id, type: demo.jobs.JobId}
    fields:
      - {name: starts_at, type: Timestamp}
    lifecycle: {initial: Scheduled, states: [Scheduled], terminal: [Scheduled], transitions: []}
errors:
  - {name: demo.jobs.TooFarAhead, fields: []}
events:
  - name: demo.jobs.JobScheduled
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: starts_at, type: Timestamp}
commands:
  - name: demo.jobs.ScheduleJob
    input:
      - {name: starts_at, type: Timestamp}
    outcomes:
      - name: too-far-ahead
        when: starts_at > now + 1h
        error: demo.jobs.TooFarAhead
      - name: scheduled
        creates: demo.jobs.Job
        instance: job_id
        sets: {starts_at: input.starts_at}
        emits: [demo.jobs.JobScheduled]
        payload:
          demo.jobs.JobScheduled: {job_id: {generated: true}, starts_at: input.starts_at}
";

fn fixture(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let specification = Specification::assemble([(SpecSource::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn schedule(starts_at: &str, claim: &str) -> String {
    format!(
        "type: ess-scenario/1\ndomain: demo.jobs\nscenario: a-scenario\n\
         summary: What this scenario proves, in one line.\ntimeline:\n  \
         - at: 2026-01-05T09:00:00Z\n    command: demo.jobs.ScheduleJob\n    \
         input: {{starts_at: '{starts_at}'}}\n{claim}"
    )
}

fn refusals(ir: &EssIr, text: &str) -> Vec<String> {
    compile_authored(ir, &[Source::new("scenario.yaml", text)])
        .refusals
        .iter()
        .map(ToString::to_string)
        .collect()
}

/// A start on 2025-01-01 is more than a minute in the past at every run (`now_offset::earliest_run`
/// is 2026-09-27), so a correct target answers `start-in-past`. The act expecting it is valid.
#[test]
fn a_literal_start_already_past_at_every_run_may_expect_the_past_refusal() {
    let ir = fixture(JOBS);
    let refused = refusals(
        &ir,
        &schedule("2025-01-01T00:00:00Z", "    outcome: start-in-past\n"),
    );
    assert!(refused.is_empty(), "a valid act is refused: {refused:#?}");
}

/// The same act claiming the refusal's error with no `outcome:`.
#[test]
fn a_literal_start_already_past_at_every_run_may_expect_the_past_refusal_s_error() {
    let ir = fixture(JOBS);
    let refused = refusals(
        &ir,
        &schedule(
            "2025-01-01T00:00:00Z",
            "    error: {name: demo.jobs.StartInPast}\n",
        ),
    );
    assert!(refused.is_empty(), "a valid act is refused: {refused:#?}");
}

/// A start on 2025-06-01 is not more than an hour ahead at any run, so a correct target answers
/// the default. The checker reads `now` as 2019-12-30 and has `too-far-ahead` answer first.
#[test]
fn a_literal_start_not_ahead_at_any_run_may_expect_the_default() {
    let ir = fixture(AHEAD);
    let refused = refusals(
        &ir,
        &schedule("2025-06-01T00:00:00Z", "    outcome: scheduled\n"),
    );
    assert!(refused.is_empty(), "a valid act is refused: {refused:#?}");
}

/// Control: a start before the reference instant is decided alike at the reference and at every
/// run, so these stay as they are.
#[test]
fn a_literal_start_before_the_reference_is_decided_alike_at_every_run() {
    let ir = fixture(JOBS);
    assert_eq!(
        refusals(
            &ir,
            &schedule("2019-01-01T00:00:00Z", "    outcome: start-in-past\n")
        ),
        Vec::<String>::new()
    );
    let wrong = refusals(
        &ir,
        &schedule("2019-01-01T00:00:00Z", "    outcome: scheduled\n"),
    );
    assert_eq!(wrong.len(), 1, "{wrong:#?}");
    assert!(wrong[0].contains("ESS-AUTHOR-041"), "{wrong:#?}");
}

/// Literal-format probes a person types and synthesis never writes: an integral decimal spelt
/// with a fraction, and a text counted by Unicode scalar values rather than bytes.
const PROBE: &str = r"
format: ess/16
system: probe
version: v1
domain: probe.p
errors:
  - {name: probe.p.Refused, fields: []}
events:
  - {name: probe.p.Done, fields: []}
commands:
  - name: probe.p.Act
    input:
      - {name: amount, type: Decimal}
      - {name: count, type: Integer}
      - {name: label, type: String}
    outcomes:
      - {name: exact, when: amount == 1.5, error: probe.p.Refused}
      - {name: two, when: count == 2, error: probe.p.Refused}
      - {name: short, when: label.count < 3, emits: [probe.p.Done]}
      - {name: long, emits: [probe.p.Done]}
";

fn probe(input: &str, outcome: &str) -> String {
    format!(
        "type: ess-scenario/1\ndomain: probe.p\nscenario: a-scenario\n\
         summary: What this scenario proves, in one line.\ntimeline:\n  \
         - at: 2026-01-05T09:00:00Z\n    command: probe.p.Act\n    \
         input: {input}\n    outcome: {outcome}\n"
    )
}

#[test]
fn literal_spellings_are_decided_as_the_value_they_name() {
    let ir = fixture(PROBE);
    for (input, outcome) in [
        ("{amount: 1.50, count: 0, label: abcd}", "exact"),
        ("{amount: 0, count: 2.0, label: abcd}", "two"),
        ("{amount: 0, count: 0, label: 日本}", "short"),
        ("{amount: 0, count: 0, label: 日本語}", "long"),
        ("{amount: 0, count: 0, label: \"e\\u0301x\"}", "long"),
    ] {
        let refused = refusals(&ir, &probe(input, outcome));
        assert!(refused.is_empty(), "{input} -> {outcome}: {refused:#?}");
    }
    // Controls, so the probe above is one the check could have failed.
    for (input, outcome) in [
        ("{amount: 1.50, count: 0, label: abcd}", "long"),
        ("{amount: 0, count: 2.0, label: abcd}", "long"),
        ("{amount: 0, count: 0, label: 日本語}", "short"),
    ] {
        let refused = refusals(&ir, &probe(input, outcome));
        assert_eq!(refused.len(), 1, "{input} -> {outcome}: {refused:#?}");
        assert!(refused[0].contains("ESS-AUTHOR-041"), "{refused:#?}");
    }
}

/// Every authored scenario committed on disk, compiled against every specification the corpus
/// holds, draws no `ESS-AUTHOR-041`.
#[test]
fn no_committed_authored_scenario_is_refused_by_the_new_code() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let mut scenarios = Vec::new();
    walk(&root, &mut scenarios, &|path| {
        std::fs::read_to_string(path).is_ok_and(|text| {
            text.lines()
                .any(|line| line.starts_with("type: ess-scenario/"))
        })
    });
    scenarios.sort();
    let corpus = corpus(&root);
    eprintln!(
        "authored files: {}, specifications: {}",
        scenarios.len(),
        corpus.len()
    );
    assert!(scenarios.len() >= 10, "{scenarios:?}");
    assert!(corpus.len() >= 60);
    let mut found = Vec::new();
    let mut compiled_clean = 0;
    for path in &scenarios {
        let text = std::fs::read_to_string(path).unwrap();
        for (label, ir) in &corpus {
            let authoring = compile_authored(ir, &[Source::new("s.yaml", text.as_str())]);
            if authoring.is_complete() {
                compiled_clean += 1;
            }
            for refusal in &authoring.refusals {
                if refusal.code().to_string() == "ESS-AUTHOR-041" {
                    found.push(format!("{} on {label}: {refusal}", path.display()));
                }
            }
        }
    }
    eprintln!("clean (file, specification) pairs: {compiled_clean}");
    assert!(
        compiled_clean >= 5,
        "the walk paired no scenario with its model"
    );
    assert!(found.is_empty(), "{found:#?}");
}

fn walk(directory: &Path, found: &mut Vec<PathBuf>, keep: &dyn Fn(&Path) -> bool) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.is_dir() {
            if name == "target" || name == "node_modules" || name.starts_with('.') {
                continue;
            }
            walk(&path, found, keep);
        } else if path
            .extension()
            .is_some_and(|it| it == "yaml" || it == "yml")
            && keep(&path)
        {
            found.push(path);
        }
    }
}

/// Each `examples/` and `models/` system and each single-document `tests/fixtures/` `.yaml` that
/// compiles, as `tests/enum_presence_guard.rs` walks them.
fn corpus(root: &Path) -> Vec<(String, EssIr)> {
    fn compiled(files: &[PathBuf], base: &Path) -> Option<EssIr> {
        let mut sources = SourceMap::new();
        let mut parsed = Vec::new();
        for path in files {
            let label = path.strip_prefix(base).unwrap().display().to_string();
            let text = std::fs::read_to_string(path).unwrap();
            let Ok(raw) = RawSpecFile::parse(&text) else {
                continue;
            };
            sources.insert(label.clone(), text);
            parsed.push((SpecSource::new(label), raw));
        }
        if parsed.is_empty() {
            return None;
        }
        let specification = Specification::assemble(parsed).ok()?;
        compile(&specification, &sources).ok()
    }
    let all = |_: &Path| true;
    let mut out = Vec::new();
    for family in ["examples", "models"] {
        let mut systems: Vec<PathBuf> = std::fs::read_dir(root.join(family))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .collect();
        systems.sort();
        for system in systems {
            let mut files = Vec::new();
            walk(&system, &mut files, &all);
            files.retain(|it| it.extension().is_some_and(|e| e == "yaml"));
            files.sort();
            if let Some(model) = compiled(&files, &system) {
                out.push((
                    system.strip_prefix(root).unwrap().display().to_string(),
                    model,
                ));
            }
        }
    }
    let mut fixtures = Vec::new();
    for area in std::fs::read_dir(root.join("crates")).unwrap() {
        for krate in std::fs::read_dir(area.unwrap().path())
            .into_iter()
            .flatten()
        {
            walk(
                &krate.unwrap().path().join("tests/fixtures"),
                &mut fixtures,
                &all,
            );
        }
    }
    fixtures.retain(|it| it.extension().is_some_and(|e| e == "yaml"));
    fixtures.sort();
    for fixture in fixtures {
        let base = fixture.parent().unwrap().to_path_buf();
        if let Some(model) = compiled(std::slice::from_ref(&fixture), &base) {
            out.push((
                fixture.strip_prefix(root).unwrap().display().to_string(),
                model,
            ));
        }
    }
    out
}

/// `tests/fixtures/related-guard-sign-in.yaml` with an input-guarded refusal over `client` added
/// after the related branches.
fn sign_in_with_an_input_refusal() -> String {
    include_str!("fixtures/related-guard-sign-in.yaml")
        .replace(
            "errors:\n",
            "errors:\n  - {name: demo.signin.ClientRequired, summary: The client is empty., fields: []}\n",
        )
        .replace(
            "      - name: initiated\n",
            "      - {name: client-required, when: client == \"\", error: demo.signin.ClientRequired}\n      - name: initiated\n",
        )
}

fn initiate(client: &str, outcome: &str) -> String {
    format!(
        "type: ess-scenario/1\ndomain: demo.signin\nscenario: a-scenario\n\
         summary: What this scenario proves, in one line.\ntimeline:\n  \
         - at: 2026-01-05T09:00:00Z\n    command: demo.signin.InitiateSignIn\n    \
         input: {{tenant: 00000000-0000-4000-8000-000000000001, client: '{client}'}}\n    \
         outcome: {outcome}\n"
    )
}

/// On a command guarded by a related row, `exists: false` answers before any input refusal
/// (the precedence order, step 1). No test of the unit pairs a related row with an input refusal,
/// so dropping `not_taken`'s early return for it leaves the unit's suite green; this case does not.
#[test]
fn a_missing_related_row_answers_before_an_input_refusal_the_input_selects() {
    let ir = fixture(&sign_in_with_an_input_refusal());
    let refused = refusals(&ir, &initiate("", "no-configuration"));
    assert!(refused.is_empty(), "a valid act is refused: {refused:#?}");
    // Control: the input refusal still answers before the default.
    let wrong = refusals(&ir, &initiate("", "initiated"));
    assert_eq!(wrong.len(), 1, "{wrong:#?}");
    assert!(
        wrong[0].contains("ESS-AUTHOR-041") && wrong[0].contains("`client-required`"),
        "{wrong:#?}"
    );
}
