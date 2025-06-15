use std::{
    default,
    i64::MAX,
    sync::{Arc, atomic::AtomicPtr},
};

use crate::{
    components::DespawnCooldown,
    systems::{
        attack_hit_sys, carnivore_eating_sys, coward_sys, despawn_inactive, finish_eating_sys,
        hostile_to_player_sys, hunger_sys, hungry_sys, remove_hungry_sys, trap_sys,
    },
};
use bevy_ecs::prelude::*;
use chrono::Utc;
use components::{
    Alive, BearEntity, Carnivore, CharacterId, Coward, Dead, DeerEntity, DefaultBundle,
    EntityDefaultBundle, H1emuEntity, HostileToPlayer, HungerLevel, PlayerEntity, Position, Trap,
    TrapsCooldown, WolfEntity, ZombieEntity,
};
use ressources::HungerTimer;
use wasm_bindgen::prelude::*;

mod components;
mod macros;
mod ressources;
mod systems;

#[wasm_bindgen]
pub enum EntityType {
    Zombie,
    Player,
    Deer,
    Wolf,
    Bear,
    Screamer,
}

#[wasm_bindgen]
pub struct Stats {
    pub entities: u32,
}

#[wasm_bindgen]
pub struct AiManager {
    world: World,
    schedule: Schedule,
}

#[wasm_bindgen]
impl AiManager {
    #[wasm_bindgen(constructor)]
    pub fn initialize(allow_zombies: Option<bool>) -> AiManager {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        world.insert_resource(HungerTimer(Utc::now().timestamp_millis()));
        if allow_zombies.is_some() && allow_zombies.unwrap() {
            schedule.add_systems(hungry_sys);
            schedule.add_systems(remove_hungry_sys);
            schedule.add_systems(hunger_sys);
            schedule.add_systems(hostile_to_player_sys);
            schedule.add_systems(attack_hit_sys);
            schedule.add_systems(carnivore_eating_sys);
            schedule.add_systems(finish_eating_sys);
            schedule.add_systems(coward_sys);
        }
        schedule.add_systems(trap_sys);
        schedule.add_systems(despawn_inactive);

        log!("h1emu-ai in debug mode");
        AiManager { world, schedule }
    }

    pub fn get_stats(&mut self) -> Stats {
        Stats {
            entities: self.world.entities().len(),
        }
    }

    pub fn run(&mut self) {
        self.schedule.run(&mut self.world);
    }
    pub fn update_pos(&mut self, entity_id: u64, position: Vec<f32>) {
        let e = Entity::from_bits(entity_id);

        let mut position_component = self.world.get_mut::<Position>(e).unwrap();
        position_component.x = position[0];
        position_component.y = position[1];
        position_component.z = position[2];
    }
    pub fn add_entity(&mut self, e: js_sys::Object, entity_type: EntityType) -> u64 {
        let h1emu_entity = Box::into_raw(Box::new(e));
        let h1emu_entity_ptr = Arc::new(AtomicPtr::new(h1emu_entity));
        let h1emu_entity_component = H1emuEntity(h1emu_entity_ptr);
        let position = h1emu_entity_component.get_position();
        let charid = h1emu_entity_component
            .get_characterId()
            .as_string()
            .unwrap();
        let mut entity = self.world.spawn(EntityDefaultBundle {
            h1emu_entity: h1emu_entity_component,
            position,
            character_id: CharacterId(charid),
            alive: Alive(),
        });
        match entity_type {
            EntityType::Player => entity.insert(PlayerEntity {}),
            EntityType::Zombie => entity.insert((
                ZombieEntity {},
                HostileToPlayer {},
                Carnivore {},
                HungerLevel(0),
            )),
            EntityType::Screamer => entity.insert((ZombieEntity {}, HostileToPlayer {})),
            EntityType::Wolf => entity.insert((WolfEntity {}, HostileToPlayer {})),
            EntityType::Bear => entity.insert((BearEntity {}, HostileToPlayer {})),
            EntityType::Deer => entity.insert((DeerEntity {}, Coward {})),
        };
        entity.id().to_bits()
    }
    pub fn entity_dead(&mut self, entity_id: u64) {
        let e = Entity::from_bits(entity_id);
        self.world.entity_mut(e).remove::<Alive>();
        self.world.entity_mut(e).insert(Dead());
    }
    pub fn entity_alive(&mut self, entity_id: u64) {
        let e = Entity::from_bits(entity_id);
        self.world.entity_mut(e).remove::<Dead>();
        self.world.entity_mut(e).insert(Alive());
    }
    pub fn remove_entity(&mut self, entity_id_bits: u64) {
        let e = Entity::from_bits(entity_id_bits);
        self.world.despawn(e);
    }
    pub fn add_trap(
        &mut self,
        e: js_sys::Object,
        radius: f32,
        trigger_cooldown: i64,
        despawn_cooldown: Option<i64>,
    ) -> u64 {
        let h1emu_entity = Box::into_raw(Box::new(e));
        let h1emu_entity_ptr = Arc::new(AtomicPtr::new(h1emu_entity));
        let h1emu_entity_component = H1emuEntity(h1emu_entity_ptr);
        let position = h1emu_entity_component.get_position();
        let mut entity = self.world.spawn(DefaultBundle {
            h1emu_entity: h1emu_entity_component,
            position,
        });
        entity.insert(Trap(radius));
        entity.insert(TrapsCooldown {
            cooldown: trigger_cooldown,
            ..Default::default()
        });
        if let Some(despawn_cooldown) = despawn_cooldown {
            log!("spawned with cooldown");
            entity.insert(DespawnCooldown::new(despawn_cooldown));
        }
        entity.id().to_bits()
    }
}
