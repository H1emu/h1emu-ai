use core::panic;
use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicPtr, Ordering},
        Arc,
    },
};

use bevy_ecs::{entity, prelude::*};
use binrw::BinReaderExt;
use chunck_schemas::{NavData, Triangle};
use components::{
    DeerEntity, EntityDefaultBundle, H1emuEntity, PlayerEntity, Position, ZombieEntity,
};
use js_sys::{Array, Float32Array, Function, Object, Reflect};
use lz4_flex::decompress_size_prepended;
use systems::{get_player_polygon, test_follow, track_players_pos};
use wasm_bindgen::prelude::*;
use web_sys::console;

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
        schedule.add_systems(test_follow);
        schedule.add_systems(track_players_pos);
        schedule.add_systems(get_player_polygon);

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
        let mut entity = self.world.spawn(EntityDefaultBundle {
            h1emu_entity: h1emu_entity_component,
            position: Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        });
        match e.entity_type {
            EntityType::Player => entity.insert(PlayerEntity {}),
            EntityType::Zombie => entity.insert(ZombieEntity {}),
            EntityType::Deer => entity.insert(DeerEntity {}),
        };
    }
}
