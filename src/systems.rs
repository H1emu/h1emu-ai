use std::{collections::HashMap, env::current_dir, fmt::format};

use bevy_ecs::prelude::*;
use js_sys::Float32Array;
use wasm_bindgen::{JsValue, UnwrapThrowExt};

use crate::{
    chunck_schemas::{Cell, Node, Triangle},
    components::{
        BreadScrum, CurrentCell, H1emuEntity, PlayerEntity, Position, Target, ZombieEntity,
    },
    log, NavDataRes,
};

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
    mut query: Query<(&mut CurrentCell, &Position), Changed<Position>>,
    nav_data: Res<NavDataRes>,
) {
    for (mut cell, position) in &mut query {
        let x: i32 = position.x as i32 / 256;
        let z: i32 = position.z as i32 / 256;
        let chuncks = &nav_data.0.cells;
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

fn get_polygon_index_from_pos(
    entity_position: &Position,
    nodes: &Vec<Node>,
    triangles: &Vec<Triangle>,
) -> Option<u32> {
    // log!(entity_position);
    // log!(format!(
    //     "search polygon between {},{} and {},{}",
    //     nodes[0].x,
    //     nodes[0].z,
    //     nodes[nodes.len() - 1].x,
    //     nodes[nodes.len() - 1].z
    // ));
    for i in 0..triangles.len() {
        let t = triangles.get(i).unwrap();
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
            // log!(format!("found triangle !!! {:?},{:?},{:?}", n1, n2, n3));
            log!(t);
            return Some(i as u32);
        }
    }

    None // Return None if no polygon contains the point
}

pub fn zombie_hunt(
    mut zombie_query: Query<(&Position, Entity), (With<ZombieEntity>,)>,
    mut others_query: Query<&Position, (Without<ZombieEntity>,)>,
    mut commands: Commands,
) {
    for (zpos, zent) in &mut zombie_query {
        for pos in &mut others_query {
            let mut e = commands.get_entity(zent).unwrap();
            e.insert(Target(pos.clone()));
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct NodePath {
    pub gcost: u32,
    pub hcost: u32,
}

impl NodePath {
    pub fn get_fcost(&self) -> u32 {
        self.gcost + self.hcost
    }
}

fn euclidean_distance(vec_a: &Position, vec_b: &Position) -> f32 {
    let dx = vec_a.x - vec_b.x;
    let dy = vec_a.y - vec_b.y;
    let dz = vec_a.z - vec_b.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}
pub fn follow_breadscrum(
    mut query: Query<(
        &mut Position,
        &mut BreadScrum,
        &H1emuEntity,
        &CurrentCell,
        Entity,
    )>,
    nav_data: Res<NavDataRes>,
    mut commands: Commands,
) {
    for (mut pos, mut bs, entity, cell_index, e) in &mut query {
        if bs.0.len() == 0 {
            commands.get_entity(e).unwrap().remove::<BreadScrum>();
            continue;
        }
        let next_triangle = match bs.0.get(0) {
            Some(index) => &nav_data.0.cells[cell_index.0 as usize]
                .triangles
                .get((*index) as usize),
            None => {
                commands.get_entity(e).unwrap().remove::<BreadScrum>();
                continue;
            } // Skip
        };
        let next_node = match next_triangle {
            Some(t) => nav_data.0.cells[cell_index.0 as usize]
                .nodes
                .get(t.vertices_index[0] as usize),
            None => {
                commands.get_entity(e).unwrap().remove::<BreadScrum>();
                continue;
            } // Skip
        };
        if next_node.is_some() {
            bs.0.remove(0);
            let node = next_node.unwrap();

            let method = &JsValue::from_str(&"goTo");
            let args = js_sys::Array::new();
            let jspa = js_sys::Array::new();
            jspa.push(&JsValue::from(node.x));
            jspa.push(&JsValue::from(node.y + 20));
            jspa.push(&JsValue::from(node.z));
            // move this in a sys
            pos.x = node.x as f32;
            pos.y = node.y as f32 + 20.0;
            pos.z = node.z as f32;

            log!(format!("go to {:?}", node));
            let js_pos = Float32Array::new(&jspa);
            args.push(&js_pos);
            entity.call_method(method, &args);
        }
    }
}

pub fn get_target_breadscrum(
    mut query: Query<(&Position, &Target, &CurrentCell, Entity)>,
    nav_data: Res<NavDataRes>,
    mut commands: Commands,
) {
    for (pos, target, cell_index, e) in &mut query {
        log!(format!(
            "I want to go from {:?} to here {:?}",
            pos, target.0
        ));
        let cell = nav_data.0.cells.get(cell_index.0 as usize).unwrap();
        let original_poly_index =
            get_polygon_index_from_pos(&pos, &cell.nodes, &cell.triangles).unwrap_throw();
        let original_poly = cell.triangles.get(original_poly_index as usize).unwrap();
        let v0_original = cell
            .nodes
            .get(original_poly.vertices_index[0] as usize)
            .unwrap();
        let original_poly_pos: Position = Position {
            x: v0_original.x as f32,
            y: v0_original.y as f32,
            z: v0_original.z as f32,
        };

        // TODO: can be null since it can be from another cell
        let target_poly_index =
            get_polygon_index_from_pos(&target.0, &cell.nodes, &cell.triangles).unwrap_or(0);
        if target_poly_index == 0 {
            log!("Target lost!!");
            return;
        }

        let target_poly = cell.triangles.get(target_poly_index as usize).unwrap();

        let v0_target = cell
            .nodes
            .get(target_poly.vertices_index[0] as usize)
            .unwrap();
        let target_poly_pos: Position = Position {
            x: v0_target.x as f32,
            y: v0_target.y as f32,
            z: v0_target.z as f32,
        };

        let paths = astar_search(
            cell,
            original_poly_index,
            target_poly_index,
            target_poly_pos,
            original_poly_pos,
        );
        let mut ec = commands.get_entity(e).unwrap();
        ec.insert(BreadScrum(paths));
    }
}

fn astar_search(
    cell: &Cell, // assuming Cell is a struct containing triangles and nodes
    original_poly_index: u32,
    target_poly_index: u32,
    target_poly_pos: Position,
    original_poly_pos: Position,
) -> Vec<u32> {
    let mut polygon_loop_index = original_poly_index;
    let mut path_nodes: HashMap<u32, NodePath> = HashMap::new();
    let mut open_list: Vec<u32> = Vec::new(); // List of nodes to explore
    let mut closed_list: HashMap<u32, NodePath> = HashMap::new();
    let mut predecessors: HashMap<u32, u32> = HashMap::new(); // To track the path

    open_list.push(polygon_loop_index);

    while let Some(current_index) = open_list.pop() {
        if current_index == target_poly_index {
            log!("Found the target polygon!");
            break;
        }

        let current_polygon = match cell.triangles.get(current_index as usize) {
            Some(polygon) => polygon,
            None => continue, // Skip if polygon not found
        };

        let current_position = match cell.nodes.get(current_polygon.vertices_index[0] as usize) {
            Some(node) => Position {
                x: node.x as f32,
                y: node.y as f32,
                z: node.z as f32,
            },
            None => continue, // Skip if node not found
        };

        for &neighbor_index in &current_polygon.neighbors {
            if closed_list.contains_key(&(neighbor_index as u32)) {
                continue; // Skip nodes already processed
            }

            let neighbor_position = match cell.nodes.get(neighbor_index as usize) {
                Some(node) => Position {
                    x: node.x as f32,
                    y: node.y as f32,
                    z: node.z as f32,
                },
                None => continue, // Skip if node not found
            };

            let gcost = euclidean_distance(&neighbor_position, &original_poly_pos) as u32;
            let hcost = euclidean_distance(&neighbor_position, &target_poly_pos) as u32;
            let path = NodePath { gcost, hcost };

            if !path_nodes.contains_key(&(neighbor_index as u32))
                || path.get_fcost() < path_nodes[&(neighbor_index as u32)].get_fcost()
            {
                path_nodes.insert(neighbor_index as u32, path);
                open_list.push(neighbor_index as u32);
                predecessors.insert(neighbor_index as u32, current_index); // Update predecessor
            }
        }

        let v = path_nodes.get(&current_index).clone();
        if v.is_some() {
            closed_list.insert(current_index, v.unwrap().clone());
        }
    }
    let mut path: Vec<u32> = vec![];
    let mut indexos = target_poly_index;
    loop {
        if indexos == original_poly_index {
            break;
        }
        let p = predecessors.get(&indexos);
        if p.is_none() {
            break;
        }
        path.push(*p.unwrap());
        indexos = *p.unwrap();
    }
    path.reverse();
    log!(path);
    return path;
}

// pub fn test_follow(
//     mut zombie_query: Query<&H1emuEntity, With<ZombieEntity>>,
//     mut player_query: Query<&H1emuEntity, With<PlayerEntity>>,
// ) {
//     let method = &JsValue::from_str(&"goTo");
//     for obj in &mut zombie_query {
//         for player in &mut player_query {
//             let pos = player.get_position();
//             let args = js_sys::Array::new();
//             let jspa = js_sys::Array::new();
//             jspa.push(&JsValue::from(pos.x));
//             jspa.push(&JsValue::from(pos.y));
//             jspa.push(&JsValue::from(pos.z));
//
//             let js_pos = Float32Array::new(&jspa);
//             args.push(&js_pos);
//             obj.call_method(method, &args);
//         }
//     }
// }
