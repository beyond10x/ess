---
format: aep.planning-md/3
id: story:digest-and-encoding-functions-in-expressions
kind: story
status: draft
title: Predicates and value expressions apply sha256, base64url and ascii
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
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

## Intake 2026-10-16: https://github.com/beyond10x/ess/issues/584

A second downstream report (ess 0.60.0, `ess/25`, an authorization-code redemption audit) asks for
exactly this story's functions: `challenge != base64url(sha256(input.verifier))` as an ordinary
held-row predicate, so PKCE `S256` is witnessed like the plain method, which it already states as
`when_subject: {predicate: {all: ['challenge != ""', challenge != input.verifier]}}`. The report
also shows why the `external:` stand-in fails twice: beside `when_subject:` it is refused
`conflicting_declaration` ("a subject fact has one selection authority"), and without it the
held-state refusal `redirect-mismatch` answers first (G4: held state is step 3, `external:` step 5,
`docs/design/generic-language-features.md`, G4), so a request with a wrong verifier and a wrong
redirect gets the other error. With the functions, the check is a held-state predicate at step 3
and declaration order decides between it and `redirect-mismatch`, as the implementation does.

Decision for #584: covered here (the requester's option 1). Its option 2 (an `external:` refusal
narrowed by `when_subject:`) is `story:external-branch-narrowed-by-subject-guard`. These functions
are not in G1's operation table yet; they join it as a fourth text/bytes family under format B, and
this story's format line follows that decision.

Added acceptance:

- `s256_digest_guard_witnessed_beside_a_held_state_refusal`: the reproduction gains a
  `redirect-mismatch` held-state refusal declared after `mismatch`; synthesis writes a scenario
  where both hold and requires `mismatch`'s error, and the targets agree.

## Order

G1 wave of `release-plan:ess-063-downstream` § "After this release" (format B, `docs/design/generic-language-features.md`
§ Delivery, wave 2 item 5, text functions): the digest and encoding functions join grid O as a
text/bytes family. Not in the 0.63.0 batch: the function family needs the shared evaluator, the
Go and TypeScript runtimes and a synthesis witness that computes the matching challenge, none of
which the batch's units touch. Until then the reporter of #584 keeps the plain method witnessed and
S256 as an `external:` cause; the ordering half of that report (option 2) is
`story:external-branch-narrowed-by-subject-guard` under the G4 wave.
