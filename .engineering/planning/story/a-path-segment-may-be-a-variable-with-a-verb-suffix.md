---
format: aep.planning-md/3
id: story:a-path-segment-may-be-a-variable-with-a-verb-suffix
kind: story
status: draft
title: An ess-http/1 path segment may be a variable followed by a literal verb suffix, as /tasks/{id}:cancel
summary: ESS-HTTP-007 refuses the gRPC-transcoding custom-method segment {id}:cancel (a2a wave 2, H1, 2026-10-22).
tags:
- adopter-report
- ess-0.70.0
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:a-command-binds-to-a-protocol-or-deployment-http-route
- depends_on: story:a-literal-path-segment-may-take-precedence-over-a-variable
revision: 4
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (a2a wave 2, story http-binding,
ESS 0.67.0, relayed 2026-10-22): **H1**, a2a wave-2 HTTP binding gaps list, line 8. The upstream
protocol's HTTP routes put a verb after a variable in one segment, gRPC-transcoding style:
`/tasks/{id}:cancel`, `/tasks/{id}:subscribe`. `ess-http/1` refuses the template:

> ESS-HTTP-007: "segment 2 of `/tasks/{id}:cancel`, `{id}:cancel`, is not one whole variable `{name}`"

## Where it lives

- Template grammar: `crates/specify/ess-http/src/lib.rs:1502` (`template`), the refusal text at
  `lib.rs:1547-1556`: a segment holding `{` or `}` must be exactly `{name}`.
- Decided in `docs/design/http-presentation-binding.md:135-136`: "a variable is a **whole** segment
  … `/items/{id}.json` is refused: a partial-segment template needs an escaping rule this format does
  not state." Admitting `{id}:cancel` reverses that decision for one shape (a variable followed by a
  literal `:verb` suffix, as Google API HTTP rules spell a custom method).
- Collision checks `ESS-HTTP-010`/`-011` (`lib.rs:5120-5142`) compare segments; a suffixed variable
  is a new segment kind they must order.

## Format

Additive to `ess-http/1` if only the suffix shape is admitted (documents valid today keep their
bytes; precedent: the stage-2 keys stayed `/1`). `ess-http-ir/1` gains a segment kind, so an
old-reader compatibility test and an explicit format decision are needed (AGENTS.md, Determinism
and formats). `ess/N` unchanged. Fit review decides; not settled here.

## Fit review

Assessed 2026-10-22 against the integration tree at 0.68.0, with
`.agents/skills/assessing-external-requests/SKILL.md`. This review reopens the whole-segment rule of
`docs/design/http-presentation-binding.md:135-136` and the "partial-segment path templates" line of
"Still refused" (`:622`).

**1. The need, apart from the proposed syntax.** Some published HTTP surfaces name an operation on
one record by appending a fixed verb to the record's identifier in the last path segment: "cancel
the task `t1`" is `POST /tasks/t1:cancel`. The operation's input carries the identifier. The verb is
fixed text that tells the operation apart from the plain read of the same record. `ess-http/1` cannot
bind such a route. Minimal reproduction: one command `CancelItem(item_id)` and the route
`{command: CancelItem, method: post, path: '/items/{item_id}:cancel', parameters: {item_id: input.item_id}}`
→ `ESS-HTTP-007`, "is not one whole variable `{name}`" (lib.rs:1547-1556).
*Requester's proposal (theirs):* the gRPC-transcoding spelling `{id}:cancel` and `{id}:subscribe`
(a2a wave-2 HTTP binding gaps list, line 8).

