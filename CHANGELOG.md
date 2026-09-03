# Changelog

## [0.6.0] — 2026-09-03

### Changed

- **`--kind site` writes a website, not inputs for one.** It used to emit markdown with frontmatter
  and a sidebar list, leaving an adopter to run a static site generator over it — measured on the
  one that did: `npm ci` plus a webpack build peaking at 3931 MiB, killed twice by a CI runner, to
  render seven pages of prose that were already written. It now writes the pages as HTML with
  navigation, a stylesheet and a diagram renderer beside them, so publishing a specification is one
  command and needs no Node. **An adopter consuming the old markdown output must change.**
- The documentation projection is built as an `ess-docs/1` document and rendered, rather than
  written as markdown directly. The markdown a repository reads is unchanged, byte for byte, and
  the documentation of every example is now pinned and compared to keep it so. The point of the
  layer is that the site renderer reads the same document instead of parsing the markdown back —
  which is what it had to do before, scanning for a heading to recover a title it had just written.
- The site's stylesheet and its vendored Mermaid bundle stay out of the committed projection
  sample. Neither derives from a specification, and the bundle alone is 3.5 MB.

### Added

- `--kind docs-ir` writes the `ess-docs/1` document as JSON, so a presentation layer of your own
  needs nothing from this crate.
- `--kind site` opens on the `README.md` beside the specification, and takes
  `--include <page-id>=<path>` for any number of pages beside the generated ones — a plan board
  another tool rendered, a runbook. Both are markdown somebody wrote, read into the document and
  styled like every other page. Raw HTML in them is dropped rather than passed through: a generated
  site that embedded it would inherit that markdown's scripts and its styling.

## [0.5.1] — 2026-09-03

### Fixed

- **The browser lab runs again.** `CreateInvoice` took an `account_id` from 0.5.0 and the lab's
  script did not send one, so the module refused the first command of the run —
  `{"kind":"undecodable","at":"input.account_id","expected":"a value","found":"nothing"}` — and
  `task site-build` failed after the tag was cut. The script now names the account its invoices
  belong to, which is the caller's to name because `billing.invoice.Account` declares it `owns`
  them `via account_id`.
- The lab read the identity for the next command out of the *first* entity in the catalogue, which
  was the invoice only for as long as the invoice was the only entity. `billing.invoice.Account` is
  declared above it, so `InvoiceCreated`'s `account_id` was taken for the new invoice's id and the
  next command was issued against an account. It now reads the identity of the entity the taken
  outcome `creates:`.
- `website/src/pages/lab/_source.ts`, the lab's committed copy of `examples/billing/domains/invoice.yaml`,
  was not refreshed for 0.5.0, so every line the left panel highlighted was off by the relation's
  own lines. Refreshed — and a stale copy is now a red gate rather than a wrong panel:
  `task site-lab` compares the two.
- `examples/billing-web/smoke.mjs` asserted the pre-relation input list and sent the pre-relation
  input; it is run by no task, which is why nothing caught it. Same fix, and it passes.
- The documentation pages that quote the command were carrying its old shape:
  `specification-to-contracts.md` said three values are chosen where there are now four and quoted
  a JSON Schema "in full" that was missing `account_id`, and `write-a-specification.md` showed the
  payload block without its first line. Both are the generated files again.

## [0.5.0] — 2026-09-03

- **An entity declares what it owns and what it references.** `relations:` on an entity names a
  relation, its kind (`owns` or `references`), the entity at the other end, how many of it there
  are (`one` or `many`), and the field that carries it. It is declared on the source and nowhere
  else: the reverse direction is a lookup, and a second declaration is a second thing to keep in
  step. Previously an ownership was a typed id field on the child plus an invariant somebody
  remembered to write, which is prose to every projection and refusable by nothing.
- `ess validate` refuses five things a relation can get wrong: a target that is not a declared
  entity, a `via` field the carrying entity does not have, a `via` field typed as anything but the
  identity the design requires — `Optional<…>` or `List<…>` where the cardinality says so — a second
  entity claiming to own one that is already owned, and two relations claiming one field. Each is an
  existing `ValidationCode` with a hint naming what to write instead, and each has a test that
  breaks it on purpose.
- **One extension key, `x-ess-relation`, carries a relation into every projection**, on the property
  that carries the field. The JSON Schema projection gains a document per entity,
  `schema/entities/<name>.schema.json`, because an entity was previously rendered by the
  documentation and by nothing a tool reads; the `OpenAPI` projection publishes the same shapes under a root
  extension, `x-ess-entities` — no path, no method, no query parameter — and adds a `$ref` to the
  schema of what the property identifies, which that document has and a self-contained schema
  document does not. An extension rather than `components.schemas` because that table is what
  `ess import openapi` reads back, and an entity's shape reaches a `Map` and a tagged union, which
  the adapter's subset does not carry: publishing them there made this repository's own adapter
  refuse the document it had generated. The synthesised Rust names the relation in the carrying field's doc comment rather than as a
  typed field: the specification describes, and nothing generated here has a store to navigate with.
