---
format: aep.planning-md/3
id: story:an-unreadable-request-may-be-answered-with-a-2xx-status-and-no-body-where-the-service-does-so
kind: story
status: draft
title: An unreadable request may be answered with a 2xx status and no body where the service does so
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 4
---
## Need

An existing service answers a request whose body it cannot read with `200` and an empty body.
`ess-http/1` cannot state it, in two places:

- The standard refusal `malformed` takes a `4xx` only: `ESS-HTTP-029`, `lib.rs:3462-3475` ("`200`
  is not a `4xx` status, and `malformed` answers a `4xx` …").
- An outcome with `error` may not answer `2xx`: `ESS-HTTP-023`, `class_reason` at
  `crates/specify/ess-http/src/lib.rs:1541-1553` ("a `2xx` status, which says the request
  succeeded, and the outcome reports an error").

Both are deliberate. § "Status per outcome" and § "Decided for stage 2", answer 17, of
`docs/design/http-presentation-binding.md` take the narrower option, "so a later release can relax
it as an addition". Third report, finding 4 (first part).

Minimal reproduction (written fresh; not run):

```yaml
refusals:
  malformed: {status: 200, body: none}
```

```console
$ ess specify http validate --path spec --binding lenient.http.yaml
error[ESS-HTTP-029]: refusals.malformed.status: `200` is not a `4xx` status, and `malformed` answers a `4xx`: `malformed`, `not_granted` and `too_large` refuse the request (`4xx`), `unfinished` is the server's (`5xx`)
```

## Outcome

Under **AC-D5**, option C (the default), `refusals.malformed` (top-level or per route) may answer a
`2xx` other than `204`, with `body: none` only. It is refused where an outcome of a route it applies
to answers the same status with `body: none`, because a client could not tell the refusal from that
success. OpenAPI writes the response under that status with a description naming the `malformed`
refusal. The generated servers answer an unreadable request that way. Declared error outcomes keep
`3xx`/`4xx`/`5xx` (`ESS-HTTP-023` unchanged). If the operator takes AC-D5 option A instead, this
story widens to error outcomes too, under the same distinguishability rule (`ESS-HTTP-025`).

## Acceptance

- The reproduction validates. `ess specify http compile` writes the status. A binding without it
  compiles to the same bytes. The case is added to `crates/specify/ess-http/tests/compile.rs`.
- Refused, `ESS-HTTP-029` narrowed and `ESS-HTTP-025` widened, each naming the key: a `2xx`
  `malformed` with any body but `none`; `204`; a `2xx` on `not_granted`, `too_large` or
  `unfinished`; a `2xx` `malformed` on a status where an outcome of an applying route answers
  `body: none`, both named.
- `ess generate --kind openapi --http` writes the `2xx` response with `x-ess-refusal: malformed` and
  a description saying the service reports an unreadable request as this status. A binding without
  it keeps its bytes.
- The generated Rust and Go servers answer a body that does not parse with that status and no body
  (`http_binding_rust`, both alike). A well-formed request is answered as before.
- `docs/design/http-presentation-binding.md` answer 17 states the exception. Its cost line says a
  client and a cache read that answer as a success.

## Fit review

1. **Need.** A service's success status for an unreadable request cannot be stated. Report: third
   report, finding 4 ("answers `200` with an empty body on an unreadable request").
2. **Class: gap under a deliberate rule** (answer 17, § "Status per outcome").
3. **Already expressible?** No (`lib.rs:3462-3475`).
4. **Fit.** It relaxes one class rule of one standard refusal and keeps the distinguishability rule
   that already guards two answers on one status. Siblings: `not_granted`, `too_large` and
   `unfinished` keep their classes, since no report names them.
5. **Second adopter.** A webhook receiver that acknowledges every delivery with `200`, readable or
   not, so the sender never retries.
6. **Cost.** No format version: an admitted value. Security and correctness: the generated server
   answers a hostile or broken body with a success status. It runs no command and stores nothing,
   since the refusal happens before dispatch, but intermediaries and clients count it as a success.
7. **Considered.** See AC-D5.

## Decisions

**AC-D5**, default C. Options and costs are in `DECISIONS.md`.

Asked again by a ninth report, 2026-11-24; open with the owner.

- S3: stays the owner's.

- S8: stays the owner's; until he picks, the state is option B: declined.

## Scope

- cited: `crates/specify/ess-http/src/lib.rs` (`class_reason` 1541-1553, refusal class 3462-3475)
- cited: `crates/generate/ess-gen/src/openapi.rs`
- inferred: `crates/generate/ess-synth/src/rust/http/bound.rs`, `go/http/bound.rs`
- cited: `docs/design/http-presentation-binding.md`
- Size: S, about 250 lines with tests (M, about 450, under option A).

## Depends on

Nothing.

## Source

A downstream application, third report, finding 4 (ESS 0.98.0).
