use std::sync::{atomic::AtomicPtr, Arc};

use bevy_ecs::prelude::*;
use chrono::Utc;
use components::{
    BearEntity, Carnivore, Coward, DeerEntity, EntityDefaultBundle, H1emuEntity, HostileToPlayer,
    HungerLevel, PlayerEntity, WolfEntity, ZombieEntity,
};
use ressources::HungerTimer;
use systems::{
    attack_hit_sys, carnivore_eating_sys, check_aliveness_sys, check_player_revived_sys,
    coward_sys, finish_eating_sys, hostile_to_player_sys, hunger_sys, hungry_sys,
    remove_hungry_sys, test_follow, track_positions,
};
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
    pub fn initialize() -> AiManager {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        world.insert_resource(HungerTimer(Utc::now().timestamp_millis()));
        schedule.add_systems(track_positions);
        schedule.add_systems(check_aliveness_sys);
        schedule.add_systems(check_player_revived_sys);
        schedule.add_systems(hungry_sys);
        schedule.add_systems(remove_hungry_sys);
        schedule.add_systems(hunger_sys);
        schedule.add_systems(hostile_to_player_sys);
        schedule.add_systems(attack_hit_sys);
        schedule.add_systems(carnivore_eating_sys);
        schedule.add_systems(finish_eating_sys);
        schedule.add_systems(coward_sys);

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
    pub fn add_entity(&mut self, e: js_sys::Object, entity_type: EntityType) -> u64 {
        let h1emu_entity = Box::into_raw(Box::new(e));
        let h1emu_entity_ptr = Arc::new(AtomicPtr::new(h1emu_entity));
        let h1emu_entity_component = H1emuEntity(h1emu_entity_ptr);
        let position = h1emu_entity_component.get_position();
        let mut entity = self.world.spawn(EntityDefaultBundle {
            h1emu_entity: h1emu_entity_component,
            position,
            ..Default::default()
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
    pub fn remove_entity(&mut self, entity_id_bits: u64) {
        let e = Entity::from_bits(entity_id_bits);
        self.world.despawn(e);
    }
}
