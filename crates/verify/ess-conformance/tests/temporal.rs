//! Complete live intervals must reject counterexamples, lost frames and stale anchors.
use ess_conformance::temporal::{Batch, Error, Ledger, Matcher, Occurrence};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

fn fields(values: &[(&str, &str)]) -> BTreeMap<String, Node> {
    values
        .iter()
        .map(|(key, value)| ((*key).into(), Node::Text((*value).into())))
        .collect()
}

fn event(sequence: u64, at_ms: u64, name: &str, values: &[(&str, &str)]) -> Occurrence {
    Occurrence {
        sequence,
        at_ms,
        event: format!("calls.live.{name}").parse().unwrap(),
        payload: fields(values),
    }
}

fn matcher(name: &str, values: &[(&str, &str)]) -> Matcher {
    Matcher {
        event: format!("calls.live.{name}").parse().unwrap(),
        payload: fields(values),
    }
}

fn batch(after: u64, complete_before_ms: u64, occurrences: Vec<Occurrence>) -> Batch {
    Batch {
        lifetime: "subscribed-before-stimulus".into(),
        after,
        complete_before_ms,
        occurrences,
    }
}

fn ledger() -> Ledger {
    Ledger::new("subscribed-before-stimulus".into()).unwrap()
}

#[test]
fn nested_snapshots_cannot_hide_a_transient_change_behind_a_parent_filter() {
    let mut history = ledger();
    let records = ["3", "0", "3"]
        .into_iter()
        .enumerate()
        .map(|(index, waiting)| {
            let mut item = event(index as u64 + 1, index as u64 + 1, "Metrics", &[]);
            item.payload.insert(
                "snapshot".into(),
                Node::Map(fields(&[("account", "owned"), ("waiting", waiting)])),
            );
            item
        })
        .collect();
    history.accept(batch(0, 20, records)).unwrap();
    let scope = matcher("Metrics", &[("snapshot.account", "owned")]);
    let required = fields(&[("snapshot.waiting", "3")]);
    assert!(matches!(
        history.stable(&scope, &required, 1, 10),
        Err(Error::Counterexample(Occurrence { sequence: 2, .. }))
    ));
    let mut filtered = matcher("Metrics", &[]);
    filtered.payload.insert(
        "snapshot".into(),
        Node::Map(fields(&[("account", "owned"), ("waiting", "3")])),
    );
    assert_eq!(
        history.stable(&filtered, &required, 1, 10),
        Err(Error::FilteredClaim)
    );
    let missing = matcher("Metrics", &[("snapshot.unknown", "3")]);
    assert!(history.find(&missing, 0).unwrap().is_none());
}

#[test]
fn actual_payload_and_occurrence_identity_control_matching_and_order() {
    let mut history = ledger();
    history
        .accept(batch(
            0,
            4,
            vec![
                event(1, 1, "Bridged", &[("call", "other"), ("agent", "alice")]),
                event(2, 2, "Bridged", &[("call", "owned"), ("agent", "bob")]),
                event(3, 3, "Bridged", &[("call", "owned"), ("agent", "alice")]),
                event(4, 3, "Bridged", &[("call", "owned"), ("agent", "alice")]),
            ],
        ))
        .unwrap();
    let claim = matcher("Bridged", &[("call", "owned"), ("agent", "alice")]);
    assert_eq!(history.find(&claim, 0).unwrap().unwrap().sequence, 3);
    assert_eq!(history.find(&claim, 3).unwrap().unwrap().sequence, 4);
    assert!(history.find(&claim, 4).unwrap().is_none());
    assert!(history.ordered(3, 4).is_ok());
    assert!(matches!(history.ordered(4, 3), Err(Error::Order { .. })));
    assert!(matches!(history.ordered(3, 3), Err(Error::Order { .. })));
    assert_eq!(history.find(&claim, 9), Err(Error::UnknownAnchor(9)));
}

#[test]
fn absence_needs_the_whole_window_and_uses_the_observed_anchor() {
    let mut history = ledger();
    history
        .accept(batch(
            0,
            101,
            vec![event(1, 100, "WrapUp", &[("agent", "alice")])],
        ))
        .unwrap();
    let offer = matcher("Offered", &[("agent", "alice")]);
    assert_eq!(
        history.absent(&offer, 1, 20_000),
        Err(Error::Unfinished {
            required_ms: 20_100,
            observed_ms: 101,
        })
    );
    history
        .accept(batch(
            1,
            20_100,
            vec![event(2, 200, "Offered", &[("agent", "bob")])],
        ))
        .unwrap();
    assert!(history.absent(&offer, 1, 20_000).is_ok());
    history
        .accept(batch(
            2,
            20_101,
            vec![event(3, 20_100, "Offered", &[("agent", "alice")])],
        ))
        .unwrap();
    assert!(history.absent(&offer, 1, 20_000).is_ok());
    assert_eq!(history.absent(&offer, 1, 0), Err(Error::InvalidWindow));
    assert_eq!(
        history.absent(&offer, 1, u64::MAX),
        Err(Error::InvalidWindow)
    );
}

