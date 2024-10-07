use std::sync::{atomic::AtomicPtr, Arc};

use bevy_ecs::prelude::*;
use components::{
    BearEntity, Coward, DeerEntity, EntityDefaultBundle, H1emuEntity, HostileToPlayer,
    PlayerEntity, WolfEntity, ZombieEntity,
};
use systems::{coward_sys, hostile_to_player_sys, test_follow, track_positions};
use wasm_bindgen::prelude::*;

mod components;
mod macros;
mod systems;

#[wasm_bindgen]
pub enum EntityType {
    Zombie,
    Player,
    Deer,
    Wolf,
    Bear,
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
        let world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(track_positions);
        schedule.add_systems(hostile_to_player_sys);
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
    pub fn add_entity(&mut self, e: js_sys::Object, entity_type: EntityType) {
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
            EntityType::Zombie => entity.insert((ZombieEntity {}, HostileToPlayer {})),
            EntityType::Wolf => entity.insert((WolfEntity {}, HostileToPlayer {})),
            EntityType::Bear => entity.insert((BearEntity {}, HostileToPlayer {})),
            EntityType::Deer => entity.insert((DeerEntity {}, Coward {})),
        };
    }
}
