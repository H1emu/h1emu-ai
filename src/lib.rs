use core::panic;
use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicPtr, Ordering},
        Arc,
    },
};

use bevy_ecs::prelude::*;
use binrw::BinReaderExt;
use chunck_schemas::ChunkData;
use js_sys::{Array, Float32Array, Function, Object, Reflect};
use lz4_flex::decompress_size_prepended;
use wasm_bindgen::prelude::*;
use web_sys::console;

mod chunck_schemas;

macro_rules! log {
    ($($t:tt)*) => {
        console::log_1(&format!("{:?}",$($t)*).into())
    };
}

#[derive(Component)]
struct H1emuEntity(Arc<AtomicPtr<js_sys::Object>>);

impl H1emuEntity {
    fn get_object(&self) -> Result<&Object, ()> {
        // Load the raw pointer
        let ptr = self.0.load(Ordering::SeqCst);

        // Check if the pointer is null
        if !ptr.is_null() {
            // Convert the raw pointer to a reference
            unsafe {
                let obj = &*ptr;

                // Ensure the conversion is valid
                if obj.is_object() {
                    return Ok(obj);
                } else {
                    log!("The stored value is not an object.");
                    Err(())
                }
            }
        } else {
            panic!("Null pointer encountered.");
        }
    }
    fn get_position(&self) -> Position {
        let position_js_value = self
            .get_property(vec![
                &JsValue::from_str("state"),
                &JsValue::from_str("position"),
            ])
            .unwrap();

        let float32_array = Float32Array::from(position_js_value);

        let vec = float32_array.to_vec();

        return Position {
            x: vec[0],
            y: vec[1],
            z: vec[2],
        };
    }
    fn get_property(&self, property_chain: Vec<&JsValue>) -> Result<JsValue, ()> {
        let mut current_obj = self.get_object().unwrap().to_owned();
        for property in property_chain {
            let property = Reflect::get(&current_obj, &property).unwrap();
            if property.is_undefined() {
                log!("specified property doesn't exist");
                break;
                // Err(())
            } else {
                current_obj = Object::from(property);
            }
        }
        if !current_obj.is_undefined() {
            Ok(JsValue::from(current_obj))
        } else {
            Err(())
        }
    }
    fn call_method(&self, method: &JsValue, args: &Array) {
        let obj = self.get_object().unwrap();
        let func: Function = Function::from(Reflect::get(&obj, &method).unwrap());
        if func.is_function() {
            func.apply(obj, &args).unwrap();
        } else {
            log!("specified method doesn't exist")
        }
    }
}

#[derive(Component, Debug)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component)]
struct ZombieEntity();
#[derive(Component)]
struct PlayerEntity();
#[derive(Component)]
struct DeerEntity();

#[derive(Bundle)]
struct EntityDefaultBundle {
    h1emu_entity: H1emuEntity,
    position: Position,
}

fn test_follow(
    mut zombie_query: Query<&H1emuEntity, With<ZombieEntity>>,
    mut player_query: Query<&H1emuEntity, With<PlayerEntity>>,
) {
    let method = &JsValue::from_str(&"goTo");
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
fn track_players_pos(mut player_query: Query<(&H1emuEntity, &mut Position), With<PlayerEntity>>) {
    for (player, mut player_position) in &mut player_query {
        let pos = player.get_position();
        player_position.x = pos.x;
        player_position.y = pos.y;
        player_position.z = pos.z;
        // log!(player_position);
    }
}

fn get_player_polygon(
    mut player_query: Query<&Position, With<PlayerEntity>>,
    nav_data: Res<NavData>,
) {
    for player_position in &mut player_query {
        log!(player_position);
        todo!()
    }
}
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
struct NavData(ChunkData);
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

        let nav_data: NavData = NavData(Cursor::new(nav_data_uncompressed).read_le().unwrap());
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
