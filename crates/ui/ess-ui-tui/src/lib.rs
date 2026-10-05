//! A terminal renderer for `ess-ui/1` documents.
//!
//! Any document loaded by [`ess_ui`] runs as a ratatui application that answers every read from
//! the document's fixtures and plays each channel's fixture event script on a virtual clock.
//!
//! - The shell: navigation pane (`g` + a section's initial, or the fuzzy palette `:`), page
//!   outlet, overlays as full-screen panes (`esc` closes), a notifications line and the
//!   live-channel status segment in the top bar.
//! - Pages keep their `layout` intent where the terminal can: columns and areas degrade to a
//!   stack in section order, as the schema's `PageLayout.degrades` declares.
//! - Every composite and primitive is drawn; capabilities the terminal lacks are declared in
//!   [`profile::TUI`] and resolved against each node's `degrades`, refusing the document with
//!   the node's path when a needed fallback is missing ([`profile::check`]).
//! - State lives where its placement says ([`placement`]).
//! - Bound to a served surface ([`App::bound`], `--model`), reads and commands go to the paths
//!   the binding names through [`HttpAdapter`], no fixture channel plays (a `live:` section polls
//!   its read instead), and a refused command shows where the user acted: on the open form, on
//!   the confirm, or beside the action row, keeping the draft.
//!
//! [`run`] is the entry point an `ess ui run --tui` command wraps; [`generate`] writes the crate
//! `ess generate ui --target tui` emits, whose `main` calls [`run_embedded`].

mod app;
pub mod data;
mod expr;
pub mod generate;
pub mod http;
pub mod keys;
pub mod live;
pub mod placement;
pub mod profile;
mod view;

use std::fmt;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

pub use app::{App, Lifecycle, Options, Region};
pub use data::{DataAdapter, FixtureAdapter, ReadRequest, ReadResult};
pub use http::HttpAdapter;
pub use profile::{Plan, Refusal, RendererProfile, TUI};

/// The environment variable the `Authorization` header of a bound run is read from: an opaque
/// value, as configured, never taken from the command line and never derived.
pub const AUTHORIZATION_VAR: &str = "ESS_UI_AUTHORIZATION";

/// Why a document does not run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiError {
    /// The document did not load.
    Load(ess_ui::LoadError),
    /// The document loaded but the terminal refuses it.
    Refused(Refusal),
    /// A fixture file is missing or malformed.
    Fixture(String),
    /// The terminal or a file failed.
    Io(String),
    /// A bound run is not set up: a base URL, the credential, or a `--model` without a binding.
    Binding(String),
    /// A `--screen-once` frame was printed, but a read of the page it shows failed: each failed
    /// read as `(view, error)`.
    ReadFailed(Vec<(String, String)>),
}

/// The exit status of a generated app whose `--screen-once` frame was printed while a read of
/// its page failed ([`TuiError::ReadFailed`]); every other refusal exits `1`, a usage error `2`.
pub const READ_FAILED_EXIT: u8 = 3;

impl TuiError {
    /// The exit status a generated app ends with on this error: [`READ_FAILED_EXIT`] for
    /// [`TuiError::ReadFailed`], else `1`.
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::ReadFailed(_) => READ_FAILED_EXIT,
            _ => 1,
        }
    }
}

