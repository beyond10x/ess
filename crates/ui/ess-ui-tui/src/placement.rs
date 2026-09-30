//! Where each piece of UI state lives in the terminal.
//!
//! Every state of the document resolves to a store by the schema's
//! `PlacementProfile.resolution` order, against the profile tables the schema itself declares.
//! The terminal then keeps each store in the place that plays its part:
//!
//! | store | in the terminal |
//! |---|---|
//! | `memory` | in-process |
//! | `url` | the TUI's own location (`/page?key=value`) |
//! | `session_storage` | a file under the state directory, under `<app>/<user_id>/<account_id>/`, emptied at start |
//! | `local_storage` | a file under the state directory, under `<app>/<user_id>/<account_id>/` |
//! | `server`, `server_session` | the data adapter |
//!
//! A sensitive state never reaches a file or the location, whatever it resolves to.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ess_ui::{Document, Located, NodePath, NodeRef, Profile, State, StateClass, Store};
use serde_yaml::{Mapping, Value};

use crate::data::DataAdapter;
use crate::profile::Refusal;

/// Where one state lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement {
    /// The resolved store.
    pub store: Store,
    /// Never written to a file or the location.
    pub sensitive: bool,
    /// The declared initial value.
    pub default: Option<Value>,
}

/// Resolves every state of `document` to a store, refusing as the schema's refusals say.
pub fn resolve(document: &Document) -> Result<BTreeMap<String, Placement>, Refusal> {
    let mut placements = BTreeMap::new();
    for Located { path, node } in document.nodes() {
        let NodeRef::State(state) = node else {
            continue;
        };
        let store = store_for(document, &path, state)?;
        let refuse = |message: String| Refusal {
            path: path.clone(),
            message,
        };
        if state.sensitive
            && matches!(
                store,
                Store::Url | Store::SessionStorage | Store::LocalStorage
            )
        {
            return Err(refuse(format!(
                "a sensitive state cannot live in {}",
                store_name(&store)
            )));
        }
        if state.class == StateClass::Credential
            && !matches!(store, Store::Memory | Store::ServerSession)
        {
            return Err(refuse(format!(
                "a credential lives in memory or server_session, not {}",
                store_name(&store)
            )));
        }
        placements.insert(
            path.to_string(),
            Placement {
                store,
                sensitive: state.sensitive,
                default: state.default.clone(),
            },
        );
    }
    Ok(placements)
}

fn store_for(document: &Document, path: &NodePath, state: &State) -> Result<Store, Refusal> {
    if let Some(store) = &state.store {
        return Ok(store.clone());
    }
    let segments = path.segments();
    let page = (segments.first().map(String::as_str) == Some("pages"))
        .then(|| segments.get(1).and_then(|name| document.pages.get(name)))
        .flatten();
    let section = page.and_then(|page| {
        (segments.get(2).map(String::as_str) == Some("sections"))
            .then(|| {
                segments
                    .get(3)
                    .and_then(|name| page.sections.iter().find(|section| &section.name == name))
            })
            .flatten()
    });
    let profiled = |profile: Option<&Profile>| match profile {
        Some(Profile::Thin) => profile_default("thin", &state.class),
        Some(Profile::Fat) => profile_default("fat", &state.class),
        _ => None,
    };
    if let Some(store) = profiled(section.and_then(|section| section.profile.as_ref())) {
        return Ok(store);
    }
    if let Some(store) = profiled(page.and_then(|page| page.profile.as_ref())) {
        return Ok(store);
    }
    if let Some(store) = document.placement_defaults.get(&state.class) {
        return Ok(store.clone());
    }
    let profile = match &document.placement_profile {
        ess_ui::PlacementProfile::Thin => "thin",
        ess_ui::PlacementProfile::Fat => "fat",
        _ => "hybrid",
    };
    profile_default(profile, &state.class).ok_or_else(|| Refusal {
        path: path.clone(),
        message: format!(
            "no store resolves for this state: it names none, and neither its section, its page, \
             `placement_defaults` nor the `{profile}` profile places its class"
        ),
    })
}

/// The store a class resolves to at document level: `placement_defaults`, else the document
/// profile's table.
pub fn class_default(document: &Document, class: &StateClass) -> Option<Store> {
    if let Some(store) = document.placement_defaults.get(class) {
        return Some(store.clone());
    }
    let profile = match &document.placement_profile {
        ess_ui::PlacementProfile::Thin => "thin",
        ess_ui::PlacementProfile::Fat => "fat",
        _ => "hybrid",
    };
    profile_default(profile, class)
}

