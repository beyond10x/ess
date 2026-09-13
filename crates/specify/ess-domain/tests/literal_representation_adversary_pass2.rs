//! Second adversary pass for `story:literal-representation-walk-exhaustion`.
//!
//! Pass 1 attacked the walk's exhaustion. This one attacks the answer the unit gave to pass 1: the
//! split of the cyclic case into `Cyclic` (refused here) and `Uninhabited` (left to
//! `check_inhabitation`), decided by counting `Optional`s crossed *inside* the ring.
//!
//! That split is a claim about two passes agreeing, and neither pass's own tests can check it: the
//! unit's cases call `validate_bindings`, which does not run `check_inhabitation` at all, so
//! `assert!(errors.is_empty())` there is equally true whether the type pass owns the shape or
//! nobody does. Everything here goes through `RawSpecFile::parse` + `Specification::assemble`,
//! which runs both, so "silence" means silence from the whole compiler.

use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_domain::Specification;

/// The `ess-domain` crate root.
fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Assemble one document; `None` when it was admitted, the joined refusals when it was not.
fn refusals(document: &str) -> Option<String> {
    let raw = match RawSpecFile::parse(document) {
        Ok(raw) => raw,
        Err(error) => return Some(error.to_string()),
    };
    Specification::assemble(vec![(Source::new("adversary2.yaml"), raw)])
        .err()
        .map(|errors| errors.to_string())
}

/// The three names the matrix declares.
const NAMES: [&str; 3] = ["Alpha", "Beta", "Gamma"];

/// What one of the three names is declared as.
///
/// Every body a `representation` walk can *continue* through, and every body it stops on with an
/// answer. The struct and the union are here rather than in a case of their own because they are
/// the shapes that close a ring *before* the walk returns to a name it has seen — the walk has no
/// second arrival to count base cases against, so whether that ring has values is a question only
/// the inhabitation fixpoint answers. Two hand-written shapes checked that; every position of
/// every three-name registry checks it now.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Body {
    /// `kind: newtype, of: <another name>` — no base case of its own.
    Bare(usize),
    /// `kind: newtype, of: Optional<another name>` — a base case, because absence is a value.
    Absent(usize),
    /// `kind: newtype, of: String`.
    Text,
    /// `kind: newtype, of: <the enum>`.
    Variants,
    /// `kind: struct` with one field naming another name — the walk stops with structure, and the
    /// struct is buildable exactly when that name is.
    Fields(usize),
    /// `kind: union` with one variant naming another name — the same stop, reached the other way.
    Choice(usize),
}

/// Every body the matrix puts each of the three names through.
const BODIES: [Body; 14] = [
    Body::Bare(0),
    Body::Bare(1),
    Body::Bare(2),
    Body::Absent(0),
    Body::Absent(1),
    Body::Absent(2),
    Body::Text,
    Body::Variants,
    Body::Fields(0),
    Body::Fields(1),
    Body::Fields(2),
    Body::Choice(0),
    Body::Choice(1),
    Body::Choice(2),
];

/// The `types:` block for one assignment of bodies.
fn declarations(bodies: &[Body; 3]) -> String {
    let mut out = String::new();
    for (index, body) in bodies.iter().enumerate() {
        let name = NAMES[index];
        let body = match body {
            Body::Bare(to) => format!(
                "    kind: newtype\n    of: notifications.core.{}\n",
                NAMES[*to]
            ),
            Body::Absent(to) => format!(
                "    kind: newtype\n    of: Optional<notifications.core.{}>\n",
                NAMES[*to]
            ),
            Body::Text => "    kind: newtype\n    of: String\n".to_owned(),
            Body::Variants => "    kind: newtype\n    of: notifications.core.Reason\n".to_owned(),
            Body::Fields(to) => format!(
                "    kind: struct\n    fields:\n      - name: only\n        type: \
                 notifications.core.{}\n",
                NAMES[*to]
            ),
            Body::Choice(to) => format!(
                "    kind: union\n    tag: kind\n    variants:\n      only: \
                 notifications.core.{}\n",
                NAMES[*to]
            ),
        };
        let _ = write!(out, "  - name: notifications.core.{name}\n{body}");
    }
    out
}

