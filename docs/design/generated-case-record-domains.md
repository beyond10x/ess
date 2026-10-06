# Generated case-record domains

Status: decided (beyond10x/ess#426). No source format, suite format or diff format moves.

A case record is the typed state of one case worked under a protocol: its claims, the evidence
recorded against them, the revisions of what they are about, and the outcomes those claims guard.
A producer that owns such a protocol — Canon is the first — wants the case record as an ESS domain,
generated from the protocol and never edited by hand, so that it validates, compiles and
synthesizes like any other specification. This page says who writes that domain, which shapes it
uses, and where its meaning is decided.

## A producer emits authored source

ESS has no ESS import adapter for `protocol/1`, and adds none. The imports ESS ships read a source
ESS does not own into ESS's own records: `ess-openapi-import/1` and `infra-ir` produce IR and
envelopes under `ess infra`, never authored domain source, and a case record is not
infrastructure. A producer that owns its format, Canon included, generates the domain itself and
emits authored `ess/N` source. It pins the ESS release that reads it with `requires:` in its
`ess-inputs.yaml`, and gates the output with validate, compile and synthesize —
`ess specify validate --strict-requires`, `ess specify compile` and `ess verify conform synthesize`
— exactly as Canon's own `ess/` gate does. A refusal there is a defect in the generator, reported
in ESS's words, at the declaration that carries it.

The cost of the other arrangement is why: an adapter in ESS would need a second `protocol/1`
parser, or a dependency on the producer's crate, and every protocol change would become an ESS
release.

## Three idioms a generator uses

Every construct a case record needs is expressible today. Three idioms keep a generator off the
refusals a hand-derived domain ran into: quote every scalar, write compound guards with structured
`all`/`any`/`not`, and make the refusal the default branch.

- **Quote every scalar.** YAML reads an unquoted `True`, `False`, `yes` or `0` as a boolean or a
  number before ESS sees it, so `variants: [True, False, Unknown]` declares two booleans and a
  name, and `sets: {tests_pass: True}` writes a boolean into an enum field. Both are refused with
  the repair, `quote it`; a generator that quotes every scalar it emits never meets them. A
  three-valued claim is better declared `[Holds, Fails, Unknown]`, which no reader takes for a
  boolean.
- **Write conjunctions with structured `all`/`any`/`not`.** The compact form has no infix `and`,
  `or` or `not` (`refused-misparsed-predicate-disjunctions.md`), and `a == "Holds" and
  b == "Holds"` is refused with "use structured any/all/not". `{all: [a == "Holds",
  b == "Holds"]}` is the same condition, and is what a generator should emit for every compound
  guard.
- **Make the refusal the default branch.** A guarded success beside an unguarded refusal
  validates at any number of claims. A guarded success beside its guarded negation needs the
  finite proof that the two cover every case, and the proof enumerates joint assignments: three
  three-valued claims are 27, four are 81, and past 64 joint assignments the proof declines and
  the command is refused `non_exhaustive_branches`, naming the count. Past that cap a default is
  required, and below it the default is still the simpler output.

## The meaning stays in Canon

The domain records what the protocol says; it does not decide it. Canon conformance, not ESS, is
authoritative for Canon's language semantics (Atlas ADR 0067), and the evaluation semantics stay in
Canon's conformance suite (Atlas ADR 0076): three-valued claim tests, invalidation of every claim
built on a revised one, and expiry. The derivation from a protocol to a case-record domain is a
Canon profile, `canon-case-record/1`, owned, versioned and tested by Canon against its own
evaluator. ESS's suite never asserts what a claim means; it asserts that the generated domain is a
valid specification and that an implementation conforms to it.

## Names

ESS's word for the generated specification is **case-record domain**. "Protocol" keeps the one
meaning it already has in ESS: `ess specify protocol` and `ess-protospec/1` model communicating
peers, and are unrelated to a protocol whose cases are recorded here.