/// `constructs.PlacementProfile.profiles.<profile>.defaults[<class>]`, read from the schema.
fn profile_default(profile: &str, class: &StateClass) -> Option<Store> {
    static TABLES: OnceLock<BTreeMap<String, BTreeMap<StateClass, Store>>> = OnceLock::new();
    let tables = TABLES.get_or_init(|| {
        let schema: Value = serde_yaml::from_str(ess_ui::SCHEMA).expect("the schema is YAML");
        let mut tables = BTreeMap::new();
        if let Some(profiles) = schema["constructs"]["PlacementProfile"]["profiles"].as_mapping() {
            for (name, entry) in profiles {
                if let (Some(name), Ok(defaults)) = (
                    name.as_str(),
                    serde_yaml::from_value::<BTreeMap<StateClass, Store>>(
                        entry["defaults"].clone(),
                    ),
                ) {
                    tables.insert(name.to_owned(), defaults);
                }
            }
        }
        tables
    });
    tables
        .get(profile)
        .and_then(|table| table.get(class))
        .cloned()
}

/// A store's schema name.
pub fn store_name(store: &Store) -> &'static str {
    match store {
        Store::Memory => "memory",
        Store::Url => "url",
        Store::SessionStorage => "session_storage",
        Store::LocalStorage => "local_storage",
        Store::ServerSession => "server_session",
        Store::Server => "server",
        Store::Unmapped(_) => "unmapped",
    }
}

/// The values of every state, kept where its placement says.
#[derive(Debug)]
pub struct StateStore {
    placements: BTreeMap<String, Placement>,
    memory: BTreeMap<String, Value>,
    location: BTreeMap<String, Value>,
    root: PathBuf,
    app: String,
    key: (Option<String>, Option<String>),
    dir: PathBuf,
}

impl StateStore {
    /// A store for `placements`, keeping files under `<state_dir>/<app>/<user_id>/<account_id>/`,
    /// the schema's storage key `[origin, actor.user_id, actor.account_id]` with the app as the
    /// origin. Each segment is encoded by [`segment`], so no two keys share a directory. Session
    /// storage begins empty, as a new browser session would.
    pub fn new(
        placements: BTreeMap<String, Placement>,
        state_dir: &Path,
        app: &str,
        user_id: Option<&str>,
        account_id: Option<&str>,
    ) -> Self {
        let mut store = Self {
            placements,
            memory: BTreeMap::new(),
            location: BTreeMap::new(),
            root: state_dir.to_path_buf(),
            app: app.to_owned(),
            key: (None, None),
            dir: PathBuf::new(),
        };
        store.rekey(user_id, account_id);
        store
    }

    /// The actor ids the storage directory is keyed by.
    pub fn key(&self) -> (Option<&str>, Option<&str>) {
        (self.key.0.as_deref(), self.key.1.as_deref())
    }

    /// Keys storage by another actor. The new actor's session storage begins empty, and memory
    /// is cleared (`clear_on` defaults to `[signout, account_switch]`); its local storage is
    /// whatever that actor left.
    pub fn rekey(&mut self, user_id: Option<&str>, account_id: Option<&str>) {
        self.key = (user_id.map(str::to_owned), account_id.map(str::to_owned));
        self.dir = self
            .root
            .join(segment(Some(&self.app)))
            .join(segment(user_id))
            .join(segment(account_id));
        self.memory.clear();
        let _ = std::fs::remove_file(self.dir.join("session_storage.yaml"));
    }

    /// Ends the session: memory, the location and session storage (on disk too) are cleared,
    /// and every sensitive or `server_session` value held by the adapter is removed. Local
    /// storage and `server` state survive, as the schema's store matrix says.
    pub fn sign_out(&mut self, adapter: &mut dyn DataAdapter) {
        self.memory.clear();
        self.location.clear();
        let _ = std::fs::remove_file(self.dir.join("session_storage.yaml"));
        for (path, placement) in &self.placements {
            if placement.sensitive || placement.store == Store::ServerSession {
                adapter.store_state(path, Value::Null);
            }
        }
    }

