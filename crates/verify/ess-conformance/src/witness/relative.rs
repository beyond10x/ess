//! A goal's order between two `Integer` facts of the input, solved by choosing one fact relative
//! to the other (beyond10x/ess#622).
//!
//! The bounded walk varies each leaf on its own ladder, so `amount <= limit` refuted beside
//! `limit > 0` and `amount > 0` is met only where some candidate happens to put `amount` above
//! `limit`. Here the goal's comparisons over `Integer` leaves — between two of them, or between one
//! and a literal — and the declared ranges of those leaves are read as bounds on differences,
//! `x - y <= k`, and a candidate's values are moved along them: each value is lowered to what the
//! bounds over it allow, and every value is then shifted so that zero stays zero. A cycle of bounds
//! that lowers a value without end is a goal no `Integer` input satisfies.
//!
//! Only conjuncts are read. A comparison under an `any`, a `!=`, a decimal or a text leaf is left
//! out, so the bounds are a relaxation of the goal: a goal they refute is refuted, and a candidate
//! they place is checked against the whole goal by the caller before it is used.

use std::collections::BTreeMap;

use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{CompareKind, CompareOp, Operand, Predicate};

/// The bound `value[to] <= value[from] + weight`, where `None` is the constant zero.
#[derive(Debug, Clone, Copy)]
struct Bound {
    to: Option<usize>,
    from: Option<usize>,
    weight: i128,
}

/// One side of a comparison: a leaf, by its index, or an integer constant.
#[derive(Debug, Clone, Copy)]
enum Term {
    Leaf(usize),
    Constant(i128),
}

/// The bounds a goal places on its `Integer` leaves.
#[derive(Debug, Default)]
pub(super) struct Bounds {
    /// The leaves the bounds read, by index.
    paths: Vec<FactPath>,
    /// Every bound read, each as `to <= from + weight`.
    limits: Vec<Bound>,
    /// Whether some bound compares two leaves with each other.
    relative: bool,
}

impl Bounds {
    /// The bounds of `goal`'s conjuncts over the leaves `integral` admits, and of each such leaf's
    /// declared range, read from `ranges` against `value`. `None` where no conjunct compares two
    /// such leaves with each other, so every other goal is left to the search alone.
    pub(super) fn of(
        goal: &Predicate,
        integral: &dyn Fn(&FactPath) -> bool,
        ranges: &dyn Fn(&FactPath) -> Vec<Predicate>,
    ) -> Option<Self> {
        let mut bounds = Self::default();
        let mut atoms = Vec::new();
        conjuncts(goal, true, &mut atoms);
        for (left, op, right) in &atoms {
            bounds.compare(left, *op, right, integral, None);
        }
        if !bounds.relative {
            return None;
        }
        let read = bounds.paths.clone();
        let value = FactPath::new("value").ok()?;
        for path in &read {
            for range in ranges(path) {
                let mut atoms = Vec::new();
                conjuncts(&range, true, &mut atoms);
                for (left, op, right) in &atoms {
                    bounds.compare(left, *op, right, &|read| *read == value, Some(path));
                }
            }
        }
        Some(bounds)
    }

    /// The index of `path`, added where it is new.
    fn leaf(&mut self, path: &FactPath) -> usize {
        if let Some(at) = self.paths.iter().position(|held| held == path) {
            return at;
        }
        self.paths.push(path.clone());
        self.paths.len() - 1
    }

    /// The bounds of `left op right`, where each side is a leaf `integral` admits or an integer
    /// literal; with `at`, a range read against `value`, which names the leaf at `at`.
    fn compare(
        &mut self,
        left: &Operand,
        op: CompareOp,
        right: &Operand,
        integral: &dyn Fn(&FactPath) -> bool,
        at: Option<&FactPath>,
    ) {
        let mut term = |operand: &Operand| match operand {
            Operand::Fact(path) if integral(path) => {
                Some(Term::Leaf(self.leaf(at.unwrap_or(path))))
            }
            Operand::Literal(FactValue::Number(number)) => number
                .as_i64()
                .map(|value| Term::Constant(i128::from(value))),
            _ => None,
        };
        let (Some(left), Some(right)) = (term(left), term(right)) else {
            return;
        };
        // `left - right op 0`, as `to <= from + weight` once each side is a leaf or zero.
        let (to, from, offset) = match (left, right) {
            (Term::Leaf(to), Term::Leaf(from)) => {
                self.relative = true;
                (Some(to), Some(from), 0)
            }
            (Term::Leaf(to), Term::Constant(constant)) => (Some(to), None, constant),
            (Term::Constant(constant), Term::Leaf(from)) => (None, Some(from), -constant),
            (Term::Constant(_), Term::Constant(_)) => return,
        };
        let mut push = |to: Option<usize>, from: Option<usize>, weight: i128| {
            self.limits.push(Bound { to, from, weight });
        };
        match op {
            CompareOp::Le => push(to, from, offset),
            CompareOp::Lt => push(to, from, offset - 1),
            CompareOp::Ge => push(from, to, -offset),
            CompareOp::Gt => push(from, to, -offset - 1),
            CompareOp::Eq => {
                push(to, from, offset);
                push(from, to, -offset);
            }
            CompareOp::Ne => {}
        }
    }

