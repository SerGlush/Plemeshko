use std::{
    any::{type_name, Any, TypeId},
    borrow::Cow,
    collections::HashMap,
};

use anyhow::{anyhow, bail, Context, Result};
use serde_json::value::RawValue;

use crate::state::components::{ComponentIndexer, SharedComponents};

use super::{
    create_config_text_id_factory, AnySendSync, Config, ConfigArray, ConfigFinalizationPriority,
    ConfigIndexerMap, ConfigLabel, ConfigsLoadingContext, Prepare,
};

pub type LabelRawMap<C> = HashMap<ConfigLabel<C>, <C as Config>::Raw>;
type LabelMapToIdMap =
    fn(ctx: &mut ConfigsLoadingContext<'_>, Box<dyn Any>) -> Result<Box<AnySendSync>>;
type ParseAddingToAnyStore = fn(&mut dyn Any, label: String, raw_cfg: &RawValue) -> Result<()>;
type CreateAnyBox = fn() -> Box<dyn Any>;
type Finalize = fn(&ComponentIndexer, &mut SharedComponents) -> Result<()>;

#[derive(Default)]
pub struct ConfigTypeRegistry {
    pub(super) type_map: HashMap<
        TypeId,
        (
            CreateAnyBox,
            LabelMapToIdMap,
            ConfigFinalizationPriority,
            Finalize,
        ),
    >,
    pub(super) tag_map: HashMap<&'static str, (TypeId, ParseAddingToAnyStore)>,
}

impl ConfigTypeRegistry {
    pub fn new() -> Self {
        ConfigTypeRegistry::default()
    }

    pub fn register<C: Config>(&mut self) -> Result<()> {
        let type_id = TypeId::of::<C>();
        self.type_map
            .try_insert(
                type_id,
                (
                    || Box::<LabelRawMap<C>>::default(),
                    label_map_to_id_map::<C>,
                    C::FINALIZATION_PRIORITY,
                    C::finalize,
                ),
            )
            .map_err(|_| anyhow!("Type already registered: {}", type_name::<C>()))?;
        self.tag_map
            .try_insert(C::TAG, (type_id, parse_adding_to_any_store::<C>))
            .map_err(|_| anyhow!("Tag already registered: {}", C::TAG))?;
        Ok(())
    }

    pub fn finalizers(&self) -> impl Iterator<Item = Finalize> {
        let mut finalizers: Vec<(_, _)> = self
            .type_map
            .iter()
            .map(|(_, (_, _, fp, fin))| (*fp, *fin))
            .collect();
        finalizers.sort_unstable_by_key(|(fp, _)| -fp);
        finalizers.into_iter().map(|(_, fin)| fin)
    }
}

fn parse_adding_to_any_store<C: Config>(
    dst: &mut dyn Any,
    label: String,
    raw_cfg: &RawValue,
) -> Result<()> {
    dst.downcast_mut::<LabelRawMap<C>>()
        .ok_or_else(|| {
            anyhow!(
                "Parsing configs: Storage had type not matching its key: {}",
                type_name::<C>()
            )
        })
        .and_then(|store| {
            let config = serde_json::from_str(raw_cfg.get()).with_context(|| "Parsing failed")?;
            match store.try_insert(ConfigLabel::new(label), config) {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!(
                    "Identifier already loaded: {}",
                    e.entry.key().clone()
                )),
            }
        })
}

fn label_map_to_id_map<C: Config>(
    ctx: &mut ConfigsLoadingContext<'_>,
    label_map: Box<dyn Any>,
) -> Result<Box<AnySendSync>> {
    let label_to_cfg = label_map
        .downcast::<LabelRawMap<C>>()
        .map_err(|_| anyhow!("Label-Config map had invalid type, tag: {}", C::TAG))?;
    let mut configs = (0..label_to_cfg.len())
        .map(|_| None)
        .collect::<Vec<Option<C>>>();
    for (label, raw_config) in label_to_cfg.into_iter() {
        let mut tif = create_config_text_id_factory(&label);
        let prepared_config = raw_config.prepare(ctx, &mut tif)?;
        let id = ctx.st.get_or_create_id(Cow::Borrowed(&label))?;
        let index: usize = id.0.try_into().unwrap();
        let stored_config = configs
            .get_mut(index)
            .ok_or_else(|| anyhow!("Config not found: {}", label.0))?;
        if stored_config.is_some() {
            bail!("Config at id initilized twice: {}", index);
        }
        *stored_config = Some(prepared_config);
    }
    let configs = configs
        .into_iter()
        .try_collect::<Vec<C>>()
        .ok_or_else(|| anyhow!("Config storage not fully initialized: {}", type_name::<C>()))?; // todo: index reporting
    Ok(Box::new(ConfigArray(configs)))
}
