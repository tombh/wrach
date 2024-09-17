//! An acceleration structure for faster particle lookups
//! [See:](https://matthias-research.github.io/pages/tenMinutePhysics/11-hashing.pdf)

use crate::particle_store::{ParticleData, ParticleStore};
use bevy::math::{IVec2, Vec2, Vec4, Vec4Swizzles};

/// The coordinates of a cell in the Spatial Binning grid
pub type SpatialBinCoord = IVec2;

/// State for constructing and mangging a spatial binning of particles
pub struct SpatialBin {
    /// The size of an individual bin/cell in the spatial bin grid. A cell is a square.
    pub cell_size: u16,
    /// Coordinates of the active view onto the simulation.
    pub viewport: Vec4,
}

/// An effcient data structure for searching particles.
#[derive(Default)]
#[allow(clippy::exhaustive_structs)]
pub struct PackedData {
    /// A vector of spatial bin cells. Each item points to the corresponding array index of the first
    /// particle in the cell. The next item, whether the cell has particles or not, contains the
    /// index of the next group of particles of a cell. This way we can quickly get the number of
    /// particles in a spatial bin cell by subtracting the the current item from the subsequent
    /// item.
    pub indices: Vec<u32>,
    /// All the particle positions ordered by cells
    pub positions: Vec<Vec2>,
    /// All the particle velocities ordered by cells
    pub velocities: Vec<Vec2>,
}

impl SpatialBin {
    /// Instantiate. `cell_size` is the size of a spatial bin.
    pub const fn new(cell_size: u16, viewport: Vec4) -> Self {
        Self {
            cell_size,
            viewport,
        }
    }

    /// Given floating point coordinates find the spatial bin cell in which those coordinates lie.
    pub fn get_cell_coord(&self, position: Vec2) -> SpatialBinCoord {
        let cell_size_f32: f32 = self.cell_size.into();

        // I don't think `as`-casting should cause any terrible problems?
        #[allow(clippy::as_conversions)]
        #[allow(clippy::cast_possible_truncation)]
        {
            let cell_x = position.x.div_euclid(cell_size_f32) as i32;
            let cell_y = position.y.div_euclid(cell_size_f32) as i32;
            IVec2::new(cell_x, cell_y)
        }
    }

    /// Calculate all the spatial bins currently visible, and required, for simulating a single
    /// frame.
    pub fn get_active_cells(&self) -> Vec<SpatialBinCoord> {
        let mut cells: Vec<SpatialBinCoord> = Vec::new();

        let bottom_left = self.get_cell_coord(self.viewport.xy());
        let top_right = self.get_cell_coord(self.viewport.zw());
        for y in bottom_left.y..=top_right.y {
            for x in bottom_left.x..=top_right.x {
                cells.push(SpatialBinCoord::new(x, y));
            }
        }
        cells
    }

    /// Create an efficient spatial representation of all the currently active particles in and
    /// around the viewpoort.
    pub fn create_packed_data(&self, store: &ParticleStore) -> PackedData {
        let cells = self.get_active_cells();
        let mut indices: Vec<u32> = Vec::new();
        let mut positions: Vec<Vec2> = Vec::new();
        let mut velocities: Vec<Vec2> = Vec::new();
        let mut current_index = 0;
        let empty_cell = ParticleData::default();

        indices.push(current_index);
        for cell in cells {
            let particles = store.hashmap.get(&cell).unwrap_or(&empty_cell);

            #[allow(clippy::expect_used)]
            let particle_count: u32 = particles
                .positions
                .len()
                .try_into()
                .expect("Couldn't convert particle count to u32");

            #[allow(clippy::arithmetic_side_effects)]
            {
                current_index += particle_count;
            };

            indices.push(current_index);

            positions.extend(particles.positions.clone());
            velocities.extend(particles.velocities.clone());
        }

        PackedData {
            indices,
            positions,
            velocities,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn calculating_active_cells_at_origin() {
        let spatial_bin = SpatialBin::new(6, Vec4::new(0.0, 0.0, 10.0, 10.0));
        let cells = spatial_bin.get_active_cells();
        assert_eq!(
            cells,
            vec![
                SpatialBinCoord::new(0, 0),
                SpatialBinCoord::new(1, 0),
                SpatialBinCoord::new(0, 1),
                SpatialBinCoord::new(1, 1)
            ]
        );
    }

    #[test]
    fn calculating_active_cells_where_viewport_offsets_cells() {
        let spatial_bin = SpatialBin::new(6, Vec4::new(3.3, 3.3, 9.0, 9.0));
        let cells = spatial_bin.get_active_cells();
        assert_eq!(
            cells,
            vec![
                SpatialBinCoord::new(0, 0),
                SpatialBinCoord::new(1, 0),
                SpatialBinCoord::new(0, 1),
                SpatialBinCoord::new(1, 1)
            ]
        );
    }

    #[test]
    fn calculating_active_cells_with_negative_viewport() {
        let spatial_bin = SpatialBin::new(6, Vec4::new(-5.0, -5.0, 0.0, 0.0));
        let cells = spatial_bin.get_active_cells();
        assert_eq!(
            cells,
            vec![
                SpatialBinCoord::new(-1, -1),
                SpatialBinCoord::new(0, -1),
                SpatialBinCoord::new(-1, 0),
                SpatialBinCoord::new(0, 0),
            ]
        );
    }
}
