use bevy_ecs::{component::Component, system::Resource};

#[derive(Resource)]
pub struct HungerTimer(pub i64);
