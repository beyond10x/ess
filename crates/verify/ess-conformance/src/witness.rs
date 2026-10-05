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
//! # The strategy, in six rules
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
//!    `alphabet[c mod |alphabet|]`, so the witness is a value the type admits. **Under a declared
//!    `prefix:`** (ess/15) the text is the prefix followed by the path, `/channel` for `channel`
//!    under `/`, before any alphabet maps it; a count resize keeps the prefix in front, and a
//!    candidate that does not start with it is refused with every other value the type refuses.
//!    **An input's
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
//! 6. **The base satisfies the invariants over the input** (beyond10x/ess#234): those of every
//!    struct the input holds, read under the path the struct is built at, and those of every
//!    entity a branch copies the input into, read through the copy (`sets: {fingerprint:
//!    input.fingerprint}` beside `fingerprint.version == "v1"` reads the input's
//!    `fingerprint.version`). Where rule 2's base refuses one, the leaves it reads are tried at its
//!    literals and, for a comparison of two leaves, at each other's base, and the first value the
//!    invariants admit is the base every candidate starts from. A base that already satisfies them
//!    moves nowhere. An equality a guard writes between two leaves (`owner == ticket.owner`) tries
//!    each side at the other's base as well. Every candidate is then held to the entity
//!    invariants of the branches it can reach: one that breaks one is solved again with the leaves
//!    it moved kept where they are, or dropped, and never sent.
//!
//! # What has no witness
//!
//! A type that refers to itself, and a field whose name cannot be spelled as a fact path. Both are
//! [`WitnessGap`], and both are reported rather than worked around. Everything else in the model has
//! a value: a list is `[]` (one element where a guard reads into it, rule 3, or where a branch copies
//! it whole into a payload or a stored field, ess#196), a union is its first
//! variant in the encoding `ess-gen` publishes, and an enum is its first declared variant.
//!
//! **A map holds one entry** (beyond10x/ess#196), so a target that drops or empties a copied map
//! fails its assertion. The key is the key primitive's own witness at the map's path — `"tags"`,
//! `1`, `true` — in the spelling a setup key is read by, and it moves with the instance as every
//! leaf does (rule 5). The value is built at `<map>.0`, as a list's element is, and recorded there.
//! A quantifier over a map binds its values, and the flattener publishes them at their ordinals in
//! key order (beyond10x/ess#240), so a quantifier's body is rebound onto `<map>.0` as rule 3 rebinds
//! a list's, and the value is tried at the guard's own literal. A guard over `<map>.count`, which the flattener publishes, is tried at the empty map and at the
//! lengths rule 3 names, each further entry a copy of the first under the next instance's key. A map
//! whose key is a `Decimal`, which has no setup spelling, or whose value has no finite witness, is
//! `{}`. A value that reaches a type already being built — a type that refers to itself through a
//! map or a copied list — is unfolded once, and every map and copied list inside that one entry is
//! empty.
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
use ess_primitives::predicate::{CompareOp, DistinctKeyKind, Operand, Predicate, TextOp};
use ess_primitives::time::{CurrentTime, Rfc3339Instant};

use crate::decision::Decision;

/// How many candidate inputs one outcome is tried against before synthesis refuses.
///
/// A bound rather than a budget to spend: §11 asks for a constrained strategy, and the honest
/// failure of one is "the values I know how to try did not satisfy this guard", reported with the
/// number. A larger number would turn a specification that needs a solver into a slower build that
/// still cannot say so.
///
/// It bounds the walk over every ladder. Where the walk is cut short, at most this many more
/// candidates follow it, solved for the guards rather than walked (`Directed`).
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

    /// The witness for an identity no scenario creates: the unknown-instance scenario's
    /// (`docs/design/typed-literals-and-unknown-instances.md`, section 2).
    ///
    /// Far past every further instance an arrangement numbers — those count from one and stop at
    /// [`MAX_CANDIDATES`] — so where a type has that many values, no arrangement reaches it.
    pub const UNKNOWN: Self = Self(1 << 20);

    /// How far from the plain witness this one is.
    pub const fn get(self) -> usize {
        self.0
    }
}

/// The wire key a union variant's value is carried under beside the tag `tag`.
///
/// `{"kind": "person", "value": …}`, or `content` where the tag is itself `value` — the encoding
/// `ess-gen` publishes in `generated/schema/types/billing.invoice.Payee.schema.json`, read from
/// there rather than decided again here, because a witness in a second encoding is a witness no
/// target can accept.
fn union_content_key(tag: &str) -> &'static str {
    ess_gen::schema::union_content_key(tag)
}

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
    /// Any JSON value (ess/15). No guard reads one, so it has no alternatives.
    Json,
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
    search(ir, command, guards, distinction, false).map(|(inputs, _)| inputs)
}

/// [`candidates`] with each `Decimal` leaf also tried between every two adjacent literals the
/// guards compare it with, or `None` where no such leaf has two distinct literals to go between.
///
/// Rule 3 tries each literal and one either side, so an interval between two literals less than two
/// apart holds no value it tries: `amount > 11` and `amount < 12` over a `Decimal` are satisfied
/// together by nothing on that ladder (beyond10x/ess#217). The midpoint of each such pair, at the
/// exact decimal precision the value is written in, is a value inside every interval the literals
/// bound, so over guards comparing one `Decimal` leaf with literals the search is exhaustive. It is
/// a second search a caller runs only where the first found nothing, so a witness the ladder finds
/// is the one it always was.
///
/// # Errors
///
/// [`WitnessGap`] when some field of the input has no safe value at all.
pub fn candidates_between(
    ir: &EssIr,
    command: &ResolvedCommand,
    guards: &[&Predicate],
    distinction: Distinction,
) -> Result<Option<Vec<BTreeMap<String, Node>>>, WitnessGap> {
    search(ir, command, guards, distinction, true)
        .map(|(inputs, between)| between.then_some(inputs))
}

/// Whether the candidates for `guards` try a value in every region the guards' literals divide each
/// leaf into, in every combination: then no candidate deciding two guards together means no input
/// does, and an overlap nothing reached is shown empty rather than merely unreached.
///
/// Holds where every guard is built from `all`, `any`, `not` and comparisons of one input leaf with
/// a literal — an order over a number, an equality over a number, a text, a Boolean or an enum —
/// and the ladders, with the midpoints [`candidates_between`] adds, fit inside [`MAX_CANDIDATES`]
/// together. Anything else — two facts compared, a text ordered or matched, a count, a list, a
/// leaf whose type carries an invariant, an optional left out — answers `false`, which a caller
/// reads as "not shown".
pub fn exhausts(ir: &EssIr, command: &ResolvedCommand, guards: &[&Predicate]) -> bool {
    fn plain(predicate: &Predicate) -> bool {
        match predicate {
            Predicate::Always
            | Predicate::Never
            | Predicate::Truthy(_)
            | Predicate::AnyOf { .. }
            | Predicate::NoneOf { .. } => true,
            Predicate::All(children) | Predicate::Any(children) => children.iter().all(plain),
            Predicate::Not(inner) => plain(inner),
            Predicate::Compare { left, right, .. } => matches!(
                (left, right),
                (Operand::Fact(_), Operand::Literal(_)) | (Operand::Literal(_), Operand::Fact(_))
            ),
            _ => false,
        }
    }
    if !guards.iter().all(|guard| plain(guard)) {
        return false;
    }
    let mut builder = Builder::new(
        ir,
        Distinction::PLAIN,
        list_reads(guards),
        positional_reads(guards),
    );
    if builder.input(command, &BTreeMap::new()).is_err() {
        return false;
    }
    // A candidate breaking an entity invariant of a branch it reaches is solved again or dropped
    // (beyond10x/ess#234), so a region it stood for may go untried.
    if outcome_constraints(ir, command)
        .iter()
        .any(|held| !held.is_empty())
    {
        return false;
    }
    let mut regions: usize = 1;
    for path in read_paths(guards) {
        let Some((leaf, at_base)) = builder.leaves.get(&path) else {
            return false;
        };
        if builder.optionals.contains(&path) {
            return false;
        }
        // A newtype invariant drops candidates the ladder counted, so a region it removes was not
        // tried and an overlap there would read as empty.
        if builder
            .invariants
            .get(&path)
            .is_some_and(|invariants| !invariants.is_empty())
        {
            return false;
        }
        let literals = literals_at(guards, &path);
        let tried = match leaf {
            Leaf::Number { integral } => {
                if *integral
                    && literals
                        .iter()
                        .any(|literal| literal.as_number().is_some_and(|n| !n.is_integral()))
                {
                    return false;
                }
                let mut values = alternatives(leaf, at_base, &literals, true);
                if !*integral {
                    values.extend(midpoints(&literals));
                }
                values.len() + 1
            }
            Leaf::Bool => 2,
            Leaf::Enum { variants } => variants.len(),
            Leaf::Text => {
                if ordered_at(guards, &path)
                    || literals
                        .iter()
                        .any(|literal| matches!((literal.as_text(), at_base), (Some(text), Node::Text(base)) if text == base))
                {
                    return false;
                }
                literals.len() + 1
            }
            Leaf::Timestamp | Leaf::Json => return false,
        };
        regions = regions.saturating_mul(tried.max(1));
    }
    regions <= MAX_CANDIDATES
}

/// The exact midpoint of every two adjacent distinct numeric literals, lowest first.
fn midpoints(literals: &[FactValue]) -> Vec<Node> {
    let mut numbers: Vec<Number> = literals.iter().filter_map(FactValue::as_number).collect();
    numbers.sort_by(|a, b| a.get().total_cmp(&b.get()));
    numbers.dedup();
    numbers
        .windows(2)
        .filter_map(|pair| midpoint(pair[0], pair[1]))
        .map(Node::Number)
        .collect()
}

/// `(a + b) / 2`, exactly where both are exact decimals and the result survives its own write;
/// otherwise the binary64 midpoint. `None` only where that is not finite.
fn midpoint(a: Number, b: Number) -> Option<Number> {
    fn parts(number: Number) -> Option<(i128, u32)> {
        let text = number.exact_text();
        let (negative, body) = match text.strip_prefix('-') {
            Some(body) => (true, body),
            None => (false, text.as_str()),
        };
        let (integer, fraction) = body.split_once('.').unwrap_or((body, ""));
        let units: i128 = format!("{integer}{fraction}").parse().ok()?;
        Some((
            if negative { -units } else { units },
            u32::try_from(fraction.len()).ok()?,
        ))
    }
    let exact = parts(a).zip(parts(b)).and_then(|((a, sa), (b, sb))| {
        let scale = sa.max(sb);
        let widen = |units: i128, from: u32| units.checked_mul(10_i128.checked_pow(scale - from)?);
        // (a + b) / 2 at one more place: (a + b) * 5 at scale + 1.
        let units = widen(a, sa)?.checked_add(widen(b, sb)?)?.checked_mul(5)?;
        let scale = scale + 1;
        let digits = units.unsigned_abs().to_string();
        let width = usize::try_from(scale).ok()?;
        let padded = format!("{digits:0>width$}", width = width + 1);
        let (integer, fraction) = padded.split_at(padded.len() - width);
        let fraction = fraction.trim_end_matches('0');
        let sign = if units < 0 { "-" } else { "" };
        let text = if fraction.is_empty() {
            format!("{sign}{integer}")
        } else {
            format!("{sign}{integer}.{fraction}")
        };
        Number::decimal_literal(&text)
    });
    exact.or_else(|| Number::new(a.get() / 2.0 + b.get() / 2.0).ok())
}

/// The candidates of a no-default command whose guards validation proved a finite partition of,
/// or `None` where that does not apply.
///
/// A newly admitted no-default partition uses exactly the domain that validation proved. Commands
/// with real defaults keep the existing candidate order.
fn finite_partition(
    ir: &EssIr,
    command: &ResolvedCommand,
    guards: &[&Predicate],
    builder: &mut Builder<'_>,
    presence_omits: &BTreeMap<FactPath, Choice>,
) -> Result<Option<Vec<BTreeMap<String, Node>>>, WitnessGap> {
    let partition =
        command.outcomes.iter().all(|outcome| {
            outcome.test_strategy != ess_domain::command::TestStrategy::DefaultBranch
        }) && guards.iter().all(|guard| {
            command
                .outcomes
                .iter()
                .filter_map(crate::when)
                .any(|ordinary| ordinary == *guard)
        });
    if !partition {
        return Ok(None);
    }
    let all_guards: Vec<_> = command.outcomes.iter().filter_map(crate::when).collect();
    let Some(cases) = ess_domain::command::finite::analyze(
        &ess_compiler::expression::Environment::new(ir, &command.input),
        &all_guards,
    ) else {
        return Ok(None);
    };
    let mut inputs = Vec::new();
    for case in cases {
        let overrides: BTreeMap<FactPath, Choice> = case
            .values
            .into_iter()
            .map(|(path, value)| {
                (
                    path,
                    Choice::Value(match value {
                        FactValue::Text(value) => Node::Text(value),
                        FactValue::Bool(value) => Node::Bool(value),
                        FactValue::Number(value) => Node::Number(value),
                    }),
                )
            })
            .collect();
        for input in paired(builder, command, &overrides, presence_omits)? {
            if !inputs.contains(&input) {
                inputs.push(input);
            }
        }
    }
    Ok(Some(admitted_inputs(ir, command, inputs)))
}

/// [`candidates`], and with `between` the midpoints [`candidates_between`] adds; the flag says
/// whether any was added.
fn search(
    ir: &EssIr,
    command: &ResolvedCommand,
    guards: &[&Predicate],
    distinction: Distinction,
    between: bool,
) -> Searched {
    crate::witness_memo::answer(ir, command, guards, distinction, between, || {
        search_uncached(ir, command, guards, distinction, between)
    })
}

/// What one [`search`] answers: the candidates, and whether midpoints were added.
pub(crate) type Searched = Result<(Vec<BTreeMap<String, Node>>, bool), WitnessGap>;