/// How a shape reads in a failure message.
fn render(bodies: &[Body; 3]) -> String {
    let mut out = String::new();
    for (index, body) in bodies.iter().enumerate() {
        let of = match body {
            Body::Bare(to) => NAMES[*to].to_owned(),
            Body::Absent(to) => format!("Optional<{}>", NAMES[*to]),
            Body::Text => "String".to_owned(),
            Body::Variants => "Reason".to_owned(),
            Body::Fields(to) => format!("struct{{{}}}", NAMES[*to]),
            Body::Choice(to) => format!("union{{{}}}", NAMES[*to]),
        };
        let _ = write!(out, "{} = {of}; ", NAMES[index]);
    }
    out
}

/// A document whose binding writes `literal` into a command input of `target`.
fn binding_document(types: &str, target: &str, literal: &str) -> String {
    format!(
        r"
format: ess/1
system: notifications
version: v1
domain: notifications.core
types:
  - name: notifications.core.Reason
    kind: enum
    variants: [declined, unavailable]
{types}commands:
  - name: notifications.core.Reject
    input:
      - name: reason
        type: {target}
    outcomes:
      - name: rejected
        emits: [notifications.core.Rejected]
events:
  - name: notifications.core.Refused
  - name: notifications.core.Rejected
bindings:
  - id: reject-on-refusal
    when:
      event: notifications.core.Refused
    invoke:
      command: notifications.core.Reject
    mapping:
      reason: {literal}
    delivery: at_most_once
    on_failure: drop
"
    )
}

/// A document whose *outcome payload* writes `literal` into an event field of `target`.
///
/// The other consumer of the same authority. The command input is the enum itself, mapped with a
/// real variant, so the only literal under test is the payload's.
fn payload_document(types: &str, target: &str, literal: &str) -> String {
    format!(
        r"
format: ess/1
system: notifications
version: v1
domain: notifications.core
types:
  - name: notifications.core.Reason
    kind: enum
    variants: [declined, unavailable]
{types}commands:
  - name: notifications.core.Reject
    input:
      - name: reason
        type: notifications.core.Reason
    outcomes:
      - name: rejected
        emits: [notifications.core.Rejected]
        payload:
          notifications.core.Rejected:
            detail: {literal}
events:
  - name: notifications.core.Refused
  - name: notifications.core.Rejected
    fields:
      - name: detail
        type: {target}
bindings:
  - id: reject-on-refusal
    when:
      event: notifications.core.Refused
    invoke:
      command: notifications.core.Reject
    mapping:
      reason: declined
    delivery: at_most_once
    on_failure: drop
"
    )
}

/// Which of the three names a value can be built for.
///
/// Written from the rule `check_inhabitation` states rather than from the walk: `Optional` is a
/// base case whatever it holds, a primitive and an enum are base cases, and a bare name is only as
/// buildable as the name it holds. This is the other half of the equivalence the unit's
/// `representation` doc asserts — "the same base-case rule `check_inhabitation` applies" — so it
/// must not be derived from the walk it is being compared against.
fn inhabited(bodies: &[Body; 3]) -> [bool; 3] {
    let mut known = [false; 3];
    loop {
        let mut grew = false;
        for index in 0..3 {
            if known[index] {
                continue;
            }
            let now = match bodies[index] {
                // A struct needs its one field, and a union with one variant needs that variant:
                // both are only as buildable as the name they hold, exactly as a bare newtype is.
                Body::Bare(to) | Body::Fields(to) | Body::Choice(to) => known[to],
                Body::Absent(_) | Body::Text | Body::Variants => true,
            };
            if now {
                known[index] = true;
                grew = true;
            }
        }
        if !grew {
            return known;
        }
    }
}

