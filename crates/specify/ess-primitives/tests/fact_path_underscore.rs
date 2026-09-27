//! A fact path may begin with a field whose name begins with an underscore (beyond10x/ess#141).
//!
//! A command input field is the first segment of the fact path a guard reads it through, so a
//! field the specification admits (`^_*[A-Za-z][A-Za-z0-9_]*$`) has to be a path the fact store
//! admits. The first character rule becomes "underscores, then a letter"; every other rule of the
//! grammar is unchanged.

use ess_primitives::facts::FactPath;

#[test]
fn a_path_may_begin_with_underscores_before_a_letter() {
    for path in [
        "_url",
        "__v",
        "_url.host",
        "_Url",
        "_a1_b.c-d",
        "input._url",
    ] {
        FactPath::new(path).unwrap_or_else(|error| panic!("{path:?} is a fact path: {error}"));
    }
}

#[test]
fn underscores_alone_or_before_anything_but_a_letter_are_still_refused() {
    for path in ["_", "__", "_1", "_-a", "_.a", "1a", "-a", ""] {
        FactPath::new(path).expect_err(path);
    }
}

#[test]
fn a_path_with_a_leading_underscore_round_trips_through_its_segments() {
    let path = FactPath::new("_url.host").unwrap();
    assert_eq!(path.to_string(), "_url.host");
    assert_eq!(FactPath::from_segments(["_url", "host"]), path);
}
