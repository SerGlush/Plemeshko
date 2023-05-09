use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use unic_langid::langid;

use crate::{
    params::{COMPONENT_CONFIGS_DIR, COMPONENT_TEXTS_DIR, COMPONENT_TEXTURES_DIR},
    state::{
        components::{app::AppComponent, shared::SharedComponent, ComponentId},
        config::{ComponentPreConfigsRef, ConfigRepositoryBuilder, ConfigTypeRegistry},
        text::TextRepository,
        texture::TextureRepository,
    },
};

use super::{app::AppComponents, shared::SharedComponents, ComponentIndexer, ComponentsRef};

pub struct ComponentLoader {
    indexer: ComponentIndexer,
    config_type_registry: ConfigTypeRegistry,
}

#[must_use]
pub struct ComponentsChangedToken(());

impl ComponentsChangedToken {
    pub fn new() -> Self {
        ComponentsChangedToken(())
    }

    pub fn consume(&self, _other: Self) {}
}

impl Drop for ComponentsChangedToken {
    fn drop(&mut self) {
        panic!("Components changed without finalization.");
    }
}

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
    ) -> Result<ComponentsChangedToken> {
        // todo: reclaim unloaded component slots (or not? misusing unloaded's ids with replacement's ones may be confusing)
        let component_id = ComponentId(self.indexer.0.create_id(label)?);

        dir.push(COMPONENT_TEXTS_DIR);
        let texts = if std::fs::try_exists(&dir)
            .with_context(|| "Checking existence of component's texts directory.")?
        {
            TextRepository::from_directory(&dir, langid!("en"))?
        } else {
            TextRepository::new()
        };
        assert!(dir.pop());

        dir.push(COMPONENT_TEXTURES_DIR);
        let textures = if std::fs::try_exists(&dir)
            .with_context(|| "Checking existence of component's textures directory.")?
        {
            TextureRepository::from_directory(&dir)?
        } else {
            TextureRepository::new()
        };
        assert!(dir.pop());

        dir.push(COMPONENT_CONFIGS_DIR);
        let configs = {
            let mut builder = ConfigRepositoryBuilder::new(&self.config_type_registry)?;
            if std::fs::try_exists(&dir)
                .with_context(|| "Checking existence of component's configs directory.")?
            {
                builder.load_directory(&self.config_type_registry, &dir)?;
            }
            builder.build(
                &self.config_type_registry,
                ComponentsRef {
                    indexer: &self.indexer,
                    app: app_comps,
                    shared: shared_comps,
                },
                ComponentPreConfigsRef::new(component_id, &textures),
            )?
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
        Ok(ComponentsChangedToken(()))
    }

    /// Loads specified directory subdirectories as components.
    pub fn load_each(
        &mut self,
        shared_comps: &mut SharedComponents,
        app_comps: &mut AppComponents,
        top_dir: &Path,
    ) -> Result<ComponentsChangedToken> {
        let changed_token = ComponentsChangedToken::new();
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
                changed_token.consume(self.load_single(
                    shared_comps,
                    app_comps,
                    label,
                    top_entry_path,
                )?);
            }
        }
        Ok(changed_token)
    }

    /// Finalizes initialization of some configs.
    /// Should runs every time components finished changing to make them ready.
    pub fn finalize(
        &self,
        changed: ComponentsChangedToken,
        shared_comps: &mut SharedComponents,
    ) -> Result<()> {
        std::mem::forget(changed);
        for f in self.config_type_registry.finalizers() {
            f(&self.indexer, shared_comps)?;
        }
        Ok(())
    }
}
