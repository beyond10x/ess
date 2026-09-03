//! The document a specification obliges: the first projection, and therefore the completeness check.
//!
//! This module walks [`EssIr`] and says what is true of it, as [`crate::document`] pages of blocks.
//! It writes no Markdown: which characters a heading, a link or a diagram is made of belongs to
//! [`crate::markdown`], which is the renderer whose bytes are pinned. What is here is the wording,
//! the order and the construct each section is about.
//!
//! Documentation is generated first because it is the cheapest way to find out what the model cannot
//! say. A construct with no rendering shows up here as a hole in a page a person reads, rather than
//! as a subtly wrong schema nobody validates — so the criterion this module is held to is *every
//! construct the IR carries appears on some page*, not *the pages look nice*.
//!
//! # Three ways a gap is made loud
//!
//! [`Generator::generate`] is infallible on purpose: a construct this crate cannot project is a gap
//! in this crate, not a fault in a specification that has already been resolved. So a gap cannot be
//! reported by failing — and must not be reported by crashing. A `panic!` here would turn "your
//! documentation is incomplete" into "the tool is broken", and it would destroy the very pages that
//! say what is missing, for a reader who cannot fix either. Instead:
//!
//! | the gap | how it becomes loud |
//! |---|---|
//! | a new variant of something this module renders | it stops compiling — no `match` on an enum here has a wildcard arm, so a new `Delivery`, `ResolvedBody`, `ResolvedCondition`, `ResolvedEffect`, `ResolvedFailure`, `ResolvedMappingValue`, `TestStrategy`, `Consistency` or `AssertionStyle` is a build failure in this file |
//! | a construct the IR holds that no page mentions | `tests/docs.rs` fails, asserted per construct |
//! | a construct the IR does not hold at all | [`Docs::known_gaps`], printed on the page where the reader went looking and counted in the index |
//!
//! The third ships nothing today: [`Docs::known_gaps`] is empty, because every construct
//! `ess-domain` parses now reaches [`EssIr`] and reaches a page — entities with their identity,
//! fields, invariants and lifecycle, views with their source, filter and consistency, actors with
//! their grants. The mechanism stays, and stays empty on purpose. It is an allowlist rather than a
//! discovery: a *new* gap is a failing test, and a *closed* one was a deleted entry that changed
//! the pages with it. A page that quietly omits an entity's lifecycle is indistinguishable from a
//! system that has none, which is the reading this table exists to prevent.
//!
//! # Determinism
//!
//! No clock, no RNG, no `HashMap`. Every list is a `BTreeMap`/`BTreeSet` iteration or a `Vec` in
//! declaration order, and Mermaid node identifiers are indices into those orders rather than hashes.
//! `tests/docs.rs` generates twice and compares bytes, because that is the only form in which this
//! paragraph is worth anything.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use ess_compiler::ir::{
    Driver, EssIr, ResolvedActor, ResolvedBinding, ResolvedBody, ResolvedCommand,
    ResolvedComponent, ResolvedCondition, ResolvedConversion, ResolvedDomain, ResolvedEffect,
    ResolvedEntity, ResolvedError, ResolvedEvent, ResolvedFailure, ResolvedField, ResolvedMapping,
    ResolvedMappingValue, ResolvedSubject, ResolvedType, ResolvedView, ResolvedWorkload,
    TypeHandle,
};
use ess_domain::binding::Delivery;
use ess_domain::command::TestStrategy;
use ess_domain::entity::{Cardinality, Invariant, RelationKind, StateMachine, StateName};
use ess_domain::name::{Naming, QualifiedName};
use ess_domain::refs::ExternalRef;
use ess_domain::view::{AssertionStyle, Consistency, Direction};

use crate::artifact::{Artifact, Generator};
use crate::document::{Block, Blocks, DiagramKind, Document, Inline, Page, PageId, Target};
use crate::graph::{label, SystemGraph};
use ess_compiler::refs::{
    ActorRef, BindingRef, CommandRef, ComponentRef, DeclaredTypeRef, DomainRef, EntityRef,
    ErrorRef, EssSemanticRef, EventRef, ViewRef,
};

use crate::provenance::{ProvenanceMint, SlicedProvenance};

/// Markdown and Mermaid: the cheapest check that every construct can be described.
pub struct Docs;

impl Generator for Docs {
    fn name(&self) -> &'static str {
        "docs"
    }

    fn describes(&self) -> &'static str {
        "Markdown and Mermaid: the cheapest check that every construct can be described"
    }

    fn directory(&self) -> &'static str {
        "docs"
    }

    /// Five kinds of page, and one per bounded context, rendered as Markdown.
    ///
    /// The split follows what a reader arrives with a question about, not what the IR happens to
    /// store: a bounded context is the unit someone reads to learn a vocabulary, the interactions
    /// are the unit someone reads to learn how two contexts meet, and the crossings and the topology
    /// are each a single system-wide question — "what is this system willing to treat as what" and
    /// "what does it need in order to run" — that would be invisible if scattered per domain.
    ///
    /// Every page is built once, as a [`Document`], and handed to one renderer. A second surface
    /// that wants the same pages asks the document for them rather than reading these bytes back.
    fn generate(&self, ir: &EssIr, mint: &ProvenanceMint) -> Vec<Artifact> {
        // The four system-wide pages derive from the whole model, honestly: the index draws the
        // whole graph, the interactions page reads every binding, the crossings and topology pages
        // are each one system-wide question. A domain page derives from its own context — plus the
        // bindings and components, which reach across contexts by design — and says so.
        let mut pages = vec![readme(ir, &mint.whole())];
        for domain in ir.domains().values() {
            pages.push(domain_page(ir, domain, &domain_slice(ir, domain, mint)));
        }
        pages.push(interactions_page(ir, &mint.whole()));
        pages.push(crossings_page(ir, &mint.whole()));
        pages.push(topology_page(ir, &mint.whole()));
        crate::markdown::render(&Document::new(
            ir.system().to_string(),
            ir.version().to_string(),
            pages,
        ))
    }
}

impl Docs {
    /// Every construct this projection knows it cannot render, and what it would take to fix.
    ///
    /// Public because the honest count belongs to whoever is deciding whether the documentation is
    /// trustworthy, and because `tests/docs.rs` asserts the list is exactly this — so a construct
    /// that goes missing without an entry here fails the build instead of vanishing.
    pub const fn known_gaps() -> &'static [Gap] {
        GAPS
    }
}

/// A construct the specification language has and this projection cannot render.
///
/// Each entry is a hole in [`EssIr`], not in this module. Naming them individually is what stops
/// "the documentation never mentions a view" from reading the same as "the system has no views".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gap {
    /// What is missing, named as the specification's author would name it.
    pub construct: &'static str,
    /// What the source says about it that the IR drops.
    pub dropped: &'static str,
    /// Where a reader would have gone looking for it.
    pub page: &'static str,
    /// What would have to change for this projection to render it.
    pub needs: &'static str,
}

/// Nothing: every construct a specification declares reaches the IR, and reaches a page.
///
/// Empty rather than removed. The three entries that were here — entities, views and actors — each
/// named the change in `ess-compiler` that would close it, and each of those changes has since
/// happened: `ResolvedEntity`, `ResolvedView` and `ResolvedActor` are reachable from
/// [`ResolvedDomain`], so the constructs are rendered instead of listed. What is left is the
/// mechanism, which is the part worth keeping: an allowlist, so the next construct the IR drops is a
/// failing test rather than a page that quietly reads like a system without one.
const GAPS: &[Gap] = &[];

// ---- pages ------------------------------------------------------------------------------------

/// The index: what the system is, how it fits together, and where everything else is.
fn readme(ir: &EssIr, provenance: &SlicedProvenance) -> Page {
    let mut blocks = Blocks::new();
    if let Some(summary) = ir.summary() {
        blocks.sentence(summary);
    }

    blocks.push(graph_section(ir));
    blocks.push(contexts_section(ir));
    blocks.push(components_section(ir));
    blocks.push(other_pages_section(ir));

    blocks.extend(gap_blocks(
        "What this projection cannot show",
        "These constructs are in the specification and not in the intermediate representation these \
         pages are generated from, so they cannot appear. They are listed rather than omitted: a \
         page that quietly leaves an entity out reads exactly like a system that has none.",
    ));
    blocks.push(trailing_blank());

    Page {
        id: PageId::from("index"),
        title: vec![Inline::text(format!("{} {}", ir.system(), ir.version()))],
        about: None,
        provenance: provenance.clone(),
        blocks: blocks.finish(),
    }
}

/// The whole system in one picture, with the reading of it a picture cannot give.
fn graph_section(ir: &EssIr) -> Block {
    let mut under = Blocks::new();
    under.push(Block::Diagram {
        kind: DiagramKind::System,
        source: system_graph(ir),
    });
    under.sentence(
        "A command is accepted by the component that owns its context, emits the events one of \
         its outcomes declares, and a dashed edge is a binding carrying an event into the next \
         command. Design §9 begins one step earlier, at the actor who invokes the first command, \
         and so does this graph: a solid edge out of an actor is a grant, and an actor drawn with \
         no edge at all may invoke nothing — which is something the model says, not an arrow \
         somebody forgot.",
    );
    section(
        2,
        vec![Inline::text("The system as a graph")],
        None,
        under.finish(),
    )
}

/// Every bounded context, with the numbers its own page then spells out.
fn contexts_section(ir: &EssIr) -> Block {
    section(
        2,
        vec![Inline::text("Bounded contexts")],
        None,
        vec![bullets(
            ir.domains()
                .values()
                .map(|domain| domain_index_entry(ir, domain))
                .collect(),
        )],
    )
}

/// Every component, and the one thing a component is not.
fn components_section(ir: &EssIr) -> Block {
    let mut under = Blocks::new();
    under.prose(vec![
        Inline::text(
            "A component is a unit of ownership, not a deployment. How many of each runs, and \
             what each needs, is ",
        ),
        Inline::Link {
            to: Target::Page {
                page: PageId::from("topology"),
            },
            text: vec![Inline::text("the topology")],
        },
        Inline::text("."),
    ]);
    for component in ir.components().values() {
        under.prose(component_prose(ir, component));
    }
    section(2, vec![Inline::text("Components")], None, under.finish())
}

/// Where everything else is, as the table a reader scans rather than a paragraph they read.
fn other_pages_section(ir: &EssIr) -> Block {
    let mut rows: Vec<Vec<Vec<Inline>>> = ir
        .domains()
        .values()
        .map(|domain| {
            vec![
                vec![Inline::Link {
                    to: Target::Page {
                        page: domain_page_id(&domain.name),
                    },
                    text: vec![Inline::text(display_of(&domain.naming, &domain.name))],
                }],
                vec![
                    Inline::text("the "),
                    Inline::code(domain.name.to_string()),
                    Inline::text(
                        " vocabulary: its types, entities, views, commands, events, errors and \
                         actors",
                    ),
                ],
            ]
        })
        .collect();
    rows.push(page_row(
        "Interactions",
        "interactions",
        "every binding, with what it guarantees and what happens when it fails",
    ));
    rows.push(page_row(
        "Type crossings",
        "crossings",
        "every conversion this system permits, and the reason someone gave for it",
    ));
    rows.push(page_row(
        "Topology",
        "topology",
        "what each component needs in order to run",
    ));
    section(
        2,
        vec![Inline::text("The other pages")],
        None,
        vec![Block::Table {
            columns: vec![
                vec![Inline::text("page")],
                vec![Inline::text("what is on it")],
            ],
            rows,
        }],
    )
}

/// One row of the index's table of pages.
fn page_row(title: &str, page: &str, about: &str) -> Vec<Vec<Inline>> {
    vec![
        vec![Inline::Link {
            to: Target::Page {
                page: PageId::from(page),
            },
            text: vec![Inline::text(title)],
        }],
        vec![Inline::text(about)],
    ]
}

