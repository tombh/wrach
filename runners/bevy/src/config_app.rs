//! User-defineable config for Wrach

/// All the config for the Wrach Bevy plugin
#[derive(Clone, Copy)]
#[allow(clippy::exhaustive_structs)]
pub struct WrachConfig {
    /// Dimensions of the realtime view onto the simulation. Doesn't necessarily imply the size of
    /// any window, that should be handled outside this plugin
    pub dimensions: (u16, u16),
    /// Should particles be limited to within the viewport dimensions? Default is false, therefore
    /// the viewport must move to interact with the entire simulation.
    pub boundaries_as_dimensions: bool,
    /// The size of a single cell in the spatial binning grid used to accelerate particle search.
    ///   - The unit is multiples of the size of a particle (therefore 1).
    ///   - Playing with this value may improve perforance on certain hardware.
    pub cell_size: u16,
}

impl Default for WrachConfig {
    #[inline]
    fn default() -> Self {
        Self {
            // 4:3
            dimensions: (480, 352),
            // dimensions: (240, 200),
            // Particles can leave the edges of the dimensions
            boundaries_as_dimensions: false,
            // Good performance on my Asahi, Apple M1, OpenGL machine
            cell_size: 6,
        }
    }
}