/// [`search`], run.
#[allow(clippy::too_many_lines)]
fn search_uncached(
    ir: &EssIr,
    command: &ResolvedCommand,
    guards: &[&Predicate],
    distinction: Distinction,
    between: bool,
) -> Searched {
    let mut added = false;
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
    // The base witness records every leaf the ladders below are built from; `enumerate` builds it
    // again as its first candidate.
    builder.input(command, &BTreeMap::new())?;
    // The base satisfies the invariants over the input, of its structs and of the entities a
    // branch copies it into (beyond10x/ess#234), before any ladder is drawn from it.
    let constrained = outcome_constraints(ir, command);
    repair(&mut builder, command, &constrained)?;
    // A presence policy (beyond10x/ess#139) is observable only on an absent value, so an optional
    // input, or an optional member of one, copied into a field that declares one and read by no
    // guard is sent absent in every candidate first: each candidate is tried with those members
    // absent before it is tried filled. A member of a copied struct is spelled as its policy spells
    // absence (`null` under `null_when_absent`), so the struct is expected exactly as a correct
    // implementation publishes it; a top-level input is left out. Whichever branch a candidate
    // selects, the first one it takes publishes the policy fields absent, and an implementation
    // that spells the absence the other way fails. No guard reads such a member, so this never
    // changes the branch.
    let presence_omits = presence_inputs(ir, command, &read_paths(&expanded), &builder.optionals);

    if !between {
        if let Some(inputs) = finite_partition(ir, command, guards, &mut builder, &presence_omits)?
        {
            return Ok((
                within_invariants(&mut builder, command, &constrained, inputs)?,
                false,
            ));
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
        if between && matches!(leaf, Leaf::Number { integral: false }) {
            // Tried first: they are what this second search adds.
            let mut inside: Vec<Node> = midpoints(&literals_at(&expanded, &path))
                .into_iter()
                .filter(|value| value != at_base && !alternatives.contains(value))
                .collect();
            added |= !inside.is_empty();
            inside.append(&mut alternatives);
            alternatives = inside;
        }
        if let (Leaf::Text, Node::Text(base)) = (leaf, at_base) {
            let invariants = builder.invariants.get(&path).map_or(&[][..], Vec::as_slice);
            for text in text_alternatives(&expanded, invariants, &path, base) {
                let node = Node::Text(text);
                if &node != at_base && !alternatives.contains(&node) {
                    alternatives.push(node);
                }
            }
        }
        // A base the repair moved onto the guard's own literal is tried at its own witness too,
        // so the guard is still refuted by some candidate (beyond10x/ess#234).
        if let Some(original) = builder.unrepaired.get(&path) {
            if original != at_base && !alternatives.contains(original) {
                alternatives.push(original.clone());
            }
        }
        if !alternatives.is_empty() {
            ladders.insert(path, alternatives.into_iter().map(Choice::Value).collect());
        }
    }
    equality_copies(&builder, &expanded, false, &mut ladders);
    offset_copies(&builder, &expanded, &mut ladders);
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
    let mut inputs = enumerate(&mut builder, command, &ladders, &omitted, &presence_omits)?;
    // A text a guard measures in UTF-8 bytes is tried first at each length it is decided at as text
    // with fewer scalars and UTF-16 units than bytes, so the branch it witnesses is sent text on
    // which no other measure agrees — even where the base text is already that long in ASCII
    // (`docs/design/expression-family-source22.md`, "String `.utf8_bytes`"). Only commands with
    // such a guard get it, so every other suite keeps its bytes.
    wide_first(&mut builder, command, &expanded, &mut inputs)?;
    // A case-insensitive guard's refuting side is witnessed by a one-character change of its
    // literal, not by whatever base text happens to refute it: a target comparing only lengths, or
    // only the first byte folded, accepts the base and passes (beyond10x/ess#140). Tried first, so
    // it is the input the refuting branch is sent; a satisfying branch skips it. Only commands
    // with such a guard get it, so every other suite keeps its bytes.
    let mut refuting = Vec::new();
    for (path, text) in fold_refutations_at(&expanded) {
        let input = builder.input(
            command,
            &BTreeMap::from([(path, Choice::Value(Node::Text(text)))]),
        )?;
        if !refuting.contains(&input) {
            refuting.push(input);
        }
    }
    if !refuting.is_empty() {
        inputs.retain(|input| !refuting.contains(input));
        refuting.extend(inputs);
        inputs = refuting;
        inputs.truncate(MAX_CANDIDATES);
    }
    // A `distinct` guard is decided by lists that hold two or more elements ([`decisive_first`]).
    let mut inputs = decisive_first(&mut builder, command, &expanded, inputs)?;
    // Where the bound cut the walk short, the leaves late in path order never left their first
    // values, and a guard over them was refused. What the walk tried stays first, in its order, so
    // a branch it witnessed is sent the input it was always sent; only a caller that found nothing
    // in it reaches the guard-directed candidates after it.
    if product(&ladders) > MAX_CANDIDATES {
        let solved = Directed::new(&mut builder, command, guards, &ladders).solve()?;
        extend_paired(&mut builder, command, &solved, &presence_omits, &mut inputs)?;
    }
    let inputs = admitted_inputs(ir, command, inputs);
    Ok((
        within_invariants(&mut builder, command, &constrained, inputs)?,
        added,
    ))
}

/// One base witness for a table of declared fields that is not a command's input — an event's
/// payload, a binding's delivery context (beyond10x/ess#195) — at `distinction`.
///
/// The base value [`candidates`] starts from, by the same rules. Two fields of one table differ
/// only where the witness is built from the field's path: `String`, `Uuid`, `Json`, and what is
/// made of them. `Integer`, `Decimal`, `Timestamp`, `Duration`, `Bytes`, `Boolean` and an enum
/// take their value from `distinction` alone, so two such fields of one type carry one value at
/// one distinction; a caller that must keep them apart gives each field its own distinction, as
/// delivery-context synthesis does. Two distinctions give a field different values where its type
/// has two. Every value is checked against its declared type and invariants.
///
/// # Errors
///
/// [`WitnessGap`] when a field has no safe value, or none its declared invariants admit.
pub(crate) fn fields(
    ir: &EssIr,
    fields: &[ess_compiler::ir::ResolvedField],
    distinction: Distinction,
) -> Result<BTreeMap<String, Node>, WitnessGap> {
    let mut builder = Builder::new(ir, distinction, BTreeSet::new(), BTreeMap::new());
    let mut values = BTreeMap::new();
    for field in fields {
        let path = FactPath::new(&field.name).map_err(|_| WitnessGap {
            path: field.name.clone(),
            type_ref: field.type_ref.to_string(),
            reason: "is named in a way no fact path can spell, so no guard could read it",
        })?;
        let Some(value) = builder.member(&field.type_ref, &path, &BTreeMap::new(), 0, true)? else {
            continue;
        };
        if crate::input::validate_typed_value(ir, &field.type_ref, &value).is_err() {
            return Err(WitnessGap {
                path: field.name.clone(),
                type_ref: field.type_ref.to_string(),
                reason: "has no base value its declared invariants admit",
            });
        }
        values.insert(field.name.clone(), value);
    }
    Ok(values)
}

/// A bounded existence candidate for history proofs. It carries no observed-value authority.
/// Uses the ordinary primitive bases, but tries productive union alternatives and empty recursive
/// containers without changing synthesis's existing candidate ordering.
#[allow(
    clippy::items_after_statements,
    clippy::too_many_lines,
    reason = "the bounded recursive builder is private to this proof candidate operation"
)]
pub(crate) fn proof_base(
    ir: &EssIr,
    kind: &ResolvedTypeRef,
    budget: &crate::input::ProofBudget,
) -> Option<Node> {
    budget.witness_candidate()?;
    fn build(
        ir: &EssIr,
        kind: &ResolvedTypeRef,
        budget: &crate::input::ProofBudget,
        active: &mut BTreeSet<String>,
        depth: usize,
    ) -> Option<Node> {
        budget.charge(1).ok()?;
        if depth > MAX_TYPE_DEPTH {
            return None;
        }
        match kind {
            ResolvedTypeRef::Optional { .. } => Some(Node::Null),
            ResolvedTypeRef::List { .. } => Some(Node::Seq(Vec::new())),
            ResolvedTypeRef::Map { .. } => Some(Node::Map(BTreeMap::new())),
            ResolvedTypeRef::Primitive { name } => {
                if *name == Primitive::Binary64 {
                    return None;
                }
                budget.charge(128).ok()?;
                Some(primitive_value(
                    *name,
                    &FactPath::new("generated").expect("static path"),
                    Distinction::PLAIN,
                ))
            }
            ResolvedTypeRef::Declared { name } => {
                let declared = ir.named_type(name);
                for segment in declared.name.segments() {
                    budget.charge(1 + segment.len()).ok()?;
                }
                let key = declared.name.to_string();
                if !active.insert(key.clone()) {
                    return None;
                }
                let result = (|| match &declared.body {
                    ResolvedBody::Newtype {
                        of,
                        prefix,
                        alphabet,
                        ..
                    } => {
                        let mut value = build(ir, of, budget, active, depth + 1)?;
                        if let Node::Text(text) = &mut value {
                            if let Some(prefix) = prefix {
                                budget.charge(prefix.len().checked_add(text.len())?).ok()?;
                                *text = with_prefix(text, prefix);
                            }
                            if let Some(alphabet) = alphabet {
                                budget
                                    .charge(alphabet.len().checked_mul(text.len().max(1))?)
                                    .ok()?;
                                *text = into_alphabet(text, alphabet);
                            }
                        }
                        Some(value)
                    }
                    ResolvedBody::Enum { variants } => {
                        let variant = variants.first()?;
                        budget.charge(variant.name().len()).ok()?;
                        Some(Node::Text(variant.name().to_owned()))
                    }
                    ResolvedBody::Struct { fields, .. } => {
                        let mut values = BTreeMap::new();
                        for field in fields {
                            budget.charge(1 + field.name.len()).ok()?;
                            values.insert(
                                field.name.clone(),
                                build(ir, &field.type_ref, budget, active, depth + 1)?,
                            );
                        }
                        Some(Node::Map(values))
                    }
                    ResolvedBody::Union { tag, variants } => {
                        for (label, kind) in variants {
                            budget.witness_candidate()?;
                            budget.charge(1 + tag.len() + label.len()).ok()?;
                            let built = match kind {
                                Some(kind) => build(ir, kind, budget, active, depth + 1)
                                    .map(|value| (ess_gen::schema::union_content_key(tag), value)),
                                // A unit variant (ess/22) is the tag alone.
                                None => None,
                            };
                            if kind.is_none() || built.is_some() {
                                let mut value =
                                    BTreeMap::from([(tag.clone(), Node::Text(label.clone()))]);
                                if let Some((content, built)) = built {
                                    value.insert(content.into(), built);
                                }
                                let value = Node::Map(value);
                                if crate::input::validate_typed_value_bounded(
                                    ir,
                                    &ResolvedTypeRef::Declared { name: name.clone() },
                                    &value,
                                    budget,
                                )
                                .is_ok()
                                {
                                    return Some(value);
                                }
                            }
                        }
                        None
                    }
                })();
                active.remove(&key);
                result
            }
        }
    }
    build(ir, kind, budget, &mut BTreeSet::new(), 0)
}

/// The optional inputs, and optional members of inputs, that a branch copies into an emitted event
/// field declaring a presence policy or holding a struct member that does (beyond10x/ess#139), and
/// that no guard reads — neither the input itself nor anything under it.
///
/// Only a direct `input.<field>` copy: `{input: x, else: {generated: true}}` mints a value when
/// the input is absent, so leaving it out observes nothing about the policy.
fn presence_inputs(
    ir: &EssIr,
    command: &ResolvedCommand,
    read: &BTreeSet<FactPath>,
    optionals: &BTreeSet<FactPath>,
) -> BTreeMap<FactPath, Choice> {
    let mut found = BTreeMap::new();
    for outcome in &command.outcomes {
        for payload in &outcome.payload {
            let Some(event) = ir.events().get(payload.event.name()) else {
                continue;
            };
            for field in &payload.fields {
                let ess_compiler::ir::ResolvedPayloadValue::InputField { field: source, .. } =
                    &field.value
                else {
                    continue;
                };
                let Some(target) = event
                    .fields
                    .iter()
                    .find(|declared| declared.name == field.target)
                else {
                    continue;
                };
                let Ok(root) = FactPath::new(source) else {
                    continue;
                };
                if read
                    .iter()
                    .any(|read| read.segments().first() == root.segments().first())
                {
                    continue;
                }
                // The field itself, and every member of a struct it is under no `Optional` of —
                // the leaves a suite carries a policy on — read at the same path in the input.
                let mut declaring = Vec::new();
                policy_members(ir, target, &root, 0, &mut declaring);
                // A member of a copied struct is spelled as its policy spells an absent value, because
                // the whole struct is asserted as the value sent. A top-level field is left out: no
                // value is asserted for an input that was not sent, only its declared shape.
                for (path, presence) in declaring {
                    if optionals.contains(&path) {
                        let member = path.segments().len() > 1;
                        found.entry(path).or_insert(
                            if member && presence == ess_domain::types::Presence::NullWhenAbsent {
                                Choice::Null
                            } else {
                                Choice::Omit
                            },
                        );
                    }
                }
            }
        }
    }
    found
}

/// The paths under `at` of `field` and of every struct member it reaches through newtypes and
/// structs that declare a presence policy. Stops at an `Optional` below the field itself, where a
/// suite carries no policy (`synthesize::mark_presence`), and at [`MAX_TYPE_DEPTH`].
fn policy_members(
    ir: &EssIr,
    field: &ess_compiler::ir::ResolvedField,
    at: &FactPath,
    depth: usize,
    found: &mut Vec<(FactPath, ess_domain::types::Presence)>,
) {
    if depth > MAX_TYPE_DEPTH {
        return;
    }
    if let Some(presence) = field.naming.presence {
        found.push((at.clone(), presence));
    }
    let mut current = &field.type_ref;
    for _ in 0..=MAX_TYPE_DEPTH {
        match current {
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => current = of,
                ResolvedBody::Struct { fields, .. } => {
                    for member in fields {
                        policy_members(ir, member, &at.child(&member.name), depth + 1, found);
                    }
                    return;
                }
                ResolvedBody::Enum { .. } | ResolvedBody::Union { .. } => return,
            },
            _ => return,
        }
    }
}

/// The base, then the distinct candidates the ladders describe, then each omission alone — at most
/// [`MAX_CANDIDATES`] in all.
fn enumerate(
    builder: &mut Builder<'_>,
    command: &ResolvedCommand,
    ladders: &[(FactPath, Vec<Choice>)],
    omitted: &[FactPath],
    presence_omits: &BTreeMap<FactPath, Choice>,
) -> Result<Vec<BTreeMap<String, Node>>, WitnessGap> {
    let total = product(ladders);
    let reserved = if total <= MAX_CANDIDATES {
        0
    } else {
        omitted.len()
    };
    let enumerated = MAX_CANDIDATES.saturating_sub(reserved).max(1);

    let mut inputs = Vec::new();
    for input in paired(builder, command, &BTreeMap::new(), presence_omits)? {
        if !inputs.contains(&input) {
            inputs.push(input);
        }
    }
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
        for input in paired(builder, command, &overrides, presence_omits)? {
            if !inputs.contains(&input) {
                inputs.push(input);
            }
        }
        index += 1;
    }
    for path in omitted {
        if inputs.len() >= MAX_CANDIDATES {
            break;
        }
        let overrides = BTreeMap::from([(path.clone(), Choice::Omit)]);
        for input in paired(builder, command, &overrides, presence_omits)? {
            if !inputs.contains(&input) {
                inputs.push(input);
            }
        }
    }
    // A pair can overshoot the bound by one; the absent half of a pair always comes first.
    inputs.truncate(MAX_CANDIDATES);
    Ok(inputs)
}

/// One candidate as `overrides` describe it: first with every presence-policy member left out
/// where `overrides` does not name it, then as written. The same single input when there is
/// nothing to leave out, so a model without a policy keeps exactly the candidates it had.
fn paired(
    builder: &mut Builder<'_>,
    command: &ResolvedCommand,
    overrides: &BTreeMap<FactPath, Choice>,
    presence_omits: &BTreeMap<FactPath, Choice>,
) -> Result<Vec<BTreeMap<String, Node>>, WitnessGap> {
    let written = builder.input(command, overrides)?;
    if presence_omits
        .keys()
        .all(|path| overrides.contains_key(path))
    {
        return Ok(vec![written]);
    }
    let mut omitting = presence_omits.clone();
    omitting.extend(
        overrides
            .iter()
            .map(|(path, choice)| (path.clone(), choice.clone())),
    );
    let absent = builder.input(command, &omitting)?;
    Ok(if absent == written {
        vec![written]
    } else {
        vec![absent, written]
    })
}

/// What a candidate puts in place of the base witness, by path.
type Overrides = BTreeMap<FactPath, Choice>;

