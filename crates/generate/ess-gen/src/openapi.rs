//! `OpenAPI` 3.1, one document per component.
//!
//! # A command is not an endpoint
//!
//! `CreateInvoice` is a semantic command. `POST /invoices` is one way to expose it, and design §6
//! draws exactly that line: an inner domain of entities, state, invariants and commands, an outer
//! surface of HTTP APIs, topics and queues, and *mappings* between them. That line is what lets one
//! specification compile to a modular monolith or to distributed services without touching the
//! domain model — and a generator that turns commands into paths is precisely where it gets erased,
//! quietly, by a convention nobody wrote down.
//!
//! So the convention is written down here. §6 puts the decision in the model:
//!
//! ```yaml
//! command:
//!   ref: invoice.CreateInvoice
//! exposures:
//!   - kind: http
//!     method: POST
//!     path: /v1/invoices
//!     response:
//!       status: 201
//! ```
//!
//! **The model has no `exposures` construct.** Nothing in [`EssIr`] carries a method, a path or a
//! status, so every row of the table below is *this generator's* decision rather than the
//! specification's, and each row is a rule a reviewer can disagree with. Until `exposures:` exists,
//! the convention is the whole answer, and it is deliberately the least inventive one available:
//! REST resource modelling would require guessing that `Create` is a verb, that `Invoice`
//! pluralises to a collection, and that an invoice is addressable at a URL the specification never
//! mentions. A command endpoint guesses none of that.
//!
//! | question | this generator's answer | what it rests on |
//! |---|---|---|
//! | one document per what? | one per component, at `openapi/{component}.yaml` | §5 — a component is the unit of ownership, so it is the unit that has an API |
//! | which commands? | the ones the component `accepts` | §5 |
//! | a command no component accepts | appears in no document at all | no component answers for it; naming one would be inventing an owner |
//! | a component that accepts nothing | still gets a document, with `paths: {}` | a missing file is indistinguishable from a generator that broke |
//! | method | always `POST` | a command changes state; the model declares no command safe and describes no resource to `PUT` or `DELETE` |
//! | path | `/{domain wire name}/commands/{command wire name}` | both are declared wire names; the `commands` segment is what stops the path pretending to be a resource |
//! | two commands claiming one path | *both* move to `/commands/{qualified name}` | qualified names are unique by construction, so the fallback cannot collide in turn |
//! | a command with no `naming.wire` | its qualified name's last segment, verbatim — `SendEmail`, capitals and all | [`Naming::wire_or`](ess_domain::name::Naming::wire_or). A generator inventing its own kebab-casing would disagree with every other projection about what this command is called on the wire |
//! | `operationId` | the command's qualified name, verbatim, unless an `ess-http/1` binding gives the route an `operation_id` | unique already; any prettifying transformation trades that guarantee for cosmetics. A binding's id is the protocol's, and compiling it refuses one that is not unique (`ESS-HTTP-032`) |
//! | request body | the command's `input`, `required` when any input field is | |
//! | an outcome with no `error` | `202` | below |
//! | an outcome declaring `returns: true`, from `ess/22` | `200`, with the command's response under `response` | `202` claims the request was queued; this branch was carried out and answered in the same response (beyond10x/ess#424). Below `ess/22` it keeps `202` and its body |
//! | an outcome whose `error` the input decides | `422` | below |
//! | an outcome whose `error` is `external` | `502` | below |
//! | an outcome whose `error` is `wrong_state` | `409` | below |
//! | several outcomes on one status | one response, `oneOf` the outcome schemas, discriminated by `outcome` | a status that collapsed two branches would lose the branch |
//! | `Idempotency-Key` | a **required** header, on exactly the commands some binding invokes with `delivery: at_least_once` | below |
//! | schemas | inline in `components.schemas`; no `$ref` ever leaves the document | below |
//!
//! # Outcomes are the interesting part
//!
//! A command's `outcomes` are the reason wave 1 restructured the model (review F1): the accepted
//! branch and the refused branch are different results, and a projection that keeps only the first
//! has thrown away the branch where the money does not move. Each outcome therefore becomes its own
//! response schema, named `{command}.{outcome}.Response`, carrying:
//!
//! * `outcome` — a `const` of the declared outcome name, so a client can tell two branches apart
//!   even when they share a status;
//! * `error` — a `const` of the declared error's qualified name, when the outcome names one;
//! * `payload` — the [`ResolvedError`]'s own fields, when it declares any. An error that carries
//!   nothing gets no `payload`, rather than an empty object that looks like a mistake.
//!
//! * `published` — every event the branch published, in the order it emits them, each
//!   `{"event": <qualified name>, "payload": {…}}`: `prefixItems` in emit order and `items: false`,
//!   so the same events in another order, or one of them twice, is not this branch's answer. The
//!   events also reach consumers through the event transport; the answer lists them so the caller
//!   learns what its command caused, including a created identity.
//!
//! # `501` is the one status no branch declares
//!
//! Every command also declares `501`: the realization is unfinished. Either a port the command
//! runs reported an unmet obligation and nothing was written, or the command's effect was committed
//! and delivering what it published to a binding failed. The body's boolean `committed` tells the
//! two apart: `false` for the first, `true` for the second. A client must not retry it, because
//! after a failed delivery a retry performs the command a second time.
//!
//! # `external` is a `502`, not a `422`
//!
//! `external` names a branch the input cannot decide — `failed: external: the provider rejects the
//! recipient address`. Reporting that as a `4xx` tells the caller it did something wrong, sends it
//! to fix the one thing it cannot fix, and tells every retry layer in between that retrying is
//! pointless. For an external cause, retrying is exactly the right move. So it is a `5xx`: `502`,
//! attributing the refusal to a dependency, rather than `500`, which would claim a fault in this
//! component, or `503`, which would claim the whole component is unavailable when one provider
//! refused one request.
//!
//! An input-decided refusal is `422` and not `400`: `400` is for a request the server could not
//! parse, which is decided by the schema and would be true of any endpoint. `amount.amount <= 0` is
//! a request the server understood and refused on domain grounds, which is what `422` means, and a
//! client can act on the difference — one means fix the value, the other means fix the serialiser.
//!
//! # `wrong_state` is a `409`, and it is a third thing
//!
//! A `wrong_state:` branch is refused for a reason the caller did not cause and can fix: the invoice
//! it asked to issue is already paid. `422` would tell it to correct a request that was correct, and
//! `502` would blame a dependency that was never involved. `409 Conflict` is the one status that
//! says what happened — the request conflicts with the state of the thing it names — and until the
//! model could declare that branch at all, every projection had to collapse it into one of the other
//! two or into nothing.
//!
//! # Idempotency comes from the bindings, not from the verb
//!
//! `delivery: at_least_once` on a binding says the command it invokes may arrive more than once for
//! one cause. The consequence lands on the receiver: someone will call `SendEmail` twice for one
//! `InvoiceCreated`, and a surface with no way to say "this is the same invocation as the last one"
//! leaves the receiver deduplicating with no key. So the header exists exactly where the model says
//! a repeat is permitted, and it is **required** there — optional, an at-least-once caller could
//! omit it and `at_least_once` would become a word with no mechanism behind it.
//!
//! `CreateInvoice`, which no binding invokes, gets no header: nothing in the specification says
//! anyone may call it twice, and adding it anyway would be this generator inventing a delivery
//! guarantee.
//!
//! # Schemas are inline, and the mapping is not this file's
//!
//! Every `$ref` resolves inside the document it appears in. The alternative — pointing at the
//! `schema` projection's files — couples two generators through a path layout that nothing checks,
//! breaks the moment either directory changes, and produces a document that cannot be pasted into a
//! validator without its siblings.
//!
//! What is *in* the document comes from `schema::types`, which is this crate's one
//! answer to "what is a valid `billing.invoice.Money`". `OpenAPI` 3.1's schema dialect **is** JSON
//! Schema 2020-12, so there is nothing to translate: the fragment this file embeds is the fragment
//! the `schema` projection writes, keyword for keyword, with its pointers retargeted at
//! `components.schemas` by `under_components`. This file used to carry a copy of that mapping and
//! `asyncapi.rs` a second one; both drifted from it and from each other, and the drift was published
//! as contradictory contracts for one event. `tests/agreement.rs` is what holds the three documents
//! to one answer now, so the way to change what a `Decimal` looks like here is to change
//! `src/types.rs` and get all three at once.
//!
//! The one thing this file still decides about a schema is which schemas exist: the named leaves of
//! every accepted command's input and of every declared error's payload, transitively. A component's
//! document describes its own surface and nothing else.
//!
//! Two `OpenAPI`-specific consequences of that dialect are worth naming. `Bytes` is
//! `contentEncoding: base64`, which is 2020-12's own keyword — `format: byte` is `OpenAPI` 3.0's
//! spelling and is not a 3.1 format. And a `title` or a `description` beside a `$ref` is meaningful
//! in 3.1 and would have been silently discarded in 3.0, which is one of the reasons this generator
//! emits 3.1.
//!
//! What this file lost in the unification was prose. It used to fold a construct's invariants and a
//! generated sentence about newtype distinctness into `description`, beside whatever the author had
//! written there. Both facts are still published — `x-ess-invariants` carries the author's own
//! statements and `x-ess-kind: newtype` says which construct it is — and both are now readable by a
//! tool, which is what neither could be as prose. The `description` is the author's words and nothing
//! else, so a reader can tell what the specification said from what the generator said, and a drift
//! check does not have to parse English. `tests/agreement.rs` compares annotations across the three
//! projections as strictly as it compares assertions, for the reason `src/asyncapi.rs` states under
//! "An annotation is a fact too": a fact with two spellings is a fact every consumer has to
//! reconcile.
//!
//! # What this refuses to guess
//!
//! Each of these is something an `OpenAPI` document usually has and this model says nothing about.
//! Emitting a plausible default would put a claim in a contract that no specification backs.
//!
//! | omitted | why |
//! |---|---|
//! | `servers` | the model has no URL for a component; `topology` describes replicas and resources, not addresses. Only an `ess-http/1` binding's `servers` for a `reached_by: network` component put them in its document |
//! | `security`, `securitySchemes` | the model states no authentication. See below: `may:` has landed, and it answers a different question |
//! | a version in the path | `info.version` carries the specification's version, which is the only version the model has. A path prefix would invent an API version that can disagree with it |
//! | pagination, filtering, sorting | commands, not collections. A view *is* in the IR — its fields, its filter, its consistency — but nothing in the model says how one is *read*: no path, no page size, no cursor, no ordering. Exposing one would invent a query surface, and §6's `exposures:` is where that decision belongs. A view is rendered by the documentation projection and by no path here |
//! | `400`, `401`, `404`, `429`, `500` | no outcome declares them. They are properties of a transport and a deployment, not of a command |
//! | `Location`, `ETag`, cache headers | each asserts a resource at an address; the model describes no addressable resource |
//! | `201 Created` | it claims a resource now exists at a URL this document does not have. `202` claims what the model actually states: the branch was taken |
//!
//! ## `may:` is authorization, and `security` is authentication
//!
//! That row's reason used to be "actors and their `may:` lists are not in this wave's IR". They are
//! now: [`ResolvedActor::may`](ess_compiler::ir::ResolvedActor::may) carries resolved command
//! handles, and [`EssIr::grants`] asks the question in the direction a generator needs it. The
//! omission stands anyway, for a different reason, and the difference matters because the old reason
//! would have made emitting a scheme the obvious next step.
//!
//! `may:` says *which actor may invoke this command*. `securitySchemes` says *how a caller proves it
//! is that actor* — a bearer token, an API key, an OAuth flow, at some URL, issued by somebody. The
//! model states none of that, so a `security` block would be this generator inventing an
//! authentication mechanism and every client generated from the document would implement the
//! invention. That is exactly what this table exists to prevent, and it is worse than a gap: a gap is
//! visible.
//!
//! The grant is not dropped, though — a fact the specification states and no artifact carries is a
//! fact that stops being true. Every operation carries `x-ess-may-invoke`, naming the actors the
//! specification permits. It is an extension and therefore an annotation: no tool enforces it, no
//! request is refused by it, and it makes no claim about how the caller was identified. When the
//! model grows an authentication construct, `security` is what this row changes to, and
//! `x-ess-may-invoke` is what tells whoever writes it which operations need one.
//!
//! # With an HTTP binding
//!
//! [`BoundOpenApi`] is the same projection with an `ess-http/1` binding beside the model
//! (beyond10x/ess#493, `docs/design/http-presentation-binding.md`). It is what the table's
//! `method` and `path` rows become where a protocol fixes them: an operation the binding routes is
//! at its declared method and path ([`http::bound_routes`]), marked `x-ess-route-source: binding`;
//! each path variable is a required `in: path` parameter; a command's input field carried in the
//! path is not in `{command}.Input`, the request body, which is absent when the path carries every
//! field; and a view's parameter carried in the path is not a query parameter. An operation the
//! binding does not name keeps exactly the projection above, and without a binding every byte is
//! unchanged.
//!
//! Stage 2 of the binding states the rest of an operation's wire shape, and the document follows
//! it on the routes the binding names ("What `OpenAPI` writes (k4)"):
//!
//! | binding | document |
//! |---|---|
//! | `request: {encoding: form}` | the request body is `application/x-www-form-urlencoded`, its schema `{command}.Input` |
//! | `request: {encoding: query}` | no request body and no `{command}.Input`; each field the path does not carry is an `in: query` parameter under its wire name, a `List` with `style: form` and `explode: true` and not required, its absent key the empty list |
//! | `outcomes.<o>.status` | the outcome is answered under that status instead of the derived one |
//! | `body: response` | the command's `{command}.Result`, bare: no `outcome`, no `published` |
//! | `body: response.<field>` | `{command}.{outcome}.Response` is that response field's schema alone |
//! | `errors` | each outcome with `error` the route does not list is described as if listed with that answer |
//! | `body: {<member>: error}` | `{command}.{outcome}.Response` is the error object: `<member>` a `const` of the error's wire name beside the error's fields; two on one status are told apart by a discriminator on `<member>` |
//! | `body: none` | no content |
//! | `headers` | response `headers`: a literal is a `const`, a source a `string` required unless it may be absent, a challenge a `string` described param by param |
//! | `redirect` | a required `Location` header, `format: uri`, saying which parameters go into its query or fragment |
//! | a view's `body: single` | the row schema at `200` and no content at its `empty` status |
//! | `refusals.<key>` | replaces the standard refusal's response on every bound operation that can answer it |
//!
//! A bare body names no branch, so every response of a command whose route uses a stage-2 key
//! carries `x-ess-outcomes`: the outcomes it answers, in the order the command declares them, and
//! none for a standard refusal. A route without a stage-2 key keeps the stage-1 bytes. A
//! `WWW-Authenticate` challenge is a response header and nothing more: `security` and
//! `securitySchemes` stay omitted (decided for stage 2, answer 8).
//!
//! # Determinism
//!
//! No clock, no RNG, and every collection a [`BTreeMap`], a [`BTreeSet`] or a [`Vec`] built from
//! one. `tests/openapi.rs` generates twice and compares bytes, because that is the only form of this
//! claim that is worth anything (review F8).

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{
    CommandHandle, EssIr, ResolvedActor, ResolvedCommand, ResolvedComponent, ResolvedCondition,
    ResolvedEffect, ResolvedError, ResolvedEvent, ResolvedField, ResolvedOutcome, ResolvedTypeRef,
    ResolvedView, TypeHandle, ViewHandle,
};
use ess_domain::binding::Delivery;
use ess_domain::component::Reach;
use ess_domain::view::Consistency;
use serde_json::{json, Map, Value};

