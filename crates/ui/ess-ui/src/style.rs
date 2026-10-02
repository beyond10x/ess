//! The token table a theme is drawn with: the schema's built-in table, the document's `tokens:`
//! merged over it, and a theme's overrides merged over that.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::model::{Document, Tokens, TypeToken};
use crate::schema::Schema;

impl Tokens {
    /// The built-in table (`constructs.Tokens.builtins` in the schema): the values of the React
    /// stylesheet before tokens existed, so a document without `tokens:` looks as it did.
    pub fn builtin() -> &'static Tokens {
        static BUILTIN: OnceLock<Tokens> = OnceLock::new();
        BUILTIN.get_or_init(|| Schema::embedded().builtin_tokens())
    }

    /// `true` when no group names a token.
    pub fn is_empty(&self) -> bool {
        self.color.is_empty()
            && self.space.is_empty()
            && self.radius.is_empty()
            && self.typography.is_empty()
            && self.tone.is_empty()
    }

    /// `over` merged over this table, group by group and name by name: an entry of `over`
    /// replaces the entry of its name whole.
    #[must_use]
    pub fn merged(&self, over: &Tokens) -> Tokens {
        fn merge<V: Clone>(
            base: &BTreeMap<String, V>,
            over: &BTreeMap<String, V>,
        ) -> BTreeMap<String, V> {
            let mut merged = base.clone();
            merged.extend(
                over.iter()
                    .map(|(name, value)| (name.clone(), value.clone())),
            );
            merged
        }
        Tokens {
            color: merge(&self.color, &over.color),
            space: merge(&self.space, &over.space),
            radius: merge(&self.radius, &over.radius),
            typography: merge(&self.typography, &over.typography),
            tone: merge(&self.tone, &over.tone),
        }
    }

    /// This table with every `type` entry complete where it can be: a field an entry leaves out
    /// falls back to this table's `body`, then to the built-in `body`.
    #[must_use]
    pub fn filled(&self) -> Tokens {
        let empty = TypeToken::default();
        let body = self.typography.get("body").unwrap_or(&empty);
        let builtin = Tokens::builtin().typography.get("body").unwrap_or(&empty);
        let mut filled = self.clone();
        for entry in filled.typography.values_mut() {
            entry.family = entry
                .family
                .take()
                .or_else(|| body.family.clone())
                .or_else(|| builtin.family.clone());
            entry.size = entry
                .size
                .take()
                .or_else(|| body.size.clone())
                .or_else(|| builtin.size.clone());
            entry.weight = entry
                .weight
                .take()
                .or_else(|| body.weight.clone())
                .or_else(|| builtin.weight.clone());
        }
        filled
    }
}

impl Document {
    /// The built-in table with this document's `tokens:` merged over it, `type` entries filled:
    /// what a theme without overrides shows.
    pub fn base_tokens(&self) -> Tokens {
        Tokens::builtin().merged(&self.tokens).filled()
    }

    /// The full table of the theme named `theme`: the built-in table, `tokens:`, then the
    /// theme's own overrides, `type` entries filled. `None` when no theme has that name.
    pub fn theme_tokens(&self, theme: &str) -> Option<Tokens> {
        let overrides = self.themes.get(theme)?;
        Some(
            Tokens::builtin()
                .merged(&self.tokens)
                .merged(overrides)
                .filled(),
        )
    }
}