/// Appends each of `solved` as [`paired`] spells it, skipping inputs already in `inputs`, at most
/// [`MAX_CANDIDATES`] of them.
fn extend_paired(
    builder: &mut Builder<'_>,
    command: &ResolvedCommand,
    solved: &[Overrides],
    presence_omits: &BTreeMap<FactPath, Choice>,
    inputs: &mut Vec<BTreeMap<String, Node>>,
) -> Result<(), WitnessGap> {
    let limit = inputs.len().saturating_add(MAX_CANDIDATES);
    for overrides in solved {
        for input in paired(builder, command, overrides, presence_omits)? {
            if !inputs.contains(&input) {
                inputs.push(input);
            }
        }
    }
    inputs.truncate(limit);
    Ok(())
}

/// Candidates solved for what a guard's branches need, rather than walked in path order.
///
/// Each atom under a guard's connectives reads some ladders. Ladders an atom reads together
/// (`w.hi > w.lo`) form one group; an atom reading one path (`a > 10`) has a group of its own. A
/// goal is a set of truth values wanted of some sub-predicates; it is broken down to truth values
/// of atoms, and each group is then searched on its own, base first, for the first combination of
/// its alternatives that gives its atoms those values. So a conjunction of seven comparisons is one
/// search per field, not the 128th of 2^7 assignments, and the result does not depend on how the
/// inputs are named.
///
/// The goals, in order: each guard satisfied with every other refuted, then satisfied alone, then
/// refuted; every guard refuted; then, for each connective of two or more children, each child
/// deciding alone against the others (the rows a connective mutant is killed by), and every child
/// deciding alike — each with the guard satisfied, refuted, and either way.
///
/// The limits are the ladders' and the bound's: a group is searched over at most
/// [`MAX_CANDIDATES`] × [`MAX_ENUMERATED_PER_CANDIDATE`] combinations, a goal gives up after
/// [`MAX_CANDIDATES`] breakdowns, and at most [`MAX_CANDIDATES`] candidates are solved. A value no
/// ladder holds (`amount > 0.1 and amount < 0.2`) is not found here; [`candidates_between`] finds
/// it. A field compared only with other fields writes no literal, so its ladder is 0 and -1 beside
/// its base: a strict chain over four such fields has no solution here.
struct Directed<'a, 'ir> {
    builder: &'a mut Builder<'ir>,
    command: &'a ResolvedCommand,
    guards: &'a [&'a Predicate],
    /// The distinct atoms under the guards' connectives.
    atoms: Vec<&'a Predicate>,
    /// Ladder paths atoms read together, each with its alternatives.
    groups: Vec<Vec<(FactPath, Vec<Choice>)>>,
    /// The group each atom reads, or `None` for an atom no ladder varies.
    reads: Vec<Option<usize>>,
    /// Each atom's decision at a group's combination (`None`: undecided), filled on demand.
    truths: BTreeMap<(usize, usize), Vec<Option<bool>>>,
}

impl<'a, 'ir> Directed<'a, 'ir> {
    fn new(
        builder: &'a mut Builder<'ir>,
        command: &'a ResolvedCommand,
        guards: &'a [&'a Predicate],
        ladders: &[(FactPath, Vec<Choice>)],
    ) -> Self {
        let mut atoms: Vec<&Predicate> = Vec::new();
        for guard in guards {
            for atom in guard_atoms(guard) {
                if !atoms.contains(&atom) {
                    atoms.push(atom);
                }
            }
        }
        // One entry per path: an optional read by a comparison and by `defined` has two ladders.
        let mut options: Vec<(FactPath, Vec<Choice>)> = Vec::new();
        for (path, alternatives) in ladders {
            let at = if let Some(at) = options.iter().position(|(known, _)| known == path) {
                at
            } else {
                options.push((path.clone(), Vec::new()));
                options.len() - 1
            };
            for choice in alternatives {
                if !options[at].1.contains(choice) {
                    options[at].1.push(choice.clone());
                }
            }
        }
        // A path read at, under or above a varied one reads what that ladder varies.
        let related = |read: &FactPath, varied: &FactPath| {
            read.segments().starts_with(varied.segments())
                || varied.segments().starts_with(read.segments())
        };
        let involved: Vec<Vec<usize>> = atoms
            .iter()
            .map(|atom| {
                let reads = atom.fact_paths();
                (0..options.len())
                    .filter(|at| reads.iter().any(|read| related(read, &options[*at].0)))
                    .collect()
            })
            .collect();
        // Every ladder starts in a group of its own; an atom reading several joins theirs.
        let mut label: Vec<usize> = (0..options.len()).collect();
        for paths in &involved {
            let joined: Vec<usize> = paths.iter().map(|at| label[*at]).collect();
            if let Some(lowest) = joined.iter().copied().min() {
                for held in &mut label {
                    if joined.contains(held) {
                        *held = lowest;
                    }
                }
            }
        }
        let mut labels: Vec<usize> = Vec::new();
        let mut groups: Vec<Vec<(FactPath, Vec<Choice>)>> = Vec::new();
        for (at, option) in options.into_iter().enumerate() {
            if let Some(group) = labels.iter().position(|known| *known == label[at]) {
                groups[group].push(option);
            } else {
                labels.push(label[at]);
                groups.push(vec![option]);
            }
        }
        let reads = involved
            .iter()
            .map(|paths| {
                let first = label[*paths.first()?];
                labels.iter().position(|known| *known == first)
            })
            .collect();
        Self {
            builder,
            command,
            guards,
            atoms,
            groups,
            reads,
            truths: BTreeMap::new(),
        }
    }

    /// The solved override sets, distinct, at most [`MAX_CANDIDATES`] of them.
    fn solve(mut self) -> Result<Vec<Overrides>, WitnessGap> {
        let mut solved: Vec<Overrides> = Vec::new();
        for goal in self.goals() {
            if solved.len() >= MAX_CANDIDATES {
                break;
            }
            let mut budget = MAX_CANDIDATES;
            if let Some(overrides) = self.search(goal, &mut BTreeMap::new(), &mut budget)? {
                if !solved.contains(&overrides) {
                    solved.push(overrides);
                }
            }
        }
        Ok(solved)
    }

    fn goals(&self) -> Vec<Vec<(&'a Predicate, bool)>> {
        let guards = self.guards;
        let mut goals = Vec::new();
        for (at, guard) in guards.iter().enumerate() {
            let mut only = vec![(*guard, true)];
            only.extend(
                guards
                    .iter()
                    .enumerate()
                    .filter(|(other, _)| *other != at)
                    .map(|(_, other)| (*other, false)),
            );
            goals.push(only);
            goals.push(vec![(*guard, true)]);
            goals.push(vec![(*guard, false)]);
        }
        goals.push(guards.iter().map(|guard| (*guard, false)).collect());
        for guard in guards {
            for children in connectives(guard) {
                let mut rows = Vec::new();
                for alone in [true, false] {
                    for at in 0..children.len() {
                        rows.push(
                            children
                                .iter()
                                .enumerate()
                                .map(|(other, child)| (child, (other == at) == alone))
                                .collect::<Vec<_>>(),
                        );
                    }
                    rows.push(children.iter().map(|child| (child, alone)).collect());
                }
                for row in rows {
                    for context in [Some(true), Some(false), None] {
                        let mut goal = row.clone();
                        goal.extend(context.map(|wanted| (*guard, wanted)));
                        goals.push(goal);
                    }
                }
            }
        }
        goals
    }

    /// The first override set meeting every wanted truth in `pending` and `wanted`.
    fn search(
        &mut self,
        mut pending: Vec<(&'a Predicate, bool)>,
        wanted: &mut BTreeMap<usize, bool>,
        budget: &mut usize,
    ) -> Result<Option<Overrides>, WitnessGap> {
        if *budget == 0 {
            return Ok(None);
        }
        let Some((predicate, value)) = pending.pop() else {
            *budget -= 1;
            return self.check(wanted);
        };
        match predicate {
            Predicate::All(children) | Predicate::Any(children)
                if matches!(predicate, Predicate::All(_)) == value =>
            {
                pending.extend(children.iter().map(|child| (child, value)));
                self.search(pending, wanted, budget)
            }
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    let mut next = pending.clone();
                    next.push((child, value));
                    if let Some(found) = self.search(next, wanted, budget)? {
                        return Ok(Some(found));
                    }
                }
                Ok(None)
            }
            Predicate::Not(inner) => {
                pending.push((inner, !value));
                self.search(pending, wanted, budget)
            }
            atom => {
                let Some(at) = self.atoms.iter().position(|known| *known == atom) else {
                    return Ok(None);
                };
                match wanted.get(&at) {
                    Some(held) if *held != value => Ok(None),
                    Some(_) => self.search(pending, wanted, budget),
                    None => {
                        wanted.insert(at, value);
                        let found = self.search(pending, wanted, budget);
                        wanted.remove(&at);
                        found
                    }
                }
            }
        }
    }

    /// Each group searched, base first, for the first combination that decides its atoms as
    /// `wanted` says; `None` when some group has none.
    fn check(&mut self, wanted: &BTreeMap<usize, bool>) -> Result<Option<Overrides>, WitnessGap> {
        let mut overrides = Overrides::new();
        let mut groups: BTreeSet<Option<usize>> = BTreeSet::new();
        for at in wanted.keys() {
            groups.insert(self.reads[*at]);
        }
        for group in groups {
            let combinations = group.map_or(1, |group| {
                product(&self.groups[group])
                    .min(MAX_CANDIDATES.saturating_mul(MAX_ENUMERATED_PER_CANDIDATE))
            });
            let mut met = None;
            for combination in 0..combinations {
                let truths = self.truths(group, combination)?;
                if wanted
                    .iter()
                    .filter(|(at, _)| self.reads[**at] == group)
                    .all(|(at, value)| truths[*at] == Some(*value))
                {
                    met = Some(combination);
                    break;
                }
            }
            let Some(combination) = met else {
                return Ok(None);
            };
            if let Some(group) = group {
                overrides.extend(self.combination(group, combination));
            }
        }
        Ok(Some(overrides))
    }

    /// The overrides at `combination` of `group`, first path fastest, `0` the base.
    fn combination(&self, group: usize, combination: usize) -> Overrides {
        let mut overrides = Overrides::new();
        let mut remaining = combination;
        for (path, alternatives) in &self.groups[group] {
            let radix = alternatives.len() + 1;
            let chosen = remaining % radix;
            remaining /= radix;
            if chosen > 0 {
                overrides.insert(path.clone(), alternatives[chosen - 1].clone());
            }
        }
        overrides
    }

    /// How every atom decides at `combination` of `group`, everything else at its base.
    fn truths(
        &mut self,
        group: Option<usize>,
        combination: usize,
    ) -> Result<Vec<Option<bool>>, WitnessGap> {
        let key = (group.unwrap_or(usize::MAX), combination);
        if let Some(known) = self.truths.get(&key) {
            return Ok(known.clone());
        }
        let overrides =
            group.map_or_else(Overrides::new, |group| self.combination(group, combination));
        let input = self.builder.input(self.command, &overrides)?;
        let truths = match crate::flatten(self.builder.ir, self.command, &input) {
            Ok(facts) => self
                .atoms
                .iter()
                .map(|atom| match facts.decide(atom) {
                    crate::Decision::Satisfied => Some(true),
                    crate::Decision::Refuted(_) => Some(false),
                    crate::Decision::Unevaluable(_) => None,
                })
                .collect(),
            Err(_) => vec![None; self.atoms.len()],
        };
        self.truths.insert(key, truths.clone());
        Ok(truths)
    }
}

/// The atoms of a guard: everything under its connectives.
fn guard_atoms(predicate: &Predicate) -> Vec<&Predicate> {
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => {
            children.iter().flat_map(guard_atoms).collect()
        }
        Predicate::Not(inner) => guard_atoms(inner),
        atom => vec![atom],
    }
}

/// The children of every connective of two or more children in a guard, outermost first.
fn connectives(predicate: &Predicate) -> Vec<&[Predicate]> {
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => {
            let mut found = Vec::new();
            if children.len() >= 2 {
                found.push(children.as_slice());
            }
            found.extend(children.iter().flat_map(connectives));
            found
        }
        Predicate::Not(inner) => connectives(inner),
        _ => Vec::new(),
    }
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
    /// The `Optional` member at this path sent as an explicit `null`: its absence spelled the way the
    /// `null_when_absent` field it is copied into spells it (beyond10x/ess#139), so the value a
    /// suite expects there is the one a correct implementation publishes.
    Null,
    /// A list of this many elements: element 0 built at `<path>.0` from the declared element type
    /// like any other value and varied there by the same ladders, and every further element a
    /// copy of it, so a quantifier is decided by element 0 alone, as it was with one element.
    Elements(usize),
    /// A list a `distinct` reads (ess/22): element 0 as [`Self::Elements`] builds it, and each further
    /// element built at its own position and a further [`Distinction`], so its key differs; with a
    /// [`Repeat`], the last element then carries element 0's key again.
    Keyed(Keyed),
}

/// The shape of a list a `distinct` reads (`docs/design/expression-family-source22.md`,
/// `distinct`): how many elements, and whether the last repeats the first one's key.
#[derive(Debug, Clone, PartialEq)]
struct Keyed {
    /// How many elements the list holds.
    held: usize,
    /// Where the last element repeats element 0's key; `None` keeps every key apart.
    repeat: Option<Repeat>,
}

/// The key a duplicate repeats: the members under the element that hold it (none for the element
/// itself), and whether it is an instant, which is repeated in another spelling.
#[derive(Debug, Clone, PartialEq)]
struct Repeat {
    member: Vec<String>,
    instant: bool,
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
    remapped(
        predicate,
        &|path: &FactPath| {
            (path.namespace() == bind).then(|| {
                let mut moved = prefix.clone();
                for segment in &path.segments()[1..] {
                    moved = moved.child(segment);
                }
                moved
            })
        },
        &[],
    )
}

