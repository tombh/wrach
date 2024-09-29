//! Just some constants to make it easier to refer to buffers in multiple places.

/// Just a place to keep buffer names.
pub struct Buffers;

impl Buffers {
    /// Config data for the simulation
    pub const WORLD_SETTINGS_UNIFORM: &'static str = "world_config";
    /// Efficient packing of particle indices and spatial bin cell counts
    pub const INDICES_MAIN: &'static str = "indices_main";
    /// A scratch buffer for prefix sum calculations
    pub const INDICES_BLOCK_SUMS: &'static str = "indices_block_sums";
    /// Pixel positions buffer ID for reading
    pub const POSITIONS_IN: &'static str = "positions_in";
    /// Pixel positions buffer ID for writing
    pub const POSITIONS_OUT: &'static str = "positions_out";
    /// Pixel velocities buffer ID for reading
    pub const VELOCITIES_IN: &'static str = "velocities_in";
    /// Pixel velocities buffer ID for writing
    pub const VELOCITIES_OUT: &'static str = "velocities_out";
}
