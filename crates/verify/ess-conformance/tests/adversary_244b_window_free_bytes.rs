//! Adversary case, beyond10x/ess#244 part b, pass 1: every ESS model in the repository that carries
//! no calendar window keeps, byte for byte against the unit's base `3b1684d1a`, its canonical IR,
//! its synthesized suite, its synthesis refusals, its mutant list (ids and changes) and, where it
//! assembles at all, the same refusal text — as written, and relabelled `format: ess/22`.
//!
//! `fixtures/adversary-244b-base-digests.tsv` was written by this same file run with
//! `ADV244B_WRITE=<path>` in an export of `3b1684d1a` (`git archive`), built in its own target
//! directory.

// Style lints only, so the probe stays byte for byte what wrote the base digests.
#![allow(clippy::needless_pass_by_value, clippy::format_collect)]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::mutate::{self, MutantClass};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use sha2::{Digest, Sha256};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap()
}

fn digest(bytes: &str) -> String {
    let hex =
        Sha256::digest(bytes.as_bytes())
            .iter()
            .take(10)
            .fold(String::new(), |mut hex, byte| {
                write!(hex, "{byte:02x}").unwrap();
                hex
            });
    format!("{hex}:{}", bytes.len())
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.is_dir() {
            if !["target", "node_modules", ".git", ".engineering"].contains(&name.as_str()) {
                walk(&path, out);
            }
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            out.push(path);
        }
    }
}

fn line(documents: Vec<(Source, RawSpecFile)>, sources: &SourceMap) -> String {
    let spec = match Specification::assemble(documents.clone()) {
        Ok(spec) => spec,
        Err(errors) => return format!("ASSEMBLE {}", digest(&errors.to_string())),
    };
    let ir = match compile(&spec, sources) {
        Ok(ir) => ir,
        Err(error) => return format!("COMPILE {}", digest(&format!("{error:?}"))),
    };
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    let suite = synthesis
        .suite
        .to_canonical_json()
        .unwrap_or_else(|error| format!("SUITE-ERROR {error:?}"));
    let mutants: Vec<String> = mutate::mutants(&documents, MutantClass::ALL)
        .iter()
        .map(|mutant| format!("{}\t{}", mutant.id, mutant.change))
        .collect();
    format!(
        "ir={} suite={} refusals={} mutants={}",
        digest(&ir.to_canonical_json()),
        digest(&suite),
        digest(&refusals.join("\n")),
        digest(&mutants.join("\n"))
    )
}

fn single(label: &str, text: &str) -> String {
    match RawSpecFile::parse(text) {
        Ok(raw) => {
            let mut sources = SourceMap::new();
            sources.insert(label.to_owned(), text.to_owned());
            line(vec![(Source::new(label.to_owned()), raw)], &sources)
        }
        Err(error) => format!("PARSE {}", digest(&error.to_string())),
    }
}

