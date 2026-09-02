# Changelog

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
