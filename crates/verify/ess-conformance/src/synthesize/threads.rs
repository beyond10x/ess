//! What crosses a thread when caller readings run elsewhere.
//!
//! A synthesis keeps work counters, settings and memories in `thread_local!` statics. A reading
//! run on another thread starts with every one of them empty. [`STATE`] names each `static` of the
//! crate with how a worker handles it; [`Context`] carries what a reading reads from its thread to
//! the worker; [`Work`] reads every work counter on one thread and adds it to another's. See
//! `docs/design/parallel-caller-readings.md`.

use std::collections::BTreeSet;

use ess_compiler::ir::EssIr;
use ess_domain::command::precedence::{phase_order, with_phase_order, Phase};
use ess_domain::name::QualifiedName;

use super::{created_memo, earlier_runs, first_kept, grant, related_guard, row_set, run_memo};
use super::{subject_fact, DroppedGrant, DroppedWrite};
use super::{AddedGrant, Synthesis};

/// How a static of this crate is handled where caller readings run on another thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(test)]
pub(super) enum Handling {
    /// Counted per thread; a worker's count is added to the calling thread's on join ([`Work`]).
    Counter,
    /// Set around a whole synthesis and read inside readings; captured where the readings are
    /// planned and set on each worker ([`Context`]).
    Context,
    /// Opened and closed inside one reading; empty wherever readings are planned (checked by
    /// [`Context::capture`] in debug builds).
    Scope,
    /// A memory that answers what the computation answers; a worker starts with it empty.
    Memory,
    /// A uniform model's memory kept across its readings; owned by the one thread that runs them.
    Lane,
    /// Read or set only on the calling thread, outside every reading.
    Caller,
    /// Read only by the crate's own unit tests, which run with one job.
    Test,
    /// A plain `static`: immutable, `Sync` by the compiler.
    Shared,
    /// In a module no synthesis runs (the process target, the web host).
    Unreached,
}

