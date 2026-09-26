//! Where a scenario's concrete values come from, and where none can be produced.
//!
//! Design §11. A scenario needs an actual command input, and the rule it is built under is one
//! sentence:
//!
//! > Never generate an arbitrary value and claim it satisfies an outcome predicate unless the
//! > generator can prove or evaluate that it does.
//!
//! [`crate::input`] makes the second half available, so this module does not have to prove anything:
//! it produces candidates, and [`InputFacts::decide`](crate::InputFacts::decide) says whether one
//! reaches the branch. What is left is choosing candidates that are *worth* deciding, in a bounded,
//! deterministic order, without a constraint solver — which §11 names as a later extension and not a
//! requirement of the first closed loop.
//!
//! # The strategy, in five rules
//!
//! 1. **One base witness per command**, built from the declared input types alone. Every field is
//!    filled, optionals included, so a guard reading one is decided rather than
//!    [`Unknown`](crate::Reason::ValueAbsent). **The one exception is presence** (ess#93): for each
//!    `Optional` member a guard tests with `defined(x)` — `exists(x)`, `x: {exists: …}` and
//!    `not defined(x)` are the same test — one further candidate leaves that member out, so both
//!    sides of the test have a witness. Where `x` is not itself optional, the member left out is
//!    the deepest optional one `x` is read through. **Omitted means absent on the wire**: the key
//!    is missing from its mapping, never present as `null`; the flattener binds nothing for either,
//!    and absence is the one a runner can send without choosing an encoding for `null`. Omissions
//!    vary slowest, after every candidate that fills them, and each is reserved inside rule 4's
//!    bound so a guard with many varied leaves cannot crowd it out.
//! 2. **A text witness is its own fact path.** `contact` carries `"contact"` and
//!    `alternate_contact` carries `"alternate_contact"`, so two same-typed fields are never
//!    interchangeable — which is the only way a swapped binding mapping is a detectable fault rather
//!    than an invisible one (`examples/oracle-fixture/README.md`). **Under a declared `alphabet:`**
//!    (ess/11) each character of that text the alphabet does not hold becomes
//!    `alphabet[c mod |alphabet|]`, so the witness is a value the type admits. **An input's
//!    `example:`** is its base at the plain instance, for every leaf kind.
//! 3. **Alternatives come from the guard, not from imagination.** The values a candidate varies are
//!    the fact paths the guard actually reads, and the values it tries are the literals the guard
//!    itself writes, one either side. `amount.amount > 0` is met by `1` and refuted by `0`, and
//!    neither number was invented. **Text orders by its UTF-8 bytes** (ess#94), the same in the
//!    Rust, Go and TypeScript evaluators, so `caller < "m"` is a boundary like any number's: an
//!    ordered text is tried at the literal, at the literal with one byte appended, and at the
//!    empty text. **A list a guard reads is tried with one element** (ess#94): the base keeps it
//!    `[]`, one candidate carries a single element built at `<list>.0` like any other value, and a
//!    quantifier's body is rebound onto that element — `exists t in tags: t == vip` reads
//!    `tags.0 == vip` — so its literal is what the element is tried at. `[vip]` satisfies that
//!    guard; `[]` and `[tags.0]`, the element's own path as every text witness is, refute it; and
//!    `tags.count > 0` is decided by `[]` and `[tags.0]`, because the flattener publishes an input
//!    list's `count` the way an observed collection publishes one. A list a guard reads **by
//!    position** — `tags.0 == vip` — holds that many elements already in the base, because a read
//!    that misses is `Unknown` and ends the search; the element is then varied like any leaf.
//!    **A `.count` compared with `v`** (ess#104) is tried at lengths `⌊v⌋`, `⌊v⌋ + 1` and
//!    `⌊v⌋ − 1`, up to [`MAX_COUNT_WITNESS`]: a `String` resized from its base and its other
//!    alternatives, and a list of that many copies of its element 0.
//!    **A string operator** (ess#95) is tried at its literal `L`, which satisfies it; at each
//!    guard's positive literals composed around the path's own text `b` — `P + C + b + S`,
//!    `P + b + C + S` and `P + C + S`, never with a literal the guard negates — which satisfies
//!    every positive operator of that guard at once; at the same layouts of a newtype's own
//!    invariant literals, so a refuting value the type admits is among them; and at `L′`, `L` with
//!    one character replaced, which refutes it. `caller: {starts_with: "+44"}` over a
//!    `PhoneNumber` whose invariant is `value: {starts_with: "+"}` is met by `+44` and refuted by
//!    `+4x` and `+caller`; the base `caller` is refused by the type.
//! 4. **Bounded.** At most [`MAX_CANDIDATES`] distinct inputs per outcome; a repeat — an element
//!    varied while its list is empty — is skipped, not counted. Exhausting them is a refusal that
//!    says how many were tried, never a longer search.
//! 5. **A second instance is a second witness.** Rule 2 keeps two *fields* apart; [`Distinction`]
//!    keeps two *instances* apart, by moving every leaf the walk records as far as its declared
//!    type allows. Without it, a scenario that runs one creating command twice submits one input
//!    twice.
//!
//! # What has no witness
//!
//! A type that refers to itself, and a field whose name cannot be spelled as a fact path. Both are
//! [`WitnessGap`], and both are reported rather than worked around. Everything else in the model has
//! a value: a list is `[]` (one element where a guard reads into it, rule 3), a map is `{}`, a
//! union is its first variant in the encoding
//! `ess-gen` publishes, and an enum is its first declared variant.
//!
//! **A witness is only as good as the type it is built from.** `currency: String` with no invariant
//! gets the text `"amount.currency"`, because that is every value the specification permits. An
//! implementation that refuses it is enforcing a rule the specification does not state, and the
//! suite catching that is the suite working.
//!
//! **A whole number is written `1.0` in the artifact.** [`Node::Number`] holds an
//! [`ess_primitives::facts::Number`], which is an `f64`, so an `Integer` witness serialises with a
//! fractional part it does not have. That is the value type's shape rather than a choice here — the
//! flattener refuses a genuinely fractional value for an `Integer`
//! ([`crate::input`]) — but a runner handing this straight to a JSON-typed target has to normalise
//! it, and that is worth knowing before the runner is written.
//!
//! # This walk mirrors the flattener's
//!
//! [`crate::input`] walks a declared type beside a candidate *value* to bind facts; this walks the
//! same type to *build* one. Same table, opposite directions, no shared step to factor out — with
//! one difference that matters: nothing inside a union is recorded as a leaf, because a fact path
//! cannot reach inside one, so no guard can be varied there.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedCommand, ResolvedTypeRef};
use ess_domain::types::{Primitive, MAX_TYPE_DEPTH};
use ess_primitives::facts::{FactPath, FactValue, Number};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Operand, Predicate, TextOp};
use ess_primitives::time::Rfc3339Instant;

/// How many candidate inputs one outcome is tried against before synthesis refuses.
///
/// A bound rather than a budget to spend: §11 asks for a constrained strategy, and the honest
/// failure of one is "the values I know how to try did not satisfy this guard", reported with the
/// number. A larger number would turn a specification that needs a solver into a slower build that
/// still cannot say so.
pub const MAX_CANDIDATES: usize = 64;

/// Which of several instances of one entity a witness is being built for.
///
/// A view that declares an order is asserted against **two** rows or against nothing:
/// [`ViewExpectation::Ranked`](crate::ViewExpectation::Ranked) compares adjacent pairs, and a view
/// holding one row has no pair. So a scenario that asserts an order arranges further instances by
/// running the declared creating outcome again — and two runs of one command with one witness
/// submit one input twice, down to the field the entity's identity is supplied in.
///
/// A distinction moves every leaf the walk records: `0` is the plain witness every scenario's own
/// subject is built from, and each further instance takes the next number. What moves is bounded by
/// the declared type rather than by this module's imagination — a `String` grows a suffix, a
/// `Timestamp` moves a day, a number counts up — and a `Boolean` says out loud what a two-valued
/// type can do: [`Distinction`] `1` and `3` give it the same value, so two rows tie on that key
/// rather than pretend not to.
///
/// It is not a seed. The value at a distinction is a function of the path and the number, so two
/// runs over one model produce one suite (§37).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Distinction(usize);

impl Distinction {
    /// The witness a scenario's own subject is built from.
    pub const PLAIN: Self = Self(0);

    /// The witness for the `nth` further instance, counting from one.
    pub const fn further(nth: usize) -> Self {
        Self(nth)
    }

    /// How far from the plain witness this one is.
    pub const fn get(self) -> usize {
        self.0
    }
}

/// The wire key a union variant's value is carried under.
///
/// `{"kind": "person", "value": …}` — the encoding `ess-gen` publishes in
/// `generated/schema/types/billing.invoice.Payee.schema.json`, read from there rather than decided
/// again here, because a witness in a second encoding is a witness no target can accept.
const UNION_VALUE: &str = "value";

