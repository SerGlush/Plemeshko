use std::{ffi::OsStr, path::PathBuf, time::Duration};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::serde_as;
use time::OffsetDateTime;

use crate::{
    params::SAVES_DIR,
    sim::{RawSimSnapshot, Sim, SimSnapshot},
};

use super::{components::ComponentsRef, serializable::Serializable, AppState};

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct SaveMetadata {
    #[serde(with = "time::serde::iso8601")]
    pub saved_date: OffsetDateTime,
    #[serde_as(as = "serde_with::DurationSeconds<f64>")]
    pub play_time: Duration,
}

pub fn saves() -> Result<Vec<(String, SaveMetadata)>> {
    let mut saves = Vec::new();
    std::fs::create_dir_all(SAVES_DIR)?;
    for dir_entry in std::fs::read_dir(SAVES_DIR)? {
        let dir_entry = dir_entry?;
        let entry_path = dir_entry.path();
        if entry_path.is_file() && entry_path.extension() == Some(OsStr::new("json")) {
            let file = std::fs::File::open(&entry_path)?;
            let reader = std::io::BufReader::new(file);
            let metadata = serde_json::de::from_reader(reader)?;
            let name = entry_path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            saves.push((name, metadata));
        }
    }
    Ok(saves)
}

fn save_path_from_name(name: &str) -> PathBuf {
    let mut path = PathBuf::from(SAVES_DIR);
    path.push(name);
    path.set_extension("json");
    path
}

pub fn save(app_st: &AppState) -> Result<()> {
    let Some(name) = app_st.session.as_ref() else {
        bail!("Current session name unknown");
    };
    let metadata = SaveMetadata {
        saved_date: time::OffsetDateTime::now_utc(),
        play_time: Duration::ZERO,
    };
    let sim = app_st
        .shared
        .sim
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .snapshot()
        .into_serializable(ComponentsRef {
            indexer: app_st.component_loader.indexer(),
            app: &app_st.components,
            shared: &app_st.shared.components.read().unwrap(),
        })?;
    let Value::Object(metadata) = serde_json::to_value(metadata)? else {
        bail!("Save metadata was not a json object when serialized");
    };
    let Value::Object(mut save) = serde_json::to_value(sim)? else {
        bail!("Sim snapshot was not a json object when serialized");
    };
    save.extend(metadata);
    let save = Value::Object(save);
    std::fs::create_dir_all(SAVES_DIR)?;
    let path = save_path_from_name(name);
    let mut file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(&mut file, &save)?;
    Ok(())
}

pub fn load(name: &str, app_st: &mut AppState) -> Result<()> {
    std::fs::create_dir_all(SAVES_DIR)?;
    let path = save_path_from_name(name);
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let sim: RawSimSnapshot = serde_json::de::from_reader(reader)?;
    let sim: SimSnapshot = Serializable::from_serializable(
        sim,
        ComponentsRef {
            indexer: app_st.component_loader.indexer(),
            app: &app_st.components,
            shared: &app_st.shared.components.read().unwrap(),
        },
    )?;
    *app_st.shared.sim.lock().unwrap() = Some(Sim::restore(
        &app_st.shared.components.read().unwrap(),
        sim,
    )?);
    Ok(())
}