/// Every model this tree holds: each ESS source file alone, as written and relabelled `ess/22`,
/// and each `examples/<name>` directory as one system.
fn digests(root: &Path) -> Vec<(String, String)> {
    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    let mut found = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if !text.lines().any(|line| line.starts_with("format: ess/")) {
            continue;
        }
        let label = path.strip_prefix(root).unwrap().display().to_string();
        found.push((label.clone(), single(&label, &text)));
        let relabelled: String = text
            .lines()
            .map(|line| {
                if line.starts_with("format: ess/") {
                    "format: ess/22".to_owned()
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        if relabelled.trim_end() != text.trim_end() {
            found.push((format!("{label}@ess22"), single(&label, &relabelled)));
        }
    }
    let examples = root.join("examples");
    if let Ok(entries) = std::fs::read_dir(&examples) {
        let mut dirs: Vec<PathBuf> = entries
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .collect();
        dirs.sort();
        for dir in dirs {
            let mut members = Vec::new();
            walk(&dir, &mut members);
            members.sort();
            let mut sources = SourceMap::new();
            let mut documents = Vec::new();
            let mut parse_error = None;
            for path in members {
                let text = std::fs::read_to_string(&path).unwrap();
                if !text.lines().any(|line| line.starts_with("format: ess/")) {
                    continue;
                }
                let label = path.strip_prefix(&dir).unwrap().display().to_string();
                match RawSpecFile::parse(&text) {
                    Ok(raw) => {
                        sources.insert(label.clone(), text);
                        documents.push((Source::new(label), raw));
                    }
                    Err(error) => parse_error = Some(error.to_string()),
                }
            }
            if documents.is_empty() {
                continue;
            }
            let name = dir.strip_prefix(root).unwrap().display().to_string();
            let value = match parse_error {
                Some(error) => format!("PARSE {}", digest(&error)),
                None => line(documents, &sources),
            };
            found.push((format!("examples-dir:{name}"), value));
        }
    }
    found
}

const BASE: &str = include_str!("fixtures/adversary-244b-base-digests.tsv");

/// Moves a later release made on purpose, each admitted only at the exact digests it produced, so
/// any further change to the same model fails the probe again.
///
/// 0.54.0 (beyond10x/ess#454, beyond10x/ess#455): an input refusal's scenario also arranges an
/// unknown identity beside stored-row branches, and the overlap of two input refusals; the
/// `id-required` and `secret-too-short` scenarios of `arrangement-input-refusal` and the
/// `blank-note` scenario of `now-stored-rows` gain steps. IR, refusals and mutants are unchanged.
///
/// 0.60.0 (beyond10x/ess#515): `ess verify conform mutate` mutates `when_subject` and
/// `when_related` predicates and accounts for every `sets` entry, so each model below that carries
/// one gains mutants. IR, suite and refusals are unchanged; `now-stored-rows` keeps its 0.54.0
/// suite.
///
/// Up to 0.67.0, first recorded at the 0.68.0 release bar: moves this probe was not run against.
/// `model-types` refuses with a text already pinned in 0.58.0; synthesis holds the clock on each
/// `now` boundary (0.62.0, beyond10x/ess#586, `8a0ca3016`), so `current-time-guard`,
/// `now-stored-rows` (superseding its 0.60.0 entry) and `occurrence-clock-offers` gain a held
/// scenario; in 0.63.0 committed suites state suite-wide rules once (beyond10x/ess#572) and the
/// ess/27 suite majors move, so the other suites shrink, a recursive value-invariant refusal names
/// what it lacks (`adversary_expression`), and the scrub `34db9a1d4` renames a name the
/// `outcome-groups` assembly refusal quotes. Each suite and refusal digest equals the one
/// `synthesis-suite-bytes-table.tsv` pinned at 0.67.0; `conditional-measures-generated`, which
/// that table excludes, is ess/22 and outside every 0.68.0 synthesis change. IR and mutants are
/// unchanged.
///
/// After 0.82.0 (story a-no-view-held-state-refusal-probes-the-record): a held-state input refusal
/// on a model with no identity view reads the record back, so the `occurrence-clock-offers` suite
/// grows; its `malformed` scenario reads the offer back in each state. IR, refusals and mutants
/// are unchanged.
///
/// 0.85.0 (beyond10x/ess#709, beyond10x/ess#710): the mutation audit swaps the error of a refusal
/// that carries its error's payload (`578fde2652`) and drops or adds actor grants (`ad383e41a5`),
/// so each model below with such a refusal or a declared actor gains or changes mutants through
/// `MutantClass::ALL`; an entry above superseded here keeps its IR, suite and refusals. IR, suite
/// and refusals are unchanged.
///
/// Up to 0.82.0, first recorded at the 0.85.0 release bar: two suite moves this probe was not run
/// against, each already re-pinned in `synthesis-suite-bytes-table.tsv` by its story.
/// `text-length-positions` reaches a Binary64 and synthesizes at `ess-conformance/62` since 0.79.0
/// (`915fe1fca8`, story a-binary64-value-round-trips-through-a-conformance-suite), so its suite and
/// refusals grow. Since 0.82.0 `binding-arrangement`'s binding scenarios compare their trigger
/// event's identities (`b1a810d3d7`, story binding-site-identity-comparisons), so its suite grows.
/// Their mutants are those of the 0.85.0 audit. IR is unchanged.
const INTENDED: &[(&str, &str)] = &[
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-input-refusal.yaml",
        "ir=8f73acbe936097824f8e:9715 suite=d10530fe00b5eb7bfc37:32088 refusals=e3b0c44298fc1c149afb:0 mutants=ec81ef6209f0321499fb:1778",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-input-refusal.yaml@ess22",
        "ir=8f73acbe936097824f8e:9715 suite=d10530fe00b5eb7bfc37:32088 refusals=e3b0c44298fc1c149afb:0 mutants=ec81ef6209f0321499fb:1778",
    ),
    (
        "crates/specify/ess-domain/tests/fixtures/text-length-positions.yaml",
        "ir=64397198ca8b3f008517:12944 suite=4396e0632c2955f3febf:26366 refusals=09a7f9efe74b5bfe99fd:1604 mutants=1b060c619d9481a75e3a:679",
    ),
    (
        "crates/specify/ess-domain/tests/fixtures/text-length-positions.yaml@ess22",
        "ir=64397198ca8b3f008517:12944 suite=4396e0632c2955f3febf:26366 refusals=09a7f9efe74b5bfe99fd:1604 mutants=1b060c619d9481a75e3a:679",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-stored-guarded-move.yaml",
        "ir=5500cec8d73ec1066754:12118 suite=c9591218350b81be905c:41344 refusals=e3b0c44298fc1c149afb:0 mutants=e718573da44217168ab5:1029",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-stored-guarded-move.yaml@ess22",
        "ir=5500cec8d73ec1066754:12118 suite=c9591218350b81be905c:41344 refusals=e3b0c44298fc1c149afb:0 mutants=e718573da44217168ab5:1029",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-paid-route.yaml",
        "ir=8078b5131c26dfc079fd:13470 suite=1cf6db0f3e0a10bbebd7:57258 refusals=e3b0c44298fc1c149afb:0 mutants=00bc8c3988bb2541f96d:1064",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-paid-route.yaml@ess22",
        "ir=8078b5131c26dfc079fd:13470 suite=1cf6db0f3e0a10bbebd7:57258 refusals=e3b0c44298fc1c149afb:0 mutants=00bc8c3988bb2541f96d:1064",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-two-creators.yaml",
        "ir=4a198eb2e776e280388f:12951 suite=835f415354c704786882:50441 refusals=e3b0c44298fc1c149afb:0 mutants=373386148355939d3b45:1068",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-two-creators.yaml@ess22",
        "ir=4a198eb2e776e280388f:12951 suite=835f415354c704786882:50441 refusals=e3b0c44298fc1c149afb:0 mutants=373386148355939d3b45:1068",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-stored-rows.yaml",
        "ir=3ade52cbdd7f200803c5:38581 suite=0415c4076059d44644d8:196402 refusals=e3b0c44298fc1c149afb:0 mutants=8d24e6739bcfc9be352b:3488",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-stored-rows.yaml@ess22",
        "ir=3ade52cbdd7f200803c5:38581 suite=0415c4076059d44644d8:196402 refusals=e3b0c44298fc1c149afb:0 mutants=8d24e6739bcfc9be352b:3488",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-copied-value.yaml",
        "ir=2f88b3a80c3a08f99c5e:12758 suite=c9885fcc7d121c6cd204:16434 refusals=e3b0c44298fc1c149afb:0 mutants=e24ad1707721d690f8c9:566",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-copied-value.yaml@ess22",
        "ir=2f88b3a80c3a08f99c5e:12758 suite=c9885fcc7d121c6cd204:16434 refusals=e3b0c44298fc1c149afb:0 mutants=e24ad1707721d690f8c9:566",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-multiple.yaml",
        "ir=ecf9fba0c35c8f05f6f6:23733 suite=40c28675f65f47ca763a:89794 refusals=dc01d36d68b12c2cacb2:1043 mutants=285e9e2394efe76358b5:3234",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-optional.yaml",
        "ir=ad19424ea5327eca6573:16818 suite=0345d2d3f2d6c5fed768:57439 refusals=d30825fef61960483efb:550 mutants=331c0d2aec9dd33c4748:2439",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-owner-link.yaml",
        "ir=838035241c8956c47216:18375 suite=8321029dd2307525da7b:47863 refusals=e3b0c44298fc1c149afb:0 mutants=e2fafa9a400d2dc8ce83:878",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-owner-link.yaml@ess22",
        "ir=838035241c8956c47216:18375 suite=8321029dd2307525da7b:47863 refusals=e3b0c44298fc1c149afb:0 mutants=e2fafa9a400d2dc8ce83:878",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-release.yaml",
        "ir=f94e97df357fa758fd87:15482 suite=13d157c274d5d2cef8d9:43973 refusals=66eb987124e594b9fa5f:1219 mutants=9e501d721b4e2280670e:943",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-release.yaml@ess22",
        "ir=f94e97df357fa758fd87:15482 suite=13d157c274d5d2cef8d9:43973 refusals=66eb987124e594b9fa5f:1219 mutants=9e501d721b4e2280670e:943",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-sign-in.yaml",
        "ir=29cfb8aef921be1c775d:13029 suite=a4720f0b8e486a522b1d:17016 refusals=e3b0c44298fc1c149afb:0 mutants=93245710a26963f67c04:711",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-sign-in.yaml@ess22",
        "ir=29cfb8aef921be1c775d:13029 suite=a4720f0b8e486a522b1d:17016 refusals=e3b0c44298fc1c149afb:0 mutants=93245710a26963f67c04:711",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-stored-reference.yaml",
        "ir=4991ac392e7b2fe208cf:10985 suite=d2806897f3d0cd08f3d6:35833 refusals=e3b0c44298fc1c149afb:0 mutants=9e54fa14795ad562c2ce:1262",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/state-in-subject-predicate.yaml",
        "ir=93ad22bc62ea70588000:11608 suite=db49e2aec3c2f0f40957:36777 refusals=e3b0c44298fc1c149afb:0 mutants=2e87de384bfa82a1c23a:1120",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/state-in-subject-predicate.yaml@ess22",
        "ir=93ad22bc62ea70588000:11608 suite=db49e2aec3c2f0f40957:36777 refusals=e3b0c44298fc1c149afb:0 mutants=2e87de384bfa82a1c23a:1120",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/stored-field-guards.yaml",
        "ir=f5690ad7914f4165a715:8364 suite=e55baa7c2c4be0a2989c:26862 refusals=06a644cde2b8fc51e318:546 mutants=bbb4499a2f68c35ba137:958",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/stored-field-guards.yaml@ess22",
        "ir=f5690ad7914f4165a715:8364 suite=e55baa7c2c4be0a2989c:26862 refusals=06a644cde2b8fc51e318:546 mutants=bbb4499a2f68c35ba137:958",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/subject-guard-copied-field.yaml",
        "ir=bbf12fce16b887c3b60d:20824 suite=2705e065bf055f8f707c:124591 refusals=e3b0c44298fc1c149afb:0 mutants=5eecdf7785afa73c00eb:2222",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/subject-guard-copied-field.yaml@ess22",
        "ir=bbf12fce16b887c3b60d:20824 suite=2705e065bf055f8f707c:124591 refusals=e3b0c44298fc1c149afb:0 mutants=5eecdf7785afa73c00eb:2222",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/subject-guard-input.yaml",
        "ir=aa83bc892312d4854d25:7016 suite=b7c14452cfcd4040e342:18848 refusals=9abbe950081eeb90978c:496 mutants=2c64ef09502d2eaffd91:482",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/subject-guard-input.yaml@ess22",
        "ir=aa83bc892312d4854d25:7016 suite=b7c14452cfcd4040e342:18848 refusals=9abbe950081eeb90978c:496 mutants=2c64ef09502d2eaffd91:482",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/when-subject-eventual.yaml",
        "ir=5781561e33e4cb327676:11017 suite=dd22493eb4d33d9bc62b:28607 refusals=2a80cca606aa08bb095f:1009 mutants=c8823e551069a1f82bcf:434",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/when-subject-eventual.yaml@ess22",
        "ir=5781561e33e4cb327676:11017 suite=dd22493eb4d33d9bc62b:28607 refusals=2a80cca606aa08bb095f:1009 mutants=c8823e551069a1f82bcf:434",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/when-subject-fallback-refusal.yaml",
        "ir=f59eafbb12aebf53ff70:11025 suite=f7ba7d533df0abd1de62:29867 refusals=2a80cca606aa08bb095f:1009 mutants=c8823e551069a1f82bcf:434",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/when-subject-fallback-refusal.yaml@ess22",
        "ir=f59eafbb12aebf53ff70:11025 suite=f7ba7d533df0abd1de62:29867 refusals=2a80cca606aa08bb095f:1009 mutants=c8823e551069a1f82bcf:434",
    ),
    (
        "crates/generate/ess-gen/tests/fixtures/model-types.yaml",
        "ir=91f0042ec0ca486d8095:4855 suite=01c5b843e68efa1d3e0a:363 refusals=bd8344ba32d2d00918b3:359 mutants=e3b0c44298fc1c149afb:0",
    ),
    (
        "crates/generate/ess-gen/tests/fixtures/model-types.yaml@ess22",
        "ir=91f0042ec0ca486d8095:4855 suite=01c5b843e68efa1d3e0a:363 refusals=bd8344ba32d2d00918b3:359 mutants=e3b0c44298fc1c149afb:0",
    ),
    (
        "crates/generate/ess-synth/tests/fixtures/bounded-accessor.yaml",
        "ir=56aa0e7b552a4b29064d:24246 suite=5e31ae628a820681794e:7176 refusals=e35b67499ded0851e93c:837 mutants=b4876b800d0cb60ea19c:221",
    ),
    (
        "crates/generate/ess-synth/tests/fixtures/conditional-measures-generated.yaml",
        "ir=d35e9f1aea844db65444:20928 suite=fda0d69e641a88ad0183:21999 refusals=b3c848d48cfb9f249ead:518 mutants=d8beb08e55fb3f499156:1097",
    ),
    (
        "crates/generate/ess-synth/tests/fixtures/generated-views.yaml",
        "ir=d684139b5be5288707c0:29577 suite=f88bb646379f8be8ac10:67900 refusals=e3b0c44298fc1c149afb:0 mutants=66176fd1765309a19637:1220",
    ),
    (
        "crates/generate/ess-synth/tests/fixtures/generated-views.yaml@ess22",
        "ir=d684139b5be5288707c0:29577 suite=f88bb646379f8be8ac10:67900 refusals=e3b0c44298fc1c149afb:0 mutants=66176fd1765309a19637:1220",
    ),
    (
        "crates/specify/ess-compiler/tests/fixtures/adversary_expression.yaml",
        "ir=a4b8c0d2ebcb25499bd0:9586 suite=4569c561f09159c1bf4c:1288 refusals=e469261ed098131f7390:7620 mutants=4a0563e816ec3b664346:373",
    ),
    (
        "crates/specify/ess-compiler/tests/fixtures/adversary_expression.yaml@ess22",
        "ir=a4b8c0d2ebcb25499bd0:9586 suite=4569c561f09159c1bf4c:1288 refusals=e469261ed098131f7390:7620 mutants=4a0563e816ec3b664346:373",
    ),
    (
        "crates/specify/ess-domain/tests/fixtures/outcome-groups/system.yaml",
        "ASSEMBLE e81263926bcf9e3938ac:451",
    ),
    (
        "crates/specify/ess-domain/tests/fixtures/outcome-groups/system.yaml@ess22",
        "ASSEMBLE e81263926bcf9e3938ac:451",
    ),
    (
        "crates/specify/ess-domain/tests/fixtures/outcome-groups-twin/system.yaml",
        "ASSEMBLE e81263926bcf9e3938ac:451",
    ),
    (
        "crates/specify/ess-domain/tests/fixtures/outcome-groups-twin/system.yaml@ess22",
        "ASSEMBLE e81263926bcf9e3938ac:451",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/adversary-explore-preconditions.yaml",
        "ir=7450b532629f518662ed:9168 suite=18d4a7deb220bf07e8bc:11304 refusals=e3b0c44298fc1c149afb:0 mutants=84c25dad3f30a55122e8:421",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/adversary-explore-preconditions.yaml@ess22",
        "ir=7450b532629f518662ed:9168 suite=18d4a7deb220bf07e8bc:11304 refusals=e3b0c44298fc1c149afb:0 mutants=84c25dad3f30a55122e8:421",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-group-selection.yaml",
        "ir=dde087b272d309d3fd18:28813 suite=4c5e94e707b5da248c17:142947 refusals=e3b0c44298fc1c149afb:0 mutants=b14f786855f38bab6e59:548",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-group-selection.yaml@ess22",
        "ir=dde087b272d309d3fd18:28813 suite=4c5e94e707b5da248c17:142947 refusals=e3b0c44298fc1c149afb:0 mutants=b14f786855f38bab6e59:548",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-optional-fields.yaml",
        "ir=b5c5844d95d664bb1f49:17481 suite=0b8961c5dae080a0863a:33980 refusals=e3b0c44298fc1c149afb:0 mutants=70f85b53de66a66b0c57:306",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-optional-fields.yaml@ess22",
        "ir=b5c5844d95d664bb1f49:17481 suite=0b8961c5dae080a0863a:33980 refusals=e3b0c44298fc1c149afb:0 mutants=70f85b53de66a66b0c57:306",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/binding-condition-selected.yaml",
        "ir=7b361612162d72a2fbd4:19496 suite=9281dc6aae632abc1d51:43631 refusals=0755454e3fe41dad0214:276 mutants=2e782cbc905c8eafa2b8:376",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/binding-condition.yaml",
        "ir=55b03bf14a61335ee1dc:14509 suite=e7999fe6cf6055b96d1e:36626 refusals=0755454e3fe41dad0214:276 mutants=2e782cbc905c8eafa2b8:376",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/current-time-guard.yaml",
        "ir=a7cacc358c4e2d309a51:6366 suite=0fc77cd2ffc7e77dbdc2:6906 refusals=e3b0c44298fc1c149afb:0 mutants=d9db1f7715d7c6ac4e8a:343",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/current-time-guard.yaml@ess22",
        "ir=a7cacc358c4e2d309a51:6366 suite=0fc77cd2ffc7e77dbdc2:6906 refusals=e3b0c44298fc1c149afb:0 mutants=d9db1f7715d7c6ac4e8a:343",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/now-stored-rows.yaml",
        "ir=fad93168fa11c42db418:30720 suite=7548062c85b64cffd671:140160 refusals=b0abbc30c3882c8c5037:423 mutants=4bb20b47d0340b004585:3915",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/occurrence-clock-offers.yaml",
        "ir=100a4d422734bdd51af9:14053 suite=4b8a770fbb6241c13f00:30533 refusals=4fd558db28f2b7a105b0:2473 mutants=9ef6a40600673b199930:1609",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/occurrence-clock-offers.yaml@ess22",
        "ir=100a4d422734bdd51af9:14053 suite=4b8a770fbb6241c13f00:30533 refusals=4fd558db28f2b7a105b0:2473 mutants=9ef6a40600673b199930:1609",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/one-time-coverage/model.yaml",
        "ir=e1adf075661567026afe:1197 suite=cc8b28aa35b5d547f10a:10318 refusals=e3b0c44298fc1c149afb:0 mutants=e3b0c44298fc1c149afb:0",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/one-time-coverage/model.yaml@ess22",
        "ir=e1adf075661567026afe:1197 suite=cc8b28aa35b5d547f10a:10318 refusals=e3b0c44298fc1c149afb:0 mutants=e3b0c44298fc1c149afb:0",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-values-chained.yaml",
        "ir=ab75682e9e4308374158:20260 suite=a35d1b135cca44237fc3:28283 refusals=e3b0c44298fc1c149afb:0 mutants=b12923907f4ed6a4f456:421",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/structured-instances.yaml",
        "ir=60a062a30bd57660f15c:13631 suite=05dd44d9c5aac092e3fb:5833 refusals=e3b0c44298fc1c149afb:0 mutants=0d05a6f55f3ef7ad4466:242",
    ),
    (
        "docs/design/one-time-response-values.example.yaml",
        "ir=e1adf075661567026afe:1197 suite=cc8b28aa35b5d547f10a:10318 refusals=e3b0c44298fc1c149afb:0 mutants=e3b0c44298fc1c149afb:0",
    ),
    (
        "docs/design/one-time-response-values.example.yaml@ess22",
        "ir=e1adf075661567026afe:1197 suite=cc8b28aa35b5d547f10a:10318 refusals=e3b0c44298fc1c149afb:0 mutants=e3b0c44298fc1c149afb:0",
    ),
    (
        "crates/generate/ess-synth/tests/fixtures/declared-behaviour-kept/head.yaml",
        "ir=fa570c8ab24273453c79:12503 suite=606309ce55c3066253f8:11675 refusals=6351d26f2df9eb142024:489 mutants=7b0e34f46010d8ae9889:529",
    ),
    (
        "crates/generate/ess-synth/tests/fixtures/declared-behaviour-kept/head.yaml@ess22",
        "ir=fa570c8ab24273453c79:12503 suite=606309ce55c3066253f8:11675 refusals=6351d26f2df9eb142024:489 mutants=7b0e34f46010d8ae9889:529",
    ),
    (
        "crates/generate/ess-synth/tests/fixtures/served-notes/system.yaml",
        "ir=a10f6b66cb12080e6ad9:10372 suite=e7c4197122f7f69497af:17268 refusals=e3b0c44298fc1c149afb:0 mutants=a1ab11e39c3732b53c0a:861",
    ),
    (
        "crates/generate/ess-synth/tests/fixtures/served-notes/system.yaml@ess22",
        "ir=a10f6b66cb12080e6ad9:10372 suite=e7c4197122f7f69497af:17268 refusals=e3b0c44298fc1c149afb:0 mutants=a1ab11e39c3732b53c0a:861",
    ),
    (
        "crates/specify/ess-compiler/tests/fixtures/absent-input.yaml",
        "ir=801a9d68b42aab16b6b2:8377 suite=9516b07e88725955db27:7512 refusals=e3b0c44298fc1c149afb:0 mutants=4ec9bf1e823b55131125:883",
    ),
    (
        "crates/specify/ess-compiler/tests/fixtures/absent-input.yaml@ess22",
        "ir=801a9d68b42aab16b6b2:8377 suite=9516b07e88725955db27:7512 refusals=e3b0c44298fc1c149afb:0 mutants=4ec9bf1e823b55131125:883",
    ),
    (
        "crates/specify/ess-compiler/tests/fixtures/outcome-shapes.yaml",
        "ir=018a7e61bd946502226b:20043 suite=ee2063f0208eb5ae0c98:69308 refusals=e3b0c44298fc1c149afb:0 mutants=5009063e39caacc80f55:671",
    ),
    (
        "crates/specify/ess-compiler/tests/fixtures/outcome-shapes.yaml@ess22",
        "ir=018a7e61bd946502226b:20043 suite=ee2063f0208eb5ae0c98:69308 refusals=e3b0c44298fc1c149afb:0 mutants=5009063e39caacc80f55:671",
    ),
    (
        "crates/specify/ess-compiler/tests/fixtures/set-effects.yaml",
        "ir=ed5d257e2d8ae1f25388:22625 suite=9a3ba662a3a80b1871d7:95405 refusals=e3b0c44298fc1c149afb:0 mutants=916f7b24935a445210ac:749",
    ),
    (
        "crates/specify/ess-compiler/tests/fixtures/set-effects.yaml@ess22",
        "ir=ed5d257e2d8ae1f25388:22625 suite=9a3ba662a3a80b1871d7:95405 refusals=e3b0c44298fc1c149afb:0 mutants=916f7b24935a445210ac:749",
    ),
    (
        "crates/specify/ess-compiler/tests/fixtures/union-unit-variants.yaml",
        "ir=a20a62e5ee676443f9a3:3431 suite=9e8c6cbbc91791a52108:2981 refusals=e3b0c44298fc1c149afb:0 mutants=162aeb4454314d0ea805:234",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/adversary-273-tree.yaml",
        "ir=d31dabe80b5d4e697cfa:10402 suite=8efcd8eaa15856def1f0:14540 refusals=7f9b01336e077ac04d0f:496 mutants=ac63aad790fe383ccb6e:232",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/adversary-273-tree.yaml@ess22",
        "ir=d31dabe80b5d4e697cfa:10402 suite=8efcd8eaa15856def1f0:14540 refusals=7f9b01336e077ac04d0f:496 mutants=ac63aad790fe383ccb6e:232",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-copied-group-parameter.yaml",
        "ir=d581148ce3b8124b0630:10865 suite=ee14d1cb0db50ab443d7:15948 refusals=e3b0c44298fc1c149afb:0 mutants=0f47b778529b1b1dddd5:390",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-copied-group-parameter.yaml@ess22",
        "ir=d581148ce3b8124b0630:10865 suite=ee14d1cb0db50ab443d7:15948 refusals=e3b0c44298fc1c149afb:0 mutants=0f47b778529b1b1dddd5:390",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-group-parameter.yaml",
        "ir=eb49f051ad3774e39931:8051 suite=df58e6911c0eb322b6bc:18098 refusals=e3b0c44298fc1c149afb:0 mutants=88f355cd6099e0ac438c:276",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-group-parameter.yaml@ess22",
        "ir=eb49f051ad3774e39931:8051 suite=df58e6911c0eb322b6bc:18098 refusals=e3b0c44298fc1c149afb:0 mutants=88f355cd6099e0ac438c:276",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-group-selection-copied.yaml",
        "ir=72ad0b125e143de9530d:11697 suite=91241babe78df0866e08:23205 refusals=e3b0c44298fc1c149afb:0 mutants=0f47b778529b1b1dddd5:390",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-group-selection-copied.yaml@ess22",
        "ir=72ad0b125e143de9530d:11697 suite=91241babe78df0866e08:23205 refusals=e3b0c44298fc1c149afb:0 mutants=0f47b778529b1b1dddd5:390",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-group-selection-guarded.yaml",
        "ir=93054a9c3b2ee689e056:13167 suite=562774d296bfd4361eee:50036 refusals=e3b0c44298fc1c149afb:0 mutants=2148e5bd54ecb35ca1fa:440",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-group-selection-guarded.yaml@ess22",
        "ir=93054a9c3b2ee689e056:13167 suite=562774d296bfd4361eee:50036 refusals=e3b0c44298fc1c149afb:0 mutants=2148e5bd54ecb35ca1fa:440",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-group-selection-owner.yaml",
        "ir=9f2295a02ee76fe82619:10500 suite=5fd8d9594ec175c13b37:12713 refusals=e3b0c44298fc1c149afb:0 mutants=76cafc7ed754805825af:318",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-group-selection-owner.yaml@ess22",
        "ir=9f2295a02ee76fe82619:10500 suite=5fd8d9594ec175c13b37:12713 refusals=e3b0c44298fc1c149afb:0 mutants=76cafc7ed754805825af:318",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-state-groups.yaml",
        "ir=65c5754ef78908b3e6cc:7836 suite=a0f338cdc7385303fdac:17194 refusals=e3b0c44298fc1c149afb:0 mutants=2753b13117fdc4a82e0b:185",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-state-groups.yaml@ess22",
        "ir=65c5754ef78908b3e6cc:7836 suite=a0f338cdc7385303fdac:17194 refusals=e3b0c44298fc1c149afb:0 mutants=2753b13117fdc4a82e0b:185",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-views.yaml",
        "ir=174462757e50a17f8cf0:16650 suite=78e10b8f9470b8a11038:52794 refusals=ad7aca5b20efcdf148f4:547 mutants=66e19517f4d2f9cde9bd:851",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-views.yaml@ess22",
        "ir=174462757e50a17f8cf0:16650 suite=78e10b8f9470b8a11038:52794 refusals=ad7aca5b20efcdf148f4:547 mutants=66e19517f4d2f9cde9bd:851",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/captured-identities.yaml",
        "ir=07dc89923f3a63619ad0:14165 suite=cf122e97922535f43e94:17432 refusals=2224aa4ae23d5583c0f0:495 mutants=0944ebe81c042fef840c:342",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/captured-identities.yaml@ess22",
        "ir=07dc89923f3a63619ad0:14165 suite=cf122e97922535f43e94:17432 refusals=2224aa4ae23d5583c0f0:495 mutants=0944ebe81c042fef840c:342",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/conditional-aggregate-measures.yaml",
        "ir=8dd5d71711712bd413e2:19229 suite=74b735b60e01ba871ba6:45677 refusals=e3b0c44298fc1c149afb:0 mutants=064ffcd5face2d60fc43:464",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/declared-behaviour.yaml",
        "ir=33c4473b60a55340621c:23496 suite=51d4f15d732ca4990af1:84063 refusals=e3b0c44298fc1c149afb:0 mutants=b3a1428916710853f8c3:2263",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/declared-behaviour.yaml@ess22",
        "ir=33c4473b60a55340621c:23496 suite=51d4f15d732ca4990af1:84063 refusals=e3b0c44298fc1c149afb:0 mutants=b3a1428916710853f8c3:2263",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/defined-over-optional-aggregates.yaml",
        "ir=87f2f3fecded4ab0adad:12829 suite=7dcf5451e1c6fa4e5f7f:41371 refusals=e3b0c44298fc1c149afb:0 mutants=faffdab4696bb8d31dad:326",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/defined-over-optional-aggregates.yaml@ess22",
        "ir=87f2f3fecded4ab0adad:12829 suite=7dcf5451e1c6fa4e5f7f:41371 refusals=e3b0c44298fc1c149afb:0 mutants=faffdab4696bb8d31dad:326",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/emit-swap-served.yaml",
        "ir=876f02f8abd5501c448c:19398 suite=caafacaf3508f707950d:33621 refusals=e3b0c44298fc1c149afb:0 mutants=3dd55ad47f8f9a8623ed:1553",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/emit-swap-served.yaml@ess22",
        "ir=876f02f8abd5501c448c:19398 suite=caafacaf3508f707950d:33621 refusals=e3b0c44298fc1c149afb:0 mutants=3dd55ad47f8f9a8623ed:1553",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/error-payload-sources.yaml",
        "ir=4015f3244356f73fd4c8:13624 suite=4506127770440fff1a53:17924 refusals=e3b0c44298fc1c149afb:0 mutants=3c65cc7e257bd813b072:977",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/error-payload-sources.yaml@ess22",
        "ir=4015f3244356f73fd4c8:13624 suite=4506127770440fff1a53:17924 refusals=e3b0c44298fc1c149afb:0 mutants=3c65cc7e257bd813b072:977",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-adv-order.yaml",
        "ir=3abef0b48a95b80ead64:10837 suite=df46ab5b3aa1157eb1d3:21792 refusals=e3b0c44298fc1c149afb:0 mutants=8769013d184be49d2363:974",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-adv-order.yaml@ess22",
        "ir=bd2bb62b724e04d58d77:10873 suite=b31e3dda7919be0fb2eb:18283 refusals=ab9ab2052eb7c32d9df7:500 mutants=8769013d184be49d2363:974",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-external-adversary.yaml",
        "ir=686e90bf6a1d9b0c069f:12561 suite=3febd88b2cb54a0db6a1:39390 refusals=e3b0c44298fc1c149afb:0 mutants=6283f6af25530302f6e8:588",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-external-adversary.yaml@ess22",
        "ir=03157448711036c39f53:12633 suite=ae8b3966b1fea2aaec23:27818 refusals=65afa85cc38eb759a3cb:1498 mutants=6283f6af25530302f6e8:588",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-external-exits.yaml",
        "ir=07a4759345deb9f1564f:11916 suite=110b5cca61360a40533a:18834 refusals=03843445ae95de8a2f8e:1169 mutants=bde81fbdcaf446ae8212:1442",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-external-exits.yaml@ess22",
        "ir=07a4759345deb9f1564f:11916 suite=110b5cca61360a40533a:18834 refusals=03843445ae95de8a2f8e:1169 mutants=bde81fbdcaf446ae8212:1442",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-external.yaml",
        "ir=6bea432cebf5f5cd90a1:12469 suite=4c73c1cd67fcd2fd10f4:29517 refusals=92126ffa688e233df54f:351 mutants=0998cfcaed5183e6d475:1424",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-external.yaml@ess22",
        "ir=751c3beb20f5a45e0eaa:12505 suite=f2a5f7b617e678103c60:21822 refusals=348686e1681154d62d51:1362 mutants=0998cfcaed5183e6d475:1424",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-guards-first.yaml",
        "ir=ac24b4784a0f6532e05a:16447 suite=fb7638dd5103a99c0e25:57256 refusals=e3b0c44298fc1c149afb:0 mutants=2706fb7ee6f1fa887c2d:2720",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-guards-first.yaml@ess22",
        "ir=93264aadd4574d845c23:16519 suite=42401573a2e88ad2e09e:60382 refusals=e3b0c44298fc1c149afb:0 mutants=2706fb7ee6f1fa887c2d:2720",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-retry/retry.yaml",
        "ir=80e1f84b6bd48152b70f:5712 suite=062041fe69fbd263c514:7329 refusals=e3b0c44298fc1c149afb:0 mutants=39dc4700e693f76427cb:190",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-retry/retry.yaml@ess22",
        "ir=80e1f84b6bd48152b70f:5712 suite=062041fe69fbd263c514:7329 refusals=e3b0c44298fc1c149afb:0 mutants=39dc4700e693f76427cb:190",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore.yaml",
        "ir=bb199ce2bf2c4709a7ac:24184 suite=dd9a46ef0015799d1d4e:114279 refusals=e3b0c44298fc1c149afb:0 mutants=64352614bbddcd918443:3661",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/input-guard-overlap.yaml",
        "ir=b1fa60bf70a77d433148:9921 suite=6869945457bdc09ca66c:25491 refusals=e3b0c44298fc1c149afb:0 mutants=5f707a054f5681a3ec87:578",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/input-guard-overlap.yaml@ess22",
        "ir=b1fa60bf70a77d433148:9921 suite=6869945457bdc09ca66c:25491 refusals=e3b0c44298fc1c149afb:0 mutants=5f707a054f5681a3ec87:578",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/mutation-survivor.yaml",
        "ir=b14305bb513764b3c344:30315 suite=500751245277d64bd27e:139284 refusals=e3b0c44298fc1c149afb:0 mutants=7d03493d632612aaf662:2102",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/occurrence-clock-claims.yaml",
        "ir=538865d7e0452497e18a:6968 suite=3901ea3654aa94d71e41:1665 refusals=b8c7e355f81df698f2a5:1946 mutants=dad70f76a7982e62b7e1:474",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/occurrence-clock-claims.yaml@ess22",
        "ir=538865d7e0452497e18a:6968 suite=3901ea3654aa94d71e41:1665 refusals=b8c7e355f81df698f2a5:1946 mutants=dad70f76a7982e62b7e1:474",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/optional-input-narrowed.yaml",
        "ir=e0025ea080b66f2bd4c4:8455 suite=e493984ab42c44829c43:4777 refusals=e3b0c44298fc1c149afb:0 mutants=34e221f5e3191d2e3dd7:336",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/optional-input-narrowed.yaml@ess22",
        "ir=e0025ea080b66f2bd4c4:8455 suite=e493984ab42c44829c43:4777 refusals=e3b0c44298fc1c149afb:0 mutants=34e221f5e3191d2e3dd7:336",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/owned-subject.yaml",
        "ir=20e2c40b8b612f0a6cad:9612 suite=7dcd3c52cb89dcde2dea:5439 refusals=e3b0c44298fc1c149afb:0 mutants=edf5ebe2d610f3194d00:323",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/owned-subject.yaml@ess22",
        "ir=20e2c40b8b612f0a6cad:9612 suite=7dcd3c52cb89dcde2dea:5439 refusals=e3b0c44298fc1c149afb:0 mutants=edf5ebe2d610f3194d00:323",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/refusal-beside-state.yaml",
        "ir=e2ab0a03303fd11f4809:15131 suite=5919188628af775ccd07:82947 refusals=e3b0c44298fc1c149afb:0 mutants=486d5c2e3463d7bb97d1:1249",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/refusal-beside-state.yaml@ess22",
        "ir=e2ab0a03303fd11f4809:15131 suite=5919188628af775ccd07:82947 refusals=e3b0c44298fc1c149afb:0 mutants=486d5c2e3463d7bb97d1:1249",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/register/register.yaml",
        "ir=756590f2274e1f57b1f1:13348 suite=d068c1f1d9c0f641a150:32901 refusals=9286ff749499808126f5:991 mutants=457e81ef5fbf2c22ad5b:691",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/register/register.yaml@ess22",
        "ir=756590f2274e1f57b1f1:13348 suite=d068c1f1d9c0f641a150:32901 refusals=9286ff749499808126f5:991 mutants=457e81ef5fbf2c22ad5b:691",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-copied-view-parameter.yaml",
        "ir=bc5cc79a54ed39b0428d:15775 suite=d5464390121dab5ffa00:43592 refusals=e3b0c44298fc1c149afb:0 mutants=85f8efe3714641739188:403",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-copied-view-parameter.yaml@ess22",
        "ir=bc5cc79a54ed39b0428d:15775 suite=d5464390121dab5ffa00:43592 refusals=e3b0c44298fc1c149afb:0 mutants=85f8efe3714641739188:403",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/restart.yaml",
        "ir=819f65d974cce32bc676:5751 suite=2a8aadba77578cd5ed44:2770 refusals=e3b0c44298fc1c149afb:0 mutants=73c3b2e7d1fb651f0aaa:230",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/retained-commit.yaml",
        "ir=ebec3c6459f94a0e7bdd:13371 suite=c9532c652864a0935a79:125449 refusals=e3b0c44298fc1c149afb:0 mutants=2fbb2a2069acd0c0d401:637",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/retained-commit.yaml@ess22",
        "ir=ebec3c6459f94a0e7bdd:13371 suite=c9532c652864a0935a79:125449 refusals=e3b0c44298fc1c149afb:0 mutants=2fbb2a2069acd0c0d401:637",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/retained-replay.yaml",
        "ir=efc7dbb71d6ef6cd0830:6978 suite=a93e280decfa407e130a:8401 refusals=e3b0c44298fc1c149afb:0 mutants=8b0c9b39c099c4608fb8:202",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/retained-replay.yaml@ess22",
        "ir=efc7dbb71d6ef6cd0830:6978 suite=a93e280decfa407e130a:8401 refusals=e3b0c44298fc1c149afb:0 mutants=8b0c9b39c099c4608fb8:202",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/state-scoped-refusals.yaml",
        "ir=8bbfabf4d1a3b7290f6d:13188 suite=dd8bec458ee376315ba2:67717 refusals=3d3441549461ff5efa99:2589 mutants=cdfdac3d9a68da469925:442",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/state-scoped-refusals.yaml@ess22",
        "ir=8bbfabf4d1a3b7290f6d:13188 suite=dd8bec458ee376315ba2:67717 refusals=3d3441549461ff5efa99:2589 mutants=cdfdac3d9a68da469925:442",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/subject-history.yaml",
        "ir=8c34bec89caf673881c5:12366 suite=7f7db8eb4d5970985f39:49742 refusals=e3b0c44298fc1c149afb:0 mutants=0f7649ac25fbe12975c8:1169",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/subject-history.yaml@ess22",
        "ir=6b4f0579ac228fa4f3b0:12474 suite=fd063aa39cc4809d1621:55067 refusals=e3b0c44298fc1c149afb:0 mutants=0f7649ac25fbe12975c8:1169",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/subject-state.yaml",
        "ir=583250f2bff528a0d5fa:11918 suite=02ec9f9351a35b423ecc:38170 refusals=e3b0c44298fc1c149afb:0 mutants=839839745600f41aeb25:1503",
    ),
    (
        "crates/verify/ess-conformance/tests/support_explore_optional_unknown/baseline.yaml",
        "ir=975874cebac75d7487fd:12573 suite=aa972eadd1174dab8fbb:25249 refusals=e3b0c44298fc1c149afb:0 mutants=fbfe2989eddc1626a03a:784",
    ),
    (
        "crates/verify/ess-conformance/tests/support_explore_optional_unknown/baseline.yaml@ess22",
        "ir=975874cebac75d7487fd:12573 suite=aa972eadd1174dab8fbb:25249 refusals=e3b0c44298fc1c149afb:0 mutants=fbfe2989eddc1626a03a:784",
    ),
    (
        "docs/design/view-grants.example.yaml",
        "ir=dd870a6cd8ddaf6941c9:6037 suite=487a465e45f1c31283a4:7846 refusals=e3b0c44298fc1c149afb:0 mutants=1f044b484ef4e3c30889:449",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/binding-arrangement.yaml",
        "ir=89aef022ceb3ea975291:18163 suite=e3dc8349a75207ae991f:74606 refusals=2d9c159e5cb844ca6a21:446 mutants=977c456c95b8b3e3b5cf:1205",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/binding-arrangement.yaml@ess22",
        "ir=89aef022ceb3ea975291:18163 suite=e3dc8349a75207ae991f:74606 refusals=2d9c159e5cb844ca6a21:446 mutants=977c456c95b8b3e3b5cf:1205",
    ),
];

#[ignore = "slow probe: `task test-slow-probes`"]
#[test]
fn adversary_244b_every_window_free_model_keeps_its_bytes_against_3b1684d1a() {
    let here = digests(&root());
    if let Ok(path) = std::env::var("ADV244B_WRITE") {
        let text: String = here
            .iter()
            .map(|(label, value)| format!("{label}\t{value}\n"))
            .collect();
        std::fs::write(path, text).unwrap();
        return;
    }
    for (label, _) in INTENDED {
        assert!(
            BASE.lines()
                .any(|line| line.split('\t').next() == Some(*label)),
            "intended move {label} names no pinned model"
        );
    }
    let mut moved = Vec::new();
    let mut compared = 0usize;
    let mut compiled = 0usize;
    for pinned in BASE.lines().filter(|line| !line.is_empty()) {
        let (label, base) = pinned.split_once('\t').expect("label, then digests");
        compared += 1;
        if base.starts_with("ir=") {
            compiled += 1;
        }
        match here.iter().find(|(name, _)| name == label) {
            Some((_, value)) if value == base => {}
            Some((_, value)) if INTENDED.contains(&(label, value.as_str())) => {}
            Some((_, value)) => moved.push(format!("{label}\n  base {base}\n  here {value}")),
            None => moved.push(format!("{label}: no longer found")),
        }
    }
    let new: Vec<&String> = here
        .iter()
        .map(|(label, _)| label)
        .filter(|label| {
            !BASE
                .lines()
                .any(|line| line.split('\t').next() == Some(label.as_str()))
        })
        .collect();
    eprintln!("compared {compared} (compiled {compiled}); only here: {new:?}");
    assert!(compared > 200, "{compared}");
    assert_eq!(moved, Vec::<String>::new(), "bytes moved against 3b1684d1a");
}
