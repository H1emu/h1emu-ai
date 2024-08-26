use bevy_ecs::{entity, prelude::*};
use js_sys::Float32Array;
use wasm_bindgen::JsValue;

use crate::{
    chunck_schemas::Triangle,
    components::{H1emuEntity, PlayerEntity, Position, ZombieEntity},
    log, NavDataRes,
};

pub fn test_follow(
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
pub fn track_players_pos(
    mut player_query: Query<(&H1emuEntity, &mut Position), With<PlayerEntity>>,
) {
    for (player, mut player_position) in &mut player_query {
        let pos = player.get_position();
        player_position.x = pos.x;
        player_position.y = pos.y;
        player_position.z = pos.z;
        // log!(player_position);
    }
}

fn get_polygon_from_pos(entity_position: &Position, polygons: &Vec<Triangle>) -> Option<i32> {
    let point = (entity_position.x, entity_position.z); // Project to 2D (ignoring y-axis)
    for p in polygons {
        if p.vertices[0] == entity_position.x as i32 {
            log!(p);
            return Some(p.vertices[0]);
        }
    }

    // for (index, polygon) in polygons.iter().enumerate() {
    //     // Convert 3D vertices to 2D points for the test
    //     let polygon_2d: Vec<(f32, f32)> = polygon.vertices
    //         .iter()
    //         .map(|v| (v.x, v.z))
    //         .collect();
    //
    //     if is_point_in_polygon(point, &polygon_2d) {
    //         return Some(index); // Return the index of the containing polygon
    //     }
    // }

    None // Return None if no polygon contains the point
}

pub fn get_player_polygon(
    mut player_query: Query<&Position, With<PlayerEntity>>,
    nav_data: Res<NavDataRes>,
) {
    for player_position in &mut player_query {
        let x: i32 = player_position.x as i32 / 256;
        let z: i32 = player_position.z as i32 / 256;
        log!(player_position);
        log!(format!("x : {}", (x)));
        log!(format!("z : {}", z));
        let chuncks = nav_data.0.cells.clone();
        let mut player_chunk = "chunck not found".to_owned();
        for c in chuncks {
            if c.x == x && c.y == z {
                player_chunk = format!("chunck found x {x} y {z}");
                let pos_from_poly = get_polygon_from_pos(
                    player_position,
                    &nav_data.0.polygons[c.start_poly_index as usize..c.end_poly_index as usize]
                        .to_vec(),
                );
                if pos_from_poly.is_some() {
                    log!("Polygon found !!!!!")
                } else {
                    log!("Didn't found polygon")
                }
            }
        }
        log!(player_chunk)
    }
}

