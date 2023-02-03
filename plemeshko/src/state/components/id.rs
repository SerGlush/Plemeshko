use anyhow::{anyhow, Result};
use bytemuck::TransparentWrapper;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::ComponentsRef;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ComponentLabel(pub(super) String);

// SAFETY:
// Has `#[repr(transparent)]` and only 1 field.
unsafe impl TransparentWrapper<String> for ComponentLabel {}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct RawFatLabel(
    /// Component prefix of the label, `None` when the label is local (allowed only inside components).
    pub Option<ComponentLabel>,
    pub String,
);

pub(super) type RawComponentId = u16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ComponentId(pub(super) RawComponentId);

#[derive(Clone, Copy)]
pub struct ComponentSlotId(pub(super) usize);

pub const COMPONENT_CORE_LABEL: &str = "";
pub const COMPONENT_LABEL_SEPARATOR: char = '/';

impl RawFatLabel {
    pub fn deserialize_component_id(&self, comps: ComponentsRef<'_>) -> Result<ComponentId> {
        let comp_label = self.0.as_ref().ok_or_else(|| {
            anyhow!(
                "Deserializing local label outside any component: ?/{}",
                self.1
            )
        })?;
        comps.indexer.id(comp_label)
    }
}

impl ComponentId {
    pub fn core() -> Self {
        ComponentId(0)
    }

    pub(super) fn to_index(self) -> usize {
        self.0.try_into().unwrap()
    }
}

impl ComponentSlotId {
    pub fn assume_occupied(self) -> ComponentId {
        ComponentId(self.0.try_into().unwrap())
    }
}

/// Separates component label prefix from object label.
/// Component label is `None` when there is no label, meaning "local" component.
fn split_label<'de, D: Deserializer<'de>>(
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

fn concat_label(comp: Option<ComponentLabel>, postfix: &str) -> String {
    match comp {
        Some(mut comp) => {
            comp.0.push(COMPONENT_LABEL_SEPARATOR);
            comp.0 + postfix
        }
        None => postfix.to_owned(),
    }
}

impl<'de> Deserialize<'de> for RawFatLabel {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let (comp, postfix) = split_label::<D>(&raw)?;
        Ok(RawFatLabel(comp, postfix.to_owned()))
    }
}

impl Serialize for RawFatLabel {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw = concat_label(self.0.clone(), &self.1);
        serializer.serialize_str(&raw)
    }
}
