//! beyond10x/ess#481: `ess-cli/2` lets a binding omit the `config` and `output` globals, `state`
//! stays required, and no spelling of YAML null is read as a flag name.
use ess_cli_contract::{compile, Binding};
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Map, Value};
use std::fmt::Write as _;

const MODEL: &str = include_str!("fixtures/model.yaml");
const FIXTURE: &str = include_str!("fixtures/cli.yaml");
const ALL: [(&str, &str); 3] = [
    ("config", "config"),
    ("state", "state-dir"),
    ("output", "output"),
];

fn model() -> EssIr {
    let specification = Specification::assemble(vec![(
        Source::new("system.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new()).unwrap()
}

/// An inputless binding whose `globals` block holds exactly `globals`, one key per line.
fn binding(format: &str, globals: &[(&str, &str)]) -> String {
    let mut text = format!("format: {format}\nbinary: demo\nabout: Globals\n");
    if globals.is_empty() {
        text.push_str("globals: {}\n");
    } else {
        text.push_str("globals:\n");
        for (key, value) in globals {
            writeln!(text, "  {key}: {value}").unwrap();
        }
    }
    text.push_str(
        "callables:\n  show:\n    target: {kind: local, owner: demo.cli, action: show}\n    input: null\n    result: demo.Stored\ncommands:\n  - path: [show]\n    callable: show\n    about: Show\n    arguments: []\n",
    );
    text
}

/// Read and compile, returning the plan as JSON or the refusal's text.
fn admit(text: &str) -> Result<Value, String> {
    let binding = Binding::from_yaml(text).map_err(|error| error.to_string())?;
    let compiled = compile(&model(), &binding).map_err(|error| error.to_string())?;
    Ok(serde_json::from_str(&compiled.to_canonical_json()).unwrap())
}

#[test]
fn an_ess_cli_2_binding_may_omit_config_and_output() {
    let cases: [&[&str]; 4] = [
        &["state"],
        &["config", "state"],
        &["state", "output"],
        &["config", "state", "output"],
    ];
    for kept in cases {
        let globals: Vec<_> = ALL
            .iter()
            .copied()
            .filter(|(key, _)| kept.contains(key))
            .collect();
        let plan = admit(&binding("ess-cli/2", &globals))
            .unwrap_or_else(|error| panic!("{kept:?} refused: {error}"));
        assert_eq!(plan["format"], "ess-cli-plan/2", "{kept:?}");
        let declared: Map<String, Value> = globals
            .iter()
            .map(|&(key, value)| (key.to_owned(), json!(value)))
            .collect();
        assert_eq!(
            plan["globals"],
            Value::Object(declared),
            "an omitted global is omitted from the plan, never written null"
        );
    }
}

#[test]
fn an_ess_cli_2_binding_still_requires_state() {
    let cases: [&[(&str, &str)]; 2] = [&[], &[("config", "config"), ("output", "output")]];
    for globals in cases {
        let error = admit(&binding("ess-cli/2", globals)).expect_err("admitted without state");
        assert!(error.contains("`state`"), "{error}");
    }
}

#[test]
fn an_ess_cli_1_binding_still_requires_every_global_and_names_the_version_that_does_not() {
    for omitted in ["config", "state", "output"] {
        let globals: Vec<_> = ALL
            .iter()
            .copied()
            .filter(|&(key, _)| key != omitted)
            .collect();
        let error = admit(&binding("ess-cli/1", &globals))
            .expect_err(&format!("ess-cli/1 admitted without {omitted}"));
        assert!(error.contains(&format!("`{omitted}`")), "{error}");
        if omitted != "state" {
            assert!(error.contains("ess-cli/2"), "{error}");
        }
    }
}

/// Every YAML spelling of null, and the empty string, in either version: refused, naming the
/// global. Before #481 a lone `config: null` in an `ess-cli/1` binding became a flag `--null`.
#[test]
fn null_tilde_and_empty_are_refused_naming_the_global_in_both_formats() {
    for format in ["ess-cli/1", "ess-cli/2"] {
        for global in ["config", "state", "output"] {
            for spelling in ["null", "Null", "NULL", "~", "", "\"\"", "''"] {
                let globals: Vec<_> = ALL
                    .iter()
                    .map(|&(key, value)| (key, if key == global { spelling } else { value }))
                    .collect();
                let error = admit(&binding(format, &globals))
                    .expect_err(&format!("{format} admitted `{global}: {spelling}`"));
                assert!(
                    error.contains(&format!("globals.{global}")),
                    "{format} `{global}: {spelling}`: {error}"
                );
            }
        }
    }
}

#[test]
fn a_quoted_null_is_a_string_and_names_a_flag() {
    let plan = admit(&binding(
        "ess-cli/2",
        &[("config", "\"null\""), ("state", "state-dir")],
    ))
    .unwrap();
    assert_eq!(
        plan["globals"],
        json!({"config": "null", "state": "state-dir"})
    );
}

#[test]
fn an_invalid_or_conflicting_global_flag_names_the_global() {
    let cases: [(&[(&str, &str)], &str); 5] = [
        (
            &[("config", "Config"), ("state", "state-dir")],
            "globals.config",
        ),
        (
            &[("config", "state-dir"), ("state", "state-dir")],
            "globals.state",
        ),
        (&[("state", "help")], "globals.state"),
        (
            &[("state", "state-dir"), ("output", "version")],
            "globals.output",
        ),
        (
            &[("state", "state-dir"), ("output", "state-dir")],
            "globals.output",
        ),
    ];
    for (globals, named) in cases {
        let error = admit(&binding("ess-cli/2", globals)).expect_err(&format!("{globals:?}"));
        assert!(error.contains(named), "{globals:?}: {error}");
    }
}

#[test]
fn an_ess_cli_1_binding_compiles_to_the_plan_it_did_before_ess_cli_2() {
    let compiled = |text: &str| {
        compile(&model(), &Binding::from_yaml(text).unwrap())
            .unwrap()
            .to_canonical_json()
    };
    let first = binding("ess-cli/1", &ALL);
    let plan = compiled(&first);
    assert!(plan.contains("\"format\": \"ess-cli-plan/1\""), "{plan}");
    assert!(
        plan.contains(
            "\"globals\": {\n    \"config\": \"config\",\n    \"state\": \"state-dir\",\n    \"output\": \"output\"\n  }"
        ),
        "{plan}"
    );
    let second = compiled(&first.replacen("ess-cli/1", "ess-cli/2", 1));
    assert_eq!(
        plan.replacen("ess-cli-plan/1", "ess-cli-plan/2", 1),
        second,
        "with every global declared, the two versions differ only in the format"
    );
}

/// `Globals` as every released reader of `ess-cli-plan/1` declares it (0.56.0,
/// `crates/specify/ess-cli-contract/src/wire.rs`): three required Strings, unknown keys refused.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ReleasedGlobals {
    config: String,
    state: String,
    output: String,
}

#[test]
fn a_released_plan_reader_reads_ess_cli_plan_1_and_refuses_a_plan_that_omits_a_global() {
    let first = admit(&binding("ess-cli/1", &ALL)).unwrap();
    let read: ReleasedGlobals = serde_json::from_value(first["globals"].clone()).unwrap();
    assert_eq!(
        (
            read.config.as_str(),
            read.state.as_str(),
            read.output.as_str()
        ),
        ("config", "state-dir", "output")
    );
    let cases: [(&str, &[(&str, &str)]); 2] = [
        ("config", &[("state", "state-dir"), ("output", "output")]),
        ("output", &[("config", "config"), ("state", "state-dir")]),
    ];
    for (omitted, globals) in cases {
        let second = admit(&binding("ess-cli/2", globals)).unwrap();
        let error = serde_json::from_value::<ReleasedGlobals>(second["globals"].clone())
            .expect_err("a released reader read a plan without a global")
            .to_string();
        assert!(
            error.contains(&format!("missing field `{omitted}`")),
            "{error}"
        );
    }
}

#[test]
fn an_omitted_global_leaves_its_flag_name_to_a_command() {
    let free = FIXTURE
        .replace("ess-cli/1", "ess-cli/2")
        .replace(
            "globals: {config: config, state: state-dir, output: output}",
            "globals: {state: state-dir}",
        )
        .replace("long: profile", "long: output");
    admit(&free).unwrap();
    let reserved = FIXTURE.replace("long: profile", "long: output");
    let error = admit(&reserved).expect_err("a declared global's flag was given to a command");
    assert!(error.contains("`output`"), "{error}");
}

#[test]
fn an_unknown_format_is_refused_naming_both_admitted_versions() {
    let error = Binding::from_yaml(&binding("ess-cli/3", &ALL))
        .expect_err("admitted ess-cli/3")
        .to_string();
    for named in ["ess-cli/3", "ess-cli/1", "ess-cli/2"] {
        assert!(error.contains(named), "{error}");
    }
}
