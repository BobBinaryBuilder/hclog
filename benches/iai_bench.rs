extern crate hclog;
#[allow(unused,dead_code)]
use iai_callgrind::{main, library_benchmark_group, library_benchmark};
use std::hint::black_box;

#[path = "common.rs"]
mod common;
use common::*;

#[library_benchmark]
#[bench::setup(common::init())]
fn hclog_hello_world(_: bool) {
    black_box(benches::log_hello_world());
}
#[library_benchmark]
#[bench::setup(common::init())]
fn hclog_random_vec(_: bool) {
    black_box(benches::log_random_vec());
}
#[library_benchmark]
#[bench::setup(common::init())]
fn hclog_simple_fmt(_: bool) {
    black_box(benches::log_simple_fmt());
}

#[library_benchmark]
#[bench::setup(common::init())]
fn hclog_level_disabled(_: bool) {
    black_box(benches::log_level_disable());
}

library_benchmark_group!(
    name = hclog_bench_group;
    benchmarks = hclog_hello_world, hclog_random_vec, hclog_simple_fmt,
                 hclog_level_disabled,
);
main!(library_benchmark_groups = hclog_bench_group);
