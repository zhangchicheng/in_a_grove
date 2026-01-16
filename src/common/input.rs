use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Jump,
    Attack1,
    Attack2,
    Attack3,
    ThrowSpear,
    Parry,
    Menu,  // Added for "M for Menu"
    Pause, // Added for "Escape to Pause"
}

pub fn default_input_map() -> InputMap<PlayerAction> {
    use KeyCode::*;

    let mut map = InputMap::default();

    // Movement
    map.insert(PlayerAction::MoveLeft, ArrowLeft);
    map.insert(PlayerAction::MoveLeft, KeyA);
    map.insert(PlayerAction::MoveRight, ArrowRight);
    map.insert(PlayerAction::MoveRight, KeyD);
    map.insert(PlayerAction::MoveUp, ArrowUp);
    map.insert(PlayerAction::MoveUp, KeyW);
    map.insert(PlayerAction::MoveDown, ArrowDown);
    map.insert(PlayerAction::MoveDown, KeyS);

    // Actions
    map.insert(PlayerAction::Jump, Space);
    map.insert(PlayerAction::Attack1, KeyJ);
    map.insert(PlayerAction::Attack2, KeyK);
    map.insert(PlayerAction::Attack3, KeyL);
    map.insert(PlayerAction::ThrowSpear, KeyU);
    map.insert(PlayerAction::Parry, KeyI);
    map.insert(PlayerAction::Menu, KeyM);
    map.insert(PlayerAction::Pause, Escape);

    map
}

/// Resource to hold the player's input map so it persists across state changes
#[derive(Resource, Clone, Deref, DerefMut)]
pub struct GameInputMap(pub InputMap<PlayerAction>);

impl Default for GameInputMap {
    fn default() -> Self {
        GameInputMap(default_input_map())
    }
}
