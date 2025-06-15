use bevy_ecs::prelude::*;
use chrono::Utc;
use js_sys::{Float32Array, Math::log};
use wasm_bindgen::JsValue;

use crate::{
    components::{
        Alive, Carnivore, CharacterId, Cooldown, Coward, Dead, Eating, H1emuEntity,
        HostileToPlayer, HungerLevel, Hungry, IsAttacking, PlayerEntity, Position, Trap,
        ZombieEntity,
    },
    error, log,
    ressources::HungerTimer,
};

pub fn is_pos_in_radius(radius: f32, player_pos: &Position, enemi_pos: &Position) -> bool {
    let player_x = player_pos.x;
    let player_z = player_pos.z;
    let enemi_x = enemi_pos.x;
    let enemi_z = enemi_pos.z;
    (player_x - radius <= enemi_x && enemi_x <= player_x + radius)
        && (player_z - radius <= enemi_z && enemi_z <= player_z + radius)
}
