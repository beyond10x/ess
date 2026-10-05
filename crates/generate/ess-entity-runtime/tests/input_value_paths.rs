//! A value read through an input path (`input.opening.label`, source `ess/22`, Family F A4) has no
//! known entity-core reach into a member of a structured argument, so lowering refuses it by name
//! (`docs/design/expression-family-source22.md`, "Target and projection obligations"), instead of
//! lowering an argument whose name merely contains dots.
use std::collections::BTreeMap;
use std::num::NonZeroU32;

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::subset::CONSTRUCTS;
use ess_entity_runtime::{lower_component, ComponentLoweringError, LoweringCode, LoweringOptions};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/input-value-paths.yaml");

const COMPONENTS: &str = "
components:
  - component: lease-service
    owns: {domains: [leases.pool]}
    accepts: {commands: [leases.pool.Open]}
    publishes: {events: [leases.pool.Opened]}
";

#[test]
fn a4_a_value_read_through_an_input_path_is_refused_by_name() {
    let text = format!("{MODEL}{COMPONENTS}");
    let spec = Specification::assemble([(
        Source::new("leases.yaml"),
        RawSpecFile::parse(&text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let options = LoweringOptions {
        definition_versions: [(
            QualifiedName::new("leases.pool.Lease").unwrap(),
            NonZeroU32::new(1).unwrap(),
        )]
        .into_iter()
        .collect(),
        scales: BTreeMap::new(),
    };
    let diagnostics =
        match lower_component(&ir, &ComponentName::new("lease-service").unwrap(), &options) {
            Ok(_) => panic!("a value read through an input path is not lowered"),
            Err(ComponentLoweringError::Lowering(diagnostics)) => diagnostics.into_vec(),
            Err(other) => panic!("the component extracts: {other:?}"),
        };
    let paths: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == LoweringCode::ValueExpressionUnsupported
                && diagnostic.construct == "a value read through an input path (`input.<path>`)"
        })
        .collect();
    assert_ne!(paths.len(), 0, "{diagnostics:#?}");
    assert!(
        paths
            .iter()
            .all(|diagnostic| diagnostic.path.starts_with("leases.pool.Open.")),
        "{paths:#?}"
    );
    for diagnostic in &diagnostics {
        assert!(
            CONSTRUCTS
                .iter()
                .any(|construct| construct.name == diagnostic.construct)
                || LoweringCode::ALL
                    .iter()
                    .any(|code| !code.is_source_construct()
                        && code.construct() == diagnostic.construct),
            "{diagnostic:?} names a construct the table does not list"
        );
    }
}