/// Where a literal filling `Alpha` ends up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Landing {
    /// `String` underneath: admitted, and nothing is said about the value.
    Text,
    /// An enum underneath: the literal must name a variant.
    Variants,
    /// The chain returned to a name. `inhabited` is whether values of *that name* exist, which is
    /// what `story:literal-representation-walk-exhaustion` decided the ring answers turn on.
    ///
    /// Deliberately **not** the question the `Structured` arm below asks, and the difference is a
    /// recorded open finding rather than an oversight. Measured on this tree:
    /// `Alpha = Optional<Beta>` with `Beta = Gamma`, `Gamma = Beta` draws two `self_reference`
    /// lines about `Beta` and `Gamma` and nothing at all about the mapping — yet `Alpha` itself is
    /// inhabited, its one value being absence, which no literal can spell. Read the way the
    /// `Structured` arm now reads, that is `OwnedByNobody`, and 36 of these shapes are in this
    /// matrix.
    ///
    /// It is left standing because deciding it belongs to the story that owns this arm, not to
    /// `story:structured-ring-is-refused-by-two-passes`: answering it would make the walk's rule
    /// identical to the mutant
    /// [`counting_optionals_outside_the_ring_would_move_shapes_the_matrix_asserts_on`] exists to
    /// rule out, and that case would then be unsatisfiable. Two stories disagree here and a person
    /// should say which is right.
    Ring { inhabited: bool },
    /// The walk stopped on a struct or a union.
    Structured {
        /// Whether values of **`Alpha`** exist, for the reason [`Landing::owner`] gives.
        filled_exists: bool,
    },
}

/// Where a literal filling `Alpha` ends up, by the model above.
fn landing(bodies: &[Body; 3]) -> Landing {
    let known = inhabited(bodies);
    let mut seen = [false; 3];
    let mut current = 0_usize;
    loop {
        if seen[current] {
            return Landing::Ring {
                inhabited: known[current],
            };
        }
        seen[current] = true;
        match bodies[current] {
            Body::Text => return Landing::Text,
            Body::Variants => return Landing::Variants,
            Body::Fields(_) | Body::Choice(_) => {
                return Landing::Structured {
                    filled_exists: known[0],
                }
            }
            Body::Bare(to) | Body::Absent(to) => current = to,
        }
    }
}

/// Where a refusal of the binding's literal is reported, which is the site every ownership arm
/// asks about: a diagnostic elsewhere in the document repairs a different mistake.
const MAPPING: &str = "binding.reject-on-refusal.mapping.reason";

/// How `check_inhabitation` names one type it refuses.
///
/// Asserted against the *name*, because "somebody said `no value of` about something in this
/// document" is true of a document whose literal nobody checked at all.
fn refusal_of(name: &str) -> String {
    format!("no value of `notifications.core.{name}`")
}

/// The three ways one shape can be wrong, which the matrix reports in separate lists.
enum Fault {
    /// A literal nobody checked was let through.
    AdmittedUnchecked,
    /// Something is wrong with the document and no pass said so.
    OwnedByNobody,
    /// One mistake, reported by two passes — or a good document refused at all.
    RefusedTwice,
}

