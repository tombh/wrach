//! An acceleration structure for faster particle lookups
//! [See:](https://matthias-research.github.io/pages/tenMinutePhysics/11-hashing.pdf)

use crate::particle_store::{ParticleData, ParticleStore};
use bevy::math::{IVec2, UVec2, Vec2, Vec4, Vec4Swizzles as _};

/// The coordinates of a cell in the Spatial Binning grid
pub type SpatialBinCoord = IVec2;

/// State for constructing and mangging a spatial binning of particles
pub struct SpatialBin {
    /// The size of an individual bin/cell in the spatial bin grid. A cell is a square.
    pub cell_size: u16,
    /// The width and height of the grid of cells, where the unit is a single cell.
    pub grid_dimensions: UVec2,
    /// Coordinates of the active view onto the simulation.
    pub viewport: Vec4,
}

/// An efficient data structure for searching particles.
#[derive(Default)]
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
    pub fn new(cell_size: u16, viewport: Vec4) -> Self {
        let mut spatial_bin = Self {
            cell_size,
            viewport,
            grid_dimensions: UVec2::default(),
        };
        spatial_bin.update_grid_size();
        spatial_bin
    }

    /// Given floating point coordinates find the spatial bin cell in which those coordinates lie.
    pub fn get_cell_coord(&self, position: Vec2) -> SpatialBinCoord {
        let cell_size_f32: f32 = self.cell_size.into();

        #[expect(
            clippy::as_conversions,
            reason = "I just can't figure out how to do it otherwise!"
        )]
        #[expect(
            clippy::cast_possible_truncation,
            reason = "We're just not going anywhere near i32's max"
        )]
        {
            let cell_x = position.x.div_euclid(cell_size_f32) as i32;
            let cell_y = position.y.div_euclid(cell_size_f32) as i32;
            IVec2::new(cell_x, cell_y)
        }
    }

    /// Calculate all the spatial bins currently visible, and required, for simulating a single
    /// frame.
    pub fn get_active_cells(&self) -> (Vec<SpatialBinCoord>, UVec2) {
        let mut cells: Vec<SpatialBinCoord> = Vec::new();
        let mut grid_dimensions = UVec2::default();

        let bottom_left = self.get_cell_coord(self.viewport.xy());
        let top_right = self.get_cell_coord(self.viewport.zw());
        #[expect(
            clippy::arithmetic_side_effects,
            reason = "We're not hittin limits nor dividing by zero."
        )]
        for y in bottom_left.y..=top_right.y {
            grid_dimensions.y += 1_u32;
            for x in bottom_left.x..=top_right.x {
                if y == top_right.y {
                    grid_dimensions.x += 1_u32;
                }
                cells.push(SpatialBinCoord::new(x, y));
            }
        }

        (cells, grid_dimensions)
    }

    /// Calculate how many cells there in the auxiliary layer of spatial bins cells. These are the
    /// cells that are used to get the particles immediately outside a cell.
    #[allow(clippy::arithmetic_side_effects)]
    pub const fn calculate_total_aux_cells(&self) -> u32 {
        (self.grid_dimensions.x + 1) * (self.grid_dimensions.y + 1)
    }

    /// Update the dimensions of the spatial bin grid. The unit is a cell.
    fn update_grid_size(&mut self) {
        let (_cell_list, dimensions) = self.get_active_cells();
        self.grid_dimensions = dimensions;
    }

    /// Create an efficient spatial representation of all the currently active particles in and
    /// around the viewport.
    #[expect(
        clippy::expect_used,
        reason = "`expect`s until there's a way to use `?` in systems"
    )]
    pub fn create_packed_data(&self, store: &ParticleStore) -> PackedData {
        let (cells, _grid) = self.get_active_cells();
        let mut indices: Vec<u32> = Vec::new();
        let mut positions: Vec<Vec2> = Vec::new();
        let mut velocities: Vec<Vec2> = Vec::new();
        let mut current_index = 0;
        let empty_cell = ParticleData::default();

        // Always start with an extra zero at the beginning for 2 reasons:
        //   1. The first index should always point to the beginning of the packed data.
        //   2. There needs to be one more item in the indices array than there are particles so
        //      that the last cell can look to one more item in the array to get its count of
        //      particles.
        indices.push(current_index);

        // And then add another zero at the beginning because our current GPU prefix sum
        // implementation shifts all its items one to the right.
        indices.push(current_index);

        for cell in cells {
            let particles = store.hashmap.get(&cell).unwrap_or(&empty_cell);

            let particle_count: u32 = particles
                .positions
                .len()
                .try_into()
                .expect("Couldn't convert particle count to u32");

            #[expect(
                clippy::arithmetic_side_effects,
                reason = "Assuming there can't be any overflow"
            )]
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
    fn calculating_a_cell_coord_in_the_middle() {
        let spatial_bin = SpatialBin::new(3, Vec4::new(0.0, 0.0, 6.0, 6.0));
        let coord = spatial_bin.get_cell_coord(Vec2::new(3.0, 3.0));
        assert_eq!(coord, SpatialBinCoord::new(1, 1));
    }

    #[test]
    fn calculating_a_cell_coord_in_the_origin_cell() {
        let spatial_bin = SpatialBin::new(3, Vec4::new(0.0, 0.0, 6.0, 6.0));
        let coord = spatial_bin.get_cell_coord(Vec2::new(0.1, 0.1));
        assert_eq!(coord, SpatialBinCoord::new(0, 0));
    }

    #[test]
    fn calculating_a_cell_coord_in_the_top_right() {
        let spatial_bin = SpatialBin::new(3, Vec4::new(0.0, 0.0, 6.0, 6.0));
        let coord = spatial_bin.get_cell_coord(Vec2::new(6.5, 6.5));
        assert_eq!(coord, SpatialBinCoord::new(2, 2));
    }

    #[test]
    fn calculating_a_negative_cell_coord() {
        let spatial_bin = SpatialBin::new(3, Vec4::new(0.0, 0.0, -6.0, -6.0));
        let coord = spatial_bin.get_cell_coord(Vec2::new(-3.0, -3.0));
        assert_eq!(coord, SpatialBinCoord::new(-1, -1));
    }

    #[test]
    fn calculating_active_cells_at_origin() {
        let spatial_bin = SpatialBin::new(6, Vec4::new(0.0, 0.0, 10.0, 10.0));
        let (cells, grid) = spatial_bin.get_active_cells();
        assert_eq!(
            cells,
            vec![
                SpatialBinCoord::new(0, 0),
                SpatialBinCoord::new(1, 0),
                SpatialBinCoord::new(0, 1),
                SpatialBinCoord::new(1, 1)
            ]
        );

        assert_eq!(grid, UVec2::new(2, 2));
    }

    #[test]
    fn calculating_grid_for_rectangle_viewport() {
        let spatial_bin = SpatialBin::new(6, Vec4::new(0.0, 0.0, 15.0, 10.0));
        let (cells, grid) = spatial_bin.get_active_cells();
        assert_eq!(
            cells,
            vec![
                SpatialBinCoord::new(0, 0),
                SpatialBinCoord::new(1, 0),
                SpatialBinCoord::new(2, 0),
                SpatialBinCoord::new(0, 1),
                SpatialBinCoord::new(1, 1),
                SpatialBinCoord::new(2, 1),
            ]
        );

        assert_eq!(grid, UVec2::new(3, 2));
    }

    #[test]
    fn calculating_active_cells_where_viewport_offsets_cells() {
        let spatial_bin = SpatialBin::new(6, Vec4::new(3.3, 3.3, 9.0, 9.0));
        let (cells, _grid) = spatial_bin.get_active_cells();
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
        let (cells, _grid) = spatial_bin.get_active_cells();
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

    #[test]
    fn calculating_total_aux_cells() {
        let spatial_bin = SpatialBin::new(2, Vec4::new(0.0, 0.0, 4.0, 4.0));
        let (cells, _grid) = spatial_bin.get_active_cells();
        assert_eq!(cells.len(), 9);

        let aux_cells = spatial_bin.calculate_total_aux_cells();
        assert_eq!(aux_cells, 16);
    }
}
