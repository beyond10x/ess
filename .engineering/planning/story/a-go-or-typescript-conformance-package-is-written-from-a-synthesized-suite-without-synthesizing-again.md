---
format: aep.planning-md/3
id: story:a-go-or-typescript-conformance-package-is-written-from-a-synthesized-suite-without-synthesizing-again
kind: story
status: draft
title: A Go or TypeScript conformance package is written from a synthesized suite without synthesizing again
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/
- confidence: inferred
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/mod.rs
- confidence: inferred
  path: website/docs/reference/
revision: 8
---
## Outcome

A project that holds its implementation to the suite in more than one language synthesizes once.
`ess verify conform synthesize --target go --from-suite <suite.json> --path <spec> --out <dir>`
(and `--target typescript`) writes the same package the command without `--from-suite` writes,
byte for byte, in seconds, after checking that the suite was synthesized from the specification
at `--path`. A second CI job that today synthesizes the whole suite again only renders it.

## Need

An adopter's CI synthesizes its largest downstream specification twice: once with `--target ir`
and once with `--target go`, in two jobs. Each job runs the whole synthesis.

Measured on this host, released 0.97.4, the downstream specification (command from the
specification's directory, the profile notes, section 1):

- `--target ir`: 1,763.8e9 instructions:u, 549.8 s user.
- `--target go`: 1,770.0e9 instructions:u, 469.3 s user (wall and user vary with the host's load;
  instructions do not).
- The Go package's `essconform/suite.json` is byte-identical (`cmp`) to the `--target ir` file,
  and both runs print the same 1,156 lines of refusals, notes and counts.

So the second run repeats 99.65 % of the first one's instructions. Rendering the package costs
the remaining 6.1e9 instructions, about 0.35 %.

Read in the code (`crates/edge/ess-cli/src/main.rs`, `synthesize_suite` and `write_suite`): the
target is not an input of synthesis. It decides only whether `--compact` is allowed and which
writer runs after the suite is serialized. `go::emit_serialized_with_model`
(`crates/verify/ess-conformance/src/go/mod.rs`) writes constant `.go` files, the suite's canonical
JSON as `suite.json`, the compiled model as `ir.json` and a README. TypeScript has the same shape
(`ts::emit_serialized_with_model`).

## Fit review

1. Need, apart from the proposed design. A project that runs the suite in two languages pays for
   synthesis once per language, although the scenarios are the same. The adopter proposed
   "derive the go suite from the ir one instead of synthesizing twice".
2. Class: gap in the tooling, not in the language. Nothing an authored document says changes.
3. Can it already be done? Partly: a project can run `--target go` alone and use the
   `suite.json` inside the package as its `ir` suite, since the bytes are the same. That
   works for this adopter today and needs no release (see Decisions, D1). It relies on a fact no
   command states or checks. For a Rust-only consumer of the `ir` file, the package's other files
   are noise. There is no way to go from an `ir` suite file to a package.
4. Fit with what exists. The design reuses three things that are already there: the writer
   `write_suite`, which already takes a serialized suite and a model; the digest check
   `interpreter_for` makes before `--target interpreted` runs a suite (`spec_digest` of the model
   compiled from `--path` must equal the suite's); and `--check`, which compares instead of
   writing. Every package target gets it: Go and TypeScript share one code path. `--target ir`
   with `--from-suite` has nothing to do and is refused by name.
5. A second adopter: any project with a Go service and a TypeScript client held to one
   specification, or one that keeps the `ir` suite under version control and writes the package
   in CI. Both pay one synthesis per package today.
6. Cost: one flag, one refusal message, no format version, no change to any synthesized byte. No
   new diagnostic code: the refusals are CLI errors like the interpreted target's.
7. Considered:
   - Change nothing and document "use the package's `suite.json`": costs nothing; covers this
     adopter if it can run both from one job; the package needs Go-side files a Rust-only job does
     not want; nothing checks that the two files stay one. Kept as the answer for today (D1).
   - Several `--target` values in one call (`--target ir --out a --target go --out b`): one
     synthesis, several writers. Rejected as the first step: clap pairs repeated `--target` and
     `--out` only by position, `--check` and `--compact` would need per-target meaning, and it
     serves only a CI that runs both in one job. This adopter runs two jobs.
   - A cache of the synthesis keyed by the digest of its inputs (model, authored scenarios,
     seeds, suite format, binary version): saves the second synthesis even across separate
     invocations, but adds a cache directory, an invalidation rule over every input and the
     binary version, and a new way for a stale suite to be written. Rejected: `--from-suite` gets
     the same saving with an explicit input and a digest check.
   - The adopter's proposal, a render step from the `ir` suite: adopted in this form, as a flag
     of the existing verb rather than a new verb, because the verb already owns `--target`,
     `--out`, `--check` and the output ownership rules.

Decision: accept, redesigned (the need as proposed; the shape is a flag of `synthesize`, not a
new verb or a cache).

## Design

- `--from-suite <FILE>` on `ess verify conform synthesize`, allowed with `--target go` and
  `--target typescript`; with `--target ir` it is refused ("`--from-suite` writes a package; the
  `ir` suite is the file itself"). It conflicts with `--component`, `--scenarios`,
  `--synthesis-seed`, `--suite-format` and `--compact`: those shape a synthesis, and the suite
  read already carries what they chose.
- The command compiles the model from `--path` as today (seconds), reads the file through the
  same admission any suite reader uses, and refuses when the model's `spec_digest` differs from
  the suite's (wording as `interpreter_for`'s). Then it calls `write_suite` with the file's own
  bytes as `json`, so `suite.json` in the package is the file, byte for byte. `--check` compares
  the package as it does today.
- A suite written by an older ess that this binary still admits is accepted; the runner files are
  those of this binary, as for any package it writes. A suite this binary does not admit is
  refused by admission, as `run --suite` refuses it.
- Format consequence: none. No synthesized byte changes; no `ess/N` or suite format version.
- The adopter's CI: job 1 `ess verify conform synthesize --path . --target ir --scenarios
  scenarios --out suite.json` (unchanged), job 2 takes that file as an artifact and runs
  `ess verify conform synthesize --path . --target go --from-suite suite.json --out go`.

## Acceptance

- On `examples/billing` and on a generic caller model, `--target go --from-suite <ir file>`
  writes a tree identical (`diff -r`) to `--target go` without it; the same for `--target
  typescript`. A CLI test in `ess-cli` holds both.
- A suite synthesized from a model that differs in one field from `--path` is refused with exit
  1, naming both digests, and nothing is written. A CLI test holds it.
- `--from-suite` with `--target ir`, `--scenarios`, `--component`, `--synthesis-seed`,
  `--suite-format` or `--compact` is refused before anything is read. A CLI test holds each.
- `--check --from-suite` exits 0 on a current package and 1 naming each drifted file.
- On the downstream specification the `--from-suite` run costs under 1 % of the instructions of
  the synthesizing run (measured once, `perf stat -e instructions:u`).
- `website/docs/reference/` documents the flag beside `--target`, and the Go and TypeScript
  runner pages show the two-job CI.

## Decisions

- D1. What the adopter does before this ships. A: run only `--target go` and read
  `go/essconform/suite.json` as the `ir` suite (works on 0.97.4 today, measured byte-identical);
  B: keep both jobs until the flag ships. Recommended: A, with a `cmp` in CI until the flag
  ships.
- D2. The shape. A: `--from-suite` on `synthesize` (this story); B: a new verb
  `ess verify conform package --suite <file>`; C: several `--target`s in one call. Recommended:
  A, the least new surface: one flag on the verb that owns targets and outputs.
- D3. Strictness of the model check. A: `spec_digest` must match (as the interpreted target);
  B: `spec_digest` and `contract_digest`; C: no check. Recommended: A, the rule ESS already uses
  for the same question.
