// generated from gatepass v1
// model digest f8ccea748a49e127ca2e18f725481394cc0eab1787fafd77d16c52485bf2abba
// contract digest a6fdd92f3a88ac0abbe59789406f3001df466e87f222e4aad1a8348c17f91d7c
// do not edit: regenerate with `ess synthesize`

//! The transport-free entry point's refusals.
//!
//! Each served component's `handle` runs a command or view by its qualified name, through the
//! same code the HTTP routes run. What it cannot answer with a declared outcome it answers with a
//! [`Refused`]: the same refusal a route answers, without the status, which [`Refused::status`]
//! still names for a caller that serves it.

/// Why `handle` answered no declared outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// The surface declares no command and no view by this qualified name.
    Unknown(String),
    /// The input is not the command's declared input; the route answers this `400`.
    Input(String),
    /// The realization is unfinished; the route answers this `501`, which the contract declares.
    ///
    /// Either a port reported an unmet obligation, or the command's effect was committed and
    /// delivering what it published failed (the detail then begins `delivering what the command
    /// published`). Not to be retried: after a failed delivery a retry performs the command twice.
    Unmet(String),
}

impl Refused {
    /// The status the HTTP surface answers this refusal with.
    pub fn status(&self) -> u16 {
        match self {
            Self::Unknown(_) => 404,
            Self::Input(_) => 400,
            Self::Unmet(_) => 501,
        }
    }
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(name) => write!(
                f,
                "`{name}` is not a command or view this surface declares"
            ),
            Self::Input(detail) | Self::Unmet(detail) => f.write_str(detail),
        }
    }
}

impl std::error::Error for Refused {}

/// A rendered outcome, read back as the value it renders.
///
/// The body is written by this crate's own encoders, so it is JSON by construction; reading it
/// back, rather than building the value a second way, keeps one rendering of every outcome.
pub(crate) fn read(body: &str) -> crate::json::Value {
    crate::json::parse(body).expect("this crate's encoders write JSON its reader reads")
}
