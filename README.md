# Wrach

A 2D pixel physics engine. Potentially as the basis for a game like Noita and to provide eye candy in the CLI tool Tattoy.

## Run

`WAYLAND_DISPLAY= DISPLAY=:0 RUST_BACKTRACE=1 RUST_LOG="info,wgpu_hal::gles=off" cargo run --example youre-a-pixel`

### Compile shaders

Using a dedicated Rust GPU shader compiler: https://github.com/tombh/rust-gpu-compiler
`cargo run ../wrach/shaders/physics ../wrach/assets/shaders/wrach_physics.spv`

## Benchmarks

Release build:

- 1,000,000 particles at ~39fps

## Workflow

- Tests: `cargo test --workspace`
- Lint `cargo clippy --all --all-targets --all-features -- --deny warnings`
- Remove unused deps: `cargo shear --fix`

## TODO

- [ ] Support changing the workgroup size without recreating the comp ute worker
- [ ] Confirm how long it takes between `.gpu_uploads()` and the change appearing on screen
- [ ] Logs should not output unless explicitly requested in `RUST_LOG`
