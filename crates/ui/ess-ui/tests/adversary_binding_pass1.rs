//! Adversary pass 1 on story:ui-binding-contract: `classify` against the body shapes the Go and
//! Rust servers emit (`crates/generate/ess-synth/src/{go,rust}/http.rs`), and the answer's JSON as
//! the TypeScript port and the vectors file read it.

use ess_ui::binding::{classify, Answer};

fn refused(error: &str, payload: Option<serde_json::Value>) -> Answer {
    Answer::Refused {
        error: error.to_owned(),
        payload,
    }
}

#[test]
fn adv1_declared_answers_win_over_surface_shapes_at_every_shared_status() {
    // A declared branch whose error and outcome are spelled like surface refusals.
    assert_eq!(
        classify(
            403,
            r#"{"outcome":"refused","published":[],"error":"not granted"}"#
        ),
        refused("not granted", None)
    );
    // The unknown-instance answer (`404`, nothing published, no payload) is declared, not a path.
    assert_eq!(
        classify(
            404,
            r#"{"outcome":"unknown","published":[],"error":"demo.visit.NoSuchVisit"}"#
        ),
        refused("demo.visit.NoSuchVisit", None)
    );
    // An external branch's refusal is a `502` that is still a declared answer.
    assert_eq!(
        classify(
            502,
            r#"{"error":"demo.pay.Declined","outcome":"declined","published":[]}"#
        ),
        refused("demo.pay.Declined", None)
    );
    assert_eq!(
        classify(
            409,
            r#"{"outcome":"wrong-state","published":[],"error":"E"}"#
        ),
        classify(
            422,
            r#"{"outcome":"wrong-state","published":[],"error":"E"}"#
        )
    );
}

#[test]
fn adv1_shapes_no_server_emits_are_transport() {
    for (status, body) in [
        (
            202,
            r#"{"outcome":"registered","published":[],"error":"E"}"#,
        ),
        (
            202,
            r#"{"outcome":"registered","published":[],"error":null}"#,
        ),
        (422, r#"{"outcome":"refused","published":[]}"#),
        (202, ""),
        (204, ""),
        (200, "[]"),
        (200, "null"),
        (418, r#"{"outcome":1}"#),
        (502, "<html>bad gateway</html>"),
        (504, r#"{"message":"gateway timeout"}"#),
    ] {
        assert_eq!(
            classify(status, body),
            Answer::Transport,
            "{status} {body:?}"
        );
    }
}

#[test]
fn adv1_surface_refusals_classify_by_members_and_status() {
    assert_eq!(
        classify(403, r#"{"actor":null,"refused":"not granted"}"#),
        Answer::NotGranted { actor: None }
    );
    // `actor` outside a `403` is no grant refusal.
    assert_eq!(
        classify(400, r#"{"refused":"x","actor":"a"}"#),
        Answer::Malformed {
            refused: "x".to_owned()
        }
    );
    assert_eq!(
        classify(501, r#"{"committed":true,"refused":"delivering"}"#),
        Answer::Unfinished { committed: true }
    );
    // `committed` outside a `501` is not an unfinished realization.
    assert_eq!(
        classify(500, r#"{"refused":"x","committed":true}"#),
        Answer::Malformed {
            refused: "x".to_owned()
        }
    );
    // The Rust server's request-level refusals.
    for status in [411, 413, 431] {
        assert_eq!(
            classify(status, r#"{"refused":"too much"}"#),
            Answer::Malformed {
                refused: "too much".to_owned()
            }
        );
    }
}

/// The vectors file and the TypeScript port compare answers as JSON. An answer `classify` returns
/// must survive its own serialisation, or a case whose expected answer is written from it can
/// never pass.
#[test]
fn adv1_every_classified_answer_round_trips_through_its_json() {
    for (status, body) in [
        (202, r#"{"outcome":"registered","published":[]}"#),
        (
            422,
            r#"{"outcome":"refused","published":[],"error":"E","payload":{"submitted":0}}"#,
        ),
        (
            422,
            r#"{"outcome":"refused","published":[],"error":"E","payload":null}"#,
        ),
        (403, r#"{"refused":"not granted","actor":null}"#),
        (403, r#"{"refused":"not granted","actor":"a.b.C"}"#),
        (501, r#"{"refused":"x","committed":false}"#),
        (404, r#"{"refused":"no such path"}"#),
        (502, "<html/>"),
    ] {
        let answer = classify(status, body);
        let json = serde_json::to_string(&answer).expect("serialises");
        let back: Answer = serde_json::from_str(&json).expect("deserialises");
        assert_eq!(back, answer, "{status} {body:?} serialised as {json}");
    }
}
