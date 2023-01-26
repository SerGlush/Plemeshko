use serde::Deserialize;

use crate::state::components::ComponentId;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct TextId(pub(super) String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FatTextId(pub ComponentId, pub TextId);

#[derive(Default)]
pub struct TextIdFactory {
    prefix: String,

    /// Occurs when the `prefix` extension is required but can't be created.
    locked: bool,
}

pub trait TextIdentifier {
    fn as_text_id(&self) -> &str;
}

impl TextId {
    pub fn in_component(self, component_id: ComponentId) -> FatTextId {
        FatTextId(component_id, self)
    }
}

impl TextIdFactory {
    const BRANCH_POSTFIX: char = '_';

    /// Creates a new empty unlocked [`TextIdFactory`].
    pub fn new() -> Self {
        TextIdFactory::default()
    }

    /// Creates an empty unlocked [`TextIdFactory`] with at least the specified capacity for the prefix.
    pub fn with_capacity(capacity: usize) -> Self {
        TextIdFactory {
            prefix: String::with_capacity(capacity),
            locked: false,
        }
    }

    /// Asserts that the string satisfies requirements for being a [`TextId`] part.
    /// It must match `Fluent`'s identifier pattern (`[a-zA-Z][a-zA-Z0-9_-]*`) and
    /// any of its chars can't collide with the [`TextId`]'s path separator (`_`).
    fn assert_valid_name(&self, name: &str) {
        let valid = 'a: {
            let mut name_chars = name.chars();
            // check first char only for the first id part
            if self.prefix.is_empty() {
                match name_chars.next() {
                    Some(first) => {
                        if !first.is_ascii_alphabetic() {
                            break 'a false;
                        }
                    }
                    None => break 'a true,
                }
            }
            for ch in name_chars {
                if ch != '-' && !ch.is_ascii_alphanumeric() {
                    break 'a false;
                }
            }
            true
        };
        assert!(valid, "Invalid `TextId` part's name: {name}");
    }

    /// Calls `f` with locked [`TextIdFactory`] which panics when used to create [`TextId`].
    /// Reverts the change after the call.
    pub fn with_lock<R>(&mut self, f: impl FnOnce(&mut TextIdFactory) -> R) -> R {
        let was_locked = std::mem::replace(&mut self.locked, true);
        let result = f(self);
        self.locked = was_locked;
        result
    }

    fn push_branch(&mut self, name: &str) {
        self.prefix.reserve(name.len() + 1);
        self.prefix.push_str(name);
        self.prefix.push(Self::BRANCH_POSTFIX);
    }

    pub fn branch(mut self, name: &str) -> Self {
        self.assert_valid_name(name);
        if !self.locked {
            self.push_branch(name);
        }
        self
    }

    /// Calls `f` with [`TextIdFactory`] where `name` and a separator are appended to the prefix.
    /// Reverts the change after the call.
    /// Calls `f` with unchanged `self` when locked.
    pub fn with_branch<R>(&mut self, name: &str, f: impl FnOnce(&mut TextIdFactory) -> R) -> R {
        self.assert_valid_name(name);
        if self.locked {
            return f(self);
        }
        self.push_branch(name);
        let result = f(self);
        self.prefix.truncate(self.prefix.len() - 1 - name.len());
        result
    }

    /// Creates [`TextId`] from the stored prefix concatenated with `name`.
    /// Panics when `self` is locked.
    pub fn create(&self, name: &str) -> TextId {
        assert!(!self.locked);
        self.assert_valid_name(name);
        TextId(self.prefix.clone() + name)
    }
}

impl TextIdentifier for str {
    fn as_text_id(&self) -> &str {
        self
    }
}

impl TextIdentifier for TextId {
    fn as_text_id(&self) -> &str {
        self.0.as_str()
    }
}