- `examples/billing` carries one of them — `billing.invoice.Account` owns many
  `billing.invoice.Invoice`, by the invoice's `account_id` — so the pattern an adopter is pointed at
  is validated, compiled and projected rather than described. `CreateInvoice` takes the account and
  `InvoiceCreated` announces it, because an owner nobody named is an owner the implementation
  invented.
- The committed `generated/` tree is regenerated, which also lands 0.4.0's documentation filenames:
  `docs/index.md` and `docs/domains/billing-invoice.md` replace the `README.md` and dotted names the
  tree still carried. `schemas/generated/ess.schema.json` is regenerated from the Rust types, which
  additionally publishes the `order_by:` and quantifier constructs 0.4.0 added and the file had not
  caught up with.

## [0.4.0] — 2026-09-02

- **The documentation projection writes different filenames.** `--kind docs` now writes `index.md`
  rather than `README.md`, and `domains/acd-routing.md` rather than `domains/acd.routing.md`: a dot
  in a path segment makes a static file server read the name as having an extension, so every one
  of those pages was unservable behind GitLab Pages and an adopter carried a rename pass of their
  own. Committed output moves once; the links inside it move with it.
- Add `forall:` and `exists:` to the predicate language, quantifying an invariant over a `List` or
  a `Map`. A collection publishes its size as `<path>.count`, so no existing fact source changes to
  gain them, and one nobody observed evaluates to `unknown` rather than to the vacuous truth an
  empty one gives. Quantifying over anything that is not a collection is refused as
  `type_mismatch`.
- Add `order_by:` to a view, so a view named for a position says something about position.
  Generated conformance scenarios assert it on adjacent rows; a key the view does not project is
  refused, because a rank over a field nobody can read is a promise nothing can check.
- Add `ess conform synthesize --target go`, which writes a Go test package — the runner, a
  three-valued predicate evaluator and the suite — so an implementation in Go is held to the
  specification by `go test`. Previously `conform run` reached only the reference targets in this
  workspace, and an adopter's suite was regenerated on every model change and never executed.
- Add `refs:` to commands, outcomes, bindings and components, written `provider:key`, so a
  construct can name the ticket or incident that explains it. `Conversion.because` was the only
  prose field in the model, and everything else went into YAML comments no projection reads.
- Add `ess generate --kind site`: the documentation with frontmatter and a sidebar, for publishing
  as a static site.
- Print the refusals `conform synthesize` finds instead of counting them. A count says something is
  unchecked without saying what.
- Introduce `ess-docs/1`, an internal document representation between the model and the pages, with
  the markdown projection as one renderer of it. No output changes; the documentation of every
  example is pinned byte for byte and compared.

## [0.3.0] — 2026-09-01

- Add reusable named struct `shape` declarations for views, preserving expanded checked fields in
  compiler IR while OpenAPI reuses one component schema through `$ref`.
- Expose the compiler-owned semantic source digest on `EssIr`, so downstream builders and
  provenance records bind to one canonical implementation.
- Expose the exact versioned browser catalog through `ess_synth::web::browser_catalog`, so service
  generators and documentation hosts consume the same document as the ESS browser target.

## [0.2.1] — 2026-09-01

- Restore the extracted `schema-contract` command surface under `ess schema`: offline validation
  against an explicit schema registry and deterministic TypeScript projection with byte-check mode.
- Document and gate both commands so consumers no longer depend on the retired repository's CLI.

## [0.2.0] — 2026-09-01

- Make `ess-conformance-report/1` the only ESS conformance handoff and remove the legacy
  workflow-shaped evidence, producer, provenance, and verifier API. AEP adaptation now lives only
  in AEP's optional adapter.
- Use the canonical `ess` command in newly generated provenance guidance and current engineering
  documentation. Persisted ESS and infrastructure IR envelopes remain version 1.

## [0.1.1] — 2026-09-01

- Publish the extracted ESS adopter documentation and browser lab at the standalone ESS Pages
  site, with commands, links, and regeneration guidance written for the canonical `ess` CLI.

## [0.1.0] — 2026-09-01

- Extract ESS, infrastructure modeling, schema contracts, generators, conformance, examples, and
  suites into a standalone repository with no AEP dependency.
- Add the canonical `ess` command and explicit import/project adapter contract.
- Add typed OpenAPI service-interface import, deterministic projection, coverage reporting, and
  semantic round-trip checks for the adapter's declared subset.
- Import the Kubernetes credential-edge scanner with pre-write Secret sanitization.
- Publish `ess-conformance-report/1` for optional workflow-side evidence adapters.
