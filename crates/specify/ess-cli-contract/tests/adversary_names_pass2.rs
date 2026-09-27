//! Adversary pass 2 on story:field-names-underscore-and-newtype-map-keys (beyond10x/ess#143).
//!
//! A CLI binding's value contracts are type spellings resolved against the compiled model. The
//! model admits a map keyed by a newtype of `String`, and the binding only admits `String` keys —
//! which a newtype of `String` is on the wire.

use ess_cli_contract::{compile, Binding};
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = "format: ess/14
system: demo
version: v1
types:
  - name: demo.Label
    kind: newtype
    of: String
  - name: demo.Input
    kind: struct
    fields:
      - {name: profile, type: String}
  - name: demo.Stored
    kind: struct
    fields:
      - {name: labels, type: \"Map<demo.Label, String>\"}
  - name: demo.Failure
    kind: struct
    fields:
      - {name: reason, type: String}
";

const BINDING: &str = "format: ess-cli/1
binary: demo
about: Fixture CLI
globals: {config: config, state: state-dir, output: output}
callables:
  store:
    target: {kind: local, owner: demo.cli, action: store-credential}
    input: demo.Input
    result: \"RESULT\"
    errors: {store_failed: demo.Failure}
commands:
  - path: [credential, store]
    callable: store
    about: Store through the fixture handler
    arguments:
      - {field: profile, source: {kind: option, long: profile}}
";

fn model() -> EssIr {
    let specification = Specification::assemble(vec![(
        Source::new("system.yaml"),
        RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("well formed: {error}")),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    ess_compiler::compile(&specification, &ess_compiler::source::SourceMap::new())
        .unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn compiled(result: &str) -> Result<(), String> {
    let binding =
        Binding::from_yaml(&BINDING.replace("RESULT", result)).map_err(|e| e.to_string())?;
    compile(&model(), &binding)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[test]
fn adversary_pass2_control_a_string_keyed_result_contract_compiles() {
    compiled("Map<String, String>").expect("a String-keyed map is a CLI value contract");
}

#[test]
fn adversary_pass2_a_newtype_keyed_result_contract_compiles_like_its_primitive() {
    compiled("Map<demo.Label, String>").expect(
        "beyond10x/ess#143: `demo.Label` is a newtype of String, which the model admits as a map \
         key; the CLI contract resolving against that model should too",
    );
}
