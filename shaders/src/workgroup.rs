#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::neighbours;
use crate::particle;

pub const THREADS: u32 = 128;

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct WorkGroupDimensions {
    pub x: i32,
    pub y: i32,
    pub width: u32,
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct WorkGroupData {
    pub pixel_grid: neighbours::PixelGridLocal,
    pub particles: [particle::ParticleLocal; neighbours::PIXEL_GRID_LOCAL_SIZE],
    pub particles_count: u32,
    pub neighbours: [[particle::ParticleIDLocal; neighbours::MAX_NEIGHBOURS]; THREADS as usize],
    pub neighbours_count: [usize; neighbours::PIXEL_GRID_LOCAL_SIZE],
    pub dimensions: WorkGroupDimensions,
    pub workgroup_id: u32,
}

impl WorkGroupData {
    pub fn populate(
        &mut self,
        thread_id: u32,
        pixel_grid_global: &mut neighbours::PixelGridGlobal,
        particles_global: &particle::ParticlesGlobal,
    ) {
        self.particles_count = 0;
        self.dimensions = self.calculate_dimensons();
        self.populater(thread_id, pixel_grid_global, particles_global);
    }

    pub fn particle_for_work_item(
        &self,
        thread_id: u32,
        iteration: u32,
    ) -> particle::ParticleLocal {
        let particle_id = (iteration * THREADS) + thread_id;
        if particle_id >= self.particles_count {
            //TODO you can do better
            let mut particle = particle::ParticleLocal::default();
            particle.id_global.id = particle::ParticleIDLocal::null();
            particle
        } else {
            self.particles[particle_id as usize]
        }
    }

    pub fn store_neighbour(
        &mut self,
        thread_id: usize,
        count: usize,
        neighbour_id: particle::ParticleIDLocal,
    ) {
        self.neighbours[thread_id][count + 1] = neighbour_id;
    }

    fn calculate_dimensons(&self) -> WorkGroupDimensions {
        let width_extra = neighbours::PIXEL_GRID_LOCAL_COLS + (particle::INFLUENCE_FACTOR * 2);
        let (x, y) = self.workgroup_id_to_global_xy();
        WorkGroupDimensions {
            x,
            y,
            width: width_extra,
        }
    }

    fn workgroup_id_to_global_xy(&self) -> (i32, i32) {
        let global_width = neighbours::PIXEL_GRID_GLOBAL_COLS as f32;
        let workgroup_width = neighbours::PIXEL_GRID_LOCAL_COLS as f32;
        let workgroups_per_global_row = (global_width / workgroup_width).ceil() as u32;
        let mut x = (self.workgroup_id % workgroups_per_global_row) as i32;
        let mut y = (self.workgroup_id as f32 / workgroups_per_global_row as f32).floor() as i32;
        x -= particle::INFLUENCE_FACTOR as i32;
        y -= particle::INFLUENCE_FACTOR as i32;
        (x, y)
    }

    fn populater(
        &mut self,
        thread_id: u32,
        _pixel_grid_global: &neighbours::PixelGridGlobal,
        _particles_global: &particle::ParticlesGlobal,
    ) {
        let (x_min, _x_max, y_min, _y_max) = self.ranges();
        let mut grid_index_local: usize = 0;
        // let id = 0;
        let mut x;
        let mut y;
        for yi in 0..(25 + 1) {
            for xi in 0..(25 + 1) {
                x = x_min + xi as i32;
                y = y_min + yi as i32;
                // if Self::is_xy_out_of_range(x, y) == super::SPIRV_TRUE {
                //     continue;
                // }
                if grid_index_local as u32 % THREADS != thread_id as u32 {
                    grid_index_local += 1;
                    continue;
                }
                // let global_index = self.local_coord_to_global_index(x as u32, y as u32);
                // let _particle_id = pixel_grid_global[global_index.id as usize];
                // let particle_id_local: particle::ParticleIDLocal = particle::ParticleIDLocal { id };
                // self.pixel_grid[grid_index_local] = particle_id_local;
                // if particle_id.id != 0 {
                //     self.add_global_particle_to_local(
                //         particle_id,
                //         particles_global,
                //         particle_id_local,
                //     );
                //     id += 1;
                // }
                let _ = x + y + grid_index_local as i32;
                grid_index_local += 1;
            }
        }
    }

    fn _is_xy_out_of_range(x: i32, y: i32) -> u32 {
        let x_upper = neighbours::PIXEL_GRID_GLOBAL_COLS as i32;
        if (x < 0) | (y < 0) {
            return super::SPIRV_TRUE;
        }
        if (x > x_upper) | (y > x_upper) {
            return super::SPIRV_TRUE;
        }
        return super::SPIRV_FALSE;
    }

    fn ranges(&self) -> (i32, i32, i32, i32) {
        let y_min = self.dimensions.y;
        let y_max = y_min + self.dimensions.width as i32;
        let x_min = self.dimensions.x;
        let x_max = x_min + self.dimensions.width as i32;
        return (x_min, x_max, y_min, y_max);
    }

    // fn local_coord_to_global_index(&self, x: u32, y: u32) -> particle::ParticleIDGlobal {
    //     let global_x = Self::clamp_global(x as i32);
    //     let global_y = Self::clamp_global(y as i32);
    //     let index = (global_y * neighbours::PIXEL_GRID_GLOBAL_COLS) + global_x;
    //     particle::ParticleIDGlobal { id: index }
    // }
    //
    // fn clamp_global(coord: i32) -> u32 {
    //     if coord < 0 {
    //         return 0;
    //     }
    //     if coord as u32 > neighbours::PIXEL_GRID_GLOBAL_COLS - 1 {
    //         return neighbours::PIXEL_GRID_GLOBAL_COLS;
    //     }
    //     return coord as u32;
    // }

    fn _add_global_particle_to_local(
        &mut self,
        particle_id_global: particle::ParticleIDGlobal,
        particles_global: &particle::ParticlesGlobal,
        particle_id_local: particle::ParticleIDLocal,
    ) {
        let particle_global = particles_global[particle_id_global.id as usize];
        let particle_local = particle::ParticleLocal {
            id_global: particle_id_global,
            color: particle_global.color,
            position: particle_global.position,
            velocity: particle_global.velocity,
            ..Default::default()
        };
        self.particles[particle_id_local.id as usize] = particle_local;
        self.particles_count += 1;
    }
}
