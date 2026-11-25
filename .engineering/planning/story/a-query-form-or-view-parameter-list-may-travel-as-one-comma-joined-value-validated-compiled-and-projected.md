---
format: aep.planning-md/3
id: story:a-query-form-or-view-parameter-list-may-travel-as-one-comma-joined-value-validated-compiled-and-projected
kind: story
status: draft
title: A query, form or view-parameter list may travel as one comma-joined value, validated, compiled and projected
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 4
---
## Need

A stream route takes a filter `?kinds=opened,closed`: one query value holding a comma-joined list.
Absent or empty means no filter. `ess-http/1` writes a `List` in `form` and `query` by repeating its
key once per element, and view parameters do the same:

- `docs/design/http-presentation-binding.md:328-330`: "A `List` repeats its key once per element,
  the rule view parameters already follow (`openapi.rs:1776-1778`)".

So the adopter declares the filter as one `String`. The model then reads text where the domain has a
list, the projected contract omits the list form, and a protocol model of the stream has to take the
filter's verdicts as given inputs, because `ess-protospec/1` reads no text
(`docs/design/protocol-specification.md:19`, "binary parsing").

Minimal reproduction (written fresh; not run): view `tickets.desk.Changes(kinds: Optional<List<String>>)`.

```yaml
routes:
  - view: tickets.desk.Changes
    method: get
    path: /changes
    lists: {kinds: comma}
```

Refused today: `lists` is an unknown field of `RouteSpec` (`crates/specify/ess-http/src/lib.rs:1561-1610`).
The key name is this story's proposal, and **AE-D13** decides it.

## Outcome

Under **AE-D13**, option A (the default), a route may declare `lists: {<field or param>: comma}`
for a `List` of scalars carried in `query` or `form`, or for a view parameter:

- The value is the elements' texts joined by `,`. An element holding `,` cannot be written and is
  refused at encoding time by the writer. A server that receives an element split by a comma reads
  two elements, as RFC 3986 permits for a comma in a query value.
- Under `comma`, the key appears once. A second occurrence is `malformed`, as for a scalar.
- An empty value is the empty list. An absent key is an absent `Optional<List>`, and for a
  non-`Optional` list it is `malformed`.
- Leniencies a service may have (whitespace around items trimmed, empty items dropped) are not
  stated, **AE-D13**. A strict writer stays readable by a lenient server, and the contract names the
  form a caller must send.
- OpenAPI writes the parameter with `style: form, explode: false` (OpenAPI 3.1 § 4.8.12.4, the
  comma form).
- The protocol kind is not touched. A protocol model keeps taking the filter's verdict as an input,
  and the HTTP binding owns the text form (PR-D5 A: query encoding stays in `ess-http/1`).

## Acceptance

1. Red today: the reproduction is refused naming `lists` (a case in
   `crates/specify/ess-http/tests/request_encoding.rs`).
2. After: it validates and compiles. `ess-http-ir/1` writes `lists` only where declared. Every
   fixture compiles to the same bytes (`compile.rs`).
3. Refused, `ESS-HTTP-021` widened, at `routes[i].lists.<name>`:
   - a name that is no input field or view parameter of the route;
   - one the path carries;
   - one that is not a `List` of scalars;
   - a `List` carried in a `json` body;
   - a word other than `comma`.
4. `ess generate --kind openapi --http` writes `explode: false` for it. A binding without `lists`
   keeps its bytes (`crates/generate/ess-gen/tests/openapi_http_presentation.rs`).
5. § "Request encoding" of the design page, the `ess-http/1` row of
   `website/docs/reference/formats.md`, and `website/docs/guides/http-routes.md` state it.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (`RouteSpec` 1561-1610, `ESS-HTTP-021` row 3206)
- cited: `crates/generate/ess-gen/src/openapi.rs` (list parameters, 1776-1778 per the design page)
- cited: `docs/design/http-presentation-binding.md` (320-336)

## Owned files

`lib.rs` (`RouteSpec.lists`, compiled form, `ESS-HTTP-021` cases), `openapi.rs` (parameter style),
tests `request_encoding.rs`, `compile.rs`, `openapi_http_presentation.rs`, the three docs.

## Not yours

Servers and apps (`ae12a-servers-and-apps-split-and-join-a-comma-list`).

## Tests first

Acceptance 1, 3, 4. All pure.

## Gate

Each test stays under 60 s, the step under 120 s, and each job under 10 minutes.

```console
task fmt-check
cargo clippy -p ess-http --all-targets --locked -- -D warnings
cargo test -p ess-http --locked --test request_encoding
cargo test -p ess-http --locked --test compile
cargo clippy -p ess-gen --all-targets --locked -- -D warnings
cargo test -p ess-gen --locked --test openapi_http_presentation
cargo test -p ess-gen --locked --test openapi
task test-xtask
task ci-lint
```

## Depends on

Nothing. It shares `lib.rs` with the other `ess-http` stories of this list. Land after `ae3a`, or
rebase.

## Changelog

- `ess-http/1`: `lists: {<name>: comma}` writes a query, form or view-parameter list as one
  comma-joined value; OpenAPI states `explode: false`.

## Fit review

1. **Need.** A list travels as one comma-joined query value. The report carried it as one
   `String` and proposed no syntax.
2. **Class: gap.** One list form only (`http-presentation-binding.md:328-330`).
3. **Expressible today?** Only as a `String`, which hides the list from the model and the contract
   (policy 1).
4. **Fit.** The binding already owns the list form (repeat), so a second form is a binding fact
   beside it. View parameters and command inputs ask the same question, and both get it. `json`
   bodies have real arrays, so it is refused there. The protocol kind stays byte-free: it reads
   verdicts, the binding reads text. That is consistent with the owner's note (wire parts in the
   protocol kind) through PR-D5 A, which keeps query encoding in `ess-http/1`.
5. **Second adopter.** A search API with `?fields=id,name,price`.
6. **Cost.** One optional key, so no format version. `ESS-HTTP-021` widens.
7. **Considered.** (a) Change nothing. (b) Admit the leniencies as declared flags (AE-D13 B): one
   server's parsing, which is local policy (the precedent: `aliases:` declined in
   `story:a-timestamp-newtype-declares-its-wire-encoding`). (c) A list type with a text form in
   `ess/N`: a wire fact in the model. Refused.

## Decisions

Accept, redesigned. **AE-D13** (SECURITY: what generated servers parse), default A.

Asked again by a ninth report, 2026-11-24; open with the owner.

- S12: A, taken by the operator session on 2026-11-25 on the owner's word of 04:03 CET; reversible until a release carries it

- A release that carries this key names this pick in its pull-request line; the operator session merges that release only after the owner has read the list.

## Size

M, about 350 lines with tests.