/// `predicate` with every read `map` answers moved to the path it answers, and every other read
/// kept. A read rooted at a quantifier's binder is the quantifier's own, so it is never offered to
/// `map`: `bound` holds the binders in scope.
fn remapped(
    predicate: &Predicate,
    map: &dyn Fn(&FactPath) -> Option<FactPath>,
    bound: &[&str],
) -> Predicate {
    let path = |read: &FactPath| {
        if bound.contains(&read.namespace()) {
            read.clone()
        } else {
            map(read).unwrap_or_else(|| read.clone())
        }
    };
    let operand = |operand: &Operand| operand.map_path(path);
    match predicate {
        Predicate::Always => Predicate::Always,
        Predicate::Never => Predicate::Never,
        Predicate::All(children) => Predicate::All(
            children
                .iter()
                .map(|child| remapped(child, map, bound))
                .collect(),
        ),
        Predicate::Any(children) => Predicate::Any(
            children
                .iter()
                .map(|child| remapped(child, map, bound))
                .collect(),
        ),
        Predicate::Not(inner) => Predicate::Not(Box::new(remapped(inner, map, bound))),
        Predicate::Compare {
            left,
            op,
            right,
            kind,
        } => Predicate::Compare {
            kind: *kind,
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
        Predicate::FoldMatch {
            path: read,
            op,
            values,
        } => Predicate::FoldMatch {
            path: path(read),
            op: *op,
            values: values.clone(),
        },
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            let mut inside = bound.to_vec();
            inside.push(&quantified.bind);
            let inner = ess_primitives::predicate::Quantified {
                over: path(&quantified.over),
                bind: quantified.bind.clone(),
                body: remapped(&quantified.body, map, &inside),
            };
            if matches!(predicate, Predicate::Forall(_)) {
                Predicate::Forall(Box::new(inner))
            } else {
                Predicate::Exists(Box::new(inner))
            }
        }
        Predicate::Distinct(distinct) => {
            Predicate::Distinct(Box::new(ess_primitives::predicate::Distinct {
                over: path(&distinct.over),
                ..(**distinct).clone()
            }))
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
            Predicate::Compare {
                left, op, right, ..
            } => {
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
        Predicate::Compare {
            left, op: _, right, ..
        } => {
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
        // A literal with its ASCII case changed satisfies its own operator as the literal does, and
        // only an implementation that folds case accepts it: tried in the literal's place, it is the
        // witness a byte-wise comparison fails (beyond10x/ess#140). A literal with no ASCII letter
        // is its own case change.
        Predicate::FoldMatch {
            path: read, values, ..
        } => {
            if read == path {
                found.extend(values.iter().map(|value| match value {
                    FactValue::Text(text) => FactValue::Text(swap_ascii_case(text)),
                    other => other.clone(),
                }));
            }
        }
        Predicate::Always
        | Predicate::Never
        | Predicate::Truthy(_)
        | Predicate::Defined(_)
        | Predicate::Distinct(_) => {}
    }
}

/// The longest list a `distinct` is shaped to ([`Choice::Keyed`]): a `.count` compared with a
/// larger literal keeps the count ladder's own lengths.
const MAX_KEYED: usize = 16;

/// One list a `distinct` among the guards reads ([`keyed_lists`]).
struct KeyedList {
    /// Where it sits in the input: `files`, or `groups.0.members` under a quantifier rebound onto
    /// its first element.
    path: FactPath,
    /// The key a duplicate repeats.
    repeat: Repeat,
    /// The lengths a guard's `.count` of the list is compared at, from two to [`MAX_KEYED`].
    required: Vec<usize>,
}

/// The candidates for `guards` with the ones that decide each `distinct` among them first
/// (`docs/design/expression-family-source22.md`, `distinct`).
///
/// A `distinct` guard's two sides are witnessed by lists that decide it rather than by a list of
/// none or one, which holds vacuously and passes a target that compares nothing: for each list,
/// a duplicate that is neither adjacent nor whole — the third element repeats the first one's key,
/// in another spelling where it is an instant — beside every other such list held distinct, and a
/// duplicate at each length a guard's `.count` of the list requires; then every list distinct with
/// two elements; then each list distinct at each such length. A list under a quantifier is shaped
/// in the first element of each enclosing list, which then holds one. Tried first, so the refusing
/// branch is sent the duplicate and the accepting branch the distinct lists. Only commands with
/// such a guard get them, so every other suite keeps its bytes.
fn decisive_first(
    builder: &mut Builder<'_>,
    command: &ResolvedCommand,
    guards: &[&Predicate],
    mut inputs: Vec<BTreeMap<String, Node>>,
) -> Result<Vec<BTreeMap<String, Node>>, WitnessGap> {
    let keyed = keyed_lists(builder, guards);
    if keyed.is_empty() {
        return Ok(inputs);
    }
    let shaped = |held: usize, repeat: Option<&Repeat>| {
        Choice::Keyed(Keyed {
            held,
            repeat: repeat.cloned(),
        })
    };
    let mut spread: BTreeMap<FactPath, Choice> = BTreeMap::new();
    for list in &keyed {
        for enclosing in enclosing_lists(builder, &list.path) {
            spread.entry(enclosing).or_insert(Choice::Elements(1));
        }
        spread.insert(list.path.clone(), shaped(2, None));
    }
    let mut choices: Vec<BTreeMap<FactPath, Choice>> = Vec::new();
    for list in &keyed {
        for held in
            std::iter::once(3).chain(list.required.iter().copied().filter(|held| *held != 3))
        {
            let mut overrides = spread.clone();
            overrides.insert(list.path.clone(), shaped(held, Some(&list.repeat)));
            choices.push(overrides);
        }
    }
    choices.push(spread.clone());
    for list in &keyed {
        for &held in list.required.iter().filter(|held| **held != 2) {
            let mut overrides = spread.clone();
            overrides.insert(list.path.clone(), shaped(held, None));
            choices.push(overrides);
        }
    }
    let mut decisive = Vec::new();
    for overrides in &choices {
        let input = builder.input(command, overrides)?;
        if !decisive.contains(&input) {
            decisive.push(input);
        }
    }
    inputs.retain(|input| !decisive.contains(input));
    decisive.extend(inputs);
    inputs = decisive;
    inputs.truncate(MAX_CANDIDATES);
    Ok(inputs)
}

/// The lists enclosing `path` at their first element — `groups` for `groups.0.members` — that the
/// builder varies: each must hold an element for the list inside it to exist.
fn enclosing_lists(builder: &Builder<'_>, path: &FactPath) -> Vec<FactPath> {
    let segments = path.segments();
    (1..segments.len())
        .filter(|&end| segments[end] == "0")
        .map(|end| FactPath::from_segments(&segments[..end]))
        .filter(|enclosing| builder.lists.contains(enclosing))
        .collect()
}

/// Every list of the input a guard's `distinct` reads outside any quantifier — and, through the
/// quantifier bodies the search rebinds onto their first element, inside one — each with the key
/// its duplicate repeats and the lengths a guard's `.count` of it is compared at: the lists
/// [`Choice::Keyed`] shapes. In guard order, each once.
fn keyed_lists(builder: &Builder<'_>, guards: &[&Predicate]) -> Vec<KeyedList> {
    let mut found: Vec<KeyedList> = Vec::new();
    for guard in guards {
        for (distinct, scope) in guard.distincts() {
            if !scope.is_empty()
                || !builder.lists.contains(&distinct.over)
                || found.iter().any(|list| list.path == distinct.over)
            {
                continue;
            }
            let member = distinct
                .key
                .as_ref()
                .map(|key| key.segments()[1..].to_vec())
                .unwrap_or_default();
            let instant = distinct.key_kind == Some(DistinctKeyKind::Timestamp);
            let mut required = Vec::new();
            for held in count_lengths(guards, &distinct.over.child("count"), false) {
                if (2..=MAX_KEYED).contains(&held) && !required.contains(&held) {
                    required.push(held);
                }
            }
            found.push(KeyedList {
                path: distinct.over.clone(),
                repeat: Repeat { member, instant },
                required,
            });
        }
    }
    found
}

/// Why no list of the input a `distinct` among `guards` reads can be distinct at a length the
/// guards compare its `.count` with: its key is a `Boolean` or an enum with fewer values than that
/// length (`docs/design/expression-family-source22.md`, `distinct`: a finite key domain that cannot
/// supply enough unequal values is the named no-witness refusal). `None` where every such domain
/// is large enough, or no `.count` is compared. A caller asks only where no candidate decided the
/// branch, so the gap explains the refusal rather than predicting it.
pub(crate) fn exhausted_key_domain(
    ir: &EssIr,
    command: &ResolvedCommand,
    guards: &[&Predicate],
) -> Option<WitnessGap> {
    let mut expanded: Vec<Predicate> = guards.iter().map(|guard| (*guard).clone()).collect();
    for guard in guards {
        element_bodies(guard, &mut expanded);
    }
    let expanded: Vec<&Predicate> = expanded.iter().collect();
    for guard in &expanded {
        for (distinct, scope) in guard.distincts() {
            if !scope.is_empty() {
                continue;
            }
            let mut key = distinct.over.child("0");
            if let Some(by) = &distinct.key {
                for segment in &by.segments()[1..] {
                    key = key.child(segment);
                }
            }
            let Ok(resolved) =
                ess_compiler::expression::resolve_path(ir, &command.input, &key, "distinct key")
            else {
                continue;
            };
            let values = match (&resolved.variants, &resolved.terminal) {
                (Some(variants), _) => variants.len(),
                (
                    None,
                    ResolvedTypeRef::Primitive {
                        name: Primitive::Boolean,
                    },
                ) => 2,
                _ => continue,
            };
            let longest = count_lengths(&expanded, &distinct.over.child("count"), false)
                .into_iter()
                .filter(|held| *held >= 2)
                .max();
            if longest.is_some_and(|longest| longest > values) || values < 2 {
                return Some(WitnessGap {
                    path: distinct.over.to_string(),
                    type_ref: resolved.declared,
                    reason: "has fewer values than the length the guards require the list at, \
                             so no list of that length holds distinct keys",
                });
            }
        }
    }
    None
}

/// The value at `member` under `value`, which is `value` itself for no member.
fn member_at<'n>(value: &'n Node, member: &[String]) -> Option<&'n Node> {
    member.iter().try_fold(value, |at, segment| match at {
        Node::Map(fields) => fields.get(segment),
        _ => None,
    })
}

/// [`member_at`], to write.
fn member_at_mut<'n>(value: &'n mut Node, member: &[String]) -> Option<&'n mut Node> {
    member.iter().try_fold(value, |at, segment| match at {
        Node::Map(fields) => fields.get_mut(segment),
        _ => None,
    })
}

/// The instant `value` names, spelled with a `-05:00` offset rather than as written, or `None`
/// where it names none: a duplicate only an instant comparison finds.
fn respelled(value: &Node) -> Option<Node> {
    let Node::Text(text) = value else {
        return None;
    };
    let instant = Rfc3339Instant::parse_rfc3339(text)?;
    let local = instant.plus_elapsed(-5 * 3_600)?.to_rfc3339();
    let spelled = format!("{}-05:00", local.strip_suffix('Z')?);
    (spelled != *text && Rfc3339Instant::parse_rfc3339(&spelled) == Some(instant))
        .then_some(Node::Text(spelled))
}

/// `text` with every ASCII letter in the other case and every other character kept.
fn swap_ascii_case(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_ascii_uppercase() {
                character.to_ascii_lowercase()
            } else {
                character.to_ascii_uppercase()
            }
        })
        .collect()
}

/// A fold literal with one character changed: the same length, and equal under ASCII folding to
/// none of `members` — every literal the guards compare the path with — so it refutes
/// `equals_ignore_case`, and `in_ignore_case` even where changing one member's first character
/// lands on another (`web`, `xeb`, `yeb`). The first character is tried first, then each later
/// one, each with `x`, `y`, `z`, `q` and `j` in turn. `None` for the empty literal, which every
/// other text refutes already, and where no such change exists.
fn fold_refuting(literal: &str, members: &[String]) -> Option<String> {
    let characters: Vec<char> = literal.chars().collect();
    for index in 0..characters.len() {
        for replacement in ['x', 'y', 'z', 'q', 'j'] {
            if characters[index].eq_ignore_ascii_case(&replacement) {
                continue;
            }
            let mut changed = characters.clone();
            changed[index] = replacement;
            let changed: String = changed.into_iter().collect();
            if !members
                .iter()
                .any(|member| member.eq_ignore_ascii_case(&changed))
            {
                return Some(changed);
            }
        }
    }
    None
}

/// Per fold-guarded path, the input text that refutes the guards there by one character: the
/// refuting side of a case-insensitive guard, tried ahead of every other candidate so that it is
/// the witness the refuting branch is sent (beyond10x/ess#140).
fn fold_refutations_at(guards: &[&Predicate]) -> Vec<(FactPath, String)> {
    let mut found = Vec::new();
    for path in read_paths(guards) {
        let members = fold_literals_at(guards, &path);
        if let Some(changed) = members
            .first()
            .and_then(|literal| fold_refuting(literal, &members))
        {
            found.push((path, changed));
        }
    }
    found
}

/// Per fold-guarded path of `guard`, each literal in the case only Unicode folding equates with it
/// ([`unicode_refuting`]) — the further witness a refuting branch is sent so that a target folding
/// Unicode, not ASCII, fails the suite (beyond10x/ess#140).
pub(crate) fn unicode_refutations(guard: &Predicate) -> Vec<(FactPath, Node)> {
    let guards = [guard];
    let mut found = Vec::new();
    for path in read_paths(&guards) {
        let members = fold_literals_at(&guards, &path);
        for literal in &members {
            if let Some(text) = unicode_refuting(literal) {
                if !members
                    .iter()
                    .any(|member| member.eq_ignore_ascii_case(&text))
                {
                    found.push((path.clone(), Node::Text(text)));
                }
            }
        }
    }
    found
}

/// A fold literal in a case only Unicode folding equates with it: each non-ASCII letter in its
/// other case (`café` to `CAFÉ`), or — where it has none — its first `k` as U+212A KELVIN SIGN or
/// first `s` as U+017F LONG S, both of which Unicode case folding maps onto the ASCII letter. ASCII
/// folding compares every non-ASCII character as itself, so this refutes the guard, and a target
/// that folds Unicode — `strings.EqualFold`, `toLowerCase` — accepts it and fails the scenario.
/// `None` for a literal with no such character.
fn unicode_refuting(literal: &str) -> Option<String> {
    let other_case = |character: char| {
        let mapped: Vec<char> = if character.is_lowercase() {
            character.to_uppercase().collect()
        } else if character.is_uppercase() {
            character.to_lowercase().collect()
        } else {
            return None;
        };
        match mapped.as_slice() {
            &[single] if single != character => Some(single),
            _ => None,
        }
    };
    let mut changed = false;
    let swapped: String = literal
        .chars()
        .map(|character| {
            if character.is_ascii_uppercase() {
                return character.to_ascii_lowercase();
            }
            if character.is_ascii() {
                return character.to_ascii_uppercase();
            }
            match other_case(character) {
                Some(single) => {
                    changed = true;
                    single
                }
                None => character,
            }
        })
        .collect();
    if changed {
        return Some(swapped);
    }
    let at = literal.find(['k', 'K', 's', 'S'])?;
    let replaced = literal[at..].chars().next()?;
    let sign = if replaced.eq_ignore_ascii_case(&'k') {
        '\u{212A}'
    } else {
        '\u{017F}'
    };
    Some(format!("{}{sign}{}", &literal[..at], &literal[at + 1..]))
}

/// Every text literal a case-insensitive operator compares `path` with, in written order.
fn fold_literals_at(guards: &[&Predicate], path: &FactPath) -> Vec<String> {
    fn walk(predicate: &Predicate, path: &FactPath, found: &mut Vec<String>) {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    walk(child, path, found);
                }
            }
            Predicate::Not(inner) => walk(inner, path, found),
            Predicate::FoldMatch {
                path: read, values, ..
            } if read == path => {
                found.extend(
                    values
                        .iter()
                        .filter_map(FactValue::as_text)
                        .map(str::to_owned),
                );
            }
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                walk(&quantified.body, path, found);
            }
            _ => {}
        }
    }
    let mut found = Vec::new();
    for guard in guards {
        walk(guard, path, &mut found);
    }
    found
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
    let mut map_lengths: BTreeMap<FactPath, Vec<usize>> = BTreeMap::new();
    for (parent, lengths) in byte_lengths(guards) {
        if builder.strings.contains(&parent) {
            text_bytes_ladder(builder, &parent, &lengths, ladders);
        }
    }
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
        } else if builder.maps.contains(&parent) {
            map_lengths
                .entry(parent)
                .or_default()
                .extend(count_lengths(guards, &path, false));
        }
    }
    invariant_ladders(builder, ladders);
    // A map's base holds one entry, so the empty map is its first alternative; only a map whose
    // count a guard reads is varied at all (beyond10x/ess#196).
    for (path, lengths) in map_lengths {
        let mut ladder = vec![Choice::Elements(0)];
        for held in lengths {
            let choice = Choice::Elements(held);
            if held != 1 && !ladder.contains(&choice) {
                ladder.push(choice);
            }
        }
        ladders.insert(path, ladder);
    }
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

