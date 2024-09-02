use std::{
    io::Cursor,
    sync::{atomic::AtomicPtr, Arc},
};

use bevy_ecs::prelude::*;
use binrw::BinReaderExt;
use chunck_schemas::NavData;
use components::{
    DeerEntity, EntityDefaultBundle, H1emuEntity, PlayerEntity, Position, ZombieEntity,
};
use lz4_flex::decompress_size_prepended;
use systems::{
    follow_breadscrum, get_target_breadscrum, track_players_pos, update_current_cell, zombie_hunt,
};
use wasm_bindgen::prelude::*;

mod chunck_schemas;
mod components;
mod macros;
mod systems;

#[wasm_bindgen]
pub struct EntityFromJs {
    h1emu_id: js_sys::Object,
    entity_type: EntityType,
}
#[wasm_bindgen]
impl EntityFromJs {
    #[wasm_bindgen(constructor)]
    pub fn new(entity_type: EntityType, h1emu_id: js_sys::Object) -> EntityFromJs {
        EntityFromJs {
            h1emu_id,
            entity_type,
        }
    }
}
#[wasm_bindgen]
pub enum EntityType {
    Zombie,
    Player,
    Deer,
}

#[wasm_bindgen]
pub struct Stats {
    pub zombies: u32,
    pub players: u32,
}

#[wasm_bindgen]
pub struct AiManager {
    world: World,
    schedule: Schedule,
}

#[derive(Resource)]
struct NavDataRes(NavData);
#[wasm_bindgen]
impl AiManager {
    #[wasm_bindgen(constructor)]
    pub fn initialize() -> AiManager {
        let world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(zombie_hunt);
        schedule.add_systems(get_target_breadscrum);
        schedule.add_systems(track_players_pos);
        schedule.add_systems(update_current_cell);
        schedule.add_systems(follow_breadscrum);

        AiManager { world, schedule }
    }

    pub fn load_nav_data(&mut self, nav_data_compressed: &[u8]) {
        log!("Start reading nav_data");
        let nav_data_uncompressed = decompress_size_prepended(&nav_data_compressed).unwrap();

        let nav_data: NavDataRes =
            NavDataRes(Cursor::new(nav_data_uncompressed).read_le().unwrap());
        log!("Finish reading nav_data");
        self.world.insert_resource(nav_data);
    }

    pub fn get_stats(&mut self) -> Stats {
        todo!()
    }

    pub fn run(&mut self) {
        self.schedule.run(&mut self.world);
    }
    pub fn add_entity(&mut self, e: EntityFromJs) {
        let h1emu_entity = Box::into_raw(Box::new(e.h1emu_id));
        let h1emu_entity_ptr = Arc::new(AtomicPtr::new(h1emu_entity));
        let h1emu_entity_component = H1emuEntity(h1emu_entity_ptr);
        let position = h1emu_entity_component.get_position();
        let mut entity = self.world.spawn(EntityDefaultBundle {
            h1emu_entity: h1emu_entity_component,
            position,
            ..Default::default()
        });
        match e.entity_type {
            EntityType::Player => entity.insert(PlayerEntity {}),
            EntityType::Zombie => entity.insert(ZombieEntity {}),
            EntityType::Deer => entity.insert(DeerEntity {}),
        };
    }
}
