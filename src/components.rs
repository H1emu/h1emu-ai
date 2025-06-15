use std::sync::{
        Arc,
        atomic::{AtomicPtr, Ordering},
    };

use bevy_ecs::prelude::*;
use chrono::Utc;
use js_sys::{Array, Float32Array, Function, Object, Reflect};
use once_cell::unsync::Lazy;
use wasm_bindgen::JsValue;

use crate::log;

pub struct Bindings {
    pub go_to: &'static str,
    pub apply_damage: &'static str,
    pub play_animation: &'static str,
    pub detonate: &'static str,
}
const BINDINGS: Bindings = Bindings {
    go_to: "goTo",
    apply_damage: "applyDamage",
    play_animation: "playAnimation",
    detonate: "detonate",
};

thread_local! {
    static IS_ALIVE_KEY: Lazy<JsValue> = Lazy::new(|| JsValue::from_str("isAlive"));
    static POSITION_KEY: Lazy<JsValue> = Lazy::new(|| JsValue::from_str("position"));
    static STATE_KEY: Lazy<JsValue> = Lazy::new(|| JsValue::from_str("state"));
    static CHARACTERID_KEY: Lazy<JsValue> = Lazy::new(|| JsValue::from_str("characterId"));
}
#[derive(Component, Default)]
pub struct H1emuEntity(pub Arc<AtomicPtr<js_sys::Object>>);
impl H1emuEntity {
    pub fn get_object(&self) -> Result<&Object, ()> {
        // Load the raw pointer
        let ptr = self.0.load(Ordering::Acquire);

        // Check if the pointer is null
        if !ptr.is_null() {
            // Convert the raw pointer to a reference
            unsafe {
                let obj = &*ptr;

                // Ensure the conversion is valid
                if obj.is_object() {
                    Ok(obj)
                } else {
                    log!("The stored value is not an object.");
                    Err(())
                }
            }
        } else {
            panic!("Null pointer encountered.");
        }
    }
    pub fn get_position(&self) -> Position {
        let position_js_value = STATE_KEY
            .with(|state_key| POSITION_KEY.with(|pos_key| self.get_property(&[state_key, pos_key])))
            .unwrap();
        let float32_array = Float32Array::from(position_js_value);

        let x = float32_array.get_index(0);
        let y = float32_array.get_index(1);
        let z = float32_array.get_index(2);

        Position { x, y, z }
    }
    pub fn get_characterId(&self) -> JsValue {
        CHARACTERID_KEY
            .with(|key| self.get_property(&[key]))
            .unwrap()
    }
    pub fn get_isAlive(&self) -> bool {
        let js_value = IS_ALIVE_KEY.with(|key| self.get_property(&[key])).unwrap();
        js_value.is_truthy()
    }
    fn get_property(&self, property_chain: &[&JsValue]) -> Result<JsValue, ()> {
        let mut current = match self.get_object() {
            Ok(obj) => JsValue::from(obj),
            _ => return Err(()),
        };

        for &prop in property_chain {
            let next = Reflect::get(&current, prop).map_err(|_| ())?;

            if next.is_undefined() {
                return Err(());
            }

            current = next;
        }

        Ok(current)
    }
    pub fn play_animation(&self, args: &Array) {
        let method = &JsValue::from_str(BINDINGS.play_animation);
        self.call_method(method, args);
    }
    pub fn detonate(&self, args: &Array) {
        let method = &JsValue::from_str(BINDINGS.detonate);
        self.call_method(method, args);
    }
    pub fn go_to(&self, args: &Array) {
        let method = &JsValue::from_str(BINDINGS.go_to);
        self.call_method(method, args);
    }
    pub fn apply_damage(&self, args: &Array) {
        let method = &JsValue::from_str(BINDINGS.apply_damage);
        self.call_method(method, args);
    }
    fn call_method(&self, method: &JsValue, args: &Array) {
        if let Ok(obj) = self.get_object() {
            if let Ok(reflect_value) = Reflect::get(obj, method) {
                let func: Function = Function::from(reflect_value);
                if func.is_function() {
                    let result = func.apply(obj, args);
                    if result.is_err() {
                        log!(format!("{:?}", result.unwrap_err()));
                    }
                } else {
                    log!("specified method doesn't exist");
                }
            } else {
                log!("reflected value doesn't exist");
                log!(format!("{:?}", Reflect::get(obj, method).unwrap_err()));
            }
        } else {
            log!("Object doesn't exist");
        }
    }
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component)]
pub struct HostileToPlayer();
#[derive(Component)]
pub struct Coward();

#[derive(Component)]
pub struct IsAttacking {
    pub target: Entity,
    pub time_to_hit: i64,
}

#[derive(Component, Clone)]
pub struct CharacterId(pub String);

#[derive(Component, Default)]
pub struct Alive();
#[derive(Component)]
pub struct Dead();

#[derive(Component)]
pub struct Eating {
    pub time: i64,
}
#[derive(Component, Default)]
pub struct HungerLevel(pub u8);
#[derive(Component)]
pub struct Hungry();
#[derive(Component)]
pub struct Carnivore();
#[derive(Component)]
pub struct Trap(pub f32);
#[derive(Component, Default)]
pub struct Cooldown {
    pub last: i64,
    pub cooldown: i64,
}
impl Cooldown {
    pub fn is_in_cooldown(&self) -> bool {
        let current_time = Utc::now().timestamp_millis();
        current_time < self.last + self.cooldown
    }
}
#[derive(Component)]
pub struct ZombieEntity();
#[derive(Component)]
pub struct PlayerEntity();
#[derive(Component)]
pub struct DeerEntity();
#[derive(Component)]
pub struct WolfEntity();
#[derive(Component)]
pub struct BearEntity();

#[derive(Bundle)]
pub struct EntityDefaultBundle {
    pub h1emu_entity: H1emuEntity,
    pub position: Position,
    pub character_id: CharacterId,
    pub alive: Alive,
}
#[derive(Bundle, Default)]
pub struct DefaultBundle {
    pub h1emu_entity: H1emuEntity,
    pub position: Position,
}
