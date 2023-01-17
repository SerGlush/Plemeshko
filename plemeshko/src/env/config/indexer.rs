use std::{
    any::TypeId,
    borrow::Cow,
    collections::{hash_map::RawEntryMut, HashMap},
};

use anyhow::{anyhow, bail, Result};

use super::{Config, ConfigId, ConfigLabel, UnderId};

#[derive(Default)]
pub struct ConfigIndexer {
    label_to_id: HashMap<String, (UnderId, Vec<TypeId>)>,
    next_id: UnderId,
    id_to_label: HashMap<UnderId, String>,
}

impl ConfigIndexer {
    pub fn new() -> Self {
        ConfigIndexer::default()
    }

    pub(super) fn get_or_create_id_raw(&mut self, type_id: TypeId, label: Cow<'_, str>) -> UnderId {
        match self.label_to_id.raw_entry_mut().from_key(label.as_ref()) {
            RawEntryMut::Occupied(mut occupied) => {
                let (id, type_ids) = occupied.get_mut();
                type_ids.push(type_id);
                *id
            }
            RawEntryMut::Vacant(vacant) => {
                let id = self.next_id;
                self.next_id += 1;
                vacant.insert(label.to_string(), (id, vec![type_id]));
                self.id_to_label.try_insert(id, label.into_owned()).unwrap();
                id
            }
        }
    }

    pub fn get_id<C: Config>(&self, label: String) -> Result<ConfigId<C>> {
        let (id, type_ids) = self
            .label_to_id
            .get(label.as_str())
            .ok_or_else(|| anyhow!("Label not registered: {}", label))?;
        if !type_ids.contains(&TypeId::of::<C>()) {
            bail!("Label '{}' not registered for '{}'", label, C::TAG);
        }
        Ok(ConfigId::new(*id))
    }

    pub fn get_or_create_id<C: Config>(&mut self, label: ConfigLabel<C>) -> ConfigId<C> {
        ConfigId::new(self.get_or_create_id_raw(TypeId::of::<C>(), Cow::Owned(label.0)))
    }

    pub fn get_label<C: Config>(&self, id: ConfigId<C>) -> Result<&ConfigLabel<C>> {
        self.id_to_label
            .get(&id.0)
            .ok_or_else(|| anyhow!("Label corresponding to the requested index wasn't found"))
            .map(|label| unsafe { std::mem::transmute(label) })
    }
}
