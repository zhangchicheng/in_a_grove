use bevy::prelude::*;

#[derive(Component)]
pub struct OnMainMenuScreen;

#[derive(Component)]
pub struct SelectedOption;

#[derive(Component)]
pub enum MenuButtonAction {
    Play,
    Settings,
    Quit,
}
