use bevy::prelude::*;
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

    PointerPosition,
    PointerClick,
    
    SwitchLockMode,
    SwitchToUnlockMode,
    SwitchTargetPlayer,

    MoveCameraUp,
    MoveCameraDown,
    MoveCameraLeft,
    MoveCameraRight,
}

// Utility function to create the input map
pub fn get_input_map() -> InputMap<PlayerAction> {
    let mut map = InputMap::new([
        (PlayerAction::MoveUp, KeyCode::KeyW),
        (PlayerAction::MoveCameraUp, KeyCode::ArrowUp),
        (PlayerAction::MoveDown, KeyCode::KeyS),
        (PlayerAction::MoveCameraDown, KeyCode::ArrowDown),
        (PlayerAction::MoveLeft, KeyCode::KeyA),
        (PlayerAction::MoveCameraLeft, KeyCode::ArrowLeft),
        (PlayerAction::MoveRight, KeyCode::KeyD),
        (PlayerAction::MoveCameraRight, KeyCode::ArrowRight),
        (PlayerAction::Interaction, KeyCode::KeyH)
    ]);
    // Add gamepad support if needed
    map.insert(PlayerAction::MoveUp, GamepadButton::DPadUp);
    map.insert(PlayerAction::MoveDown, GamepadButton::DPadDown);
    map.insert(PlayerAction::MoveLeft, GamepadButton::DPadLeft);
    map.insert(PlayerAction::MoveRight, GamepadButton::DPadRight);
    map.insert(PlayerAction::Interaction, GamepadButton::North);
    // Add more bindings...

    map.insert(PlayerAction::SwitchLockMode, KeyCode::KeyP);
    map.insert(PlayerAction::SwitchToUnlockMode, KeyCode::KeyO);

    map.with_dual_axis(PlayerAction::Pan, GamepadStick::LEFT)

}