/// Every `static` declared in `crates/verify/ess-conformance/src`: the path under `src/`, the
/// name, and its handling. Sorted by path, then name. A row is keyed by both: `KEPT`, `MADE`,
/// `DROPPED` and `ASKED` each name statics of two files.
///
/// A tuple counter is one row. Three hold a flag that is context rather than a count
/// (`first_kept.rs` `WORK`, `related_guard.rs` `COMPANION_WORK`, `row_set.rs` `ATTEMPTS`): their
/// rows say `Counter`, and [`Context`] carries their flags.
#[cfg(test)]
pub(super) const STATE: &[(&str, &str, Handling)] = &[
    ("admission.rs", "SUITES_PARSED", Handling::Counter),
    ("authored.rs", "DOCUMENTS_READ", Handling::Counter),
    ("authored/ties.rs", "REPLAYS", Handling::Counter),
    ("fixtures.rs", "FIXTURE_STEP_CLONES", Handling::Counter),
    ("fixtures.rs", "PRECONDITION_STEP_CLONES", Handling::Counter),
    ("input.rs", "SEGMENTS_COPIED", Handling::Counter),
    ("mutate.rs", "GRANTS_KEPT", Handling::Caller),
    ("now_offset.rs", "ANALYSED", Handling::Memory),
    ("now_offset.rs", "ANALYSES", Handling::Counter),
    ("process/exact.rs", "VISITED", Handling::Unreached),
    ("process/tests.rs", "SUITE", Handling::Shared),
    ("scenario.rs", "FEATURE_SCANS", Handling::Counter),
    ("scenario.rs", "SERIALIZED", Handling::Counter),
    ("synthesize.rs", "BINDING_TRIGGERS_TRIED", Handling::Counter),
    ("synthesize.rs", "BUILT_VIEW_INDEXES", Handling::Counter),
    ("synthesize.rs", "COMMANDS_TAKEN", Handling::Counter),
    ("synthesize.rs", "DERIVED_EDGES", Handling::Counter),
    ("synthesize.rs", "DROPPED", Handling::Context),
    ("synthesize.rs", "JOBS", Handling::Caller),
    ("synthesize.rs", "NOTHING", Handling::Shared),
    ("synthesize.rs", "OFF_THREAD", Handling::Caller),
    ("synthesize.rs", "OVERLAP_ANSWERS", Handling::Memory),
    ("synthesize.rs", "OVERLAP_SEARCHES", Handling::Counter),
    ("synthesize.rs", "PRECEDENCE_REPLAYS", Handling::Counter),
    ("synthesize.rs", "SCOPE", Handling::Context),
    ("synthesize.rs", "SYNTHESES", Handling::Counter),
    ("synthesize/aggregate.rs", "LITERALS", Handling::Counter),
    (
        "synthesize/aggregate/contrast.rs",
        "DECIDED",
        Handling::Counter,
    ),
    ("synthesize/caller.rs", "UNFOCUSED", Handling::Test),
    ("synthesize/caller.rs", "WHOLE", Handling::Test),
    ("synthesize/caller/readings.rs", "PANIC_IN", Handling::Test),
    ("synthesize/created_memo.rs", "ASKED", Handling::Counter),
    ("synthesize/created_memo.rs", "KEPT", Handling::Memory),
    ("synthesize/created_memo.rs", "MADE", Handling::Counter),
    ("synthesize/created_memo.rs", "UNKEPT", Handling::Context),
    (
        "synthesize/disclosure.rs",
        "ORIGINS_BUILT",
        Handling::Counter,
    ),
    ("synthesize/earlier_runs.rs", "EARLIER", Handling::Scope),
    ("synthesize/first_kept.rs", "WORK", Handling::Counter),
    ("synthesize/grant.rs", "ADDED", Handling::Context),
    ("synthesize/grant.rs", "DROPPED", Handling::Context),
    ("synthesize/made.rs", "KEPT", Handling::Lane),
    ("synthesize/made.rs", "MADE", Handling::Memory),
    ("synthesize/made.rs", "MAKES", Handling::Counter),
    ("synthesize/made.rs", "PICKS", Handling::Counter),
    ("synthesize/made.rs", "RELATED_PICKS", Handling::Counter),
    (
        "synthesize/model_memo.rs",
        "ABSENT_BUILT",
        Handling::Counter,
    ),
    ("synthesize/model_memo.rs", "KEPT", Handling::Memory),
    (
        "synthesize/ordered_head.rs",
        "STEERING_WORK",
        Handling::Counter,
    ),
    (
        "synthesize/related_guard.rs",
        "BESIDE_OPEN",
        Handling::Scope,
    ),
    (
        "synthesize/related_guard.rs",
        "COMPANION_WORK",
        Handling::Counter,
    ),
    ("synthesize/related_guard.rs", "OWN_OPEN", Handling::Scope),
    (
        "synthesize/related_guard.rs",
        "OWN_OVERLAY_WORK",
        Handling::Counter,
    ),
    ("synthesize/related_guard.rs", "OWN_WORK", Handling::Counter),
    ("synthesize/row_set.rs", "ATTEMPTS", Handling::Counter),
    ("synthesize/row_set.rs", "TRIED", Handling::Counter),
    ("synthesize/run_memo.rs", "ASKED", Handling::Counter),
    ("synthesize/run_memo.rs", "KEPT", Handling::Memory),
    ("synthesize/run_memo.rs", "MADE", Handling::Counter),
    ("synthesize/run_memo.rs", "UNKEPT", Handling::Context),
    ("synthesize/seeds.rs", "DIGESTED", Handling::Memory),
    ("synthesize/seeds.rs", "EMPTY", Handling::Shared),
    ("synthesize/shorthand.rs", "TRAVERSALS", Handling::Counter),
    (
        "synthesize/subject_fact.rs",
        "FIELDS_BOUND",
        Handling::Counter,
    ),
    ("synthesize/subject_fact.rs", "HELD", Handling::Scope),
    (
        "synthesize/subject_fact.rs",
        "INPUTS_MADE",
        Handling::Counter,
    ),
    ("synthesize/subject_fact.rs", "ROWS", Handling::Memory),
    ("web_execution/host.rs", "HOST", Handling::Unreached),
    ("witness.rs", "BUILT", Handling::Counter),
    ("witness.rs", "CLONED", Handling::Counter),
    ("witness.rs", "REFERENCE", Handling::Scope),
    ("witness.rs", "STEPS", Handling::Counter),
    ("witness.rs", "UNDER_TEST", Handling::Scope),
    ("witness_memo.rs", "GENERATIONS", Handling::Test),
    ("witness_memo.rs", "KEPT", Handling::Lane),
    ("witness_memo.rs", "MEMO", Handling::Memory),
    ("witness_memo.rs", "SHARED", Handling::Memory),
    ("witness_memo.rs", "WORK", Handling::Test),
];

