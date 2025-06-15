use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query},
};

use crate::{
    components::{DespawnCooldown, H1emuEntity, Position},
    log,
};

pub fn is_pos_in_radius(radius: f32, player_pos: &Position, enemi_pos: &Position) -> bool {
    let player_x = player_pos.x;
    let player_z = player_pos.z;
    let enemi_x = enemi_pos.x;
    let enemi_z = enemi_pos.z;
    (player_x - radius <= enemi_x && enemi_x <= player_x + radius)
        && (player_z - radius <= enemi_z && enemi_z <= player_z + radius)
}
pub fn despawn_inactive(
    query: Query<(Entity, &H1emuEntity, &DespawnCooldown)>,
    mut commands: Commands,
) {
    for (e, h, cooldown) in query {
        if cooldown.has_expired() {
            log!("cooldown hit");
            h.destroy();
            if let Ok(mut e_cmds) = commands.get_entity(e) {
                e_cmds.despawn();
            }
        }
    }
}
