use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use bevy_asset_loader::prelude::*;

/// Holds preloaded game-wide assets so level spawns can be fast.
#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "textures/Naginata/Naginata.aseprite")]
    pub player: Handle<Aseprite>,
    #[asset(path = "textures/Sohei/Sohei.aseprite")]
    pub sohei: Handle<Aseprite>,
    #[allow(dead_code)]
    #[asset(path = "textures/Tanegashima/Tanegashima.aseprite")]
    pub tanegashima: Handle<Aseprite>,
    #[allow(dead_code)]
    #[asset(path = "sounds/nature_sound/Ambiance_Forest_Birds_Loop_Stereo.wav")]
    pub nature_sound: Handle<AudioSource>,
}
