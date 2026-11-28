---
format: aep.planning-md/3
id: story:a-command-guard-compares-a-value-with-the-digest-of-several-named-inputs-hex-sha256-tuple-and-the-interpreted
kind: story
status: draft
title: A command guard compares a value with the digest of several named inputs, hex(sha256(tuple(…))), and the interpreted target decides it
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess
revision: 5
---
## Need

A value is defined as the digest of named inputs: a cache key is the digest of what the cached
result was computed from, and a content address is the digest of a blob's identifying fields. The
specification wants to say that the value *is* that digest, and wants the suite to check it. A
downstream application states "a result is a pure function of its inputs" in two ways today. By
identity: one row per key, and a repeated request preserves it. By refusal: a key already stored
with other inputs is refused, and a second result that differs from the first is refused. It
cannot state that the key is computed from the inputs.

What ESS has, read at `bcfc51555`:

- From `ess/27`, a command guard may read a fact through `sha256(x)`, `base64url(x)` and `ascii(x)`
  (beyond10x/ess#584, RFC 7636 PKCE `S256`: `challenge != base64url(sha256(ascii(input.verifier)))`),
  and `{expr: …}` computes a value with the same functions
  (`crates/specify/ess-primitives/src/expr.rs:162-177`, evaluated at `:1226-1229`;
  `website/docs/reference/predicates.md:453-490`). Synthesis already sends an input whose digest
  matches and one whose digest does not.
- Those functions nest around **one** fact (`predicates.md:475`). Grid O has no String
  concatenation (`docs/design/language-grids.md:71`, `fact + - * / fact`: no), so several inputs
  cannot be combined into one digest. There is no hex encoding: `base64url` is the only text form
  of Bytes.
- "Same key, same inputs" is a row-set refusal today, and the application writes one. "Same
  inputs, one row" is `unique: [fields]`
  (`story:an-identity-may-be-a-composite-of-fields`, active, not yet implemented). Neither says
  how the key is computed.

Minimal reproduction, written fresh and not run:

```yaml
format: ess/38
# …
commands:
  - name: demo.cache.Request
    input:
      - {name: cache_key, type: String}
      - {name: source, type: String}
      - {name: revision, type: Integer}
      - {name: profile, type: Optional<String>}
    outcomes:
      - name: key-not-derived
        when: cache_key != hex(sha256(tuple(input.source, input.revision, input.profile)))
        error: demo.cache.KeyNotDerived
      - name: requested
        creates: demo.cache.Result
        instance: cache_key
        # …
```

At `ess/38` and below, `tuple` and `hex` are unknown functions and the guard is refused. The exact
refusal text is recorded when the story starts. The requester's words: "ESS cannot state that a
digest is computed from named inputs, nor check it"; they proposed no syntax.

## Outcome

At `ess/39`, two functions join the `ess/27` family:

- `tuple(f1, …, fn)`, with n ≥ 2 facts, gives Bytes in one canonical encoding;
- `hex(x)` gives Bytes as lowercase hexadecimal text.

A command guard compares a value with `hex(sha256(tuple(…)))`. The interpreted target computes it
as it computes the PKCE digest today. Generated Rust and Go keep such a command an obligation, and
Entity Runtime refuses it by name, as for every function of the family. `{expr: …}` values accept
both functions through the shared function set.

## Acceptance

Format (D77, 2026-11-24): this story's construct lands in `ess/38`, opened by story:the-specification-format-ess-38-is-open-as-the-newest-reads-and-compiles-every-specification-exactly-as-ess; where this text names `ess/37`, `ess/39`, "the new format", "the next source format" or a format still to be decided as the format of this construct, read `ess/38`, and where it names `ess/36` as the format below it, read `ess/37` (`ess/37` shipped empty in 0.93.0); a reproduction model keeps the format it declares.

- Red first: `crates/specify/ess-domain/tests/digest_of_inputs.rs` validates the reproduction at
  `ess/39` (exit 0). The same text at `ess/38` is refused as `unsupported_format_version` naming
  `ess/39`, and the test pins that exact message. Today the first case fails.
- `tuple` takes two or more facts, each a scalar (String, Integer, Decimal, Boolean, Timestamp,
  Duration, Uuid, an enum, or a newtype over one of them) or an `Optional` of one. Each of these is
  refused as `type_mismatch`, with a message naming what `tuple` takes: one fact, a `List`, a
  `Map`, a struct and `Json`. `hex` takes Bytes and gives a String.
- The encoding is written in `docs/design/digest-of-named-inputs.md` and fixed by golden vectors in
  `crates/specify/ess-primitives/tests/`. For each fact in order: one byte `0x00` if it is absent,
  or `0x01` followed by the byte length (eight bytes, big-endian) and the fact's canonical wire
  text in UTF-8. The canonical text of a Decimal, a Timestamp and a Duration is the one the
  interpreted target compares by. The vectors cover an absent `Optional`, an empty String, two
  facts whose plain concatenation is equal (`("ab", "c")` against `("a", "bc")`), a Decimal
  written `1.0` and `1`, and a non-ASCII String.
- The interpreted target answers the reproduction `requested` for a key computed from the inputs,
  and `key-not-derived` when any one named input differs.
- A value `{expr: "hex(sha256(tuple(input.source, input.revision)))"}` in `sets:` stores the digest
  on the interpreted target.
- Generated Rust and Go keep the command an obligation named by the function. Entity Runtime
  refuses it as `FunctionCallUnsupported`, and the `website/docs/reference/entity-runtime-lowering.md`
  rows (`:94`, `:144`) list `tuple(…)` and `hex(…)`.
- A model using neither function keeps its IR bytes and digest. A refusal text that lists every
  function changes: each pin it moves is named in the report.
- `website/docs/reference/predicates.md` lists both functions in the `ess/27` table's successor
  section for `ess/39`. `spec-versions.md` names them in the `ess/39` row. Grid O in
  `docs/design/language-grids.md` gains "`tuple` of several scalar facts" and "`hex` of Bytes" in
  the functions rows, citing this story.

## Fit review

Tree `bcfc51555`. Received 2026-11-24 from a downstream application, report item 15.

1. **Need.** A value is the digest of named inputs, and the suite checks it. Reproduction above.
   No syntax was proposed.
2. **Class.** *Gap.* A digest of one fact is stateable, a digest of several is not
   (`predicates.md:475`, `language-grids.md:71`).
3. **Already expressible?** In part. The functional dependency between key and inputs among
   stored rows is a row-set refusal (same key, other inputs), plus `unique:` once it lands (same
   inputs, another key). Neither states how the key is computed, so the figure "the key is the
   digest" stays out of the suite. Policy 1 (`generic-language-features.md:590-595`) rules that
   idiom out as the whole answer.
4. **Fit.** Grid O, functions rows: the `ess/27` family is extended, not doubled. `sha256` keeps
   taking one argument, and the new `tuple` is the only function over several facts, so the
   "one fact through any chain" reading stays true for every other function. The positions are
   those of the family: command guards and `{expr: …}`. Invariants, view filters and selectors
   keep refusing a function (`predicates.md:477-478`). Targets: interpreter computes; Rust and Go
   obligation; Entity Runtime refuses by name; `ess verify diff` reports a changed guard as
   `outcome-condition-changed` (`generic-language-features.md:627-629`).
   - Boundary: ESS computes a digest of adopter data in the interpreted target and in the suite's
     expected inputs. That has been the case since `ess/27` for PKCE. Generated code computes
     nothing new. Operator decision `P2B-D2`.
5. **Second adopter.** An artifact store whose blob id is the lowercase hex SHA-256 of
   (media type, size, content digest); an idempotency key that is the digest of (caller, route,
   body digest).
6. **Cost.** Two function names at `ess/39` (`g15-ess39-opens`), no keyword, no diagnostic code,
   new `Function` variants in the IR only for models that use them, one design page, golden
   vectors. A pinned refusal text lists more functions. Size M.
7. **Considered.** (a) Change nothing. (b) Identity plus `unique:`: no computation, but the digest
   stays unstated (policy 1). (c) `concat(a, b, …)` of Strings: `("ab","c")` and `("a","bc")`
   collide, and it opens String `+` in grid O, a separate decision. (d) RFC 8785 canonical JSON of
   an array: an exact JSON canonicaliser in every runner, for no gain over a length-prefixed
   encoding. (e) Chosen: `tuple` plus `hex`, beside `sha256`.

## Decisions

**Accept, redesigned** (`P2B-D2` (a)). Open questions:

- **D1 (`P2B-D2`; open, with the owner as AQ1). May ESS compute a digest of several named inputs?** (a) Yes: extend the
  `ess/27` family (this story and `g15-digest-witness`). (b) No computation: state the dependency
  with identity plus `unique:` (`story:an-identity-may-be-a-composite-of-fields`, active). (c)
  Decline: the application keeps its identity, its refusals and its own determinism tests.
  **Recommended: (a).** (b) lands anyway through its own story and complements (a).
- **D2 (open, with the owner as AQ1). The encoding.** (a) A presence tag, then a length-prefixed canonical wire text per fact.
  (b) RFC 8785 canonical JSON of an array. (c) Concatenation with a separator. **Recommended:
  (a).**
- **D3 (decided: a, 2026-11-24). The text form.** (a) Add `hex(x)` beside `base64url(x)`. (b) `base64url` only.
  **Recommended: (a).** `sha256:<hex>` is the common shape of a content address.
- **D4 (decided: a, 2026-11-24). A key newtype with `prefix:`.** A newtype's prefix is part of its value. (a) First cut:
  the guard compares a String value, and an author whose key carries a prefix declares the digest
  part as its own field. (b) String concatenation with a literal (`"sha256:" + hex(…)`), which
  opens String `+` in grid O, a G1 decision of its own. **Recommended: (a).**

- AQ1a: stays the owner's.

- AQ1b: stays the owner's.

## Scope

- cited: `crates/specify/ess-primitives/src/expr.rs` (`Function`, `ALL`, typing at `:1095-1114`,
  evaluation at `:1226-1229`)
- inferred: `crates/specify/ess-domain/src/expression.rs` (function typing and the format gate)
- inferred: `crates/generate/ess-entity-runtime/src/lib.rs` (the `FunctionCallUnsupported` site)
- inferred: `crates/specify/ess-domain/tests/digest_of_inputs.rs` (new),
  `crates/specify/ess-primitives/tests/` (vectors)
- cited: `website/docs/reference/predicates.md`, `website/docs/reference/spec-versions.md`,
  `website/docs/reference/entity-runtime-lowering.md`, `docs/design/language-grids.md`
- inferred: `docs/design/digest-of-named-inputs.md` (new)
- Size: M (about 450 changed lines).

## Depends on

- `g15-ess39-opens` (or `ess/38`, if `P2B-D5` (a))

## Source

A downstream application, 2026-11-24, report item 15.