/// What one shape's refusals say about who owns it, or `None` when exactly one pass spoke.
///
/// The whole property of this file, per shape: `landed` is where the model says a literal filling
/// `Alpha` ends up, `found` is what the whole compiler said about that document, and every arm
/// names the single pass entitled to speak.
///
/// Every ownership arm turns on whether values of **`Alpha`** exist, and that is the correction
/// `review-result:adversary-wave23-unit1-pass-2` forced. `check_inhabitation` refuses
/// *declarations*; the literal fills `Alpha`; so the only way that pass repairs this literal is by
/// refusing `Alpha` itself. A model that asked instead about the name the walk stopped on agreed
/// with the code by construction — `Alpha = Optional<Beta>` with `Beta` a refused ring or struct
/// leaves `Alpha` perfectly inhabited, its one value being absence, which no literal spells — and
/// 2744 shapes of agreement said nothing about either.
fn audit(
    shape: &str,
    landed: Landing,
    found: Option<&str>,
    buildable: [bool; 3],
) -> Option<(Fault, String)> {
    match landed {
        Landing::Variants => match found {
            None => Some((
                Fault::AdmittedUnchecked,
                format!("{shape}: `nope` admitted into an enum-backed input"),
            )),
            Some(errors) if !errors.contains("not a variant") => Some((
                Fault::AdmittedUnchecked,
                format!("{shape}: refused, but not as a variant: {errors}"),
            )),
            Some(_) => None,
        },
        Landing::Ring { inhabited: true } => match found {
            None => Some((
                Fault::AdmittedUnchecked,
                format!("{shape}: a literal filling a ring with nothing underneath it, admitted"),
            )),
            Some(errors) if !errors.contains("resolves through") => Some((
                Fault::OwnedByNobody,
                format!(
                    "{shape}: the ring has values, so only the literal check can refuse it, and \
                     it did not: {errors}"
                ),
            )),
            Some(_) => None,
        },
        Landing::Ring { inhabited: false } => match found {
            None => Some((
                Fault::OwnedByNobody,
                format!("{shape}: a ring no value inhabits, admitted by every pass"),
            )),
            Some(errors) if errors.contains("resolves through") => Some((
                Fault::RefusedTwice,
                format!(
                    "{shape}: the literal check refused a ring it says the type pass owns: {errors}"
                ),
            )),
            Some(errors) if !errors.contains("no value of") => Some((
                Fault::OwnedByNobody,
                format!(
                    "{shape}: the literal check is silent because `check_inhabitation` owns this, \
                     and `check_inhabitation` did not refuse it: {errors}"
                ),
            )),
            Some(_) => None,
        },
        // `Alpha` can be built, so `check_inhabitation` says nothing about it whatever it says
        // about the struct underneath, and an unrefused literal here was checked by no pass.
        Landing::Structured {
            filled_exists: true,
        } => match found {
            None => Some((
                Fault::AdmittedUnchecked,
                format!("{shape}: a literal filling a shape with structure, admitted"),
            )),
            Some(errors) if !errors.contains(MAPPING) => Some((
                Fault::OwnedByNobody,
                format!(
                    "{shape}: `Alpha` can be built, so only the literal check can refuse it, and \
                     nothing was said about the mapping: {errors}"
                ),
            )),
            Some(_) => None,
        },
        // `Alpha` cannot be built, so `check_inhabitation` refuses `Alpha` itself. Saying it again
        // over the literal reports one mistake twice and repairs it neither time.
        Landing::Structured {
            filled_exists: false,
        } => match found {
            None => Some((
                Fault::OwnedByNobody,
                format!("{shape}: a shape no value inhabits, admitted by every pass"),
            )),
            Some(errors) if errors.contains(MAPPING) => Some((
                Fault::RefusedTwice,
                format!("{shape}: the literal check refused a shape the type pass owns: {errors}"),
            )),
            Some(errors) if !errors.contains(&refusal_of("Alpha")) => Some((
                Fault::OwnedByNobody,
                format!(
                    "{shape}: the literal check is silent because `check_inhabitation` owns \
                     `Alpha`, and `check_inhabitation` did not refuse `Alpha`: {errors}"
                ),
            )),
            Some(_) => None,
        },
        Landing::Text => match found {
            Some(errors) if errors.contains("resolves through") => Some((
                Fault::RefusedTwice,
                format!("{shape}: `String` underneath, and still refused as cyclic: {errors}"),
            )),
            Some(errors) if buildable == [true; 3] => Some((
                Fault::RefusedTwice,
                format!(
                    "{shape}: every name can be built and the literal is text, so this is a good \
                     document: {errors}"
                ),
            )),
            Some(_) | None => None,
        },
    }
}

