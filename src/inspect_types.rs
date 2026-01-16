use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;

fn inspect_frame(ase: Res<Assets<Aseprite>>) {
    use bevy_aseprite_ultra::prelude::*;

pub fn inspect(ase: Aseprite) {
    for (_, tag) in &ase.tags {
        let _ = tag.non_existent_field;
    }
}

}

