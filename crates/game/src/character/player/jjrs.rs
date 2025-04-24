
use bevy_ggrs::GgrsConfig;
use bevy_matchbox::prelude::PeerId;
use serde::{Deserialize, Serialize};


#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BoxInput(pub u16);

pub type BoxConfig = GgrsConfig<BoxInput>;
pub type PeerConfig = GgrsConfig<BoxInput, PeerId>;