impl fmt::Display for TuiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(error) => write!(formatter, "{error}"),
            Self::Refused(refusal) => write!(formatter, "{refusal}"),
            Self::Fixture(message) => write!(formatter, "fixtures: {message}"),
            Self::Io(message) | Self::Binding(message) => formatter.write_str(message),
            Self::ReadFailed(reads) => {
                formatter.write_str("--screen-once: the page's read failed: ")?;
                for (at, (view, error)) in reads.iter().enumerate() {
                    if at > 0 {
                        formatter.write_str("; ")?;
                    }
                    let error: String = error
                        .chars()
                        .map(|character| {
                            if character.is_control() {
                                ' '
                            } else {
                                character
                            }
                        })
                        .collect();
                    write!(formatter, "{view}: {error}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for TuiError {}

/// The command line of `ess ui run --tui`.
#[derive(Debug, Clone, PartialEq, Eq, clap::Args)]
pub struct TuiArgs {
    /// The `ess-ui/1` document to run.
    #[arg(long)]
    pub path: PathBuf,
    /// A fixture directory replacing the one the document names.
    #[arg(long, conflicts_with = "model")]
    pub fixtures: Option<PathBuf>,
    /// The ESS specification the document's `model:` names (a directory, its `ess-inputs.yaml`,
    /// or one file): reads and commands go to the HTTP surface its `reached_by: network`
    /// components serve instead of the fixtures, and no fixture channel plays. The
    /// `Authorization` header is read from `ESS_UI_AUTHORIZATION`, never from the command line.
    #[arg(long)]
    pub model: Option<PathBuf>,
    /// Where a served component is reached, `http://` only: `<url>` when the document binds one
    /// component, else `<component>=<url>`, once per component.
    #[arg(long = "base-url", value_name = "URL", requires = "model")]
    pub base_url: Vec<String>,
}

/// Where file-placed state is kept: `$XDG_STATE_HOME/ess-ui-tui`, else
/// `$HOME/.local/state/ess-ui-tui`.
pub fn state_dir() -> PathBuf {
    let base = std::env::var_os("XDG_STATE_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("state"))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("ess-ui-tui")
}

/// Runs a document in the terminal until the user quits (`q` or ctrl-c).
///
/// `binding` is the route table `--model` resolves to (`ess_ui_check::binding`), which the caller
/// computes: this crate reads a binding and never compiles a model. With one, the run is bound
/// ([`App::bound`]) to the base URLs of `--base-url` ([`http::base_urls`]), the `Authorization`
/// header from [`AUTHORIZATION_VAR`]. Every refusal comes before the terminal is touched.
pub fn run(args: &TuiArgs, binding: Option<&ess_ui::binding::Binding>) -> Result<(), TuiError> {
    let mut options = Options::new(state_dir());
    options.fixtures.clone_from(&args.fixtures);
    let mut app = match (binding, &args.model) {
        (None, Some(model)) => {
            return Err(TuiError::Binding(format!(
                "--model {}: no binding was computed for it",
                model.display()
            )))
        }
        (None, None) => App::from_path(&args.path, options)?,
        (Some(binding), _) => bound_app(binding, &args.base_url, options, || {
            let text = std::fs::read_to_string(&args.path).map_err(|error| {
                TuiError::Io(format!("cannot read {}: {error}", args.path.display()))
            })?;
            ess_ui::load_str_with(&text, binding).map_err(TuiError::Load)
        })?,
    };
    interactive(&mut app)
}

/// What a crate `ess generate ui --target tui` generated compiles in: its `ess-ui/1` document
/// and the route table [`generate`] computed for it, as JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Embedded {
    /// The document's text.
    pub document: &'static str,
    /// The [`ess_ui::binding::Binding`] the document was generated with, as JSON.
    pub binding: &'static str,
}

/// The size of the one frame `--screen-once` prints: `<width>x<height>`, each side digits only,
/// from 1 to [`ScreenSize::MAX`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenSize {
    /// Columns.
    pub width: u16,
    /// Rows.
    pub height: u16,
}

impl ScreenSize {
    /// The largest side: a frame is drawn into memory, so its size is bounded.
    pub const MAX: u16 = 1000;
}

impl std::str::FromStr for ScreenSize {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let refused = || {
            format!(
                "`{value}` is not <width>x<height>, such as 120x40, each side from 1 to {}",
                Self::MAX
            )
        };
        let (width, height) = value.split_once('x').ok_or_else(refused)?;
        let side = |text: &str| {
            Some(text)
                .filter(|text| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()))
                .and_then(|text| text.parse::<u16>().ok())
                .filter(|side| (1..=Self::MAX).contains(side))
        };
        match (side(width), side(height)) {
            (Some(width), Some(height)) => Ok(Self { width, height }),
            _ => Err(refused()),
        }
    }
}

/// Runs a generated terminal app: its embedded document, bound to the served surface its
/// binding describes at the `--base-url` values given ([`http::base_urls`]), the
/// `Authorization` header from [`AUTHORIZATION_VAR`].
///
/// With `screen_once`, no terminal is touched: the app opens its home page, reads it once, and
/// prints one rendered frame of that size to stdout, one line per row with trailing blanks
/// trimmed. When a read of that page failed, the frame is still printed and the run ends with
/// [`TuiError::ReadFailed`] naming each failed read ([`READ_FAILED_EXIT`]). Otherwise it runs in
/// the terminal until the user quits, as [`run`] does.
pub fn run_embedded(
    embedded: &Embedded,
    base_url: &[String],
    screen_once: Option<ScreenSize>,
) -> Result<(), TuiError> {
    let binding: ess_ui::binding::Binding =
        serde_json::from_str(embedded.binding).map_err(|error| {
            TuiError::Binding(format!("the embedded binding does not read: {error}"))
        })?;
    let mut app = bound_app(&binding, base_url, Options::new(state_dir()), || {
        ess_ui::load_str_with(embedded.document, &binding).map_err(TuiError::Load)
    })?;
    let Some(size) = screen_once else {
        return interactive(&mut app);
    };
    app.advance(Duration::ZERO);
    let frame = app.render_text(size.width, size.height);
    let mut stdout = io::stdout().lock();
    io::Write::write_all(&mut stdout, frame.as_bytes())
        .and_then(|()| io::Write::flush(&mut stdout))
        .map_err(|error| TuiError::Io(error.to_string()))?;
    let failed = app.failed_reads();
    if failed.is_empty() {
        Ok(())
    } else {
        Err(TuiError::ReadFailed(failed))
    }
}

