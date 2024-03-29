#[derive(Default)]
pub struct LabelFactory {
    prefix: String,

    /// Occurs when the `prefix` extension is required but can't be created.
    locked: bool,
}

impl LabelFactory {
    /// Character used to separate label parts.
    /// Goes after every branch.
    pub const BRANCH_POSTFIX: char = '_';

    /// Creates a new empty unlocked [`LabelFactory`].
    pub fn new() -> Self {
        LabelFactory::default()
    }

    /// Creates an empty unlocked [`LabelFactory`] with at least the specified capacity for the prefix.
    pub fn with_capacity(capacity: usize) -> Self {
        LabelFactory {
            prefix: String::with_capacity(capacity),
            locked: false,
        }
    }

    /// Asserts that the string satisfies requirements for being a label part.
    fn assert_valid_name(&self, name: &str) {
        let valid = 'a: {
            let mut name_chars = name.chars();
            // check first char only for the first label part
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
        assert!(valid, "Invalid label part's name: {name}");
    }

    /// Calls `f` with locked [`LabelFactory`] which panics when used to create a label.
    /// Reverts the change after the call.
    pub fn with_lock<R>(&mut self, f: impl FnOnce(&mut LabelFactory) -> R) -> R {
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

    /// Calls `f` with [`LabelFactory`] where `name` and a branch postfix character are appended to the prefix.
    /// `name` must match the `[a-zA-Z][a-zA-Z0-9-]*` pattern.
    /// Reverts the change after the call.
    /// Calls `f` with unchanged `self` when locked.
    pub fn with_branch<R>(&mut self, name: &str, f: impl FnOnce(&mut LabelFactory) -> R) -> R {
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
    /// `name` must match the `[a-zA-Z][a-zA-Z0-9-]*` pattern.
    /// Panics when `self` is locked.
    pub fn create(&self, name: &str) -> String {
        assert!(!self.locked);
        self.assert_valid_name(name);
        self.prefix.clone() + name
    }
}
