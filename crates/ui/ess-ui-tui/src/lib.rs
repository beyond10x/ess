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
//! [`run`] is the entry point an `ess ui run --tui` command wraps.

mod app;
pub mod data;
mod expr;
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
}

impl fmt::Display for TuiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(error) => write!(formatter, "{error}"),
            Self::Refused(refusal) => write!(formatter, "{refusal}"),
            Self::Fixture(message) => write!(formatter, "fixtures: {message}"),
            Self::Io(message) | Self::Binding(message) => formatter.write_str(message),
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
        (Some(binding), _) => {
            let bases = http::base_urls(binding, &args.base_url)?;
            let authorization = match std::env::var(AUTHORIZATION_VAR) {
                Ok(value) if value.is_empty() => None,
                Ok(value) => Some(value),
                Err(std::env::VarError::NotPresent) => None,
                Err(std::env::VarError::NotUnicode(_)) => {
                    return Err(TuiError::Binding(format!(
                        "{AUTHORIZATION_VAR} is not UTF-8"
                    )))
                }
            };
            let adapter = HttpAdapter::new(binding.clone(), &bases, authorization)?;
            let text = std::fs::read_to_string(&args.path).map_err(|error| {
                TuiError::Io(format!("cannot read {}: {error}", args.path.display()))
            })?;
            let document = ess_ui::load_str(&text).map_err(TuiError::Load)?;
            App::bound(document, Box::new(adapter), binding.clone(), options)?
        }
    };
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
        drive(&mut app)
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
