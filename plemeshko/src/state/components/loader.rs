use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::state::{
    components::{app::AppComponent, shared::SharedComponent, ComponentId},
    config::{ConfigRepository, ConfigTypeRegistry},
    text::TextRepository,
    texture::TextureRepository,
};

use super::{app::AppComponents, shared::SharedComponents, ComponentIndexer};

pub struct ComponentLoader {
    indexer: ComponentIndexer,
    config_type_registry: ConfigTypeRegistry,
}

pub struct ComponentLoadingContext<'a, S> {
    pub component_indexer: &'a ComponentIndexer,
    pub app_components: &'a mut AppComponents,
    pub shared_components: &'a mut SharedComponents,

    /// Id of the component being loaded.
    pub(super) component_id: ComponentId,

    /// State of the current loading stage.
    pub st: &'a mut S,
}

impl<'a, S> ComponentLoadingContext<'a, S> {
    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn with_st<'b, T>(&'b mut self, st: &'b mut T) -> ComponentLoadingContext<'b, T> {
        ComponentLoadingContext {
            st,
            component_indexer: self.component_indexer,
            app_components: self.app_components,
            shared_components: self.shared_components,
            component_id: self.component_id,
        }
    }
}

const COMPONENT_DIR_CONFIGS: &str = "configs";
const COMPONENT_DIR_TEXTS: &str = "texts";
const COMPONENT_DIR_TEXTURES: &str = "textures";

impl ComponentLoader {
    pub fn new() -> Result<Self> {
        let config_type_registry = crate::sim::config::register()?;
        let indexer = ComponentIndexer::default();
        Ok(ComponentLoader {
            indexer,
            config_type_registry,
        })
    }

    pub fn indexer(&self) -> &ComponentIndexer {
        &self.indexer
    }

    pub fn config_type_registry(&self) -> &ConfigTypeRegistry {
        &self.config_type_registry
    }

    /// Load single component with its data read from the specified directory subdirectories.
    pub fn load_single(
        &mut self,
        shared_comps: &mut SharedComponents,
        app_comps: &mut AppComponents,
        label: String,
        mut dir: PathBuf,
    ) -> Result<()> {
        // todo: reclaim unloaded component slots (or not? misusing unloaded's ids with replacement's ones may be confusing)
        let component_id = ComponentId(self.indexer.0.create_id(label)?);
        let mut ctx = ComponentLoadingContext {
            component_indexer: &self.indexer,
            app_components: app_comps,
            shared_components: shared_comps,
            component_id,
            st: &mut (),
        };

        dir.push(COMPONENT_DIR_TEXTS);
        let texts = if std::fs::try_exists(&dir)
            .with_context(|| "Checking existence of component's texts directory.")?
        {
            TextRepository::from_directory(&dir)?
        } else {
            TextRepository::new()
        };
        assert!(dir.pop());

        dir.push(COMPONENT_DIR_CONFIGS);
        let configs = if std::fs::try_exists(&dir)
            .with_context(|| "Checking existence of component's configs directory.")?
        {
            ConfigRepository::from_directory(&self.config_type_registry, &mut ctx, &dir)?
        } else {
            ConfigRepository::new(&self.config_type_registry, &mut ctx)?
        };
        assert!(dir.pop());

        dir.push(COMPONENT_DIR_TEXTURES);
        let textures = if std::fs::try_exists(&dir)
            .with_context(|| "Checking existence of component's textures directory.")?
        {
            TextureRepository::from_directory(&dir)?
        } else {
            TextureRepository::new()
        };
        assert!(dir.pop());

        let shared_comp = SharedComponent { configs };
        let app_comp = AppComponent { texts, textures };
        let component_index = component_id.0 as usize;
        if app_comps.0.len() == component_index {
            app_comps.0.push(Some(app_comp));
            shared_comps.0.push(Some(shared_comp));
        } else {
            assert!(app_comps.0.len() > component_index);
            app_comps.0[component_index] = Some(app_comp);
            shared_comps.0[component_index] = Some(shared_comp);
        }
        Ok(())
    }

    /// Loads specified directory subdirectories as components.
    pub fn load_each(
        &mut self,
        shared_comps: &mut SharedComponents,
        app_comps: &mut AppComponents,
        top_dir: &Path,
    ) -> Result<()> {
        for top_entry in std::fs::read_dir(top_dir)? {
            let top_entry_path = top_entry?.path();
            if top_entry_path.is_dir() {
                let label = top_entry_path
                    .file_stem()
                    .ok_or_else(|| {
                        anyhow!(
                            "Can't get component's name from its path: {}",
                            top_entry_path.display()
                        )
                    })?
                    .to_string_lossy()
                    .into_owned();
                self.load_single(shared_comps, app_comps, label, top_entry_path)?;
            }
        }
        Ok(())
    }
}