/// A construct of the input that no safe value could be produced for.
///
/// Two of them exist, and neither is a defect in this module: a type that refers to itself has no
/// finite value, and a field name that is not a fact path is a `CommandSpec` no document can
/// produce. Both travel as a refusal (§11) rather than as a value that would have been a guess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessGap {
    /// Where in the input it sits, as the path a guard would read it by.
    pub path: String,
    /// The type that has no witness, rendered.
    pub type_ref: String,
    /// Why it has none.
    pub reason: &'static str,
}

impl fmt::Display for WitnessGap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{}` is `{}`, which {}",
            self.path, self.type_ref, self.reason
        )
    }
}

/// What a fact path lands on in a command's input, when a candidate can vary it.
///
/// Only the four shapes a [`FactValue`] can hold, because those are the only ones a guard can
/// compare — the flattener binds nothing else, so varying anything else changes no decision.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Leaf {
    /// A `Decimal` or an `Integer`; `integral` when a fractional value would be refused.
    Number {
        /// `true` for `Integer`, where `1.5` is not a value of the type.
        integral: bool,
    },
    /// Anything a fact reads as text.
    Text,
    /// A `Timestamp`, ordered by the RFC 3339 instant it names.
    Timestamp,
    /// A `Boolean`.
    Bool,
    /// An enum, whose alternatives are its own declared variants.
    Enum {
        /// The variants, in declaration order.
        variants: Vec<String>,
    },
}

/// Every candidate input for one command, in the order synthesis tries them.
///
/// The base witness comes first when its declared types and invariants admit it. Candidates whose
/// complete values fail that check are omitted without changing the order of the remaining ones.
/// An empty result means none of this bounded strategy's candidates was admitted, not that no
/// value could ever satisfy the type's invariants.
///
/// `distinction` says which instance the input is for. [`Distinction::PLAIN`] is the witness every
/// scenario's own subject is built from; a further one moves the base value of every leaf, and the
/// ladder is built from that moved base rather than from the plain one — offering a candidate the
/// base already carries would spend a try on a value already decided.
///
/// # Errors
///
/// [`WitnessGap`] when some field of the input has no safe value at all.
pub fn candidates(
    ir: &EssIr,
    command: &ResolvedCommand,
    guards: &[&Predicate],
    distinction: Distinction,
) -> Result<Vec<BTreeMap<String, Node>>, WitnessGap> {
    // A quantifier's body reads its element through a binder. Rebound onto the element a one-
    // element list carries — `t == vip` over `tags` becomes `tags.0 == vip` — it is an ordinary
    // guard over an ordinary leaf, and rule 3 finds its literal like any other.
    let mut expanded: Vec<Predicate> = guards.iter().map(|guard| (*guard).clone()).collect();
    for guard in guards {
        element_bodies(guard, &mut expanded);
    }
    let expanded: Vec<&Predicate> = expanded.iter().collect();

    let mut builder = Builder::new(
        ir,
        distinction,
        list_reads(&expanded),
        // The guards as written, not the rebound quantifier bodies: `tags.0` in a body is the
        // element a candidate may add, not a position the author read.
        positional_reads(guards),
    );
    let base = builder.input(command, &BTreeMap::new())?;

    // A newly admitted no-default partition uses exactly the domain that validation proved.
    // Preserve the existing candidate order for commands with real defaults.
    if command
        .outcomes
        .iter()
        .all(|outcome| outcome.test_strategy != ess_domain::command::TestStrategy::DefaultBranch)
        && guards.iter().all(|guard| {
            command
                .outcomes
                .iter()
                .filter_map(crate::when)
                .any(|ordinary| ordinary == *guard)
        })
    {
        let all_guards: Vec<_> = command.outcomes.iter().filter_map(crate::when).collect();
        if let Some(cases) = ess_domain::command::finite::analyze(
            &ess_compiler::expression::Environment::new(ir, &command.input),
            &all_guards,
        ) {
            let inputs = cases
                .into_iter()
                .map(|case| {
                    let overrides = case
                        .values
                        .into_iter()
                        .map(|(path, value)| (path, Choice::Value(Node::Text(value))))
                        .collect();
                    builder.input(command, &overrides)
                })
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(admitted_inputs(ir, command, inputs));
        }
    }

    let mut ladders: BTreeMap<FactPath, Vec<Choice>> = BTreeMap::new();
    for path in read_paths(&expanded) {
        let Some((leaf, at_base)) = builder.leaves.get(&path) else {
            // A path the guard reads and the input does not bind as a scalar: a list's count, a
            // map, the inside of a union, or a segment no type declares. The list is varied below;
            // no value this module chooses changes the rest, and `InputFacts::decide` is what
            // names which of them it is.
            continue;
        };
        let mut alternatives = alternatives(
            leaf,
            at_base,
            &literals_at(&expanded, &path),
            ordered_at(&expanded, &path),
        );
        if let (Leaf::Text, Node::Text(base)) = (leaf, at_base) {
            let invariants = builder.invariants.get(&path).map_or(&[][..], Vec::as_slice);
            for text in text_alternatives(&expanded, invariants, &path, base) {
                let node = Node::Text(text);
                if &node != at_base && !alternatives.contains(&node) {
                    alternatives.push(node);
                }
            }
        }
        if !alternatives.is_empty() {
            ladders.insert(path, alternatives.into_iter().map(Choice::Value).collect());
        }
    }
    count_ladders(&builder, &expanded, &mut ladders);
    let mut ladders: Vec<(FactPath, Vec<Choice>)> = ladders.into_iter().collect();

    // Omissions vary slowest, so every candidate that fills the optionals is tried before any
    // that leaves one out: an omitted value makes a comparison that reads it `Unknown`, and an
    // `Unknown` ends the search for its outcome.
    let omitted = omissions(&expanded, &builder.optionals);
    ladders.extend(
        omitted
            .iter()
            .map(|path| (path.clone(), vec![Choice::Omit])),
    );
    // Rule 4 bounds the list of *distinct* candidates. An element's alternatives change nothing
    // while its list is empty, so the enumeration produces the same input more than once, and a
    // repeat is skipped rather than counted. Where the bound cuts the enumeration short, each
    // omission's own candidate — every other value at its base — is reserved inside it, so a guard
    // with many varied leaves cannot crowd the absent side of `defined(x)` out of it.
    let inputs = enumerate(&mut builder, command, base, &ladders, &omitted)?;
    Ok(admitted_inputs(ir, command, inputs))
}

/// The base, then the distinct candidates the ladders describe, then each omission alone — at most
/// [`MAX_CANDIDATES`] in all.
fn enumerate(
    builder: &mut Builder<'_>,
    command: &ResolvedCommand,
    base: BTreeMap<String, Node>,
    ladders: &[(FactPath, Vec<Choice>)],
    omitted: &[FactPath],
) -> Result<Vec<BTreeMap<String, Node>>, WitnessGap> {
    let total = product(ladders);
    let reserved = if total <= MAX_CANDIDATES {
        0
    } else {
        omitted.len()
    };
    let enumerated = MAX_CANDIDATES.saturating_sub(reserved).max(1);

    let mut inputs = vec![base];
    let mut index = 1;
    while index < total
        && inputs.len() < enumerated
        && index < MAX_CANDIDATES.saturating_mul(MAX_ENUMERATED_PER_CANDIDATE)
    {
        let mut overrides = BTreeMap::new();
        let mut remaining = index;
        for (path, alternatives) in ladders {
            let radix = alternatives.len() + 1;
            let chosen = remaining % radix;
            remaining /= radix;
            if chosen > 0 {
                overrides.insert(path.clone(), alternatives[chosen - 1].clone());
            }
        }
        let input = builder.input(command, &overrides)?;
        if !inputs.contains(&input) {
            inputs.push(input);
        }
        index += 1;
    }
    for path in omitted {
        if inputs.len() >= MAX_CANDIDATES {
            break;
        }
        let input = builder.input(command, &BTreeMap::from([(path.clone(), Choice::Omit)]))?;
        if !inputs.contains(&input) {
            inputs.push(input);
        }
    }
    Ok(inputs)
}

/// How many positions of the enumeration are walked, per candidate the bound allows, before
/// synthesis stops looking for distinct ones. Repeats are skipped, so without this a ladder whose
/// alternatives are all repeats would be walked to its full product.
const MAX_ENUMERATED_PER_CANDIDATE: usize = 16;

/// What a candidate puts at one fact path in place of the base witness.
#[derive(Debug, Clone, PartialEq)]
enum Choice {
    /// This value.
    Value(Node),
    /// Nothing: the `Optional` member at this path is left out of the input — absent from its
    /// mapping, never present as `null`.
    Omit,
    /// A list of this many elements: element 0 built at `<path>.0` from the declared element type
    /// like any other value and varied there by the same ladders, and every further element a
    /// copy of it, so a quantifier is decided by element 0 alone, as it was with one element.
    Elements(usize),
}

/// Filter only after building all bounded alternatives: an invalid base does not rule out later
/// values. The caller counts exactly these admitted candidates when deciding outcome guards.
///
/// An `Optional` field a candidate leaves out is admitted as the absence it is.
fn admitted_inputs(
    ir: &EssIr,
    command: &ResolvedCommand,
    mut inputs: Vec<BTreeMap<String, Node>>,
) -> Vec<BTreeMap<String, Node>> {
    inputs.retain(|input| {
        command
            .input
            .iter()
            .all(|field| match input.get(&field.name) {
                Some(value) => {
                    crate::input::validate_typed_value(ir, &field.type_ref, value).is_ok()
                }
                None => field.type_ref.is_optional(),
            })
    });
    inputs
}

/// How many candidates the ladders describe, capped at [`MAX_CANDIDATES`].
///
/// Each ladder has one more position than it has alternatives, because position zero is "leave the
/// base value alone". The product is saturating: a command with many varied fields is bounded, not
/// overflowed. Only a count now: the candidates enumerated skip repeats, so the list is at most
/// this long.
#[cfg(test)]
fn combinations<T>(ladders: &[(FactPath, Vec<T>)]) -> usize {
    product(ladders).min(MAX_CANDIDATES)
}

/// How many candidates the ladders describe, uncapped and saturating.
fn product<T>(ladders: &[(FactPath, Vec<T>)]) -> usize {
    ladders.iter().fold(1usize, |total, (_, alternatives)| {
        total.saturating_mul(alternatives.len() + 1)
    })
}

/// Every fact path the guards read, in path order.
fn read_paths(guards: &[&Predicate]) -> BTreeSet<FactPath> {
    guards
        .iter()
        .flat_map(|guard| guard.fact_paths())
        .cloned()
        .collect()
}

/// Every path a list could sit at for the guards to read into it: each proper prefix of a path
/// they read, and each collection a quantifier walks.
///
/// Over-approximate on purpose. Only a prefix that lands on a declared `List` is expanded, so a
/// prefix that names a struct costs nothing — and a list no guard reads is never expanded at all,
/// which keeps a type that refers to itself through a list as finite as it was.
fn list_reads(guards: &[&Predicate]) -> BTreeSet<FactPath> {
    let mut found = BTreeSet::new();
    for path in read_paths(guards) {
        let segments = path.segments();
        for end in 1..segments.len() {
            found.insert(FactPath::from_segments(&segments[..end]));
        }
    }
    for guard in guards {
        for over in guard.quantified_collections() {
            found.insert(over.clone());
        }
    }
    found
}

/// How many elements a list must hold for the guards' reads of it by position to land: one more
/// than the greatest ordinal read under it. `tags.0 == vip` needs one; `lines.2.quantity` three.
///
/// Keyed by every prefix followed by a numeric segment; only a prefix that is a declared `List`
/// is ever looked up.
fn positional_reads(guards: &[&Predicate]) -> BTreeMap<FactPath, usize> {
    let mut found: BTreeMap<FactPath, usize> = BTreeMap::new();
    for path in read_paths(guards) {
        let segments = path.segments();
        for end in 1..segments.len() {
            if let Ok(ordinal) = segments[end].parse::<usize>() {
                let needed = found
                    .entry(FactPath::from_segments(&segments[..end]))
                    .or_default();
                *needed = (*needed).max(ordinal.saturating_add(1));
            }
        }
    }
    found
}

/// The `Optional` members the guards test for presence, one per `defined(x)`.
///
/// `x` itself where it is an optional member, and otherwise the deepest optional member it reaches
/// through: leaving out `address` is the only way `defined(address.line2)` can be false when
/// `line2` is required.
fn omissions(guards: &[&Predicate], optionals: &BTreeSet<FactPath>) -> Vec<FactPath> {
    fn walk(predicate: &Predicate, found: &mut Vec<FactPath>) {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    walk(child, found);
                }
            }
            Predicate::Not(inner) => walk(inner, found),
            Predicate::Defined(path) => found.push(path.clone()),
            _ => {}
        }
    }
    let mut read = Vec::new();
    for guard in guards {
        walk(guard, &mut read);
    }
    let mut omitted = BTreeSet::new();
    for path in read {
        let segments = path.segments();
        if let Some(member) = (1..=segments.len())
            .rev()
            .map(|end| FactPath::from_segments(&segments[..end]))
            .find(|prefix| optionals.contains(prefix))
        {
            omitted.insert(member);
        }
    }
    omitted.into_iter().collect()
}

/// Every quantifier body under `predicate`, rebound onto the first element of what it walks.
///
/// Nested quantifiers are rebound in turn, so `exists o in orders: exists l in o.lines: …` reaches
/// `orders.0.lines.0`. A binder a nested quantifier shadows is left to that quantifier.
fn element_bodies(predicate: &Predicate, found: &mut Vec<Predicate>) {
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                element_bodies(child, found);
            }
        }
        Predicate::Not(inner) => element_bodies(inner, found),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            let body = rebind(
                &quantified.body,
                &quantified.bind,
                &quantified.over.child("0"),
            );
            element_bodies(&body, found);
            found.push(body);
        }
        _ => {}
    }
}

/// `predicate` with every read rooted at `bind` moved under `prefix`.
fn rebind(predicate: &Predicate, bind: &str, prefix: &FactPath) -> Predicate {
    let path = |path: &FactPath| {
        if path.namespace() == bind {
            let mut moved = prefix.clone();
            for segment in &path.segments()[1..] {
                moved = moved.child(segment);
            }
            moved
        } else {
            path.clone()
        }
    };
    let operand = |operand: &Operand| match operand {
        Operand::Fact(read) => Operand::Fact(path(read)),
        Operand::Literal(value) => Operand::Literal(value.clone()),
    };
    match predicate {
        Predicate::Always => Predicate::Always,
        Predicate::Never => Predicate::Never,
        Predicate::All(children) => Predicate::All(
            children
                .iter()
                .map(|child| rebind(child, bind, prefix))
                .collect(),
        ),
        Predicate::Any(children) => Predicate::Any(
            children
                .iter()
                .map(|child| rebind(child, bind, prefix))
                .collect(),
        ),
        Predicate::Not(inner) => Predicate::Not(Box::new(rebind(inner, bind, prefix))),
        Predicate::Compare { left, op, right } => Predicate::Compare {
            left: operand(left),
            op: *op,
            right: operand(right),
        },
        Predicate::Truthy(read) => Predicate::Truthy(path(read)),
        Predicate::Defined(read) => Predicate::Defined(path(read)),
        Predicate::AnyOf { path: read, values } => Predicate::AnyOf {
            path: path(read),
            values: values.clone(),
        },
        Predicate::NoneOf { path: read, values } => Predicate::NoneOf {
            path: path(read),
            values: values.clone(),
        },
        Predicate::TextMatch {
            path: read,
            op,
            value,
        } => Predicate::TextMatch {
            path: path(read),
            op: *op,
            value: value.clone(),
        },
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            let inner = ess_primitives::predicate::Quantified {
                over: path(&quantified.over),
                bind: quantified.bind.clone(),
                body: if quantified.bind == bind {
                    quantified.body.clone()
                } else {
                    rebind(&quantified.body, bind, prefix)
                },
            };
            if matches!(predicate, Predicate::Forall(_)) {
                Predicate::Forall(Box::new(inner))
            } else {
                Predicate::Exists(Box::new(inner))
            }
        }
    }
}

/// Every literal the guards compare `path` against, in the order they write them.
fn literals_at(guards: &[&Predicate], path: &FactPath) -> Vec<FactValue> {
    let mut found = Vec::new();
    for guard in guards {
        collect_literals(guard, path, &mut found);
    }
    found
}

/// Whether any guard orders `path` with `<`, `<=`, `>` or `>=`, against a literal or another fact.
fn ordered_at(guards: &[&Predicate], path: &FactPath) -> bool {
    fn walk(predicate: &Predicate, path: &FactPath) -> bool {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                children.iter().any(|child| walk(child, path))
            }
            Predicate::Not(inner) => walk(inner, path),
            Predicate::Compare { left, op, right } => {
                op.needs_ordering()
                    && [left, right]
                        .into_iter()
                        .any(|operand| matches!(operand, Operand::Fact(read) if read == path))
            }
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                walk(&quantified.body, path)
            }
            _ => false,
        }
    }
    guards.iter().any(|guard| walk(guard, path))
}

/// The walk behind [`literals_at`].
fn collect_literals(predicate: &Predicate, path: &FactPath, found: &mut Vec<FactValue>) {
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                collect_literals(child, path, found);
            }
        }
        Predicate::Not(inner) => collect_literals(inner, path, found),
        Predicate::Compare { left, op: _, right } => {
            for (operand, other) in [(left, right), (right, left)] {
                if matches!(operand, Operand::Fact(read) if read == path) {
                    if let Operand::Literal(value) = other {
                        found.push(value.clone());
                    }
                }
            }
        }
        Predicate::AnyOf { path: read, values } | Predicate::NoneOf { path: read, values } => {
            if read == path {
                found.extend(values.iter().cloned());
            }
        }
        // Walked, not skipped: a body may compare a free path against a literal beside the
        // element ones. A binder-rooted read cannot collide with `path`, which is a model path, so
        // no filtering is needed here — the equality test does it.
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            collect_literals(&quantified.body, path, found);
        }
        // `L` itself satisfies its own operator, so it is tried as any text literal is.
        Predicate::TextMatch {
            path: read, value, ..
        } => {
            if read == path {
                found.push(value.clone());
            }
        }
        Predicate::Always | Predicate::Never | Predicate::Truthy(_) | Predicate::Defined(_) => {}
    }
}

/// One string-operator literal a predicate reads at a path, and whether it is read positively —
/// under an even number of `not`/`none`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TextRead {
    op: TextOp,
    literal: String,
    positive: bool,
}

/// The string-operator literals `predicate` reads at `path`, in written order.
fn text_reads(predicate: &Predicate, path: &FactPath, positive: bool, found: &mut Vec<TextRead>) {
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                text_reads(child, path, positive, found);
            }
        }
        Predicate::Not(inner) => text_reads(inner, path, !positive, found),
        Predicate::TextMatch {
            path: read,
            op,
            value: FactValue::Text(literal),
        } if read == path => found.push(TextRead {
            op: *op,
            literal: literal.clone(),
            positive,
        }),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            text_reads(&quantified.body, path, positive, found);
        }
        _ => {}
    }
}

/// Per guard, the string-operator literals it reads at `path`: `(op, L, positive)`, with a literal
/// the same guard also negates under the same operator never taken as positive.
fn text_matches_at(guards: &[&Predicate], path: &FactPath) -> Vec<Vec<TextRead>> {
    guards
        .iter()
        .map(|guard| {
            let mut reads = Vec::new();
            text_reads(guard, path, true, &mut reads);
            reads
        })
        .filter(|reads| !reads.is_empty())
        .collect()
}

/// The three layouts of one guard's (or one newtype invariant's) positive literals around `base`.
///
/// `P` is the longest positive `starts_with` literal, `S` the longest positive `ends_with`, and `C`
/// the positive `contains` literals in written order that neither of them nor an earlier one
/// already contains; a literal the same predicate negates under the same operator is never taken. The layouts are `P + C + b + S`, `P + b + C + S` — both keep the
/// path's own text, so rule 2 still tells two fields apart — and `P + C + S` where that is not
/// empty. Composing negated literals would build the value the guard refuses.
fn compositions(reads: &[TextRead], base: &str) -> Vec<String> {
    let negated = |op: TextOp, literal: &str| {
        reads
            .iter()
            .any(|read| !read.positive && read.op == op && read.literal == literal)
    };
    let taken = |read: &&TextRead| read.positive && !negated(read.op, &read.literal);
    // The longest, not the first: a later positive prefix that extends an earlier one
    // (`starts_with: A` beside `starts_with: AB`) is met only by a value built on the longer.
    let longest = |op: TextOp| {
        reads
            .iter()
            .filter(taken)
            .filter(|read| read.op == op)
            .map(|read| read.literal.as_str())
            .fold("", |kept, literal| {
                if literal.len() > kept.len() {
                    literal
                } else {
                    kept
                }
            })
    };
    let (prefix, suffix) = (longest(TextOp::StartsWith), longest(TextOp::EndsWith));
    let mut contained: Vec<&str> = Vec::new();
    for read in reads.iter().filter(taken) {
        let covered = prefix.contains(read.literal.as_str())
            || suffix.contains(read.literal.as_str())
            || contained
                .iter()
                .any(|kept| kept.contains(read.literal.as_str()));
        if read.op == TextOp::Contains && !covered {
            contained.push(&read.literal);
        }
    }
    let contained = contained.concat();
    if prefix.is_empty() && suffix.is_empty() && contained.is_empty() {
        return Vec::new();
    }
    // Not empty here, so the third layout is always tried.
    vec![
        format!("{prefix}{contained}{base}{suffix}"),
        format!("{prefix}{base}{contained}{suffix}"),
        format!("{prefix}{contained}{suffix}"),
    ]
}

/// The most characters, or list elements, a count witness is built with.
///
/// It covers the 64 keys of beyond10x/ess#104 and a column limit of 255; a scenario carrying a
/// 1025-character text costs about a kilobyte. A guard whose boundary lies above it is refused as
/// `ESS-SYNTH-018`, with the repair, rather than searched for
/// (`docs/design/string-alphabet-and-length.md`, section 4).
pub const MAX_COUNT_WITNESS: usize = 1024;

/// [`MAX_COUNT_WITNESS`], in the width a number witness compares at exactly.
const MAX_COUNT_WITNESS_U32: u32 = 1024;

/// The count ladders, the newtype-invariant ladders, and each list's element ladder.
///
/// A `.count` compared with a number is tried at the lengths either side of it (ess#104,
/// `story:count-guards-above-one-are-synthesized`): a text resized to each, a list of that many
/// elements. Invariant ladders come after the text ladders, so a leaf a guard already varies is
/// left to the guard.
fn count_ladders(
    builder: &Builder<'_>,
    guards: &[&Predicate],
    ladders: &mut BTreeMap<FactPath, Vec<Choice>>,
) {
    let mut list_lengths: BTreeMap<FactPath, Vec<usize>> = BTreeMap::new();
    for path in read_paths(guards) {
        let Some(parent) = counted(&path) else {
            continue;
        };
        if builder.strings.contains(&parent) {
            let lengths = count_lengths(guards, &path, true);
            text_length_ladder(builder, &parent, &lengths, ladders);
        } else if builder.lists.contains(&parent) {
            list_lengths
                .entry(parent)
                .or_default()
                .extend(count_lengths(guards, &path, false));
        }
    }
    invariant_ladders(builder, ladders);
    for path in &builder.lists {
        // One element first, as unit A1 built it; length 0 is the base `[]`.
        let mut ladder = vec![Choice::Elements(1)];
        for &held in list_lengths.get(path).map_or(&[][..], Vec::as_slice) {
            let choice = Choice::Elements(held);
            if held > 1 && !ladder.contains(&choice) {
                ladder.push(choice);
            }
        }
        ladders.insert(path.clone(), ladder);
    }
}

/// The path `path` counts, when it reads `<parent>.count`.
fn counted(path: &FactPath) -> Option<FactPath> {
    let (last, parent) = path.segments().split_last()?;
    (last == "count" && !parent.is_empty()).then(|| FactPath::from_segments(parent))
}

/// The lengths a `.count` read at `path` is tried at: `⌊v⌋`, `⌊v⌋ + 1` and `⌊v⌋ − 1` for each
/// numeric literal `v` it is compared with, the number rule clipped to lengths — negatives and
/// repeats dropped, nothing above [`MAX_COUNT_WITNESS`].
///
/// A text count compared only with another fact is tried either side of [`BASE_NUMBER`], the value
/// every number witness starts at, so the other fact's own ladder decides the rest. A list keeps
/// the literal-only rule unit A1's suites were built under.
fn count_lengths(guards: &[&Predicate], path: &FactPath, against_facts: bool) -> Vec<usize> {
    let mut values: Vec<f64> = literals_at(guards, path)
        .iter()
        .filter_map(FactValue::as_number)
        .map(Number::get)
        .collect();
    if values.is_empty() && against_facts && compared_with_a_fact(guards, path) {
        values.push(BASE_NUMBER);
    }
    let mut lengths = Vec::new();
    for value in values {
        let floor = value.floor();
        for candidate in [floor, floor + 1.0, floor - 1.0] {
            if !(0.0..=f64::from(MAX_COUNT_WITNESS_U32)).contains(&candidate) {
                continue;
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let length = candidate as usize;
            if !lengths.contains(&length) {
                lengths.push(length);
            }
        }
    }
    lengths
}

/// Whether some guard compares `path` with another fact.
fn compared_with_a_fact(guards: &[&Predicate], path: &FactPath) -> bool {
    fn walk(predicate: &Predicate, path: &FactPath) -> bool {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                children.iter().any(|child| walk(child, path))
            }
            Predicate::Not(inner) => walk(inner, path),
            Predicate::Compare { left, right, .. } => matches!(
                (left, right),
                (Operand::Fact(read), Operand::Fact(_)) | (Operand::Fact(_), Operand::Fact(read))
                    if read == path
            ),
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                walk(&quantified.body, path)
            }
            _ => false,
        }
    }
    guards.iter().any(|guard| walk(guard, path))
}

/// The text at `path` resized to each of `lengths`, appended to the ladder already there.
///
/// Each resize is taken from the base and then from every alternative already on the ladder, in
/// ladder order, so a length guard and a prefix guard on one path can be met by one value.
fn text_length_ladder(
    builder: &Builder<'_>,
    path: &FactPath,
    lengths: &[usize],
    ladders: &mut BTreeMap<FactPath, Vec<Choice>>,
) {
    let Some((Leaf::Text, Node::Text(base))) = builder.leaves.get(path) else {
        return;
    };
    let plain = builder
        .plain_texts
        .get(path)
        .map_or(base.as_str(), String::as_str);
    let ladder = ladders.entry(path.clone()).or_default();
    let mut sources = vec![base.clone()];
    sources.extend(ladder.iter().filter_map(|choice| match choice {
        Choice::Value(Node::Text(text)) => Some(text.clone()),
        _ => None,
    }));
    for source in &sources {
        for &length in lengths {
            let node = Node::Text(resize(source, length, plain));
            if node != Node::Text(base.clone()) && !ladder.contains(&Choice::Value(node.clone())) {
                ladder.push(Choice::Value(node));
            }
        }
    }
    if ladder.is_empty() {
        ladders.remove(path);
    }
}

/// `text` at exactly `length` Unicode scalar values: its first `length` when it has that many,
/// otherwise its own characters cycled from its start, and `plain` cycled when it is empty.
///
/// Every character of the result is one of `text`'s, or of `plain`'s, so a text inside an alphabet
/// stays inside it and one outside it stays outside it, to be dropped by [`admitted_inputs`].
fn resize(text: &str, length: usize, plain: &str) -> String {
    let own: Vec<char> = text.chars().collect();
    let cycled: Vec<char> = if own.is_empty() {
        plain.chars().collect()
    } else {
        own
    };
    if cycled.is_empty() {
        return String::new();
    }
    cycled.iter().copied().cycle().take(length).collect()
}

/// `text` mapped into `alphabet` (rule 2 under an alphabet): a character the alphabet holds is
/// kept, and every other character `c` becomes `alphabet[c mod |alphabet|]`.
///
/// Keeps the path's own characters wherever the alphabet allows, so an alphabet that admits the
/// path text leaves the witness unchanged.
fn into_alphabet(text: &str, alphabet: &str) -> String {
    let characters: Vec<char> = alphabet.chars().collect();
    if characters.is_empty() {
        return text.to_owned();
    }
    text.chars()
        .map(|character| {
            if characters.contains(&character) {
                character
            } else {
                characters[u32::from(character) as usize % characters.len()]
            }
        })
        .collect()
}

/// The effective alphabet of the newtype chain `type_ref` is declared through, if any layer
/// declares one — `ess-domain`'s intersection, so the witness and validation agree.
fn chain_alphabet(ir: &EssIr, type_ref: &ResolvedTypeRef) -> Option<String> {
    let mut alphabets = Vec::new();
    let mut current = type_ref;
    for _ in 0..=MAX_TYPE_DEPTH {
        match current {
            ResolvedTypeRef::Optional { of } => current = of,
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, alphabet, .. } => {
                    if let Some(alphabet) = alphabet {
                        alphabets.push(alphabet.as_str());
                    }
                    current = of;
                }
                _ => break,
            },
            _ => break,
        }
    }
    ess_domain::types::effective_alphabet(alphabets)
}

/// Ladders for the leaves no guard reads whose own type refuses their base witness.
///
/// Without a value the type admits, no candidate survives [`admitted_inputs`] and no branch has a
/// witness. The values tried are the type's own invariant literals, laid out as a guard's are.
fn invariant_ladders(builder: &Builder<'_>, ladders: &mut BTreeMap<FactPath, Vec<Choice>>) {
    let value = FactPath::new("value").expect("the newtype pseudo-field is a fact path");
    for (path, invariants) in &builder.invariants {
        let Some((leaf, at_base)) = builder.leaves.get(path) else {
            continue;
        };
        if ladders.contains_key(path)
            || admits_base(invariants, builder.alphabets.get(path), at_base)
        {
            continue;
        }
        let declared: Vec<&Predicate> = invariants.iter().collect();
        let mut alternatives = alternatives(
            leaf,
            at_base,
            &literals_at(&declared, &value),
            ordered_at(&declared, &value),
        );
        if let (Leaf::Text, Node::Text(base)) = (leaf, at_base) {
            for invariant in invariants {
                let mut reads = Vec::new();
                text_reads(invariant, &value, true, &mut reads);
                for text in compositions(&reads, base) {
                    let node = Node::Text(text);
                    if &node != at_base && !alternatives.contains(&node) {
                        alternatives.push(node);
                    }
                }
            }
        }
        if !alternatives.is_empty() {
            ladders.insert(
                path.clone(),
                alternatives.into_iter().map(Choice::Value).collect(),
            );
        }
        // The same length rule as a guard's, for an invariant over `value.count`.
        if builder.strings.contains(path) {
            let lengths = count_lengths(&declared, &value.child("count"), true);
            text_length_ladder(builder, path, &lengths, ladders);
        }
    }
}

/// Whether every newtype invariant recorded at a leaf, and its alphabet, hold for its base
/// witness, read as `value`.
///
/// Only `True` admits: an invariant the base leaves `Unknown` or refutes is one `admitted_inputs`
/// would refuse the whole input for. A value that is no scalar is left to that check.
fn admits_base(invariants: &[Predicate], alphabet: Option<&String>, base: &Node) -> bool {
    if let (Some(alphabet), Node::Text(text)) = (alphabet, base) {
        if !text.chars().all(|character| alphabet.contains(character)) {
            return false;
        }
    }
    let fact = match base {
        Node::Text(text) => FactValue::Text(text.clone()),
        Node::Number(number) => FactValue::Number(*number),
        Node::Bool(value) => FactValue::Bool(*value),
        _ => return true,
    };
    let mut facts = ess_primitives::facts::FactStore::new();
    facts.set(
        FactPath::new("value").expect("the newtype pseudo-field is a fact path"),
        fact,
    );
    invariants
        .iter()
        .all(|invariant| invariant.evaluate(&facts).is_satisfied())
}

/// `L′`: `L` with one character replaced by `x` (by `y` where it is `x`) — the last for
/// `starts_with`, the first for `ends_with`, the middle one (index ⌊n/2⌋) for `contains`.
///
/// No longer than `L` in bytes and different from it, so `L` is not a prefix, suffix or substring
/// of it: it refutes its own operator. The position keeps the rest of `L`, so a newtype invariant
/// written on the same affix — `starts_with: "+"` under `starts_with: "+44"` — still holds for it.
fn refuting(op: TextOp, literal: &str) -> Option<String> {
    let mut characters: Vec<char> = literal.chars().collect();
    let last = characters.len().checked_sub(1)?;
    let index = match op {
        TextOp::StartsWith => last,
        TextOp::EndsWith => 0,
        TextOp::Contains => characters.len() / 2,
    };
    characters[index] = if characters[index] == 'x' { 'y' } else { 'x' };
    Some(characters.into_iter().collect())
}

/// The text candidates rule 3 adds for string operators at `path`, after the literals: each
/// guard's compositions, then each newtype invariant's, then each literal's `L′`.
fn text_alternatives(
    guards: &[&Predicate],
    invariants: &[Predicate],
    path: &FactPath,
    base: &str,
) -> Vec<String> {
    let per_guard = text_matches_at(guards, path);
    let mut found = Vec::new();
    for reads in &per_guard {
        found.extend(compositions(reads, base));
    }
    let value = FactPath::new("value").expect("the newtype pseudo-field is a fact path");
    for invariant in invariants {
        let mut reads = Vec::new();
        text_reads(invariant, &value, true, &mut reads);
        found.extend(compositions(&reads, base));
    }
    let mut seen: Vec<(TextOp, &str)> = Vec::new();
    for read in per_guard.iter().flatten() {
        if !seen.contains(&(read.op, read.literal.as_str())) {
            seen.push((read.op, &read.literal));
            found.extend(refuting(read.op, &read.literal));
        }
    }
    found
}

/// The values one leaf is tried at, beside the base value it already carries, in the order they are
/// tried.
///
/// Derived from what the guard writes, never from a range this module imagines. The one addition is
/// `0` and `-1` for a number: a guard that compares two facts writes no literal at all, and those
/// two are the values that decide sign and truthiness — which is what `> 0`, `>= 0` and a bare
/// truthiness test are made of.
///
/// `base` is what this leaf already holds at its [`Distinction`], and it is excluded rather than
/// assumed to be the plain one: the first candidate is the base witness, so offering the same value
/// again spends a try on a decision already taken.
fn alternatives(leaf: &Leaf, base: &Node, literals: &[FactValue], ordered: bool) -> Vec<Node> {
    let mut values = Vec::new();
    let mut push = |value: Node| {
        if &value != base && !values.contains(&value) {
            values.push(value);
        }
    };
    match leaf {
        Leaf::Number { integral } => {
            let mut numbers: Vec<f64> = Vec::new();
            for literal in literals {
                if let Some(number) = literal.as_number() {
                    numbers.extend([number.get(), number.get() + 1.0, number.get() - 1.0]);
                }
            }
            numbers.extend([0.0, -1.0]);
            for value in numbers {
                let Ok(candidate) = Number::new(value) else {
                    continue;
                };
                if !*integral || candidate.is_integral() {
                    push(Node::Number(candidate));
                }
            }
        }
        Leaf::Bool => push(Node::Bool(!matches!(base, Node::Bool(true)))),
        Leaf::Text => {
            let texts: Vec<&str> = literals.iter().filter_map(FactValue::as_text).collect();
            for text in &texts {
                push(Node::Text((*text).to_owned()));
            }
            if ordered {
                // Text orders by its bytes (ess#94), so a literal decides at a boundary like a
                // number does: the literal itself, the literal with one byte appended — the
                // nearest text above it that is still recognisably the guard's own — and the empty
                // text, which is below every other.
                for text in &texts {
                    push(Node::Text(format!("{text}{TEXT_STEP}")));
                }
                if !texts.is_empty() {
                    push(Node::Text(String::new()));
                }
            }
        }
        Leaf::Timestamp => {
            let texts: Vec<&str> = literals.iter().filter_map(FactValue::as_text).collect();
            for text in &texts {
                push(Node::Text((*text).to_owned()));
            }
            if ordered {
                // An ordering decides at a boundary, so each instant the guard writes is tried a
                // second either side; two facts ordered against each other write none, and a day
                // either side of the base is what moves one past the other.
                let steps = texts.iter().map(|text| (*text, 1)).chain(match base {
                    Node::Text(text) => Some((text.as_str(), SECONDS_PER_DAY)),
                    _ => None,
                });
                for (text, seconds) in steps {
                    let Some(instant) = Rfc3339Instant::parse_rfc3339(text) else {
                        continue;
                    };
                    for step in [seconds, -seconds] {
                        if let Some(moved) = instant.plus_seconds(step) {
                            push(Node::Text(moved.to_rfc3339()));
                        }
                    }
                }
            }
        }
        Leaf::Enum { variants } => {
            for variant in variants {
                push(Node::Text(variant.clone()));
            }
        }
    }
    values
}

/// The most elements a base witness holds for a list read by position. A guard reading
/// `tags.100000` is not met by building a hundred thousand elements; it is refused as undecidable.
const MAX_POSITIONAL_ELEMENTS: usize = MAX_CANDIDATES;

/// What an ordered text literal is extended by to make a text above it.
const TEXT_STEP: char = 'a';

/// The number every numeric witness starts at.
const BASE_NUMBER: f64 = 1.0;

/// The boolean every boolean witness starts at.
const BASE_BOOL: bool = true;

/// The day the first `Timestamp` witness lands on. A constant, so nothing here reads a clock.
///
/// A further instance takes the next day of the same month, which is why the month is written here
/// and the day is not.
const BASE_TIMESTAMP_MONTH: &str = "2020-01";

/// How many instances a `Timestamp` witness can be told apart by, before the days repeat.
///
/// Twenty-eight, because every month has a twenty-eighth and no month has a thirty-second: a witness
/// that is not a date is a witness a target refuses for a reason that has nothing to do with what
/// the scenario tests.
const DISTINGUISHABLE_DAYS: usize = 28;

/// One day, the step a `Timestamp` witness moves by when two facts are ordered against each other.
const SECONDS_PER_DAY: i64 = 86_400;

/// The number of seconds the first `Duration` witness carries.
const BASE_DURATION_SECONDS: usize = 1;

/// The byte the first `Bytes` witness carries, encoded base64 as `AA==`.
const BASE_BYTE: u8 = 0;

/// Builds one input from the declared types, recording where a candidate could vary it.
struct Builder<'ir> {
    ir: &'ir EssIr,
    /// Which instance the input is for.
    distinction: Distinction,
    /// Every path at which a declared list is built with its element as well, so the element's
    /// leaves are recorded and a [`Choice::Elements`] has something to put there.
    ///
    /// Only the lists a guard reads. Expanding every list would walk into every element type, and
    /// a type that refers to itself through a list — finite today because its list is empty —
    /// would stop being witnessable.
    expand: BTreeSet<FactPath>,
    /// The lists a guard reads by position, with how many elements the base must hold for every
    /// such read to land (ess#94). The base carries them, rather than a candidate: a read that
    /// misses is `Unknown`, and an `Unknown` ends the search before a candidate is reached.
    positional: BTreeMap<FactPath, usize>,
    /// Every scalar the input holds, by the fact path that reads it, with the base value it took.
    ///
    /// The value is kept beside the leaf because the ladder is built relative to it, and at a
    /// further [`Distinction`] the base is not the one this module's constants name.
    leaves: BTreeMap<FactPath, (Leaf, Node)>,
    /// Every expanded path that holds a declared list.
    lists: BTreeSet<FactPath>,
    /// Every path that holds an `Optional` member of an input or of a struct, which a
    /// [`Choice::Omit`] can leave out.
    optionals: BTreeSet<FactPath>,
    /// The invariants of every newtype a recorded leaf is declared through, at any depth, by the
    /// leaf's fact path and read against `value`: what the invariant-composed text candidates are
    /// built from, so a refuting candidate survives [`admitted_inputs`].
    invariants: BTreeMap<FactPath, Vec<Predicate>>,
    /// The effective alphabet of every text a declared newtype chain constrains, by path: the
    /// outermost alphabet's characters every inner one also holds. Recorded inside a union as
    /// well, because a value there must be legal even where no guard can vary it.
    alphabets: BTreeMap<FactPath, String>,
    /// Every recorded leaf declared `String`, the one text that has a length. [`Leaf::Text`] also
    /// covers `Uuid`, `Duration` and `Bytes`.
    strings: BTreeSet<FactPath>,
    /// Each `String` leaf's base from rules 2 and 3 — its own path, mapped into its alphabet —
    /// which is never empty and is what a resize of an empty text cycles.
    plain_texts: BTreeMap<FactPath, String>,
    /// The authored `example:` of each command input, used as that input's base at
    /// [`Distinction::PLAIN`] only.
    examples: BTreeMap<FactPath, Node>,
}

impl<'ir> Builder<'ir> {
    fn new(
        ir: &'ir EssIr,
        distinction: Distinction,
        expand: BTreeSet<FactPath>,
        positional: BTreeMap<FactPath, usize>,
    ) -> Self {
        Self {
            ir,
            distinction,
            expand,
            positional,
            leaves: BTreeMap::new(),
            lists: BTreeSet::new(),
            optionals: BTreeSet::new(),
            invariants: BTreeMap::new(),
            alphabets: BTreeMap::new(),
            strings: BTreeSet::new(),
            plain_texts: BTreeMap::new(),
            examples: BTreeMap::new(),
        }
    }

    /// One whole command input, with `overrides` applied at the paths they name.
    fn input(
        &mut self,
        command: &ResolvedCommand,
        overrides: &BTreeMap<FactPath, Choice>,
    ) -> Result<BTreeMap<String, Node>, WitnessGap> {
        // Examples are the base at the plain instance only: an example is one value, and rule 5
        // needs instances that differ.
        if self.distinction == Distinction::PLAIN {
            self.examples = command
                .examples
                .iter()
                .filter_map(|(name, example)| {
                    FactPath::new(name).ok().map(|path| (path, example.clone()))
                })
                .collect();
        }
        let mut input = BTreeMap::new();
        for field in &command.input {
            let path = FactPath::new(&field.name).map_err(|_| WitnessGap {
                path: field.name.clone(),
                type_ref: field.type_ref.to_string(),
                reason: "is named in a way no fact path can spell, so no guard could read it",
            })?;
            if let Some(value) = self.member(&field.type_ref, &path, overrides, 0, true)? {
                input.insert(field.name.clone(), value);
            }
        }
        Ok(input)
    }

    /// One member of an input or a struct: its value, or `None` where a candidate leaves an
    /// `Optional` member out.
    fn member(
        &mut self,
        type_ref: &ResolvedTypeRef,
        path: &FactPath,
        overrides: &BTreeMap<FactPath, Choice>,
        depth: usize,
        record: bool,
    ) -> Result<Option<Node>, WitnessGap> {
        if type_ref.is_optional() {
            if record {
                self.optionals.insert(path.clone());
            }
            if overrides.get(path) == Some(&Choice::Omit) {
                return Ok(None);
            }
        }
        self.value(type_ref, path, overrides, depth, record)
            .map(Some)
    }

    /// The base of one primitive leaf, recorded: its path mapped into its alphabet (rule 2), or
    /// the input's example at the plain instance.
    fn primitive(&mut self, name: Primitive, path: &FactPath, record: bool) -> Node {
        let mut base = primitive_value(name, path, self.distinction);
        if let (Node::Text(text), Some(alphabet)) = (&base, self.alphabets.get(path)) {
            base = Node::Text(into_alphabet(text, alphabet));
        }
        if name == Primitive::String {
            if let Node::Text(plain) = &base {
                self.plain_texts.insert(path.clone(), plain.clone());
            }
            if record {
                self.strings.insert(path.clone());
            }
        }
        if let Some(example) = self.examples.get(path) {
            base = example.clone();
        }
        if record {
            self.leaves
                .insert(path.clone(), (Leaf::of_primitive(name), base.clone()));
        }
        base
    }

    /// One value of `List<of>` at `path`: `[]`, unless a guard reads into it (rule 3).
    fn list(
        &mut self,
        of: &ResolvedTypeRef,
        path: &FactPath,
        overrides: &BTreeMap<FactPath, Choice>,
        depth: usize,
        record: bool,
    ) -> Result<Node, WitnessGap> {
        if !record || !self.expand.contains(path) {
            return Ok(Node::Seq(Vec::new()));
        }
        if let Some(&held) = self
            .positional
            .get(path)
            .filter(|&&held| held <= MAX_POSITIONAL_ELEMENTS)
        {
            let mut elements = Vec::with_capacity(held);
            for ordinal in 0..held {
                let at = path.child(&ordinal.to_string());
                elements.push(self.value(of, &at, overrides, depth + 1, record)?);
            }
            return Ok(Node::Seq(elements));
        }
        self.lists.insert(path.clone());
        // Built even where the base keeps the list empty, so its leaves are recorded
        // before any ladder is drawn.
        let element = self.value(of, &path.child("0"), overrides, depth + 1, record)?;
        Ok(match overrides.get(path) {
            Some(Choice::Elements(held)) => Node::Seq(vec![element; *held]),
            _ => Node::Seq(Vec::new()),
        })
    }

    /// One value of `type_ref`, at `path`.
    ///
    /// `record` is false inside a union, where no fact path reaches — so nothing there is offered
    /// to a candidate ladder that could not change a decision anyway.
    fn value(
        &mut self,
        type_ref: &ResolvedTypeRef,
        path: &FactPath,
        overrides: &BTreeMap<FactPath, Choice>,
        depth: usize,
        record: bool,
    ) -> Result<Node, WitnessGap> {
        if depth > MAX_TYPE_DEPTH {
            return Err(WitnessGap {
                path: path.to_string(),
                type_ref: type_ref.to_string(),
                reason: "refers to itself, so it has no finite value to send",
            });
        }
        let chosen = |base: Node| match overrides.get(path) {
            Some(Choice::Value(value)) => value.clone(),
            _ => base,
        };
        match type_ref {
            // Transparent, exactly as the flattener reads them: neither an optional nor a newtype
            // has a segment of its own, so the path does not grow. The base fills an optional;
            // leaving one out is a candidate's choice, made by `member`.
            ResolvedTypeRef::Optional { of } => self.value(of, path, overrides, depth + 1, record),
            ResolvedTypeRef::List { of } => self.list(of, path, overrides, depth, record),
            ResolvedTypeRef::Map { .. } => Ok(Node::Map(BTreeMap::new())),
            ResolvedTypeRef::Primitive { name } => {
                if *name == Primitive::Binary64 {
                    return Err(WitnessGap { path: path.to_string(), type_ref: name.to_string(), reason: "requires a finite Binary64 conformance codec that this suite format does not admit" });
                }
                Ok(chosen(self.primitive(*name, path, record)))
            }
            ResolvedTypeRef::Declared { name } => {
                // Read through the IR reference rather than through `self`, so what comes back
                // lives as long as the IR and the recursive calls below can still borrow `self`.
                let ir = self.ir;
                match &ir.named_type(name).body {
                    ResolvedBody::Newtype { of, invariants, .. } => {
                        // The outermost newtype of a chain sees every layer below it, so its
                        // answer is the effective one and an inner layer never replaces it.
                        if !self.alphabets.contains_key(path) {
                            if let Some(alphabet) = chain_alphabet(ir, type_ref) {
                                self.alphabets.insert(path.clone(), alphabet);
                            }
                        }
                        if record {
                            let recorded = self.invariants.entry(path.clone()).or_default();
                            for invariant in invariants {
                                if !recorded.contains(&invariant.predicate) {
                                    recorded.push(invariant.predicate.clone());
                                }
                            }
                        }
                        self.value(of, path, overrides, depth + 1, record)
                    }
                    ResolvedBody::Enum { variants } => {
                        // The variants cycle, so a closed set of two names distinguishes two
                        // instances and no more — which is the type's answer, not a shortfall here.
                        let variant = if variants.is_empty() {
                            String::new()
                        } else {
                            variants[self.distinction.get() % variants.len()]
                                .name()
                                .to_owned()
                        };
                        let base = self
                            .examples
                            .get(path)
                            .cloned()
                            .unwrap_or(Node::Text(variant));
                        if record {
                            self.leaves.insert(
                                path.clone(),
                                (
                                    Leaf::Enum {
                                        variants: variants
                                            .iter()
                                            .map(|variant| variant.name().to_owned())
                                            .collect(),
                                    },
                                    base.clone(),
                                ),
                            );
                        }
                        Ok(chosen(base))
                    }
                    ResolvedBody::Union { tag, variants } => {
                        let Some((label, variant)) = variants.iter().next() else {
                            return Ok(Node::Map(BTreeMap::new()));
                        };
                        let inner = self.value(variant, path, overrides, depth + 1, false)?;
                        Ok(Node::Map(BTreeMap::from([
                            (tag.clone(), Node::Text(label.clone())),
                            (UNION_VALUE.to_owned(), inner),
                        ])))
                    }
                    ResolvedBody::Struct { fields, .. } => {
                        let mut value = BTreeMap::new();
                        for field in fields {
                            let child = path.child(&field.name);
                            if let Some(inner) =
                                self.member(&field.type_ref, &child, overrides, depth + 1, record)?
                            {
                                value.insert(field.name.clone(), inner);
                            }
                        }
                        Ok(Node::Map(value))
                    }
                }
            }
        }
    }
}

impl Leaf {
    /// What a candidate may vary a primitive to.
    fn of_primitive(primitive: Primitive) -> Self {
        match primitive {
            Primitive::Binary64 => {
                unreachable!("Binary64 witnesses are refused before construction")
            }
            Primitive::Boolean => Self::Bool,
            Primitive::Integer => Self::Number { integral: true },
            Primitive::Decimal => Self::Number { integral: false },
            Primitive::Timestamp => Self::Timestamp,
            Primitive::String | Primitive::Duration | Primitive::Uuid | Primitive::Bytes => {
                Self::Text
            }
        }
    }
}

/// The base value of one primitive, at one path, for one instance.
///
/// At [`Distinction::PLAIN`] every value is the constant or the path this module documents. Further
/// instances move each value inside its own declared type — a whole day for a `Timestamp`, a whole
/// second for a `Duration`, a suffix for a `String` — so a target that parses the plain witness
/// parses this one too. A `Boolean` is the exception the type imposes rather than a choice here: it
/// has two values, and beyond the second instance it repeats.
fn primitive_value(primitive: Primitive, path: &FactPath, distinction: Distinction) -> Node {
    let nth = distinction.get();
    // The same ordinal, in the width the two numeric types can carry exactly. A scenario arranges
    // one further instance per row a declared order needs, so `nth` is a small count and the
    // saturation below is unreachable; it is written rather than cast because a witness that
    // silently rounded would be two instances nothing could tell apart.
    let ordinal = u32::try_from(nth).unwrap_or(u32::MAX);
    match primitive {
        Primitive::Binary64 => unreachable!("Binary64 witnesses are refused before construction"),
        Primitive::Boolean => Node::Bool(BASE_BOOL != (nth % 2 == 1)),
        Primitive::Integer | Primitive::Decimal => {
            Node::Number(number(BASE_NUMBER + f64::from(ordinal)))
        }
        // A whole day apart, and inside one month, so every instance carries a date that exists.
        Primitive::Timestamp => Node::Text(format!(
            "{BASE_TIMESTAMP_MONTH}-{:02}T00:00:00Z",
            1 + nth % DISTINGUISHABLE_DAYS
        )),
        Primitive::Duration => Node::Text(format!("PT{}S", BASE_DURATION_SECONDS + nth)),
        Primitive::Bytes => Node::Text(base64_byte(
            BASE_BYTE.wrapping_add(ordinal.to_le_bytes()[0]),
        )),
        Primitive::Uuid => Node::Text(uuid(path, distinction)),
        // The path itself, so two fields of one type never carry one value — see rule 2. The suffix
        // is what keeps two *instances* apart, and it is a suffix rather than a prefix so the field
        // a value came from is still the first thing a reader of a failing diagnostic sees.
        Primitive::String if nth == 0 => Node::Text(path.to_string()),
        Primitive::String => Node::Text(format!("{path}-{nth}")),
    }
}

/// One byte, base64. [`BASE_BYTE`] encodes as `AA==`.
fn base64_byte(value: u8) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let high = char::from(ALPHABET[usize::from(value >> 2)]);
    let low = char::from(ALPHABET[usize::from((value & 0b11) << 4)]);
    format!("{high}{low}==")
}

/// A number that is known to be finite.
fn number(value: f64) -> Number {
    Number::new(value).unwrap_or_else(|error| panic!("a witness is a finite number: {error}"))
}

/// A syntactically well-formed UUID, derived from the path it sits at and the instance it is for.
///
/// Derived rather than fixed so two `Uuid` fields of one input differ, and derived by hashing rather
/// than counting so inserting a field renumbers nothing. Version 4 and the standard variant, because
/// a target that parses it should not have to accept a shape no generator would produce.
///
/// It identifies nothing, and synthesis never asks it to: an outcome that needs an instance that
/// already exists is refused before a witness is built.
fn uuid(path: &FactPath, distinction: Distinction) -> String {
    let seed = match distinction.get() {
        0 => path.to_string(),
        nth => format!("{path}#{nth}"),
    };
    let digest = fnv1a(seed.as_bytes()) & 0xffff_ffff_ffff;
    format!("00000000-0000-4000-8000-{digest:012x}")
}

/// The [`uuid`] builder seeded with a text of the caller's choosing rather than a fact path.
///
/// An aggregate view's scoped group values are `uuid("<view>/<group>")` for a `Uuid` key
/// (`docs/design/aggregate-views.md`, "Scoping"), so the digest is the same 48-bit FNV-1a spread.
pub(crate) fn uuid_of(seed: &str) -> String {
    let digest = fnv1a(seed.as_bytes()) & 0xffff_ffff_ffff;
    format!("00000000-0000-4000-8000-{digest:012x}")
}

/// FNV-1a, 64-bit.
///
/// Written out rather than taken as a dependency: `ess-gen` has `sha2` for a provenance digest that
/// is published and compared, and this is neither — it is a way to spread twelve hexadecimal digits
/// over the fields of one command, deterministically and with no clock. Twelve lines against a
/// crate in every downstream lockfile is the trade this repository already records making.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    use ess_primitives::facts::FactValue;

    fn path(value: &str) -> FactPath {
        FactPath::new(value).expect("a valid fact path")
    }

    #[test]
    fn a_text_witness_is_its_own_path_so_two_fields_of_one_type_never_agree() {
        // The property `examples/oracle-fixture/` exists for: `contact` and `alternate_contact` are
        // both `oracle.order.Email`, so a binding that maps the wrong one is only detectable if the
        // two carry different values.
        let contact = primitive_value(Primitive::String, &path("contact"), Distinction::PLAIN);
        let alternate = primitive_value(
            Primitive::String,
            &path("alternate_contact"),
            Distinction::PLAIN,
        );

        assert_eq!(contact, Node::Text("contact".to_owned()));
        assert_ne!(
            contact, alternate,
            "a constant witness would make a swapped mapping invisible"
        );
    }

    #[test]
    fn two_uuid_witnesses_differ_and_neither_moves_when_a_third_field_appears() {
        let first = primitive_value(Primitive::Uuid, &path("invoice_id"), Distinction::PLAIN);
        let second = primitive_value(Primitive::Uuid, &path("order_id"), Distinction::PLAIN);

        assert_ne!(first, second, "two ids of one input must not collide");
        assert_eq!(
            first,
            primitive_value(Primitive::Uuid, &path("invoice_id"), Distinction::PLAIN),
            "a witness derived from a counter would move when a field is inserted before it"
        );
        let Node::Text(rendered) = first else {
            panic!("a uuid witness is text")
        };
        assert_eq!(rendered.len(), 36, "{rendered} is not uuid-shaped");
        assert_eq!(
            rendered.chars().nth(14),
            Some('4'),
            "the version nibble is 4: {rendered}"
        );
    }

    #[test]
    fn the_alternatives_for_a_number_are_the_guards_own_literals_either_side() {
        let alternatives = alternatives(
            &Leaf::Number { integral: false },
            &Node::Number(number(BASE_NUMBER)),
            &[FactValue::number(0.0).expect("finite")],
            false,
        );

        assert_eq!(
            alternatives,
            vec![Node::Number(number(0.0)), Node::Number(number(-1.0))],
            "the literal, then either side of it, with `0 + 1` dropped as the base value"
        );
        assert!(
            !alternatives.contains(&Node::Number(number(BASE_NUMBER))),
            "the base value is the first candidate, so repeating it wastes a try"
        );
    }

    #[test]
    fn an_integer_leaf_is_never_offered_a_fractional_candidate() {
        // The flattener refuses `1.5` for an `Integer` rather than rounding it, so a candidate that
        // carried one would be rejected as misshapen before any guard was decided.
        let literal = FactValue::number(0.5).expect("finite");
        let literals = std::slice::from_ref(&literal);
        let base = Node::Number(number(BASE_NUMBER));
        let integral = alternatives(&Leaf::Number { integral: true }, &base, literals, false);
        let decimal = alternatives(&Leaf::Number { integral: false }, &base, literals, false);

        assert_eq!(
            integral,
            vec![Node::Number(number(0.0)), Node::Number(number(-1.0))],
            "only the two whole numbers survive"
        );
        assert!(
            decimal.contains(&Node::Number(number(0.5))),
            "and the fixture's literal is a candidate where the type allows it: {decimal:?}"
        );
    }

    #[test]
    fn an_ordered_timestamp_is_tried_either_side_of_each_instant() {
        let base = Node::Text("2020-01-01T00:00:00Z".to_owned());
        let literal = FactValue::text("2020-06-01T12:00:00+01:00");
        let text = |value: &str| Node::Text(value.to_owned());
        assert_eq!(
            alternatives(
                &Leaf::Timestamp,
                &base,
                std::slice::from_ref(&literal),
                true
            ),
            vec![
                text("2020-06-01T12:00:00+01:00"),
                text("2020-06-01T11:00:01Z"),
                text("2020-06-01T10:59:59Z"),
                text("2020-01-02T00:00:00Z"),
                text("2019-12-31T00:00:00Z"),
            ]
        );
        assert_eq!(
            alternatives(
                &Leaf::Timestamp,
                &base,
                std::slice::from_ref(&literal),
                false
            ),
            vec![text("2020-06-01T12:00:00+01:00")],
            "a Timestamp only compared for equality keeps the literal-only ladder"
        );
    }

    #[test]
    fn an_enum_offers_every_variant_it_declares_and_the_first_one_only_once() {
        let alternatives = alternatives(
            &Leaf::Enum {
                variants: vec!["Email".to_owned(), "Post".to_owned(), "Portal".to_owned()],
            },
            &Node::Text("Email".to_owned()),
            &[],
            false,
        );

        assert_eq!(
            alternatives,
            vec![
                Node::Text("Post".to_owned()),
                Node::Text("Portal".to_owned())
            ],
            "`Email` is the base value, so the ladder holds the other two"
        );
    }

    #[test]
    fn the_count_cap_is_one_number_in_both_widths() {
        assert_eq!(
            MAX_COUNT_WITNESS,
            usize::try_from(MAX_COUNT_WITNESS_U32).expect("fits")
        );
    }

    #[test]
    fn the_candidate_count_is_bounded_however_many_fields_a_guard_reads() {
        let ladder = (
            path("a"),
            vec![Node::Bool(false), Node::Bool(true), Node::Null],
        );
        let many: Vec<(FactPath, Vec<Node>)> = (0..40).map(|_| ladder.clone()).collect();

        assert_eq!(
            combinations(&many),
            MAX_CANDIDATES,
            "4^40 must saturate rather than overflow or be enumerated"
        );
        assert_eq!(
            combinations::<Node>(&[]),
            1,
            "one base witness and nothing else"
        );
    }
}
