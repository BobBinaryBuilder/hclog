use hclog_macros::HCLog;
pub use hclog::*;

#[derive(HCLog, Copy, Clone, Debug)]
#[hclog(
    default_level = Level::Debug10,
    default_facade = FacadeVariant::File("/dev/null".into(), false),
)]
pub enum BenchmarkKeys {
    Key0,
    #[hclog(name = "static_str")]
    KeyA,
    #[hclog(name = "print_vec")]
    KeyB,
    #[hclog(name = "simple_fmtstring")]
    KeyC,
    #[hclog(name = "level_disable", level = Level::Warn)]
    KeyD,
}
pub use BenchmarkKeys::*;

pub const RAND_VEC: &[i16; 64] = &[
    0x70, 0x23, 0xbd, 0xcb, 0x3a, 0xfd, 0x73, 0x48, 0x46, 0x1c, 0x06, 0xcd,
    0x81, 0xfd, 0x38, 0xeb, 0xfd, 0xa8, 0xfb, 0xba, 0x90, 0x4f, 0x8e, 0x3e,
    0xa9, 0xb5, 0x43, 0xf6, 0x54, 0x5d, 0xa1, 0xf2, 0xd5, 0x43, 0x29, 0x55,
    0x61, 0x3f, 0x0f, 0xcf, 0x62, 0xd4, 0x97, 0x05, 0x24, 0x2a, 0x9a, 0xf9,
    0xe6, 0x1e, 0x85, 0xdc, 0x0d, 0x65, 0x1e, 0x40, 0xdf, 0xcf, 0x01, 0x7b,
    0x45, 0x57, 0x58, 0x87
];

pub fn init() -> bool {
    println!("init std::stdio structures");
    eprintln!("init std::stdio structures");
    BenchmarkKeys::init_with_defaults("iai_bench").unwrap();
    true
}

pub mod benches {
    use super::*;

    pub fn log_hello_world() { lI!(KeyA, "Hello, World"); }
    pub fn log_random_vec() { lI!(KeyB, "{:?}", RAND_VEC); }
    pub fn log_simple_fmt() { lI!(KeyC, "This {} a simple test {} + {} = {}", "is", 1, 2, 1 + 2); }
    pub fn log_level_disable() { lI!(KeyD, "This won't get printed: {:?}", RAND_VEC); }
}
