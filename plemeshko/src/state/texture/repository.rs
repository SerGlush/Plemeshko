use std::{io::Read, path::Path};

use anyhow::{anyhow, Context, Result};
use egui_extras::RetainedImage;

use crate::state::{label_factory::LabelFactory, raw_indexer::RawIndexer};

use super::{RawTextureId, TextureId, TextureLabel};

#[derive(Default)]
pub struct TextureRepository {
    textures: Vec<RetainedImage>,
    indexer: RawIndexer<String, RawTextureId>,
}

impl TextureRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_directory<P: AsRef<Path>>(directory: P) -> Result<Self> {
        let mut repo = Self::new();
        let mut lf = LabelFactory::new();
        repo.load_directory(&mut Vec::new(), &mut lf, directory)?;
        Ok(repo)
    }

    pub fn get(&self, id: TextureId) -> Option<&RetainedImage> {
        let index: usize = id.0.try_into().unwrap();
        self.textures.get(index)
    }

    pub fn id(&self, label: &TextureLabel) -> Result<TextureId> {
        self.indexer.id(&label.0).map(TextureId)
    }

    pub fn id_from_raw(&self, label: &str) -> Result<TextureId> {
        self.indexer.id(label).map(TextureId)
    }

    fn load_directory<P: AsRef<Path>>(
        &mut self,
        buffer: &mut Vec<u8>,
        lf: &mut LabelFactory,
        directory: P,
    ) -> Result<()> {
        let dir_entries = std::fs::read_dir(&directory).with_context(|| {
            format!(
                "Loading textures from directory: {}",
                directory.as_ref().display()
            )
        })?;
        for dir_entry in dir_entries {
            let dir_entry_path = dir_entry?.path();
            let dir_entry_name = dir_entry_path
                .file_stem()
                .ok_or_else(|| {
                    anyhow!(
                        "Can't get stem of the textures dir '{}' entry: {}",
                        directory.as_ref().display(),
                        dir_entry_path.display()
                    )
                })?
                .to_string_lossy();
            if dir_entry_path.is_dir() {
                lf.with_branch(&dir_entry_name, |lf| {
                    self.load_directory(buffer, lf, &dir_entry_path)
                })?;
            } else if dir_entry_path.is_file() {
                let label = lf.create(&dir_entry_name);
                self.load_file(buffer, label, &dir_entry_path)?;
            }
        }
        Ok(())
    }

    fn load_file<P: AsRef<Path>>(
        &mut self,
        buffer: &mut Vec<u8>,
        label: String,
        file: P,
    ) -> Result<()> {
        let index: usize = self.indexer.create_id(label.clone())?.try_into().unwrap();
        assert_eq!(index, self.textures.len());
        buffer.clear();
        std::fs::File::open(file)?.read_to_end(buffer)?;
        self.textures.push(
            RetainedImage::from_image_bytes(label, buffer.as_mut()).map_err(anyhow::Error::msg)?,
        );
        Ok(())
    }
}
