#import types::WorldSettings;
#import types::GRID_MAIN;
#import types::GRID_AUX;
#import types::get_cell_index;

@group(0) @binding(0) var<uniform> settings: WorldSettings;

@group(0) @binding(1) var<storage, read_write> indices_main: array<atomic<u32>>;
@group(0) @binding(2) var<storage, read> positions_out: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read> velocities_out: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read_write> positions_in: array<vec2<f32>>;
@group(0) @binding(5) var<storage, read_write> velocities_in: array<vec2<f32>>;

@group(0) @binding(6) var<storage, read_write> indices_aux: array<atomic<u32>>;
@group(0) @binding(7) var<storage, read_write> positions_aux: array<vec2<f32>>;
@group(0) @binding(8) var<storage, read_write> velocities_aux: array<vec2<f32>>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let particle_index = global_id.x;
    if particle_index >= settings.particles_in_frame_count {
        return;
    }

    let particle = positions_out[particle_index];

    pack_cell(settings, particle, GRID_MAIN, particle_index);
    pack_cell(settings, particle, GRID_AUX, particle_index);
}

fn pack_cell(
    settings: WorldSettings,
    particle: vec2<f32>,
    grid_type: u32,
    particle_index: u32
) {
    // NB:
    //   We add one to the cell index because our current implementation of prefix sums shifts all
    //   its items one to the right.
    let prefix_sum_offset_hack = 1u;

    let cell_index = get_cell_index(settings, particle, grid_type, prefix_sum_offset_hack);

    var count: u32;
    if grid_type == GRID_MAIN {
        count = atomicSub(&indices_main[cell_index], 1u);
    }
    if grid_type == GRID_AUX {
        count = atomicSub(&indices_aux[cell_index], 1u);
    }
    let destination_index = count - 1;

    if grid_type == GRID_MAIN {
        positions_in[destination_index] = particle;
        velocities_in[destination_index] = velocities_out[particle_index];
    }
    if grid_type == GRID_AUX {
        positions_aux[destination_index] = particle;
        velocities_aux[destination_index] = velocities_out[particle_index];
    }
}
