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
