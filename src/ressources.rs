use bevy_ecs::{component::Component, resource::Resource};

#[derive(Resource)]
pub struct HungerTimer(pub i64);
