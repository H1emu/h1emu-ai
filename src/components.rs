use std::{
    default,
    sync::{
        Arc,
        atomic::{AtomicPtr, Ordering},
    },
};

use bevy_ecs::prelude::*;
use chrono::Utc;
use js_sys::{Array, Boolean, Float32Array, Function, JsString, Object, Reflect};
use wasm_bindgen::JsValue;

use crate::{error, log};

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

#[derive(Component, Default)]
pub struct H1emuEntity(pub Arc<AtomicPtr<js_sys::Object>>);
impl H1emuEntity {
    pub fn get_object(&self) -> Result<&Object, ()> {
        // Load the raw pointer
        let ptr = self.0.load(Ordering::SeqCst);

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
        let position_js_value = self
            .get_property(vec![
                &JsValue::from_str("state"),
                &JsValue::from_str("position"),
            ])
            .unwrap();

        let float32_array = Float32Array::from(position_js_value);

        let vec = float32_array.to_vec();

        Position {
            x: vec[0],
            y: vec[1],
            z: vec[2],
        }
    }
    pub fn get_characterId(&self) -> String {
        let js_value = self
            .get_property(vec![&JsValue::from_str("characterId")])
            .unwrap();

        JsString::from(js_value).into()
    }
    pub fn get_isAlive(&self) -> bool {
        let js_value = self
            .get_property(vec![&JsValue::from_str("isAlive")])
            .unwrap();

        js_sys::Boolean::from(js_value).into()
    }
    fn get_property(&self, property_chain: Vec<&JsValue>) -> Result<JsValue, ()> {
        let mut current_obj = self.get_object().unwrap().to_owned();
        for property_name in property_chain {
            let property = Reflect::get(&current_obj, property_name).unwrap();
            if property.is_undefined() {
                error!(format!(
                    "specified property {:?} doesn't exist",
                    property_name
                ));
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
    pub target_character_id: String,
    pub time_to_hit: i64,
}

#[derive(Component, Clone)]
pub struct CharacterId(String);

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
        if current_time < self.last + self.cooldown {
            true
        } else {
            false
        }
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

#[derive(Bundle, Default)]
pub struct EntityDefaultBundle {
    pub h1emu_entity: H1emuEntity,
    pub position: Position,
    pub alive: Alive,
}
#[derive(Bundle, Default)]
pub struct DefaultBundle {
    pub h1emu_entity: H1emuEntity,
    pub position: Position,
}