/// Every three-name registry of `Optional`/newtype wrappers lands where the two passes say it does.
///
/// 2744 shapes — every assignment of fourteen bodies to three names — driven through the whole
/// compiler. The properties, and the third is the one this pass exists for:
///
/// 1. a literal filling an enum-backed input is checked against the variants, however long or
///    twisted the chain (the acceptance statement);
/// 2. a ring that has values is refused by the literal check, because nothing else refuses it;
/// 3. a ring that has no values is refused by *somebody* — if the literal check is silent because
///    `check_inhabitation` owns it, `check_inhabitation` has to actually say so;
/// 4. a chain that reaches `String` through names that can all be built is a good document;
/// 5. and the same ownership question for the shapes that stop the walk on a name: a struct or a
///    union that can be built is the literal check's alone, and one that cannot is
///    `check_inhabitation`'s alone. That half is what
///    `story:structured-ring-is-refused-by-two-passes` was filed for, and stating it over the whole
///    matrix rather than over two hand-written registries is what keeps it stated.
#[test]
fn every_three_name_wrapper_registry_is_owned_by_exactly_one_pass() {
    let mut admitted_unchecked: Vec<String> = Vec::new();
    let mut owned_by_nobody: Vec<String> = Vec::new();
    let mut refused_twice: Vec<String> = Vec::new();
    // Non-vacuity: `text`, `variants`, `inhabited ring`, `uninhabited ring`, `good document`,
    // `inhabited structure`, `uninhabited structure`.
    let mut reached = [0_usize; 7];

    for alpha in BODIES {
        for beta in BODIES {
            for gamma in BODIES {
                let bodies = [alpha, beta, gamma];
                let shape = render(&bodies);
                let found = refusals(&binding_document(
                    &declarations(&bodies),
                    "notifications.core.Alpha",
                    "nope",
                ));
                let buildable = inhabited(&bodies);

                let where_it_landed = landing(&bodies);
                reached[match where_it_landed {
                    Landing::Text if buildable == [true; 3] => 4,
                    Landing::Text => 0,
                    Landing::Variants => 1,
                    Landing::Ring { inhabited: true } => 2,
                    Landing::Ring { inhabited: false } => 3,
                    Landing::Structured {
                        filled_exists: true,
                    } => 5,
                    Landing::Structured {
                        filled_exists: false,
                    } => 6,
                }] += 1;

                if let Some((fault, note)) =
                    audit(&shape, where_it_landed, found.as_deref(), buildable)
                {
                    match fault {
                        Fault::AdmittedUnchecked => admitted_unchecked.push(note),
                        Fault::OwnedByNobody => owned_by_nobody.push(note),
                        Fault::RefusedTwice => refused_twice.push(note),
                    }
                }
            }
        }
    }

    // A filter that matches nothing exits green, and so does a matrix whose branches are never
    // taken. Every class this case claims to cover has to have been populated by a real document.
    assert!(
        reached.iter().all(|count| *count > 0),
        "vacuous matrix — text / variants / inhabited ring / uninhabited ring / good document / \
         inhabited structure / uninhabited structure reached {reached:?} times"
    );
    assert!(
        admitted_unchecked.is_empty() && owned_by_nobody.is_empty() && refused_twice.is_empty(),
        "admitted with no check ({}):\n{}\n\nowned by no pass ({}):\n{}\n\nrefused twice or \
         wrongly ({}):\n{}",
        admitted_unchecked.len(),
        admitted_unchecked.join("\n"),
        owned_by_nobody.len(),
        owned_by_nobody.join("\n"),
        refused_twice.len(),
        refused_twice.join("\n"),
    );
}

