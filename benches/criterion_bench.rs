extern crate hclog;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

#[path = "common.rs"]
mod common;
use common::{
    *,
    benches::*,
};

fn hclog(bencher: &mut Criterion) {
    common::init();
    bencher.bench_function(&KeyA.to_string(), move |b| b.iter(|| black_box(log_hello_world())));
    bencher.bench_function(&KeyB.to_string(), move |b| b.iter(|| black_box(log_random_vec())));
    bencher.bench_function(&KeyC.to_string(), move |b| b.iter(|| black_box(log_simple_fmt())));
    bencher.bench_function(&KeyC.to_string(), move |b| b.iter(|| black_box(log_level_disable())));
}
criterion_group!{
    name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(500);
    targets = hclog
}
criterion_main!(benches);