/// What a caller reading reads from the thread it runs on, as the planning thread holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // five independent switches, each read by one setter
pub(super) struct Context {
    scope: Option<BTreeSet<QualifiedName>>,
    dropped_write: Option<DroppedWrite>,
    dropped_grant: Option<DroppedGrant>,
    added_grant: Option<AddedGrant>,
    phase_order: [Phase; 8],
    created_unkept: bool,
    runs_unkept: bool,
    first_fresh: bool,
    attempts_fresh: bool,
    companion_fresh: bool,
}

impl Context {
    /// The context of this thread. Called where the readings are planned, which is outside every
    /// reading's scope.
    pub(super) fn capture() -> Self {
        debug_assert!(
            !crate::witness::under_test_open()
                && !earlier_runs::open()
                && !related_guard::beside_or_own_open()
                && !subject_fact::holding_any(),
            "caller readings are planned outside every reading's scope"
        );
        Self {
            scope: super::scope(),
            dropped_write: super::dropped_write(),
            dropped_grant: grant::dropped_grant(),
            added_grant: grant::added_grant(),
            phase_order: phase_order(),
            created_unkept: created_memo::is_unkept(),
            runs_unkept: run_memo::is_unkept(),
            first_fresh: first_kept::fresh(),
            attempts_fresh: row_set::attempts_fresh(),
            companion_fresh: related_guard::companion_fresh(),
        }
    }

    /// Runs `run` with this context set on the current thread, and restores what was there
    /// before, also on a panic.
    pub(super) fn enter<R>(&self, run: impl FnOnce() -> R) -> R {
        with_phase_order(self.phase_order, || {
            super::with_dropped_write(self.dropped_write.clone(), || {
                super::with_dropped_grant(self.dropped_grant.clone(), || {
                    super::with_added_grant(self.added_grant.clone(), || {
                        let _scope = self.scope.clone().map(super::scoped_to);
                        let run: Box<dyn FnOnce() -> R + '_> = if self.created_unkept {
                            Box::new(|| created_memo::unkept(run))
                        } else {
                            Box::new(run)
                        };
                        let run: Box<dyn FnOnce() -> R + '_> = if self.runs_unkept {
                            Box::new(|| run_memo::unkept(run))
                        } else {
                            run
                        };
                        first_kept::first_rows_made(self.first_fresh, || {
                            super::row_set_attempts(self.attempts_fresh, || {
                                super::companion_work(self.companion_fresh, run).0
                            })
                            .0
                        })
                        .0
                    })
                })
            })
        })
    }
}

/// Declares [`Work`]: one field per count of a Counter row of [`STATE`], read by [`Work::taken`]
/// and compared by [`Work::since`].
macro_rules! work {
    ($($field:ident : $ty:ty = $taken:expr;)*) => {
        /// Every work counter of this crate on one thread, in [`STATE`] order. A tuple counter
        /// has one field per element.
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        pub(super) struct Work {
            $(pub(super) $field: $ty,)*
        }

        impl Work {
            /// The current thread's values.
            pub(super) fn taken() -> Self {
                Self { $($field: $taken,)* }
            }

            /// The field-wise difference from `earlier`; the most open at once
            /// (`own_deepest`) is the later value.
            #[cfg(test)]
            pub(super) fn since(&self, earlier: &Self) -> Self {
                let mut out = Self { $($field: self.$field.saturating_sub(earlier.$field),)* };
                out.own_deepest = self.own_deepest;
                out
            }
        }
    };
}