use crate::artifact::{Artifact, Generator};
use crate::http::{self, ANSWERED, CONFLICT, FORBIDDEN, NO_INPUT, READ, REFUSED, UPSTREAM};
use ess_compiler::refs::{ActorRef, BindingRef, CommandRef, ComponentRef, EssSemanticRef};

use crate::provenance::{Provenance, ProvenanceMint, SlicedProvenance};
use crate::schema::types::{self, Message, Node};

/// The specification version emitted. 3.1 rather than 3.0 because 3.1's schema dialect is JSON
/// Schema 2020-12, which is the dialect the type mapping below actually produces — a 3.0 document
/// would have to lie about `$ref` siblings, `null` and `contentEncoding`.
const VERSION: &str = "3.1.0";

/// The one media type. The model describes payloads as typed fields, and nothing in it selects an
/// encoding, so there is exactly one and it is the one every other projection here uses.
const MEDIA_TYPE: &str = "application/json";

/// The request media type of an `ess-http/1` route encoded `form` (RFC 6749 §4.1.3, Appendix B):
/// the only place a request is not [`MEDIA_TYPE`], and only because a binding says so.
const FORM_MEDIA_TYPE: &str = "application/x-www-form-urlencoded";

/// The header a redirect's URI travels in; a binding never declares it, `redirect` writes it.
const LOCATION: &str = "Location";

/// The header that carries a caller's invocation identity, spelt as the ecosystem spells it.
const IDEMPOTENCY_KEY: &str = "Idempotency-Key";

/// What `x-ess-route-source` says on an operation an `ess-http/1` binding routes: its method and
/// path are the binding's, not derived. The `AsyncAPI` projection says `x-ess-address-source:
/// transport` of a channel the same way.
const ROUTE_SOURCE: &str = "binding";

/// The response property naming which declared branch was taken.
const OUTCOME: &str = "outcome";

/// The response property listing what the branch published.
const PUBLISHED: &str = "published";

/// The keyword a reference is spelt under, in every dialect this crate emits.
const REFERENCE: &str = "$ref";

/// The pointer prefix [`types`] emits, because the `schema` projection keeps its definitions in
/// `$defs`. No `components.schemas` document uses it, so every one of them is retargeted.
const DEFS: &str = "#/$defs/";

/// Who the specification permits to invoke each command, as [`EssIr::grants`] answers it.
///
/// `may` is declared on the actor, so the question an operation asks — who may invoke *this* command
/// — is the inversion, and the IR is where the inversion lives so that two projections cannot
/// disagree about it.
type Grants<'a> = BTreeMap<&'a CommandHandle, Vec<&'a ResolvedActor>>;

/// an `OpenAPI` document for every command a component accepts.
pub struct OpenApi;

impl Generator for OpenApi {
    fn name(&self) -> &'static str {
        "openapi"
    }

    fn describes(&self) -> &'static str {
        "an OpenAPI document for every command a component accepts"
    }

    fn directory(&self) -> &'static str {
        "openapi"
    }

    fn generate(&self, ir: &EssIr, mint: &ProvenanceMint) -> Vec<Artifact> {
        documents(ir, mint, None)
    }
}

/// The same documents, with the methods and paths an `ess-http/1` binding declares
/// (`docs/design/http-presentation-binding.md`, stage 1).
///
/// A bound operation is served at the binding's method and path and says so with
/// `x-ess-route-source: binding`, as a bound channel says `x-ess-address-source: transport` in
/// `AsyncAPI`. Each path variable is a required `in: path` parameter whose schema is the input
/// field's or view parameter's, a command's mapped input field leaves the request body (and a
/// command whose every field is mapped has none), and a view's mapped parameter leaves the query.
/// An operation the binding does not name keeps its derived route and its bytes, so a binding that
/// routes nothing changes no byte. [`Binding::new`](http::Binding::new) refuses a binding with an
/// open route before anything is generated.
pub struct BoundOpenApi(pub http::Binding);

impl Generator for BoundOpenApi {
    fn name(&self) -> &'static str {
        "openapi"
    }

    fn describes(&self) -> &'static str {
        "an OpenAPI document for every command a component accepts, at the methods and paths its \
         HTTP binding declares"
    }

    fn directory(&self) -> &'static str {
        "openapi"
    }

    fn generate(&self, ir: &EssIr, mint: &ProvenanceMint) -> Vec<Artifact> {
        documents(ir, mint, Some(&self.0))
    }
}

/// One document per component.
fn documents(ir: &EssIr, mint: &ProvenanceMint, binding: Option<&http::Binding>) -> Vec<Artifact> {
    // `ir.components()` is a `BTreeMap`, so this order is the same on every machine. The file name
    // comes from the component's own name rather than its wire name: a wire name is free text
    // and a file name is not, and this path is not something a consumer reads off the wire.
    ir.components()
        .values()
        .map(|component| {
            let sliced = component_slice(ir, component, mint);
            Artifact::sliced(
                format!("{}.yaml", component.name),
                render(
                    &document(ir, component, &sliced.provenance, binding),
                    &sliced.provenance,
                ),
                sliced.slice,
            )
        })
        .collect()
}

/// One component's document as JSON, which is what a server that answers it serves.
///
/// The same document the committed YAML projection carries — same title, same paths, same
/// schemas, same provenance — rendered in the other dialect `OpenAPI` 3.1 permits. Not a second
/// document: `tests/openapi.rs` parses both and compares the values, so a change that reached one
/// and not the other fails rather than shipping a server whose published contract disagrees with
/// the committed one.
///
/// JSON rather than YAML on the wire because every HTTP client parses JSON and `application/yaml`
/// is a media type a caller has to go and find a library for. The provenance is `info`'s
/// `x-ess-provenance`, which survives the crossing where the YAML file's comment header does not.
pub fn json(ir: &EssIr, component: &ResolvedComponent) -> String {
    bound_json(ir, component, None)
}

/// [`json()`], with an `ess-http/1` binding's routes in place of the derived ones: the document
/// [`BoundOpenApi`] commits, as JSON, which is what a server serving that binding publishes. With no
/// binding it is [`json()`], byte for byte.
pub fn bound_json(
    ir: &EssIr,
    component: &ResolvedComponent,
    binding: Option<&http::Binding>,
) -> String {
    let mint = ProvenanceMint::new(ir);
    let sliced = component_slice(ir, component, &mint);
    let mut out =
        serde_json::to_string_pretty(&document(ir, component, &sliced.provenance, binding))
            .unwrap_or_else(|error| panic!("an OpenAPI document serialises as JSON: {error}"));
    out.push('\n');
    out
}

/// The slice one component's document derives from: the component, every actor and every binding.
///
/// The component's own closure brings in what it accepts, owns and publishes — commands with their
/// outcomes, the domains, the input and error payload types. The two flat additions are the
/// constructs this document reads by *inversion*, which no forward walk from the component can
/// reach: grants live on the actors (`ir.grants()` is walked for every operation's security
/// answer), and the `Idempotency-Key` header exists exactly where some binding invokes a command
/// `at_least_once`. Every actor and every binding rather than the currently-relevant ones,
/// because "which ones are relevant" is itself an answer that changes — an actor granted its
/// first accepted command tomorrow must move this digest today.
fn component_slice(
    ir: &EssIr,
    component: &ResolvedComponent,
    mint: &ProvenanceMint,
) -> SlicedProvenance {
    let mut seeds: Vec<EssSemanticRef> = vec![ComponentRef::new(component.name.clone()).into()];
    seeds.extend(
        ir.actors()
            .keys()
            .map(|name| ActorRef::new(name.clone()).into()),
    );
    seeds.extend(
        ir.bindings()
            .keys()
            .map(|name| BindingRef::new(name.clone()).into()),
    );
    seeds.extend(
        ir.commands()
            .values()
            .filter(|command| {
                command
                    .outcomes
                    .iter()
                    .any(|outcome| !outcome.one_time_response.is_empty())
            })
            .map(|command| CommandRef::new(command.name.clone()).into()),
    );
    mint.of_seeds(seeds)
}

/// The document as YAML, behind the provenance header.
///
/// YAML rather than JSON so that the provenance can be a comment a person reads at the top of the
/// file, and repeated as `info.x-ess-provenance` so that it survives a tool reparsing the document —
/// a comment is stripped by everything that round-trips YAML, and provenance nobody can read after
/// one round trip is provenance nobody can audit (design §10).
fn render(document: &Document, provenance: &Provenance) -> String {
    let mut out = provenance.commented("#");
    out.push_str(
        &ess_yaml::to_string(document)
            .unwrap_or_else(|error| panic!("an OpenAPI document serialises: {error}")),
    );
    out
}

/// One component's whole surface.
fn document(
    ir: &EssIr,
    component: &ResolvedComponent,
    provenance: &Provenance,
    binding: Option<&http::Binding>,
) -> Document {
    Document {
        openapi: VERSION,
        one_time_response: crate::one_time_response::all(ir),
        info: Info {
            title: component
                .naming
                .display
                .clone()
                .unwrap_or_else(|| component.name.to_string()),
            summary: component.naming.summary.clone(),
            description: description(ir, component, binding),
            version: ir.version().to_string(),
            provenance: provenance.clone(),
            reached_by: (component.reached_by == Reach::Network)
                .then(|| component.reached_by.as_str()),
        },
        servers: binding
            .and_then(|binding| binding.ir().servers().get(component.name.as_str()))
            .cloned()
            .unwrap_or_default(),
        tags: tags(ir, component),
        paths: paths(ir, component, binding),
        components: Components {
            schemas: schemas(ir, component, binding),
        },
        entities: entity_model(ir, component),
    }
}

/// The sentence a document adds when one of its commands has a branch that answers with the
/// command's response (beyond10x/ess#424), and nothing otherwise, so every other document keeps its
/// bytes.
fn direct_answers(ir: &EssIr, component: &ResolvedComponent) -> &'static str {
    let answers = component.accepts.iter().any(|handle| {
        ir.command(handle)
            .outcomes
            .iter()
            .any(|outcome| http::answers_with_response(ir, outcome))
    });
    if answers {
        " A branch that returns the command's response is carried out in the request that took \
         it, so it is answered 200, and its body carries that response under `response`."
    } else {
        ""
    }
}

/// What a reader of this file needs to know before reading the paths.
///
/// The convention is restated in the artifact and not only in this module's documentation, because
/// the person who has to argue with the mapping is holding the generated file, not the generator.
fn description(
    ir: &EssIr,
    component: &ResolvedComponent,
    binding: Option<&http::Binding>,
) -> String {
    let mut text = format!(
        "The HTTP surface of `{}`, one of the components of `{}` {}.\n\n",
        component.name,
        ir.system(),
        ir.version()
    );
    text.push_str(
        "Every path here is one semantic command, so the method is always POST and the path is the \
         command's wire name under its domain's: a command is not a resource, and this document \
         does not invent one. A status code is the outcome the specification declares — 202 for a \
         branch that was taken, 422 for a refusal the input decides, 502 for a refusal decided \
         outside the request — and the `outcome` property of every response body names the branch. \
         Events emitted by a branch are published to consumers through the event transport, and \
         the `published` property of every response body lists them too, in publication order.",
    );
    text.push_str(direct_answers(ir, component));
    if http::grants_checked_on(ir, component) {
        text.push_str(
            "\n\nThe specification declares who may invoke what (`x-ess-may-invoke` on each \
             operation), so every command is sent as a declared actor. How a request proves which \
             actor it is belongs to the realization, which authenticates it and hands the surface \
             the caller; nothing in the request itself names one. The grant is checked before the \
             command runs, and a request authenticated as no actor, or as one without the grant, \
             is answered 403 with one standard refusal, the same for every command: \
             `{\"refused\": \"not granted\", \"actor\": <name or null>}`. A 403 a declared branch \
             answers carries `outcome` and the declared `error` instead, so a client tells the two \
             apart by the members present. A command no declared actor may invoke is refused to \
             every caller.",
        );
        text.push_str(if ir.grants_reads() {
            // beyond10x/ess#286: a model naming a view in a grant states who may read it.
            " A view an actor's grant names is read-checked the same way (`x-ess-may-read` on its \
             operation): a request authenticated as no actor, or as one the grant does not name, \
             is answered 403 with the same standard refusal before the view is read. A view no \
             grant names is open, and any caller may read what it publishes."
        } else {
            " Views are not grant-checked: a grant names commands, and any caller may read what a \
             view publishes."
        });
    }
    if component.reached_by == Reach::Network {
        text.push_str(
            "\n\nThis component declares `reached_by: network`, so its callers are not deployed \
             with it and this contract is the surface they reach it through. That declaration is \
             also why the views its domains declare have paths here: a GET under `views` per \
             projection, answering every row it holds. There is no page size, no cursor, no \
             ordering and no filter parameter, because the specification states none — a view's \
             filter is part of the projection, not of the request.",
        );
        if ir.views().values().any(|view| !view.params.is_empty()) {
            text.push_str(
                " A view that declares `params:` is the exception: each is a query parameter, \
                 described on its operation, and a view that declares `paging:` takes its page \
                 and size there.",
            );
        }
    }
    if http::bound_routes(ir, component, binding)
        .iter()
        .any(|route| route.declared.is_some())
    {
        text.push_str(
            "\n\nAn `ess-http/1` binding declares the method and path of every operation marked \
             `x-ess-route-source: binding`, so those are the protocol's and not derived from wire \
             names. Each variable of such a path is a required path parameter carrying the input \
             field or view parameter the binding maps to it, and a field carried in the path is \
             not in the request body. Every other operation keeps the method and path described \
             above.",
        );
    }
    if http::bound_routes(ir, component, binding)
        .iter()
        .any(|route| answered_by_binding(route.declared, refusals_of(binding)))
    {
        text.push_str(
            "\n\nThe binding also declares how some of those operations are encoded and answered: \
             a request body that is form-encoded, or input carried in the query string; the \
             status, body and headers of an outcome; a redirect; a view that answers its one row; \
             or a standard refusal's answer. Such an operation is described as the binding \
             declares it, and each response of such a command lists the outcomes it answers under \
             `x-ess-outcomes`, because a bare body carries no `outcome` naming the branch and no \
             `published` list. A `WWW-Authenticate` challenge is a response header: the \
             specification states what a refusal answers, not how a caller proves who it is, so \
             no security scheme is declared.",
        );
    }
    text
}

