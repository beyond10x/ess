---
format: aep.planning-md/3
id: story:schema-tools-accept-projection
kind: story
status: draft
title: ESS's own schema tools accept what ess generate --kind schema writes
summary: Projected roots can carry an $id, YAML instances validate, alphabet is asserted, import-document accepts x-ess-* (#590-#593)
refs:
- provider: github
  reference: beyond10x/ess#590
- provider: github
  reference: beyond10x/ess#591
- provider: github
  reference: beyond10x/ess#592
- provider: github
  reference: beyond10x/ess#593
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

ESS's own JSON Schema tools accept what `ess generate --kind schema` writes: a projected document can
carry an absolute root `$id` and be used by `ess generate schema validate` as it stands; instance
files written in YAML validate as the JSON they denote; a newtype's `alphabet:` is asserted by the
projected schema and not only annotated; and `ess generate schema import-document` accepts the
`x-ess-*` annotations the projector writes. One adopter report, four issues (beyond10x/ess#590,
#591, #592, #593), one story, because all four are the same defect: two tools in one CLI that do
not compose.

## Fit review

Tree `2c0378386` (0.62.0 unreleased); binary run: ess 0.61.0 (newest release). Minimal
reproductions in `~/.cache/ess-schema-tools/repro/` (examples/billing and a one-type spec
`alpha/system.yaml`); the adopter's files were not copied.

1. **Need.** A specification author projects JSON Schema and wants to check data files against it
   with the same CLI, with no post-processing. Today, on `examples/billing`:
   - `ess generate --kind schema --out gen` then `ess generate schema validate --schemas
     gen/schema/types inst` prints `[missing_schema_id] …: expected a non-empty absolute $id` for
     all 13 type documents (#590). The projector writes no `$id`
     (`crates/generate/ess-gen/src/schema.rs:166-187`), and the registry indexes only absolute ids
     (`crates/generate/schema-contract/src/validate.rs:124-139`).
   - a directory of `*.yaml` instances is refused, `the supplied paths contain no JSON instance
     files` (#591; `crates/edge/ess-cli/src/schema.rs:91-95`, `collect_json` keeps `.json` only,
     `:204-231`). `validate-bundle` reads JSON only too (`schema_bundle.rs:241-248`).
   - `alphabet: "0123456789abcdef"` projects to `{"type":"string","x-ess-alphabet":…}` (repro
     `alpha/`), which no validator asserts, so `ZZZ` passes (#592; `types.rs:304-309` records the
     annotation-only choice).
   - `ess generate schema import-document --path gen/schema/types/billing.email.EmailAddress.schema.json
     --root R --dialect draft-2020-12` refuses `/x-ess-provenance`, `/$defs/…/x-ess-kind` and
     `/$defs/…/x-ess-name` as `unsupported schema keyword` (#593; `bundle.rs:584-595`).
   Requester's proposed syntax, labelled as theirs: a declaration- or system-level id or a
   `--schema-id` option (#590); YAML accepted or `--format yaml` (#591); a derived `pattern`
   keeping `x-ess-alphabet` (#592); accept or ignore the `x-ess-*` keywords (#593).
2. **Class.** #590, #591, #593: *defect* — the CLI's two schema areas contradict each other on the
   CLI's own output; `docs/design/review-schema-resource-identity.md` documents #590 as accepted
   behaviour ("Neither producer emits a root `$id`", "does not select … automatic ID projection"),
   so it is a defect of composition rather than of one tool. #592: *defect* against
   `website/docs/reference/spec-versions.md:133` (the alphabet is a constraint on every value),
   which the projection silently drops.
3. **Already expressible?** #590: yes, by editing a copy of each projected file (the identity
   review's "accepted local representation"); that moves the id out of the projection, so every
   `--check` and every regeneration needs the edit again. #591: `yq -o=json` per file outside ESS.
   #592: no — no authored construct puts a `pattern` in the projection. #593: no.
   A selector-free path exists: `import-document` + `validate-bundle --root` needs no `$id` and no
   `schema` property in the instance (`schema_bundle.rs:241-265`, `bundle.rs:266-279`) — it is
   blocked only by #593.
4. **Fit.**
   - #590 reuses the `$id` rule the registry already states (absolute, `validate.rs:311-331`) and
     the adopter-owned identity policy of the identity review (`review-schema-resource-identity.md`
     "The adopter owns uniqueness, versioning and publication policy"). The id is
     `<base><path under schema/>`, the path the artifact already has (`schema.rs:182-186`,
     `artifact.rs:184`), so one base gives 29 distinct ids and a tree published at the base
     resolves. A per-kind CLI input follows `--http` / `--transport` (`main.rs:1683-1689`) and
     their modelled refusal (`models/toolchain/domains/generate.yaml`, `binding-without-openapi`).
   - #591 reuses `ess-yaml`, the one YAML reader of the workspace (`crates/specify/ess-yaml/src/lib.rs:1-12`),
     the JSON-shape refusals `ess-openapi` applies to a YAML document (`ess-openapi/src/accounting.rs:256-300`:
     tags, non-string keys, non-finite numbers), and ESS's number rule
     (`ess_primitives::json::number`). Extension decides, as `read_document` does (`main.rs:1860-1872`).
   - #592 keeps the annotation and adds an assertion where it is certainly right: every alphabet
     character in the Basic Multilingual Plane, written as `\uXXXX` in the class, which ECMA-262
     reads the same with or without the `u` flag — the exact reason `types.rs:304-307` gives for
     refusing a class. A prefix (`types.rs:762-778`) is inside the alphabet by validation
     (`ess-domain/src/types.rs:1142-1147`, `:1481-1508`), so `^<prefix>[<class>]*$` is the
     conjunction of both.
   - #593: `x-ess-` is ESS's annotation namespace (`types.rs:239-326`), and the realizer already
     reads these keywords from a bundle as annotations or obligations (`realize.rs:913-933`). The
     importer retains them unchanged and never asserts them; any other `x-` keyword is still
     refused.
   - Targets: nothing in Rust, Go, TypeScript, web, the interpreter, `ess verify diff`, the entity
     runtime or synthesis reads a projected JSON Schema's `$id` or the instance file format. #592
     changes the bytes of every projection that embeds the shared type mapping (schema, OpenAPI,
     AsyncAPI: `schema.rs:8-12`).
5. **Second adopter.** A team that projects JSON Schema for configuration files kept in YAML (a
   deployment manifest, a feature-flag file) and checks them in CI with `ess`; a service that
   publishes its projected schemas at `https://schemas.example.org/orders/` and validates incoming
   documents against the published ids; any `String` newtype over a code alphabet (hex digests,
   base32 tokens, DTMF digits as in `crates/verify/ess-conformance/tests/fixtures/keypad.yaml`).
6. **Cost.** No authored key, no source format, no new diagnostic code in `ess-domain`. #590: one
   CLI option, one `Node` field; default output byte-identical. #591: none outside the CLI.
   #593: no bundle format change; a document without `x-ess-*` keywords imports to the same bytes
   (`ess-schema-bundle/1`, `/2`). #592: a byte change in generated schema, OpenAPI and AsyncAPI
   documents of alphabet newtypes (one `pattern` line); adopters' `--check` reports drift once;
   the repository's committed projections and site data regenerate.
7. **Designs considered.** (a) Change nothing and document the workarounds — refused: the
   workaround re-edits generated files after every generation, and policy 1 of
   `docs/design/generic-language-features.md:468-473` refuses idioms that take a checked figure out
   of ESS. (b) Requester's authored id (declaration- or system-level) — refused: an id is a
   publication fact, not a domain fact; it would cost a source format and a diff kind. (c) One
   `--schema-id` — refused: 29 documents would share it (`duplicate_schema_id`). (d) An ESS-owned
   namespace (`urn:ess:…`) always written — refused: changes every adopter's bytes and identity,
   which AGENTS.md "Determinism and formats" ties to a coordinated migration, and the identity
   review leaves the namespace to the adopter. **(e) chosen:** opt-in `--schema-id-base`. For #591
   a `--format yaml` flag was refused: the extension already says it, and a mixed directory would
   need two runs.

## Decisions

**Accept, redesigned** (all four issues).

- #590: `ess generate --kind schema --schema-id-base <base>` writes a root `$id` of `<base>` followed
  by the document's path under `schema/` (`types/<name>.schema.json`, `commands/…`, …), placed
  after `$schema`. The base is an absolute URI (RFC 3986 scheme), ends in `/` or `:`, and has no
  `?`, `#`, whitespace or control character; anything else is refused before resolution.
  `--schema-id-base` with any other `--kind`, or with none, is refused. Without it nothing moves,
  so no format version: the projection's default bytes and identity are unchanged, and the opted-in
  identity is the adopter's. The registry keeps selecting by an instance's `schema` property.
- #591: `ess generate schema validate` and `validate-bundle` read instance files ending `.yaml` or
  `.yml` as YAML through `ess-yaml` (one document; duplicate keys, tags, non-string keys and
  non-finite numbers refused with the file and pointer), numbers as ESS reads JSON numbers. Every
  other instance file is read as JSON, as before. Schemas stay `*.schema.json`.
- #592: a newtype's projection carries `pattern: ^<escaped prefix>[<class>]*$` (or `^[<class>]*$`)
  when every alphabet character is in the BMP, the class written as `\uXXXX`; with a supplementary
  character it keeps today's annotation and prefix pattern. `x-ess-alphabet` stays. The
  types-only realizer must not report the alphabet twice. Second slice.
- #593: `import-bundle` and `import-document` accept every keyword starting `x-ess-` as an
  annotation, retained unchanged and never transformed; other `x-` keywords are still refused.
- Not in this story: a selector-free mode for the registry `validate` (the bundle path covers it);
  #594, assessed separately.

## Acceptance

- Specification: `models/toolchain/domains/generate.yaml` `PlanProjection` takes
  `schema_id_base`, refuses `schema-id-without-schema` and `schema-id-base-refused`;
  `ess specify validate --path models/toolchain` and `ess verify conform author --path
  models/toolchain --scenarios models/toolchain-scenarios` are green.
- `crates/generate/ess-gen/tests/schema.rs`: `issue_590_a_schema_id_base_identifies_every_document_by_its_path`,
  `issue_590_without_a_base_no_document_moves`, `issue_590_a_base_that_is_not_an_absolute_prefix_is_refused`.
- `crates/edge/ess-cli/tests/schema_registry_identity.rs`, module `projected`:
  `issue_590_the_registry_validates_against_identified_projections`,
  `issue_590_a_schema_id_base_needs_kind_schema`,
  `issue_591_a_yaml_instance_validates_as_the_json_it_denotes`,
  `issue_591_a_yaml_instance_outside_json_is_refused_by_file`,
  `issue_593_a_projected_document_imports_and_validates_a_yaml_instance`.
- `crates/generate/schema-contract/tests/bundle.rs`: `issue_593_ess_annotations_are_retained`,
  `issue_593_a_foreign_extension_keyword_is_still_refused`.
- Second slice (#592): `crates/generate/ess-gen/tests/schema.rs`
  `issue_592_an_alphabet_projects_an_anchored_class_pattern`,
  `issue_592_a_supplementary_alphabet_keeps_the_annotation_only`; `crates/edge/ess-cli/tests/schema_registry_identity.rs`
  `issue_592_validate_refuses_a_value_outside_the_alphabet`; `task projection-check` green after
  regeneration.

## Scope

Cited: `crates/generate/ess-gen/src/schema.rs`, `crates/generate/ess-gen/src/types.rs` (`Node`,
`body`), `crates/edge/ess-cli/src/main.rs` (`GenerateArgs`, `generate_projections`, `generate`),
`crates/edge/ess-cli/src/schema.rs`, `crates/edge/ess-cli/src/schema_bundle.rs`,
`crates/generate/schema-contract/src/bundle.rs` (`transform`), `models/toolchain/domains/generate.yaml`,
`website/docs/reference/cli.md`, `CHANGELOG.md`. Second slice also: `crates/generate/schema-contract/src/realize.rs`,
`generated/`, `website/data/`. Must not change: `ess-schema-bundle/1` and `/2` bytes for documents
without `x-ess-*` keywords; default `--kind schema` bytes in the first slice.

## Sequencing

First slice: #590, #591, #593 (no projection bytes move). Second slice: #592 (regenerates committed
projections; lands alone so its drift is the only drift).
