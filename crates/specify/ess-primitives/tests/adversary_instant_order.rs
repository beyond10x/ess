//! Adversarial cases against ordering declared `Timestamp` facts by instant (beyond10x/ess#74).

use ess_primitives::facts::{FactPath, FactSource, FactStore, FactValue};
use ess_primitives::{Predicate, Truth};

/// A source that declares every `window.*` path a Timestamp, as the conformance input source does.
struct Declared(FactStore);

impl FactSource for Declared {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.0.fact(path)
    }

    fn orders_as_instant(&self, path: &FactPath) -> bool {
        path.namespace() == "window"
    }
}

fn window(starts_at: &str, ends_at: &str) -> Declared {
    let mut store = FactStore::new();
    store.set_path("window.starts_at", FactValue::text(starts_at));
    store.set_path("window.ends_at", FactValue::text(ends_at));
    Declared(store)
}

fn truth(expression: &str, facts: &Declared) -> Truth {
    Predicate::parse_expression(expression)
        .expect("parses")
        .evaluate(facts)
}

/// Trichotomy: two declared Timestamps naming one instant are `<=` and `>=` each other, so they
/// must also be `==`. Otherwise `>`, `==` and `<` outcomes all miss the same input.
#[test]
fn equal_instants_spelled_differently_are_equal_under_every_operator() {
    let facts = window("2020-01-01T01:00:00+01:00", "2020-01-01T00:00:00Z");
    assert_eq!(
        truth("window.ends_at >= window.starts_at", &facts),
        Truth::True
    );
    assert_eq!(
        truth("window.ends_at <= window.starts_at", &facts),
        Truth::True
    );
    assert_eq!(
        truth("window.ends_at == window.starts_at", &facts),
        Truth::True,
        "`>=` and `<=` both hold, so the instants are equal, but `==` compares spellings"
    );
    assert_eq!(
        truth("window.ends_at != window.starts_at", &facts),
        Truth::False
    );
}

/// Fractional seconds, lowercase separators and `-00:00` order as the instants they name.
#[test]
fn instant_spellings_order_by_the_moment() {
    let facts = window(
        "2020-01-01t00:00:00.999999999z",
        "2020-01-01T00:00:01-00:00",
    );
    assert_eq!(
        truth("window.ends_at > window.starts_at", &facts),
        Truth::True
    );
    let facts = window("2020-01-01T23:30:00-01:00", "2020-01-02T00:00:00.1Z");
    assert_eq!(
        truth("window.ends_at < window.starts_at", &facts),
        Truth::True
    );
}