/// The standard refusals a binding answers its own way; `None` without a binding.
fn refusals_of(binding: Option<&http::Binding>) -> Option<&Refusals> {
    binding.map(|binding| binding.ir().refusals())
}

/// The standard refusals' overrides of a binding, by key.
type Refusals = BTreeMap<ess_http::RefusalKey, ess_http::Refusal>;

/// Whether an operation is answered as an `ess-http/1` binding declares beyond its route: its
/// route carries a stage-2 key, or the binding overrides a standard refusal, which applies to every
/// route it names. A derived route never is, and neither is a route of a binding with no stage-2
/// key, so both keep their stage-1 bytes.
fn answered_by_binding(declared: Option<&ess_http::Route>, refusals: Option<&Refusals>) -> bool {
    declared.is_some_and(|route| {
        route.request.is_some()
            || route.declares_answers()
            || route.body != ess_http::ViewBody::Rows
            || route.empty.is_some()
            || refusals.is_some_and(|refusals| !refusals.is_empty())
    })
}

/// One tag per domain any accepted command belongs to.
///
/// Keyed by the tag name while collecting, so two domains sharing a wire name produce one tag rather
/// than a document with a duplicate tag.
fn tags(ir: &EssIr, component: &ResolvedComponent) -> Vec<Tag> {
    let mut named: BTreeMap<String, Option<String>> = BTreeMap::new();
    for handle in &component.accepts {
        let domain = ir.domain(&ir.command(handle).domain);
        named.insert(
            domain.naming.wire_or(&domain.name).to_owned(),
            domain.naming.summary.clone(),
        );
    }
    named
        .into_iter()
        .map(|(name, description)| Tag { name, description })
        .collect()
}

/// The path each accepted command is exposed at, and the operation there.
///
/// Two commands can derive the same path — two domains may share a wire name, and `Naming::wire` is
/// free text with no charset the model enforces. When that happens *both* move to their qualified
/// names, rather than one keeping the short path: a path whose meaning depends on which other
/// commands exist is a path that changes when an unrelated command is added.
///
/// With a binding, an operation it names is at the binding's method and path instead, and one path
/// may then carry several methods ([`http::bound_routes`]).
fn paths(
    ir: &EssIr,
    component: &ResolvedComponent,
    binding: Option<&http::Binding>,
) -> BTreeMap<String, PathItem> {
    // Once for the document rather than once per operation: the IR holds `may` on the actor, so the
    // question a path asks — who may invoke *this* command — is an inversion, and inverting it per
    // operation would make the cost quadratic in a specification with many actors.
    let grants = ir.grants();
    let checked = http::grants_checked_on(ir, component);

    let mut out: BTreeMap<String, PathItem> = BTreeMap::new();
    for route in http::bound_routes(ir, component, binding) {
        // A standard refusal's override applies to the routes the binding names, never to a
        // derived one (decided for stage 2, answer 17).
        let refusals = route.declared.and(refusals_of(binding));
        let served = match route.serves {
            http::Served::Command(handle) => {
                operation(ir, handle, &grants, checked, route.declared, refusals)
            }
            http::Served::View(handle) => query(ir, handle, checked, route.declared, refusals),
        };
        *out.entry(route.path).or_default().slot(route.method) = Some(served);
    }
    out
}

/// The input fields or view parameters a bound route carries in its path, by declared name. Empty
/// for a derived route.
fn carried(declared: Option<&ess_http::Route>) -> BTreeSet<&str> {
    declared
        .map(|route| {
            route
                .parameters
                .values()
                .map(ess_http::Source::field)
                .collect()
        })
        .unwrap_or_default()
}

/// A bound route's path parameters, in the order their variables appear in its path: each required
/// and named after its variable, its schema the field's or parameter's own. Empty for a derived
/// route, whose path holds no variable.
///
/// `fields` are the command's input or the view's parameters, and `described` says what one of them
/// is where its declaration carries no summary.
fn path_parameters(
    fields: &[ResolvedField],
    declared: Option<&ess_http::Route>,
    described: impl Fn(&ResolvedField) -> String,
) -> Vec<Parameter> {
    let Some(declared) = declared else {
        return Vec::new();
    };
    declared
        .segments()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|segment| {
            let ess_http::Segment::Variable(variable) = segment else {
                return None;
            };
            let source = declared.parameters.get(&variable)?;
            let field = fields.iter().find(|field| field.name == source.field())?;
            Some(Parameter {
                name: variable,
                location: "path",
                description: field
                    .naming
                    .summary
                    .clone()
                    .unwrap_or_else(|| described(field)),
                required: true,
                style: None,
                explode: None,
                schema: embedded(&types::field(field)),
            })
        })
        .collect()
}

/// One command, as an operation.
fn operation(
    ir: &EssIr,
    handle: &CommandHandle,
    grants: &Grants<'_>,
    checks_grants: bool,
    declared: Option<&ess_http::Route>,
    refusals: Option<&Refusals>,
) -> Operation {
    let command = ir.command(handle);
    let domain = ir.domain(&command.domain);
    let request = declared.and_then(|route| route.request).unwrap_or_default();
    let carried = carried(declared);
    let mut parameters = path_parameters(&command.input, declared, |field| {
        format!(
            "The input field `{}`, carried in the path rather than the request body.",
            field.name
        )
    });
    if request.encoding == ess_http::Encoding::Query {
        parameters.extend(query_parameters(command, &carried));
    }
    parameters.extend(idempotency(ir, command));
    Operation {
        id: declared
            .and_then(|route| route.operation_id.clone())
            .unwrap_or_else(|| command.name.to_string()),
        route_source: declared.map(|_| ROUTE_SOURCE),
        periodic: ir
            .bindings()
            .values()
            .filter(|binding| binding.command == *handle && binding.cause.periodic().is_some())
            .cloned()
            .collect(),
        summary: Some(command.naming.display_or(&command.name).to_owned()),
        description: command.naming.summary.clone(),
        tags: vec![domain.naming.wire_or(&domain.name).to_owned()],
        may_invoke: may_invoke(handle, grants),
        may_read: Vec::new(),
        caller: http::caller_attributes(ir, command),
        accessors: ir
            .bindings()
            .values()
            .filter(|b| b.command == *handle)
            .flat_map(|b| {
                b.mapping.iter().filter_map(|m| {
                    if let ess_compiler::ir::ResolvedMappingValue::EventAccessor { plan, .. } =
                        &m.value
                    {
                        Some(BindingAccessor {
                            binding: b.name.to_string(),
                            event: b.cause.event()?.to_string(),
                            target: m.target.clone(),
                            path: plan.path(),
                            may_miss: plan.may_miss(),
                        })
                    } else {
                        None
                    }
                })
            })
            .collect(),
        consistency: None,
        parameters,
        request_body: request_body(command, &carried, request),
        // A route without a stage-2 key, and a derived one, keeps the stage-1 answers and bytes.
        responses: match declared {
            Some(route) if answered_by_binding(declared, refusals) => {
                bound_responses(ir, command, checks_grants, route, refusals)
            }
            _ => responses(ir, command, checks_grants),
        },
    }
}

/// A `query`-encoded command's input fields the path does not carry, each an `in: query`
/// parameter under its wire name, in declaration order.
///
/// A field is required unless it is `Optional` or a `List`, or the command declares
/// `input_absent:`, whose branch answers a request that carries no input at all. A `List` repeats
/// its key once per element (`style: form`, `explode: true`), the rule a view's parameters follow,
/// and an absent key is the empty list, as both generated servers read it.
fn query_parameters(command: &ResolvedCommand, carried: &BTreeSet<&str>) -> Vec<Parameter> {
    let answers_absent = command
        .outcomes
        .iter()
        .any(|outcome| outcome.condition == ResolvedCondition::InputAbsent);
    command
        .input
        .iter()
        .filter(|field| !carried.contains(field.name.as_str()))
        .map(|field| Parameter {
            name: types::wire_name(field).to_owned(),
            location: "query",
            description: field.naming.summary.clone().unwrap_or_else(|| {
                format!(
                    "The input field `{}`, carried in the query string: the route is encoded \
                     `query` and has no request body.",
                    field.name
                )
            }),
            required: !field.type_ref.is_optional() && !is_list(&field.type_ref) && !answers_absent,
            style: is_list(&field.type_ref).then_some("form"),
            explode: is_list(&field.type_ref).then_some(true),
            schema: embedded(&types::field(field)),
        })
        .collect()
}

/// What the standard refusal for an ungranted actor means, for the `403` response's description.
const NOT_GRANTED_MEANING: &str = "the standard refusal for an actor no grant admits: the request \
                                   was authenticated as no actor, or as one `x-ess-may-invoke` does \
                                   not list. It is checked before the command runs, so nothing was \
                                   read, written or published, and the same request from a granted \
                                   actor is decided by the command. Its body carries `refused` and \
                                   `actor`, never the `outcome` and `error` a declared branch \
                                   carries.";

/// What the standard refusal means on a read-granted view (beyond10x/ess#286), for its `403`
/// response's description.
const NOT_GRANTED_READ_MEANING: &str = "the standard refusal for an actor no grant admits: the \
                                        request was authenticated as no actor, or as one \
                                        `x-ess-may-read` does not list. It is checked before the \
                                        view is read, so no row was read, and the same request \
                                        from a granted actor answers the rows. Its body is the \
                                        one a command answers an ungranted actor.";

/// The standard refusal's body: `{"refused": "not granted", "actor": …}`, the same for every
/// command (beyond10x/ess#265).
fn not_granted_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["refused", "actor"],
        "properties": {
            "refused": {
                "type": "string",
                "enum": [http::NOT_GRANTED],
                "description": "The standard refusal for an actor no grant admits.",
            },
            "actor": {
                "type": ["string", "null"],
                "description": "The qualified name of the actor the request was authenticated \
                                as, or null where it was authenticated as none.",
            },
        },
    })
}

/// One view, as an operation.
///
/// It exists only because the component declares that something outside the process reaches it —
/// see [`http::routes`]. Its parameters are exactly the view's declared `params:`, each a query
/// parameter under its wire name (a paged view's page and size among them). A synthesized server
/// decodes them and hands them to the view's port; every code target refuses a paged view, so
/// none serves those two. What it does *not* carry is as deliberate: no
/// cursor, no ordering and no filter parameter, because the model states none of them. The view's
/// filter is declared in the specification and is a property of the projection, not of the
/// request: a caller supplies the values it reads as `param.<name>`, never the predicate.
fn query(
    ir: &EssIr,
    handle: &ViewHandle,
    checks_grants: bool,
    declared: Option<&ess_http::Route>,
    refusals: Option<&Refusals>,
) -> Operation {
    let view = ir.view(handle);
    let mut parameters = path_parameters(&view.params, declared, |param| {
        format!(
            "Read by the view's filter as `param.{}`; carried in the path rather than the query.",
            param.name
        )
    });
    parameters.extend(view_parameters(view, &carried(declared)));
    let domain = ir.domain(&view.domain);
    // A read-granted view on a surface that checks grants answers the standard refusal too
    // (beyond10x/ess#286); an open view, and every view of a model naming none, keeps its bytes.
    let read_checked = checks_grants && ir.read_granted(&view.name);
    let responses = match declared {
        Some(route) if answered_by_binding(declared, refusals) => {
            bound_view_responses(ir, view, read_checked, route, refusals)
        }
        _ => {
            let mut responses: BTreeMap<String, Response> = [(
                READ.to_owned(),
                Response::new(
                    view_description(ir, view),
                    Some(content(json!({"$ref": reference(&view_key(view))}))),
                ),
            )]
            .into_iter()
            .collect();
            if read_checked {
                responses.insert(
                    FORBIDDEN.to_owned(),
                    Response::new(
                        format!("No declared outcome: {NOT_GRANTED_READ_MEANING}"),
                        Some(content(not_granted_schema())),
                    ),
                );
            }
            responses
        }
    };
    Operation {
        id: declared
            .and_then(|route| route.operation_id.clone())
            .unwrap_or_else(|| view.name.to_string()),
        route_source: declared.map(|_| ROUTE_SOURCE),
        periodic: Vec::new(),
        summary: Some(view.naming.display_or(&view.name).to_owned()),
        description: view.naming.summary.clone(),
        tags: vec![domain.naming.wire_or(&domain.name).to_owned()],
        may_invoke: Vec::new(),
        may_read: if read_checked {
            ir.readers(&view.name)
                .map(|actor| actor.name.to_string())
                .collect()
        } else {
            Vec::new()
        },
        caller: Vec::new(),
        accessors: Vec::new(),
        consistency: Some(view.consistency.as_str()),
        parameters,
        request_body: None,
        responses,
    }
}

/// What the specification says about one view, as the response's required description.
fn view_description(ir: &EssIr, view: &ResolvedView) -> String {
    let mut parts = vec![format!(
        "Every row of `{}`, a projection of `{}`.",
        view.name,
        ir.entity(&view.source).name
    )];
    if view.paging.is_some() {
        parts[0] = format!(
            "One page of the rows of `{}`, a projection of `{}`, in the declared order.",
            view.name,
            ir.entity(&view.source).name
        );
    }
    if let Some(filter) = &view.filter {
        parts.push(format!("Contains the instances where `{filter}` holds."));
    } else {
        parts.push("Contains every instance.".to_owned());
    }
    if let Some(aggregation) = &view.aggregation {
        parts.push(aggregation.grouping_sentence());
        parts.push(format!("{}.", aggregation.clauses().join("; ")));
    }
    parts.extend(paging_sentences(view));
    parts.extend(absent_sentence(ir, view));
    parts.push(match view.consistency {
        Consistency::ReadYourWrites => {
            "Read-your-writes: a caller that has just issued a command sees its effect here."
                .to_owned()
        }
        Consistency::Eventual => {
            "Eventually consistent: a row may not yet reflect a command that has already returned."
                .to_owned()
        }
    });
    parts.join(" ")
}