/// One bounded context: everything declared inside it, in the order a reader needs it.
///
/// Each section is written in terms of the ones above it, and nothing links downwards: types before
/// entities, because an entity is made of them; entities before views, because a view projects one;
/// commands, events and errors next; and actors last, because a grant is a link *up* the page to the
/// command it names. A reader who meets `Money` first does not have to jump.
/// The slice a domain page derives from: the context and everything declared in it, plus every
/// binding and every component.
///
/// The members, not only the domain: membership edges point *at* the context (`type X is declared
/// in domain D`), so a slice seeded at the domain alone would close over nothing the page renders.
/// The bindings and components are included whole because they are the constructs that reach
/// across contexts by design — a binding into this domain's commands appears on this page, and
/// which bindings those are is itself a fact that changes. The cost of the width is a regeneration
/// when an unrelated binding moves; the alternative is a page claiming to stand still while its
/// own crossings section is stale, and those are not comparable errors.
fn domain_slice(ir: &EssIr, domain: &ResolvedDomain, mint: &ProvenanceMint) -> SlicedProvenance {
    let mut seeds: Vec<EssSemanticRef> = vec![DomainRef::new(domain.name.clone()).into()];
    seeds.extend(
        domain
            .types
            .iter()
            .map(|handle| DeclaredTypeRef::from(handle).into()),
    );
    seeds.extend(
        domain
            .entities
            .iter()
            .map(|handle| EntityRef::from(handle).into()),
    );
    seeds.extend(
        domain
            .commands
            .iter()
            .map(|handle| CommandRef::from(handle).into()),
    );
    seeds.extend(
        domain
            .events
            .iter()
            .map(|handle| EventRef::from(handle).into()),
    );
    seeds.extend(
        domain
            .errors
            .iter()
            .map(|handle| ErrorRef::from(handle).into()),
    );
    seeds.extend(
        domain
            .views
            .iter()
            .map(|handle| ViewRef::from(handle).into()),
    );
    seeds.extend(
        domain
            .actors
            .iter()
            .map(|handle| ActorRef::from(handle).into()),
    );
    seeds.extend(
        ir.bindings()
            .keys()
            .map(|name| BindingRef::new(name.clone()).into()),
    );
    seeds.extend(
        ir.components()
            .keys()
            .map(|name| ComponentRef::new(name.clone()).into()),
    );
    mint.of_seeds(seeds)
}

/// The prose one served component publishes about itself.
///
/// The committed domain page for each bounded context the component owns, in name order, and
/// nothing rewritten: for a component owning one domain these are byte for byte the bytes of
/// `generated/docs/domains/{domain}.md`, which is what `tests/docs.rs` holds it to. A served
/// document that was rendered a second way would be a second answer to "what does this system do",
/// and the two would drift on the first edit.
///
/// The relative links inside a page point at siblings this surface does not serve — the index, the
/// interactions page. They are left exactly as the projection wrote them rather than rewritten or
/// stripped, because rewriting them makes the served bytes differ from the committed ones, and that
/// difference is the one thing this document exists to make checkable.
pub fn served(ir: &EssIr, component: &ResolvedComponent) -> String {
    let mint = ProvenanceMint::new(ir);
    let mut out = String::new();
    for handle in &component.owns {
        let domain = ir.domain(handle);
        if !out.is_empty() {
            out.push('\n');
        }
        let page = domain_page(ir, domain, &domain_slice(ir, domain, &mint));
        out.push_str(&crate::markdown::page(&page).contents);
    }
    out
}

fn domain_page(ir: &EssIr, domain: &ResolvedDomain, provenance: &SlicedProvenance) -> Page {
    let mut blocks = Blocks::new();
    if let Some(summary) = &domain.naming.summary {
        blocks.sentence(summary);
    }
    let mut opening = vec![
        Inline::code(domain.name.to_string()),
        Inline::text(format!(" is one of {}'s bounded contexts. ", ir.system())),
    ];
    opening.extend(back_to_the_index());
    blocks.prose(opening);

    blocks.extend(types_section(ir, domain));
    blocks.extend(entities_section(ir, domain));
    blocks.extend(views_section(ir, domain));
    blocks.extend(commands_section(ir, domain));
    blocks.extend(events_section(ir, domain));
    blocks.extend(errors_section(ir, domain));
    blocks.extend(actors_section(ir, domain));
    blocks.extend(crossings_section(ir, domain));

    blocks.extend(gap_blocks(
        "What this page cannot show",
        "This context declares more than appears above. What is missing is missing from the \
         intermediate representation this page is generated from, not from the specification.",
    ));
    blocks.push(trailing_blank());

    Page {
        id: domain_page_id(&domain.name),
        title: vec![Inline::text(display_of(&domain.naming, &domain.name))],
        about: Some(DomainRef::new(domain.name.clone()).into()),
        provenance: provenance.clone(),
        blocks: blocks.finish(),
    }
}

/// Every binding: what it reacts to, what it invokes, and what it promises while doing so.
fn interactions_page(ir: &EssIr, provenance: &SlicedProvenance) -> Page {
    let mut blocks = Blocks::new();
    blocks.sentence(
        "A binding is the only way an event in one context causes a command in another. Each one \
         states how many times the command may run and what happens when it does not, because a \
         binding that can fail quietly is the difference between specifying a system and specifying \
         a demo.",
    );
    blocks.prose(back_to_the_index());

    if ir.bindings().is_empty() {
        blocks.sentence("This system declares no bindings: nothing here reacts to anything.");
    }
    for binding in ir.bindings().values() {
        blocks.push(binding_section(ir, binding));
    }

    let unread = unread_events(ir);
    if !unread.is_empty() {
        let mut under = Blocks::new();
        under.sentence(
            "Legal, and worth seeing. An event with no reader inside the system is either a \
             deliberate boundary — something outside consumes it — or a binding somebody forgot, \
             and only a person can tell which.",
        );
        under.push(bullets(names(unread)));
        blocks.push(section(
            2,
            vec![Inline::text("Events nothing reacts to")],
            None,
            under.finish(),
        ));
    }
    blocks.push(trailing_blank());

    Page {
        id: PageId::from("interactions"),
        title: vec![Inline::text("Interactions")],
        about: None,
        provenance: provenance.clone(),
        blocks: blocks.finish(),
    }
}

/// Every declared conversion, with the reason attached to it.
///
/// A page of its own, and linked from the index by name, because this is where an audit lands. The
/// same reason is repeated at each point of use — on the binding that relies on it, and on the pages
/// of both contexts whose types it joins — so that a reader who never thought to ask "what may be
/// treated as what here" still meets the answer beside the type it concerns.
fn crossings_page(ir: &EssIr, provenance: &SlicedProvenance) -> Page {
    let mut blocks = Blocks::new();
    blocks.sentence(
        "A conversion is this system's permission for a value of one type to be used as another. \
         Every one of them carries a reason, and the reason is required rather than optional \
         precisely so that this page can exist: someone asking why an invoice's email address is \
         allowed to become a mailbox address gets an answer written by the person who allowed it, \
         not a shrug.",
    );
    blocks.prose(vec![
        Inline::text("Declaring a crossing is also the only way to make one. Two newtypes over "),
        Inline::code("String"),
        Inline::text(
            " do not convert because they are both strings; they convert because a line in the \
             specification says they may.",
        ),
    ]);

    if ir.conversions().is_empty() {
        blocks.sentence("This system declares no crossings. Every type is used only as itself.");
    }
    for conversion in ir.conversions() {
        let title = vec![
            Inline::code(conversion.from.to_string()),
            Inline::text(" may be used as "),
            Inline::code(conversion.to.to_string()),
        ];
        let mut under = Blocks::new();
        under.push(quoted_reason(&conversion.because));
        let users = crossing_users(ir, conversion);
        if users.is_empty() {
            under.sentence(
                "Nothing uses this crossing yet. It is still part of what the system permits, \
                 which is why it is written down.",
            );
        } else {
            under.sentence("Relied on by:");
            under.push(bullets(users));
        }
        blocks.push(Block::Section {
            level: 2,
            anchor: anchor_of(&title),
            title,
            // A crossing is a pair of types and the sentence joining them; the IR names no
            // construct for it, so there is nothing honest to point a consumer at.
            about: None,
            blocks: under.finish(),
        });
    }

    blocks.prose(back_to_the_index());
    Page {
        id: PageId::from("crossings"),
        title: vec![Inline::text("Type crossings")],
        about: None,
        provenance: provenance.clone(),
        blocks: blocks.finish(),
    }
}

/// What each component needs in order to run, and what a replica floor is claiming.
fn topology_page(ir: &EssIr, provenance: &SlicedProvenance) -> Page {
    let mut blocks = Blocks::new();
    blocks.sentence(
        "Runtime requirements, stated semantically. None of this is a deployment and nothing \
         generates a manifest from it: a replica floor of two is a claim that the system is not \
         correct with one instance, which is a fact about the design and survives every change of \
         hosting.",
    );

    for workload in ir.workloads().values() {
        let component = ir.component(&workload.component);
        let mut under = Blocks::new();
        if let Some(summary) = &component.naming.summary {
            under.sentence(summary);
        }
        under.sentence(replicas_sentence(workload));
        under.sentence(stateless_sentence(workload));
        if workload.requires.is_empty() {
            under.sentence("It requires nothing beyond itself.");
        } else {
            under.sentence("It requires:");
            under.push(bullets(
                workload
                    .requires
                    .iter()
                    .map(|resource| {
                        vec![
                            Inline::code(resource.kind.clone()),
                            Inline::text(" — "),
                            Inline::code(resource.name.clone()),
                        ]
                    })
                    .collect(),
            ));
        }
        blocks.push(section(
            2,
            vec![Inline::code(component.name.to_string())],
            Some(ComponentRef::new(component.name.clone()).into()),
            under.finish(),
        ));
    }

    let idle: Vec<_> = ir
        .components()
        .keys()
        .filter(|name| !ir.workloads().contains_key(*name))
        .collect();
    if !idle.is_empty() {
        let mut under = Blocks::new();
        under.sentence(
            "Declared as a unit of ownership, with nothing in the topology running it. That is \
             legal — a context can be owned by a library — but it is the kind of legal worth \
             reading twice.",
        );
        under.push(bullets(names(idle)));
        blocks.push(section(
            2,
            vec![Inline::text("Components that run nowhere")],
            None,
            under.finish(),
        ));
    }

    blocks.prose(back_to_the_index());
    Page {
        id: PageId::from("topology"),
        title: vec![Inline::text("Topology")],
        about: None,
        provenance: provenance.clone(),
        blocks: blocks.finish(),
    }
}

/// The link every page ends with.
///
/// A target and not a path. Which file the index is, and how far up it sits from here, is the
/// renderer's business — this says only which page is meant.
fn back_to_the_index() -> Vec<Inline> {
    vec![
        Inline::Link {
            to: Target::Page {
                page: PageId::from("index"),
            },
            text: vec![Inline::text("Back to the index")],
        },
        Inline::text("."),
    ]
}

// ---- sections ---------------------------------------------------------------------------------

/// The types an author declared in a context, one paragraph each.
///
/// An entity's state enum is left out because it is not an author's declaration: the compiler
/// synthesises it from a lifecycle, and it is rendered with that lifecycle, in the entity's own
/// section. Found by comparing handles with [`ResolvedEntity::state_type`] rather than by reading
/// `State` out of a name, because a name read for meaning is an identity used as a key.
fn types_section(ir: &EssIr, domain: &ResolvedDomain) -> Vec<Block> {
    let synthesised = state_types(ir, domain);
    let declared: Vec<(&TypeHandle, &ResolvedType)> = domain
        .types
        .iter()
        .filter(|handle| !synthesised.contains(*handle))
        .map(|handle| (handle, ir.named_type(handle)))
        .collect();
    if declared.is_empty() {
        return Vec::new();
    }
    let mut under = Blocks::new();
    for (handle, named) in &declared {
        under.push(section(
            3,
            vec![Inline::code(relative(&named.name, &domain.name))],
            Some(DeclaredTypeRef::from(*handle).into()),
            type_prose(named),
        ));
    }
    let types: Vec<&ResolvedType> = declared.iter().map(|(_, named)| *named).collect();
    if let Some(note) = orphan_note(ir, &types) {
        under.prose(note);
    }
    vec![section(
        2,
        vec![Inline::text("Types")],
        None,
        under.finish(),
    )]
}

/// The types nothing else in the IR mentions.
///
/// Worth a paragraph rather than a silent omission, and worth *only* a paragraph: nothing declares a
/// field of one of these, which makes it either vocabulary something outside this specification
/// uses or a leftover, and only a person can tell which. The one reading it must not invite is
/// "reached through a construct the projection dropped" — every construct that reaches a type
/// (entity, view, command, event, error, crossing) is counted below, so an orphan here is an orphan
/// in the model.
fn orphan_note(ir: &EssIr, declared: &[&ResolvedType]) -> Option<Vec<Inline>> {
    let referenced = referenced_types(ir);
    let orphans: Vec<Vec<Inline>> = declared
        .iter()
        .filter(|it| !referenced.contains(&it.name))
        .map(|it| vec![Inline::code(it.name.to_string())])
        .collect();
    if orphans.is_empty() {
        return None;
    }
    let count = orphans.len();
    let mut out = vec![Inline::text(format!(
        "{} of the types above {} reached by nothing else in this system: ",
        capitalise(&number(count)),
        if count == 1 { "is" } else { "are" },
    ))];
    out.extend(inline_list(orphans));
    out.push(Inline::text(format!(
        ". No entity, view, command, event, error or crossing names {}, so it is either vocabulary \
         something outside this specification uses or a leftover — and only a person can tell \
         which.",
        if count == 1 { "it" } else { "them" }
    )));
    Some(out)
}