#[test]
fn every_early_offer_is_a_counterexample_including_the_anchors_clock_tick() {
    for at_ms in [100, 101, 5_000, 20_099] {
        let mut history = ledger();
        let early = event(2, at_ms, "Offered", &[("agent", "alice")]);
        history
            .accept(batch(
                0,
                20_100,
                vec![event(1, 100, "WrapUp", &[]), early.clone()],
            ))
            .unwrap();
        assert_eq!(
            history.absent(&matcher("Offered", &[("agent", "alice")]), 1, 20_000),
            Err(Error::Counterexample(early))
        );
    }
}

#[test]
fn stability_checks_the_baseline_and_every_snapshot_including_a_transient_zero() {
    let scope = matcher("Metrics", &[("queue", "owned")]);
    let required = fields(&[("waiting", "3")]);
    for incorrect in 0..3 {
        let mut records = vec![
            event(1, 100, "Metrics", &[("queue", "owned"), ("waiting", "3")]),
            event(2, 500, "Metrics", &[("queue", "owned"), ("waiting", "3")]),
            event(
                3,
                10_099,
                "Metrics",
                &[("queue", "owned"), ("waiting", "3")],
            ),
        ];
        records[incorrect]
            .payload
            .insert("waiting".into(), Node::Text("0".into()));
        let wrong = records[incorrect].clone();
        let mut history = ledger();
        history.accept(batch(0, 10_100, records)).unwrap();
        assert_eq!(
            history.stable(&scope, &required, 1, 10_000),
            Err(Error::Counterexample(wrong))
        );
    }
    let mut history = ledger();
    history
        .accept(batch(
            0,
            10_100,
            vec![
                event(1, 100, "Metrics", &[("queue", "owned"), ("waiting", "3")]),
                event(2, 500, "Metrics", &[("queue", "other"), ("waiting", "0")]),
            ],
        ))
        .unwrap();
    assert!(history.stable(&scope, &required, 1, 10_000).is_ok());
    assert_eq!(
        history.stable(&scope, &BTreeMap::new(), 1, 10_000),
        Err(Error::EmptyClaim)
    );
    assert_eq!(
        history.stable(
            &matcher("Metrics", &[("waiting", "3")]),
            &required,
            1,
            10_000
        ),
        Err(Error::FilteredClaim)
    );
}

#[test]
fn malformed_batches_are_atomic_and_poison_the_lifetime() {
    let bad_batches = [
        Batch {
            lifetime: "reconnected".into(),
            ..batch(1, 4, vec![])
        },
        batch(0, 4, vec![]),
        batch(1, 1, vec![]),
        batch(1, 4, vec![event(3, 3, "Bridged", &[])]),
        batch(1, 4, vec![event(1, 3, "Bridged", &[])]),
        batch(1, 4, vec![event(2, 1, "Bridged", &[])]),
        batch(1, 4, vec![event(2, 4, "Bridged", &[])]),
        batch(
            1,
            5,
            vec![event(2, 3, "Bridged", &[]), event(3, 2, "Bridged", &[])],
        ),
    ];
    for bad in bad_batches {
        let mut history = ledger();
        history
            .accept(batch(0, 2, vec![event(1, 1, "WrapUp", &[])]))
            .unwrap();
        let failure = history.accept(bad).unwrap_err();
        assert!(matches!(failure, Error::Incomplete(_)));
        assert_eq!(
            history.cursor(),
            1,
            "a partially admitted batch could fabricate a match"
        );
        assert_eq!(
            history.accept(batch(1, 20_100, vec![])),
            Err(failure.clone())
        );
        assert_eq!(
            history.absent(&matcher("Offered", &[]), 1, 20_000),
            Err(failure)
        );
    }
}

#[test]
fn disconnect_decode_overflow_and_cancellation_cannot_be_repaired_by_later_good_frames() {
    for reason in ["disconnect", "decode", "overflow", "canceled"] {
        let mut history = ledger();
        history
            .accept(batch(0, 2, vec![event(1, 1, "WrapUp", &[])]))
            .unwrap();
        let failure = history.invalidate(reason);
        assert_eq!(history.invalidate("later error"), failure);
        assert_eq!(
            history.accept(batch(1, 30_000, vec![])),
            Err(failure.clone())
        );
        assert_eq!(
            history.find(&matcher("WrapUp", &[]), 0),
            Err(failure.clone())
        );
        assert_eq!(
            history.absent(&matcher("Offered", &[]), 1, 20_000),
            Err(failure)
        );
    }
}
