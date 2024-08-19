# Wrach

A 2D pixel physics engine. Potentially as the basis for a game like Noita and to provide eye candy in the CLI tool Tattoy.

## Run

`WGPU_BACKEND=vulkan cargo run --example youre-a-pixel`

### Compile shaders

Using a dedicated Rust GPU shader compiler: https://github.com/tombh/rust-gpu-compiler
`cargo run ../wrach/shaders/physics ../wrach/assets/shaders/wrach_physics.spv`

## Workflow

- Lint `cargo clippy --all --all-targets --all-features`
- Remove unused deps: `cargo shear --fix`
