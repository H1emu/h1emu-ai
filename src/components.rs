use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc,
};

use bevy_ecs::{entity, prelude::*};
use js_sys::{Array, Float32Array, Function, Object, Reflect};
use wasm_bindgen::JsValue;

use crate::log;

#[derive(Component)]
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
    pub fn get_position(&self) -> Position {
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
    pub fn get_property(&self, property_chain: Vec<&JsValue>) -> Result<JsValue, ()> {
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
    pub fn call_method(&self, method: &JsValue, args: &Array) {
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
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component)]
pub struct ZombieEntity();
#[derive(Component)]
pub struct PlayerEntity();
#[derive(Component)]
pub struct DeerEntity();

#[derive(Bundle)]
pub struct EntityDefaultBundle {
    pub h1emu_entity: H1emuEntity,
    pub position: Position,
}