/// Every named type reached from a field, an input, a payload, a union variant or a crossing.
///
/// An entity's fields and a view's projected fields are in here, because they are the reason most
/// types exist: leaving them out would report `LineItem` as reached by nothing while the page above
/// draws it inside `Invoice`. An entity's state enum is deliberately not counted as a reference —
/// nothing *names* it, the compiler makes it.
fn referenced_types(ir: &EssIr) -> BTreeSet<QualifiedName> {
    let mut out = BTreeSet::new();
    let mut note = |reference: &ess_compiler::ir::ResolvedTypeRef| {
        for handle in reference.named_leaves() {
            out.insert(handle.name().clone());
        }
    };
    for declared in ir.types().values() {
        match &declared.body {
            ResolvedBody::Newtype { of, .. } => note(of),
            ResolvedBody::Struct { fields, .. } => {
                for field in fields {
                    note(&field.type_ref);
                }
            }
            ResolvedBody::Enum { .. } => {}
            ResolvedBody::Union { variants, .. } => {
                for variant in variants.values() {
                    note(variant);
                }
            }
        }
    }
    for entity in ir.entities().values() {
        note(&entity.identity.type_ref);
        for field in &entity.fields {
            note(&field.type_ref);
        }
    }
    for view in ir.views().values() {
        for field in &view.fields {
            note(&field.type_ref);
        }
    }
    for command in ir.commands().values() {
        for field in &command.input {
            note(&field.type_ref);
        }
    }
    for event in ir.events().values() {
        for field in &event.fields {
            note(&field.type_ref);
        }
    }
    for error in ir.errors().values() {
        for field in &error.fields {
            note(&field.type_ref);
        }
    }
    for conversion in ir.conversions() {
        note(&conversion.from);
        note(&conversion.to);
    }
    out
}

/// Every entity: what identifies it, what it holds, what stays true, and where it may move.
///
/// The lifecycle is the part that cannot be a table. `Paid` not becoming `Cancelled` is expressed by
/// the *absence* of a transition, so the section carries three things a list of states cannot: the
/// diagram, the initial and terminal states, and — because absence does not draw — the pairs no move
/// connects.
fn entities_section(ir: &EssIr, domain: &ResolvedDomain) -> Vec<Block> {
    if domain.entities.is_empty() {
        return Vec::new();
    }
    let projections = ir.projections();
    let drivers = ir.drivers();
    let mut under = Blocks::new();
    under.sentence(
        "An entity is what this context is about: something with an identity that outlives any one \
         request, a shape, and a lifecycle. The lifecycle is exhaustive — a move that is not drawn \
         below is a move this specification does not permit, and that is the only way it says so. \
         Every move is labelled with the command that takes it, because a move nothing can trigger \
         is refused rather than drawn.",
    );
    for handle in &domain.entities {
        let entity = ir.entity(handle);
        let mut about = Blocks::new();
        about.prose(naming_sentence(&entity.naming, &entity.name));
        about.prose(identity_sentence(entity));
        if entity.fields.is_empty() {
            about.sentence("It holds nothing beyond its identity and its state.");
        } else {
            about.sentence("It holds:");
            about.push(bullets(entity.fields.iter().map(field_bullet).collect()));
        }
        about.prose(relations_sentence(ir, domain, entity));
        about.prose(entity_invariants_sentence(entity));
        about.prose(state_type_sentence(entity));
        about.prose(resting_sentence(&entity.lifecycle));
        let driven = drivers.get(handle).map_or(&[][..], Vec::as_slice);
        about.push(Block::Diagram {
            kind: DiagramKind::Lifecycle,
            source: state_diagram(&entity.lifecycle, driven),
        });
        about.extend(driven_blocks(&entity.lifecycle, driven));
        about.extend(legality_note(&entity.lifecycle));
        about.prose(observed_by_sentence(ir, domain, projections.get(handle)));
        under.push(section(
            3,
            vec![Inline::code(relative(&entity.name, &domain.name))],
            Some(EntityRef::from(handle).into()),
            about.finish(),
        ));
    }
    vec![section(
        2,
        vec![Inline::text("Entities")],
        None,
        under.finish(),
    )]
}

/// What one entity owns, what it names, and what owns it.
///
/// Both directions on one page, because a relation is declared on one end and read from both: a
/// reader of the invoice wants to know whose it is, and that fact is written in the account's
/// declaration. Nothing here is inferred — the reverse direction is a lookup over the declarations,
/// which is what `EssIr::relations_carried_by` answers.
fn relations_sentence(ir: &EssIr, domain: &ResolvedDomain, entity: &ResolvedEntity) -> Vec<Inline> {
    let mut sentences: Vec<Vec<Inline>> = Vec::new();

    for relation in &entity.relations {
        let target = ir.entity(&relation.target);
        sentences.push(vec![
            Inline::text(format!(
                "It {} {} ",
                relation.kind,
                match relation.cardinality {
                    Cardinality::One => "at most one",
                    Cardinality::Many => "any number of",
                }
            )),
            section_link(ir, domain, &target.name, &target.domain),
            Inline::text(", as "),
            Inline::code(relation.name.clone()),
            Inline::text(", carried by "),
            Inline::code(format!(
                "{}.{}",
                relative(
                    match relation.kind {
                        RelationKind::Owns => &target.name,
                        RelationKind::References => &entity.name,
                    },
                    &domain.name
                ),
                relation.via
            )),
            Inline::text("."),
        ]);
    }

    for (field, carried) in ir.relations_carried_by(&entity.name) {
        if carried.source == &entity.name {
            continue;
        }
        let source = &ir.entities()[carried.source];
        sentences.push(vec![
            Inline::text("Its "),
            Inline::code(field.to_string()),
            Inline::text(" is what "),
            section_link(ir, domain, &source.name, &source.domain),
            Inline::text(format!(" {} it by, as ", carried.relation.kind)),
            Inline::code(carried.relation.name.clone()),
            Inline::text("."),
        ]);
    }

    if sentences.is_empty() {
        return vec![Inline::text(
            "It declares no relation to another entity, and no other entity names it.",
        )];
    }
    let mut out = Vec::new();
    for (position, sentence) in sentences.into_iter().enumerate() {
        if position > 0 {
            out.push(Inline::text(" "));
        }
        out.extend(sentence);
    }
    out
}

/// Every view: what it reads, which instances it holds, and how soon it holds them.
fn views_section(ir: &EssIr, domain: &ResolvedDomain) -> Vec<Block> {
    if domain.views.is_empty() {
        return Vec::new();
    }
    let mut under = Blocks::new();
    under.sentence(
        "A view is what the outside world is promised it can observe. Each one says which instances \
         it contains and how soon it reflects a command that has already returned, because \"you \
         can read this\" without \"how soon\" is the promise every flaky suite is built on.",
    );
    for handle in &domain.views {
        let view = ir.view(handle);
        let source = ir.entity(&view.source);
        let mut about = Blocks::new();
        about.prose(naming_sentence(&view.naming, &view.name));
        about.prose(vec![
            Inline::text("It reads "),
            section_link(ir, domain, &source.name, &source.domain),
            Inline::text("."),
        ]);
        about.prose(filter_sentence(view));
        if view.fields.is_empty() {
            about.sentence(
                "It exposes no fields, so it answers \"does an instance match\" and nothing about \
                 the instance.",
            );
        } else {
            about.sentence("It exposes:");
            about.push(bullets(view.fields.iter().map(field_bullet).collect()));
        }
        about.prose(order_sentence(view));
        about.prose(consistency_sentence(view.consistency));
        about.sentence(assertion_sentence(view.assertion_style));
        under.push(section(
            3,
            vec![Inline::code(relative(&view.name, &domain.name))],
            Some(ViewRef::from(handle).into()),
            about.finish(),
        ));
    }
    vec![section(
        2,
        vec![Inline::text("Views")],
        None,
        under.finish(),
    )]
}

/// The records a construct names, where it names any.
///
/// Written as the shorthand and never as a link. The address is built from the project's own
/// `providers:` map, which this projection does not read: a page that guessed a host would carry a
/// link that opens the wrong page, and a wrong link cannot be told from a right one by looking at
/// it.
fn refs_sentence(refs: &[ExternalRef]) -> Option<Vec<Inline>> {
    if refs.is_empty() {
        return None;
    }
    let mut out = vec![Inline::text("Recorded at ")];
    for (position, reference) in refs.iter().enumerate() {
        if position > 0 {
            out.push(Inline::text(", "));
        }
        out.push(Inline::code(reference.to_string()));
    }
    out.push(Inline::text("."));
    Some(out)
}

/// What order the rows come back in, or that the view does not say.
///
/// Absence is written down rather than left out. A reader who is not told is entitled to assume
/// there is an order and that it is the obvious one, and a view named for a position is exactly
/// where that assumption gets made.
fn order_sentence(view: &ResolvedView) -> Vec<Inline> {
    if view.order_by.is_empty() {
        return vec![Inline::text(
            "It declares no order, so the rows come back in whatever order the implementation has, \
             and two reads may disagree.",
        )];
    }
    let mut out = vec![Inline::text("Its rows are ordered by ")];
    // `a`, then `b`, then `c` — the ranking keys in significance order, which is not an English
    // list: `and` between two keys would read as though either one of them decided the order.
    for (position, ranking) in view.order_by.iter().enumerate() {
        if position > 0 {
            out.push(Inline::text(", then "));
        }
        out.push(Inline::code(ranking.field.clone()));
        out.push(Inline::text(match ranking.direction {
            Direction::Ascending => " ascending",
            Direction::Descending => " descending",
        }));
    }
    out.push(Inline::text("."));
    out
}

/// Every actor, and the commands each of them may invoke.
fn actors_section(ir: &EssIr, domain: &ResolvedDomain) -> Vec<Block> {
    if domain.actors.is_empty() {
        return Vec::new();
    }
    let mut under = Blocks::new();
    under.sentence(
        "An actor is who may ask this context for something. Every grant below points at a command \
         this specification declares — a grant is a resolved reference, so \"may invoke\" something \
         nobody wrote is not a permission this model can express, and an authorisation that \
         authorises nothing cannot ship quietly.",
    );
    for handle in &domain.actors {
        let actor = ir.actor(handle);
        let mut about = Blocks::new();
        about.prose(naming_sentence(&actor.naming, &actor.name));
        about.prose(grants_sentence(ir, domain, actor));
        under.push(section(
            3,
            vec![Inline::code(relative(&actor.name, &domain.name))],
            Some(ActorRef::from(handle).into()),
            about.finish(),
        ));
    }
    vec![section(
        2,
        vec![Inline::text("Actors")],
        None,
        under.finish(),
    )]
}

/// Every command, with its input and — the part that matters — every outcome.
fn commands_section(ir: &EssIr, domain: &ResolvedDomain) -> Vec<Block> {
    if domain.commands.is_empty() {
        return Vec::new();
    }
    let mut under = Blocks::new();
    for handle in &domain.commands {
        let command = ir.command(handle);
        let mut about = Blocks::new();
        about.prose(naming_sentence(&command.naming, &command.name));
        if let Some(recorded) = refs_sentence(&command.refs) {
            about.prose(recorded);
        }
        if command.input.is_empty() {
            about.sentence("It takes no input.");
        } else {
            about.sentence("It takes:");
            about.push(bullets(command.input.iter().map(field_bullet).collect()));
        }
        about.prose(outcome_count_sentence(
            command.outcomes.len(),
            &command.name,
        ));
        for outcome in &command.outcomes {
            about.prose(outcome_prose(ir, command, outcome));
        }
        under.push(section(
            3,
            vec![Inline::code(relative(&command.name, &domain.name))],
            Some(CommandRef::from(handle).into()),
            about.finish(),
        ));
    }
    vec![section(
        2,
        vec![Inline::text("Commands")],
        None,
        under.finish(),
    )]
}

