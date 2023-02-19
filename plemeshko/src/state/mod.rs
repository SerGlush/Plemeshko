#[macro_use]
pub mod config;
#[macro_use]
pub mod serializable;
pub mod components;
pub mod has;
pub mod label_factory;
pub mod raw_indexer;
pub mod sound;
pub mod text;
pub mod texture;

use std::sync::{Mutex, RwLock};

use anyhow::{anyhow, Context, Result};
use egui_extras::RetainedImage;

use crate::sim::{config::resource::ResourceId, RawSimSnapshot, Sim};

use self::{
    components::{
        AppComponents, ComponentId, ComponentLoader, ComponentsRef, SharedComponents,
        COMPONENT_CORE_LABEL,
    },
    config::FatConfigId,
    serializable::Serializable,
    texture::FatTextureId,
};

const COMPONENTS_OTHER_DIR: &str = "mods";
const COMPONENT_CORE_DIR: &str = "core";
const RESOURCE_LABEL_HUMAN: &str = "human";
const RESOURCE_LABEL_FOOD: &str = "food";

/// State shared between threads (ui/simulation).
/// Resides behind an immutable reference.
/// Never dropped because is always "leaked" at the start of the program.
// Shared *mutable* state must be behind a lock.
pub struct SharedState {
    pub components: RwLock<SharedComponents>,
    pub sim: Mutex<Option<Sim>>,
    pub audio: Option<Audio>,
    pub human_id: ResourceId,
    pub food_id: ResourceId,
}

pub struct AppState {
    pub shared: &'static SharedState,
    pub components: AppComponents,
    pub component_loader: ComponentLoader,
    fallback_texture: RetainedImage,
}

pub struct Audio {
    stream_handle: rodio::OutputStreamHandle,
    sink_sfx: rodio::Sink,
}

fn load_sim(comps: ComponentsRef<'_>) -> anyhow::Result<Sim> {
    let mut cli_args_iter = std::env::args();
    cli_args_iter.next(); // exe
    Ok(match cli_args_iter.next() {
        Some(snapshot_path) => {
            let file = std::fs::File::open(snapshot_path)?;
            let reader = std::io::BufReader::new(file);
            let snapshot = serde_json::from_reader::<_, RawSimSnapshot>(reader)?;
            let snapshot = Serializable::from_serializable(snapshot, comps)?;
            Sim::restore(comps.shared, snapshot)?
        }
        None => Sim::new(),
    })
}

/// Create environments and load core component
pub fn initialize_state() -> Result<(Option<rodio::OutputStream>, &'static SharedState, AppState)> {
    let mut shared_comps = SharedComponents::default();
    let mut app_comps = AppComponents::default();
    let mut component_loader = ComponentLoader::new()?;
    let components_changed = component_loader.load_single(
        &mut shared_comps,
        &mut app_comps,
        COMPONENT_CORE_LABEL.to_owned(),
        COMPONENT_CORE_DIR.into(),
    )?;
    match std::fs::try_exists(COMPONENTS_OTHER_DIR) {
        Ok(true) => {
            components_changed.consume(component_loader.load_each(
                &mut shared_comps,
                &mut app_comps,
                std::path::Path::new(COMPONENTS_OTHER_DIR),
            )?);
        }
        Ok(false) => {
            log::warn!(
                "Skipping loading other components: Directory not found: {COMPONENTS_OTHER_DIR}"
            );
        }
        Err(e) => {
            log::warn!("Skipping loading other components: Error checking directory: {e}");
        }
    }
    component_loader.finalize(components_changed, &mut shared_comps)?;

    let human_id = shared_comps
        .core()?
        .configs
        .id_from_raw(RESOURCE_LABEL_HUMAN)?;

    let food_id = shared_comps
        .core()?
        .configs
        .id_from_raw(RESOURCE_LABEL_FOOD)?;

    let sim = {
        let comps = ComponentsRef {
            indexer: component_loader.indexer(),
            app: &app_comps,
            shared: &shared_comps,
        };
        load_sim(comps).with_context(|| "Error reading Sim snapshot")?
    };

    let (audio_stream, audio_handle) = Audio::new();

    let shared_st: &SharedState = Box::leak(Box::new(SharedState {
        components: RwLock::new(shared_comps),
        sim: Mutex::new(Some(sim)),
        audio: audio_handle,
        human_id: FatConfigId::new_core(human_id),
        food_id: FatConfigId::new_core(food_id),
    }));
    let app_st = AppState {
        shared: shared_st,
        components: app_comps,
        component_loader,
        fallback_texture: RetainedImage::from_color_image(
            "<fallback>",
            egui::ColorImage::example(),
        ),
    };
    Ok((audio_stream, shared_st, app_st))
}

impl AppState {
    pub fn texture(&self, id: FatTextureId) -> Result<&RetainedImage> {
        Ok(self
            .components
            .component(id.0)?
            .textures
            .get(id.1)
            .unwrap_or(&self.fallback_texture))
    }

    pub fn texture_core(&self, label: &str) -> Result<&RetainedImage> {
        let core_textures = &self.components.component(ComponentId::core())?.textures;
        let id = match core_textures.id_from_raw(label) {
            Ok(id) => id,
            Err(e) => {
                log::warn!("Core texture retrieval failed: {e}");
                return Ok(&self.fallback_texture);
            }
        };
        core_textures
            .get(id)
            .ok_or_else(|| anyhow!("Invalid associated label's id: {label}"))
    }
}

impl SharedState {
    pub fn play_sfx<S>(&self, source: S)
    where
        S: rodio::Source + Send + 'static,
        f32: rodio::cpal::FromSample<S::Item>,
        S::Item: rodio::Sample + Send,
    {
        if let Some(audio) = &self.audio {
            audio.play_sfx(source);
        }
    }
}

impl Audio {
    pub fn new() -> (Option<rodio::OutputStream>, Option<Self>) {
        let (stream, stream_handle) = match rodio::OutputStream::try_default() {
            Ok(stream) => stream,
            Err(e) => {
                log::warn!("Couldn't initialize audio output stream: {e}");
                return (None, None);
            }
        };
        let sink_sfx = match rodio::Sink::try_new(&stream_handle) {
            Ok(sink) => sink,
            Err(e) => {
                log::warn!("Couldn't initialize audio sfx sink: {e}");
                return (None, None);
            }
        };
        (
            Some(stream),
            Some(Audio {
                stream_handle,
                sink_sfx,
            }),
        )
    }

    pub fn play_sfx<S>(&self, source: S)
    where
        S: rodio::Source + Send + 'static,
        f32: rodio::cpal::FromSample<S::Item>,
        S::Item: rodio::Sample + Send,
    {
        self.sink_sfx.append(source);
    }
}
