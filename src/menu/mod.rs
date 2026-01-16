use crate::states::GameState;
use bevy::prelude::*;

mod components;
mod systems;

use crate::common::SelectedButtonIndex;
use systems::*;

/// System sets for menu-related systems
///
/// Used in menu.rs with .chain() to enforce: Interaction → Actions
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuSet {
    /// Button interaction handling (detect clicks, update button visuals)
    /// Must run first to detect user interactions
    Interaction,
    /// Menu action processing (respond to button presses, change screens)
    /// Depends on Interaction detecting which button was clicked
    Actions,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        // Configure system set ordering
        app.configure_sets(
            Update,
            (MenuSet::Interaction, MenuSet::Actions)
                .chain()
                .run_if(in_state(GameState::Menu)),
        );

        app.init_resource::<SelectedButtonIndex>()
            .add_systems(OnEnter(GameState::Menu), menu_setup)
            .add_systems(
                Update,
                (input_navigation, button_interaction_system).in_set(MenuSet::Interaction),
            )
            .add_systems(Update, menu_action.in_set(MenuSet::Actions));
    }
}
