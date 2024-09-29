#import types::WorldSettings;

@group(0) @binding(0) var<uniform> settings: WorldSettings;
@group(0) @binding(1) var<storage, read> positions: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read_write> indices_main: array<atomic<u32>>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if index >= settings.particles_in_frame_count {
        return;
    }

    let position_relative_to_viewport_x = positions[index].x - settings.view_anchor.x;
    let position_relative_to_viewport_y = positions[index].y - settings.view_anchor.y;

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
    let cell_index = (cell_y * settings.grid_dimensions.x) + cell_x;

    atomicAdd(&indices_main[cell_index], 1u);
}