/// Every event, what it carries, and who causes and reads it.
fn events_section(ir: &EssIr, domain: &ResolvedDomain) -> Vec<Block> {
    if domain.events.is_empty() {
        return Vec::new();
    }
    let reactions = ir.reactions();
    let mut under = Blocks::new();
    for handle in &domain.events {
        let event = ir.event(handle);
        let mut about = Blocks::new();
        about.prose(naming_sentence(&event.naming, &event.name));
        if event.fields.is_empty() {
            about.sentence("It carries nothing: the fact that it happened is the whole payload.");
        } else {
            about.sentence("It carries:");
            about.push(bullets(event.fields.iter().map(field_bullet).collect()));
        }
        for sentence in emitters(ir, event) {
            about.prose(sentence);
        }
        match reactions.get(handle) {
            None => {
                about.sentence("Nothing in this system reacts to it.");
            }
            Some(bindings) => {
                let mut reacts = inline_list(
                    bindings
                        .iter()
                        .map(|it| vec![Inline::code(it.name.to_string())])
                        .collect(),
                );
                reacts.push(Inline::text(" reacts to it — see "));
                reacts.push(Inline::Link {
                    to: Target::Page {
                        page: PageId::from("interactions"),
                    },
                    text: vec![Inline::text("Interactions")],
                });
                reacts.push(Inline::text("."));
                about.prose(reacts);
            }
        }
        under.push(section(
            3,
            vec![Inline::code(relative(&event.name, &domain.name))],
            Some(EventRef::from(handle).into()),
            about.finish(),
        ));
    }
    vec![section(
        2,
        vec![Inline::text("Events")],
        None,
        under.finish(),
    )]
}

/// Every error, what it carries, and which branch reports it.
fn errors_section(ir: &EssIr, domain: &ResolvedDomain) -> Vec<Block> {
    if domain.errors.is_empty() {
        return Vec::new();
    }
    let mut under = Blocks::new();
    for handle in &domain.errors {
        let error = ir.error(handle);
        let mut about = Blocks::new();
        if let Some(summary) = &error.summary {
            about.sentence(summary);
        }
        if error.fields.is_empty() {
            about.sentence(
                "It carries nothing beyond its name, so a caller can tell what went wrong and not \
                 which value caused it.",
            );
        } else {
            about.sentence("It carries:");
            about.push(bullets(error.fields.iter().map(field_bullet).collect()));
        }
        for sentence in reporters(ir, error) {
            about.prose(sentence);
        }
        under.push(section(
            3,
            vec![Inline::code(relative(&error.name, &domain.name))],
            Some(ErrorRef::from(handle).into()),
            about.finish(),
        ));
    }
    vec![section(
        2,
        vec![Inline::text("Errors")],
        None,
        under.finish(),
    )]
}

/// The crossings with an end in this context, repeated here on purpose.
///
/// This is the answer to "where does a conversion's reason go so that someone finds it without
/// knowing to look": beside the type. A reader on this page is reading about `Email`; that is where
/// the sentence saying `Email` may become somebody else's address has to be.
fn crossings_section(ir: &EssIr, domain: &ResolvedDomain) -> Vec<Block> {
    let relevant: Vec<_> = ir
        .conversions()
        .iter()
        .filter(|conversion| {
            touches(&conversion.from, &domain.name) || touches(&conversion.to, &domain.name)
        })
        .collect();
    if relevant.is_empty() {
        return Vec::new();
    }
    let mut under = Blocks::new();
    under.sentence(
        "Types in this context that the specification permits to be used as another type, or the \
         other way round. Nothing else crosses: two newtypes over the same primitive stay distinct \
         until a line in the specification says otherwise.",
    );
    for conversion in relevant {
        under.prose(vec![
            Inline::Strong {
                text: vec![
                    Inline::code(conversion.from.to_string()),
                    Inline::text(" may be used as "),
                    Inline::code(conversion.to.to_string()),
                ],
            },
            Inline::text(", because:"),
        ]);
        under.push(quoted_reason(&conversion.because));
    }
    under.prose(vec![
        Inline::text("Every crossing in the system is on one page: "),
        Inline::Link {
            to: Target::Page {
                page: PageId::from("crossings"),
            },
            text: vec![Inline::text("Type crossings")],
        },
        Inline::text("."),
    ]);
    vec![section(
        2,
        vec![Inline::text("Type crossings")],
        None,
        under.finish(),
    )]
}

/// One binding, its guarantees in prose, its mapping, and the flow a table cannot show.
fn binding_section(ir: &EssIr, binding: &ResolvedBinding) -> Block {
    let event = ir.event(&binding.event);
    let command = ir.command(&binding.command);
    let owner = ir.domain(&command.domain);
    let mut under = Blocks::new();
    if let Some(summary) = &binding.naming.summary {
        under.sentence(summary);
    }
    if let Some(recorded) = refs_sentence(&binding.refs) {
        under.prose(recorded);
    }
    under.prose(vec![
        Inline::code(event.name.to_string()),
        Inline::text(" causes "),
        Inline::code_link(
            Target::Anchor {
                page: domain_page_id(&owner.name),
                anchor: slug(&relative(&command.name, &owner.name)),
            },
            command.name.to_string(),
        ),
        Inline::text("."),
    ]);

    under.push(Block::Diagram {
        kind: DiagramKind::BindingFlow,
        source: binding_flow(ir, binding),
    });

    under.prose(delivery_sentence(binding.delivery, command));
    under.prose(failure_sentence(ir, binding));

    if binding.mapping.is_empty() {
        under.sentence(
            "It fills none of the command's input: every value the command needs has to come from \
             somewhere else.",
        );
    } else {
        under.sentence("It fills the command's input like this:");
        under.push(bullets(
            binding.mapping.iter().map(mapping_bullet).collect(),
        ));
    }

    section(
        2,
        vec![Inline::code(binding.name.to_string())],
        Some(BindingRef::new(binding.name.clone()).into()),
        under.finish(),
    )
}

// ---- prose ------------------------------------------------------------------------------------

/// A named type as a sentence, because its shape is one fact and a table of one fact is furniture.
fn type_prose(declared: &ResolvedType) -> Vec<Block> {
    let name = Inline::code(declared.name.to_string());
    let mut out = Blocks::new();
    match &declared.body {
        ResolvedBody::Newtype { of, invariants } => {
            let mut text = vec![
                name,
                Inline::text(" wraps "),
                Inline::code(of.to_string()),
                Inline::text(
                    " and is not interchangeable with one: the whole value of naming it separately \
                     is the crossings the model then refuses.",
                ),
            ];
            let clause = invariants_clause(invariants);
            if !clause.is_empty() {
                text.push(Inline::text(" "));
                text.extend(clause);
            }
            out.prose(text);
        }
        ResolvedBody::Struct { fields, invariants } => {
            out.prose(vec![
                name,
                Inline::text(format!(
                    " is a record of {}:",
                    plural(fields.len(), "field")
                )),
            ]);
            out.push(bullets(fields.iter().map(field_bullet).collect()));
            let clause = invariants_clause(invariants);
            if !clause.is_empty() {
                out.prose(clause);
            }
        }
        ResolvedBody::Enum { variants } => {
            let mut text = vec![name, Inline::text(" is one of ")];
            text.extend(inline_list(names(variants)));
            text.push(Inline::text("."));
            out.prose(text);
        }
        ResolvedBody::Union { tag, variants } => {
            out.prose(vec![
                name,
                Inline::text(format!(
                    " is one of {}, told apart by a ",
                    plural(variants.len(), "shape")
                )),
                Inline::code(tag.clone()),
                Inline::text(
                    " field — tagged, so a decoder never has to guess which branch it is reading:",
                ),
            ]);
            out.push(bullets(
                variants
                    .iter()
                    .map(|(variant, type_ref)| {
                        vec![
                            Inline::code(variant.clone()),
                            Inline::text(" — "),
                            Inline::code(type_ref.to_string()),
                        ]
                    })
                    .collect(),
            ));
        }
    }
    if let Some(display) = &declared.naming.display {
        out.sentence(format!("Shown to a person as \"{display}\"."));
    }
    out.finish()
}

/// One outcome, including the two things a name alone loses: what decides it, and what it costs.
fn outcome_prose(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ess_compiler::ir::ResolvedOutcome,
) -> Vec<Inline> {
    let mut out = vec![
        Inline::Strong {
            text: vec![Inline::code(outcome.name.to_string())],
        },
        Inline::text(" — "),
    ];
    if let Some(summary) = &outcome.summary {
        out.push(Inline::text(format!("{summary} ")));
    }
    out.extend(condition_sentence(ir, command, &outcome.condition));
    out.push(Inline::text(" "));
    out.extend(effect_sentence(ir, outcome.subject.as_ref()));
    if let Some(error) = &outcome.error {
        let reported = ir.error(error);
        out.push(Inline::text(" It reports "));
        out.push(Inline::code(reported.name.to_string()));
        if reported.fields.is_empty() {
            out.push(Inline::text("."));
        } else {
            out.push(Inline::text(", carrying "));
            out.extend(inline_list(
                reported
                    .fields
                    .iter()
                    .map(|it| vec![Inline::code(it.name.clone())])
                    .collect(),
            ));
            out.push(Inline::text("."));
        }
    }
    match outcome.emits.as_slice() {
        [] => out.push(Inline::text(" It emits nothing.")),
        emitted => {
            out.push(Inline::text(" It emits "));
            out.extend(inline_list(names(emitted)));
            out.push(Inline::text("."));
        }
    }
    out.push(Inline::text(format!(
        " {}",
        strategy_sentence(outcome.test_strategy)
    )));
    out
}

/// What this branch does to an entity, including the case where it does nothing.
///
/// Written for every outcome and not only for the ones with a subject, because silence is the one
/// answer a reader cannot interpret: "this branch changes no entity" and "the projection dropped the
/// field" look identical on a page, and the first is a fact about the system.
fn effect_sentence(ir: &EssIr, subject: Option<&ResolvedSubject>) -> Vec<Inline> {
    let Some(subject) = subject else {
        return vec![Inline::text("No entity in this specification changes.")];
    };
    let entity = ir.entity(&subject.entity);
    let mut out = match &subject.effect {
        ResolvedEffect::Creates => vec![
            Inline::text("It creates a "),
            Inline::code(entity.name.to_string()),
            Inline::text(", which starts in "),
            Inline::code(entity.lifecycle.initial.to_string()),
            Inline::text("."),
        ],
        ResolvedEffect::Moves { transition } => {
            let mut moved = vec![
                Inline::text("It moves a "),
                Inline::code(entity.name.to_string()),
                Inline::text(" from "),
            ];
            moved.extend(inline_list(names(&transition.from)));
            moved.push(Inline::text(" to "));
            moved.push(Inline::code(transition.to.to_string()));
            moved.push(Inline::text(", along the declared move "));
            moved.push(Inline::code(transition.name.clone()));
            moved.push(Inline::text("."));
            moved
        }
        ResolvedEffect::Updates => vec![
            Inline::text("It changes a "),
            Inline::code(entity.name.to_string()),
            Inline::text(" without moving it along its lifecycle."),
        ],
    };
    out.push(Inline::text(" "));
    out.extend(instance_sentence(ir, subject));
    out
}

/// Which instance the branch acts on, and where a reader finds its identity.
///
/// A page that says an invoice moved and not *which* invoice describes a system nobody can call.
/// The two sentences differ because the two surfaces do: an existing instance is named by the caller
/// in the request, and a new one is announced by the event the branch emits, because it did not
/// exist when the request was made.
fn instance_sentence(ir: &EssIr, subject: &ResolvedSubject) -> Vec<Inline> {
    let field = subject.instance.field();
    match subject.instance.event() {
        None => vec![
            Inline::text("The instance is the one named by the input field "),
            Inline::code(field.name.clone()),
            Inline::text("."),
        ],
        Some(event) => vec![
            Inline::text("The new instance's identity is published as "),
            Inline::code(field.name.clone()),
            Inline::text(" on "),
            Inline::code(ir.event(event).name.to_string()),
            Inline::text("."),
        ],
    }
}

/// What decides that an outcome is the one taken.
///
/// [`ResolvedCondition::WrongState`] is the one case that reads the rest of the specification, and
/// deliberately so: the document does not say which states the branch answers in, because the
/// transitions already do. A page that printed only "the subject is in the wrong state" would leave
/// a reader to do that subtraction by hand across a lifecycle and a command, which is exactly the
/// work [`EssIr::wrong_states`] exists to have already done.
fn condition_sentence(
    ir: &EssIr,
    command: &ResolvedCommand,
    condition: &ResolvedCondition,
) -> Vec<Inline> {
    match condition {
        ResolvedCondition::When { predicate } => vec![
            Inline::text("Taken when "),
            Inline::code(predicate.to_string()),
            Inline::text(" holds of the input."),
        ],
        ResolvedCondition::Otherwise => vec![Inline::text(
            "The default branch, taken when no other outcome's condition matched.",
        )],
        ResolvedCondition::External { cause } => vec![
            Inline::text(format!(
                "Decided outside the input: {cause}. No predicate over the input reaches this \
                 branch, and saying "
            )),
            Inline::code("when: false"),
            Inline::text(
                " instead would have claimed it is unreachable, which is a different and false \
                 statement.",
            ),
        ],
        ResolvedCondition::WrongState => {
            let mut out = vec![Inline::text(
                "Taken when the subject is resting in a state none of this command's moves start \
                 from",
            )];
            for (handle, states) in ir.wrong_states(command) {
                out.push(Inline::text(" — a "));
                out.push(Inline::code(ir.entity(handle).name.to_string()));
                out.push(Inline::text(" in "));
                out.extend(inline_list(names(states)));
            }
            out.push(Inline::text(
                ", which is what is left of the lifecycle once this command's own moves are taken \
                 away. The document lists none of it.",
            ));
            out
        }
    }
}

