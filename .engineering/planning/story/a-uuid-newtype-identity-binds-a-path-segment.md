---
format: aep.planning-md/3
id: story:a-uuid-newtype-identity-binds-a-path-segment
kind: story
status: draft
title: A path segment carries a Uuid and every scalar with one text spelling
summary: ess-http/1 refuses a Uuid-newtype identity as a path segment (ESS-HTTP-009) (0.77.0)
tags:
- ess-batch-5
- ess-next
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#755
relations:
- decomposes: epic:zero-open-issues-2026-11
revision: 6
---
## Need

A resource addressed by a UUID in its path cannot be bound when its identity is typed as what it
is. `ess-http/1` lets a path segment carry `String`, `Integer`, an enum or an unconstrained newtype
of one of them, and nothing else (`crates/specify/ess-http/src/lib.rs:6734-6755`, `segment_type`),
although `Uuid`, `Boolean`, `Decimal`, `Timestamp` and `Duration` each have one text spelling that
the same servers already read from a query string or a form (`Flat::Text` and `Flat::Boolean`,
`crates/generate/ess-synth/src/rust/http/bound.rs:132-142`). Retyping the identity as `String`
loses the format in every projection, not only in the binding.

Minimal reproduction (identity `uuid: demo.apps.AppId`, a newtype of `Uuid`; command
`demo.apps.ResetKey(uuid)`; view `demo.apps.AppById` filtered `uuid == param.uuid`; ess 0.92.0):

```yaml
routes:
  - {command: demo.apps.ResetKey, method: post, path: "/apps/{uuid}/reset_key", parameters: {uuid: input.uuid}}
  - {view: demo.apps.AppById, method: get, path: "/apps/{uuid}", parameters: {uuid: param.uuid}, body: single, empty: {status: 404}}
```

```console
$ ess specify http validate --path spec --binding uuid-path.http.yaml
error[ESS-HTTP-009]: routes[0].parameters.uuid: `input.uuid` has type `demo.apps.AppId`, which a path segment cannot carry: a segment is always present and holds one `String`, `Integer`, enum, or unconstrained newtype of one of them
error[ESS-HTTP-009]: routes[1].parameters.uuid: `param.uuid` has type `demo.apps.AppId`, which a path segment cannot carry: a segment is always present and holds one `String`, `Integer`, enum, or unconstrained newtype of one of them
```

The newtype's OpenAPI schema already says `type: string, format: uuid` (`openapi/apps.yaml` of the
same run), so only the binding refuses it.

## Acceptance

- `ess specify http validate` accepts the two routes above (exit 0), and a path segment mapped to a
  `Uuid`, `Boolean`, `Decimal`, `Timestamp` or `Duration` source, or an unconstrained newtype of one;
  `Bytes`, `Binary64` and `Json` stay refused with `ESS-HTTP-009`, whose text names the admitted
  set; `Optional`, `List`, `Map`, struct, union and constrained newtypes stay refused as today.
- `ess generate --kind openapi --http` writes the path parameter with the field's schema
  (`$ref` to `demo.apps.AppId`, `format: uuid`).
- The generated Rust and Go servers run `POST /apps/0b6f3c1e-8f0a-4c55-9d0e-6f1a2b3c4d5e/reset_key`
  and answer `GET /apps/<that uuid>` with the row; `/apps/not-a-uuid` is `malformed` (`400`, or the
  binding's `refusals.malformed`), naming `path.uuid`; both servers answer alike
  (`the_go_and_rust_servers_answer_one_binding_alike` gains the case).
- `docs/design/http-presentation-binding.md` § "Path templates" and the `ESS-HTTP-009` row state
  the admitted set; `website/docs/guides/http-routes.md` names it.

## Scope

- cited: crates/specify/ess-http/src/lib.rs
- inferred: crates/specify/ess-http/tests/compile.rs
- cited: crates/generate/ess-synth/src/rust/http/bound.rs
- cited: crates/generate/ess-synth/tests/http_binding/go.rs
- inferred: crates/generate/ess-gen/tests/openapi_http.rs
- cited: docs/design/http-presentation-binding.md
- cited: website/docs/guides/http-routes.md
confidence: medium

## Fit review

1. **Need.** An identity that is a UUID in the model is the path segment that addresses its
   resource; the binding refuses the declared type. Reproduction above. Reported first on 0.77.0
   through a downstream session (this story's earlier draft), again in beyond10x/ess#755 on 0.87.0
   and reproduced on 0.92.0. The requester's proposal (theirs): admit `Uuid` and an unconstrained
   newtype of it, written in lower-case canonical form, `format: uuid` in OpenAPI, and "other
   primitives with a canonical text form (`Timestamp`, `Decimal`) could follow the same rule".
2. **Class: gap.** The restriction is documented (`docs/design/http-presentation-binding.md:163-166`,
   § "Path templates"), so not a defect; the domain fact cannot be bound.
3. **Expressible today? No.** Retyping the identity as `String` removes `format: uuid` from every
   schema and the type check from every target: it moves a checked fact out of the specification
   (`docs/design/generic-language-features.md` § Policy, policy 1).
4. **Fit.** The language grids do not apply (no expression position). The binding's column is
   "types one path segment carries", opened whole to every scalar a single `query` key already
   carries, rather than to `Uuid` alone. The admitted set equals the flat scalars the servers'
   `path_value` and `flat_object` read (`rust/http/bound.rs:264-345`), less `Bytes` (its base64 text
   holds `/`, `+` and `=`) and `Binary64` (no one text spelling of a JSON number). Constrained
   newtypes keep their refusal: why they were excluded is not recorded (`git log -S` finds only the
   first commit, `d03c9a56c6`), and no request asks for them. Targets: OpenAPI already writes the
   field's schema; the servers read the segment with the field's flat kind (`bound.rs:811-834`);
   UI refuses `--http`. Requester's "lower-case canonical form": the servers parse what the type's
   decoder accepts, and the schema's pattern admits either case (`openapi/apps.yaml`), so no
   case rule is added.
5. **Second adopter.** An event log addressed by time: `GET /days/{day}/entries` with `day` a
   `Timestamp`, or `GET /flags/{enabled}` with a `Boolean`.
6. **Cost.** No format: the change only admits documents that were refused. `ESS-HTTP-009` keeps
   its code; its text changes. No new key.
7. **Alternatives.** (a) Change nothing: identities typed as UUIDs stay unbound. (b) `Uuid` only, as
   requested: opens one cell of the column and leaves `Timestamp` and `Decimal` for the next
   request (`docs/design/generic-language-features.md` § "Closing rule"). Chosen: every scalar with
   one text spelling.

## Decisions

**Accept, redesigned.** Kept: `Uuid` and unconstrained newtypes of it in a path segment, OpenAPI
`format: uuid`, refusal of an unparsable segment as `malformed`. Changed: the column opens to
`Boolean`, `Decimal`, `Timestamp` and `Duration` as well, as the requester suggested; no lower-case
rule is added. Constrained newtypes stay refused.

## Source

github:beyond10x/ess#755
