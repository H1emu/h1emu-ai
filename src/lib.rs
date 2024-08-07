use core::panic;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc,
};

use bevy_ecs::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::console;

macro_rules! log {
    ($($t:tt)*) => {
        console::log_1(&format!("{}",$($t)*).into())
    };
}

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

#[derive(Bundle)]
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
    fn call_js_function(&self, arg: &JsValue) {
        // Load the raw pointer
        let ptr = self.0.load(Ordering::SeqCst);

        // Check if the pointer is null
        if !ptr.is_null() {
            // Convert the raw pointer to a reference
            unsafe {
                let func = &*ptr;

                // Ensure the conversion is valid
                if func.is_function() {
                    log!("Function detected.");

                    // Call the JavaScript function
                    func.call1(&JsValue::NULL, arg).unwrap();
                } else {
                    log!("The stored value is not a function.");
                }
            }
        } else {
            panic!("Null pointer encountered.");
        }
    }
}

fn movement(mut query: Query<&CB>) {
    for cb in &mut query {
        cb.call_js_function(&JsValue::from_str(&"test"))
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
        let action_cb = Box::into_raw(Box::new(e.action_cb));
        let action_cb_ptr = Arc::new(AtomicPtr::new(action_cb));
        let mut entity = self.world.spawn(EntityDefaultBundle {
            h1emu_id: H1emuId(e.h1emu_id),
            position: Position(e.position),
            velocity: Velocity(e.velocity),
            cb: CB(action_cb_ptr),
        });
        match e.entity_type {
            EntityType::Player => entity.insert(PlayerEntity {}),
            EntityType::Zombie => entity.insert(ZombieEntity {}),
            EntityType::Deer => entity.insert(DeerEntity {}),
        };
    }
}
