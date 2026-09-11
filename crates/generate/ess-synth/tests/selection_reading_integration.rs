//! Selection must not erase a clock contract hidden behind a nested alias.
use ess_compiler::{resolve::compile_locating, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};

fn model(reading: bool) -> String {
    let contract = if reading {
        "    reading:\n      encoding: offset_date_time_text\n      origins: [{role: producer_process, offset: encoded_offset}]\n"
    } else {
        ""
    };
    include_str!("fixtures/binding-selection.yaml")
        .replace("      - name: from\n        type: String", "      - name: from\n        type: selection.core.ClockAlias")
        .replace("events:\n", &format!("  - name: selection.core.ClockValue\n    kind: newtype\n    of: String\n{contract}  - name: selection.core.ClockAlias\n    kind: newtype\n    of: selection.core.ClockValue\nevents:\n"))
}
fn compile(model: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("selection-reading.yaml"),
        RawSpecFile::parse(model).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("selection-reading.yaml", model);
    compile_locating(&spec, &sources, &["selection-reading.yaml"]).unwrap()
}
#[test]
fn direct_and_prepared_native_selection_refuse_reachable_clock_contracts() {
    for prepared in [false, true] {
        for reading in [false, true] {
            let mut source = model(reading);
            if prepared {
                source = source.replacen("type: List<Optional<selection.core.Leg>>", "type: String", 1)
                    + "\nconversions:\n  - from: String\n    to: List<Optional<selection.core.Leg>>\n    because: host prepares ordered records\n";
            }
            let ir = compile(&source);
            for target in [Target::Rust, Target::Go] {
                let result = synthesize_for(&ir, target);
                if reading {
                    let failure = result.err().expect("clock semantics must not be erased");
                    let value = serde_json::to_string(&failure).unwrap();
                    assert!(value.contains("selection-constraint"), "{value}");
                    assert!(value.contains("clock-reading"), "{value}");
                    assert!(value.contains("ess-target-failure/3"), "{value}");
                } else {
                    assert!(result.is_ok(), "ordinary typed aliases remain executable");
                }
            }
        }
    }
}
