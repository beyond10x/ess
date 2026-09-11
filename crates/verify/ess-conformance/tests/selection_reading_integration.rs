//! A selection certificate cannot reconstruct clock-bearing declarations without their contract.
use ess_compiler::{ir::ResolvedMappingValue, resolve::compile_locating, source::SourceMap};
use ess_conformance::selection::Observation;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
#[test]
fn observation_refuses_clock_contract_reachable_through_unread_alias_member() {
    for reading in [false, true] {
        let contract = if reading {
            "    reading:\n      encoding: offset_date_time_text\n      origins: [{role: producer_process, offset: encoded_offset}]\n"
        } else {
            ""
        };
        let model = include_str!("fixtures/binding-selection.yaml")
            .replace("      - name: from\n        type: String", "      - name: from\n        type: selection.core.ClockAlias")
            .replace("events:\n", &format!("  - name: selection.core.ClockValue\n    kind: newtype\n    of: String\n{contract}  - name: selection.core.ClockAlias\n    kind: newtype\n    of: selection.core.ClockValue\nevents:\n"));
        let spec = Specification::assemble([(
            Source::new("selection-reading.yaml"),
            RawSpecFile::parse(&model).unwrap(),
        )])
        .unwrap();
        let mut sources = SourceMap::new();
        sources.insert("selection-reading.yaml", model);
        let ir = compile_locating(&spec, &sources, &["selection-reading.yaml"]).unwrap();
        let binding = ir.bindings().values().next().unwrap();
        let mapped = &binding.mapping[0];
        let ResolvedMappingValue::Selection {
            selector,
            projection,
            ..
        } = &mapped.value
        else {
            panic!("selection fixture")
        };
        let observation = Observation::of(&ir, binding, *selector, projection, &mapped.target_type);
        if reading {
            let refusal =
                observation.expect_err("the clock attachment must survive capability admission");
            assert!(refusal.contains("selection-constraint"), "{refusal}");
            assert!(refusal.contains("clock-reading"), "{refusal}");
        } else {
            assert!(
                observation.is_ok(),
                "ordinary alias facts remain observable"
            );
        }
    }
}
