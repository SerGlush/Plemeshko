use serde::{Deserialize, Serialize};

use crate::state::components::ComponentId;

pub(super) type RawTextureId = u32;

#[derive(Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct TextureLabel(String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureId(pub(super) RawTextureId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FatTextureId(pub ComponentId, pub TextureId);