    /// The values the bounds place each leaf at, moved from `near` — the values a candidate holds,
    /// zero where it holds none — by as little as lowering along the bounds moves them; `None`
    /// where the bounds admit no values at all, or none an `i64` holds.
    fn placed(&self, near: &[i128]) -> Result<Vec<i64>, Infeasible> {
        let zero = self.paths.len();
        let mut value: Vec<i128> = near.to_vec();
        value.push(0);
        let at = |index: Option<usize>| index.unwrap_or(zero);
        // A value still lowered after as many rounds as there are values lies on a cycle of
        // bounds whose weights sum below zero.
        for round in 0..=value.len() {
            let mut lowered = false;
            for bound in &self.limits {
                let limit = value[at(bound.from)].saturating_add(bound.weight);
                if value[at(bound.to)] > limit {
                    value[at(bound.to)] = limit;
                    lowered = true;
                }
            }
            if !lowered {
                break;
            }
            if round == value.len() {
                return Err(Infeasible::Bounds);
            }
        }
        let shift = value[zero];
        value[..zero]
            .iter()
            .map(|held| i64::try_from(held - shift).map_err(|_| Infeasible::Unrepresentable))
            .collect()
    }

    /// Whether the bounds admit no values at all: then no `Integer` input satisfies the goal.
    pub(super) fn infeasible(&self) -> bool {
        matches!(
            self.placed(&vec![0; self.paths.len()]),
            Err(Infeasible::Bounds)
        )
    }

    /// `input` with every leaf the bounds read moved to the value they place it at, starting from
    /// the value `input` holds there; `None` where they place none, or a leaf is not a number of
    /// the input's own.
    pub(super) fn moved(&self, input: &BTreeMap<String, Node>) -> Option<BTreeMap<String, Node>> {
        let near: Vec<i128> = self
            .paths
            .iter()
            .map(|path| match held(input, path) {
                Some(Node::Number(number)) => number.as_i64().map(i128::from),
                _ => None,
            })
            .collect::<Option<_>>()?;
        let placed = self.placed(&near).ok()?;
        let mut moved = input.clone();
        for (path, value) in self.paths.iter().zip(placed) {
            if !put(&mut moved, path, Node::Number(value.into())) {
                return None;
            }
        }
        Some(moved)
    }
}

/// Why no values were placed.
#[derive(Debug, PartialEq, Eq)]
enum Infeasible {
    /// The bounds contradict each other.
    Bounds,
    /// A value placed is beyond what an `i64` holds.
    Unrepresentable,
}

/// Every comparison `goal` holds as a conjunct, under `positive` or its negation, with the
/// operator negated where the comparison is: `not (a <= b)` is `a > b`, and `not` of an `any` holds
/// each of its children negated.
fn conjuncts<'p>(
    predicate: &'p Predicate,
    positive: bool,
    out: &mut Vec<(&'p Operand, CompareOp, &'p Operand)>,
) {
    match predicate {
        Predicate::All(children) if positive => {
            for child in children {
                conjuncts(child, true, out);
            }
        }
        Predicate::Any(children) if !positive => {
            for child in children {
                conjuncts(child, false, out);
            }
        }
        Predicate::Not(inner) => conjuncts(inner, !positive, out),
        Predicate::Compare {
            left,
            op,
            right,
            kind: CompareKind::Value,
        } => out.push((left, if positive { *op } else { negated(*op) }, right)),
        _ => {}
    }
}

fn negated(op: CompareOp) -> CompareOp {
    match op {
        CompareOp::Eq => CompareOp::Ne,
        CompareOp::Ne => CompareOp::Eq,
        CompareOp::Lt => CompareOp::Ge,
        CompareOp::Le => CompareOp::Gt,
        CompareOp::Gt => CompareOp::Le,
        CompareOp::Ge => CompareOp::Lt,
    }
}

