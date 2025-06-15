use bevy_ecs::prelude::*;
use chrono::Utc;
use wasm_bindgen::JsValue;

use crate::{
    components::{
        Alive, CharacterId, Cooldown, H1emuEntity, Position, Trap,
    },
    systems::common::is_pos_in_radius,
};

pub fn trap_sys(
    mut trap_query: Query<(&Trap, &Position, &H1emuEntity, &mut Cooldown)>,
    mut others_query: Query<(&Position, &CharacterId), (With<Alive>, Changed<Position>)>,
) {
    for (ent, pos, h1emu_ent, mut cooldown) in &mut trap_query {
        if cooldown.is_in_cooldown() {
            continue;
        }
        for (other_pos, other_h1emu_ent) in &mut others_query {
            if is_pos_in_radius(ent.0, other_pos, pos) {
                // TODO: store characterId directly
                let target_character_id = other_h1emu_ent.0.clone();
                let args = js_sys::Array::new();
                let character_id_jsvalue: JsValue = target_character_id.into();
                args.push(&character_id_jsvalue);
                cooldown.last = Utc::now().timestamp_millis();
                h1emu_ent.detonate(&args);
                break;
            }
        }
    }
}