/// The byte lengths each text a guard measures with `{utf8_bytes: <parent>}` is tried at
/// (`docs/design/expression-family-source22.md`, "String `.utf8_bytes`"): `.count`'s rule in bytes —
/// `⌊v⌋`, `⌊v⌋ + 1` and `⌊v⌋ − 1` for each numeric literal `v` it is compared with, negatives,
/// repeats and lengths past [`MAX_COUNT_WITNESS`] dropped — and either side of [`BASE_NUMBER`] where
/// it is compared only with another fact, whose own ladder decides the rest. Keyed by the parent.
fn byte_lengths(guards: &[&Predicate]) -> BTreeMap<FactPath, Vec<usize>> {
    fn walk(predicate: &Predicate, found: &mut BTreeMap<FactPath, Vec<f64>>) {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    walk(child, found);
                }
            }
            Predicate::Not(inner) => walk(inner, found),
            Predicate::Compare { left, right, .. } => {
                for (operand, other) in [(left, right), (right, left)] {
                    let Operand::Derived(derived) = operand else {
                        continue;
                    };
                    let values = found.entry(derived.parent().clone()).or_default();
                    match other {
                        Operand::Literal(value) => {
                            values.extend(value.as_number().map(Number::get));
                        }
                        Operand::Fact(_) | Operand::Derived(_) | Operand::Offset(_) => {
                            values.push(BASE_NUMBER);
                        }
                    }
                }
            }
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                walk(&quantified.body, found);
            }
            _ => {}
        }
    }
    let mut found = BTreeMap::new();
    for guard in guards {
        walk(guard, &mut found);
    }
    found
        .into_iter()
        .map(|(parent, values)| {
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
            (parent, lengths)
        })
        .collect()
}

/// The text at `path` resized to each of `lengths` UTF-8 bytes, appended to the ladder already
/// there; [`text_length_ladder`]'s rule, in bytes.
///
/// Each length is tried first as text whose bytes outnumber its scalar values and its UTF-16 code
/// units — U+1F600 (four bytes, one scalar, two units) where it fits, `é` (two bytes, one scalar)
/// where only that does — and then in the text's own characters, so a target that counts anything
/// but bytes decides a witnessed branch differently, and an alphabet that admits neither still has
/// the text's own characters, which [`admitted_inputs`] keeps.
fn text_bytes_ladder(
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
            for text in resize_bytes(source, length, plain) {
                let node = Node::Text(text);
                if node != Node::Text(base.clone())
                    && !ladder.contains(&Choice::Value(node.clone()))
                {
                    ladder.push(Choice::Value(node));
                }
            }
        }
    }
    if ladder.is_empty() {
        ladders.remove(path);
    }
}

/// `text` at exactly `length` UTF-8 bytes, never cutting a scalar: first led by one wide scalar
/// (U+1F600 where four bytes fit, else `é` where two do), then in `text`'s own characters — or
/// `plain`'s where it has none — cycled from its start, closed with the narrowest of them that
/// still fits. Where no character fits the last bytes the variant is not built.
fn resize_bytes(text: &str, length: usize, plain: &str) -> Vec<String> {
    let own: Vec<char> = if text.is_empty() {
        plain.chars().collect()
    } else {
        text.chars().collect()
    };
    let narrowest = own
        .iter()
        .copied()
        .min_by_key(|character| character.len_utf8())
        .unwrap_or('a');
    let fill = |mut out: String| -> Option<String> {
        let mut cycle = own.iter().copied().cycle();
        while out.len() < length && !own.is_empty() {
            let next = cycle.next().unwrap_or(narrowest);
            let wanted = length - out.len();
            if next.len_utf8() <= wanted {
                out.push(next);
            } else if narrowest.len_utf8() <= wanted {
                out.push(narrowest);
            } else {
                return None;
            }
        }
        while out.len() < length {
            out.push('a');
        }
        Some(out)
    };
    let mut found = Vec::new();
    let lead = if length >= 4 {
        Some('\u{1F600}')
    } else if length >= 2 {
        Some('\u{e9}')
    } else {
        None
    };
    if let Some(wide) = lead {
        found.extend(fill(wide.to_string()));
    }
    if let Some(own_text) = fill(String::new()) {
        if !found.contains(&own_text) {
            found.push(own_text);
        }
    }
    // Packed with the widest scalars that fit, so a text is short in scalars and long in bytes at
    // once: what a guard reading `.count` and `.utf8_bytes` of one text needs to see them disagree
    // (eight bytes in two scalars, `😀😀`).
    if length >= 2 {
        let mut packed = "\u{1F600}".repeat(length / 4);
        packed.push_str(match length % 4 {
            3 => "\u{20ac}",
            2 => "\u{e9}",
            1 => "a",
            _ => "",
        });
        if !found.contains(&packed) {
            found.push(packed);
        }
    }
    found
}

/// The witnesses tried for two byte lengths compared with each other, as `(left, right)`: one side
/// wide and the other narrow at the lengths that decide every operator — equal in bytes and not in
/// scalars (`é` and `ab`), and apart in bytes and equal or reversed in scalars (`😀` against `a` and
/// against `abc`) — so a target counting anything but bytes decides one of them differently.
const WIDE_NARROW: [(&str, usize); 3] = [("\u{e9}", 2), ("\u{1F600}", 1), ("\u{1F600}", 3)];

/// Every pair of texts the guards compare by their UTF-8 byte lengths, `{utf8_bytes: p} <op>
/// {utf8_bytes: q}`, in the order they are written.
fn byte_length_pairs(guards: &[&Predicate]) -> Vec<(FactPath, FactPath)> {
    fn walk(predicate: &Predicate, found: &mut Vec<(FactPath, FactPath)>) {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    walk(child, found);
                }
            }
            Predicate::Not(inner) => walk(inner, found),
            Predicate::Compare {
                left: Operand::Derived(left),
                right: Operand::Derived(right),
                ..
            } if left.parent() != right.parent() => {
                let pair = (left.parent().clone(), right.parent().clone());
                if !found.contains(&pair) {
                    found.push(pair);
                }
            }
            _ => {}
        }
    }
    let mut found = Vec::new();
    for guard in guards {
        walk(guard, &mut found);
    }
    found
}

/// The overrides [`WIDE_NARROW`] spells for each pair of [`byte_length_pairs`], each way round: the
/// narrow side in the text's own characters where they are single bytes.
fn wide_narrow_pairs(
    builder: &Builder<'_>,
    guards: &[&Predicate],
) -> Vec<BTreeMap<FactPath, Choice>> {
    let narrow = |path: &FactPath, length: usize| {
        let Some((Leaf::Text, Node::Text(base))) = builder.leaves.get(path) else {
            return None;
        };
        let plain = builder
            .plain_texts
            .get(path)
            .map_or(base.as_str(), String::as_str);
        resize_bytes(base, length, plain)
            .into_iter()
            .find(|text| text.is_ascii())
    };
    let mut found = Vec::new();
    for (left, right) in byte_length_pairs(guards) {
        if !builder.strings.contains(&left) || !builder.strings.contains(&right) {
            continue;
        }
        for (wide, length) in WIDE_NARROW {
            for (wide_path, narrow_path) in [(&left, &right), (&right, &left)] {
                let Some(narrow_text) = narrow(narrow_path, length) else {
                    continue;
                };
                found.push(BTreeMap::from([
                    (
                        wide_path.clone(),
                        Choice::Value(Node::Text(wide.to_owned())),
                    ),
                    (narrow_path.clone(), Choice::Value(Node::Text(narrow_text))),
                ]));
            }
        }
    }
    found
}

/// `inputs` led by one input per [`wide_narrow_pairs`] pair, then one per [`wide_texts`] text, each
/// the base with those texts changed.
fn wide_first(
    builder: &mut Builder<'_>,
    command: &ResolvedCommand,
    guards: &[&Predicate],
    inputs: &mut Vec<BTreeMap<String, Node>>,
) -> Result<(), WitnessGap> {
    let mut wide = Vec::new();
    let mut overrides = wide_narrow_pairs(builder, guards);
    overrides.extend(
        wide_texts(builder, guards)
            .into_iter()
            .map(|(path, text)| BTreeMap::from([(path, Choice::Value(Node::Text(text)))])),
    );
    for changed in overrides {
        let input = builder.input(command, &changed)?;
        if !wide.contains(&input) {
            wide.push(input);
        }
    }
    if !wide.is_empty() {
        inputs.retain(|input| !wide.contains(input));
        wide.append(inputs);
        *inputs = wide;
        inputs.truncate(MAX_CANDIDATES);
    }
    Ok(())
}

/// For each text the guards measure in UTF-8 bytes and each length they decide it at, the text at
/// that length led by a wide scalar ([`resize_bytes`]'s first variant), where it has one.
fn wide_texts(builder: &Builder<'_>, guards: &[&Predicate]) -> Vec<(FactPath, String)> {
    let mut found = Vec::new();
    for (parent, lengths) in byte_lengths(guards) {
        let Some((Leaf::Text, Node::Text(base))) = builder.leaves.get(&parent) else {
            continue;
        };
        if !builder.strings.contains(&parent) {
            continue;
        }
        let plain = builder
            .plain_texts
            .get(&parent)
            .map_or(base.as_str(), String::as_str);
        for length in lengths {
            if let Some(text) = resize_bytes(base, length, plain)
                .into_iter()
                .find(|text| !text.is_ascii())
            {
                found.push((parent.clone(), text));
            }
        }
    }
    found
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

/// The effective prefix of the newtype chain `type_ref` is declared through: the longest any layer
/// declares, which `ess-domain` holds every other layer's prefix to be a prefix of.
fn chain_prefix(ir: &EssIr, type_ref: &ResolvedTypeRef) -> Option<String> {
    let mut longest: Option<&str> = None;
    let mut current = type_ref;
    for _ in 0..=MAX_TYPE_DEPTH {
        match current {
            ResolvedTypeRef::Optional { of } => current = of,
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, prefix, .. } => {
                    if let Some(prefix) = prefix.as_deref() {
                        if longest.is_none_or(|held| prefix.chars().count() > held.chars().count())
                        {
                            longest = Some(prefix);
                        }
                    }
                    current = of;
                }
                _ => break,
            },
            _ => break,
        }
    }
    longest.map(str::to_owned)
}

/// `text` as a value of a type whose values start with `prefix` (rule 2 under a prefix): itself
/// where it already starts with it, otherwise the prefix followed by it, so the path's own text
/// still tells two fields apart.
fn with_prefix(text: &str, prefix: &str) -> String {
    if text.starts_with(prefix) {
        text.to_owned()
    } else {
        format!("{prefix}{text}")
    }
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
            // And for one over `{utf8_bytes: value}`, in bytes.
            if let Some(lengths) = byte_lengths(&declared).get(&value) {
                text_bytes_ladder(builder, path, lengths, ladders);
            }
        }
    }
}

/// `read` moved under `prefix`: `start` under `window` is `window.start`.
fn joined(prefix: &FactPath, read: &FactPath) -> FactPath {
    let mut moved = prefix.clone();
    for segment in read.segments() {
        moved = moved.child(segment);
    }
    moved
}

/// Every pair of facts a guard compares with each other, with the operator, in the order the
/// guards write them.
fn fact_comparisons(guards: &[&Predicate]) -> Vec<(FactPath, CompareOp, FactPath)> {
    fn walk(predicate: &Predicate, found: &mut Vec<(FactPath, CompareOp, FactPath)>) {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    walk(child, found);
                }
            }
            Predicate::Not(inner) => walk(inner, found),
            Predicate::Compare {
                left: Operand::Fact(left),
                op,
                right: Operand::Fact(right),
                ..
            } => {
                let triple = (left.clone(), *op, right.clone());
                if !found.contains(&triple) {
                    found.push(triple);
                }
            }
            _ => {}
        }
    }
    let mut found = Vec::new();
    for guard in guards {
        walk(guard, &mut found);
    }
    found
}

/// For each comparison between two leaves of the input — `owner == ticket.owner` — each side tried
/// at the other's base as well (beyond10x/ess#234); with `ordered`, for an order between two numbers
/// — `low < high` — also at the other's base plus and minus one.
///
/// Rule 2 gives two fields two different texts, so no literal of the guard and no value either
/// side is otherwise tried at makes them equal. The other side's base does. An order is met by the
/// neighbours of the other side's base, which move with the instance as that base does (rule 5), so
/// two instances do not share the value that met it. Every leaf the comparison does not read keeps
/// its own witness. `ordered` is the invariants' ([`repair`]); a guard's order between two facts
/// keeps the ladder it had.
fn equality_copies(
    builder: &Builder<'_>,
    guards: &[&Predicate],
    ordered: bool,
    ladders: &mut BTreeMap<FactPath, Vec<Choice>>,
) {
    for (left, op, right) in fact_comparisons(guards) {
        let equality = matches!(op, CompareOp::Eq | CompareOp::Ne);
        if !equality && !ordered {
            continue;
        }
        for (to, from) in [(&left, &right), (&right, &left)] {
            let (Some((to_leaf, to_base)), Some((from_leaf, from_base))) =
                (builder.leaves.get(to), builder.leaves.get(from))
            else {
                continue;
            };
            if std::mem::discriminant(to_leaf) != std::mem::discriminant(from_leaf) {
                continue;
            }
            let mut values = vec![from_base.clone()];
            if let (false, Leaf::Number { .. }, Node::Number(number)) =
                (equality, to_leaf, from_base)
            {
                values.extend(
                    [number.get() + 1.0, number.get() - 1.0]
                        .into_iter()
                        .filter_map(|value| Number::new(value).ok())
                        .map(Node::Number),
                );
            }
            let ladder = ladders.entry(to.clone()).or_default();
            for value in values {
                let choice = Choice::Value(value);
                if choice != Choice::Value(to_base.clone()) && !ladder.contains(&choice) {
                    ladder.push(choice);
                }
            }
            if ladder.is_empty() {
                ladders.remove(to);
            }
        }
    }
}

/// Every comparison of a fact with one constant offset of another, in the order the guards write
/// them (`docs/design/expression-family-source22.md`, A2).
fn offset_comparisons(
    guards: &[&Predicate],
) -> Vec<(FactPath, ess_primitives::predicate::OffsetOperand)> {
    fn walk(
        predicate: &Predicate,
        found: &mut Vec<(FactPath, ess_primitives::predicate::OffsetOperand)>,
    ) {
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    walk(child, found);
                }
            }
            Predicate::Not(inner) => walk(inner, found),
            Predicate::Compare {
                left: Operand::Fact(left),
                right: Operand::Offset(offset),
                ..
            } => {
                let pair = (left.clone(), offset.clone());
                if !found.contains(&pair) {
                    found.push(pair);
                }
            }
            _ => {}
        }
    }
    let mut found = Vec::new();
    for guard in guards {
        walk(guard, &mut found);
    }
    found
}