/// The actors the specification permits to invoke this command, by qualified name.
///
/// The identity rather than the display name, for the reason every other `x-ess-` field carries one:
/// a display name is free text an author may change without changing who may do what, and a
/// generator reading this document needs the name the rest of the model uses. Empty for a command no
/// actor names — which is a legal specification and not a grant of "anyone", and the difference is
/// why the keyword is then absent rather than an empty list.
fn may_invoke(handle: &CommandHandle, grants: &Grants<'_>) -> Vec<String> {
    grants
        .get(handle)
        .map(|actors| {
            actors
                .iter()
                .map(|actor| actor.name.to_string())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default()
}

/// The `Idempotency-Key` header, when the model says this command may arrive twice.
///
/// The bindings are named in the description on purpose: a required header is an obligation, and the
/// reader's first question is who imposed it.
fn idempotency(ir: &EssIr, command: &ResolvedCommand) -> Option<Parameter> {
    let causes: Vec<String> = ir
        .bindings()
        .values()
        .filter(|binding| binding.command.name() == &command.name)
        .filter(|binding| match binding.delivery {
            // Matched rather than defaulted: a delivery guarantee that does not permit a repeat is
            // the arm the compiler asks about, and `at_most_once` is that arm. It permits no
            // second arrival, so it imposes no key: a required header here would be this generator
            // obliging a caller to name an invocation the model says happens once.
            Delivery::AtLeastOnce => true,
            Delivery::AtMostOnce => false,
        })
        .map(|binding| format!("`{}`", binding.name))
        .collect();
    if causes.is_empty() {
        return None;
    }
    Some(Parameter {
        name: IDEMPOTENCY_KEY.to_owned(),
        location: "header",
        description: format!(
            "{} {} this command with `delivery: at_least_once`, so it may arrive more than once for \
             one cause. The key names the invocation, so a repeat can be recognised as the same one \
             rather than performed again. It is required rather than optional because a retrying \
             caller could otherwise omit it, and the declared guarantee would have no mechanism \
             behind it.",
            list(&causes),
            if causes.len() == 1 {
                "invokes"
            } else {
                "invoke"
            }
        ),
        required: true,
        style: None,
        explode: None,
        schema: written(json!({"type": "string", "minLength": 1})),
    })
}

/// The command's input, as a body.
///
/// A command with no input gets no body at all rather than an empty object: `{}` in a request would
/// be a shape a client has to construct for no reason. Neither does one whose every input field a
/// bound route `carried` in its path, and a field carried there is not in the body. A route encoded
/// `query` has no body either, and one encoded `form` sends the same fields form-encoded.
fn request_body(
    command: &ResolvedCommand,
    carried: &BTreeSet<&str>,
    request: ess_http::Request,
) -> Option<RequestBody> {
    if request.encoding == ess_http::Encoding::Query
        || command
            .input
            .iter()
            .all(|field| carried.contains(field.name.as_str()))
    {
        return None;
    }
    let schema = json!({"$ref": reference(&format!("{}.Input", command.name))});
    let (description, content) = match request.encoding {
        ess_http::Encoding::Form => (
            format!(
                "The input `{}` declares, form-encoded as the `ess-http/1` binding declares: UTF-8, \
                 each field under its wire name, a list repeating its key once per element.{}",
                command.name,
                if request.unknown == ess_http::Unknown::Ignore {
                    " A parameter no field names is ignored."
                } else {
                    ""
                }
            ),
            content_as(FORM_MEDIA_TYPE, schema),
        ),
        ess_http::Encoding::Json | ess_http::Encoding::Query => (
            format!("The input `{}` declares.", command.name),
            content(schema),
        ),
    };
    Some(RequestBody {
        description,
        // One answer for this document and every server that reads the body.
        required: http::body_required_without(command, carried),
        content,
    })
}

/// One response per status the command's outcomes reach, and the `501` every served command can
/// answer — and, where the model declares an actor, the standard refusal for one no grant admits,
/// under `403` (beyond10x/ess#265).
///
/// A command whose caller decides a branch already answers `403` with that branch; the two share
/// the status, and the schema is `oneOf` the declared branch (or branches, with their
/// discriminator kept inside) and the standard refusal. They cannot be confused: a branch's body
/// requires `outcome`, the refusal's is closed over `refused` and `actor`.
fn responses(
    ir: &EssIr,
    command: &ResolvedCommand,
    checks_grants: bool,
) -> BTreeMap<String, Response> {
    let mut out: BTreeMap<String, Response> = by_status(ir, command)
        .into_iter()
        .map(|(status, outcomes)| {
            let names: Vec<String> = outcomes
                .iter()
                .map(|outcome| format!("`{}`", outcome.name))
                .collect();
            let schema = if let [only] = outcomes[..] {
                json!({"$ref": reference(&response_key(command, only))})
            } else {
                json!({
                    "oneOf": outcomes
                        .iter()
                        .map(|outcome| json!({"$ref": reference(&response_key(command, outcome))}))
                        .collect::<Vec<Value>>(),
                    // The branch is what a shared status would otherwise lose, so the document says
                    // in machine-readable form which property distinguishes them.
                    "discriminator": {
                        "propertyName": OUTCOME,
                        "mapping": outcomes
                            .iter()
                            .map(|outcome| (
                                outcome.name.to_string(),
                                reference(&response_key(command, outcome)),
                            ))
                            .collect::<BTreeMap<String, String>>(),
                    },
                })
            };
            let mut description = format!(
                "{} {}: {}",
                if names.len() == 1 {
                    "Outcome"
                } else {
                    "Outcomes"
                },
                list(&names),
                meaning(status)
            );
            let schema = if checks_grants && status == FORBIDDEN {
                description.push_str(" Or: ");
                description.push_str(NOT_GRANTED_MEANING);
                json!({"oneOf": [schema, not_granted_schema()]})
            } else {
                schema
            };
            (
                status.to_owned(),
                Response::new(description, Some(content(schema))),
            )
        })
        .collect();
    if checks_grants && !out.contains_key(FORBIDDEN) {
        out.insert(
            FORBIDDEN.to_owned(),
            Response::new(
                format!("No declared outcome: {NOT_GRANTED_MEANING}"),
                Some(content(not_granted_schema())),
            ),
        );
    }
    out.insert(
        http::UNFINISHED.to_owned(),
        Response::new(
            UNFINISHED_MEANING.to_owned(),
            Some(content(unfinished_schema())),
        ),
    );
    out
}

/// What the unfinished realization means, for the `501` response's description.
const UNFINISHED_MEANING: &str = "No declared outcome: the realization is unfinished. Either a \
                                  port this command runs reported an unmet obligation, and nothing \
                                  was written: `committed` is `false`. Or the command's effect was \
                                  committed and delivering what this command published to a \
                                  binding failed: `committed` is `true`, the effect and its events \
                                  stand, and each binding whose delivery failed keeps its event \
                                  and is attempted again on a later delivery, while no other \
                                  binding receives it twice. Do not retry: after a failed delivery \
                                  a retry performs the command a second time.";

/// The unfinished realization's body: `{"refused": <words>, "committed": <bool>}`.
fn unfinished_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["refused", "committed"],
        "properties": {
            "refused": {
                "type": "string",
                "description": "What was left unfinished, in words.",
            },
            "committed": {
                "type": "boolean",
                "description": "`true` when the command's effect and events were \
                                committed and delivering what it published failed; \
                                `false` when an unmet obligation stopped the command \
                                before anything was written.",
            },
        },
    })
}