work! {
    suites_parsed: u64 = crate::admission::taken_suites_parsed();
    documents_read: u64 = crate::authored::taken_documents_read();
    replays: u64 = crate::authored::taken_replays();
    fixture_step_clones: u64 = crate::fixtures::taken_fixture_step_clones();
    precondition_step_clones: u64 = crate::fixtures::taken_precondition_step_clones();
    segments_copied: u64 = crate::input::taken_segments_copied();
    analyses: u64 = crate::now_offset::taken_analyses();
    feature_scans: u64 = crate::scenario::taken_feature_scans();
    serialized: u64 = crate::scenario::taken_serialized();
    binding_triggers_tried: u64 = super::BINDING_TRIGGERS_TRIED.with(std::cell::Cell::get);
    built_view_indexes: u64 = super::BUILT_VIEW_INDEXES.with(std::cell::Cell::get);
    commands_taken: u64 = super::COMMANDS_TAKEN.with(std::cell::Cell::get);
    derived_edges: u64 = super::DERIVED_EDGES.with(std::cell::Cell::get);
    overlap_searches: u64 = super::OVERLAP_SEARCHES.with(std::cell::Cell::get);
    precedence_replays: u64 = super::PRECEDENCE_REPLAYS.with(std::cell::Cell::get);
    syntheses: u64 = super::SYNTHESES.with(std::cell::Cell::get);
    literals: u64 = super::aggregate::taken_literals();
    decided: u64 = super::aggregate::taken_decided();
    created_asked: u64 = created_memo::taken_asked();
    created_made: u64 = created_memo::taken_made();
    origins_built: u64 = super::disclosure::taken_origins_built();
    first_asked: u64 = first_kept::taken_work().0;
    first_made: u64 = first_kept::taken_work().1;
    makes: u64 = super::made::taken_makes();
    picks: u64 = super::made::taken_picks();
    related_picks: u64 = super::made::taken_related_picks();
    absent_built: u64 = super::model_memo::taken_absent_built();
    steering_maps: u64 = super::ordered_head::taken_steering_work().0;
    steering_checks: u64 = super::ordered_head::taken_steering_work().1;
    companion_asked: u64 = related_guard::taken_companion_work().0;
    companion_searched: u64 = related_guard::taken_companion_work().1;
    own_overlay_built: u64 = related_guard::taken_own_overlay_work().0;
    own_overlay_asked_at_the_bound: u64 = related_guard::taken_own_overlay_work().1;
    own_arranged: u64 = related_guard::taken_own_work().0;
    own_strict: u64 = related_guard::taken_own_work().1;
    own_deepest: usize = related_guard::taken_own_work().2;
    attempts: u64 = row_set::taken_attempts();
    tried: u64 = row_set::taken_tried();
    run_asked: u64 = run_memo::taken_asked();
    run_made: u64 = run_memo::taken_made();
    traversals: u64 = super::shorthand::taken_traversals();
    fields_bound: u64 = subject_fact::taken_fields_bound();
    inputs_asked: u64 = subject_fact::taken_inputs_made().0;
    inputs_made: u64 = subject_fact::taken_inputs_made().1;
    inputs_built: usize = crate::witness::taken_built();
    builders_cloned: usize = crate::witness::taken_cloned();
    directed_steps: usize = crate::witness::taken_steps();
}