/// The same matrix, against the other consumer of the same authority.
///
/// The unit applied the four-answer split to the outcome payload as well as to the binding
/// mapping, and the acceptance names both. A difference between the two columns is the drift
/// "one representation authority" was supposed to make impossible.
#[test]
fn the_payload_consumer_lands_every_shape_exactly_where_the_binding_consumer_does() {
    let mut divergent: Vec<String> = Vec::new();

    for alpha in BODIES {
        for beta in BODIES {
            for gamma in BODIES {
                let bodies = [alpha, beta, gamma];
                let types = declarations(&bodies);
                let from_binding = refusals(&binding_document(
                    &types,
                    "notifications.core.Alpha",
                    "nope",
                ));
                let from_payload = refusals(&payload_document(
                    &types,
                    "notifications.core.Alpha",
                    "nope",
                ));

                // Compared on which claim each pass made, not on wording: the two messages name
                // different sites by design.
                let claims = |found: &Option<String>| {
                    found.as_deref().map(|errors| {
                        (
                            errors.contains("not a variant"),
                            errors.contains("resolves through"),
                            errors.contains("no value of"),
                            errors.contains("has structure"),
                        )
                    })
                };
                if claims(&from_binding) != claims(&from_payload) {
                    divergent.push(format!(
                        "{}: binding {:?} / payload {:?}\n  binding: {}\n  payload: {}",
                        render(&bodies),
                        claims(&from_binding),
                        claims(&from_payload),
                        from_binding.as_deref().unwrap_or("admitted"),
                        from_payload.as_deref().unwrap_or("admitted"),
                    ));
                }
            }
        }
    }

    assert!(
        divergent.is_empty(),
        "{} shapes where the two consumers of one authority disagree:\n{}",
        divergent.len(),
        divergent.join("\n"),
    );
}

/// A ring whose only base case is a `List`, a `Map` or a union variant, rather than an `Optional`.
///
/// `check_inhabitation`'s own hint names three ways to give a ring a base case — "put one step
/// behind `Optional` or `List`, which have a base case, or give a union a variant that does not
/// recurse" — and the walk counts only one of them. Each of these types *can* be built, so
/// `check_inhabitation` is silent about all of them, and the literal filling it must therefore be
/// refused here or nowhere.
#[test]
fn a_ring_whose_base_case_is_not_an_optional_is_still_refused_by_somebody() {
    for (label, types) in [
        (
            "List",
            "  - name: notifications.core.Ring\n    kind: newtype\n    of: \
             List<notifications.core.Ring>\n",
        ),
        (
            "Map",
            "  - name: notifications.core.Ring\n    kind: newtype\n    of: \
             Map<String, notifications.core.Ring>\n",
        ),
        (
            "union",
            "  - name: notifications.core.Ring\n    kind: newtype\n    of: \
             notifications.core.Pick\n  - name: notifications.core.Pick\n    kind: union\n    \
             tag: kind\n    variants:\n      leaf: notifications.core.Reason\n      deep: \
             notifications.core.Ring\n",
        ),
    ] {
        let found = refusals(&binding_document(types, "notifications.core.Ring", "nope"));
        let errors = found.unwrap_or_else(|| {
            panic!("{label}: a literal filling a ring with no text underneath it was admitted")
        });
        assert!(
            !errors.contains("no value of"),
            "{label}: this ring can be built, so the type pass has nothing to say: {errors}"
        );
    }
}

/// Every variant of the new `Resolution` enum has a consumer-coverage row, and no row is stale.
///
/// `consumer_coverage::proposal::classify` bails both ways: `unclassified concrete consumer entry`
/// for a variant with no row, and `stale or reasonless consumer classification` for a row naming an
/// entry the source no longer mints. Pass 1's case checks presence for the three variants that
/// existed then; the answer added a fourth, so the set has to be checked against the source rather
/// than against a list written by hand.
#[test]
fn the_resolution_rows_match_the_variants_the_source_declares() {
    let binding = fs::read_to_string(crate_root().join("src/binding.rs")).expect("read binding.rs");
    let text = fs::read_to_string(
        crate_root().join("../../edge/ess-xtask/src/consumer_coverage/entry-classifications.json"),
    )
    .expect("read entry-classifications.json");

    let body = binding
        .split_once("enum Resolution")
        .expect("binding.rs declares `enum Resolution`")
        .1;
    let body = body.split_once('{').expect("the enum has a body").1;
    let body = body.split("\n}").next().expect("the enum body ends");
    let declared: Vec<String> = body
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.ends_with('(')
                || line.ends_with(',')
                || line.ends_with("),")
                || !line.contains(' ')
        })
        .filter(|line| line.starts_with(|c: char| c.is_ascii_uppercase()))
        .map(|line| {
            line.trim_end_matches(',')
                .split(['(', ' ', '{'])
                .next()
                .unwrap_or_default()
                .to_owned()
        })
        .filter(|name| !name.is_empty())
        .collect();
    assert!(
        declared.len() >= 4,
        "control: the source declares the variants this case reads: {declared:?}"
    );

    let prefix = "ess_domain::lib(ess_domain)::binding::enum::Resolution";
    let unclassified: Vec<&String> = declared
        .iter()
        .filter(|name| !text.contains(&format!("\"{prefix}/variant/{name}\"")))
        .collect();
    assert!(
        unclassified.is_empty(),
        "unclassified concrete consumer entries, which `consumer-check` refuses: {unclassified:?}"
    );

    let stale: Vec<String> = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&format!("\"{prefix}/variant/")))
        .filter_map(|rest| rest.split('"').next())
        .filter(|name| !declared.iter().any(|declared| declared == name))
        .map(str::to_owned)
        .collect();
    assert!(
        stale.is_empty(),
        "stale consumer classifications, which `consumer-check` refuses just as loudly: {stale:?}"
    );
}