**2. Class: gap.** ESS refuses on purpose, "a partial-segment template needs an escaping rule this
format does not state" (`http-presentation-binding.md:135-136`), so it is not a defect. Today the
adopter serves both operations on their derived routes instead (`ess/http-unmapped.tsv` in the
adopter's repository, `projected` rows for `tasks:cancel` and `tasks:subscribe`).

**3. Can it already be expressed? No.** A literal segment may hold `:` (RFC 3986 `pchar`,
lib.rs:1558-1564), so `/message:send` binds today, but a literal cannot carry a value. Moving the
identifier to the body or the query changes the path the protocol fixes. That moves the figure out
of the binding, so policy 1 rules it out as an answer
(`docs/design/generic-language-features.md` § Policy, policy 1).

**4. Fit with what is there.**
- *Outside grids R and O.* The binding is not part of `ess/N` (`http-presentation-binding.md` §
  Decision). The grid this decision opens is the template grammar.
- *What the escaping rule needs, and the narrowest shape that states it.* The page's objection is
  that a partial segment needs a rule for where the value ends. That rule exists for one shape: a
  variable followed by a fixed suffix that starts with `:`, in the **last** segment only. This is the
  verb of the gRPC HTTP-transcoding rule grammar the upstream uses, where the verb is allowed only at the
  end of the template (`Template = "/" Segments [ Verb ]`). The value is the raw last segment with
  the suffix stripped from the right, then percent-decoded, and it must not be empty.
  - Stripping from the right makes a value that itself holds `:` unambiguous: in `a:b:cancel` the
    value is `a:b`.
  - A client percent-encodes the value and appends the suffix.
  - `{id}.json`, a prefix before a variable, two variables in one segment, and a suffix on any
    segment but the last stay refused. Each needs its own boundary rule, and no protocol in hand
    requires one.
- *Collisions.* A suffixed variable `{a}:v` is a third segment kind for `clash`
  (lib.rs:5029-5047):
  - `{a}:v` beside `{b}:v` is `ESS-HTTP-010`, the same template;
  - `{a}:v` beside `{b}:w` is disjoint when neither suffix ends with the other, and otherwise one is
    more specific;
  - `{a}:v` beside a literal is disjoint unless the literal ends with `:v` and has text before it,
    in which case the literal is more specific;
  - `{a}:v` beside a plain `{b}` overlaps, the suffixed one being strictly more specific.
  The adopter's surface has `get /tasks/{id}` beside `get /tasks/{id}:subscribe`, the last case.
  `ESS-HTTP-011` refuses that case today, and the nested rule of
  `story:a-literal-path-segment-may-take-precedence-over-a-variable` admits it. This story
  therefore lands with or after that one.
- *Every target handles it or refuses it by name.*
  - `template`/`segment` (lib.rs:1502-1566) and `Segment` (lib.rs:1472-1478) gain the kind.
  - `HttpIr::from_json` re-parses every path (`Route::segments`, lib.rs:1813-1820).
  - The Rust and Go servers' `matched` strips the suffix (`rust/http/bound.rs:178-196`,
    `go/http/bound.rs:136`).
  - OpenAPI emits `/items/{item_id}:cancel` with `item_id` as an `in: path` parameter. OpenAPI path
    templating marks a section of a path, not only a whole segment.
  - Refinement (`ESS-HTTP-012`) compares paths as text and needs nothing new.
  - `ess ui` and the other targets still refuse `--http` by name.
- *Siblings.* No other ESS binding has path templates. `ess-transport` addresses and `ess-cli`
  commands hold none (`docs/design/event-transport-binding.md`,
  `docs/design/cli-presentation-binding.md`), so there is no sibling to extend.

**5. A second, unrelated adopter.** Any API following the AIP-136 custom-method convention
(`POST /v1/books/{book}:undelete`, `POST /v1/jobs/{job}:run`). A second protocol, a job
runner, has `POST /runs/{run_id}:retry` beside `GET /runs/{run_id}`.

**6. Cost.**
- Authored format: `ess-http/1` keeps its version. Every document valid today keeps its meaning and
  its IR bytes. An older compiler refuses the new shape as `ESS-HTTP-007`, which is fail-closed.
- IR: `ess-http-ir/1` keeps its version, because the path is stored as text (`Route.path`,
  lib.rs:1777-1779). An older reader re-checks the grammar in `from_json` and refuses the path by
  name. The stage-2 precedent applies: an older reader refuses a newer construct by name
  (`http-presentation-binding.md:180-182`).
