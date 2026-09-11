//! An admitted literal must remain readable when the canonical predicate writer emits it.
use ess_primitives::{
    facts::FactValue,
    node::Node,
    predicate::{Operand, Predicate},
};

#[test]
fn admitted_quoted_text_roundtrips_through_the_actual_canonical_writer() {
    let structured: Node = serde_json::from_value(serde_json::json!({
        "to": {"eq": "\"busy\" status"}
    }))
    .unwrap();
    let compact = Node::Text(r#"to == '"busy" status'"#.to_owned());
    for input in [structured, compact] {
        let admitted = Predicate::from_node(&input).expect("valid literal data is admitted");
        let Predicate::Compare { ref right, .. } = admitted else {
            panic!("comparison")
        };
        assert_eq!(
            right,
            &Operand::Literal(FactValue::Text("\"busy\" status".to_owned()))
        );
        let emitted = admitted.to_node();
        let reread = Predicate::from_node(&emitted);
        assert!(
            reread.is_ok(),
            "admitted literal serialized into refused bytes: {emitted:?}: {reread:?}"
        );
        assert_eq!(reread.unwrap(), admitted, "literal meaning is preserved");
    }
}
