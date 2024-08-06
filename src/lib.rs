use core::panic;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc, Mutex,
};

use bevy_ecs::prelude::*;
use wasm_bindgen::prelude::*;
#[derive(Component)]
struct Position(Vec<f32>);

#[derive(Component)]
struct H1emuId(String);
#[derive(Component)]
struct ZombieEntity();
#[derive(Component)]
struct PlayerEntity();
#[derive(Component)]
struct DeerEntity();

#[derive(Component)]
struct EntityDefaultBundle {
    h1emu_id: H1emuId,
    position: Position,
    velocity: Velocity,
    cb: CB,
}
#[derive(Component)]
struct Velocity(Vec<f32>);

#[derive(Component)]
struct CB(Arc<AtomicPtr<js_sys::Function>>);

impl CB {
    // Safely call the JavaScript function stored in the AtomicPtr
    fn call_js_function(&self, arg: &JsValue) {
        panic!("fuck");
        // Load the raw pointer
        let ptr = self.0.load(Ordering::SeqCst);

        // Check if the pointer is null
        if !ptr.is_null() {
            // Convert the raw pointer to a reference
            unsafe {
                let js_func: &js_sys::Function = &*ptr;
                // Call the JavaScript function
                js_func.call1(&JsValue::NULL, arg).unwrap();
            }
        } else {
            panic!("fuck")
        }
    }
}

// This system moves each entity with a Position and Velocity component
fn movement(mut query: Query<&CB>) {
    // ZEBI pk la query est viiiide
    query.get_single().unwrap();

    for cb in &mut query {
        panic!("fuck");
        cb.call_js_function(&JsValue::from_str(&"hey"))
    }
}
#[wasm_bindgen]
pub struct AiManager {
    world: World,
    schedule: Schedule,
}
#[wasm_bindgen]
pub struct EntityFromJs {
    h1emu_id: String,
    entity_type: EntityType,
    position: Vec<f32>,
    velocity: Vec<f32>,
    action_cb: js_sys::Function,
}
#[wasm_bindgen]
impl EntityFromJs {
    #[wasm_bindgen(constructor)]
    pub fn new(
        entity_type: EntityType,
        position: Vec<f32>,
        velocity: Vec<f32>,
        action_cb: js_sys::Function,
        h1emu_id: String,
    ) -> EntityFromJs {
        EntityFromJs {
            h1emu_id,
            entity_type,
            position,
            velocity,
            action_cb,
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
impl AiManager {
    #[wasm_bindgen(constructor)]
    pub fn initialize() -> AiManager {
        let world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(movement);

        AiManager { world, schedule }
    }

    pub fn run(&mut self) {
        self.schedule.run(&mut self.world);
    }
    pub fn add_entity(&mut self, mut e: EntityFromJs) {
        let mut entity = self.world.spawn(EntityDefaultBundle {
            h1emu_id: H1emuId(e.h1emu_id),
            position: Position(e.position),
            velocity: Velocity(e.velocity),
            cb: CB(Arc::new(AtomicPtr::new(&mut e.action_cb))),
        });
        match e.entity_type {
            EntityType::Player => entity.insert(PlayerEntity {}),
            EntityType::Zombie => entity.insert(ZombieEntity {}),
            EntityType::Deer => entity.insert(DeerEntity {}),
        };
    }
}