/// The value `input` holds at `path`, through the members of its mappings.
fn held<'n>(input: &'n BTreeMap<String, Node>, path: &FactPath) -> Option<&'n Node> {
    let (first, rest) = path.segments().split_first()?;
    let mut node = input.get(first)?;
    for segment in rest {
        let Node::Map(entries) = node else {
            return None;
        };
        node = entries.get(segment)?;
    }
    Some(node)
}

/// `input` with `value` at `path`, where a value is already held there.
fn put(input: &mut BTreeMap<String, Node>, path: &FactPath, value: Node) -> bool {
    let Some((first, rest)) = path.segments().split_first() else {
        return false;
    };
    let Some(mut node) = input.get_mut(first) else {
        return false;
    };
    for segment in rest {
        let Node::Map(entries) = node else {
            return false;
        };
        let Some(next) = entries.get_mut(segment) else {
            return false;
        };
        node = next;
    }
    *node = value;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact(text: &str) -> Operand {
        Operand::Fact(FactPath::new(text).expect("a path"))
    }

    fn number(value: i64) -> Operand {
        Operand::Literal(FactValue::Number(value.into()))
    }

    fn compare(left: Operand, op: CompareOp, right: Operand) -> Predicate {
        Predicate::Compare {
            left,
            op,
            right,
            kind: CompareKind::Value,
        }
    }

    fn not(inner: Predicate) -> Predicate {
        Predicate::Not(Box::new(inner))
    }

    fn integers(_: &FactPath) -> bool {
        true
    }

    fn unranged(_: &FactPath) -> Vec<Predicate> {
        Vec::new()
    }

    fn input(limit: i64, amount: i64) -> BTreeMap<String, Node> {
        BTreeMap::from([
            ("limit".to_owned(), Node::Number(limit.into())),
            ("amount".to_owned(), Node::Number(amount.into())),
        ])
    }

    /// `limit > 0`, `amount > 0`, and `amount <= limit` held or refuted.
    fn terms(within: bool, limit_positive: bool) -> Predicate {
        let limit = compare(fact("limit"), CompareOp::Gt, number(0));
        let within_limit = compare(fact("amount"), CompareOp::Le, fact("limit"));
        Predicate::All(vec![
            if limit_positive { limit } else { not(limit) },
            compare(fact("amount"), CompareOp::Gt, number(0)),
            if within {
                within_limit
            } else {
                not(within_limit)
            },
        ])
    }

    #[test]
    fn an_order_between_two_leaves_is_met_relative_to_the_other() {
        let bounds =
            Bounds::of(&terms(false, true), &integers, &unranged).expect("two leaves compared");
        assert!(!bounds.infeasible());
        assert_eq!(bounds.moved(&input(2, 1)), Some(input(1, 2)));
        assert_eq!(bounds.moved(&input(5, 9)), Some(input(5, 9)));
    }

    #[test]
    fn contradicting_bounds_are_infeasible() {
        let bounds =
            Bounds::of(&terms(true, false), &integers, &unranged).expect("two leaves compared");
        assert!(bounds.infeasible());
        assert_eq!(bounds.moved(&input(1, 1)), None);
    }

    #[test]
    fn a_declared_range_bounds_the_values_placed() {
        let above = compare(fact("amount"), CompareOp::Gt, fact("limit"));
        let range = |low: i64, high: i64| {
            Predicate::All(vec![
                compare(fact("value"), CompareOp::Ge, number(low)),
                compare(fact("value"), CompareOp::Le, number(high)),
            ])
        };
        let ranges = |_: &FactPath| vec![range(3, 4)];
        let bounds = Bounds::of(&above, &integers, &ranges).expect("two leaves compared");
        assert_eq!(bounds.moved(&input(9, 0)), Some(input(3, 4)));
        let narrow = |_: &FactPath| vec![range(3, 3)];
        let bounds = Bounds::of(&above, &integers, &narrow).expect("two leaves compared");
        assert!(bounds.infeasible());
    }

    #[test]
    fn a_goal_comparing_no_two_leaves_is_left_to_the_search() {
        let literal_only = Predicate::All(vec![
            compare(fact("limit"), CompareOp::Gt, number(0)),
            compare(fact("amount"), CompareOp::Gt, number(0)),
        ]);
        assert!(Bounds::of(&literal_only, &integers, &unranged).is_none());
        let texts = |_: &FactPath| false;
        assert!(Bounds::of(&terms(false, true), &texts, &unranged).is_none());
        let under_any = Predicate::Any(vec![compare(fact("amount"), CompareOp::Le, fact("limit"))]);
        assert!(Bounds::of(&under_any, &integers, &unranged).is_none());
    }
}
