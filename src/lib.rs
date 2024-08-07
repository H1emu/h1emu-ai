use core::panic;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc,
};

use bevy_ecs::prelude::*;
use js_sys::{Function, Object, Reflect};
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
struct H1emuEntity(Arc<AtomicPtr<js_sys::Object>>);

impl H1emuEntity {
    fn call_method(&self, arg: &JsValue) {
        // Load the raw pointer
        let ptr = self.0.load(Ordering::SeqCst);

        // Check if the pointer is null
        if !ptr.is_null() {
            // Convert the raw pointer to a reference
            unsafe {
                let obj = &*ptr;

                // Ensure the conversion is valid
                if obj.is_object() {
                    let func: Function = Function::from(Reflect::get(&obj, &arg).unwrap());
                    if func.is_function() {
                        let args = js_sys::Array::new();
                        func.apply(obj, &args).unwrap();
                    } else {
                        log!("specified method doesn't exist")
                    }
                } else {
                    log!("The stored value is not an object.");
                }
            }
        } else {
            panic!("Null pointer encountered.");
        }
    }
}
#[derive(Component)]
struct ZombieEntity();
#[derive(Component)]
struct PlayerEntity();
#[derive(Component)]
struct DeerEntity();

#[derive(Bundle)]
struct EntityDefaultBundle {
    h1emu_id: H1emuEntity,
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

fn test_cb(mut query: Query<&CB>) {
    for cb in &mut query {
        cb.call_js_function(&JsValue::from_str(&"test"))
    }
}
fn test_obj(mut query: Query<&H1emuEntity>) {
    for obj in &mut query {
        let method = &JsValue::from_str(&"hurle");
        obj.call_method(method);
    }
}
#[wasm_bindgen]
pub struct AiManager {
    world: World,
    schedule: Schedule,
}
#[wasm_bindgen]
pub struct EntityFromJs {
    h1emu_id: js_sys::Object,
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
        h1emu_id: js_sys::Object,
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
        schedule.add_systems(test_cb);
        schedule.add_systems(test_obj);

        AiManager { world, schedule }
    }

    pub fn run(&mut self) {
        self.schedule.run(&mut self.world);
    }
    pub fn add_entity(&mut self, mut e: EntityFromJs) {
        let action_cb = Box::into_raw(Box::new(e.action_cb));
        let action_cb_ptr = Arc::new(AtomicPtr::new(action_cb));
        let h1emu_entity = Box::into_raw(Box::new(e.h1emu_id));
        let h1emu_entity_ptr = Arc::new(AtomicPtr::new(h1emu_entity));
        let mut entity = self.world.spawn(EntityDefaultBundle {
            h1emu_id: H1emuEntity(h1emu_entity_ptr),
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
