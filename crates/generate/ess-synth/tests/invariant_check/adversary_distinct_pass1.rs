//! Adversary, pass 1, for
//! `story:generated-rust-and-go-decoders-check-a-structs-distinct-invariant`.
//!
//! Acceptance 3 asks for the reported shape as the regression model: six structs, each declaring
//! `distinct:`, nine `distinct:` invariants in all. This reads the unit's model and counts the
//! structs that declare one.

use super::ir;
use ess_compiler::ir::ResolvedBody;

/// Acceptance 3: the regression model holds the reported shape, six structs that each declare
/// `distinct:` and nine `distinct:` invariants among them. The unit's own check counts every
/// struct of the model, two of which declare no invariant at all.
#[test]
fn adv_the_regression_model_has_six_structs_that_each_declare_distinct() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/invariant_check/distinct_invariants.rs"
    ))
    .expect("the unit's test file reads");
    let start = source
        .find("const MODEL: &str = \"")
        .expect("the unit declares its model")
        + "const MODEL: &str = \"".len();
    let length = source[start..]
        .find("\";\n")
        .expect("the model literal closes");
    let model = ir(&source[start..start + length]);
    let mut declaring = Vec::new();
    let mut distinct = 0;
    for declared in model.types().values() {
        if let ResolvedBody::Struct { invariants, .. } = &declared.body {
            let reading = invariants
                .iter()
                .filter(|invariant| invariant.predicate.reads_distinct())
                .count();
            if reading > 0 {
                declaring.push(declared.name.to_string());
            }
            distinct += reading;
        }
    }
    assert_eq!(
        (declaring.len(), distinct),
        (6, 9),
        "structs declaring distinct: {declaring:?}"
    );
}