/// The matrix above would go red if the base-case rule were the obvious wrong one.
///
/// A green exhaustive matrix is worth exactly as much as its ability to fail, and the cheapest way
/// to overstate one is to build a model that cannot disagree with the code. The mutant at hand is
/// the rule the unit's own `representation` doc warns against — "an `Optional` crossed on the way
/// *to* the ring does not give the ring one" — so counting every `Optional` on the path instead of
/// only those inside the ring must move at least one shape from `Uninhabited` to `Cyclic`, and the
/// matrix asserts opposite things about those two.
#[test]
fn counting_optionals_outside_the_ring_would_move_shapes_the_matrix_asserts_on() {
    /// The mutant: any `Optional` anywhere on the walk is taken as the ring's base case.
    fn naive(bodies: &[Body; 3]) -> Landing {
        let mut seen = [false; 3];
        let mut crossed = false;
        let mut current = 0_usize;
        loop {
            if seen[current] {
                return Landing::Ring { inhabited: crossed };
            }
            seen[current] = true;
            match bodies[current] {
                Body::Text => return Landing::Text,
                Body::Variants => return Landing::Variants,
                // Not the rule under mutation: the mutant differs only in what it counts as the
                // ring's base case, so the shapes that stop the walk on a name answer as stated.
                Body::Fields(_) | Body::Choice(_) => {
                    return Landing::Structured {
                        filled_exists: inhabited(bodies)[0],
                    }
                }
                Body::Bare(to) => current = to,
                Body::Absent(to) => {
                    crossed = true;
                    current = to;
                }
            }
        }
    }

    let mut separating: Vec<String> = Vec::new();
    for alpha in BODIES {
        for beta in BODIES {
            for gamma in BODIES {
                let bodies = [alpha, beta, gamma];
                if naive(&bodies) != landing(&bodies) {
                    separating.push(format!(
                        "{}: stated {:?}, mutant {:?}",
                        render(&bodies),
                        landing(&bodies),
                        naive(&bodies)
                    ));
                }
            }
        }
    }
    assert!(
        !separating.is_empty(),
        "the matrix cannot tell the two rules apart, so its green says nothing about either"
    );
}

