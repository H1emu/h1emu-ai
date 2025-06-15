use bevy_ecs::resource::Resource;

#[derive(Resource)]
pub struct HungerTimer(pub i64);
