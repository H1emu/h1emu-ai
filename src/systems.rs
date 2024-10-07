use bevy_ecs::prelude::*;
use js_sys::Float32Array;
use wasm_bindgen::JsValue;

use crate::{
    components::{Coward, H1emuEntity, HostileToPlayer, PlayerEntity, Position, ZombieEntity},
    log,
};

pub fn track_positions(mut query: Query<(&H1emuEntity, &mut Position), With<PlayerEntity>>) {
    for (entity, mut position) in &mut query {
        let pos = entity.get_position();
        position.x = pos.x;
        position.y = pos.y;
        position.z = pos.z;
        // log!(player_position);
    }
}

pub fn test_follow(
    mut zombie_query: Query<&H1emuEntity, With<ZombieEntity>>,
    mut player_query: Query<&H1emuEntity, With<PlayerEntity>>,
) {
    let method = &JsValue::from_str("goTo");
    for obj in &mut zombie_query {
        for player in &mut player_query {
            let pos = player.get_position();
            let args = js_sys::Array::new();
            let jspa = js_sys::Array::new();
            jspa.push(&JsValue::from(pos.x));
            jspa.push(&JsValue::from(pos.y));
            jspa.push(&JsValue::from(pos.z));

            let js_pos = Float32Array::new(&jspa);
            args.push(&js_pos);
            obj.call_method(method, &args);
        }
    }
}
pub fn is_pos_in_radius(radius: f32, player_pos: &Position, enemi_pos: &Position) -> bool {
    let player_x = player_pos.x;
    let player_z = player_pos.z;
    let enemi_x = enemi_pos.x;
    let enemi_z = enemi_pos.z;
    (player_x - radius <= enemi_x && enemi_x <= player_x + radius)
        && (player_z - radius <= enemi_z && enemi_z <= player_z + radius)
}
pub fn hostile_to_player_sys(
    mut all_positions_query: Query<(&H1emuEntity, &Position), With<PlayerEntity>>,
    mut hostile_query: Query<(&H1emuEntity, &Position), With<HostileToPlayer>>,
) {
    for (hostile_ent, hostile_pos) in &mut hostile_query {
        for (player_ent, player_pos) in &mut all_positions_query {
            // let hostile_pos = hostile_ent.get_position();
            if is_pos_in_radius(2.0, &player_pos, &hostile_pos) {
                // Just a quick test nothing fancy but even with 800 entities this run taking only
                // a microsec probably even less that's crazy
                log!("attack");
                break;
            }
        }
    }
}

pub fn coward_sys(
    mut coward_query: Query<(&H1emuEntity, &Position), With<Coward>>,
    mut others_query: Query<&Position, Without<Coward>>,
) {
    for (coward_ent, coward_pos) in &mut coward_query {
        for other_pos in &mut others_query {
            if is_pos_in_radius(2.0, &other_pos, &coward_pos) {
                log!("i'm afraid");
                break;
            }
        }
    }
}
