use components::Position;
use criterion::{criterion_group, criterion_main, Criterion};

#[path = "../src/components.rs"]
mod components;
#[path = "../src/systems.rs"]
mod systems;
use h1emu_ai::*;

fn public_function(c: &mut Criterion) {
    c.bench_function("AiManager::initialize", |b| b.iter(AiManager::initialize));
    // let mut ai = AiManager::initialize();
    // c.bench_function("AiManager::add_entity", |b| {
    //     b.iter(|| {
    //         let e = EntityFromJs::new(EntityType::Player, JsValue::null().into());
    //         ai.add_entity(e)
    //     })
    // });
    let radius = 2.0;
    let player_pos = Position {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    let enemi_pos = Position {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    c.bench_function("systems::is_pos_in_radius", |b| {
        b.iter(|| systems::is_pos_in_radius(radius, &player_pos, &enemi_pos))
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    public_function(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
