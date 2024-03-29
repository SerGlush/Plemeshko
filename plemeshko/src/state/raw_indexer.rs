use std::{
    borrow::{Borrow, Cow},
    collections::{hash_map::RawEntryMut, HashMap},
    fmt::{Debug, Display},
    hash::Hash,
};

use anyhow::{anyhow, Result};
use educe::Educe;

#[derive(Educe)]
#[educe(Default)]
pub struct RawIndexer<Label, Id> {
    pub label_to_id: HashMap<Label, Id>,
    pub id_to_label: Vec<Label>,
}

impl<Label: Hash + Eq + Into<String>, Id: Copy> RawIndexer<Label, Id> {
    pub fn new() -> Self {
        RawIndexer::default()
    }

    pub fn create_id(&mut self, label: Label) -> Result<Id>
    where
        Id: TryFrom<usize, Error: Debug>,
        Label: Display + Clone,
    {
        match self.label_to_id.raw_entry_mut().from_key(&label) {
            RawEntryMut::Occupied(_) => Err(anyhow!("Label already registered: {}", label)),
            RawEntryMut::Vacant(vacant) => {
                let id = self.id_to_label.len().try_into().unwrap();
                self.id_to_label.push(label.clone());
                vacant.insert(label, id);
                Ok(id)
            }
        }
    }

    pub fn pop(&mut self) -> Option<(Label, Id)>
    where
        Label: Display,
    {
        let label = self.id_to_label.pop()?;
        match self.label_to_id.remove(&label) {
            Some(id) => Some((label, id)),
            None => panic!("Label doesn't have corresponding id: {label}"),
        }
    }

    pub fn declare_id<L: ?Sized + ToOwned<Owned = Label> + Hash + Eq>(
        &mut self,
        label: Cow<'_, L>,
    ) -> Result<Id>
    where
        Label: Borrow<L>,
        Id: TryFrom<usize, Error: Debug>,
    {
        match self.label_to_id.raw_entry_mut().from_key(label.as_ref()) {
            RawEntryMut::Occupied(occupied) => Ok(*occupied.get()),
            RawEntryMut::Vacant(vacant) => {
                let id = self.id_to_label.len();
                self.id_to_label.push(label.clone().into_owned());
                let id = id.try_into().unwrap();
                vacant.insert(label.into_owned(), id);
                Ok(id)
            }
        }
    }

    pub fn id<L: ?Sized + Hash + Eq + Display>(&self, label: &L) -> Result<Id>
    where
        Label: Borrow<L>,
    {
        match self.label_to_id.get(label) {
            Some(&id) => Ok(id),
            None => Err(anyhow!("Label not registered: {}", label)),
        }
    }

    pub fn label(&self, id: Id) -> Result<&Label>
    where
        Id: TryInto<usize, Error: Debug>,
    {
        let id: usize = id.try_into().unwrap();
        self.id_to_label
            .get(id)
            .ok_or_else(|| anyhow!("Label corresponding to the requested index wasn't found"))
    }
}

impl<Id> RawIndexer<String, Id> {
    pub fn report_id(&self, id: Id) -> String
    where
        Id: TryInto<usize, Error: Debug>,
    {
        let id: usize = id.try_into().unwrap();
        match self.id_to_label.get(id) {
            Some(label) => label.clone(),
            None => id.to_string() + "?!",
        }
    }
}