impl Work {
    /// The counters, as `"<path>:<STATIC>"`, in [`STATE`] order.
    #[cfg(test)]
    pub(super) const NAMES: &[&str] = &[
        "admission.rs:SUITES_PARSED",
        "authored.rs:DOCUMENTS_READ",
        "authored/ties.rs:REPLAYS",
        "fixtures.rs:FIXTURE_STEP_CLONES",
        "fixtures.rs:PRECONDITION_STEP_CLONES",
        "input.rs:SEGMENTS_COPIED",
        "now_offset.rs:ANALYSES",
        "scenario.rs:FEATURE_SCANS",
        "scenario.rs:SERIALIZED",
        "synthesize.rs:BINDING_TRIGGERS_TRIED",
        "synthesize.rs:BUILT_VIEW_INDEXES",
        "synthesize.rs:COMMANDS_TAKEN",
        "synthesize.rs:DERIVED_EDGES",
        "synthesize.rs:OVERLAP_SEARCHES",
        "synthesize.rs:PRECEDENCE_REPLAYS",
        "synthesize.rs:SYNTHESES",
        "synthesize/aggregate.rs:LITERALS",
        "synthesize/aggregate/contrast.rs:DECIDED",
        "synthesize/created_memo.rs:ASKED",
        "synthesize/created_memo.rs:MADE",
        "synthesize/disclosure.rs:ORIGINS_BUILT",
        "synthesize/first_kept.rs:WORK",
        "synthesize/made.rs:MAKES",
        "synthesize/made.rs:PICKS",
        "synthesize/made.rs:RELATED_PICKS",
        "synthesize/model_memo.rs:ABSENT_BUILT",
        "synthesize/ordered_head.rs:STEERING_WORK",
        "synthesize/related_guard.rs:COMPANION_WORK",
        "synthesize/related_guard.rs:OWN_OVERLAY_WORK",
        "synthesize/related_guard.rs:OWN_WORK",
        "synthesize/row_set.rs:ATTEMPTS",
        "synthesize/row_set.rs:TRIED",
        "synthesize/run_memo.rs:ASKED",
        "synthesize/run_memo.rs:MADE",
        "synthesize/shorthand.rs:TRAVERSALS",
        "synthesize/subject_fact.rs:FIELDS_BOUND",
        "synthesize/subject_fact.rs:INPUTS_MADE",
        "witness.rs:BUILT",
        "witness.rs:CLONED",
        "witness.rs:STEPS",
    ];

    /// Adds these counts to the current thread's: sums, except the most open at once
    /// (`own_deepest`), which is the larger.
    pub(super) fn add(&self) {
        crate::admission::add_suites_parsed(self.suites_parsed);
        crate::authored::add_documents_read(self.documents_read);
        crate::authored::add_replays(self.replays);
        crate::fixtures::add_fixture_step_clones(self.fixture_step_clones);
        crate::fixtures::add_precondition_step_clones(self.precondition_step_clones);
        crate::input::add_segments_copied(self.segments_copied);
        crate::now_offset::add_analyses(self.analyses);
        crate::scenario::add_feature_scans(self.feature_scans);
        crate::scenario::add_serialized(self.serialized);
        bump(&super::BINDING_TRIGGERS_TRIED, self.binding_triggers_tried);
        bump(&super::BUILT_VIEW_INDEXES, self.built_view_indexes);
        bump(&super::COMMANDS_TAKEN, self.commands_taken);
        bump(&super::DERIVED_EDGES, self.derived_edges);
        bump(&super::OVERLAP_SEARCHES, self.overlap_searches);
        bump(&super::PRECEDENCE_REPLAYS, self.precedence_replays);
        bump(&super::SYNTHESES, self.syntheses);
        super::aggregate::add_literals(self.literals);
        super::aggregate::add_decided(self.decided);
        created_memo::add_asked(self.created_asked);
        created_memo::add_made(self.created_made);
        super::disclosure::add_origins_built(self.origins_built);
        first_kept::add_work((self.first_asked, self.first_made));
        super::made::add_makes(self.makes);
        super::made::add_picks(self.picks);
        super::made::add_related_picks(self.related_picks);
        super::model_memo::add_absent_built(self.absent_built);
        super::ordered_head::add_steering_work((self.steering_maps, self.steering_checks));
        related_guard::add_companion_work((self.companion_asked, self.companion_searched));
        related_guard::add_own_overlay_work((
            self.own_overlay_built,
            self.own_overlay_asked_at_the_bound,
        ));
        related_guard::add_own_work((self.own_arranged, self.own_strict, self.own_deepest));
        row_set::add_attempts(self.attempts);
        row_set::add_tried(self.tried);
        run_memo::add_asked(self.run_asked);
        run_memo::add_made(self.run_made);
        super::shorthand::add_traversals(self.traversals);
        subject_fact::add_fields_bound(self.fields_bound);
        subject_fact::add_inputs_made((self.inputs_asked, self.inputs_made));
        crate::witness::add_built(self.inputs_built);
        crate::witness::add_cloned(self.builders_cloned);
        crate::witness::add_steps(self.directed_steps);
    }
}

