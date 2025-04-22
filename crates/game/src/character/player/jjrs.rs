
use bevy::prelude::*;
use bevy_ggrs::GgrsConfig;
use ggrs::{Config, PlayerHandle};
use serde::{Deserialize, Serialize};



#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BoxInput(pub u16);

pub type BoxConfig = GgrsConfig<BoxInput>;