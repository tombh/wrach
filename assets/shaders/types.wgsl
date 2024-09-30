#define_import_path types

struct WorldSettings {
    /// Dimensions of the view onto the simulation
    view_dimensions: vec2<f32>,
    /// Current position of the viewoport. Measured from the bottom-left corner
    view_anchor: vec2<f32>,
    /// The dimensions of the spatial bin grid, the unit is a cell
    grid_dimensions: vec2<u32>,
    /// The size of a spatial bin cell
    cell_size: u32,
    /// Total number of particles simulated in this frame. This will normally be much smaller than
    /// the total number of particles that we have a record of.
    particles_in_frame_count: u32,
}

const GRID_MAIN: u32 = 0;
// The "auxiliary grid" is offset by half a cell size and extends such that it perfectly wraps the main grid.
const GRID_AUX: u32 = 1;

fn get_cell_index(settings: WorldSettings, particle: vec2<f32>, grid_type: u32, cell_offset: u32) -> u32 {
    var viewport_offset = 0.0;
    if grid_type == GRID_AUX {
        viewport_offset = f32(settings.cell_size) / 2.0;
    }
    let position_relative_to_viewport_x = particle.x - settings.view_anchor.x - viewport_offset;
    let position_relative_to_viewport_y = particle.y - settings.view_anchor.y - viewport_offset;
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

    var grid_width = settings.grid_dimensions.x;
    if grid_type == GRID_AUX {
        grid_width += 1u;
    }

    let cell_index = ((cell_y * grid_width) + cell_x) + cell_offset;

    return cell_index;
}
