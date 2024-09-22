#import types::WorldSettings;

@group(0) @binding(0) var<uniform> settings: WorldSettings;
@group(0) @binding(1) var<storage, read> positions_out: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read> velocities_out: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read_write> indices: array<atomic<u32>>;
@group(0) @binding(4) var<storage, read_write> positions_in: array<vec2<f32>>;
@group(0) @binding(5) var<storage, read_write> velocities_in: array<vec2<f32>>;

@compute @workgroup_size(1024)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if global_id.x >= settings.particles_in_frame_count {
        return;
    }

    let particle_index = global_id.x;

    // TODO: may need an offset in the future if we decide not to use 0,0 as the origin
    let position_relative_to_viewport_x = positions_out[particle_index].x - settings.view_anchor.x;
    let position_relative_to_viewport_y = positions_out[particle_index].y - settings.view_anchor.y;

    let cell_x = u32(
        floor(
            position_relative_to_viewport_x / f32(settings.cell_size)
        )
    );
    let cell_y = u32(
        floor(
            position_relative_to_viewport_y / f32(settings.cell_size)
        )
    );

    // NB:
    //   We add one to the cell index because our current implementation of prefix sums shifts all
    //   its items one to the right.
    let prefix_hack = 1u;
    let cell_index = ((cell_y * settings.grid_dimensions.x) + cell_x) + prefix_hack;

    let count = atomicSub(&indices[cell_index], 1u);
    let destination_index = count - 1;

    // TODO: probably best to put positions_out[index] in a variable to prevent double reads.
    positions_in[destination_index] = positions_out[particle_index];
    velocities_in[destination_index] = velocities_out[particle_index];
}
