use core::panic;
use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc,
};

use bevy_ecs::prelude::*;
use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::prelude::*;
use web_sys::console;

macro_rules! log {
    ($($t:tt)*) => {
        console::log_1(&format!("{}",$($t)*).into())
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
    fn get_position(&self) -> JsValue {
        let position_js_value = self
            .get_property(vec![
                &JsValue::from_str("state"),
                &JsValue::from_str("position"),
            ])
            .unwrap();

        return position_js_value;
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
#[derive(Component)]
struct ZombieEntity();
#[derive(Component)]
struct PlayerEntity();
#[derive(Component)]
struct DeerEntity();

#[derive(Bundle)]
struct EntityDefaultBundle {
    h1emu_entity: H1emuEntity,
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
            args.push(&pos);
            obj.call_method(method, &args);
        }
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
impl AiManager {
    #[wasm_bindgen(constructor)]
    pub fn initialize() -> AiManager {
        let world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(test_follow);

        AiManager { world, schedule }
    }

    pub fn run(&mut self) {
        self.schedule.run(&mut self.world);
    }
    pub fn add_entity(&mut self, e: EntityFromJs) {
        let h1emu_entity = Box::into_raw(Box::new(e.h1emu_id));
        let h1emu_entity_ptr = Arc::new(AtomicPtr::new(h1emu_entity));
        let mut entity = self.world.spawn(EntityDefaultBundle {
            h1emu_entity: H1emuEntity(h1emu_entity_ptr),
        });
        match e.entity_type {
            EntityType::Player => entity.insert(PlayerEntity {}),
            EntityType::Zombie => entity.insert(ZombieEntity {}),
            EntityType::Deer => entity.insert(DeerEntity {}),
        };
    }
}
