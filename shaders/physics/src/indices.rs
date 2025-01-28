//! Track the indices in indices buffers to give us convenient access to cell particles.

use spirv_std::arch::IndexUnchecked;

use crate::cell::MAX_PARTICLES_IN_CELL;

/// All the indices into global memeory that we might need
#[allow(clippy::missing_docs_in_private_items)]
#[derive(Debug)]
pub struct Indices {
    pub workgroup_offset: usize,
    pub centre: Storage,
    pub bottom_left: Storage,
    pub bottom_right: Storage,
    pub top_left: Storage,
    pub top_right: Storage,
}

/// Where the indices are pointing, local or global memory.
#[derive(Debug, Copy, Clone)]
pub struct Storage {
    /// Global GPU VRAM
    pub global: Range,
    /// Memory local to the shader
    pub local: Range,
}

/// Starting and ending indices for particles in a cell.
#[derive(Debug, Copy, Clone)]
pub struct Range {
    /// The index at which the particles starte
    pub from: usize,
    /// The index by which the particles are finished, ie non-inclusive range.
    pub end_by: usize,
}

/// Poor person's enums.
/// Might be some progress tracked [here](https://github.com/gfx-rs/wgpu/issues/4424).
/// Or at least enums should be possible with SPIRV PASSTHROUGH
pub struct Location;

#[allow(clippy::missing_docs_in_private_items)]
impl Location {
    const CENTRE: u32 = 1;
    const BOTTOM_LEFT: u32 = 1;
    const BOTTOM_RIGHT: u32 = 2;
    const TOP_LEFT: u32 = 3;
    const TOP_RIGHT: u32 = 4;
}

impl Indices {
    /// Instantiate
    pub fn new(
        index_main: &[u32],
        index_aux: &[u32],
        cell_index: usize,
        grid_width: u32,
        workgroup_offset: usize,
    ) -> Self {
        Self {
            workgroup_offset,
            centre: Self::get_start_end_indices_of_particles_in_cell(
                index_main,
                Location::CENTRE,
                cell_index,
                grid_width,
                0,
            ),
            bottom_left: Self::get_start_end_indices_of_particles_in_cell(
                index_aux,
                Location::BOTTOM_LEFT,
                cell_index,
                grid_width,
                workgroup_offset,
            ),
            bottom_right: Self::get_start_end_indices_of_particles_in_cell(
                index_aux,
                Location::BOTTOM_RIGHT,
                cell_index,
                grid_width,
                workgroup_offset,
            ),
            top_left: Self::get_start_end_indices_of_particles_in_cell(
                index_aux,
                Location::TOP_LEFT,
                cell_index,
                grid_width,
                workgroup_offset,
            ),
            top_right: Self::get_start_end_indices_of_particles_in_cell(
                index_aux,
                Location::TOP_RIGHT,
                cell_index,
                grid_width,
                workgroup_offset,
            ),
        }
    }

    /// Based on the data structure for spatial binning, get the indices of where the first and last
    /// particles of a cell are.
    fn get_start_end_indices_of_particles_in_cell(
        index_buffer: &[u32],
        location: u32,
        centre_cell_index: usize,
        grid_width: u32,
        workgroup_offset: usize,
    ) -> Storage {
        let aux_grid_width = grid_width as usize + 2;

        let (cell_index, local_index) = match location {
            Location::CENTRE => (centre_cell_index, 0),
            Location::BOTTOM_LEFT => (centre_cell_index, MAX_PARTICLES_IN_CELL),
            Location::BOTTOM_RIGHT => (centre_cell_index + 1, MAX_PARTICLES_IN_CELL * 2),
            Location::TOP_LEFT => (
                centre_cell_index + aux_grid_width,
                MAX_PARTICLES_IN_CELL * 3,
            ),
            Location::TOP_RIGHT => (
                centre_cell_index + aux_grid_width + 1,
                MAX_PARTICLES_IN_CELL * 4,
            ),
            _ => (0, 0),
        };

        // SAFETY:
        //   Getting data with bounds checks is obviously undefined behaviour. We rely on the
        //   rest of the pipeline to ensure that indices are always within limits.
        #[allow(clippy::multiple_unsafe_ops_per_block)]
        let (particles_start_at, marker) = unsafe {
            (
                index_buffer.index_unchecked(cell_index),
                index_buffer.index_unchecked(cell_index + 1),
            )
        };
        let particles_count = marker - particles_start_at;
        let particles_end_by = particles_start_at + particles_count;

        let global = Range {
            from: *particles_start_at as usize,
            end_by: particles_end_by as usize,
        };
        let local = Range {
            from: local_index - workgroup_offset,
            end_by: local_index + particles_count as usize - workgroup_offset,
        };

        Storage { global, local }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(cell_index: usize) -> Indices {
        #[rustfmt::skip]
        let indices_main = [
            0,  1, 1,
                1, 1
        ];
        #[rustfmt::skip]
        let indices_aux = [
            0,  0, 0, 0,
                0, 0, 0,
                0, 0, 1
        ];
        Indices::new(&indices_main, &indices_aux, cell_index, 2, 0)
    }

    #[test]
    fn calculates_indices_at_first_cell() {
        let indices = setup(0);

        assert_eq!(indices.centre.global.from, 0);
        assert_eq!(indices.centre.global.end_by, 1);
        assert_eq!(indices.centre.local.from, 0);
        assert_eq!(indices.centre.local.end_by, 1);

        assert_eq!(indices.top_left.global.from, 0);
        assert_eq!(indices.top_left.global.end_by, 0);
        assert_eq!(indices.top_left.local.from, 12);
        assert_eq!(indices.top_left.local.end_by, 12);

        assert_eq!(indices.top_right.global.from, 0);
        assert_eq!(indices.top_right.global.end_by, 0);
        assert_eq!(indices.top_right.local.from, 16);
        assert_eq!(indices.top_right.local.end_by, 16);
    }

    #[test]
    fn calculates_indices_at_last_cell() {
        let indices = setup(3);

        assert_eq!(indices.centre.global.from, 1);
        assert_eq!(indices.centre.global.end_by, 1);
        assert_eq!(indices.centre.local.from, 0);
        assert_eq!(indices.centre.local.end_by, 0);

        assert_eq!(indices.top_left.global.from, 0);
        assert_eq!(indices.top_left.global.end_by, 0);
        assert_eq!(indices.top_left.local.from, 12);
        assert_eq!(indices.top_left.local.end_by, 12);

        assert_eq!(indices.top_right.global.from, 0);
        assert_eq!(indices.top_right.global.end_by, 1);
        assert_eq!(indices.top_right.local.from, 16);
        assert_eq!(indices.top_right.local.end_by, 17);
    }
}
