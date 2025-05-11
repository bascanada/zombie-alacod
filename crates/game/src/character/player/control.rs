use bevy::{input::keyboard::Key, prelude::*};
use leafwing_input_manager::prelude::*;

// === Leafwing Input Actions ===
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    #[actionlike(DualAxis)]
    Pan,

    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,

    Interaction,

    SwitchWeapon,

    Reload,

    PointerPosition,
    PointerClick,
}

// Utility function to create the input map
pub fn get_input_map() -> InputMap<PlayerAction> {
    let mut map = InputMap::new([
        (PlayerAction::MoveUp, KeyCode::KeyW),
        (PlayerAction::MoveUp, KeyCode::ArrowUp),
        (PlayerAction::MoveDown, KeyCode::KeyS),
        (PlayerAction::MoveDown, KeyCode::ArrowDown),
        (PlayerAction::MoveLeft, KeyCode::KeyA),
        (PlayerAction::MoveLeft, KeyCode::ArrowLeft),
        (PlayerAction::MoveRight, KeyCode::KeyD),
        (PlayerAction::MoveRight, KeyCode::ArrowRight),
        (PlayerAction::Interaction, KeyCode::KeyH),
        (PlayerAction::SwitchWeapon, KeyCode::Tab),
        (PlayerAction::Reload, KeyCode::KeyR)
    ]);
    // Add gamepad support if needed
    map.insert(PlayerAction::MoveUp, GamepadButton::DPadUp);
    map.insert(PlayerAction::MoveDown, GamepadButton::DPadDown);
    map.insert(PlayerAction::MoveLeft, GamepadButton::DPadLeft);
    map.insert(PlayerAction::MoveRight, GamepadButton::DPadRight);
    map.insert(PlayerAction::Interaction, GamepadButton::North);
    map.insert(PlayerAction::Reload, GamepadButton::West);
    // Add more bindings...
    map.insert(PlayerAction::PointerClick, MouseButton::Left);

    map.with_dual_axis(PlayerAction::Pan, GamepadStick::LEFT)

}


