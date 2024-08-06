use bevy_ecs::prelude::*;
use wasm_bindgen::prelude::*;
#[derive(Component)]
#[wasm_bindgen]
struct Position(Vec<f32>);

#[derive(Component)]
struct ZombieEntity();
#[derive(Component)]
struct PlayerEntity();
#[derive(Component)]
struct DeerEntity();

#[derive(Component)]
#[wasm_bindgen]
struct EntityDefaultBundle {
    position: Position,
    velocity: Velocity,
}
#[derive(Component)]
#[wasm_bindgen]
struct Velocity(Vec<f32>);

#[wasm_bindgen]
struct CB(js_sys::Function);

#[derive(Resource)]
struct StupidValue(u32);

// This system moves each entity with a Position and Velocity component
fn movement(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut query {}
}
fn test(mut tet: ResMut<StupidValue>) {
    tet.0 = 2;
}
#[wasm_bindgen]
pub struct AiManager {
    world: World,
    schedule: Schedule,
}
#[wasm_bindgen]
pub struct EntityFromJs {
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
    ) -> EntityFromJs {
        EntityFromJs {
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
        let mut world = World::new();
        world.insert_resource(StupidValue(0));
        let mut schedule = Schedule::default();
        schedule.add_systems(movement);
        schedule.add_systems(test);

        AiManager { world, schedule }
    }

    pub fn getdata(&mut self) -> u32 {
        let d: &StupidValue = self.world.resource();
        d.0
    }

    pub fn run(&mut self) {
        self.schedule.run(&mut self.world);
    }
    pub fn add_entity(&mut self, e: EntityFromJs) {
        e.action_cb.call0(&JsValue::null()).unwrap();
        let mut entity = self.world.spawn(EntityDefaultBundle {
            position: Position(e.position),
            velocity: Velocity(e.velocity),
        });
        match e.entity_type {
            EntityType::Player => entity.insert(PlayerEntity {}),
            EntityType::Zombie => entity.insert(ZombieEntity {}),
            EntityType::Deer => entity.insert(DeerEntity {}),
        };
    }
}
