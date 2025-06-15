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
    systems::common::is_pos_in_radius,
};

pub fn test_follow(
    mut zombie_query: Query<&H1emuEntity, With<ZombieEntity>>,
    mut player_query: Query<&H1emuEntity, With<PlayerEntity>>,
) {
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
            obj.go_to(&args);
        }
    }
}

pub fn hostile_to_player_sys(
    mut hostile_query: Query<
        (&H1emuEntity, &Position, Entity),
        (With<HostileToPlayer>, Without<IsAttacking>, With<Alive>),
    >,
    mut all_positions_query: Query<
        (Entity, &Position, &H1emuEntity),
        (With<PlayerEntity>, With<Alive>),
    >,
    mut commands: Commands,
) {
    for (hostile_h1emu_ent, hostile_pos, hostile_ent) in &mut hostile_query {
        for (player_ent, player_pos, player_h1emu_ent) in &mut all_positions_query {
            // let hostile_pos = hostile_ent.get_position();
            if is_pos_in_radius(1.5, &player_pos, &hostile_pos) {
                // Just a quick test nothing fancy but even with 800 entities this run taking only
                // a microsec probably even less that's crazy
                let args = js_sys::Array::new();
                args.push(&JsValue::from_str("KnifeSlash"));
                hostile_h1emu_ent.play_animation(&args);
                let mut ec = commands.get_entity(hostile_ent).unwrap();
                let current_time = Utc::now().timestamp_millis();
                ec.insert(IsAttacking {
                    target: player_ent,
                    time_to_hit: current_time + 1000_i64,
                });
                break;
            }
        }
    }
}
pub fn attack_hit_sys(
    mut query: Query<(&IsAttacking, Entity, &H1emuEntity, &Position), With<Alive>>,
    pos_query: Query<(&Position, &CharacterId), With<Alive>>,
    mut commands: Commands,
) {
    let current_time = Utc::now().timestamp_millis();
    for (attack, attack_ent, attacker_h1emu_ent, attacker_pos) in &mut query {
        if current_time < attack.time_to_hit {
            continue;
        }
        let target_pos = pos_query.get(attack.target);

        if let Ok((target_pos, charid)) = target_pos {
            if is_pos_in_radius(1.5, attacker_pos, target_pos) {
                let args = js_sys::Array::new();
                let character_id_jsvalue: JsValue = charid.0.clone().into();
                args.push(&character_id_jsvalue);
                attacker_h1emu_ent.apply_damage(&args);
            }
        } else {
            log!("Failed to get target position, attack canceled");
        }

        commands.entity(attack_ent).remove::<IsAttacking>();
    }
}

pub fn coward_sys(
    mut coward_query: Query<(&H1emuEntity, &Position), (With<Coward>, With<Alive>)>,
    mut others_query: Query<&Position, (Without<Coward>, With<Alive>, Changed<Position>)>,
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

pub fn check_aliveness_sys(
    mut query: Query<(&H1emuEntity, Entity), With<Alive>>,

    mut commands: Commands,
) {
    for (h1emu_ent, ent) in &mut query {
        if !h1emu_ent.get_isAlive() {
            commands.entity(ent).remove::<Alive>();
            commands.entity(ent).insert(Dead());
        }
    }
}

pub fn check_player_revived_sys(
    mut query: Query<(&H1emuEntity, Entity), With<Dead>>,

    mut commands: Commands,
) {
    for (h1emu_ent, ent) in &mut query {
        if h1emu_ent.get_isAlive() {
            commands.entity(ent).remove::<Dead>();
            commands.entity(ent).insert(Alive());
        }
    }
}

pub fn carnivore_eating_sys(
    mut dead_query: Query<(&Position, Entity), (With<Dead>, With<PlayerEntity>)>,
    mut zombie_query: Query<
        (&H1emuEntity, &Position, Entity),
        (With<Carnivore>, With<Alive>, Without<Eating>, With<Hungry>),
    >,
    mut commands: Commands,
) {
    for (dead_pos, dead_ent) in &mut dead_query {
        for (h1emu_ent, zombie_pos, ent) in &mut zombie_query {
            if is_pos_in_radius(1.5, dead_pos, zombie_pos) {
                let args = js_sys::Array::new();
                args.push(&JsValue::from_str("Eating"));
                h1emu_ent.play_animation(&args);

                let current_time = Utc::now().timestamp_millis();
                commands.entity(ent).insert(Eating { time: current_time });
            }
        }
    }
}

pub fn finish_eating_sys(
    mut query: Query<(&H1emuEntity, Entity, &Eating, &mut HungerLevel), (With<Alive>)>,
    mut commands: Commands,
) {
    let current_time = Utc::now().timestamp_millis();
    for (h1emu_ent, ent, eating, mut hunger_level) in &mut query {
        if eating.time + 10000_i64 <= current_time {
            log!("finish eating");
            let args = js_sys::Array::new();
            args.push(&JsValue::from_str("EatingDone"));
            h1emu_ent.play_animation(&args);
            commands.entity(ent).remove::<Eating>();
            hunger_level.0 = 100;
        }
    }
}

pub fn hungry_sys(mut query: Query<(Entity, &HungerLevel), With<Alive>>, mut commands: Commands) {
    for (ent, hunger_level) in &mut query {
        if hunger_level.0 < 10 {
            commands.entity(ent).insert(Hungry());
        }
    }
}
pub fn remove_hungry_sys(
    mut query: Query<(Entity, &HungerLevel), (With<Alive>, With<Hungry>)>,
    mut commands: Commands,
) {
    for (ent, hunger_level) in &mut query {
        if hunger_level.0 > 50 {
            commands.entity(ent).remove::<Hungry>();
        }
    }
}
pub fn hunger_sys(
    mut query: Query<&mut HungerLevel, (With<Alive>)>,
    mut hunger_timer: ResMut<HungerTimer>,
) {
    let current_time = Utc::now().timestamp_millis();
    if hunger_timer.0 <= current_time {
        for mut hunger_level in &mut query {
            if hunger_level.0 > 0 {
                hunger_level.0 = hunger_level.0 - 1;
            }
        }
        hunger_timer.0 = current_time + 10_000;
    }
}