/// For each comparison `left <op> base ± k` between two leaves of the input, each side tried at the
/// boundary the other side's base makes and one unit either side of it
/// (`docs/design/expression-family-source22.md`, A2): `left` at `base ± k`, `base` at `left ∓ k`.
/// Synthesis composes the two sides' candidates rather than solving an equation; an `Integer` moves
/// by one, a `Timestamp` by one second, and a value its type cannot hold — past `i64`, past what a
/// `date-time` spells — is not tried. A command with no offset keeps its ladders.
fn offset_copies(
    builder: &Builder<'_>,
    guards: &[&Predicate],
    ladders: &mut BTreeMap<FactPath, Vec<Choice>>,
) {
    use ess_primitives::predicate::{OffsetMagnitude, OffsetOperand};
    // `from ± k` and one unit either side, as the nodes a leaf of `leaf` holds.
    let boundary = |offset: &OffsetOperand, leaf: &Leaf, from: &Node| -> Vec<Node> {
        match (offset.magnitude, leaf, from) {
            (OffsetMagnitude::Integer(_), Leaf::Number { .. }, Node::Number(number)) => {
                let Some(bound) = number.as_i64().and_then(|value| offset.integer_at(value)) else {
                    return Vec::new();
                };
                [bound, bound - 1, bound + 1]
                    .into_iter()
                    .filter_map(|value| i64::try_from(value).ok())
                    .map(|value| Node::Number(Number::from(value)))
                    .collect()
            }
            (OffsetMagnitude::ElapsedSeconds { .. }, Leaf::Timestamp, Node::Text(text)) => {
                let Some(bound) =
                    Rfc3339Instant::parse_rfc3339(text).and_then(|value| offset.instant_at(value))
                else {
                    return Vec::new();
                };
                [0, -1, 1]
                    .into_iter()
                    .filter_map(|step| bound.plus_elapsed(step))
                    .map(|value| Node::Text(value.to_rfc3339()))
                    .collect()
            }
            _ => Vec::new(),
        }
    };
    for (left, offset) in offset_comparisons(guards) {
        let (Some((left_leaf, left_base)), Some((base_leaf, base_base))) =
            (builder.leaves.get(&left), builder.leaves.get(&offset.base))
        else {
            continue;
        };
        // The base read back from the left: the same offset the other way.
        let inverse = offset.reversed(left.clone());
        for (to, to_leaf, to_base, values) in [
            (
                &left,
                left_leaf,
                left_base,
                boundary(&offset, left_leaf, base_base),
            ),
            (
                &offset.base,
                base_leaf,
                base_base,
                boundary(&inverse, base_leaf, left_base),
            ),
        ] {
            // Tried first: the boundary is what decides the comparison, so the first candidate
            // that takes a branch the offset guards takes it at the boundary, not a day away.
            let ladder = ladders.entry(to.clone()).or_default();
            let mut first: Vec<Choice> = Vec::new();
            for value in values {
                if matches!(to_leaf, Leaf::Number { integral: false }) {
                    continue;
                }
                let choice = Choice::Value(value);
                if choice != Choice::Value(to_base.clone()) && !first.contains(&choice) {
                    first.push(choice);
                }
            }
            ladder.retain(|choice| !first.contains(choice));
            first.append(ladder);
            *ladder = first;
            if ladder.is_empty() {
                ladders.remove(to);
            }
        }
    }
}

/// One entity invariant a branch holds its input to ([`outcome_constraints`]).
#[derive(Debug, Clone, PartialEq)]
struct Held {
    /// The invariant with every read moved to the input path the branch copies into it.
    predicate: Predicate,
    /// The entity that declares it.
    entity: String,
    /// The invariant as the author wrote it.
    statement: String,
}

impl Held {
    /// How a refusal names it.
    fn describe(&self) -> String {
        format!("`{}` invariant `{}`", self.entity, self.statement)
    }
}

/// The entity invariants every branch of `command` holds its input to, by the branch's index, each
/// read moved from the stored field to the input path the branch copies into it
/// (beyond10x/ess#234).
///
/// `sets: {fingerprint: input.fingerprint}` beside `fingerprint.version == "canonical-v1"` reads
/// `fingerprint.version == "canonical-v1"` of the input. Only an invariant every read of which lands
/// on a copied input is one: a read of a field the branch leaves alone, sets from a literal or
/// generates is not the input's to satisfy, and the invariant is left out rather than half-read.
fn outcome_constraints(ir: &EssIr, command: &ResolvedCommand) -> Vec<Vec<Held>> {
    use ess_compiler::ir::{ResolvedPayloadField, ResolvedPayloadValue};
    fn copies(
        fields: &[ResolvedPayloadField],
        at: Option<&FactPath>,
        found: &mut Vec<(FactPath, FactPath)>,
    ) {
        for field in fields {
            let Ok(target) = FactPath::new(&field.target) else {
                continue;
            };
            let stored = at.map_or_else(|| target.clone(), |at| joined(at, &target));
            match &field.value {
                ResolvedPayloadValue::InputField { field, .. }
                | ResolvedPayloadValue::InputOrGenerated { field, .. } => {
                    if let Ok(input) = FactPath::new(field) {
                        found.push((stored, input));
                    }
                }
                ResolvedPayloadValue::Struct { fields } => copies(fields, Some(&stored), found),
                _ => {}
            }
        }
    }
    let mut constraints = Vec::new();
    for outcome in &command.outcomes {
        let mut held: Vec<Held> = Vec::new();
        let mut written: Vec<(&ess_compiler::ir::EntityHandle, &[ResolvedPayloadField])> =
            Vec::new();
        if let Some(subject) = &outcome.subject {
            written.push((&subject.entity, &outcome.sets));
        }
        for affect in &outcome.affects {
            written.push((&affect.entity, &affect.sets));
        }
        for (entity, sets) in written {
            let mut copied = Vec::new();
            copies(sets, None, &mut copied);
            if copied.is_empty() {
                continue;
            }
            let declared = ir.entity(entity);
            for invariant in &declared.invariants {
                let unmapped = std::cell::Cell::new(false);
                let moved = remapped(
                    &invariant.predicate,
                    &|read: &FactPath| {
                        let found = copied.iter().find_map(|(stored, input)| {
                            read.segments().strip_prefix(stored.segments()).map(|rest| {
                                if rest.is_empty() {
                                    input.clone()
                                } else {
                                    joined(input, &FactPath::from_segments(rest))
                                }
                            })
                        });
                        if found.is_none() {
                            unmapped.set(true);
                        }
                        found
                    },
                    &[],
                );
                if !unmapped.get() && !held.iter().any(|known| known.predicate == moved) {
                    held.push(Held {
                        predicate: moved,
                        entity: declared.name.to_string(),
                        statement: invariant.statement.clone(),
                    });
                }
            }
        }
        constraints.push(held);
    }
    constraints
}

/// Every invariant [`outcome_constraints`] holds any branch to, once, in branch order.
fn entity_constraints(constrained: &[Vec<Held>]) -> Vec<Predicate> {
    let mut all: Vec<Predicate> = Vec::new();
    for held in constrained.iter().flatten() {
        if !all.contains(&held.predicate) {
            all.push(held.predicate.clone());
        }
    }
    all
}

/// The part of `outcome`'s condition that reads the input alone, where it has one.
fn input_guard(outcome: &ess_compiler::ir::ResolvedOutcome) -> Option<&Predicate> {
    use ess_compiler::ir::ResolvedCondition;
    match &outcome.condition {
        ResolvedCondition::When { predicate }
        | ResolvedCondition::ExternalWhen { predicate, .. } => Some(predicate),
        ResolvedCondition::SubjectField { predicate, .. }
        | ResolvedCondition::SubjectState { predicate, .. }
        | ResolvedCondition::StateChange { predicate, .. } => predicate.as_ref(),
        ResolvedCondition::SubjectPredicate { input, .. }
        | ResolvedCondition::Related { input, .. } => input.as_ref(),
        _ => None,
    }
}

/// The branches of `command`, by index, that can answer an input with these facts, whatever row
/// or provider the scenario arranges: every branch whose input guard the facts do not refute, up
/// to and including the first accepting `when:` they satisfy, and none where an input-guarded
/// refusal they satisfy answers first. Over-approximate on purpose: a branch left in costs a
/// candidate, a branch left out could send an input that branch's entity refuses.
fn reachable(command: &ResolvedCommand, facts: &crate::InputFacts<'_>) -> Vec<usize> {
    use ess_compiler::ir::ResolvedCondition;
    let refused = command.outcomes.iter().any(|outcome| {
        outcome.error.is_some()
            && matches!(&outcome.condition, ResolvedCondition::When { predicate }
                if matches!(facts.decide(predicate), Decision::Satisfied))
    });
    if refused {
        return Vec::new();
    }
    let mut reached = Vec::new();
    for (index, outcome) in command.outcomes.iter().enumerate() {
        let decided = input_guard(outcome).map(|guard| facts.decide(guard));
        if matches!(decided, Some(Decision::Refuted(_))) {
            continue;
        }
        reached.push(index);
        if outcome.error.is_none()
            && matches!(outcome.condition, ResolvedCondition::When { .. })
            && matches!(decided, Some(Decision::Satisfied))
        {
            break;
        }
    }
    reached
}

/// The first entity invariant of a branch `input` can reach that `input` refutes, or `None`. An
/// input that does not flatten is [`admitted_inputs`]'s to refuse.
fn broken<'h>(
    ir: &EssIr,
    command: &ResolvedCommand,
    constrained: &'h [Vec<Held>],
    input: &BTreeMap<String, Node>,
) -> Option<&'h Held> {
    let facts = crate::input::flatten(ir, command, input).ok()?;
    reachable(command, &facts).into_iter().find_map(|index| {
        constrained[index]
            .iter()
            .find(|held| matches!(facts.decide(&held.predicate), Decision::Refuted(_)))
    })
}

/// The value `input` holds at `path`, reading a list's element by its position.
fn node_at<'n>(input: &'n BTreeMap<String, Node>, path: &FactPath) -> Option<&'n Node> {
    let (first, rest) = path.segments().split_first()?;
    let mut node = input.get(first)?;
    for segment in rest {
        node = match node {
            Node::Map(members) => members.get(segment)?,
            Node::Seq(items) => items.get(segment.parse::<usize>().ok()?)?,
            _ => return None,
        };
    }
    Some(node)
}

/// `inputs`, each held to the entity invariants of every branch it can reach ([`reachable`],
/// beyond10x/ess#234).
///
/// A guard's ladder moves one leaf, and an invariant may tie it to another (`origin == route`
/// beside `origin == "eu"`, `low < high` beside `low > 10`); two branches may copy one input into
/// two entities whose invariants disagree. A candidate that breaks an invariant of a branch it can
/// reach is solved again for those invariants with every leaf it moved off the base kept where it
/// is ([`resolved`]), so the partner follows; where that finds nothing it is dropped, never sent.
/// A command no branch of which copies its input into an entity with such an invariant keeps its
/// candidates unchanged.
fn within_invariants(
    builder: &mut Builder<'_>,
    command: &ResolvedCommand,
    constrained: &[Vec<Held>],
    inputs: Vec<BTreeMap<String, Node>>,
) -> Result<Vec<BTreeMap<String, Node>>, WitnessGap> {
    if constrained.iter().all(Vec::is_empty) {
        return Ok(inputs);
    }
    let ir = builder.ir;
    let mut kept: Vec<BTreeMap<String, Node>> = Vec::new();
    for input in inputs {
        let input = if broken(ir, command, constrained, &input).is_none() {
            input
        } else {
            match resolved(builder, command, constrained, &input)? {
                Some(solved)
                    if broken(ir, command, constrained, &solved).is_none()
                        && !admitted_inputs(ir, command, vec![solved.clone()]).is_empty() =>
                {
                    solved
                }
                _ => continue,
            }
        };
        if !kept.contains(&input) {
            kept.push(input);
        }
    }
    Ok(kept)
}

/// `input` solved again for the invariants of the structs it holds and of every branch it can
/// reach, with each leaf it holds off the base pinned where it is: `None` where it holds a shape
/// only a guard's choice builds (a length, an omission), or no bounded candidate satisfies them.
/// The base itself is solved too: [`repair`] may have met only the struct invariants, where the
/// branches' entities disagree. The builder's base is left as it was.
fn resolved(
    builder: &mut Builder<'_>,
    command: &ResolvedCommand,
    constrained: &[Vec<Held>],
    input: &BTreeMap<String, Node>,
) -> Result<Option<BTreeMap<String, Node>>, WitnessGap> {
    let Ok(facts) = crate::input::flatten(builder.ir, command, input) else {
        return Ok(None);
    };
    let mut owed = builder.struct_invariants.clone();
    for index in reachable(command, &facts) {
        for held in &constrained[index] {
            if !owed.contains(&held.predicate) {
                owed.push(held.predicate.clone());
            }
        }
    }
    let pins: BTreeMap<FactPath, Choice> = builder
        .leaves
        .iter()
        .filter_map(|(path, (_, base))| {
            node_at(input, path)
                .filter(|held| *held != base)
                .map(|held| (path.clone(), Choice::Value(held.clone())))
        })
        .collect();
    let frozen: BTreeSet<FactPath> = pins.keys().cloned().collect();
    let saved = builder.fixed.clone();
    let unrepaired = builder.unrepaired.clone();
    builder.fixed.extend(pins);
    let rebuilt = builder.input(command, &BTreeMap::new())?;
    let found = if rebuilt == *input {
        match solve(builder, command, &owed.iter().collect::<Vec<_>>(), &frozen)?.0 {
            Some(overrides) => Some(builder.input(command, &overrides)?),
            None => None,
        }
    } else {
        None
    };
    builder.fixed = saved;
    builder.unrepaired = unrepaired;
    builder.input(command, &BTreeMap::new())?;
    Ok(found)
}

/// [`solve`], and where it finds nothing and the invariants count a list no guard expanded
/// (`parts.count == 2`), again on a builder that expands it — which becomes `builder` where it
/// finds a solution, so the list's length is a choice the base can carry. Also answers how many
/// candidates were tried.
fn widened_solve(
    builder: &mut Builder<'_>,
    command: &ResolvedCommand,
    constraints: &[Predicate],
) -> Result<(Option<BTreeMap<FactPath, Choice>>, usize), WitnessGap> {
    let refs: Vec<&Predicate> = constraints.iter().collect();
    let (found, tried) = solve(builder, command, &refs, &BTreeSet::new())?;
    if found.is_some() {
        return Ok((found, tried));
    }
    let lists = list_reads(&refs);
    if lists.is_subset(&builder.expand) {
        return Ok((None, tried));
    }
    let mut expand = builder.expand.clone();
    expand.extend(lists);
    let mut wider = Builder::new(
        builder.ir,
        builder.distinction,
        expand,
        builder.positional.clone(),
    );
    wider.input(command, &BTreeMap::new())?;
    let (found, more) = solve(&mut wider, command, &refs, &BTreeSet::new())?;
    if found.is_some() {
        *builder = wider;
    }
    Ok((found, tried + more))
}

/// Solves the base witness for the invariants over the input: those of every struct it holds, and
/// those of every entity a branch copies it into ([`outcome_constraints`]) — beyond10x/ess#234.
///
/// Rule 2's base is a value of each leaf's own type, and a struct's or an entity's invariant over
/// two leaves (`zone == "utc"`, `origin == route`, `low < high`) is refused by it: every candidate
/// built on it is dropped by [`admitted_inputs`], or creates a row the model's own invariant
/// refuses. Where the base already satisfies them, or no invariant reads the input, nothing moves
/// and every suite keeps its bytes. Otherwise the leaves the invariants read are tried at the
/// invariants' own literals, one either side, and at each other's base for an equality, in the
/// bounded order rule 4 walks; the first candidate the invariants and the declared types admit
/// becomes the base of every candidate. A leaf a guard reads is still varied from there.
///
/// Where the bounded search satisfies the struct invariants but not every branch's entity
/// invariants together — two branches may copy one input into entities that disagree — the struct
/// invariants alone are solved for, and [`within_invariants`] solves each candidate for the
/// branches it reaches. A candidate reaching a branch whose invariants nothing satisfies is
/// dropped, and synthesis refuses that branch naming them ([`unmet_invariants`]).
fn repair(
    builder: &mut Builder<'_>,
    command: &ResolvedCommand,
    constrained: &[Vec<Held>],
) -> Result<(), WitnessGap> {
    let structs = builder.struct_invariants.clone();
    let mut all = structs.clone();
    for constraint in entity_constraints(constrained) {
        if !all.contains(&constraint) {
            all.push(constraint);
        }
    }
    for constraints in [all, structs] {
        if constraints.is_empty() {
            return Ok(());
        }
        if let (Some(fixed), _) = widened_solve(builder, command, &constraints)? {
            if !fixed.is_empty() {
                builder.fixed = fixed;
                // Recorded again, so every ladder is built from the repaired base.
                builder.input(command, &BTreeMap::new())?;
            }
            return Ok(());
        }
    }
    Ok(())
}

