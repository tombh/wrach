#import types::WorldSettings;
#import types::get_cell_index;
#import types::GRID_MAIN;
#import types::GRID_AUX;

@group(0) @binding(0) var<uniform> settings: WorldSettings;
@group(0) @binding(1) var<storage, read> positions_out: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> indices_main: array<atomic<u32>>;
@group(0) @binding(3) var<storage, read_write> indices_aux: array<atomic<u32>>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if index >= settings.particles_in_frame_count {
        return;
    }

    let cell_offset = 0u;

    let particle = positions_out[index];

    let main_cell_index = get_cell_index(settings, particle, GRID_MAIN, cell_offset);
    atomicAdd(&indices_main[main_cell_index], 1u);

    let aux_cell_index = get_cell_index(settings, particle, GRID_AUX, cell_offset);
    atomicAdd(&indices_aux[aux_cell_index], 1u);
}