/// Adds `more` to a counter declared in `synthesize.rs`.
fn bump(counter: &'static std::thread::LocalKey<std::cell::Cell<u64>>, more: u64) {
    counter.with(|it| it.set(it.get() + more));
}

const _: () = {
    const fn send<T: Send>() {}
    const fn sync<T: Sync>() {}
    sync::<EssIr>();
    send::<EssIr>();
    sync::<super::AdmittedSeeds>();
    send::<Synthesis>();
    sync::<Context>();
};

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::Path;

    use ess_compiler::ir::EssIr;
    use ess_compiler::resolve::compile;
    use ess_compiler::source::SourceMap;
    use ess_domain::command::precedence::{with_phase_order, Phase};
    use ess_domain::name::QualifiedName;
    use ess_domain::spec::{RawSpecFile, Specification};
    use ess_domain::system::Source;

    use super::super::{
        created_memo, creations_made, earlier_runs, first_rows_made, scoped_to, with_added_grant,
        with_dropped_grant, with_dropped_write, AddedGrant, DroppedGrant, DroppedWrite,
    };
    use super::{Context, Handling, Work, STATE};

    /// Every `.rs` file under `dir`, as its path under `base` with `/` and its text, in byte
    /// order of the path.
    fn sources(dir: &Path, base: &Path, out: &mut Vec<(String, String)>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
            .map(|entry| entry.expect("a directory entry").path())
            .collect();
        entries.sort_by(|a, b| {
            a.to_string_lossy()
                .as_bytes()
                .cmp(b.to_string_lossy().as_bytes())
        });
        for path in entries {
            if path.is_dir() {
                sources(&path, base, out);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let relative = path
                    .strip_prefix(base)
                    .expect("under the base")
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                out.push((relative, text));
            }
        }
    }

    /// The names of the statics a line declares: `static ` followed by an upper-case identifier
    /// and `:`, outside a comment and not the `'static` lifetime.
    fn statics_in(line: &str) -> Vec<String> {
        if line.trim_start().starts_with("//") {
            return Vec::new();
        }
        let mut found = Vec::new();
        let mut rest = line;
        while let Some(at) = rest.find("static ") {
            let before = &rest[..at];
            let after = &rest[at + "static ".len()..];
            rest = after;
            if before.ends_with('\'')
                || before
                    .chars()
                    .last()
                    .is_some_and(|last| last.is_alphanumeric() || last == '_')
            {
                continue;
            }
            let name: String = after
                .chars()
                .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
                .collect();
            if name.starts_with(|c: char| c.is_ascii_uppercase())
                && after[name.len()..].trim_start().starts_with(':')
            {
                found.push(name);
            }
        }
        found
    }

    #[test]
    fn every_static_of_the_crate_is_classified() {
        let base = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let mut files = Vec::new();
        sources(base, base, &mut files);
        let mut declared = BTreeSet::new();
        for (path, text) in &files {
            for line in text.lines() {
                for name in statics_in(line) {
                    declared.insert((path.clone(), name));
                }
            }
        }
        let classified: BTreeSet<(String, String)> = STATE
            .iter()
            .map(|(path, name, _)| ((*path).to_owned(), (*name).to_owned()))
            .collect();
        let missing: Vec<_> = declared.difference(&classified).collect();
        let extra: Vec<_> = classified.difference(&declared).collect();
        assert!(
            missing.is_empty() && extra.is_empty(),
            "the statics of the crate and `STATE` differ\nmissing from `STATE`:\n{}\nin `STATE` and not declared:\n{}\nclassify it in `STATE` (`src/synthesize/threads.rs`) and, for a Counter, add it to `Work`",
            missing
                .iter()
                .map(|(path, name)| format!("  src/{path}: {name}"))
                .collect::<Vec<_>>()
                .join("\n"),
            extra
                .iter()
                .map(|(path, name)| format!("  src/{path}: {name}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let keys: Vec<_> = STATE.iter().map(|(path, name, _)| (*path, *name)).collect();
        assert!(
            keys.windows(2).all(|pair| pair[0] < pair[1]),
            "`STATE` is sorted by path, then name, with no row twice"
        );
    }

    #[test]
    fn work_names_every_counter_row() {
        let counters: Vec<String> = STATE
            .iter()
            .filter(|(_, _, handling)| *handling == Handling::Counter)
            .map(|(path, name, _)| format!("{path}:{name}"))
            .collect();
        assert_eq!(Work::NAMES, counters.as_slice());
    }

    fn compiled(text: &str) -> EssIr {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
        let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("{errors}"));
        compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
    }

    #[test]
    fn a_worker_sees_the_context_of_the_planning_thread() {
        let command = "shop.order.Place".to_owned();
        let actor = "shop.order.Clerk".to_owned();
        let write = DroppedWrite {
            command: command.clone(),
            outcome: "placed".to_owned(),
            target: "note".to_owned(),
            source: "note".to_owned(),
        };
        let mut exchanged = Phase::PRECEDENCE;
        exchanged.swap(0, 1);
        let context = with_dropped_write(Some(write), || {
            with_dropped_grant(
                Some(DroppedGrant {
                    actor: actor.clone(),
                    command: command.clone(),
                }),
                || {
                    with_added_grant(
                        Some(AddedGrant {
                            actor: actor.clone(),
                            command: command.clone(),
                        }),
                        || {
                            let _scope = scoped_to(BTreeSet::from([
                                QualifiedName::new(&command).expect("a name")
                            ]));
                            created_memo::unkept(|| {
                                first_rows_made(true, || {
                                    with_phase_order(exchanged, Context::capture)
                                })
                                .0
                            })
                        },
                    )
                },
            )
        });
        let untouched = Context::capture();
        assert_ne!(
            context, untouched,
            "the planning thread's context is not the default"
        );
        assert!(context.scope.is_some() && context.created_unkept && context.first_fresh);
        assert_eq!(context.phase_order, exchanged);
        let (inside, outside) = std::thread::scope(|s| {
            s.spawn(|| (context.enter(Context::capture), Context::capture()))
                .join()
                .expect("the worker ends")
        });
        assert_eq!(
            inside, context,
            "inside `enter` a worker holds the planning thread's context"
        );
        assert_eq!(outside, untouched, "outside `enter` it holds the default");
        assert_eq!(untouched.phase_order, Phase::PRECEDENCE);
    }

    #[test]
    fn a_workers_counts_are_added_to_the_planning_thread() {
        let worker = std::thread::scope(|s| {
            s.spawn(|| {
                let ir = compiled(include_str!(
                    "../../tests/fixtures/caller-struct-identity.yaml"
                ));
                let _ = super::super::synthesize(&ir);
                Work::taken()
            })
            .join()
            .expect("the worker ends")
        });
        assert_ne!(worker, Work::default(), "the synthesis counted something");
        let before = Work::taken();
        worker.add();
        assert_eq!(Work::taken().since(&before), worker);
        assert_eq!(creations_made(|| worker.add()).1, worker.makes);
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "caller readings are planned outside every reading's scope")]
    fn capturing_inside_a_reading_scope_is_refused_in_debug() {
        let _open = earlier_runs::scenario();
        let _ = Context::capture();
    }
}
