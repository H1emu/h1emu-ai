use std::sync::{Arc, atomic::AtomicPtr};

use crate::systems::trap_sys;
use bevy_ecs::prelude::*;
use chrono::Utc;
use components::{
    Alive, BearEntity, Carnivore, CharacterId, Cooldown, Coward, Dead, DeerEntity, DefaultBundle,
    EntityDefaultBundle, H1emuEntity, HostileToPlayer, HungerLevel, PlayerEntity, Position, Trap,
    WolfEntity, ZombieEntity,
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
    pub fn initialize() -> AiManager {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        world.insert_resource(HungerTimer(Utc::now().timestamp_millis()));
        #[cfg(feature = "zombies")]
        {
            schedule.add_systems(hungry_sys);
            schedule.add_systems(remove_hungry_sys);
            schedule.add_systems(hunger_sys);
            schedule.add_systems(hostile_to_player_sys);
            schedule.add_systems(attack_hit_sys);
            schedule.add_systems(carnivore_eating_sys);
            schedule.add_systems(finish_eating_sys);
            schedule.add_systems(coward_sys);
        }
        #[cfg(feature = "traps")]
        {
            schedule.add_systems(trap_sys);
        }

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
    pub fn add_trap(&mut self, e: js_sys::Object, radius: f32, cooldown: i64) -> u64 {
        let h1emu_entity = Box::into_raw(Box::new(e));
        let h1emu_entity_ptr = Arc::new(AtomicPtr::new(h1emu_entity));
        let h1emu_entity_component = H1emuEntity(h1emu_entity_ptr);
        let position = h1emu_entity_component.get_position();
        let mut entity = self.world.spawn(DefaultBundle {
            h1emu_entity: h1emu_entity_component,
            position,
            ..Default::default()
        });
        entity.insert(Trap(radius));
        entity.insert(Cooldown {
            cooldown,
            ..Default::default()
        });
        entity.id().to_bits()
    }
}
