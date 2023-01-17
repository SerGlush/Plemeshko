use std::{
    any::{type_name, Any, TypeId},
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io, path,
};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::value::RawValue;

use super::{indexer::ConfigIndexer, AnyIdMap, Config, ConfigLabel, ConfigRepository, IdMap};

#[derive(Deserialize)]
pub struct RawConfig {
    pub tag: String,
    pub label: String,
    // todo: something similar to `#[serde(flatten)]` (or custom deserializer?) can remove unnecessary nesting
    pub payload: Box<RawValue>,
}

type SomeConfigLoadFn = fn(&mut dyn Any, id: String, raw_cfg: &RawValue) -> Result<()>;

type SomeLabelMapToIdMap = fn(&mut ConfigIndexer, Box<dyn Any>) -> Result<Box<AnyIdMap>>;

type LabelRawMap<C> = HashMap<String, <C as Config>::Raw>;

#[derive(Default)]
pub struct ConfigRepositoryBuilder {
    configs: HashMap<&'static str, Box<dyn Any>>,
    tagged_loaders: HashMap<&'static str, (TypeId, SomeConfigLoadFn, SomeLabelMapToIdMap)>,
}

fn parse_adding_to_any_store<C: Config>(
    dst: &mut dyn Any,
    id: String,
    raw_cfg: &RawValue,
) -> Result<()> {
    dst.downcast_mut::<LabelRawMap<C>>()
        .ok_or_else(|| {
            anyhow!(
                "Storage had type not matching its key, expected: {}",
                type_name::<C>()
            )
        })
        .and_then(|store| {
            let config = serde_json::from_str(raw_cfg.get()).with_context(|| "Parsing failed")?;
            match store.try_insert(id, config) {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!(
                    "Identifier already loaded: {}",
                    e.entry.key().clone()
                )),
            }
        })
}

fn label_map_to_id_map<C: Config>(
    indexer: &mut ConfigIndexer,
    label_map: Box<dyn Any>,
) -> Result<Box<AnyIdMap>> {
    let label_to_cfg = label_map
        .downcast::<LabelRawMap<C>>()
        .map_err(|_| anyhow!("Label-Config map had invalid type, tag: {}", C::TAG))?;
    Ok(Box::new(
        label_to_cfg
            .into_iter()
            .map(|(label, cfg)| {
                let id =
                    indexer.get_or_create_id_raw(TypeId::of::<C>(), Cow::Borrowed(label.as_str()));
                let cfg = C::prepare(cfg, ConfigLabel::new(label), indexer);
                Ok::<_, anyhow::Error>((id, cfg))
            })
            .try_collect::<IdMap<C>>()?,
    ))
}

impl ConfigRepositoryBuilder {
    pub fn new() -> Self {
        ConfigRepositoryBuilder {
            configs: HashMap::new(),
            tagged_loaders: HashMap::new(),
        }
    }

    pub fn register<C: Config>(&mut self) -> Result<()> {
        self.configs
            .try_insert(C::TAG, Box::<LabelRawMap<C>>::default())
            .map_err(|_| anyhow!("Type already registered: {}", type_name::<C>()))?;
        match self.tagged_loaders.try_insert(
            C::TAG,
            (
                TypeId::of::<C>(),
                parse_adding_to_any_store::<C>,
                label_map_to_id_map::<C>,
            ),
        ) {
            Ok(_) => Ok(()),
            Err(_) => Err(anyhow!("Tag already registered: {}", C::TAG)),
        }
    }

    pub fn build(self) -> Result<ConfigRepository> {
        let mut indexer = ConfigIndexer::new();
        let mut configs = HashMap::with_capacity(self.configs.len());
        for (tag, label_map) in self.configs {
            let &(type_id, _, lmap_to_imap) = self
                .tagged_loaders
                .get(tag)
                .ok_or_else(|| anyhow!("Tagged loader doesn't exist: {}", tag))?;
            let id_map = lmap_to_imap(&mut indexer, label_map)?;
            configs.try_insert(type_id, id_map).unwrap();
        }

        // todo: when some cfg param enabled, vaidate that all ids point to existing configs
        // consider (on the same flag or another one) disabling cfg existence checks when retrieving by id
        Ok(ConfigRepository { configs, indexer })
    }

    pub fn load_raw(&mut self, raw: RawConfig) -> Result<()> {
        let insert_cfg = self
            .tagged_loaders
            .get(raw.tag.as_str())
            .ok_or_else(|| anyhow!("Tag not registered: {}", raw.tag))?
            .1;
        let store = self
            .configs
            .get_mut(raw.tag.as_str())
            .ok_or_else(|| anyhow!("Storage for requested tag doesn't exist: {}", raw.tag))?;
        insert_cfg(store.as_mut(), raw.label, raw.payload.as_ref())
    }

    pub fn load_file(&mut self, path: &path::Path) -> Result<()> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        let raw_cfgs = serde_json::from_reader::<_, Vec<RawConfig>>(reader)
            .with_context(|| format!("Pre-parsing file {} failed", path.display()))?;
        for raw_cfg in raw_cfgs {
            self.load_raw(raw_cfg).with_context(|| {
                format!(
                    "Failed to parse config payload when loading \"{}\": ",
                    path.display()
                )
            })?;
        }
        Ok(())
    }

    pub fn load_directory(&mut self, path: &path::Path) -> Result<()> {
        for dir_entry in std::fs::read_dir(path)? {
            let dir_entry = dir_entry?;
            let entry_path = dir_entry.path();
            if entry_path.is_file() {
                self.load_file(&entry_path)?;
            } else if entry_path.is_dir() {
                self.load_directory(&entry_path)?;
            }
        }
        Ok(())
    }
}