/// How a generated test is meant to reach a branch.
///
/// On the page because the specification computes it once, on the model, so that no two projections
/// can disagree about whether a branch can be reached by constructing an input.
fn strategy_sentence(strategy: TestStrategy) -> &'static str {
    match strategy {
        TestStrategy::ConstructInput => {
            "A test reaches it by constructing an input that satisfies that condition."
        }
        TestStrategy::DefaultBranch => {
            "A test reaches it by constructing an input that satisfies no other outcome's condition."
        }
        TestStrategy::InjectFault => {
            "A test reaches it by injecting the declared fault, because no input can."
        }
        TestStrategy::ArrangeState => {
            "A test reaches it by driving an instance into one of those states and then issuing the \
             command, because no input selects this branch."
        }
    }
}

/// How many times the command may run, and what that obliges the command to be.
fn delivery_sentence(delivery: Delivery, command: &ResolvedCommand) -> Vec<Inline> {
    match delivery {
        Delivery::AtLeastOnce => vec![
            Inline::text("Delivered "),
            Inline::Strong {
                text: vec![Inline::text("at least once")],
            },
            Inline::text(", so "),
            Inline::code(command.name.to_string()),
            Inline::text(
                " must be idempotent: the same event arriving twice must not do the work twice. \
                 \"Exactly once\" is what everyone believes they have until a retry proves \
                 otherwise, which is why this is written down rather than assumed.",
            ),
        ],
    }
}

/// What happens when the command does not run, and how a reader could tell that it did.
///
/// The escalation's event is named rather than left as "surfaced to a person somehow", because the
/// page is what a conformance target is written against: a sentence that names no observable
/// describes a requirement nobody can be asked to prove.
fn failure_sentence(ir: &EssIr, binding: &ResolvedBinding) -> Vec<Inline> {
    match binding.on_failure() {
        ResolvedFailure::Retry => vec![
            Inline::text("When it fails it is "),
            Inline::Strong {
                text: vec![Inline::text("retried")],
            },
            Inline::text(
                ", on whatever schedule the transport provides. Nothing here says how many times, \
                 so nothing here says when it stops. A retry publishes nothing of its own, because \
                 it is already observable: it is another invocation of the command.",
            ),
        ],
        ResolvedFailure::Escalate { emits } => vec![
            Inline::text("When it fails it is "),
            Inline::Strong {
                text: vec![Inline::text("escalated")],
            },
            Inline::text(
                " — surfaced to a person, who decides what happens next — and the system publishes ",
            ),
            Inline::code(ir.event(emits).name.to_string()),
            Inline::text(
                " to say so. Surfacing something to a person happens outside the system, so that \
                 event is the only way a reader, a test or a conformance target can tell that the \
                 escalation happened at all.",
            ),
        ],
        ResolvedFailure::Drop => vec![
            Inline::text("When it fails the work is "),
            Inline::Strong {
                text: vec![Inline::text("dropped")],
            },
            Inline::text(
                ". The system loses it, silently, and that is a decision someone made \
                 deliberately: ",
            ),
            Inline::code("drop"),
            Inline::text(
                " is never a default, so this word was typed. Nothing is published, on purpose — \
                 an event here would make this a notification, which is a different decision.",
            ),
        ],
    }
}

/// One filled command input, and the reason its types were allowed to meet.
fn mapping_bullet(mapping: &ResolvedMapping) -> Vec<Inline> {
    let mut out = vec![
        Inline::code(mapping.target.clone()),
        Inline::text(" ("),
        Inline::code(mapping.target_type.to_string()),
        Inline::text(") ← "),
    ];
    match &mapping.value {
        ResolvedMappingValue::EventField { field, type_ref } => {
            out.push(Inline::text("the event's "));
            out.push(Inline::code(field.clone()));
            out.push(Inline::text(" ("));
            out.push(Inline::code(type_ref.to_string()));
            out.push(Inline::text(")"));
            if let Some(because) = &mapping.conversion {
                out.push(Inline::text(format!(
                    ". The two types differ, and the crossing is declared: \"{}.\"",
                    because.trim().trim_end_matches('.')
                )));
            } else {
                out.push(Inline::text("."));
            }
        }
        ResolvedMappingValue::Literal { value } => {
            out.push(Inline::text("the literal "));
            out.push(Inline::code(value.clone()));
            out.push(Inline::text(
                ". Nothing in the model says how to read that as a ",
            ));
            out.push(Inline::code(mapping.target_type.to_string()));
            out.push(Inline::text(
                ", so the compiler took it on trust rather than checking it.",
            ));
        }
    }
    out
}

/// A component's ownership, which is the only claim it makes.
fn component_prose(ir: &EssIr, component: &ResolvedComponent) -> Vec<Inline> {
    let mut out = vec![Inline::Strong {
        text: vec![Inline::code(component.name.to_string())],
    }];
    if let Some(display) = &component.naming.display {
        out.push(Inline::text(format!(" (shown as \"{display}\")")));
    }
    if let Some(summary) = &component.naming.summary {
        out.push(Inline::text(format!(" — {summary}")));
    } else {
        out.push(Inline::text("."));
    }
    out.push(Inline::text(" It owns "));
    out.extend(owned_list(ir, component));
    out.push(Inline::text("."));
    if component.accepts.is_empty() {
        out.push(Inline::text(" It accepts no commands."));
    } else {
        out.push(Inline::text(" It accepts "));
        out.extend(inline_list(names(&component.accepts)));
        out.push(Inline::text("."));
    }
    if component.publishes.is_empty() {
        out.push(Inline::text(" It publishes no events."));
    } else {
        out.push(Inline::text(" It publishes "));
        out.extend(inline_list(names(&component.publishes)));
        out.push(Inline::text("."));
    }
    out
}

/// A run of handles as the monospaced names a reader knows them by.
fn names<'a, T: ToString + 'a>(handles: impl IntoIterator<Item = &'a T>) -> Vec<Vec<Inline>> {
    handles
        .into_iter()
        .map(|handle| vec![Inline::code(handle.to_string())])
        .collect()
}

/// The contexts a component owns, or the fact that it owns none.
fn owned_list(ir: &EssIr, component: &ResolvedComponent) -> Vec<Inline> {
    if component.owns.is_empty() {
        return vec![Inline::text(
            "no bounded context — it is a unit of ownership that owns nothing, which is worth a \
             second look",
        )];
    }
    inline_list(
        component
            .owns
            .iter()
            .map(|handle| {
                let domain = ir.domain(handle);
                vec![Inline::code_link(
                    Target::Page {
                        page: domain_page_id(&domain.name),
                    },
                    domain.name.to_string(),
                )]
            })
            .collect(),
    )
}

/// One line in the index for a bounded context, with the numbers rather than an adjective.
fn domain_index_entry(ir: &EssIr, domain: &ResolvedDomain) -> Vec<Inline> {
    let mut out = vec![
        Inline::Strong {
            text: vec![Inline::Link {
                to: Target::Page {
                    page: domain_page_id(&domain.name),
                },
                text: vec![Inline::text(display_of(&domain.naming, &domain.name))],
            }],
        },
        Inline::text(" ("),
        Inline::code(domain.name.to_string()),
        Inline::text(")"),
    ];
    if let Some(summary) = &domain.naming.summary {
        out.push(Inline::text(format!(" — {summary}")));
    }
    out.push(Inline::text(format!(
        " {}.",
        capitalise(&list(&member_counts(ir, domain)))
    )));
    out
}

/// What a context holds, counted in the order its page renders it.
///
/// The type count excludes the enum each entity's lifecycle forms, because the page's `Types`
/// section excludes it too: a count that does not match the list under it is a count a reader stops
/// trusting.
fn member_counts(ir: &EssIr, domain: &ResolvedDomain) -> Vec<String> {
    let authored_types = domain.types.len() - state_types(ir, domain).len();
    vec![
        plural(authored_types, "type"),
        plural(domain.entities.len(), "entity"),
        plural(domain.views.len(), "view"),
        plural(domain.commands.len(), "command"),
        plural(domain.events.len(), "event"),
        plural(domain.errors.len(), "error"),
        plural(domain.actors.len(), "actor"),
    ]
}

/// What a replica floor claims, which is not a capacity plan.
fn replicas_sentence(workload: &ResolvedWorkload) -> String {
    let floor = match workload.replicas.min {
        0 => {
            "No replica floor is declared, so the specification does not say that running this is \
              necessary."
                .to_owned()
        }
        1 => "One instance is enough: nothing about the design needs a second.".to_owned(),
        min => format!(
            "At least {min} instances. That is a statement about correctness, not about load — the \
             specification says this system is not correct with fewer."
        ),
    };
    match workload.replicas.max {
        None => format!("{floor} No ceiling is declared."),
        Some(max) => format!("{floor} At most {max}."),
    }
}

/// Whether an instance holds anything that outlives a request.
fn stateless_sentence(workload: &ResolvedWorkload) -> &'static str {
    if workload.stateless {
        "Stateless: an instance holds nothing that outlives a request, so instances are \
         interchangeable."
    } else {
        "Stateful: an instance holds state that outlives a request, so instances are not \
         interchangeable."
    }
}

/// What a construct is called, on the wire and to a person.
fn naming_sentence(naming: &Naming, name: &QualifiedName) -> Vec<Inline> {
    let mut parts: Vec<Vec<Inline>> = Vec::new();
    if let Some(display) = &naming.display {
        parts.push(vec![Inline::text(format!(
            "shown to a person as \"{display}\""
        ))]);
    }
    if let Some(wire) = &naming.wire {
        parts.push(vec![
            Inline::text("called "),
            Inline::code(wire.clone()),
            Inline::text(" on the wire"),
        ]);
    }
    let mut out = vec![Inline::code(name.to_string())];
    if !parts.is_empty() {
        out.push(Inline::text(", "));
        out.extend(inline_list(parts));
    }
    out.push(Inline::text("."));
    out
}

/// How an instance is identified, and why the field's *name* is on this page at all.
fn identity_sentence(entity: &ResolvedEntity) -> Vec<Inline> {
    let mut out = vec![
        Inline::text("An instance is identified by "),
        Inline::code(entity.identity.name.clone()),
        Inline::text(", a "),
        Inline::code(entity.identity.type_ref.to_string()),
    ];
    if let Some(wire) = &entity.identity.naming.wire {
        if wire != &entity.identity.name {
            out.push(Inline::text(", called "));
            out.push(Inline::code(wire.clone()));
            out.push(Inline::text(" on the wire"));
        }
    }
    if let Some(display) = &entity.identity.naming.display {
        out.push(Inline::text(format!(", shown as \"{display}\"")));
    }
    out.push(Inline::text(
        ". The name is part of the model and not a convention: a view projects the identity under \
         that name, so a projection inventing its own would disagree with the view.",
    ));
    out
}

/// What must hold of an instance at rest, or the fact that nothing does.
fn entity_invariants_sentence(entity: &ResolvedEntity) -> Vec<Inline> {
    if entity.invariants.is_empty() {
        return vec![Inline::text(
            "No invariant is declared, so nothing here constrains an instance at rest.",
        )];
    }
    let mut out = vec![Inline::text("Every instance satisfies ")];
    out.extend(inline_list(statements(&entity.invariants)));
    out.push(Inline::text(
        " — a predicate over this entity's own fields, checked against them rather than stored as \
         a sentence, so an invariant reading something the entity does not have is refused instead \
         of documented.",
    ));
    out
}

/// The invariants of a construct, as the predicates a reader can check them against.
fn statements(invariants: &[Invariant]) -> Vec<Vec<Inline>> {
    invariants
        .iter()
        .map(|it| vec![Inline::code(it.statement.clone())])
        .collect()
}

