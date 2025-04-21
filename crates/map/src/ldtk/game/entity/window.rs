use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{game::entity::map::window::WindowComponent, generation::entity::window::WindowConfig};

impl WindowComponent {
    pub fn from_field(_: &EntityInstance) -> Self {
        Self {
            config: WindowConfig {},
        }
    }
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct WindowBundle {
    #[with(WindowComponent::from_field)]
    door: WindowComponent,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}