/// The entity invariants `outcome` holds the input of `command` to that no bounded candidate
/// satisfies together with the invariants of the structs the input holds, named, with how many
/// candidates were tried; `None` where they are met, or `outcome` copies no input into an entity
/// with an invariant over it (beyond10x/ess#234).
///
/// Every candidate reaching such a branch is dropped ([`within_invariants`]), so the branch has
/// no input to send; this names why, so synthesis refuses it rather than reporting a guard.
///
/// With `guarded`, the branch's own input guard is solved for as well, and the invariants are
/// named only where the guard alone is met and the guard with them is not: a guard that
/// contradicts the entity it creates (`weight < 0` beside `weight >= 0`) sends nothing either.
///
/// # Errors
///
/// [`WitnessGap`] when some field of the input has no safe value at all.
pub(crate) fn unmet_invariants(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ess_compiler::ir::ResolvedOutcome,
    guarded: bool,
) -> Result<Option<(String, usize)>, WitnessGap> {
    let Some(index) = command
        .outcomes
        .iter()
        .position(|branch| branch.name == outcome.name)
    else {
        return Ok(None);
    };
    let constrained = outcome_constraints(ir, command);
    let owed = &constrained[index];
    if owed.is_empty() {
        return Ok(None);
    }
    let solves = |extra: &[Predicate]| -> Result<(bool, usize), WitnessGap> {
        let mut builder = Builder::new(ir, Distinction::PLAIN, BTreeSet::new(), BTreeMap::new());
        builder.input(command, &BTreeMap::new())?;
        let mut constraints = builder.struct_invariants.clone();
        for predicate in extra {
            if !constraints.contains(predicate) {
                constraints.push(predicate.clone());
            }
        }
        let (found, tried) = widened_solve(&mut builder, command, &constraints)?;
        Ok((found.is_some(), tried))
    };
    let mut named: Vec<String> = owed.iter().map(Held::describe).collect();
    let mut invariants: Vec<Predicate> = owed.iter().map(|held| held.predicate.clone()).collect();
    if guarded {
        let Some(guard) = input_guard(outcome) else {
            return Ok(None);
        };
        if !solves(std::slice::from_ref(guard))?.0 {
            return Ok(None);
        }
        invariants.push(guard.clone());
        named.push(format!("the branch's guard `{guard}`"));
    }
    let (found, tried) = solves(&invariants)?;
    Ok((!found).then(|| (named.join(" and "), tried)))
}

/// The first entity invariant of `outcome` that `input` refutes, named ([`Held::describe`]), or
/// `None` (beyond10x/ess#234): the last check before an input is sent for a branch that copies it
/// into an entity.
pub(crate) fn invariant_broken_by(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ess_compiler::ir::ResolvedOutcome,
    input: &BTreeMap<String, Node>,
) -> Option<String> {
    let index = command
        .outcomes
        .iter()
        .position(|branch| branch.name == outcome.name)?;
    let constrained = outcome_constraints(ir, command);
    let owed = &constrained[index];
    if owed.is_empty() {
        return None;
    }
    let facts = crate::input::flatten(ir, command, input).ok()?;
    owed.iter()
        .find(|held| matches!(facts.decide(&held.predicate), Decision::Refuted(_)))
        .map(Held::describe)
}