/// The states an instance can be in, and where that enum comes from.
///
/// The states are read from the lifecycle rather than from the enum's variants: both say the same
/// thing, and taking them from the declaration that owns the rule means this page cannot show a
/// state the diagram below does not.
fn state_type_sentence(entity: &ResolvedEntity) -> Vec<Inline> {
    let mut out = vec![
        Inline::text("Its state is a "),
        Inline::code(entity.state_type.to_string()),
        Inline::text(", one of "),
    ];
    out.extend(inline_list(names(&entity.lifecycle.states)));
    out.push(Inline::text(
        ". That enum is synthesised from the lifecycle rather than declared beside it, so the \
         states a view's filter compares and the states drawn below cannot disagree.",
    ));
    out
}

/// Where an instance starts, and where it is allowed to stop.
fn resting_sentence(lifecycle: &StateMachine) -> Vec<Inline> {
    let mut out = vec![
        Inline::text("An instance is created in "),
        Inline::code(lifecycle.initial.to_string()),
        Inline::text("."),
    ];
    if lifecycle.terminal.is_empty() {
        out.push(Inline::text(
            " No state is terminal: nothing in this lifecycle says an instance may stop moving.",
        ));
        return out;
    }
    out.push(Inline::text(" "));
    out.extend(inline_list(names(&lifecycle.terminal)));
    out.push(Inline::text(format!(
        " {} terminal, so an instance may rest there forever. That is declared rather than \
         inferred from having no way out: an entity that cannot leave a state is either finished \
         or stuck, and only its author knows which.",
        if lifecycle.terminal.len() == 1 {
            "is"
        } else {
            "are"
        }
    )));
    out
}

/// Which views expose an entity, so a reader learns what of it leaves the context.
fn observed_by_sentence(
    ir: &EssIr,
    domain: &ResolvedDomain,
    views: Option<&Vec<&ResolvedView>>,
) -> Vec<Inline> {
    let Some(views) = views else {
        return vec![Inline::text(
            "No view projects it, so nothing outside this context is promised a way to observe \
             one.",
        )];
    };
    let links: Vec<Vec<Inline>> = views
        .iter()
        .map(|view| vec![section_link(ir, domain, &view.name, &view.domain)])
        .collect();
    let mut out = vec![Inline::text(format!(
        "{} {} it: ",
        capitalise(&plural(links.len(), "view")),
        if links.len() == 1 {
            "projects"
        } else {
            "project"
        }
    ))];
    out.extend(inline_list(links));
    out.push(Inline::text("."));
    out
}

/// Which instances a view holds, including the case where it holds all of them.
fn filter_sentence(view: &ResolvedView) -> Vec<Inline> {
    match &view.filter {
        None => vec![Inline::text(
            "It contains every instance of that entity: no filter narrows it, which is a decision \
             somebody made and not a line somebody omitted.",
        )],
        Some(filter) => vec![
            Inline::text("It contains the instances where "),
            Inline::code(filter.to_string()),
            Inline::text(
                " holds, and only those — so an instance a caller cannot find in here has been \
                 filtered out rather than lost.",
            ),
        ],
    }
}

/// How soon a view reflects a command that has already returned.
fn consistency_sentence(consistency: Consistency) -> Vec<Inline> {
    let (name, rest) = match consistency {
        Consistency::ReadYourWrites => (
            "Read-your-writes",
            ": it is current the moment the command that changed it returns. A caller that has \
             just created an invoice and cannot see it in here has been told a lie about what it \
             did.",
        ),
        Consistency::Eventual => (
            "Eventual",
            ": it catches up some time after the command returns, so a caller that reads it \
             immediately may legitimately not see its own write yet. Nothing here says how long \
             that takes, so nothing here lets a caller wait a fixed time and call it correct.",
        ),
    };
    vec![
        Inline::Strong {
            text: vec![Inline::text(name)],
        },
        Inline::text(rest),
    ]
}

/// What that consistency obliges a generated test to do, which is where it stops being a word.
fn assertion_sentence(style: AssertionStyle) -> &'static str {
    match style {
        AssertionStyle::Expect => {
            "A generated scenario asserts it once, immediately after the command: a view promising \
             this and not keeping the promise has to fail the suite rather than be retried until it \
             passes."
        }
        AssertionStyle::Eventually => {
            "A generated scenario therefore retries the assertion until the projection catches up, \
             rather than asserting once and racing it. The repair everyone reaches for instead is a \
             sleep, which turns the suite into a test of the machine it runs on."
        }
    }
}

/// The commands an actor may invoke, as links to where each one is written.
fn grants_sentence(ir: &EssIr, domain: &ResolvedDomain, actor: &ResolvedActor) -> Vec<Inline> {
    if actor.may.is_empty() {
        return vec![Inline::text(
            "It may invoke nothing: it observes. \"Who is in this picture\" is part of what a \
             specification describes, so an actor with no grant is a statement rather than an \
             unfinished line.",
        )];
    }
    let mut out = vec![Inline::text("It may invoke ")];
    out.extend(inline_list(
        actor
            .may
            .iter()
            .map(|handle| {
                let command = ir.command(handle);
                vec![section_link(ir, domain, &command.name, &command.domain)]
            })
            .collect(),
    ));
    out.push(Inline::text("."));
    out
}

/// The invariants a type's values satisfy, as a clause rather than a heading.
fn invariants_clause(invariants: &[Invariant]) -> Vec<Inline> {
    if invariants.is_empty() {
        return Vec::new();
    }
    let mut out = vec![Inline::text("Every value satisfies ")];
    out.extend(inline_list(statements(invariants)));
    out.push(Inline::text("."));
    out
}

/// One field, with the two things its type does not say.
fn field_bullet(field: &ResolvedField) -> Vec<Inline> {
    let mut out = vec![
        Inline::code(field.name.clone()),
        Inline::text(" — "),
        Inline::code(field.type_ref.to_string()),
    ];
    if field.type_ref.is_optional() {
        out.push(Inline::text(", which may be absent"));
    }
    if let Some(wire) = &field.naming.wire {
        if wire != &field.name {
            out.push(Inline::text(", called "));
            out.push(Inline::code(wire.clone()));
            out.push(Inline::text(" on the wire"));
        }
    }
    if let Some(display) = &field.naming.display {
        out.push(Inline::text(format!(", shown as \"{display}\"")));
    }
    out
}

/// Which command and branch — or which binding's escalation — causes an event.
///
/// A binding is the second way an event happens. Leaving it out would print "no command in this
/// system emits it, so something outside the specification does" on the page of an event this
/// specification is the only possible source of, which is the reverse of the truth.
fn emitters(ir: &EssIr, event: &ResolvedEvent) -> Vec<Vec<Inline>> {
    let mut out = Vec::new();
    for binding in ir.bindings().values() {
        if let ResolvedFailure::Escalate { emits } = binding.on_failure() {
            if emits.name() == &event.name {
                out.push(vec![
                    Inline::text("Emitted when binding "),
                    Inline::code(binding.name.to_string()),
                    Inline::text(" escalates: "),
                    Inline::code(ir.command(&binding.command).name.to_string()),
                    Inline::text(" failed and a person was told."),
                ]);
            }
        }
    }
    for command in ir.commands().values() {
        let branches: Vec<&ess_compiler::ir::ResolvedOutcome> = command
            .outcomes
            .iter()
            .filter(|outcome| outcome.emits.iter().any(|it| it.name() == &event.name))
            .collect();
        if !branches.is_empty() {
            out.push(branch_sentence("Emitted by ", command, &branches));
        }
    }
    if out.is_empty() {
        out.push(vec![Inline::text(
            "No command in this system emits it, so something outside the specification does.",
        )]);
    }
    out
}

/// Which command, and which of its branches, does something to a construct.
fn branch_sentence(
    opening: &str,
    command: &ResolvedCommand,
    branches: &[&ess_compiler::ir::ResolvedOutcome],
) -> Vec<Inline> {
    let mut out = vec![
        Inline::text(opening.to_owned()),
        Inline::code(command.name.to_string()),
        Inline::text(" on its "),
    ];
    out.extend(inline_list(
        branches
            .iter()
            .map(|outcome| vec![Inline::code(outcome.name.to_string())])
            .collect(),
    ));
    out.push(Inline::text(format!(
        " {}.",
        plural_bare(branches.len(), "outcome")
    )));
    out
}

/// Which command and branch reports an error.
fn reporters(ir: &EssIr, error: &ResolvedError) -> Vec<Vec<Inline>> {
    let mut out = Vec::new();
    for command in ir.commands().values() {
        let branches: Vec<&ess_compiler::ir::ResolvedOutcome> = command
            .outcomes
            .iter()
            .filter(|outcome| {
                outcome
                    .error
                    .as_ref()
                    .is_some_and(|it| it.name() == &error.name)
            })
            .collect();
        if !branches.is_empty() {
            out.push(branch_sentence("Reported by ", command, &branches));
        }
    }
    if out.is_empty() {
        out.push(vec![Inline::text(
            "No outcome in this system reports it: it is declared and unreachable.",
        )]);
    }
    out
}

/// How many outcomes a command has, said as a person would say it.
fn outcome_count_sentence(count: usize, name: &QualifiedName) -> Vec<Inline> {
    match count {
        0 => vec![
            Inline::code(name.to_string()),
            Inline::text(
                " declares no outcomes, so nothing here says what it does or when it refuses.",
            ),
        ],
        1 => vec![Inline::text("It has one outcome.")],
        _ => vec![Inline::text(format!("It has {} outcomes.", number(count)))],
    }
}

// ---- Mermaid ----------------------------------------------------------------------------------

