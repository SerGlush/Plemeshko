#[macro_use]
pub mod config;
pub mod components;
pub mod indexer;
pub mod text;
#[macro_use]
pub mod serializable;

use std::{
    borrow::Cow,
    sync::{Mutex, RwLock},
};

use anyhow::{Context, Result};
use fluent::FluentArgs;

use crate::sim::{config::resource::ResourceId, RawSimSnapshot, Sim};

use self::{
    components::{
        AppComponents, ComponentId, ComponentLoader, SharedComponents, COMPONENT_CORE_LABEL,
    },
    config::{ConfigIndexerMap, FatConfigId},
    serializable::{Serializable, SerializationContext},
    text::{FatTextId, TextIdentifier},
};

const COMPONENTS_OTHER_DIR: &str = "mods";
const COMPONENT_CORE_DIR: &str = "core";
const RESOURCE_LABEL_HUMAN: &str = "human";

/// State shared between threads (ui/simulation).
/// Resides behind an immutable reference.
/// Never dropped because is always "leaked" at the start of the program.
// Shared *mutable* state must be behind a lock.
pub struct SharedState {
    pub components: RwLock<SharedComponents>,
    pub sim: Mutex<Option<Sim>>,
    pub human_id: ResourceId,
}

pub struct AppState {
    pub shared: &'static SharedState,
    pub components: AppComponents,
    pub component_loader: ComponentLoader,
}

fn load_sim(ctx: &mut SerializationContext<'_>) -> anyhow::Result<Sim> {
    let mut cli_args_iter = std::env::args();
    cli_args_iter.next(); // exe
    Ok(match cli_args_iter.next() {
        Some(snapshot_path) => {
            let file = std::fs::File::open(snapshot_path)?;
            let reader = std::io::BufReader::new(file);
            let snapshot = serde_json::from_reader::<_, RawSimSnapshot>(reader)?;
            let snapshot = Serializable::from_serializable(snapshot, ctx)?;
            Sim::restore(ctx.shared_components, snapshot)?
        }
        None => Sim::new(),
    })
}

/// Create environments and load core component
pub fn initialize_state() -> Result<(&'static SharedState, AppState)> {
    let mut shared_comps = SharedComponents::default();
    let mut app_comps = AppComponents::default();
    let mut component_loader = ComponentLoader::new()?;
    component_loader.load_single(
        &mut shared_comps,
        &mut app_comps,
        COMPONENT_CORE_LABEL.to_owned(),
        COMPONENT_CORE_DIR.into(),
    )?;
    match std::fs::try_exists(COMPONENTS_OTHER_DIR) {
        Ok(true) => {
            component_loader.load_each(
                &mut shared_comps,
                &mut app_comps,
                std::path::Path::new(COMPONENTS_OTHER_DIR),
            )?;
        }
        Ok(false) => {
            println!(
                "Skipping loading other components: Directory not found: {COMPONENTS_OTHER_DIR}"
            );
        }
        Err(e) => {
            println!("Skipping loading other components: Error checking directory: {e}");
        }
    }
    let human_id = shared_comps
        .get_core()?
        .configs
        .get_id_from_raw(RESOURCE_LABEL_HUMAN)?;

    let sim = {
        let mut ser_ctx = SerializationContext {
            component_indexer: component_loader.indexer(),
            app_components: &app_comps,
            shared_components: &mut shared_comps,
        };
        load_sim(&mut ser_ctx).with_context(|| "Error reading Sim snapshot")?
    };
    let shared_st: &SharedState = Box::leak(Box::new(SharedState {
        components: RwLock::new(shared_comps),
        sim: Mutex::new(Some(sim)),
        human_id: FatConfigId::new_core(human_id),
    }));
    let app_st = AppState {
        shared: shared_st,
        components: app_comps,
        component_loader,
    };
    Ok((shared_st, app_st))
}

// todo: return TextId prefixed with ComponentLabel When text is not found
impl AppState {
    /// Retrieve text entry.
    pub fn get_text<'a>(&'a self, id: &FatTextId) -> Result<Cow<'a, str>> {
        self.components.get_component(id.0)?.get_text(&id.1, None)
    }

    /// Retrieve text entry from the core component.
    pub fn get_text_core<'a>(
        &'a self,
        id: &(impl TextIdentifier + ?Sized),
    ) -> Result<Cow<'a, str>> {
        self.components
            .get_component(ComponentId::core())?
            .get_text(id, None)
    }

    /// Format text entry using specified arguments.
    pub fn fmt_text<'a>(&'a self, id: FatTextId, args: &'a FluentArgs<'_>) -> Result<Cow<'a, str>> {
        self.components
            .get_component(id.0)?
            .get_text(&id.1, Some(args))
    }

    /// Format text entry from the core component using specified arguments.
    pub fn fmt_text_core<'a>(
        &'a self,
        id: &(impl TextIdentifier + ?Sized),
        args: &'a FluentArgs<'_>,
    ) -> Result<Cow<'a, str>> {
        self.components
            .get_component(ComponentId::core())?
            .get_text(id, Some(args))
    }
}
