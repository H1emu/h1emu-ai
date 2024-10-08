use bevy_ecs::component::Component;
use bevy_time::Timer;

#[derive(Component)]
pub struct HungerTimer(Timer);
