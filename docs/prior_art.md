## PIC/FLIP Resources

- @matthias-research https://github.com/matthias-research/pages/blob/master/tenMinutePhysics/18-flip.html
  - https://matthias-research.github.io/pages/tenMinutePhysics/18-flip.html
- @dli https://github.com/dli/fluid
  - http://david.li/fluid/
- @austinEng https://github.com/austinEng/WebGL-PIC-FLIP-Fluid
  - https://www.cs.ubc.ca/~rbridson/docs/zhu-siggraph05-sandfluid.pdf
- Thesis: https://www.diva-portal.org/smash/get/diva2:441801/FULLTEXT01.pdf

- https://github.com/Wumpf/blub

## Eulerian

- https://github.com/PavelDoGreat/WebGL-Fluid-Simulation
  - https://github.com/PavelDoGreat/WebGL-Fluid-Simulation/blob/master/script.js

## Prefix Sum

- https://github.com/kishimisu/WebGPU-Radix-Sort/blob/main/src/shaders/prefix_sum.js
- https://github.com/chronicl/particle_life/blob/d4e3482d73bffd15a04474a7cec54b1d31a15736/assets/prefix_sum.wgsl

## Rendering

- https://blog.maximeheckel.com/posts/painting-with-math-a-gentle-study-of-raymarching/
- https://github.com/electricsquare/raymarching-workshop?tab=readme-ov-file
- Overview of voxel rendering methods: https://github.com/dubiousconst282/VoxelRT
- Lots of graphics ideas from Path Of Exile dev: https://www.youtube.com/watch?v=TrHHTQqmAaM

### Radiance Cascades

- https://news.ycombinator.com/item?id=41957008
- https://jason.today/rc
- https://simondevyoutube.github.io/Shaders_RadianceCascades/

### Shadertoy

- Hybrid Voxel-SDF Traversal: https://www.shadertoy.com/view/dtVSzw
- Voxel Tunnel: https://www.shadertoy.com/view/NtfXDl
- Octree Buffer, add/remove voxels (proves that `texFetch()` isn't so expensive?): https://www.shadertoy.com/view/XtdyDH
- Random Octree Pathtracing: https://www.shadertoy.com/view/4dKfRd
- Fast Octree Pathtracing: https://www.shadertoy.com/view/wtKXWV
- Procedural Octree (just 150 LoC!): https://www.shadertoy.com/view/3dSGDR
- Random Octree (with point light): https://www.shadertoy.com/view/XdyfRy
- Octree Lightly Styld (well styled): https://www.shadertoy.com/view/NdGXWh
- Compact SVO Representation: https://www.shadertoy.com/view/dlBGRc
- Compact SVO Representation (simpler): https://www.shadertoy.com/view/WlXXWf

### Inigo Quilez

I think special mention needs to be given to this guy, he's so talented, so helpful and such an inspiration.
His Shader Toy, Happy Jumping, is rightly famous, it shows off a lot of fundamental raymarching, lighting and
colour grading techniques. And he made a 6 hour long video explaining it all https://www.youtube.com/watch?v=Cfe5UQ-1L9Q ❤️

### Overview of particle shaders:

- https://michaelmoroz.github.io/TODO/2021-3-13-Overview-of-Shadertoy-particle-algorithms/

## Voxel Engines

### Youtubers

- https://www.youtube.com/@DouglasDwyer
  - The best of the Voxel Engine Youtubers
  - Rust, supports integrated graphics
  - Compiled binaries (no source code): https://github.com/DouglasDwyer/octo-release
- https://www.youtube.com/@frozein
  - C, Vulkan
  - https://github.com/frozein/
  - https://store.steampowered.com/app/2435320/Terra_Toy
- https://www.youtube.com/@Tooley1998
  - The most advanced and impressive of all the voxel engines
  - Is making https://store.steampowered.com/app/2776090/Lay_of_the_Land/
- https://www.youtube.com/@GabeRundlett
  - Implemented https://github.com/EmbarkStudios/kajiya
- https://www.youtube.com/@Tantandev
  - Lots of videos, Rust
- https://www.youtube.com/@dreamtowards
  - Rust/Bevy
  - Open Source and playable
- https://www.youtube.com/@BodhiDon
  - https://github.com/Dreamtowards/Ethertum
  - Making great progress on an actual game
  - Unity
  - Dev builds available at https://discord.com/invite/KzQVEFnNQb
- https://www.youtube.com/@ethangore8697
- https://www.youtube.com/@774
- https://www.youtube.com/@TheRaticide
- https://www.youtube.com/@MaxMakesGames
- https://www.youtube.com/channel/UCM2RhfMLoLqG24e_DYgTQeA
- https://www.youtube.com/@_rey
- https://www.youtube.com/@xima1 Raytraced audio!
- https://www.youtube.com/@voxelbee
- https://www.youtube.com/@danygankoff
- https://www.youtube.com/@cyber-gate
- https://www.youtube.com/@NephPlays C#/OpenGL
- https://www.youtube.com/@DestinyHailstorm
- https://www.youtube.com/@DmytroMomotov

### Code

- https://github.com/DouglasDwyer/octo-release
- https://github.com/Blatko1/wgpu-voxel-engine
- https://github.com/voxel-rs/voxel-rs
- https://github.com/Technici4n/rust-voxel-game
- https://github.com/Technici4n/voxel-rs
- https://github.com/Defernus/bevy-voxel-engine
- https://github.com/TanTanDev/first_voxel_engine
- https://github.com/bfops/playform
- https://github.com/veloren/veloren
- https://github.com/dust-engine/dust, also has great blog: https://dust.rs/
- https://github.com/splashdust/bevy_voxel_world: Bevy plugin
- https://github.com/Game4all/vx_bevy: Bevy plugin
- https://github.com/Game4all/unnamed-voxel-tracer: Zig, OpenGL
- https://github.com/mwbryant/logic_voxels: Bevy
- https://github.com/kpreid/all-is-cubes
  - Actually got the local WASM server running!
  - Rust, WebGL/WASM
  - Blog https://kpreid.dreamwidth.org/tag/all+is+cubes
  - Like his code style and Github commentary

### Steam

- https://store.steampowered.com/app/1128000/Cube_World/
- https://store.steampowered.com/app/2776090/Lay_of_the_Land/
- https://store.steampowered.com/app/2435320/Terra_Toy/
- https://store.steampowered.com/app/1167630/Teardown/

### Miscellaneous

- https://voxel.wiki
- https://www.reddit.com/r/VoxelGameDev