- Rust API: `ess_http::Segment` gains a variant. That is a breaking change for any exhaustive match
  outside the crate (in this workspace: `ess-gen`, `ess-synth`, `ess-cli`). It ships in the minor
  release like other 0.x API changes.
- Codes: none new. `ESS-HTTP-007`'s text and hint change (the hint names the suffix shape), so run
  `task test-xtask`.
- Docs: `http-presentation-binding.md:135-136` and `:622` are amended by a dated decision.

**7. Alternatives.**
- (a) *Change nothing.* The operations stay on derived routes. Rejected: the protocol's paths cannot
  be bound.
- (b) *General partial segments* (`{id}.json`, `v{n}`, any prefix or suffix). Rejected for now: each
  needs its own boundary rule, and a prefix rule interacts with literal precedence. The suffix shape
  is a subset, so this can be added later without changing it.
- (c) *Multi-segment variables* (`{name=projects/*/books/*}`). Out: the upstream does not use them
  here (`{id=*}` is one segment), and they change what a variable is.
- (d) *Chosen:* a `:`-led literal suffix after a variable in the last segment, matching the
  requester's syntax in that position.

## Decisions

**Accept, redesigned** (verdict: redesigned).

- The last segment of an `ess-http/1` path template may be `{name}:<suffix>`, where `<suffix>` is
  non-empty RFC 3986 `pchar` text without `%`, `{` or `}`.
- A request matches when its last raw segment ends with `:<suffix>` and holds at least one character
  before it. That text, percent-decoded, is the variable's value.
- Every other partial-segment shape stays `ESS-HTTP-007`, its hint naming the admitted shape.
- What changed from the request:
  - the verb is restricted to the last segment, as in the upstream grammar, not any segment;
  - the right-strip and non-empty rule is stated;
  - the overlap with a plain variable at the same position is ordered by the nested-overlap rule of
    `story:a-literal-path-segment-may-take-precedence-over-a-variable`, on which this story depends.
- Format: `ess-http/1` and `ess-http-ir/1` unchanged in version. Older readers refuse the new shape
  by name. `ess/N` unchanged.

Acceptance, each test written to fail first, as a module of an existing test binary. The stage-3 HTTP
conformance target is not built, so acceptance runs through the compiler and the generated servers:

- `crates/specify/ess-http/src/lib.rs` unit tests beside the `segment` cases (lib.rs:5215-5230):
  - `{id}:cancel` in the last segment parses;
  - `{id}:cancel` in a middle segment, `{id}.json`, `x{id}`, `{a}{b}` and `{id}:` are refused as
    `ESS-HTTP-007`.
- `crates/specify/ess-http/tests/compile.rs`:
  - `a_suffixed_variable_compiles_and_maps_its_source`;
  - `two_suffixed_variables_with_one_suffix_are_ess_http_010`;
  - `different_suffixes_do_not_overlap`;
  - `a_suffixed_and_a_plain_variable_on_one_method_compile_as_nested`.
- `crates/generate/ess-synth/tests/http_binding/shared.rs`, the Rust server:
  - `POST /items/a%2Fb:cancel` reaches `CancelItem` with `item_id == "a/b"`;
  - `POST /items/x:y:cancel` reaches it with `x:y`;
  - `POST /items/:cancel` is `404`;
  - `GET /items/t1:subscribe` answers the suffixed route, not `GET /items/{item_id}`.
- `crates/generate/ess-synth/tests/http_binding/go.rs`:
  `the_go_and_rust_servers_answer_one_binding_alike` extended with these requests.
- `crates/generate/ess-gen/tests/openapi_http.rs`: the path `/items/{item_id}:cancel` with one
  `in: path` parameter.
- `crates/edge/ess-cli/tests/http.rs`: `ess specify http validate` accepts the binding.

The requester is told: accepted, redesigned. `/tasks/{id}:cancel` and `/tasks/{id}:subscribe` bind as
written. The subscribe route beside `get /tasks/{id}` needs the nested-overlap rule from the H5
story, and both land in one release.