/// The choices that make the base satisfy `constraints` ([`repair`]), with every leaf in `frozen`
/// kept at its base: empty where the base already does, `None` where no candidate of the bounded
/// walk does; and how many candidates were tried.
///
/// A list's length is a choice here only where an invariant counts the list; an omission never
/// is. At a further instance each numeric literal is tried moved by the instance's ordinal first,
/// so an invariant that bounds a leaf without pinning it (`start >= 5`) leaves two instances apart
/// (rule 5).
fn solve(
    builder: &mut Builder<'_>,
    command: &ResolvedCommand,
    constraints: &[&Predicate],
    frozen: &BTreeSet<FactPath>,
) -> Result<(Option<BTreeMap<FactPath, Choice>>, usize), WitnessGap> {
    let ir = builder.ir;
    let holds = |input: &BTreeMap<String, Node>| {
        !admitted_inputs(ir, command, vec![input.clone()]).is_empty()
            && crate::input::flatten(ir, command, input).is_ok_and(|facts| {
                constraints
                    .iter()
                    .all(|constraint| !matches!(facts.decide(constraint), Decision::Refuted(_)))
            })
    };
    if holds(&builder.input(command, &BTreeMap::new())?) {
        return Ok((Some(BTreeMap::new()), 1));
    }
    let ordinal = f64::from(u32::try_from(builder.distinction.get()).unwrap_or(u32::MAX));
    let mut ladders: BTreeMap<FactPath, Vec<Choice>> = BTreeMap::new();
    for path in read_paths(constraints) {
        if frozen.contains(&path) {
            continue;
        }
        let Some((leaf, at_base)) = builder.leaves.get(&path) else {
            continue;
        };
        let literals = literals_at(constraints, &path);
        let mut alternatives =
            alternatives(leaf, at_base, &literals, ordered_at(constraints, &path));
        if let (Leaf::Number { integral }, true) = (leaf, ordinal > 0.0) {
            let moved: Vec<Node> = literals
                .iter()
                .filter_map(FactValue::as_number)
                .filter_map(|literal| Number::new(literal.get() + ordinal).ok())
                .filter(|moved| !*integral || moved.is_integral())
                .map(Node::Number)
                .filter(|moved| moved != at_base)
                .collect();
            for value in moved.into_iter().rev() {
                alternatives.retain(|known| known != &value);
                alternatives.insert(0, value);
            }
        }
        if let (Leaf::Text, Node::Text(base)) = (leaf, at_base) {
            let invariants = builder.invariants.get(&path).map_or(&[][..], Vec::as_slice);
            for text in text_alternatives(constraints, invariants, &path, base) {
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
    equality_copies(builder, constraints, true, &mut ladders);
    offset_copies(builder, constraints, &mut ladders);
    count_ladders(builder, constraints, &mut ladders);
    // Values, and the length of a list an invariant counts; an omission is a guard's to vary.
    let counted_lists: BTreeSet<FactPath> =
        read_paths(constraints).iter().filter_map(counted).collect();
    let ladders: Vec<(FactPath, Vec<Choice>)> = ladders
        .into_iter()
        .filter(|(path, _)| !frozen.contains(path))
        .map(|(path, ladder)| {
            let kept: Vec<Choice> = ladder
                .into_iter()
                .filter(|choice| match choice {
                    Choice::Value(_) => true,
                    Choice::Elements(_) => counted_lists.contains(&path),
                    Choice::Omit | Choice::Null | Choice::Keyed(_) => false,
                })
                .collect();
            (path, kept)
        })
        .filter(|(_, ladder)| !ladder.is_empty())
        .collect();
    let total = product(&ladders);
    let mut index = 1;
    while index < total && index < MAX_CANDIDATES.saturating_mul(MAX_ENUMERATED_PER_CANDIDATE) {
        let mut overrides = BTreeMap::new();
        let mut remaining = index;
        for (path, alternatives) in &ladders {
            let radix = alternatives.len() + 1;
            let chosen = remaining % radix;
            remaining /= radix;
            if chosen > 0 {
                overrides.insert(path.clone(), alternatives[chosen - 1].clone());
            }
        }
        if holds(&builder.input(command, &overrides)?) {
            return Ok((Some(overrides), index + 1));
        }
        index += 1;
    }
    Ok((None, index))
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
/// guard's compositions, then each newtype invariant's, then each literal's `L′`, then each
/// case-insensitive literal with one character changed.
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
    // A fold literal with one character changed: the refuting side of a case-insensitive guard.
    let members = fold_literals_at(guards, path);
    for literal in &members {
        found.extend(fold_refuting(literal, &members));
        found.extend(unicode_refuting(literal));
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
            let mut numbers: Vec<Number> = Vec::new();
            for literal in literals {
                if let Some(number) = literal.as_number() {
                    // A literal binary64 does not carry — an `Integer` beyond 2^53 a row holds
                    // (beyond10x/ess#413) — is stepped exactly; its binary64 neighbours would name
                    // another value. Every literal binary64 carries keeps its candidates.
                    if Number::new(number.get()).ok() == Some(number) {
                        numbers.extend(
                            [number.get(), number.get() + 1.0, number.get() - 1.0]
                                .into_iter()
                                .filter_map(|value| Number::new(value).ok()),
                        );
                    } else {
                        numbers.extend(
                            [
                                Some(number),
                                number.checked_add(Number::from(1_i64)),
                                number.checked_add(Number::from(-1_i64)),
                            ]
                            .into_iter()
                            .flatten(),
                        );
                    }
                }
            }
            numbers.extend(
                [0.0, -1.0]
                    .into_iter()
                    .filter_map(|value| Number::new(value).ok()),
            );
            for candidate in numbers {
                if !*integral || candidate.is_integral() {
                    push(Node::Number(candidate));
                }
            }
        }
        Leaf::Bool => push(Node::Bool(!matches!(base, Node::Bool(true)))),
        Leaf::Json => {}
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
            let (current, texts): (Vec<&str>, Vec<&str>) = literals
                .iter()
                .filter_map(FactValue::as_text)
                .partition(|text| CurrentTime::parse(text).is_some());
            for value in current_time_alternatives(&current) {
                push(value);
            }
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

/// The instants a `Timestamp` compared with the current time is tried at (beyond10x/ess#171): a
/// second either side of each boundary, against the reference instant synthesis decides such a
/// guard at ([`crate::now_offset::reference`]), and never the boundary itself.
///
/// The boundary is what a latency flips: `starts_at >= now` holds of an input sent at the moment it
/// names only if the target reads its clock at that same moment, and never does. A second inside
/// and a second outside decide the same way for any target that handles the request within a
/// second of its being sent. The operand's text itself is no instant and is never tried.
fn current_time_alternatives(literals: &[&str]) -> Vec<Node> {
    let reference = crate::now_offset::reference();
    let mut values = Vec::new();
    for literal in literals {
        let Some(boundary) = CurrentTime::parse(literal).and_then(|now| now.at(reference)) else {
            continue;
        };
        for step in [-1, 1] {
            if let Some(moved) = boundary.plus_seconds(step) {
                values.push(Node::Text(moved.to_rfc3339()));
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
#[derive(Clone)]
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
    /// Every recorded path that holds a map with one entry in its base (beyond10x/ess#196): what a
    /// `.count` guard over it varies by [`Choice::Elements`].
    maps: BTreeSet<FactPath>,
    /// The declared types being built on the way down to the current value, outermost first.
    building: Vec<String>,
    /// The input paths a `sets:` or payload value copies whole: a list at or under one holds one
    /// element in the base rather than `[]`, as a map holds one entry (beyond10x/ess#196).
    copied: BTreeSet<FactPath>,
    /// Whether the value being built is already inside the one entry a type that refers to itself
    /// through a map or a copied list is unfolded to.
    unfolded: bool,
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
    /// The effective prefix of every text a declared newtype chain constrains, by path: the
    /// longest any layer declares (ess/15). Recorded where alphabets are, for the same reason.
    prefixes: BTreeMap<FactPath, String>,
    /// Every recorded leaf declared `String`, the one text that has a length. [`Leaf::Text`] also
    /// covers `Uuid`, `Duration` and `Bytes`.
    strings: BTreeSet<FactPath>,
    /// Each `String` leaf's base from rules 2 and 3 — its own path, mapped into its alphabet —
    /// which is never empty and is what a resize of an empty text cycles.
    plain_texts: BTreeMap<FactPath, String>,
    /// The authored `example:` of each command input, used as that input's base at
    /// [`Distinction::PLAIN`] only.
    examples: BTreeMap<FactPath, Node>,
    /// The invariants of every struct a recorded value is declared as, at any depth, each read
    /// moved under the path the struct is built at: `start < 5` of a `Window` at `window` reads
    /// `window.start < 5` (beyond10x/ess#234). What [`repair`] solves the base for.
    struct_invariants: Vec<Predicate>,
    /// The choices [`repair`] made so the base satisfies the invariants over the input, by path:
    /// the value each of those leaves takes in place of its own witness, and the length of each
    /// list an invariant counts, in every candidate.
    fixed: BTreeMap<FactPath, Choice>,
    /// The base each leaf [`repair`] fixed would have taken without it, by path: tried on the
    /// leaf's ladder as well, so a guard whose literal the repair moved the base onto is still
    /// refuted by some candidate.
    unrepaired: BTreeMap<FactPath, Node>,
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
            maps: BTreeSet::new(),
            building: Vec::new(),
            copied: BTreeSet::new(),
            unfolded: false,
            optionals: BTreeSet::new(),
            invariants: BTreeMap::new(),
            alphabets: BTreeMap::new(),
            prefixes: BTreeMap::new(),
            strings: BTreeSet::new(),
            plain_texts: BTreeMap::new(),
            examples: BTreeMap::new(),
            struct_invariants: Vec::new(),
            fixed: BTreeMap::new(),
            unrepaired: BTreeMap::new(),
        }
    }

    /// The value [`repair`] fixed the leaf at `path` to, where it fixed one.
    fn fixed_value(&self, path: &FactPath) -> Option<&Node> {
        match self.fixed.get(path) {
            Some(Choice::Value(value)) => Some(value),
            _ => None,
        }
    }

    /// The base of the leaf at `path`: the value [`repair`] fixed there, with `own` kept as its
    /// unrepaired witness when `record`; otherwise `own`.
    fn repaired(&mut self, path: &FactPath, own: Node, record: bool) -> Node {
        match self.fixed_value(path).cloned() {
            Some(fixed) => {
                if record {
                    self.unrepaired.insert(path.clone(), own);
                }
                fixed
            }
            None => own,
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
        self.copied = copied_inputs(command);
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
            match overrides.get(path) {
                Some(Choice::Omit) => return Ok(None),
                Some(Choice::Null) => return Ok(Some(Node::Null)),
                _ => {}
            }
            // Absence is the base case of a type that refers to itself through `Optional`
            // (beyond10x/ess#416): filling this member would build a type already under
            // construction again, the same way, to the depth limit.
            if rebuilds(self.ir, type_ref, &self.building) {
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
        // The prefix goes in front first, and the alphabet then maps the whole text: every
        // character of the prefix is in the alphabet, which validation holds, so mapping keeps it.
        if let (Node::Text(text), Some(prefix)) = (&base, self.prefixes.get(path)) {
            base = Node::Text(with_prefix(text, prefix));
        }
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
        let base = self.repaired(path, base, record);
        if record {
            self.leaves
                .insert(path.clone(), (Leaf::of_primitive(name), base.clone()));
        }
        base
    }

    /// The one element of a copied list, or the value of a map's entry, built at `<path>.0`; or
    /// `None` where it has none, and then nothing the attempt recorded is kept.
    ///
    /// A value that reaches a type already being built is unfolded once — `{name, children:
    /// {"children": {name, children: {}}}}` — and every map and copied list inside that one entry is
    /// empty, so a type that refers to itself through a map keeps a finite witness (beyond10x/ess#196).
    fn entry(
        &mut self,
        of: &ResolvedTypeRef,
        path: &FactPath,
        overrides: &BTreeMap<FactPath, Choice>,
        depth: usize,
        record: bool,
    ) -> Option<Node> {
        let recursive = reaches(self.ir, of, &self.building);
        if recursive && self.unfolded {
            return None;
        }
        let mut probe = self.clone();
        probe.unfolded |= recursive;
        let value = probe
            .value(of, &path.child("0"), overrides, depth + 1, record)
            .ok()?;
        probe.unfolded = self.unfolded;
        *self = probe;
        Some(value)
    }

    /// Whether `path` is, or lies inside, an input a branch copies whole.
    fn is_copied(&self, path: &FactPath) -> bool {
        self.copied
            .iter()
            .any(|copied| path.segments().starts_with(copied.segments()))
    }

    /// One value of `List<of>` at `path`: `[]`, unless a guard reads into it (rule 3) or a branch
    /// copies it (beyond10x/ess#196), where it holds one element.
    fn list(
        &mut self,
        of: &ResolvedTypeRef,
        path: &FactPath,
        overrides: &BTreeMap<FactPath, Choice>,
        depth: usize,
        record: bool,
    ) -> Result<Node, WitnessGap> {
        if !record || !self.expand.contains(path) {
            // A copied list holds one element, so a target that drops it, or anything inside it,
            // fails the copy's assertion; a guard's list keeps the ladders below.
            if record && self.is_copied(path) {
                if let Some(element) = self.entry(of, path, overrides, depth, record) {
                    return Ok(Node::Seq(vec![element]));
                }
            }
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
        if let Some(Choice::Keyed(keyed)) = overrides.get(path) {
            return self.keyed(of, path, element, keyed, depth).map(Node::Seq);
        }
        // A length the repaired base carries (an invariant counts the list) is the base's; a
        // candidate's own length wins over it.
        Ok(
            match overrides
                .get(path)
                .filter(|choice| matches!(choice, Choice::Elements(_)))
                .or_else(|| self.fixed.get(path))
            {
                Some(Choice::Elements(held)) => Node::Seq(vec![element; *held]),
                _ => Node::Seq(Vec::new()),
            },
        )
    }

    /// The elements of a list a `distinct` reads ([`Choice::Keyed`]): `first`, then each further one
    /// built at its own position and the next [`Distinction`] — a `String` from its path, every
    /// other primitive and an enum from the distinction — so the keys differ wherever the key's type
    /// has enough values; a finite one that has not is left to the evaluator to call a duplicate.
    /// With a repeat, the last element takes element 0's key, spelled as another instant where it is
    /// one, and keeps every other member it was built with: a nonadjacent duplicate that only the
    /// key makes one.
    fn keyed(
        &mut self,
        of: &ResolvedTypeRef,
        path: &FactPath,
        first: Node,
        keyed: &Keyed,
        depth: usize,
    ) -> Result<Vec<Node>, WitnessGap> {
        let mut elements = vec![first];
        for ordinal in 1..keyed.held {
            let mut further = self.clone();
            further.distinction = Distinction::further(self.distinction.get() + ordinal);
            elements.push(further.value(
                of,
                &path.child(&ordinal.to_string()),
                &BTreeMap::new(),
                depth + 1,
                false,
            )?);
        }
        if let (Some(repeat), [first, .., last]) = (&keyed.repeat, elements.as_mut_slice()) {
            if let Some(key) = member_at(first, &repeat.member).cloned() {
                let key = if repeat.instant {
                    respelled(&key).unwrap_or(key)
                } else {
                    key
                };
                if let Some(slot) = member_at_mut(last, &repeat.member) {
                    *slot = key;
                }
            }
        }
        Ok(elements)
    }

    /// One value of `Map<key, of>` at `path`: one entry (beyond10x/ess#196), or as many as a
    /// [`Choice::Elements`] asks for where a guard reads `<path>.count`.
    ///
    /// The key is [`map_key`]; the value is built at `<path>.0`, as a list's element is, and every
    /// further entry holds a copy of it under the key of the next instance. Where no entry can be
    /// built — a `Decimal` key has no setup spelling, and a value type that refers to itself or
    /// holds a `Binary64` has no finite witness — the map is `{}`, as it always was, and nothing
    /// the attempt recorded is kept.
    fn map(
        &mut self,
        type_ref: &ResolvedTypeRef,
        path: &FactPath,
        overrides: &BTreeMap<FactPath, Choice>,
        depth: usize,
        record: bool,
    ) -> Node {
        let ResolvedTypeRef::Map { key, value: of } = type_ref else {
            unreachable!("called for a map only");
        };
        let key = *key;
        let Some(first) = map_key(key, path, self.distinction) else {
            return Node::Map(BTreeMap::new());
        };
        let Some(value) = self.entry(of, path, overrides, depth, record) else {
            return Node::Map(BTreeMap::new());
        };
        if record {
            self.maps.insert(path.clone());
        }
        let held = match overrides.get(path) {
            Some(Choice::Elements(held)) => *held,
            _ => 1,
        };
        let mut entries = BTreeMap::new();
        if held > 0 {
            entries.insert(first, value.clone());
        }
        for ordinal in 1..held {
            let at = Distinction(self.distinction.get().saturating_add(ordinal));
            if let Some(spelling) = map_key(key, path, at) {
                entries.insert(spelling, value.clone());
            }
        }
        Node::Map(entries)
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
            ResolvedTypeRef::Map { .. } => Ok(self.map(type_ref, path, overrides, depth, record)),
            ResolvedTypeRef::Primitive { name } => {
                if *name == Primitive::Binary64 {
                    return Err(WitnessGap { path: path.to_string(), type_ref: name.to_string(), reason: "requires a finite Binary64 conformance codec that this suite format does not admit" });
                }
                Ok(chosen(self.primitive(*name, path, record)))
            }
            ResolvedTypeRef::Declared { name } => {
                // Every declared type on the way down is known, so a map whose value reaches one
                // is witnessed `{}` at once (beyond10x/ess#196) rather than at the depth limit.
                let declared = self.ir.named_type(name).name.to_string();
                self.building.push(declared);
                let built = self.declared(type_ref, path, overrides, depth, record);
                self.building.pop();
                built
            }
        }
    }

    /// A tagged union's witness: the variant a unit union's distinction names, or its first label.
    fn union_value(
        &mut self,
        type_ref: &ResolvedTypeRef,
        tag: &str,
        variants: &BTreeMap<String, Option<ResolvedTypeRef>>,
        path: &FactPath,
        overrides: &BTreeMap<FactPath, Choice>,
        depth: usize,
    ) -> Result<Node, WitnessGap> {
        let content = union_content_key(tag);
        let labelled: Vec<_> = variants.iter().collect();
        if labelled.is_empty() {
            return Ok(Node::Map(BTreeMap::new()));
        }
        // A union with no unit variant keeps its one witness, its first label. One with a
        // unit variant (ess/22) starts at the label this instance's distinction names, so
        // further instances witness every variant, and takes the first from there that
        // builds; a unit variant, the tag alone, always does — so a union admitted
        // because of it is witnessed whatever the order of its labels.
        let (start, tries) = if variants.values().any(Option::is_none) {
            (self.distinction.get() % labelled.len(), labelled.len())
        } else {
            (0, 1)
        };
        let mut failed = None;
        for nth in 0..tries {
            let (label, variant) = labelled[(start + nth) % labelled.len()];
            let tagged = (tag.to_owned(), Node::Text(label.clone()));
            let Some(variant) = variant else {
                return Ok(Node::Map(BTreeMap::from([tagged])));
            };
            // A payload that leads back to a type being built would nest to the depth
            // limit; where a unit variant ends the recursion, it is skipped for one that
            // does.
            if tries > 1 && reaches(self.ir, variant, &self.building) {
                continue;
            }
            match self.value(variant, path, overrides, depth + 1, false) {
                Ok(inner) => {
                    return Ok(Node::Map(BTreeMap::from([
                        tagged,
                        (content.to_owned(), inner),
                    ])))
                }
                Err(gap) => {
                    failed.get_or_insert(gap);
                }
            }
        }
        Err(failed.unwrap_or_else(|| WitnessGap {
            path: path.to_string(),
            type_ref: type_ref.to_string(),
            reason: "refers to itself, so it has no finite value to send",
        }))
    }

    /// One value of the declared type `type_ref`, at `path`: [`value`](Self::value)'s arm for it.
    fn declared(
        &mut self,
        type_ref: &ResolvedTypeRef,
        path: &FactPath,
        overrides: &BTreeMap<FactPath, Choice>,
        depth: usize,
        record: bool,
    ) -> Result<Node, WitnessGap> {
        let ResolvedTypeRef::Declared { name } = type_ref else {
            unreachable!("called for a declared type only");
        };
        let chosen = |base: Node| match overrides.get(path) {
            Some(Choice::Value(value)) => value.clone(),
            _ => base,
        };
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
                if !self.prefixes.contains_key(path) {
                    if let Some(prefix) = chain_prefix(ir, type_ref) {
                        self.prefixes.insert(path.clone(), prefix);
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
                let own = self
                    .examples
                    .get(path)
                    .cloned()
                    .unwrap_or(Node::Text(variant));
                let base = self.repaired(path, own, record);
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
                self.union_value(type_ref, tag, variants, path, overrides, depth)
            }
            ResolvedBody::Struct { fields, invariants } => {
                if record {
                    for invariant in invariants {
                        let moved = remapped(
                            &invariant.predicate,
                            &|read: &FactPath| Some(joined(path, read)),
                            &[],
                        );
                        if !self.struct_invariants.contains(&moved) {
                            self.struct_invariants.push(moved);
                        }
                    }
                }
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
            Primitive::Json => Self::Json,
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
        // A small object (beyond10x/ess#138), keyed and valued by the path as a text witness is, so
        // two `Json` fields — and two instances — never carry one value, and a payload that copies
        // it is held to the whole structure rather than to a scalar.
        Primitive::Json => Node::Map(std::collections::BTreeMap::from([(
            path.to_string(),
            Node::Text(if nth == 0 {
                path.to_string()
            } else {
                format!("{path}-{nth}")
            }),
        )])),
    }
}

/// Whether a value of `type_ref` reaches one of the declared types in `building`: a type that
/// refers to itself through a map or a copied list, which is witnessed empty there rather than
/// nested to the depth limit (beyond10x/ess#196).
fn reaches(ir: &EssIr, type_ref: &ResolvedTypeRef, building: &[String]) -> bool {
    fn walk(
        ir: &EssIr,
        type_ref: &ResolvedTypeRef,
        building: &[String],
        seen: &mut BTreeSet<String>,
    ) -> bool {
        match type_ref {
            ResolvedTypeRef::Primitive { .. } => false,
            ResolvedTypeRef::Optional { of } | ResolvedTypeRef::List { of } => {
                walk(ir, of, building, seen)
            }
            ResolvedTypeRef::Map { value, .. } => walk(ir, value, building, seen),
            ResolvedTypeRef::Declared { name } => {
                let declared = ir.named_type(name);
                let spelled = declared.name.to_string();
                if building.contains(&spelled) {
                    return true;
                }
                if !seen.insert(spelled) {
                    return false;
                }
                match &declared.body {
                    ResolvedBody::Newtype { of, .. } => walk(ir, of, building, seen),
                    ResolvedBody::Enum { .. } => false,
                    ResolvedBody::Union { variants, .. } => variants
                        .values()
                        .flatten()
                        .any(|variant| walk(ir, variant, building, seen)),
                    ResolvedBody::Struct { fields, .. } => fields
                        .iter()
                        .any(|field| walk(ir, &field.type_ref, building, seen)),
                }
            }
        }
    }
    !building.is_empty() && walk(ir, type_ref, building, &mut BTreeSet::new())
}

/// Whether the base value of `type_ref` would build one of the declared types in `building`
/// again: the builder's own choices, followed — every member filled, a union's first variant,
/// and no list or map, whose base is empty or [`reaches`]'s to bound.
///
/// Such a value has no end, so where this holds for an `Optional` member, leaving it out is the
/// only finite base. A model where it holds was refused at the depth limit before this rule, so
/// no witness that was built before changes.
fn rebuilds(ir: &EssIr, type_ref: &ResolvedTypeRef, building: &[String]) -> bool {
    fn walk(
        ir: &EssIr,
        type_ref: &ResolvedTypeRef,
        building: &[String],
        seen: &mut BTreeSet<String>,
    ) -> bool {
        match type_ref {
            ResolvedTypeRef::Primitive { .. }
            | ResolvedTypeRef::List { .. }
            | ResolvedTypeRef::Map { .. } => false,
            ResolvedTypeRef::Optional { of } => walk(ir, of, building, seen),
            ResolvedTypeRef::Declared { name } => {
                let declared = ir.named_type(name);
                let spelled = declared.name.to_string();
                if building.contains(&spelled) {
                    return true;
                }
                if !seen.insert(spelled) {
                    return false;
                }
                match &declared.body {
                    ResolvedBody::Newtype { of, .. } => walk(ir, of, building, seen),
                    ResolvedBody::Enum { .. } => false,
                    // A unit variant (ess/22, beyond10x/ess#418) always builds, so such a union never
                    // rebuilds; otherwise its witness is its first variant.
                    ResolvedBody::Union { variants, .. } => {
                        !variants.values().any(Option::is_none)
                            && variants
                                .values()
                                .next()
                                .and_then(Option::as_ref)
                                .is_some_and(|variant| walk(ir, variant, building, seen))
                    }
                    ResolvedBody::Struct { fields, .. } => fields
                        .iter()
                        .any(|field| walk(ir, &field.type_ref, building, seen)),
                }
            }
        }
    }
    !building.is_empty() && walk(ir, type_ref, building, &mut BTreeSet::new())
}

/// Every input path a branch copies whole into an event payload or a stored field: `input.<x>`,
/// and `{input: x, else: …}`, at any depth of a struct-valued target.
fn copied_inputs(command: &ResolvedCommand) -> BTreeSet<FactPath> {
    use ess_compiler::ir::{ResolvedPayloadField, ResolvedPayloadValue};
    fn walk(fields: &[ResolvedPayloadField], found: &mut BTreeSet<FactPath>) {
        for field in fields {
            match &field.value {
                ResolvedPayloadValue::InputField { field, .. }
                | ResolvedPayloadValue::InputOrGenerated { field, .. } => {
                    if let Ok(path) = FactPath::new(field) {
                        found.insert(path);
                    }
                }
                ResolvedPayloadValue::Struct { fields } => walk(fields, found),
                _ => {}
            }
        }
    }
    let mut found = BTreeSet::new();
    for outcome in &command.outcomes {
        for payload in &outcome.payload {
            walk(&payload.fields, &mut found);
        }
        walk(&outcome.sets, &mut found);
        for affect in &outcome.affects {
            walk(&affect.sets, &mut found);
        }
    }
    found
}

/// The key of a map's witness entry: the key primitive's own witness at the map's path and
/// instance (rules 2 and 5), in the spelling a setup key is read by — `true`, `1`, the text itself —
/// or `None` where the primitive has no setup spelling.
///
/// Held to the same reader the flattener admits a key by, so a key it builds is never refused.
fn map_key(key: Primitive, path: &FactPath, distinction: Distinction) -> Option<String> {
    if matches!(
        key,
        Primitive::Binary64 | Primitive::Json | Primitive::Decimal
    ) {
        return None;
    }
    let spelling = match primitive_value(key, path, distinction) {
        Node::Text(text) => text,
        Node::Bool(flag) => flag.to_string(),
        Node::Number(number) => number.as_i64()?.to_string(),
        _ => return None,
    };
    crate::input::setup_map_key(key, &spelling)
        .is_ok()
        .then_some(spelling)
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
