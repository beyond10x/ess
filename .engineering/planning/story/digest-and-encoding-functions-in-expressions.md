---
format: aep.planning-md/3
id: story:digest-and-encoding-functions-in-expressions
kind: story
status: draft
title: Predicates and value expressions apply sha256, base64url and ascii
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A predicate or a value expression can apply pure digest and encoding functions to text and bytes:
`sha256(<text or bytes>)` gives bytes, `base64url(<bytes>)` gives unpadded base64url text, and
`ascii(<text>)` gives the text's ASCII bytes (refusing a non-ASCII value). They compose in `when:`,
`when_subject:`, payload and other value expressions, so a rule such as
`code_challenge == base64url(sha256(ascii(input.code_verifier)))` (RFC 7636 PKCE `S256`) is stated
in the specification, and the interpreted target and synthesis evaluate it.

## Evidence

A consumer writing OAuth contract specifications on ess 0.56.0: a proof-key check can only be an
`external:` cause today, so no synthesized scenario fails a server that skips the check. Neutral
reproduction to write first: domain `demo.pkce`, entity `Code {challenge, method}`, command
`Exchange {code, verifier}`, outcome `mismatch` with `when_subject: method == "S256"` and the
stored `challenge` not equal to the `S256` transform of `input.verifier`.

## Acceptance

- The reproduction validates on the release this starts from only with an `external:` cause,
  written first; after the change it validates with the transform in `when_subject:`.
- Synthesis witnesses both sides: it picks a verifier and computes the matching challenge for the
  accepting branch, and sends a non-matching verifier for `mismatch`. The interpreted target and
  the native, Go and TypeScript runners agree on every scenario.
- Types are checked: `sha256` takes `String` (its UTF-8 bytes) or `Bytes`, `base64url` takes
  `Bytes`, `ascii` takes `String`; a misuse is refused by name with the expected type.
- Each function's output is defined by its standard (FIPS 180-4, RFC 4648 §5 without padding) and
  pinned by known-answer vectors, including the RFC 7636 Appendix B example.
- The format consequence (a new `ess/N`) is decided explicitly; below it the functions are refused
  naming the version.

## Scope (inferred)

`crates/specify/ess-primitives` (expression and predicate evaluation), `ess-domain` and
`ess-compiler` (parsing, typing), `crates/verify/ess-conformance` (interpreter, synthesis, Go and
TypeScript runtimes), `website/docs/reference/predicates.md`.
