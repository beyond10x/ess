//! Second adversarial pass on `story:concurrent-history-lanes`: where the page draws a
//! linearization point, held to `lanes.rs`'s own module documentation ("drawn inside the
//! operation's interval, after the point of the operation ordered before it on the same subject").

use std::collections::BTreeMap;

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::history::{self, Verdict};
use ess_conformance::lanes;
use ess_conformance::linearize::{self, DEFAULT_BUDGET};
use ess_conformance::scenario::SuiteProvenance;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// Every `(subject, position) -> cx` the page draws.
fn points(page: &str) -> BTreeMap<(String, usize), u64> {
    let mut found = BTreeMap::new();
    for rest in page.split("<circle class=\"point\" cx=\"").skip(1) {
        let cx: u64 = rest[..rest.find('"').expect("a closed cx")]
            .parse()
            .expect("cx is an integer");
        let title = rest
            .split("<title>linearization point ")
            .nth(1)
            .expect("a point carries its title");
        let (position, rest) = title.split_once(" of subject `").expect("a subject");
        let subject = &rest[..rest.find('`').expect("a closed subject")];
        found.insert(
            (subject.to_owned(), position.parse().expect("a position")),
            cx,
        );
    }
    found
}

#[test]
fn points_of_one_subject_stay_in_their_order_when_calls_share_an_instant() {
    // `history.rs`: instants are readings of a clock the writer chooses, "milliseconds,
    // nanoseconds or a logical counter all serve". A millisecond runner against a fast target
    // records calls that invoke and return in the same millisecond. Here every call of the
    // committed linearizable register history does, which the checker admits and still finds
    // linearizable: calls at one instant do not precede one another.
    let raw = RawSpecFile::parse(include_str!("fixtures/register/register.yaml"))
        .expect("the register fixture parses");
    let specification = Specification::assemble([(Source::new("register.yaml"), raw)])
        .expect("the register fixture validates");
    let model = compile(&specification, &SourceMap::new()).expect("the register fixture resolves");
    let digest = SuiteProvenance::of(&model).spec_digest;
    let committed = history::read(
        include_bytes!("fixtures/register/linearizable.json"),
        &digest,
    )
    .expect("the register history is admitted");
    let mut same_instant = committed.clone();
    for operation in &mut same_instant.operations {
        operation.invoked_at = 7;
        if operation.returned_at.is_some() {
            operation.returned_at = Some(7);
        }
    }
    let history = history::read(
        &serde_json::to_vec(&same_instant).expect("serializes"),
        &digest,
    )
    .unwrap_or_else(|refusal| panic!("one instant for every call is admitted: {refusal}"));
    let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
    assert_eq!(checked.verdict, Verdict::Linearizable);

    let page = lanes::render(&model, &history, DEFAULT_BUDGET).expect("rendered");
    let points = points(&page);
    assert!(points.len() >= 2, "the page draws the order found:\n{page}");
    let mut previous: Option<((String, usize), u64)> = None;
    for (key, cx) in &points {
        if let Some((before, before_cx)) = &previous {
            if before.0 == key.0 {
                assert!(
                    cx > before_cx,
                    "point {} of subject `{}` is drawn at x={cx}, not after point {} at x={before_cx}; \
                     every point: {points:?}",
                    key.1,
                    key.0,
                    before.1
                );
            }
        }
        previous = Some((key.clone(), *cx));
    }
}
