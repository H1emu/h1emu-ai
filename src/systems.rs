use std::fmt::format;

use bevy_ecs::prelude::*;
use js_sys::Float32Array;
use wasm_bindgen::JsValue;

use crate::{
    chunck_schemas::{Node, Triangle},
    components::{CurrentCell, H1emuEntity, PlayerEntity, Position, ZombieEntity},
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

pub fn update_current_cell(
    mut query: Query<(&mut CurrentCell, &Position)>,
    nav_data: Res<NavDataRes>,
) {
    for (mut cell, position) in &mut query {
        let x: i32 = position.x as i32 / 256;
        let z: i32 = position.z as i32 / 256;
        let chuncks = nav_data.0.cells.clone();
        for i in 0..chuncks.len() {
            let c = chuncks.get(i).unwrap();
            if c.x == x && c.y == z {
                cell.0 = i as u32;
                break;
            }
        }
    }
}

fn is_point_in_triangle_2d(
    px: f32,
    py: f32,
    ax: f32,
    ay: f32,
    bx: f32,
    by: f32,
    cx: f32,
    cy: f32,
) -> bool {
    let cross1 = (bx - ax) * (py - ay) - (by - ay) * (px - ax);
    let cross2 = (cx - bx) * (py - by) - (cy - by) * (px - bx);
    let cross3 = (ax - cx) * (py - cy) - (ay - cy) * (px - cx);

    (cross1 >= 0.0 && cross2 >= 0.0 && cross3 >= 0.0)
        || (cross1 <= 0.0 && cross2 <= 0.0 && cross3 <= 0.0)
}

#[cfg(test)]
mod tests {

    #[test]
    fn point_triangle() {
        let r = super::is_point_in_triangle_2d(
            31.28, 69.43, 32 as f32, 69 as f32, 32 as f32, 70 as f32, 29 as f32, 73 as f32,
        );
        assert_eq!(r, false);
    }
    #[test]
    fn point_triangle2() {
        let r = super::is_point_in_triangle_2d(31.28, 69.43, 32.0, 69.0, 32.0, 70.0, 29.0, 73.0);
        assert_eq!(r, false);
    }
    #[test]
    fn point_inside_triangle() {
        let r = super::is_point_in_triangle_2d(
            2.5, 2.0, // Point P
            0.0, 0.0, // Vertex A
            5.0, 0.0, // Vertex B
            2.5, 5.0, // Vertex C
        );
        assert_eq!(r, true);
    }
}

fn get_polygon_from_pos(
    entity_position: &Position,
    nodes: &Vec<Node>,
    triangles: &Vec<Triangle>,
) -> Option<()> {
    // log!(entity_position);
    // log!(format!(
    //     "search polygon between {},{} and {},{}",
    //     nodes[0].x,
    //     nodes[0].z,
    //     nodes[nodes.len() - 1].x,
    //     nodes[nodes.len() - 1].z
    // ));
    for t in triangles {
        let n1 = nodes.get(t.vertices_index[0] as usize).unwrap();
        let n2 = nodes.get(t.vertices_index[1] as usize).unwrap();
        let n3 = nodes.get(t.vertices_index[2] as usize).unwrap();

        if is_point_in_triangle_2d(
            entity_position.x,
            entity_position.z,
            n1.x as f32,
            n1.z as f32,
            n2.x as f32,
            n2.z as f32,
            n3.x as f32,
            n3.z as f32,
        ) {
            log!(entity_position);
            log!(format!("found triangle !!! {:?},{:?},{:?}", n1, n2, n3));
            ()
        }
        ()
    }

    None // Return None if no polygon contains the point
}

pub fn get_player_polygon(
    mut query: Query<(&Position, &CurrentCell), With<PlayerEntity>>,
    nav_data: Res<NavDataRes>,
) {
    for (player_position, cell_index) in &mut query {
        let cell = nav_data.0.cells.get(cell_index.0 as usize).unwrap();
        let poly = get_polygon_from_pos(player_position, &cell.nodes, &cell.triangles);
    }
}
