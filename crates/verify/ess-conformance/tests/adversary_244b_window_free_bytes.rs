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
const INTENDED: &[(&str, &str)] = &[
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-input-refusal.yaml",
        "ir=8f73acbe936097824f8e:9715 suite=d10530fe00b5eb7bfc37:32088 refusals=e3b0c44298fc1c149afb:0 mutants=9b0ca26989288b2d82cb:1856",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-input-refusal.yaml@ess22",
        "ir=8f73acbe936097824f8e:9715 suite=d10530fe00b5eb7bfc37:32088 refusals=e3b0c44298fc1c149afb:0 mutants=9b0ca26989288b2d82cb:1856",
    ),
    (
        "crates/specify/ess-domain/tests/fixtures/text-length-positions.yaml",
        "ir=64397198ca8b3f008517:12944 suite=1fc3b8f3846fac6fbb2b:323 refusals=b364fba60dba0462e7c1:334 mutants=898f900fbd76ae62a1bb:658",
    ),
    (
        "crates/specify/ess-domain/tests/fixtures/text-length-positions.yaml@ess22",
        "ir=64397198ca8b3f008517:12944 suite=1fc3b8f3846fac6fbb2b:323 refusals=b364fba60dba0462e7c1:334 mutants=898f900fbd76ae62a1bb:658",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-stored-guarded-move.yaml",
        "ir=5500cec8d73ec1066754:12118 suite=c9591218350b81be905c:41344 refusals=e3b0c44298fc1c149afb:0 mutants=fc6d5af3ab402a5a71c6:901",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-stored-guarded-move.yaml@ess22",
        "ir=5500cec8d73ec1066754:12118 suite=c9591218350b81be905c:41344 refusals=e3b0c44298fc1c149afb:0 mutants=fc6d5af3ab402a5a71c6:901",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-paid-route.yaml",
        "ir=8078b5131c26dfc079fd:13470 suite=1cf6db0f3e0a10bbebd7:57258 refusals=e3b0c44298fc1c149afb:0 mutants=f8ab55352caeafcc1558:1206",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-paid-route.yaml@ess22",
        "ir=8078b5131c26dfc079fd:13470 suite=1cf6db0f3e0a10bbebd7:57258 refusals=e3b0c44298fc1c149afb:0 mutants=f8ab55352caeafcc1558:1206",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-two-creators.yaml",
        "ir=4a198eb2e776e280388f:12951 suite=835f415354c704786882:50441 refusals=e3b0c44298fc1c149afb:0 mutants=141c10daad914ab05627:1210",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/arrangement-two-creators.yaml@ess22",
        "ir=4a198eb2e776e280388f:12951 suite=835f415354c704786882:50441 refusals=e3b0c44298fc1c149afb:0 mutants=141c10daad914ab05627:1210",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-stored-rows.yaml",
        "ir=3ade52cbdd7f200803c5:38581 suite=0415c4076059d44644d8:196402 refusals=e3b0c44298fc1c149afb:0 mutants=7da604504e62e6d0c5dc:3555",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/explore-stored-rows.yaml@ess22",
        "ir=3ade52cbdd7f200803c5:38581 suite=0415c4076059d44644d8:196402 refusals=e3b0c44298fc1c149afb:0 mutants=7da604504e62e6d0c5dc:3555",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-copied-value.yaml",
        "ir=2f88b3a80c3a08f99c5e:12758 suite=c9885fcc7d121c6cd204:16434 refusals=e3b0c44298fc1c149afb:0 mutants=ec1d182e4932ee05c224:476",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-copied-value.yaml@ess22",
        "ir=2f88b3a80c3a08f99c5e:12758 suite=c9885fcc7d121c6cd204:16434 refusals=e3b0c44298fc1c149afb:0 mutants=ec1d182e4932ee05c224:476",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-multiple.yaml",
        "ir=ecf9fba0c35c8f05f6f6:23733 suite=40c28675f65f47ca763a:89794 refusals=dc01d36d68b12c2cacb2:1043 mutants=3ef47f665a160f847bdc:2757",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-optional.yaml",
        "ir=ad19424ea5327eca6573:16818 suite=0345d2d3f2d6c5fed768:57439 refusals=d30825fef61960483efb:550 mutants=7712e3612fb39c49500e:2038",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-owner-link.yaml",
        "ir=838035241c8956c47216:18375 suite=8321029dd2307525da7b:47863 refusals=e3b0c44298fc1c149afb:0 mutants=c5c8a6851a640e3b28ca:854",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-owner-link.yaml@ess22",
        "ir=838035241c8956c47216:18375 suite=8321029dd2307525da7b:47863 refusals=e3b0c44298fc1c149afb:0 mutants=c5c8a6851a640e3b28ca:854",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-release.yaml",
        "ir=f94e97df357fa758fd87:15482 suite=13d157c274d5d2cef8d9:43973 refusals=66eb987124e594b9fa5f:1219 mutants=32f97ac94a7d324118f2:1104",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/related-guard-release.yaml@ess22",
        "ir=f94e97df357fa758fd87:15482 suite=13d157c274d5d2cef8d9:43973 refusals=66eb987124e594b9fa5f:1219 mutants=32f97ac94a7d324118f2:1104",
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
        "ir=4991ac392e7b2fe208cf:10985 suite=d2806897f3d0cd08f3d6:35833 refusals=e3b0c44298fc1c149afb:0 mutants=75e051220592a5b9c31f:1097",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/state-in-subject-predicate.yaml",
        "ir=93ad22bc62ea70588000:11608 suite=db49e2aec3c2f0f40957:36777 refusals=e3b0c44298fc1c149afb:0 mutants=9b686dfbaa8859f1a79a:1151",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/state-in-subject-predicate.yaml@ess22",
        "ir=93ad22bc62ea70588000:11608 suite=db49e2aec3c2f0f40957:36777 refusals=e3b0c44298fc1c149afb:0 mutants=9b686dfbaa8859f1a79a:1151",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/stored-field-guards.yaml",
        "ir=f5690ad7914f4165a715:8364 suite=e55baa7c2c4be0a2989c:26862 refusals=06a644cde2b8fc51e318:546 mutants=a60b1be30f50bcf9e065:932",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/stored-field-guards.yaml@ess22",
        "ir=f5690ad7914f4165a715:8364 suite=e55baa7c2c4be0a2989c:26862 refusals=06a644cde2b8fc51e318:546 mutants=a60b1be30f50bcf9e065:932",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/subject-guard-copied-field.yaml",
        "ir=bbf12fce16b887c3b60d:20824 suite=2705e065bf055f8f707c:124591 refusals=e3b0c44298fc1c149afb:0 mutants=d2aee002c3d5342df8af:2425",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/subject-guard-copied-field.yaml@ess22",
        "ir=bbf12fce16b887c3b60d:20824 suite=2705e065bf055f8f707c:124591 refusals=e3b0c44298fc1c149afb:0 mutants=d2aee002c3d5342df8af:2425",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/subject-guard-input.yaml",
        "ir=aa83bc892312d4854d25:7016 suite=b7c14452cfcd4040e342:18848 refusals=9abbe950081eeb90978c:496 mutants=16c6a166a4d9b002aa32:429",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/subject-guard-input.yaml@ess22",
        "ir=aa83bc892312d4854d25:7016 suite=b7c14452cfcd4040e342:18848 refusals=9abbe950081eeb90978c:496 mutants=16c6a166a4d9b002aa32:429",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/when-subject-eventual.yaml",
        "ir=5781561e33e4cb327676:11017 suite=dd22493eb4d33d9bc62b:28607 refusals=2a80cca606aa08bb095f:1009 mutants=b20a6db7d368a06c594e:575",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/when-subject-eventual.yaml@ess22",
        "ir=5781561e33e4cb327676:11017 suite=dd22493eb4d33d9bc62b:28607 refusals=2a80cca606aa08bb095f:1009 mutants=b20a6db7d368a06c594e:575",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/when-subject-fallback-refusal.yaml",
        "ir=f59eafbb12aebf53ff70:11025 suite=f7ba7d533df0abd1de62:29867 refusals=2a80cca606aa08bb095f:1009 mutants=b20a6db7d368a06c594e:575",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/when-subject-fallback-refusal.yaml@ess22",
        "ir=f59eafbb12aebf53ff70:11025 suite=f7ba7d533df0abd1de62:29867 refusals=2a80cca606aa08bb095f:1009 mutants=b20a6db7d368a06c594e:575",
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
        "ir=d35e9f1aea844db65444:20928 suite=fda0d69e641a88ad0183:21999 refusals=b3c848d48cfb9f249ead:518 mutants=c826069de550b57df70e:1061",
    ),
    (
        "crates/generate/ess-synth/tests/fixtures/generated-views.yaml",
        "ir=d684139b5be5288707c0:29577 suite=d51cd175899fabc318ca:58807 refusals=bfb251ea7183b9cfb016:399 mutants=8eea32c81ed7d126b3d2:996",
    ),
    (
        "crates/generate/ess-synth/tests/fixtures/generated-views.yaml@ess22",
        "ir=d684139b5be5288707c0:29577 suite=d51cd175899fabc318ca:58807 refusals=bfb251ea7183b9cfb016:399 mutants=8eea32c81ed7d126b3d2:996",
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
        "ir=dde087b272d309d3fd18:28813 suite=4c5e94e707b5da248c17:142947 refusals=e3b0c44298fc1c149afb:0 mutants=1e14eff476478e9ca097:425",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-group-selection.yaml@ess22",
        "ir=dde087b272d309d3fd18:28813 suite=4c5e94e707b5da248c17:142947 refusals=e3b0c44298fc1c149afb:0 mutants=1e14eff476478e9ca097:425",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-optional-fields.yaml",
        "ir=b5c5844d95d664bb1f49:17481 suite=0b8961c5dae080a0863a:33980 refusals=e3b0c44298fc1c149afb:0 mutants=227583c6431f7c06dfd9:95",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/aggregate-optional-fields.yaml@ess22",
        "ir=b5c5844d95d664bb1f49:17481 suite=0b8961c5dae080a0863a:33980 refusals=e3b0c44298fc1c149afb:0 mutants=227583c6431f7c06dfd9:95",
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
        "ir=fad93168fa11c42db418:30720 suite=7548062c85b64cffd671:140160 refusals=b0abbc30c3882c8c5037:423 mutants=bc6271e9f57ec341f933:3988",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/occurrence-clock-offers.yaml",
        "ir=100a4d422734bdd51af9:14053 suite=2381a42504a5cc0ab314:28272 refusals=4fd558db28f2b7a105b0:2473 mutants=fad5957bb2e5091b3d81:1764",
    ),
    (
        "crates/verify/ess-conformance/tests/fixtures/occurrence-clock-offers.yaml@ess22",
        "ir=100a4d422734bdd51af9:14053 suite=2381a42504a5cc0ab314:28272 refusals=4fd558db28f2b7a105b0:2473 mutants=fad5957bb2e5091b3d81:1764",
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
        "ir=ab75682e9e4308374158:20260 suite=a35d1b135cca44237fc3:28283 refusals=e3b0c44298fc1c149afb:0 mutants=090c711ebd31d81488c5:334",
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