/// A lifecycle as a Mermaid state diagram, from `ess-domain`'s own [`StateMachine`].
///
/// Rendered from the domain type directly rather than from a mirror of it: the machine holds no
/// reference that points outside itself, so a copy would only be a second place for the states to
/// disagree with the transitions.
///
/// Every state appears whether or not a transition touches it: a state with no arrows is a fact
/// about the model, and dropping it would hide exactly the sort of dead end the compiler refuses.
///
/// Each arrow carries the command that takes it, which is the whole of gate G14 as a reader meets
/// it: a lifecycle whose moves have no verbs is a diagram of what may happen with no way to make any
/// of it happen. The commands come from [`EssIr::drivers`] rather than from a name that looks like
/// the transition's — the spelling of a move says nothing about who performs it.
fn state_diagram(lifecycle: &StateMachine, drivers: &[Driver<'_>]) -> String {
    let mut out = String::from("stateDiagram-v2\n");
    let _ = writeln!(out, "    [*] --> {}", lifecycle.initial);
    for transition in &lifecycle.transitions {
        let label = match takers(drivers, &transition.name).as_slice() {
            // Unreachable for a validated specification — `missing_causation` refuses it — so this
            // draws the arrow rather than hiding it, exactly as an untouched state is drawn.
            [] => transition.name.clone(),
            takers => format!("{} ({})", transition.name, takers.join(", ")),
        };
        for from in &transition.from {
            let _ = writeln!(out, "    {from} --> {}: {label}", transition.to);
        }
    }
    for state in &lifecycle.states {
        if lifecycle.is_terminal(state) {
            let _ = writeln!(out, "    {state} --> [*]");
        } else if !touched(lifecycle, state) {
            // Mermaid draws a bare identifier as an unconnected state, which is precisely what this
            // is: declared, and reached by no move. `StateMachine::validate` refuses one, so this
            // arm only fires for a machine nothing validated — and it draws it rather than hide it.
            let _ = writeln!(out, "    {state}");
        }
    }
    trimmed(out)
}

/// The local names of the commands that take one transition, in the order the IR holds them.
///
/// Local rather than qualified because this goes inside a Mermaid arrow label, where the context is
/// already the entity's own page and a fully qualified name would push the label past the arrow.
fn takers(drivers: &[Driver<'_>], transition: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for driver in drivers {
        if driver.takes(transition) {
            let local = driver.command.name.local().to_owned();
            if !out.contains(&local) {
                out.push(local);
            }
        }
    }
    out
}

/// Which command and branch takes each declared move, spelt out beneath the diagram.
///
/// The diagram's labels say *which command*; this says which of its branches, which is the unit a
/// generated scenario is built per. It also states the rule that makes the list exhaustive, so a
/// reader knows an unlisted move is impossible rather than merely undocumented.
fn driven_blocks(lifecycle: &StateMachine, drivers: &[Driver<'_>]) -> Vec<Block> {
    if lifecycle.transitions.is_empty() {
        let mut only = Blocks::new();
        only.sentence("It declares no moves, so nothing changes its state once it exists.");
        return only.finish();
    }
    let mut out = Blocks::new();
    out.prose(vec![
        Inline::text(
            "Each move is taken by a declared command outcome, and a move nothing takes is refused \
             as ",
        ),
        Inline::code("missing_causation"),
        Inline::text(" rather than left as a state change nobody can trigger:"),
    ]);
    out.push(bullets(
        lifecycle
            .transitions
            .iter()
            .map(|transition| {
                let taken = taken_by(
                    drivers
                        .iter()
                        .filter(|driver| driver.takes(&transition.name)),
                );
                let mut item = vec![
                    Inline::code(transition.name.clone()),
                    Inline::text(" — taken by "),
                ];
                if taken.is_empty() {
                    item.push(Inline::text("nothing in this specification"));
                } else {
                    item.extend(inline_list(taken));
                }
                item
            })
            .collect(),
    ));
    let creators = taken_by(
        drivers
            .iter()
            .filter(|driver| matches!(driver.effect, ResolvedEffect::Creates)),
    );
    if creators.is_empty() {
        out.sentence(
            "No command here creates one, so an instance arrives from outside this specification.",
        );
    } else {
        let mut brought = vec![Inline::text("An instance is brought into existence by ")];
        brought.extend(inline_list(creators));
        brought.push(Inline::text("."));
        out.prose(brought);
    }
    out.finish()
}

/// The command and branch behind each of a set of moves.
fn taken_by<'a>(drivers: impl Iterator<Item = &'a Driver<'a>>) -> Vec<Vec<Inline>> {
    drivers
        .map(|driver| {
            vec![
                Inline::code(driver.command.name.to_string()),
                Inline::text(" on its "),
                Inline::code(driver.outcome.name.to_string()),
                Inline::text(" outcome"),
            ]
        })
        .collect()
}

/// `true` when any transition or the initial state mentions this state.
fn touched(lifecycle: &StateMachine, state: &StateName) -> bool {
    &lifecycle.initial == state
        || lifecycle.transitions.iter().any(|transition| {
            &transition.to == state || transition.from.iter().any(|from| from == state)
        })
}

/// What the diagram above cannot say, and the enumeration that repairs it.
///
/// The model expresses "a paid invoice may not be cancelled" as the *absence* of a transition, and
/// absence does not draw: a missing arrow looks like an arrow nobody has added yet. So the pairs no
/// move connects are listed, derived from the same transitions the diagram is drawn from, which is
/// why the two cannot come apart.
fn legality_note(lifecycle: &StateMachine) -> Vec<Block> {
    let mut out = Blocks::new();
    if lifecycle.states.len() < 2 {
        out.sentence("It has one state, so there is no move to permit or to forbid.");
        return out.finish();
    }
    let unconnected = forbidden(lifecycle);
    if unconnected.is_empty() {
        out.sentence(
            "Every ordered pair of these states is connected by some move, so this lifecycle \
             forbids nothing.",
        );
        return out.finish();
    }
    out.sentence(
        "Illegal transitions are illegal by absence: no rule forbids them, there is simply no \
         arrow, because a rule would be a second place for the same truth to live. A diagram cannot \
         show an absence, so the pairs it does not connect are listed here, derived from the same \
         transitions — anything named below is a move this specification does not permit.",
    );
    out.push(bullets(
        unconnected
            .into_iter()
            .map(|(from, to)| {
                vec![
                    Inline::code(from.to_string()),
                    Inline::text(" may not become "),
                    Inline::code(to.to_string()),
                ]
            })
            .collect(),
    ));
    out.finish()
}

/// Every ordered pair of distinct states with no transition between them.
fn forbidden(lifecycle: &StateMachine) -> Vec<(&StateName, &StateName)> {
    let mut out = Vec::new();
    for from in &lifecycle.states {
        for to in &lifecycle.states {
            if from != to && !lifecycle.can_move(from, to) {
                out.push((from, to));
            }
        }
    }
    out
}

/// One binding as a flow: the event, the command, each branch, and where a failure goes.
///
/// A diagram rather than a table because the failure path is the part a table flattens: `escalate`
/// means there is an edge out of this system to a person, and that edge is the whole reason the word
/// is required.
fn binding_flow(ir: &EssIr, binding: &ResolvedBinding) -> String {
    let event = ir.event(&binding.event);
    let command = ir.command(&binding.command);
    let mut out = String::from("flowchart LR\n");
    let _ = writeln!(out, "    event[\"{}\"]", label(&event.name.to_string()));
    let _ = writeln!(out, "    command[\"{}\"]", label(&command.name.to_string()));
    let _ = writeln!(
        out,
        "    event -->|\"{}\"| command",
        label(binding.name.as_str())
    );
    let mut reached_failure = false;
    for (index, outcome) in command.outcomes.iter().enumerate() {
        let _ = writeln!(
            out,
            "    outcome{index}[\"{}\"]",
            label(outcome.name.as_str())
        );
        let _ = writeln!(out, "    command --> outcome{index}");
        for (emitted, handle) in outcome.emits.iter().enumerate() {
            let _ = writeln!(
                out,
                "    emit{index}_{emitted}[\"{}\"]",
                label(&handle.to_string())
            );
            let _ = writeln!(out, "    outcome{index} --> emit{index}_{emitted}");
        }
        if let Some(handle) = &outcome.error {
            let _ = writeln!(out, "    error{index}[\"{}\"]", label(&handle.to_string()));
            let _ = writeln!(out, "    outcome{index} --> error{index}");
            let _ = writeln!(
                out,
                "    error{index} --> failure[\"{}\"]",
                label(&failure_label(ir, binding))
            );
            reached_failure = true;
        }
    }
    // The edge the whole diagram is for. `escalate` is a hand-off out of this system, and the event
    // is the only mark it leaves inside it — so a reader who cannot see the event on the page cannot
    // tell an escalation from nothing happening.
    if let (true, ResolvedFailure::Escalate { emits }) = (reached_failure, binding.on_failure()) {
        let _ = writeln!(
            out,
            "    escalation[\"{}\"]",
            label(&ir.event(emits).name.to_string())
        );
        let _ = writeln!(out, "    failure --> escalation");
    }
    trimmed(out)
}

/// Where a failed binding's work goes, in a few words for a diagram node.
fn failure_label(ir: &EssIr, binding: &ResolvedBinding) -> String {
    match binding.on_failure() {
        ResolvedFailure::Retry => "retried by the transport".to_owned(),
        ResolvedFailure::Escalate { emits } => {
            format!("escalated to a person, emitting {}", ir.event(emits).name)
        }
        ResolvedFailure::Drop => "dropped: the work is lost".to_owned(),
    }
}

/// The whole system: actors, and the commands and events each component declares.
///
/// The diagram is [`SystemGraph`]'s, unchanged. The graph itself is not read here: `ess graph`
/// publishes the same picture, and a second reading of the IR in this file is how the two came to
/// be different graphs wearing one name — see [`crate::graph`] for what they disagreed about.
fn system_graph(ir: &EssIr) -> String {
    trimmed(SystemGraph::of(ir).mermaid())
}

/// Mermaid source as a [`Block::Diagram`] carries it: the lines, and not the newline after the last.
///
/// The renderer writes the newline that closes the fence. A source that ended with one of its own
/// would put a blank line inside every diagram on every page.
fn trimmed(mut source: String) -> String {
    source.truncate(source.trim_end_matches('\n').len());
    source
}

// ---- plumbing ---------------------------------------------------------------------------------

/// The known gaps as a table, under a heading the page chooses, or nothing when there are none.
///
/// Nothing, rather than an empty table under its heading: a section that says "what this cannot
/// show" and then shows an empty table teaches a reader to skip it, and the day it has a row in it
/// is the day that habit costs something.
fn gap_blocks(heading: &str, preamble: &str) -> Vec<Block> {
    if Docs::known_gaps().is_empty() {
        return Vec::new();
    }
    let mut under = Blocks::new();
    under.sentence(preamble);
    under.push(Block::Table {
        columns: [
            "construct",
            "what is dropped",
            "where it would go",
            "what it needs",
        ]
        .into_iter()
        .map(|column| vec![Inline::text(column)])
        .collect(),
        rows: Docs::known_gaps()
            .iter()
            .map(|gap| {
                vec![
                    vec![Inline::text(gap.construct)],
                    vec![Inline::text(cell(gap.dropped))],
                    vec![Inline::text(gap.page)],
                    vec![Inline::text(cell(gap.needs))],
                ]
            })
            .collect(),
    });
    vec![section(
        2,
        vec![Inline::text(heading)],
        None,
        under.finish(),
    )]
}

/// The identity of a bounded context's page, which is also where a renderer files it.
///
/// The dots of a qualified name become hyphens. A dot in a path segment makes a static file server
/// read `acd.routing` as a name with an extension, so `domains/acd.routing.md` is served as
/// something it is not or not at all — every one of these pages 404'd behind GitLab Pages until
/// the adopter carried a rename pass of their own. A generator that emits a route nothing can serve
/// has not generated documentation.
fn domain_page_id(name: &QualifiedName) -> PageId {
    PageId(format!("domains/{}", name.to_string().replace('.', "-")))
}

/// The enum each of a context's entities forms from its lifecycle.
///
/// Structural: a handle equal to some [`ResolvedEntity::state_type`], not a name with `State` read
/// out of it. Intersected with the context's own types, so the count and the section that skips
/// these cannot disagree.
fn state_types<'a>(ir: &'a EssIr, domain: &'a ResolvedDomain) -> BTreeSet<&'a TypeHandle> {
    domain
        .entities
        .iter()
        .map(|handle| &ir.entity(handle).state_type)
        .filter(|state| domain.types.contains(*state))
        .collect()
}

/// A link from a bounded context's page to a construct's own section.
///
/// Its own page means a bare fragment, and another context's page means the sibling file: writing
/// `domains/billing.invoice.md#invoice` from inside `domains/billing.invoice.md` would be a second
/// spelling of one place, and the second spelling is the one that rots.
fn section_link(
    ir: &EssIr,
    from: &ResolvedDomain,
    name: &QualifiedName,
    owner: &ess_compiler::ir::DomainHandle,
) -> Inline {
    let owner = ir.domain(owner);
    let to = Target::Anchor {
        page: domain_page_id(&owner.name),
        anchor: slug(&relative(name, &owner.name)),
    };
    if owner.name == from.name {
        Inline::code_link(to, relative(name, &from.name))
    } else {
        Inline::code_link(to, name.to_string())
    }
}

/// A name with its context's prefix removed, so a page does not repeat its own title on every line.
fn relative(name: &QualifiedName, domain: &QualifiedName) -> String {
    name.segments()
        .strip_prefix(domain.segments())
        .map_or_else(|| name.to_string(), |rest| rest.join("."))
}

/// `true` when either end of a reference is declared inside a context.
fn touches(reference: &ess_compiler::ir::ResolvedTypeRef, domain: &QualifiedName) -> bool {
    reference
        .named_leaves()
        .iter()
        .any(|handle| handle.name().is_within(domain))
}

/// The bindings that rely on a crossing, and the input each of them fills with it.
fn crossing_users(ir: &EssIr, conversion: &ResolvedConversion) -> Vec<Vec<Inline>> {
    let mut out = Vec::new();
    for binding in ir.bindings().values() {
        for mapping in &binding.mapping {
            let crossed = matches!(
                &mapping.value,
                ResolvedMappingValue::EventField { type_ref, .. }
                    if type_ref == &conversion.from && mapping.target_type == conversion.to
            );
            if crossed {
                out.push(vec![
                    Inline::code_link(binding_target(binding), binding.name.to_string()),
                    Inline::text(", filling "),
                    Inline::code(mapping.target.clone()),
                ]);
            }
        }
    }
    out
}

/// Where a binding is written up, which is its own section of the interactions page.
fn binding_target(binding: &ResolvedBinding) -> Target {
    Target::Anchor {
        page: PageId::from("interactions"),
        anchor: slug(binding.name.as_str()),
    }
}

/// Events no binding reacts to.
fn unread_events(ir: &EssIr) -> Vec<&QualifiedName> {
    let reactions = ir.reactions();
    ir.events()
        .keys()
        .filter(|name| !reactions.keys().any(|handle| handle.name() == *name))
        .collect()
}

/// The display name, or the last segment when nothing overrides it.
fn display_of<'a>(naming: &'a Naming, name: &'a QualifiedName) -> &'a str {
    naming.display_or(name)
}

/// Somebody's reason, quoted, so it reads as their words rather than as this page's.
///
/// Each line is trimmed before it is quoted: a reason written as a folded block in the
/// specification arrives carrying the indentation of the file it was written in, and that
/// indentation would be four spaces inside a quotation — which every Markdown renderer reads as
/// code.
fn quoted_reason(text: &str) -> Block {
    Block::Quote {
        blocks: vec![Block::Prose {
            text: vec![Inline::text(
                text.lines().map(str::trim).collect::<Vec<_>>().join("\n"),
            )],
        }],
    }
}

/// A headed run of blocks, with the anchor derived from the heading rather than written twice.
fn section(
    level: u8,
    title: Vec<Inline>,
    about: Option<EssSemanticRef>,
    blocks: Vec<Block>,
) -> Block {
    Block::Section {
        level,
        anchor: anchor_of(&title),
        title,
        about,
        blocks,
    }
}

/// An unordered list of one paragraph each, which is every list these pages write.
fn bullets(items: Vec<Vec<Inline>>) -> Block {
    Block::List {
        ordered: false,
        items: items
            .into_iter()
            .map(|text| vec![Block::Prose { text }])
            .collect(),
    }
}

/// An English list of runs: `a`, `a and b`, `a, b and c`.
///
/// [`list`]'s shape over inlines rather than over strings, because a list of links is the common
/// case and a link cannot survive being turned into a sentence fragment first.
fn inline_list(items: Vec<Vec<Inline>>) -> Vec<Inline> {
    let mut out = Vec::new();
    let last = items.len().saturating_sub(1);
    for (position, item) in items.into_iter().enumerate() {
        if position > 0 {
            out.push(Inline::text(if position == last { " and " } else { ", " }));
        }
        out.extend(item);
    }
    out
}

/// The anchor a heading gets, derived from the words in it rather than from its markup.
fn anchor_of(title: &[Inline]) -> String {
    slug(&plain(title))
}

/// The words of a run of inlines, with everything that is not a word dropped.
fn plain(text: &[Inline]) -> String {
    let mut out = String::new();
    for inline in text {
        match inline {
            Inline::Text { text } | Inline::Code { text } => out.push_str(text),
            Inline::Emphasis { text } | Inline::Strong { text } | Inline::Link { text, .. } => {
                out.push_str(&plain(text));
            }
        }
    }
    out
}

/// The blank line the index, the interactions page and every context page end with.
///
/// A list with nothing in it, which renders as nothing; what reaches the page is the blank line the
/// renderer puts between one block and the next. That blank is what the string writer this replaced
/// left at the end of those three pages, and `tests/corpus` pins it. Tidying it away would change
/// what is committed in every adopter's repository, which is a different change from this one and
/// belongs in its own commit.
fn trailing_blank() -> Block {
    Block::List {
        ordered: false,
        items: Vec::new(),
    }
}

/// Text safe inside a Markdown table cell.
fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

/// An English list: `a`, `a and b`, `a, b and c`.
fn list(items: &Vec<String>) -> String {
    match items.as_slice() {
        [] => String::new(),
        [only] => only.clone(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

/// A count and its noun, agreeing.
fn plural(count: usize, noun: &str) -> String {
    format!("{} {}", number(count), plural_bare(count, noun))
}

/// A noun, agreeing with a count that is printed elsewhere.
fn plural_bare(count: usize, noun: &str) -> String {
    if count == 1 {
        return noun.to_owned();
    }
    match noun.strip_suffix('y') {
        // `entity` becomes `entities`, by the general rule for a consonant before the `y` — because
        // "two entitys" in an index is the sort of detail that makes a reader distrust the numbers
        // beside it.
        Some(stem) if stem.ends_with(|it: char| !"aeiou".contains(it)) => format!("{stem}ies"),
        _ => format!("{noun}s"),
    }
}

/// A phrase that has become the start of a sentence.
fn capitalise(text: &str) -> String {
    let mut characters = text.chars();
    match characters.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(characters).collect(),
    }
}

/// A small number as a word, because a sentence with a digit in it reads like a form.
fn number(count: usize) -> String {
    match count {
        0 => "no".to_owned(),
        1 => "one".to_owned(),
        2 => "two".to_owned(),
        3 => "three".to_owned(),
        4 => "four".to_owned(),
        5 => "five".to_owned(),
        6 => "six".to_owned(),
        7 => "seven".to_owned(),
        8 => "eight".to_owned(),
        9 => "nine".to_owned(),
        other => other.to_string(),
    }
}

/// The anchor a Markdown renderer derives from a heading.
///
/// Lowercased, spaces hyphenated, everything else dropped — the rule GitHub applies. Computed rather
/// than guessed because a link to `#createinvoice` that should have been `#create-invoice` fails
/// silently: the page opens, at the wrong place.
fn slug(heading: &str) -> String {
    let mut out = String::with_capacity(heading.len());
    for character in heading.chars() {
        if character.is_ascii_alphanumeric() {
            out.extend(character.to_lowercase());
        } else if character == '-' || character == '_' {
            out.push(character);
        } else if character == ' ' {
            out.push('-');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    use ess_domain::entity::Transition;

    /// A state name, or a panic naming the spelling that is not one.
    fn state(name: &str) -> StateName {
        StateName::new(name).unwrap_or_else(|error| panic!("`{name}` is a state name: {error}"))
    }

    /// A machine of `ess-domain`'s own type, so the renderer is exercised against what the IR hands
    /// it rather than against a fixture shaped to suit it.
    fn machine(
        initial: &str,
        states: &[&str],
        terminal: &[&str],
        transitions: Vec<Transition>,
    ) -> StateMachine {
        StateMachine {
            states: states.iter().map(|it| state(it)).collect(),
            initial: state(initial),
            terminal: terminal.iter().map(|it| state(it)).collect(),
            transitions,
        }
    }

    /// The blocks as the Markdown renderer writes them.
    ///
    /// The assertions below are about what a reader is told, and the renderer is what turns blocks
    /// back into lines. A test that walked the blocks instead would be checking the structure it
    /// had just been handed against itself.
    fn rendered(blocks: Vec<Block>) -> String {
        crate::markdown::page(&Page {
            id: PageId::from("page"),
            title: vec![Inline::text("Page")],
            about: None,
            provenance: SlicedProvenance {
                provenance: crate::provenance::Provenance {
                    system: "billing".to_owned(),
                    specification_version: "v3".to_owned(),
                    source_digest: "0".repeat(64),
                    contract_digest: "f".repeat(64),
                },
                slice: crate::provenance::ModelSlice::WholeModel,
            },
            blocks,
        })
        .contents
    }

    /// One move, or a panic naming the transition that is not one.
    fn moves(name: &str, from: &[&str], to: &str) -> Transition {
        Transition::new(name, from.iter().map(|it| state(it)), state(to))
            .unwrap_or_else(|error| panic!("`{name}` is a transition name: {error}"))
    }

    /// The billing example's lifecycle, as `examples/billing/domains/invoice.yaml` declares it.
    ///
    /// The same shape `ResolvedEntity::lifecycle` carries, built here so the diagram's expected
    /// output is asserted without compiling a specification — `tests/docs.rs` does that over the
    /// example itself.
    fn invoice_lifecycle() -> StateMachine {
        machine(
            "Draft",
            &["Draft", "Issued", "Paid", "Cancelled"],
            &["Paid", "Cancelled"],
            vec![
                moves("issue", &["Draft"], "Issued"),
                moves("settle", &["Issued"], "Paid"),
                moves("cancel", &["Draft", "Issued"], "Cancelled"),
            ],
        )
    }

    #[test]
    fn a_lifecycle_renders_as_a_state_diagram_with_its_initial_and_terminal_states_marked() {
        let diagram = state_diagram(&invoice_lifecycle(), &[]);

        assert!(diagram.starts_with("stateDiagram-v2\n"), "{diagram}");
        assert!(diagram.contains("    [*] --> Draft\n"), "{diagram}");
        assert!(
            diagram.contains("    Draft --> Issued: issue\n"),
            "{diagram}"
        );
        assert!(
            diagram.contains("    Issued --> Paid: settle\n"),
            "{diagram}"
        );
        assert!(
            diagram.contains("    Draft --> Cancelled: cancel\n"),
            "{diagram}"
        );
        assert!(
            diagram.contains("    Issued --> Cancelled: cancel\n"),
            "{diagram}"
        );
        assert!(diagram.contains("    Cancelled --> [*]\n"), "{diagram}");
        // The last line carries no newline of its own: the renderer writes the one that closes the
        // fence, and a source ending in one would put a blank line inside every diagram.
        assert!(diagram.ends_with("    Paid --> [*]"), "{diagram}");
    }

    #[test]
    fn a_transition_from_two_states_draws_one_arrow_from_each() {
        let diagram = state_diagram(&invoice_lifecycle(), &[]);

        assert_eq!(
            diagram.matches("cancel").count(),
            2,
            "`cancel` leaves both Draft and Issued: {diagram}"
        );
    }

    #[test]
    fn a_state_no_transition_touches_is_still_drawn() {
        // `StateMachine::validate` refuses an unreachable state, so this machine cannot come out of
        // a compiled specification. It is rendered anyway: a projection that silently dropped the
        // state would hide exactly the dead end the compiler exists to refuse, and the diagram is
        // the artifact somebody looks at when asking why the refusal happened.
        let stranded = machine("Draft", &["Draft", "Void"], &[], Vec::new());

        let diagram = state_diagram(&stranded, &[]);

        assert!(diagram.contains("    [*] --> Draft\n"), "{diagram}");
        assert!(diagram.ends_with("\n    Void"), "{diagram}");
    }

    #[test]
    fn the_page_names_every_transition_the_specification_does_not_permit() {
        let note = rendered(legality_note(&invoice_lifecycle()));

        // The example's own headline case: a paid invoice may not be cancelled, and the model says
        // so by not saying anything.
        assert!(note.contains("`Paid` may not become `Cancelled`"), "{note}");
        assert!(note.contains("`Cancelled` may not become `Paid`"), "{note}");
        assert!(note.contains("`Draft` may not become `Paid`"), "{note}");
        assert!(
            !note.contains("`Draft` may not become `Issued`"),
            "that transition exists: {note}"
        );
    }

    #[test]
    fn a_lifecycle_with_one_state_forbids_nothing_rather_than_forbidding_everything() {
        // A single state is the only zero-transition machine `StateMachine::validate` accepts, and
        // the complement of nothing over one state is nothing. Listing "may not become" pairs here
        // would be inventing a prohibition out of an empty set.
        let single = machine("Draft", &["Draft"], &["Draft"], Vec::new());

        let note = rendered(legality_note(&single));

        assert!(!note.contains("may not become"), "{note}");
        assert!(note.contains("one state"), "{note}");
    }

    #[test]
    fn a_lifecycle_that_connects_every_pair_says_it_forbids_nothing() {
        let open = machine(
            "Draft",
            &["Draft", "Paid"],
            &[],
            vec![
                moves("settle", &["Draft"], "Paid"),
                moves("reopen", &["Paid"], "Draft"),
            ],
        );

        let note = rendered(legality_note(&open));

        // The distinction the page has to keep: "nothing is forbidden" and "nothing was carried"
        // read the same to a reader and are opposite statements about the model.
        assert!(!note.contains("may not become"), "{note}");
        assert!(note.contains("forbids nothing"), "{note}");
    }

    #[test]
    fn a_heading_and_its_anchor_agree() {
        assert_eq!(slug("`CreateInvoice`"), "createinvoice");
        assert_eq!(slug("Create invoice"), "create-invoice");
        assert_eq!(
            slug("notify-on-invoice-created"),
            "notify-on-invoice-created"
        );
        assert_eq!(slug("Invoice.State"), "invoicestate");
    }

    #[test]
    fn a_list_of_three_reads_as_a_person_would_write_it() {
        assert_eq!(list(&vec![]), "");
        assert_eq!(list(&vec!["a".to_owned()]), "a");
        assert_eq!(list(&vec!["a".to_owned(), "b".to_owned()]), "a and b");
        assert_eq!(
            list(&vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]),
            "a, b and c"
        );
    }

    #[test]
    fn a_gap_that_ships_says_which_crate_closes_it() {
        // The allowlist is empty: every construct a specification declares reaches the IR and
        // reaches a page. The rule each future entry has to satisfy is asserted anyway, because the
        // day someone adds one is the day nobody is reading this file.
        for gap in Docs::known_gaps() {
            assert!(
                gap.needs.contains("ess-compiler"),
                "a gap says which crate closes it, or nobody closes it: {gap:?}"
            );
            assert!(
                !gap.page.is_empty(),
                "a gap nobody is told where to look for is a gap nobody looks for: {gap:?}"
            );
        }
    }

    #[test]
    fn a_plural_of_entity_is_entities() {
        assert_eq!(plural(1, "entity"), "one entity");
        assert_eq!(plural(2, "entity"), "two entities");
        assert_eq!(plural(0, "view"), "no views");
    }
}