/// One answer an operation whose route an `ess-http/1` binding answers its own way can give
/// ([`bound_responses`], [`bound_view_responses`]).
enum Answer<'a> {
    /// A declared outcome, with the binding's answer for it: `None` when the binding does not list
    /// it, which keeps its derived status and the envelope.
    Outcome(&'a ResolvedOutcome, Option<&'a ess_http::Outcome>),
    /// A standard refusal the binding overrides.
    Refusal(ess_http::RefusalKey, &'a ess_http::Refusal),
    /// The standard refusal for an actor no grant admits, as a command answers it today.
    NotGranted,
    /// The same refusal on a read-granted view.
    NotGrantedRead,
    /// The unfinished realization, as today.
    Unfinished,
    /// A `rows` view's rows.
    Rows(&'a ResolvedView),
    /// A `single` view's one row, bare.
    Row(&'a ResolvedView),
    /// A `single` view's answer when it holds no row: no body.
    Empty(&'a ResolvedView),
}

/// One `oneOf` member of a response, and what tells it apart from the others where a
/// discriminator can: the property, its value and the member's pointer.
struct Member {
    schema: Value,
    discriminated: Option<(String, String, String)>,
}

/// One response per status a command whose route an `ess-http/1` binding answers its own way
/// can answer (`docs/design/http-presentation-binding.md`, "What `OpenAPI` writes (k4)").
///
/// A listed outcome answers its declared status, body and headers; an unlisted one keeps its
/// derived status and the envelope. A standard refusal the binding overrides answers its declared
/// status instead of today's: `not_granted` where the surface checks grants, `malformed` on a
/// command with input, `unfinished` on every command. Every response carries `x-ess-outcomes`.
fn bound_responses(
    ir: &EssIr,
    command: &ResolvedCommand,
    checks_grants: bool,
    route: &ess_http::Route,
    refusals: Option<&Refusals>,
) -> BTreeMap<String, Response> {
    let mut by_status: BTreeMap<String, Vec<Answer<'_>>> = BTreeMap::new();
    for outcome in &command.outcomes {
        let listed = route.answer(outcome.name.as_str(), outcome.error.is_some());
        let status = listed.map_or_else(
            || http::outcome_status(ir, outcome).to_owned(),
            |listed| listed.status.to_string(),
        );
        by_status
            .entry(status)
            .or_default()
            .push(Answer::Outcome(outcome, listed));
    }
    // Each standard refusal the command can answer, with today's answer where it has one: the
    // binding's override replaces it, at the status the override declares.
    let standard = [
        (
            ess_http::RefusalKey::NotGranted,
            checks_grants,
            Some((FORBIDDEN, Answer::NotGranted)),
        ),
        (
            ess_http::RefusalKey::Malformed,
            !command.input.is_empty(),
            None,
        ),
        (
            ess_http::RefusalKey::Unfinished,
            true,
            Some((http::UNFINISHED, Answer::Unfinished)),
        ),
    ];
    for (key, answers, today) in standard {
        if !answers {
            continue;
        }
        match (refusals.and_then(|refusals| refusals.get(&key)), today) {
            (Some(refusal), _) => by_status
                .entry(refusal.status.to_string())
                .or_default()
                .push(Answer::Refusal(key, refusal)),
            (None, Some((status, today))) => {
                by_status.entry(status.to_owned()).or_default().push(today);
            }
            (None, None) => {}
        }
    }
    by_status
        .into_iter()
        .map(|(status, answers)| {
            let response = bound_response(ir, Some(command), &status, &answers, true);
            (status, response)
        })
        .collect()
}

/// The responses of a view whose route an `ess-http/1` binding answers its own way: `single`, or
/// a binding that overrides a standard refusal the view can answer — `not_granted` where its read
/// is checked, `malformed` where it declares a parameter. A view has no outcome, so its responses
/// carry no `x-ess-outcomes`.
fn bound_view_responses<'a>(
    ir: &EssIr,
    view: &'a ResolvedView,
    read_checked: bool,
    route: &ess_http::Route,
    refusals: Option<&'a Refusals>,
) -> BTreeMap<String, Response> {
    let mut by_status: BTreeMap<String, Vec<Answer<'a>>> = BTreeMap::new();
    let overridden = |key| refusals.and_then(|refusals| refusals.get(&key).map(|r| (key, r)));
    match (route.body, route.empty) {
        (ess_http::ViewBody::Single, Some(empty)) => {
            by_status
                .entry(READ.to_owned())
                .or_default()
                .push(Answer::Row(view));
            by_status
                .entry(empty.status.to_string())
                .or_default()
                .push(Answer::Empty(view));
        }
        // Compiling refuses `single` without `empty` (`ESS-HTTP-030`).
        _ => by_status
            .entry(READ.to_owned())
            .or_default()
            .push(Answer::Rows(view)),
    }
    if read_checked {
        match overridden(ess_http::RefusalKey::NotGranted) {
            Some((key, refusal)) => by_status
                .entry(refusal.status.to_string())
                .or_default()
                .push(Answer::Refusal(key, refusal)),
            None => by_status
                .entry(FORBIDDEN.to_owned())
                .or_default()
                .push(Answer::NotGrantedRead),
        }
    }
    if !view.params.is_empty() {
        if let Some((key, refusal)) = overridden(ess_http::RefusalKey::Malformed) {
            by_status
                .entry(refusal.status.to_string())
                .or_default()
                .push(Answer::Refusal(key, refusal));
        }
    }
    by_status
        .into_iter()
        .map(|(status, answers)| {
            let response = bound_response(ir, None, &status, &answers, false);
            (status, response)
        })
        .collect()
}

/// One answer of a status, rendered: its body, the label its headers are described under, the
/// headers it writes, and what it means. An unlisted outcome says what it means once with the
/// others at its derived status, under `unlisted`.
struct Rendered {
    member: Option<Member>,
    label: String,
    headers: BTreeMap<String, HeaderDoc>,
    sentence: Option<String>,
    unlisted: Option<String>,
}

impl Rendered {
    /// A standard answer: a body, if any, and a sentence; no headers.
    fn standard(schema: Option<Value>, sentence: String) -> Self {
        Self {
            member: schema.map(|schema| Member {
                schema,
                discriminated: None,
            }),
            label: String::new(),
            headers: BTreeMap::new(),
            sentence: Some(sentence),
            unlisted: None,
        }
    }
}

/// One status's response: the bodies of its answers (one, or a `oneOf` of them), their headers,
/// and with `outcomes` the declared outcomes it answers under `x-ess-outcomes`.
///
/// A `oneOf` keeps a discriminator where every member is a pointer told apart by one property:
/// `outcome` between envelopes, `<member>` between error objects. A standard refusal's body
/// beside them is told apart by its own closed members, and the discriminator is then left out,
/// because `OpenAPI` reads one only over pointers.
fn bound_response(
    ir: &EssIr,
    command: Option<&ResolvedCommand>,
    status: &str,
    answers: &[Answer<'_>],
    outcomes: bool,
) -> Response {
    let mut members: Vec<Member> = Vec::new();
    let mut headers: Vec<(String, BTreeMap<String, HeaderDoc>)> = Vec::new();
    let mut unlisted: Vec<String> = Vec::new();
    let mut sentences: Vec<String> = Vec::new();
    for answer in answers {
        let rendered = rendered(ir, command, status, answer);
        members.extend(rendered.member);
        headers.push((rendered.label, rendered.headers));
        sentences.extend(rendered.sentence);
        unlisted.extend(rendered.unlisted);
    }
    if !unlisted.is_empty() {
        sentences.insert(
            0,
            format!(
                "{} {}: {}",
                if unlisted.len() == 1 {
                    "Outcome"
                } else {
                    "Outcomes"
                },
                list(&unlisted),
                meaning(status)
            ),
        );
    }
    Response {
        description: sentences.join(" "),
        headers: merged_headers(headers),
        content: one_of(members).map(content),
        outcomes: outcomes.then(|| {
            answers
                .iter()
                .filter_map(|answer| match answer {
                    Answer::Outcome(outcome, _) => Some(outcome.name.to_string()),
                    _ => None,
                })
                .collect()
        }),
    }
}

/// One answer of a status, rendered.
fn rendered(
    ir: &EssIr,
    command: Option<&ResolvedCommand>,
    status: &str,
    answer: &Answer<'_>,
) -> Rendered {
    match answer {
        Answer::Outcome(outcome, listed) => {
            // An outcome is only ever an answer of a command.
            let command = command.expect("an outcome belongs to a command");
            rendered_outcome(ir, command, status, outcome, *listed)
        }
        Answer::Refusal(key, refusal) => Rendered {
            member: match &refusal.body {
                ess_http::RefusalBody::None => None,
                ess_http::RefusalBody::Literal { member, text } => Some(Member {
                    schema: json!({
                        "type": "object",
                        "additionalProperties": false,
                        "required": [member],
                        "properties": {member: {"type": "string", "const": text}},
                    }),
                    discriminated: None,
                }),
            },
            label: format!("the `{key}` refusal"),
            headers: refusal
                .headers
                .iter()
                .map(|(name, header)| {
                    (
                        name.clone(),
                        command.map_or_else(
                            || refusal_header_doc(header),
                            |command| header_doc(ir, command, None, header),
                        ),
                    )
                })
                .collect(),
            sentence: Some(refusal_sentence(*key, refusal)),
            unlisted: None,
        },
        Answer::NotGranted => Rendered::standard(
            Some(not_granted_schema()),
            format!("No declared outcome: {NOT_GRANTED_MEANING}"),
        ),
        Answer::NotGrantedRead => Rendered::standard(
            Some(not_granted_schema()),
            format!("No declared outcome: {NOT_GRANTED_READ_MEANING}"),
        ),
        Answer::Unfinished => {
            Rendered::standard(Some(unfinished_schema()), UNFINISHED_MEANING.to_owned())
        }
        Answer::Rows(view) => Rendered::standard(
            Some(json!({"$ref": reference(&view_key(view))})),
            view_description(ir, view),
        ),
        Answer::Row(view) => Rendered::standard(
            Some(json!({"$ref": reference(&row_schema_key(view))})),
            format!(
                "The one row of `{}`, bare, as the `ess-http/1` binding declares (`body: \
                 single`): its filter selects by the identity of `{}`, so it holds one row at \
                 most.",
                view.name,
                ir.entity(&view.source).name
            ),
        ),
        Answer::Empty(view) => Rendered::standard(
            None,
            format!(
                "`{}` holds no row for these parameters, as the `ess-http/1` binding declares \
                 (`empty`). No body.",
                view.name
            ),
        ),
    }
}

/// One declared outcome's answer: the binding's, where it lists the outcome, else the envelope at
/// its derived status.
fn rendered_outcome(
    ir: &EssIr,
    command: &ResolvedCommand,
    status: &str,
    outcome: &ResolvedOutcome,
    listed: Option<&ess_http::Outcome>,
) -> Rendered {
    let body = listed.map_or(ess_http::Body::Envelope, |listed| listed.body.clone());
    let key = reference(&response_key(command, outcome));
    let member = match &body {
        ess_http::Body::Envelope => Some(Member {
            schema: json!({"$ref": key}),
            discriminated: Some((OUTCOME.to_owned(), outcome.name.to_string(), key)),
        }),
        ess_http::Body::Response => Some(Member {
            schema: json!({"$ref": reference(&format!("{}.Result", command.name))}),
            discriminated: None,
        }),
        ess_http::Body::ResponseField(_) => Some(Member {
            schema: json!({"$ref": key}),
            discriminated: None,
        }),
        ess_http::Body::Error { member } => Some(Member {
            schema: json!({"$ref": key}),
            discriminated: outcome.error.as_ref().map(|error| {
                let error = ir.error(error);
                (
                    member.clone(),
                    error.naming.wire_or(&error.name).to_owned(),
                    key,
                )
            }),
        }),
        ess_http::Body::None => None,
    };
    let mut headers = BTreeMap::new();
    if let Some(listed) = listed {
        for (name, header) in &listed.headers {
            headers.insert(name.clone(), header_doc(ir, command, Some(outcome), header));
        }
        if let Some(redirect) = &listed.redirect {
            headers.insert(LOCATION.to_owned(), location_doc(redirect));
        }
    }
    Rendered {
        member,
        label: format!("`{}`", outcome.name),
        headers,
        sentence: listed.map(|listed| listed_sentence(ir, outcome, listed, status)),
        unlisted: listed.is_none().then(|| format!("`{}`", outcome.name)),
    }
}

/// The schema of a response whose answers have these bodies: none, one, or a `oneOf` of them.
fn one_of(members: Vec<Member>) -> Option<Value> {
    match members.len() {
        0 => None,
        1 => members.into_iter().next().map(|member| member.schema),
        _ => {
            let property = members.first().and_then(|member| {
                member
                    .discriminated
                    .as_ref()
                    .map(|(property, ..)| property.clone())
            });
            let discriminator = property.filter(|property| {
                members.iter().all(|member| {
                    member
                        .discriminated
                        .as_ref()
                        .is_some_and(|(other, ..)| other == property)
                })
            });
            let mut schema = json!({});
            if let Some(property) = discriminator {
                let mapping: BTreeMap<String, String> = members
                    .iter()
                    .filter_map(|member| member.discriminated.clone())
                    .map(|(_, value, pointer)| (value, pointer))
                    .collect();
                schema["discriminator"] = json!({"propertyName": property, "mapping": mapping});
            }
            schema["oneOf"] =
                Value::Array(members.into_iter().map(|member| member.schema).collect());
            // `oneOf` first, as an unbound document writes it.
            let mut ordered = Map::new();
            for key in ["oneOf", "discriminator"] {
                if let Some(value) = schema.get(key) {
                    ordered.insert(key.to_owned(), value.clone());
                }
            }
            Some(Value::Object(ordered))
        }
    }
}

/// What a listed outcome answers, for its response's description.
fn listed_sentence(
    ir: &EssIr,
    outcome: &ResolvedOutcome,
    listed: &ess_http::Outcome,
    status: &str,
) -> String {
    let body = match &listed.body {
        ess_http::Body::Envelope => "ESS's own body: `outcome` names the branch and `published` \
                                     lists what it published."
            .to_owned(),
        ess_http::Body::Response => "the command's response, bare: no `outcome` names the branch \
                                     and no `published` lists what it published."
            .to_owned(),
        ess_http::Body::ResponseField(field) => format!(
            "the command's response field `{field}`, bare: its value alone, with no object around \
             it."
        ),
        ess_http::Body::Error { member } => {
            let wire = outcome.error.as_ref().map_or("", |error| {
                let error = ir.error(error);
                error.naming.wire_or(&error.name)
            });
            format!(
                "an object whose `{member}` is `{wire}`, beside the error's own fields under \
                 their wire names."
            )
        }
        ess_http::Body::None if listed.redirect.is_some() => {
            "no body: the caller is sent to `Location`.".to_owned()
        }
        ess_http::Body::None => "no body.".to_owned(),
    };
    format!(
        "Outcome `{}`, answered {status} as the `ess-http/1` binding declares, with {body}",
        outcome.name
    )
}

/// What an overridden standard refusal answers, for its response's description.
fn refusal_sentence(key: ess_http::RefusalKey, refusal: &ess_http::Refusal) -> String {
    let what = match key {
        ess_http::RefusalKey::Malformed => "a request whose input cannot be read",
        ess_http::RefusalKey::NotGranted => "a caller no grant admits",
        ess_http::RefusalKey::Unfinished => "a realization that is unfinished",
    };
    let body = match &refusal.body {
        ess_http::RefusalBody::None => "no body".to_owned(),
        ess_http::RefusalBody::Literal { member, text } => {
            format!("a body whose one member `{member}` is `{text}`")
        }
    };
    format!(
        "No declared outcome: the `{key}` refusal, for {what}, answered as the `ess-http/1` \
         binding declares, with {body}."
    )
}

/// One header an answer writes, before the answers of one status are merged.
#[derive(Debug, Clone, PartialEq)]
struct HeaderDoc {
    description: String,
    required: bool,
    schema: Value,
}

/// A declared header of a command's answer: `outcome` is `None` for a standard refusal's.
fn header_doc(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: Option<&ResolvedOutcome>,
    header: &ess_http::Header,
) -> HeaderDoc {
    let absent = |source: &ess_http::AnswerSource| may_be_absent(ir, command, outcome, source);
    match header {
        ess_http::Header::Literal(text) => HeaderDoc {
            description: format!("Always `{text}`."),
            required: true,
            schema: json!({"type": "string", "const": text}),
        },
        ess_http::Header::Source(source) => HeaderDoc {
            description: if absent(source) {
                format!(
                    "The value of `{source}`, written as text; not written when `{source}` is \
                     absent."
                )
            } else {
                format!("The value of `{source}`, written as text.")
            },
            required: !absent(source),
            schema: json!({"type": "string"}),
        },
        ess_http::Header::Challenge(challenge) => HeaderDoc {
            description: challenge_description(challenge, absent),
            required: true,
            schema: json!({"type": "string"}),
        },
    }
}

/// A standard refusal's header on a view, which has no command to read a source from; compiling
/// refuses a source in a refusal's header (`ESS-HTTP-026`).
fn refusal_header_doc(header: &ess_http::Header) -> HeaderDoc {
    match header {
        ess_http::Header::Literal(text) => HeaderDoc {
            description: format!("Always `{text}`."),
            required: true,
            schema: json!({"type": "string", "const": text}),
        },
        ess_http::Header::Source(source) => HeaderDoc {
            description: format!("The value of `{source}`, written as text."),
            required: true,
            schema: json!({"type": "string"}),
        },
        ess_http::Header::Challenge(challenge) => HeaderDoc {
            description: challenge_description(challenge, |_| false),
            required: true,
            schema: json!({"type": "string"}),
        },
    }
}

/// One RFC 9110 §11.6.1 challenge, param by param.
fn challenge_description(
    challenge: &ess_http::Challenge,
    absent: impl Fn(&ess_http::AnswerSource) -> bool,
) -> String {
    let params: Vec<String> = challenge
        .params
        .iter()
        .map(|(name, param)| match param {
            ess_http::Param::Literal(text) => format!("`{name}` is `{text}`"),
            ess_http::Param::Source(source) if absent(source) => format!(
                "`{name}` is the value of `{source}`, and is left out when `{source}` is absent"
            ),
            ess_http::Param::Source(source) => format!("`{name}` is the value of `{source}`"),
        })
        .collect();
    if params.is_empty() {
        format!("One `{}` challenge (RFC 9110 §11.6.1).", challenge.scheme)
    } else {
        format!(
            "One `{}` challenge (RFC 9110 §11.6.1), each param a quoted string: {}.",
            challenge.scheme,
            list(&params)
        )
    }
}

/// A redirect's `Location`: the URI its source holds, with its parameters in the query or the
/// fragment.
fn location_doc(redirect: &ess_http::Redirect) -> HeaderDoc {
    let parameters: Vec<String> = redirect
        .parameters
        .iter()
        .map(|(name, source)| format!("`{name}` from `{source}`"))
        .collect();
    let placed = if parameters.is_empty() {
        String::new()
    } else {
        format!(
            ", with {} {}, form-encoded in name order; a parameter whose source is absent is \
             left out",
            list(&parameters),
            match redirect.mode {
                ess_http::RedirectMode::Query => {
                    "appended to its query, keeping a query it already has"
                }
                ess_http::RedirectMode::Fragment => "written as its fragment",
            }
        )
    };
    HeaderDoc {
        description: format!(
            "Where the caller is sent: the absolute URI in `{}`{placed}.",
            redirect.location
        ),
        required: true,
        schema: json!({"type": "string", "format": "uri"}),
    }
}

/// Whether `source` may be absent when `outcome` answers, so the header or param it fills is left
/// out: an `Optional` field, or an error field of the answer for an instance no record carries,
/// which has no payload. `error.wire` is always present.
fn may_be_absent(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: Option<&ResolvedOutcome>,
    source: &ess_http::AnswerSource,
) -> bool {
    let optional = |fields: &[ResolvedField], name: &str| {
        fields
            .iter()
            .find(|field| field.name == name)
            .is_some_and(|field| field.type_ref.is_optional())
    };
    match source {
        ess_http::AnswerSource::Input(name) => optional(&command.input, name),
        ess_http::AnswerSource::Response(name) => optional(&command.response, name),
        ess_http::AnswerSource::Error(name) => outcome.is_some_and(|outcome| {
            let unknown = crate::unknown_instance::unknown_instance_answer(ir, command)
                .is_some_and(|declared| declared.name == outcome.name);
            unknown
                || outcome
                    .error
                    .as_ref()
                    .is_some_and(|error| optional(&ir.error(error).fields, name))
        }),
        ess_http::AnswerSource::ErrorWire => false,
    }
}

/// The headers of one status, from the headers each of its answers writes, labelled by answer.
///
/// A header every answer writes alike is described once. One that differs between them, or that
/// only some write, is described answer by answer, required only where every answer writes it
/// always, and a `string` unless every answer gives it one schema.
fn merged_headers(
    answers: Vec<(String, BTreeMap<String, HeaderDoc>)>,
) -> BTreeMap<String, ResponseHeader> {
    let count = answers.len();
    let mut by_name: BTreeMap<String, Vec<(String, HeaderDoc)>> = BTreeMap::new();
    for (label, headers) in answers {
        for (name, doc) in headers {
            by_name.entry(name).or_default().push((label.clone(), doc));
        }
    }
    by_name
        .into_iter()
        .map(|(name, docs)| {
            let first = docs[0].1.clone();
            let alike = docs.iter().all(|(_, doc)| *doc == first);
            let doc = if docs.len() == count && alike {
                first
            } else {
                let mut description = docs
                    .iter()
                    .map(|(label, doc)| format!("For {label}: {}", doc.description))
                    .collect::<Vec<_>>()
                    .join(" ");
                if docs.len() < count {
                    description.push_str(" Not written by the other answers of this status.");
                }
                HeaderDoc {
                    description,
                    required: docs.len() == count && docs.iter().all(|(_, doc)| doc.required),
                    schema: if docs.iter().all(|(_, doc)| doc.schema == first.schema) {
                        first.schema
                    } else {
                        json!({"type": "string"})
                    },
                }
            };
            (
                name,
                ResponseHeader {
                    description: doc.description,
                    required: doc.required.then_some(true),
                    schema: written(doc.schema),
                },
            )
        })
        .collect()
}

/// An outcome's `{<member>: error}` body (decided for stage 2, answer 9): `<member>` a `const` of
/// the error's wire name, beside the error's own fields under their wire names, unchanged. For
/// the answer to an instance no record carries, which has no payload, only `<member>` is required.
fn error_object(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    member: &str,
) -> Fragment {
    let unknown = crate::unknown_instance::unknown_instance_answer(ir, command)
        .is_some_and(|declared| declared.name == outcome.name);
    let (wire, summary, fields) = outcome.error.as_ref().map_or_else(
        || (String::new(), None, Vec::new()),
        |error| {
            let error = ir.error(error);
            (
                error.naming.wire_or(&error.name).to_owned(),
                error.summary.clone(),
                error.fields.clone(),
            )
        },
    );
    let object = embedded(&types::object(&fields));
    let mut properties = ess_yaml::Mapping::new();
    let mut identity = json!({"type": "string", "const": wire});
    identity["description"] = Value::String(
        summary.unwrap_or_else(|| "The wire name of the error the branch reports.".to_owned()),
    );
    properties.insert(Fragment::String(member.to_owned()), written(identity));
    let mut required = vec![Fragment::String(member.to_owned())];
    if let Fragment::Mapping(object) = object {
        if let Some(Fragment::Mapping(declared)) = object.get("properties") {
            for (name, schema) in declared {
                properties.insert(name.clone(), schema.clone());
            }
        }
        if let (false, Some(Fragment::Sequence(declared))) = (unknown, object.get("required")) {
            required.extend(declared.iter().cloned());
        }
    }
    let mut out = ess_yaml::Mapping::new();
    out.insert("type".into(), "object".into());
    out.insert("additionalProperties".into(), Fragment::Bool(false));
    out.insert(
        "description".into(),
        Fragment::String(outcome_description(ir, outcome)),
    );
    out.insert("required".into(), Fragment::Sequence(required));
    out.insert("properties".into(), Fragment::Mapping(properties));
    Fragment::Mapping(out)
}

/// A command's outcomes, grouped by the status each is answered with ([`http::outcome_status`]),
/// in declaration order within a status.
fn by_status<'a>(
    ir: &EssIr,
    command: &'a ResolvedCommand,
) -> BTreeMap<&'static str, Vec<&'a ResolvedOutcome>> {
    let mut grouped: BTreeMap<&'static str, Vec<&'a ResolvedOutcome>> = BTreeMap::new();
    for outcome in &command.outcomes {
        grouped
            .entry(http::outcome_status(ir, outcome))
            .or_default()
            .push(outcome);
    }
    grouped
}

/// What a status means here, for the response's required description.
fn meaning(status: &str) -> &'static str {
    match status {
        REFUSED => {
            "the request was understood and refused on domain grounds. The body names the declared \
             error and carries whatever that error declares."
        }
        UPSTREAM => {
            "something outside the request refused. The input was acceptable, so the caller has \
             nothing to correct and a retry is meaningful."
        }
        CONFLICT => {
            "the input was acceptable and the subject is in a state this command does not act \
             from. Resending the same request changes nothing until something else moves it."
        }
        NO_INPUT => {
            "the request carried no input at all. The body names the declared error the command \
             reports for a request without its body."
        }
        FORBIDDEN => {
            "the caller is not one this branch admits: its guard compares the authenticated caller \
             with the input or the record. The same request from an admitted caller is accepted."
        }
        ANSWERED => {
            "the branch the specification declares for this input, carried out in this request: \
             the body carries the command's response under `response`. Events this branch emits \
             are published to consumers and listed under `published`."
        }
        _ => {
            "the branch the specification declares for this input. Events this branch emits are \
             published to consumers and listed under `published`."
        }
    }
}

/// Every schema the document's `$ref`s point at, and nothing else.
///
/// Every shape here comes from [`types`]; what this function decides is which of them the document
/// needs. The one schema shaped by this file is an outcome's response body, because an outcome is
/// not a message the model declares — it is this projection's rendering of a branch.
///
/// A command's `{command}.Input` is its request body: under a binding, the input less the fields
/// its route carries in the path, and absent when the path carries every one or the route is
/// encoded `query`. Their types stay, because the path and query parameters point at them.
///
/// Under a binding's stage-2 keys an outcome's body decides what it needs: the envelope its
/// `{command}.{outcome}.Response`, its events and its error's payload, as without one; `response`
/// the command's `{command}.Result`; `response.<field>` that field's schema under
/// `{command}.{outcome}.Response`; `{<member>: error}` the error object under
/// `{command}.{outcome}.Response`; `none` nothing. An outcome a route's `errors` answers needs
/// what that answer's body needs. A `single` view answers its row, so its rows
/// wrapper is not described.
fn schemas(
    ir: &EssIr,
    component: &ResolvedComponent,
    binding: Option<&http::Binding>,
) -> BTreeMap<String, Fragment> {
    let mut out: BTreeMap<String, Fragment> = BTreeMap::new();
    let mut roots: Vec<&TypeHandle> = Vec::new();

    for handle in &component.accepts {
        let route = binding.and_then(|binding| binding.route(http::Served::Command(handle)));
        command_schemas(ir, ir.command(handle), route, &mut out, &mut roots);
    }

    for route in http::routes(ir, component) {
        let http::Served::View(handle) = route.serves else {
            continue;
        };
        let view = ir.view(handle);
        if let Some(shape) = &view.shape {
            roots.push(shape);
        } else {
            out.insert(
                row_key(view),
                embedded(&types::message(&Message::of_view(view))),
            );
            roots.extend(types::field_leaves(&view.fields));
        }
        roots.extend(types::field_leaves(&view.params));
        let single = binding
            .and_then(|binding| binding.route(http::Served::View(handle)))
            .is_some_and(|route| route.body == ess_http::ViewBody::Single);
        if !single {
            out.insert(view_key(view), written(view_schema(view)));
        }
    }

    for (name, definition) in types::definitions(ir, roots) {
        out.insert(name, embedded(&definition));
    }
    out
}

/// The schemas one accepted command's operation needs, under its binding `route` if it has one,
/// and the named types they reach, as [`schemas`] describes.
fn command_schemas<'a>(
    ir: &'a EssIr,
    command: &'a ResolvedCommand,
    route: Option<&ess_http::Route>,
    out: &mut BTreeMap<String, Fragment>,
    roots: &mut Vec<&'a TypeHandle>,
) {
    let carried = carried(route);
    let encoding = route
        .and_then(|route| route.request)
        .unwrap_or_default()
        .encoding;
    let body: Vec<ResolvedField> = command
        .input
        .iter()
        .filter(|field| !carried.contains(field.name.as_str()))
        .cloned()
        .collect();
    if !body.is_empty() && encoding != ess_http::Encoding::Query {
        out.insert(
            format!("{}.Input", command.name),
            embedded(&types::message(&Message {
                fields: &body,
                ..Message::of_command(command)
            })),
        );
    }
    roots.extend(types::field_leaves(&command.input));
    let answered = |outcome: &ResolvedOutcome| {
        route
            .and_then(|route| route.answer(outcome.name.as_str(), outcome.error.is_some()))
            .map_or(ess_http::Body::Envelope, |listed| listed.body.clone())
    };
    if command
        .outcomes
        .iter()
        .any(|outcome| match answered(outcome) {
            ess_http::Body::Envelope => carries_result(ir, outcome),
            ess_http::Body::Response => true,
            ess_http::Body::ResponseField(_)
            | ess_http::Body::Error { .. }
            | ess_http::Body::None => false,
        })
    {
        out.insert(
            format!("{}.Result", command.name),
            embedded(&types::message(&Message::of_response(command))),
        );
        roots.extend(types::field_leaves(&command.response));
    }

    for outcome in &command.outcomes {
        match answered(outcome) {
            ess_http::Body::Envelope => {}
            ess_http::Body::Error { member } => {
                out.insert(
                    response_key(command, outcome),
                    error_object(ir, command, outcome, &member),
                );
                if let Some(error) = &outcome.error {
                    roots.extend(types::field_leaves(&ir.error(error).fields));
                }
                continue;
            }
            ess_http::Body::ResponseField(field) => {
                if let Some(declared) = command
                    .response
                    .iter()
                    .find(|declared| declared.name == field)
                {
                    out.insert(
                        response_key(command, outcome),
                        embedded(&types::field(declared)),
                    );
                    roots.extend(types::field_leaves(std::slice::from_ref(declared)));
                }
                continue;
            }
            ess_http::Body::Response | ess_http::Body::None => continue,
        }
        out.insert(
            response_key(command, outcome),
            written(outcome_schema(ir, command, outcome)),
        );
        for handle in &outcome.emits {
            let event = ir.event(handle);
            out.insert(
                event_key(event),
                embedded(&types::message(&Message::of_event(event))),
            );
            roots.extend(types::field_leaves(&event.fields));
        }
        if let Some(error) = &outcome.error {
            let declared = ir.error(error);
            if !declared.fields.is_empty() {
                out.insert(
                    error_key(declared),
                    embedded(&types::message(&Message::of_error(declared))),
                );
                roots.extend(types::field_leaves(&declared.fields));
            }
        }
    }
}

