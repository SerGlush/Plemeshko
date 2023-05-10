use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    fs::File,
    io,
    path::Path,
};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::value::RawValue;

use crate::state::{
    components::{ComponentId, ComponentsRef},
    texture::TextureRepository,
};

use super::{indexer::ConfigIndexer, type_registry::ConfigTypeRegistry, ConfigRepository};

#[derive(Deserialize)]
pub struct RawConfig {
    pub tag: String,
    pub label: String,
    // todo: something similar to `#[serde(flatten)]` (or custom deserializer?) can remove unnecessary nesting
    #[serde(default = "deser_default_json_object")]
    pub payload: Box<RawValue>,
}

pub struct ConfigRepositoryBuilder(HashMap<TypeId, (Box<dyn Any>, ConfigIndexer)>);

#[derive(Clone, Copy)]
pub struct ComponentPreConfigsRef<'a> {
    pub(super) id: ComponentId,
    pub textures: &'a TextureRepository,
}

pub struct ConfigsLoadingContext<'a> {
    /// Components already loaded.
    /// Used to resolve other components' indices and their assets' indices.
    pub other_components: ComponentsRef<'a>,

    /// Already loaded parts of the current component.
    /// Used to resolve other local assets' indices.
    pub this_component: ComponentPreConfigsRef<'a>,

    // todo: encapsulate
    pub st: &'a mut HashMap<TypeId, (Box<dyn Any>, ConfigIndexer)>,

    loaded: &'a HashSet<TypeId>,
}

impl<'a> ComponentPreConfigsRef<'a> {
    pub fn new(id: ComponentId, textures: &'a TextureRepository) -> Self {
        ComponentPreConfigsRef { id, textures }
    }

    pub fn id(self) -> ComponentId {
        self.id
    }
}

impl ConfigRepositoryBuilder {
    pub fn new(reg: &ConfigTypeRegistry) -> Result<Self> {
        let mut configs = HashMap::new();
        for (&type_id, (new_map, _, _, _)) in reg.type_map.iter() {
            let indexer = ConfigIndexer::new();
            let map = new_map();
            configs
                .try_insert(type_id, (map, indexer))
                .map_err(|_| anyhow!("Type registered more than once: {:?}", type_id))?;
        }
        Ok(ConfigRepositoryBuilder(configs))
    }

    pub fn build<'a>(
        mut self,
        cfg_ty_reg: &ConfigTypeRegistry,
        components: ComponentsRef<'a>,
        pre_cfg: ComponentPreConfigsRef<'a>,
    ) -> Result<ConfigRepository> {
        let mut loaded_cfg_tys = HashSet::with_capacity(cfg_ty_reg.type_map.len());
        for (type_id, (_, labelmap_to_idmap, _, _)) in cfg_ty_reg.type_map.iter() {
            let Some(label_to_raw) = self.0.get_mut(type_id) else {
                continue;
            };
            let label_to_raw = std::mem::replace(&mut label_to_raw.0, Box::new(()));
            let config_storage = labelmap_to_idmap(
                &mut ConfigsLoadingContext {
                    other_components: components,
                    this_component: pre_cfg,
                    st: &mut self.0,
                    loaded: &loaded_cfg_tys,
                },
                label_to_raw,
            )?;
            if !loaded_cfg_tys.insert(*type_id) {
                panic!("Config type loaded multiple times");
            }
            self.0.get_mut(type_id).unwrap().0 = config_storage;
        }

        // todo: when some cfg param enabled, vaidate that all ids point to existing configs
        // consider (on the same flag or another one) disabling cfg existence checks when retrieving by id

        Ok(ConfigRepository {
            // SAFETY:
            // Should be safe because all `Raw` configs are prepared, and thus impl `Config` and `Send + Sync`
            configs: unsafe {
                std::mem::transmute::<
                    HashMap<TypeId, (Box<dyn Any>, _)>,
                    HashMap<TypeId, (Box<dyn Any + Send + Sync>, _)>,
                >(self.0)
            },
        })
    }

    pub fn load_raw(&mut self, reg: &ConfigTypeRegistry, raw: RawConfig) -> Result<()> {
        let (type_id, insert_cfg) = reg
            .tag_map
            .get(raw.tag.as_str())
            .ok_or_else(|| anyhow!("Tag not registered: {}", raw.tag))?;
        let store = self
            .0
            .get_mut(type_id)
            .ok_or_else(|| anyhow!("Storage for requested tag doesn't exist: {}", raw.tag))?;
        insert_cfg(store.0.as_mut(), raw.label, raw.payload.as_ref())
    }

    pub fn load_file(&mut self, reg: &ConfigTypeRegistry, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        let raw_cfgs = serde_json::from_reader::<_, Vec<RawConfig>>(reader)
            .with_context(|| format!("Pre-parsing file {} failed", path.display()))?;
        for raw_cfg in raw_cfgs {
            self.load_raw(reg, raw_cfg).with_context(|| {
                format!(
                    "Failed to parse config payload when loading \"{}\": ",
                    path.display()
                )
            })?;
        }
        Ok(())
    }

    pub fn load_directory(&mut self, reg: &ConfigTypeRegistry, path: &Path) -> Result<()> {
        for dir_entry in std::fs::read_dir(path)? {
            let dir_entry = dir_entry?;
            let entry_path = dir_entry.path();
            if entry_path.is_file() {
                self.load_file(reg, &entry_path)?;
            } else if entry_path.is_dir() {
                self.load_directory(reg, &entry_path)?;
            }
        }
        Ok(())
    }
}

impl<'a> ConfigsLoadingContext<'a> {
    pub fn is_loaded<C: 'static>(&self) -> bool {
        self.loaded.contains(&TypeId::of::<C>())
    }
}

fn deser_default_json_object() -> Box<RawValue> {
    RawValue::from_string("{}".to_owned()).unwrap()
}
