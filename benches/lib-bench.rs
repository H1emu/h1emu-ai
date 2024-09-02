use criterion::{black_box, criterion_group, criterion_main, Criterion};

// #[path = "../src/lib.rs"]
// mod lib;
use h1emu_ai::*;
use wasm_bindgen::JsValue;

fn public_function(c: &mut Criterion) {
    c.bench_function("AiManager::initialize", |b| {
        b.iter(|| AiManager::initialize())
    });
    let mut ai = AiManager::initialize();
    c.bench_function("AiManager::add_entity", |b| {
        b.iter(|| {
            let e = EntityFromJs::new(EntityType::Player, JsValue::null().into());
            ai.add_entity(e)
        })
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    public_function(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