/// Every entity of the domains this component owns, and every named type they reach.
///
/// A root extension rather than `components.schemas`, and the reason is measured rather than
/// stylistic. `components.schemas` is the half of this document `ess import openapi` reads back —
/// the subset with a round-trip guarantee — and an entity's shape reaches constructs that subset
/// does not carry: `billing.invoice.Invoice` alone brings a `Map` field and a tagged union, and
/// putting them there makes this repository's own adapter refuse the document it generated —
/// "schema must declare a reference or one supported type", on `billing.invoice.Payee`. An
/// extension is ignored by a reader that does not know it, so the contract stays exactly what it
/// was and the model view sits beside it.
///
/// Entities and the types they reach share one table, keyed by qualified name, because they share
/// one pointer prefix: a reference inside it resolves the same way whether it names an aggregate or
/// a value.
fn entity_model(ir: &EssIr, component: &ResolvedComponent) -> BTreeMap<String, Fragment> {
    let mut out: BTreeMap<String, Fragment> = BTreeMap::new();
    let mut roots: Vec<&TypeHandle> = Vec::new();

    for domain in &component.owns {
        for handle in &ir.domain(domain).entities {
            let entity = ir.entity(handle);
            let fields = types::entity_fields(entity);
            out.insert(
                entity.name.to_string(),
                under(
                    &types::message(&Message::of_entity(
                        entity,
                        &fields,
                        types::carried(ir, entity, true),
                    )),
                    MODEL,
                ),
            );
            // Taken from the entity rather than from `fields`, which is a local clone: a handle
            // borrowed from it would not outlive the loop.
            roots.push(&entity.state_type);
            roots.extend(entity.identity.type_ref.named_leaves());
            roots.extend(types::field_leaves(&entity.fields));
        }
    }

    for (name, definition) in types::definitions(ir, roots) {
        out.insert(name, under(&definition, MODEL));
    }
    out
}

/// The response body for one view: the rows, under a key.
///
/// An object rather than a bare array, because a bare array is a body with nowhere to put a second
/// fact — and the first one a view will need is how much of the projection this answer is. There is
/// no such fact today and none is invented here; what the shape buys is that adding one later is
/// not a breaking change to every client.
fn view_schema(view: &ResolvedView) -> Value {
    let row = view
        .shape
        .as_ref()
        .map_or_else(|| row_key(view), ToString::to_string);
    let mut schema = json!({
        "type": "object",
        "additionalProperties": false,
        "description": format!("The rows of `{}`.", view.name),
        "required": ["rows"],
        "properties": {
            "rows": {
                "type": "array",
                "description": "Every row the projection holds, in the order it holds them.",
                "items": {"$ref": reference(&row)},
            },
        },
    });
    paged_schema(view, &mut schema);
    schema
}

/// The `components.schemas` key for one view's row.
fn row_key(view: &ResolvedView) -> String {
    format!("{}.Row", view.name)
}

/// The `components.schemas` key of the schema one row of a view has: its declared shape, or its
/// `{view}.Row`.
fn row_schema_key(view: &ResolvedView) -> String {
    view.shape
        .as_ref()
        .map_or_else(|| row_key(view), ToString::to_string)
}

/// The `components.schemas` key for one view's response body.
fn view_key(view: &ResolvedView) -> String {
    format!("{}.Response", view.name)
}

/// The response body for one outcome.
fn outcome_schema(ir: &EssIr, command: &ResolvedCommand, outcome: &ResolvedOutcome) -> Value {
    // The unknown-instance rule answers this same branch for an instance no record carries, and
    // that answer has no payload: nothing describes an instance that does not exist
    // (`docs/design/unknown-instance-seams.md`).
    let unknown = crate::unknown_instance::unknown_instance_answer(ir, command)
        .is_some_and(|declared| declared.name == outcome.name);
    let mut required = vec![Value::String(OUTCOME.to_owned())];
    let mut properties = Map::new();
    properties.insert(
        OUTCOME.to_owned(),
        json!({
            "const": outcome.name.as_str(),
            "description": "Which declared outcome the command took.",
        }),
    );
    required.push(Value::String(PUBLISHED.to_owned()));
    properties.insert(PUBLISHED.to_owned(), published_schema(ir, outcome, unknown));

    if let Some(handle) = &outcome.error {
        let declared = ir.error(handle);
        required.push(Value::String("error".to_owned()));
        let mut identity = json!({
            "const": declared.name.to_string(),
            "type": "string",
        });
        if let (Some(object), Some(summary)) = (identity.as_object_mut(), &declared.summary) {
            object.insert("description".to_owned(), Value::String(summary.clone()));
        }
        properties.insert("error".to_owned(), identity);

        if !declared.fields.is_empty() {
            if !unknown {
                required.push(Value::String("payload".to_owned()));
            }
            properties.insert(
                "payload".to_owned(),
                json!({"$ref": reference(&error_key(declared))}),
            );
        }
    }

    if carries_result(ir, outcome) {
        required.push(Value::String("response".to_owned()));
        properties.insert(
            "response".into(),
            json!({"$ref": reference(&format!("{}.Result", command.name))}),
        );
    }
    let mut schema = json!({
        "type": "object",
        "additionalProperties": false,
        "description": if unknown {
            format!(
                "{} For an instance no record carries, `payload` is absent, because an \
                 instance that does not exist has nothing for the error to describe.",
                outcome_description(ir, outcome)
            )
        } else {
            outcome_description(ir, outcome)
        },
        "required": required,
        "properties": properties,
    });
    if let Some(replay) = &outcome.replays {
        schema["x-ess-replays"] = json!({"command": command.name, "outcome": replay.origin});
    }
    schema
}