/// The `Authorization` header of a bound run, from [`AUTHORIZATION_VAR`]: none when unset or
/// empty.
fn authorization() -> Result<Option<String>, TuiError> {
    match std::env::var(AUTHORIZATION_VAR) {
        Ok(value) if value.is_empty() => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(TuiError::Binding(format!(
            "{AUTHORIZATION_VAR} is not UTF-8"
        ))),
    }
}

/// The app bound to `binding` at the `--base-url` values given; the document is loaded after
/// the base URLs and the credential are accepted.
fn bound_app(
    binding: &ess_ui::binding::Binding,
    base_url: &[String],
    options: Options,
    document: impl FnOnce() -> Result<ess_ui::Document, TuiError>,
) -> Result<App, TuiError> {
    let bases = http::base_urls(binding, base_url)?;
    let adapter = HttpAdapter::new(binding.clone(), &bases, authorization()?)?;
    App::bound(document()?, Box::new(adapter), binding.clone(), options)
}

/// Runs `app` in the terminal until the user quits, restoring the terminal on every exit.
fn interactive(app: &mut App) -> Result<(), TuiError> {
    let io = |error: io::Error| TuiError::Io(error.to_string());
    // Restores the terminal on every exit: a returned error, a normal quit (the guard's drop)
    // and a panic (the hook restores before the panic message prints, so it is readable).
    // A hook cannot be replaced while a thread is panicking, so the installed hook only acts
    // while the terminal is ours: once restored it delegates to the previous hook, which is
    // also reinstalled on every exit that does not unwind.
    let previous = std::sync::Arc::new(std::panic::take_hook());
    let chained = std::sync::Arc::clone(&previous);
    std::panic::set_hook(Box::new(move |info| {
        if TERMINAL_OURS.load(Ordering::SeqCst) {
            restore_terminal();
        }
        chained(info);
    }));
    let result = (|| {
        let _guard = Restore;
        TERMINAL_OURS.store(true, Ordering::SeqCst);
        enable_raw_mode().map_err(io)?;
        io::stdout().execute(EnterAlternateScreen).map_err(io)?;
        drive(app)
    })();
    let _ = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| previous(info)));
    result
}

/// Whether the terminal is in raw mode on the alternate screen for [`run`].
static TERMINAL_OURS: AtomicBool = AtomicBool::new(false);

/// Restores the terminal when dropped.
struct Restore;

impl Drop for Restore {
    fn drop(&mut self) {
        restore_terminal();
    }
}

/// Leaves the alternate screen and raw mode, once.
fn restore_terminal() {
    if TERMINAL_OURS.swap(false, Ordering::SeqCst) {
        let _ = io::stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

/// Whether `key` is the press of ctrl-c, which quits whatever else is going on.
fn is_quit(key: crossterm::event::KeyEvent) -> bool {
    key.kind == KeyEventKind::Press
        && key.code == crossterm::event::KeyCode::Char('c')
        && key
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
}

fn drive(app: &mut App) -> Result<(), TuiError> {
    let io = |error: io::Error| TuiError::Io(error.to_string());
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout())).map_err(io)?;
    let mut last = Instant::now();
    app.advance(Duration::ZERO);
    while !app.should_quit() {
        terminal.draw(|frame| app.draw(frame)).map_err(io)?;
        if event::poll(Duration::from_millis(100)).map_err(io)? {
            if let Event::Key(key) = event::read().map_err(io)? {
                if key.kind == KeyEventKind::Press {
                    let sent = app.commands_sent;
                    app.key(key);
                    // One command in flight per place: in a bound run, what was typed while a
                    // command waited for its answer is dropped, so a second submit, confirm or
                    // action is not sent. Ctrl-c is never dropped: it quits once the answer (or
                    // the timeout) has come.
                    if app.bound.is_some() && app.commands_sent != sent {
                        while event::poll(Duration::ZERO).map_err(io)? {
                            if let Event::Key(key) = event::read().map_err(io)? {
                                if is_quit(key) {
                                    app.key(key);
                                }
                            }
                        }
                    }
                }
            }
        }
        let now = Instant::now();
        app.advance(now - last);
        last = now;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    use super::ScreenSize;

    /// `--screen-once` takes digits only on each side, from 1 to 1000.
    #[test]
    fn a_screen_size_is_bounded_and_unsigned() {
        let size = |width, height| Ok(ScreenSize { width, height });
        assert_eq!("120x40".parse(), size(120, 40));
        assert_eq!("1000x1000".parse(), size(1000, 1000));
        for refused in [
            "+120x40", "120x+40", "1001x1", "1x1001", "0x5", "12x", " 1x1", "1X1",
        ] {
            assert!(refused.parse::<ScreenSize>().is_err(), "{refused}");
        }
    }

    /// The keys typed while a bound command waits are dropped, all but ctrl-c.
    #[test]
    fn only_a_ctrl_c_press_survives_the_drain() {
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(super::is_quit(ctrl_c));
        let mut released = ctrl_c;
        released.kind = KeyEventKind::Release;
        for kept in [
            released,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        ] {
            assert!(!super::is_quit(kept), "{kept:?}");
        }
    }
}