/// A ring that closes through a struct or a union is one mistake, and draws one diagnostic a name.
///
/// The rule `story:literal-representation-walk-exhaustion` adopted for the walk is that a mistake
/// another pass already owns gets silence here: a ring nothing can be built for belongs to
/// `check_inhabitation`, and saying it again over the literal "would report one mistake twice and
/// repair it neither time". `Alpha = newtype of Pick` with `Pick = struct {only: Alpha}` is such a
/// ring — nothing can be built for either name — and the walk meets the struct before it meets the
/// second `Alpha`, so for as long as the `Structured` answer did not share that rule the same
/// document drew three messages for it.
///
/// Two, now: one `self_reference` a name, from the pass that owns the shape, and nothing from the
/// literal check. Asserted for the struct shape and the union shape, against both consumers of the
/// one representation authority — a binding's `mapping:` and an outcome's `payload:` — because a
/// rule applied in the walk has to arrive at both or it is not the walk's rule.
///
/// Deliberately an equality on the count rather than an absence of one string: it goes red if the
/// literal check speaks again, and equally red if a third pass starts to.
#[test]
fn a_ring_closing_through_a_struct_or_union_is_reported_only_by_the_pass_that_owns_it() {
    for (shape, types) in [
        (
            "struct",
            "  - name: notifications.core.Alpha\n    kind: newtype\n    of: \
             notifications.core.Pick\n  - name: notifications.core.Pick\n    kind: struct\n    \
             fields:\n      - name: only\n        type: notifications.core.Alpha\n",
        ),
        (
            "union",
            "  - name: notifications.core.Alpha\n    kind: newtype\n    of: \
             notifications.core.Pick\n  - name: notifications.core.Pick\n    kind: union\n    \
             tag: kind\n    variants:\n      only: notifications.core.Alpha\n",
        ),
    ] {
        for (consumer, document) in [
            (
                "binding",
                binding_document(types, "notifications.core.Alpha", "nope"),
            ),
            (
                "payload",
                payload_document(types, "notifications.core.Alpha", "nope"),
            ),
        ] {
            let errors = refusals(&document).unwrap_or_else(|| {
                panic!("{shape}/{consumer}: a ring no value inhabits, admitted by every pass")
            });
            let diagnostics: Vec<&str> = errors
                .lines()
                .filter(|line| line.trim_start().starts_with("- ["))
                .collect();

            assert_eq!(
                diagnostics
                    .iter()
                    .filter(|line| line.contains("[self_reference]"))
                    .count(),
                2,
                "{shape}/{consumer}: `check_inhabitation` owns this shape and refuses both names \
                 of the ring:\n{errors}"
            );
            assert_eq!(
                diagnostics.len(),
                2,
                "{shape}/{consumer}: one mistake, and only the pass that owns it speaks — a \
                 literal check that refuses a shape the type pass already refused reports the \
                 same mistake twice and repairs it neither time:\n{errors}"
            );
        }
    }
}

/// The `Undeclared` answer's claim, at the two places it is easiest to lose.
///
/// `Resolution::Undeclared` says the silence is safe because the pass that resolves references
/// reports the name. That is checked here where the undeclared name is reached only by following
/// the walk — through an `Optional` written on the input itself, and at the far end of a chain
/// longer than the budget the walk used to have — rather than by reading one declaration.
#[test]
fn an_undeclared_name_the_walk_reaches_is_reported_by_the_pass_that_owns_it() {
    let mut chain = String::new();
    for index in 0..=ess_domain::binding::WRAPPER_LIMIT {
        let of = if index == ess_domain::binding::WRAPPER_LIMIT {
            "notifications.core.Missing".to_owned()
        } else {
            format!("notifications.core.Link{}", index + 1)
        };
        let _ = write!(
            chain,
            "  - name: notifications.core.Link{index}\n    kind: newtype\n    of: {of}\n"
        );
    }

    for (label, types, target) in [
        ("directly", "", "notifications.core.Missing"),
        (
            "under an Optional",
            "",
            "Optional<notifications.core.Missing>",
        ),
        (
            "past the old walk bound",
            chain.as_str(),
            "notifications.core.Link0",
        ),
    ] {
        let errors = refusals(&binding_document(types, target, "nope")).unwrap_or_else(|| {
            panic!("{label}: a literal filling an undeclared type was admitted by every pass")
        });
        assert!(
            !errors.contains("resolves through"),
            "{label}: an undeclared name is not a ring: {errors}"
        );
        assert!(
            errors.contains("notifications.core.Missing"),
            "{label}: the refusal has to name what nothing declares: {errors}"
        );
    }
}