/// Whether an outcome's body carries the command's response under `response`: a retained result,
/// or a branch that [answers with it](http::answers_with_response) (beyond10x/ess#423).
fn carries_result(ir: &EssIr, outcome: &ResolvedOutcome) -> bool {
    outcome.retains_result || http::answers_with_response(ir, outcome)
}

/// The `published` property of one outcome's response body: the events the branch published, in
/// publication order, each as `{"event": <qualified name>, "payload": {…}}`.
///
/// One item per emit, in emit order, and nothing after them (`prefixItems`, `items: false`); none for
/// a branch that emits nothing, and none for the unknown-instance answer, which published nothing
/// because nothing happened. Every surface that answers a command — the served Rust and Go
/// applications, the transport-free `handle`, the web bridge — writes this list, so the contract
/// declares it rather than leaving each of them to contradict `additionalProperties: false`.
fn published_schema(ir: &EssIr, outcome: &ResolvedOutcome, unknown: bool) -> Value {
    let description = "Every event this branch published, in publication order: its qualified \
                       name and its payload.";
    if outcome.emits.is_empty() {
        return json!({"type": "array", "description": description, "maxItems": 0});
    }
    let entry = |event: &ResolvedEvent| {
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["event", "payload"],
            "properties": {
                "event": {"const": event.name.to_string(), "type": "string"},
                "payload": {"$ref": reference(&event_key(event))},
            },
        })
    };
    // One entry per emit, in emit order, and nothing after them: the list is ordered, so the same
    // events in another order, or one event twice, is not this branch's answer.
    let entries: Vec<Value> = outcome
        .emits
        .iter()
        .map(|handle| entry(ir.event(handle)))
        .collect();
    let count = entries.len();
    json!({
        "type": "array",
        "description": if unknown {
            format!(
                "{description} Empty for an instance no record carries, because nothing happened \
                 to it."
            )
        } else {
            description.to_owned()
        },
        "prefixItems": entries,
        "items": false,
        "minItems": if unknown { 0 } else { count },
        "maxItems": count,
    })
}

/// The `components.schemas` key for one event's payload.
fn event_key(event: &ResolvedEvent) -> String {
    format!("{}.Event", event.name)
}

/// What decides one outcome, as the sentence its `OpenAPI` description opens with.
fn condition_description(condition: &ResolvedCondition) -> String {
    match condition {
        ResolvedCondition::SubjectField {
            field,
            equals,
            predicate,
        } => format!("Subject {field} equals {equals}; input condition {predicate:?}."),
        ResolvedCondition::When { predicate } => {
            format!("Taken when `{predicate}` holds of the input.")
        }
        // Through `Display`, as `SubjectState` is: the stored fields are part of the contract.
        ResolvedCondition::SubjectPredicate { predicate, input } => format!(
            "Taken when the existing subject's stored fields satisfy `{predicate}`{}.",
            input.as_ref().map_or(String::new(), |guard| format!(
                " and `{guard}` holds of the input"
            )),
        ),
        ResolvedCondition::Related {
            via,
            entity,
            test,
            input,
            also,
        } => format!(
            "{}{}.",
            ess_compiler::ir::related_rows_sentence(via, entity, test, also),
            input.as_ref().map_or(String::new(), |guard| format!(
                " and `{guard}` holds of the input"
            )),
        ),
        ResolvedCondition::RelatedSet {
            selection,
            test,
            input,
        } => format!(
            "{}{}.",
            ess_compiler::ir::row_set_sentence(selection, test),
            input.as_ref().map_or(String::new(), |guard| format!(
                " and `{guard}` holds of the input"
            )),
        ),
        ResolvedCondition::SubjectState { state, predicate } => format!(
            "Taken when the existing subject is in {state}{}.",
            predicate.as_ref().map_or(String::new(), |guard| format!(
                " and `{guard}` holds of the input"
            )),
        ),
        ResolvedCondition::StateChange {
            changes,
            states,
            predicate,
        } => format!(
            "Taken when the existing subject is in {}{}{}.",
            list(&states.iter().map(ToString::to_string).collect::<Vec<_>>()),
            if *changes {
                ", so this branch's move changes the state it holds rather than restating it"
            } else {
                ", the state this branch's move arrives at, so it restates what is already held"
            },
            predicate.as_ref().map_or(String::new(), |guard| format!(
                ", and `{guard}` holds of the input"
            )),
        ),
        ResolvedCondition::Otherwise => {
            "Taken when no other outcome's condition matched.".to_owned()
        }
        ResolvedCondition::ExternalWhen { cause, predicate } => {
            format!("Eligible when {predicate}; externally decided: {cause}.")
        }
        ResolvedCondition::ExternalSubject {
            cause,
            predicate,
            input,
        } => format!(
            "Eligible when the existing subject's stored fields satisfy `{predicate}`{}; \
             externally decided: {cause}.",
            input.as_ref().map_or(String::new(), |guard| format!(
                " and `{guard}` holds of the input"
            )),
        ),
        ResolvedCondition::External { cause } => {
            format!("Decided outside the request: {cause}.")
        }
        ResolvedCondition::WrongState => {
            "Taken when the subject is in a state none of this command's declared moves start \
             from. Which states those are is the lifecycle's answer, not this command's."
                .to_owned()
        }
        ResolvedCondition::UnknownInstance => {
            "Taken when the request names an identity no record carries.".to_owned()
        }
        ResolvedCondition::InputAbsent => {
            "Taken when the request carries no input at all — an absent body, not `{}` — before \
             any input field is read."
                .to_owned()
        }
        ResolvedCondition::ExistingInstance => {
            "Taken when a record already carries the identity the request would create, and no \
             input-guarded refusal applies."
                .to_owned()
        }
    }
}

/// What the model says about one outcome, as prose.
fn outcome_description(ir: &EssIr, outcome: &ResolvedOutcome) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(summary) = &outcome.summary {
        parts.push(summary.clone());
    }
    parts.push(condition_description(&outcome.condition));
    // What the caller's request did to the system's state, in the response that reports it. A
    // caller reading `202` learns that a branch was taken; without this it does not learn that an
    // invoice now exists, and the specification does say so.
    parts.extend(retained_result_description(outcome));
    if let Some(subject) = &outcome.subject {
        let entity = ir.entity(&subject.entity);
        parts.push(match &subject.effect {
            ResolvedEffect::Preserves => format!(
                "The existing `{}` is preserved without an error or event.",
                entity.name
            ),
            ResolvedEffect::Creates => format!(
                "A `{}` now exists, in `{}`.",
                entity.name,
                subject.into.as_ref().unwrap_or(&entity.lifecycle.initial)
            ),
            ResolvedEffect::Deletes => format!("A `{}` no longer exists.", entity.name),
            ResolvedEffect::Moves { transition } => format!(
                "A `{}` has moved to `{}`, along `{}`.",
                entity.name, transition.to, transition.name
            ),
            ResolvedEffect::Updates => format!(
                "A `{}` has changed, and its state is unchanged.",
                entity.name
            ),
        });
        // Which one. A caller reading "an invoice has moved" and not which invoice has been told
        // half of what the specification says, and the half it is missing is the half it supplied.
        let field = subject.instance.field();
        parts.push(match subject.instance.event() {
            None => format!("The instance is the one `{}` names.", field.name),
            Some(event) => format!(
                "Its identity is published as `{}` on `{}`.",
                field.name,
                ir.event(event).name
            ),
        });
    }
    if !outcome.emits.is_empty() {
        let events: Vec<String> = outcome
            .emits
            .iter()
            .map(|handle| {
                let event = ir.event(handle);
                format!("`{}`", event.naming.wire_or(&event.name))
            })
            .collect();
        parts.push(format!(
            "Emits {}, published to consumers and listed under `published`.",
            list(&events)
        ));
    }
    parts.join(" ")
}

/// A schema fragment, as a document carries it.
///
/// A YAML value rather than a `serde_json::Value` because a YAML mapping keeps the order it was
/// built in and `serde_json`'s map is a `BTreeMap`: through JSON, a struct's properties would come
/// out alphabetical, and the one ordering the specification's author expressed would be gone from
/// the published contract. Both are deterministic; only one of them still shows the model.
type Fragment = ess_yaml::Value;

fn retained_result_description(outcome: &ResolvedOutcome) -> Option<String> {
    outcome.replays.as_ref().map(|replay| {
        format!(
            "Returns the exact retained result of `{}` without an error, event, or subject change.",
            replay.origin
        )
    })
}

/// The fragment [`types`] publishes for one construct, aimed at this document's own table.
///
/// No prefix, because this table already spells a message's key as `{name}.Input` or `{name}.Error`:
/// a named type can be filed under its bare qualified name without colliding with either, so a
/// pointer to one is the name. `asyncapi` keys its table differently and passes `type.` for that
/// reason.
fn embedded(node: &Node) -> Fragment {
    under_components(node, "")
}

/// A schema this file wrote by hand, as the document carries it.
///
/// An outcome's response body, the `Idempotency-Key`'s value, and the pointers a request body and a
/// response wrap. Every one of them describes something the model has no construct for — a branch
/// rendered as HTTP, a header, a reference — and nothing describing a *model* construct comes through
/// here. Anything that did would be a third copy of the mapping starting.
fn written(schema: Value) -> Fragment {
    json_fragment(schema)
}

/// Converts one hand-written JSON fragment without crossing serde's data-model bridge.
///
/// `serde_json` represents a number through a private token when its `arbitrary_precision`
/// feature is unified into a build. `ess_yaml::to_value` writes that token as the number it
/// denotes (beyond10x/ess#579); matching the two value models directly does the same without the
/// round trip through serde's data model.
fn json_fragment(value: Value) -> Fragment {
    match value {
        Value::Null => Fragment::Null,
        Value::Bool(value) => Fragment::Bool(value),
        Value::Number(value) => Fragment::Number(if let Some(value) = value.as_i64() {
            ess_yaml::Number::from(value)
        } else if let Some(value) = value.as_u64() {
            ess_yaml::Number::from(value)
        } else {
            ess_yaml::Number::from(value.as_f64().expect("a JSON number is finite"))
        }),
        Value::String(value) => Fragment::String(value),
        Value::Array(values) => Fragment::Sequence(values.into_iter().map(json_fragment).collect()),
        Value::Object(values) => Fragment::Mapping(
            values
                .into_iter()
                .map(|(key, value)| (Fragment::String(key), json_fragment(value)))
                .collect(),
        ),
    }
}

/// One fragment from [`types`], with every pointer retargeted at a `components.schemas` key.
///
/// [`types::pointer`] spells one pointer — `#/$defs/{name}` — because the `schema` projection writes
/// self-contained documents whose definitions live in `$defs`. The same reference is
/// `#/components/schemas/{name}` here and `#/components/schemas/type.{name}` in `asyncapi`, whose
/// table is keyed per kind so that an event and a type sharing a name cannot replace each other.
/// Each spelling is right for the document it appears in, which is why `tests/agreement.rs`
/// normalises a pointer and nothing else.
///
/// Shared with `asyncapi` rather than copied into it, and `pub(crate)` only for that: a second copy
/// of this rewrite would be the same class of defect as the two type mappings this crate has just
/// finished deleting. It belongs beside [`types::pointer`], as that function's own documentation
/// says, and moves there the next time `src/types.rs` is open for editing.
pub(crate) fn under_components(node: &Node, prefix: &str) -> Fragment {
    retargeted(
        ess_yaml::to_value(node)
            .unwrap_or_else(|error| panic!("a schema fragment converts to YAML: {error}")),
        SCHEMAS,
        prefix,
    )
}

/// The same fragment with every pointer retargeted at a table other than `components.schemas`.
///
/// One table per base, and the base is the whole prefix: the model view under `x-ess-entities` is
/// keyed by qualified name exactly as `components.schemas` is, so nothing but the first segment of
/// the pointer differs.
fn under(node: &Node, base: &str) -> Fragment {
    retargeted(
        ess_yaml::to_value(node)
            .unwrap_or_else(|error| panic!("a schema fragment converts to YAML: {error}")),
        base,
        "",
    )
}

/// The same fragment with every `$ref` pointing into `components.schemas`.
///
/// Recurses as deep as the fragment nests and does not count. The fragment is a [`Node`] this crate
/// built, so its depth is a small multiple of the type reference it describes — an `Optional` costs
/// an `anyOf`, a `List` an `items` — and a type reference is at most
/// [`MAX_TYPE_DEPTH`](ess_domain::types::MAX_TYPE_DEPTH) deep, refused there rather than here. A
/// named type is a `$ref` into a flat table, not an inlined subtree, so nothing compounds.
fn retargeted(fragment: Fragment, base: &str, prefix: &str) -> Fragment {
    match fragment {
        Fragment::Mapping(entries) => Fragment::Mapping(
            entries
                .into_iter()
                .map(|(keyword, value)| {
                    let value = if keyword.as_str() == Some(REFERENCE) {
                        Fragment::String(format!("{base}{prefix}{}", pointed_at(&value)))
                    } else {
                        retargeted(value, base, prefix)
                    };
                    (keyword, value)
                })
                .collect(),
        ),
        Fragment::Sequence(items) => Fragment::Sequence(
            items
                .into_iter()
                .map(|item| retargeted(item, base, prefix))
                .collect(),
        ),
        other => other,
    }
}

/// The qualified name a pointer from [`types`] resolves to.
///
/// A panic rather than a pass-through for anything else: a pointer this function does not recognise
/// is a reference into a table this document does not have, and copying it through would publish a
/// document whose own `$ref` resolves to nothing — which parses, validates as a document, and fails
/// the first time somebody validates a payload against it.
fn pointed_at(pointer: &Fragment) -> &str {
    pointer
        .as_str()
        .and_then(|text| text.strip_prefix(DEFS))
        .unwrap_or_else(|| {
            panic!("a `$ref` from the type mapping is `{DEFS}…`, not {pointer:?}");
        })
}

/// The `components.schemas` key for one outcome's response body.
fn response_key(command: &ResolvedCommand, outcome: &ResolvedOutcome) -> String {
    format!("{}.{}.Response", command.name, outcome.name)
}