    /// The placement of a declared state.
    pub fn placement(&self, path: &str) -> Option<&Placement> {
        self.placements.get(path)
    }

    /// The store a state lives in; an undeclared one lives in memory.
    pub fn store_of(&self, path: &str) -> Store {
        self.placements
            .get(path)
            .map_or(Store::Memory, |placement| placement.store.clone())
    }

    /// The value of a state, or its default.
    pub fn get(&self, path: &str, adapter: &dyn DataAdapter) -> Value {
        let placement = self.placements.get(path);
        let stored = match placement {
            Some(placement) if !placement.sensitive => match placement.store {
                Store::Url => self.location.get(path).cloned(),
                Store::SessionStorage | Store::LocalStorage => {
                    self.file(&placement.store).get(path).cloned()
                }
                Store::Server | Store::ServerSession => adapter.load_state(path),
                _ => self.memory.get(path).cloned(),
            },
            _ => self.memory.get(path).cloned(),
        };
        stored
            .or_else(|| placement.and_then(|placement| placement.default.clone()))
            .unwrap_or(Value::Null)
    }

    /// Writes a state where its placement says.
    pub fn set(&mut self, path: &str, value: Value, adapter: &mut dyn DataAdapter) {
        let placement = self.placements.get(path).cloned();
        match placement {
            Some(placement) if !placement.sensitive => match placement.store {
                Store::Url => {
                    self.location.insert(path.to_owned(), value);
                }
                Store::SessionStorage | Store::LocalStorage => {
                    let mut file = self.file(&placement.store);
                    file.insert(Value::String(path.to_owned()), value);
                    self.write_file(&placement.store, &file);
                }
                Store::Server | Store::ServerSession => adapter.store_state(path, value),
                _ => {
                    self.memory.insert(path.to_owned(), value);
                }
            },
            _ => {
                self.memory.insert(path.to_owned(), value);
            }
        }
    }

    /// The location query: every url-placed state under `prefix` with a value other than its
    /// default, by its last path segment.
    pub fn query(&self, prefix: &str) -> Vec<(String, Value)> {
        let mut query = Vec::new();
        for (path, value) in &self.location {
            let Some(name) = path
                .strip_prefix(prefix)
                .and_then(|rest| rest.strip_prefix("/state/"))
            else {
                continue;
            };
            let default = self
                .placements
                .get(path)
                .and_then(|placement| placement.default.clone());
            let empty = matches!(value, Value::Null)
                || matches!(value, Value::String(text) if text.is_empty())
                || matches!(value, Value::Sequence(items) if items.is_empty());
            if !empty && default.as_ref() != Some(value) {
                query.push((name.to_owned(), value.clone()));
            }
        }
        query
    }

    fn file_path(&self, store: &Store) -> PathBuf {
        self.dir.join(format!("{}.yaml", store_name(store)))
    }

    fn file(&self, store: &Store) -> Mapping {
        std::fs::read_to_string(self.file_path(store))
            .ok()
            .and_then(|text| serde_yaml::from_str(&text).ok())
            .unwrap_or_default()
    }

    fn write_file(&self, store: &Store, content: &Mapping) {
        if std::fs::create_dir_all(&self.dir).is_ok() {
            if let Ok(text) = serde_yaml::to_string(content) {
                let _ = std::fs::write(self.file_path(store), text);
            }
        }
    }
}

/// One directory segment of a storage key, never empty (an empty segment would add no
/// directory): ASCII letters, digits, `-` and `_` as they are, every other byte as `%XX`, an
/// empty value as a lone `%` and an absent value as `=`. No encoded value is `%` or contains
/// `=`, so absent, empty and every non-empty value each have their own segment.
pub fn segment(value: Option<&str>) -> String {
    let Some(value) = value else {
        return "=".to_owned();
    };
    if value.is_empty() {
        return "%".to_owned();
    }
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' {
            encoded.push(char::from(byte));
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::segment;

    #[test]
    fn storage_key_segments_are_unambiguous() {
        assert_eq!(segment(Some("us-01")), "us-01");
        assert_eq!(segment(None), "=");
        assert_eq!(segment(Some("")), "%");
        assert_ne!(segment(Some("")), segment(None));
        assert_eq!(segment(Some("../x")), "%2E%2E%2Fx");
        assert_ne!(segment(Some("a/b")), segment(Some("a%2Fb")));
    }
}
