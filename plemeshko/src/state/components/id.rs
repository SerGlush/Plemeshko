use anyhow::Result;
use serde::Deserializer;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ComponentLabel(pub(super) String);

pub const COMPONENT_CORE_LABEL: &str = "";
const COMPONENT_LABEL_SEPARATOR: char = '/';

/// Separates component label prefix from object label.
/// Component label is `None` when there is no label, meaning "local" component.
pub fn split_label<'de, D: Deserializer<'de>>(
    raw: &str,
) -> Result<(Option<ComponentLabel>, &str), D::Error> {
    if raw
        .chars()
        .filter(|c| COMPONENT_LABEL_SEPARATOR == *c)
        .count()
        > 1
    {
        return Err(serde::de::Error::custom(format!(
            "Multiple namespace separators in a label: {raw}"
        )));
    }
    Ok(match raw.split_once(COMPONENT_LABEL_SEPARATOR) {
        Some((comp, postfix)) => (Some(ComponentLabel(comp.to_owned())), postfix),
        None => (None, raw),
    })
}

pub fn concat_label(comp: Option<ComponentLabel>, postfix: &str) -> String {
    match comp {
        Some(mut comp) => {
            comp.0.push(COMPONENT_LABEL_SEPARATOR);
            comp.0 + postfix
        }
        None => postfix.to_owned(),
    }
}

pub(super) type RawComponentId = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ComponentId(pub(super) RawComponentId);

impl ComponentId {
    pub fn core() -> Self {
        ComponentId(0)
    }

    pub(super) fn to_index(self) -> usize {
        self.0.try_into().unwrap()
    }
}