/// The `components.schemas` key for one error's payload.
fn error_key(declared: &ResolvedError) -> String {
    format!("{}.Error", declared.name)
}

/// A pointer to a schema in this document.
///
/// Always in-document. A qualified name contains neither `/` nor `~`, so no JSON Pointer escaping is
/// needed and the key is the name a reader is looking for.
fn reference(key: &str) -> String {
    format!("{SCHEMAS}{key}")
}

/// Where the contract's schemas live.
const SCHEMAS: &str = "#/components/schemas/";

/// Where the model view lives: the entities of this component's domains, and what they reach.
const MODEL: &str = "#/x-ess-entities/";

/// One media type, holding one schema.
fn content(schema: Value) -> BTreeMap<String, MediaType> {
    content_as(MEDIA_TYPE, schema)
}

/// One media type an `ess-http/1` binding names, holding one schema.
fn content_as(media_type: &str, schema: Value) -> BTreeMap<String, MediaType> {
    let mut out = BTreeMap::new();
    out.insert(
        media_type.to_owned(),
        MediaType {
            schema: written(schema),
        },
    );
    out
}

/// `a`, `a and b`, `a, b and c`.
fn list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// An `OpenAPI` 3.1 document.
///
/// Typed rather than a `serde_json::Value` tree so that field order in the emitted YAML is a
/// decision rather than an alphabetical accident: `openapi` belongs at the top of the file, and a
/// reader looking for it should not have to scroll past `components`.
#[derive(Debug, serde::Serialize)]
struct Document {
    openapi: &'static str,
    #[serde(
        rename = "x-ess-one-time-response",
        skip_serializing_if = "Vec::is_empty"
    )]
    one_time_response: Vec<crate::one_time_response::Policy>,
    info: Info,
    /// The base URLs an `ess-http/1` binding declares for this component, in the order written, so
    /// the first is the default. Absent without them: the model has no URL for a component.
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    servers: Vec<ess_http::Server>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<Tag>,
    paths: BTreeMap<String, PathItem>,
    components: Components,
    /// The model view: what this component's domains are about, beside the contract they answer.
    ///
    /// Last, because a reader of an `OpenAPI` document is looking for the paths and the schemas
    /// they reference; this is the part no HTTP client needs and every reader asking *what is an
    /// invoice* wants. [`entity_model`] carries why it is an extension.
    #[serde(rename = "x-ess-entities", skip_serializing_if = "BTreeMap::is_empty")]
    entities: BTreeMap<String, Fragment>,
}

/// What this document describes.
#[derive(Debug, serde::Serialize)]
struct Info {
    title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    description: String,
    /// The specification's version, which is the only version the model has.
    version: String,
    /// Design §10, as structured data rather than only as a comment: a comment is stripped by
    /// everything that round-trips YAML, and it is a tool that asks which model a document came from.
    #[serde(rename = "x-ess-provenance")]
    provenance: Provenance,
    /// Where the specification says this component's callers are.
    ///
    /// Present only where the specification said something — an absent keyword is the model's
    /// silence, and writing `in_process` into every document would put a deployment claim in
    /// thirty-six contracts no author made. Where it says `network`, this document is not
    /// documentation of an internal surface: it is the surface.
    #[serde(rename = "x-ess-reached-by", skip_serializing_if = "Option::is_none")]
    reached_by: Option<&'static str>,
}

/// One domain, as a grouping.
#[derive(Debug, serde::Serialize)]
struct Tag {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

/// One path, and the methods it answers.
///
/// Without a binding exactly one field is ever set, because a derived path is one construct: a
/// command's path is a `POST` and a view's is a `GET`, and the two cannot collide — the segment
/// between the domain and the name is `commands` for one and `views` for the other. An `ess-http/1`
/// binding may put several operations on one path, each on its own method. Fields rather than an
/// enum so that the emitted YAML keeps `OpenAPI`'s own spelling, where the method is the key, in
/// `OpenAPI`'s own order.
#[derive(Debug, Default, serde::Serialize)]
struct PathItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    get: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    put: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    post: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete: Option<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch: Option<Operation>,
}

impl PathItem {
    /// The operation this path answers on `method`.
    fn slot(&mut self, method: http::Method) -> &mut Option<Operation> {
        match method {
            http::Method::Get => &mut self.get,
            http::Method::Put => &mut self.put,
            http::Method::Post => &mut self.post,
            http::Method::Delete => &mut self.delete,
            http::Method::Patch => &mut self.patch,
        }
    }
}

/// One command, exposed.
#[derive(Debug, serde::Serialize)]
struct Operation {
    #[serde(
        rename = "x-ess-periodic-bindings",
        skip_serializing_if = "Vec::is_empty"
    )]
    periodic: Vec<ess_compiler::ir::ResolvedBinding>,
    #[serde(rename = "operationId")]
    id: String,
    /// `binding` where an `ess-http/1` binding declares this operation's method and path; absent
    /// where they are derived, so a document generated without a binding keeps its bytes.
    #[serde(rename = "x-ess-route-source", skip_serializing_if = "Option::is_none")]
    route_source: Option<&'static str>,
    #[serde(
        rename = "x-ess-binding-accessors",
        skip_serializing_if = "Vec::is_empty"
    )]
    accessors: Vec<BindingAccessor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    /// The actors the specification permits to invoke this command.
    ///
    /// An annotation, not a requirement: `security` would claim an authentication mechanism the
    /// model does not state, and this says only what the specification says — who may, not how they
    /// prove it. See the module documentation's "What this refuses to guess".
    #[serde(rename = "x-ess-may-invoke", skip_serializing_if = "Vec::is_empty")]
    may_invoke: Vec<String>,
    /// The actors the specification permits to read this view (beyond10x/ess#286): the actors whose
    /// `may:` names it. Absent on a command, and on a view no actor names, which is open to every
    /// caller. An annotation, for the reason `x-ess-may-invoke` is one.
    #[serde(rename = "x-ess-may-read", skip_serializing_if = "Vec::is_empty")]
    may_read: Vec<String>,
    /// The attributes of the caller the command reads (ess/16, beyond10x/ess#168): what the
    /// request has to be authenticated as, which its body does not carry. An annotation, for the
    /// reason `x-ess-may-invoke` is one: the model states what the credential carries, not how a
    /// caller proves it.
    #[serde(rename = "x-ess-caller", skip_serializing_if = "Vec::is_empty")]
    caller: Vec<String>,
    /// How soon a view reflects a command that has already returned.
    ///
    /// An annotation and not a header, because it is a property of the projection rather than of
    /// this request: `eventual` says a caller may read a row that does not yet reflect its own
    /// write, and `read_your_writes` says it may not. A `Cache-Control` here would claim a caching
    /// policy the model does not state. Absent on a command, which projects nothing.
    #[serde(rename = "x-ess-consistency", skip_serializing_if = "Option::is_none")]
    consistency: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    parameters: Vec<Parameter>,
    #[serde(rename = "requestBody", skip_serializing_if = "Option::is_none")]
    request_body: Option<RequestBody>,
    /// Keyed by status as text, which sorts numerically while every status is three digits.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    responses: BTreeMap<String, Response>,
}

/// A header, in this crate's case always a header.
#[derive(Debug, serde::Serialize)]
struct Parameter {
    name: String,
    #[serde(rename = "in")]
    location: &'static str,
    description: String,
    required: bool,
    /// How a list travels in the query: stated for a `List<T>` view parameter only.
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<&'static str>,
    /// Whether each element of a list is its own `name=value` pair.
    #[serde(skip_serializing_if = "Option::is_none")]
    explode: Option<bool>,
    schema: Fragment,
}

/// The command's input.
#[derive(Debug, serde::Serialize)]
struct RequestBody {
    description: String,
    required: bool,
    content: BTreeMap<String, MediaType>,
}

/// One outcome, or a `oneOf` of the outcomes sharing a status.
#[derive(Debug, serde::Serialize)]
struct Response {
    description: String,
    /// The headers an `ess-http/1` binding declares for this status, a redirect's `Location`
    /// among them; empty, and absent, everywhere else.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    headers: BTreeMap<String, ResponseHeader>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<BTreeMap<String, MediaType>>,
    /// The declared outcomes this status answers, in the order the command declares them; empty
    /// for a status only a standard refusal answers. Written on every response of a command whose
    /// route an `ess-http/1` binding answers its own way, because a bare body names no branch, and
    /// absent everywhere else.
    #[serde(rename = "x-ess-outcomes", skip_serializing_if = "Option::is_none")]
    outcomes: Option<Vec<String>>,
}

impl Response {
    /// A response with no headers and no `x-ess-outcomes`: every response but a bound one.
    fn new(description: String, content: Option<BTreeMap<String, MediaType>>) -> Self {
        Self {
            description,
            headers: BTreeMap::new(),
            content,
            outcomes: None,
        }
    }
}

/// One response header, as `OpenAPI` writes it.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
struct ResponseHeader {
    description: String,
    /// `true` when the header is always written; absent when an absent source omits it, or when
    /// only some answers of the status write it.
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,
    schema: Fragment,
}

/// One encoding of one schema.
#[derive(Debug, serde::Serialize)]
struct MediaType {
    schema: Fragment,
}

/// Everything the document's `$ref`s resolve against.
#[derive(Debug, serde::Serialize)]
struct Components {
    schemas: BTreeMap<String, Fragment>,
}

/// Declared bounded mappings on operations reached by accessor bindings.
#[derive(Debug, serde::Serialize)]
struct BindingAccessor {
    binding: String,
    event: String,
    target: String,
    path: String,
    may_miss: bool,
}

/// Every parameter a view declares, as a query parameter under its wire name, in declaration order.
/// Empty for a view with no `params:`, whose operation keeps its bytes. A parameter a bound route
/// `carried` in its path is not one of them.
///
/// A filter parameter is required unless it is declared `Optional`, and its schema is its declared
/// type. A paging parameter (`paging:`, ess/16, beyond10x/ess#174) is an optional integer whatever
/// type it is declared at, because `paging:` declares that a read sending neither answers every row.
/// A `List<T>` parameter repeats its key once per element, `queues=1&queues=2` (beyond10x/ess#438):
/// `style: form` and `explode: true` are `OpenAPI`'s default for a query parameter, stated so that no
/// reader settles on `queues=1,2`, which is ambiguous for an element holding a comma.
fn view_parameters(view: &ResolvedView, carried: &BTreeSet<&str>) -> Vec<Parameter> {
    view.params
        .iter()
        .filter(|param| !carried.contains(param.name.as_str()))
        .map(|param| {
            let name = types::wire_name(param).to_owned();
            match &view.paging {
                Some(paging) if paging.page == param.name => Parameter {
                    name,
                    location: "query",
                    description: format!(
                        "Which page of the declared order to answer, numbered from {}.",
                        paging.first_page
                    ),
                    required: false,
                    style: None,
                    explode: None,
                    schema: written(json!({"type": "integer", "minimum": paging.first_page})),
                },
                Some(paging) if paging.size == param.name => Parameter {
                    name,
                    location: "query",
                    description: "How many rows a page holds at most.".to_owned(),
                    required: false,
                    style: None,
                    explode: None,
                    schema: written(json!({"type": "integer", "minimum": 1})),
                },
                _ => Parameter {
                    name,
                    location: "query",
                    description: param.naming.summary.clone().unwrap_or_else(|| {
                        format!(
                            "Read by the view's filter as `param.{}`; it selects which rows the \
                             view holds.",
                            param.name
                        )
                    }),
                    required: !matches!(param.type_ref, ResolvedTypeRef::Optional { .. }),
                    style: is_list(&param.type_ref).then_some("form"),
                    explode: is_list(&param.type_ref).then_some(true),
                    schema: embedded(&types::field(param)),
                },
            }
        })
        .collect()
}

/// Whether a parameter is a list, present or `Optional`.
fn is_list(type_ref: &ResolvedTypeRef) -> bool {
    match type_ref {
        ResolvedTypeRef::List { .. } => true,
        ResolvedTypeRef::Optional { of } => is_list(of),
        _ => false,
    }
}

/// What a paged view's page and size select, and whether the answer carries a total.
fn paging_sentences(view: &ResolvedView) -> Vec<String> {
    let Some(paging) = &view.paging else {
        return Vec::new();
    };
    // The query keys a request carries, not the declared names: they differ under `wire:`.
    let wire = |name: &str| {
        view.params
            .iter()
            .find(|param| param.name == name)
            .map_or(name, types::wire_name)
            .to_owned()
    };
    let (page, size) = (wire(&paging.page), wire(&paging.size));
    let start = if paging.first_page == 0 {
        format!("`{page} * {size}`")
    } else {
        format!("`({page} - {}) * {size}`", paging.first_page)
    };
    let mut sentences = vec![format!(
        "Paged: `{size}` rows of the declared order starting at {start}, pages numbered from {}; a \
         read that sends neither answers every row.",
        paging.first_page
    )];
    if paging.total {
        sentences.push("The answer carries the number of rows the filter admits.".to_owned());
    }
    sentences
}

/// What a view declaring `absent:` (ess/25, beyond10x/ess#533) answers for an identity no row
/// carries: one default row. `None` for every other view, whose description keeps its bytes.
fn absent_sentence(ir: &EssIr, view: &ResolvedView) -> Option<String> {
    let absent = view.absent.as_ref()?;
    let identity = &ir.entity(&view.source).identity.name;
    let mut values: Vec<String> = Vec::new();
    if view.fields.iter().any(|field| &field.name == identity) {
        values.push(format!("`{identity}` the requested `{}`", absent.param));
    }
    values.extend(
        absent
            .row
            .iter()
            .map(|(field, literal)| format!("`{field}` `{literal}`")),
    );
    let mut sentence = format!(
        "For a `{}` no row carries, the answer holds one default row",
        absent.param
    );
    if !values.is_empty() {
        sentence.push_str(": ");
        sentence.push_str(&values.join(", "));
    }
    sentence.push_str("; every other field is absent.");
    Some(sentence)
}

/// A paged view's response body (`paging:`, ess/16): `rows` is one page of the declared order, and
/// with `total: true` an optional integer `total` carries the number of rows the filter admits. A
/// view without `paging:` keeps its schema's bytes.
fn paged_schema(view: &ResolvedView, schema: &mut Value) {
    let Some(paging) = &view.paging else {
        return;
    };
    let properties = &mut schema["properties"];
    properties["rows"]["description"] = Value::String(
        "One page of the rows the filter admits, in the declared order; every row where the read \
         sends no page and no size."
            .to_owned(),
    );
    if paging.total {
        properties["total"] = json!({
            "type": "integer",
            "minimum": 0,
            "description": "The number of rows the filter admits, over every page.",
        });
    }
}
