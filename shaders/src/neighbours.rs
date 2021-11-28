/// Here we very crudely place particles in a roughly pixel-sized grid of memory, this
/// allows us to efficiently find neighbouring particles because we can just add and
/// subtract to a particle's pixel-grid version of its coords to get access to nearby
/// particles. This is possible because we're limiting the simulation to pixel-type particles
/// anyway. Note that `particle::MAX_VEL` helps with this if it less than the size of pixel box.
/// However this approach is not a fullproof, mainly because it doesn't allow more than one
/// particle to occupy one of the pixel-sized grid points. There are some tricks
/// we can do to help though;
///   * increasing the pixel-grid to have a slightly higher resolution (implemented)
///   * keeping a history of the pixel-grid (TODO)
///
/// If this approach gets too problematic, there's a more traditional approach in commit: ff6a519
//

// For all the glam maths like trigonometry
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;

use crate::particle;
use crate::particle::ParticleaAsPixel;
use crate::workgroup;
use crate::world;

cfg_if::cfg_if! {
    if #[cfg(not(test))] {
        pub const RESOLUTION: u32 = 2;
        // My integrated GPU has a max local bufffer of 65536 bytes
        pub const PIXEL_GRID_LOCAL_COLS: u32 = 25;
    } else {
        pub const PIXEL_GRID_LOCAL_COLS: u32 = 2;
        pub const RESOLUTION: u32 = 1;
    }
}

pub const PIXEL_GRID_GLOBAL_COLS: u32 = world::MAP_WIDTH * RESOLUTION;
pub const PIXEL_GRID_GLOBAL_ROWS: u32 = world::MAP_HEIGHT * RESOLUTION;
pub const PIXEL_GRID_GLOBAL_SIZE: usize =
    (PIXEL_GRID_GLOBAL_COLS * PIXEL_GRID_GLOBAL_ROWS) as usize;
pub type PixelGridGlobal = [particle::ParticleIDGlobal; PIXEL_GRID_GLOBAL_SIZE];

pub const PIXEL_GRID_LOCAL_ROWS: u32 = PIXEL_GRID_LOCAL_COLS;
pub const PIXEL_GRID_LOCAL_SIZE: usize = (PIXEL_GRID_LOCAL_COLS * PIXEL_GRID_LOCAL_COLS) as usize;
pub type PixelGridLocal = [particle::ParticleIDLocal; PIXEL_GRID_LOCAL_SIZE];

pub const WORKGROUPS: u32 = (PIXEL_GRID_GLOBAL_SIZE / PIXEL_GRID_LOCAL_SIZE) as u32;

const NEIGHBOURS_PER_ROW: usize = (((particle::INFLUENCE_FACTOR) * 2) + 1) as usize;
pub const MAX_NEIGHBOURS: usize = NEIGHBOURS_PER_ROW * NEIGHBOURS_PER_ROW;
// const MAX_GRID_NEIGHBOURS: usize = MAX_NEIGHBOURS * RESOLUTION as usize;

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
pub struct NeighbouringParticles {
    count: usize,
    particle: particle::ParticleLocal,
    thread_id: usize,
}

impl NeighbouringParticles {
    pub fn place_particle_in_pixel_grid(
        particle_id: particle::ParticleIDGlobal,
        particles: &mut particle::ParticlesGlobal,
        pixel_grid: &mut PixelGridGlobal,
    ) {
        let pixel_position = particles[particle_id.id as usize].pixel_position();
        let coord = Self::position_to_pixel_grid_index(
            pixel_position.x.floor() as u32,
            pixel_position.y.floor() as u32,
        );
        pixel_grid[coord] = particle_id;
    }

    pub fn new(thread_id: usize, particle: particle::ParticleLocal) -> NeighbouringParticles {
        NeighbouringParticles {
            count: 0,
            particle,
            thread_id,
        }
    }

    fn position_to_pixel_grid_index(x: u32, y: u32) -> usize {
        ((y * PIXEL_GRID_GLOBAL_COLS as u32) + x) as usize
    }

    fn position_to_workgroup_map_index(
        &self,
        workgroup_data: &mut workgroup::WorkGroupData,
        x: u32,
        y: u32,
    ) -> usize {
        ((y * workgroup_data.dimensions.width as u32) + x) as usize
    }

    pub fn find(
        thread_id: usize,
        particle: particle::ParticleLocal,
        workgroup_data: &mut workgroup::WorkGroupData,
    ) {
        let mut neighbours = Self::new(thread_id, particle);
        neighbours.finder(thread_id, workgroup_data);
    }

    fn finder(&mut self, thread_id: usize, workgroup_data: &mut workgroup::WorkGroupData) {
        self.count = 0;
        let (x_min, x_max) = Self::range(self.particle.pixel_position().x, PIXEL_GRID_GLOBAL_COLS);
        let (y_min, y_max) = Self::range(self.particle.pixel_position().y, PIXEL_GRID_GLOBAL_ROWS);
        for y in y_min..(y_max + 1) {
            for x in x_min..(x_max + 1) {
                self.check_pixel(workgroup_data, x, y);
            }
        }
        self.store_neighbour_count(thread_id, workgroup_data);
    }

    fn range(pixel_position: f32, scale: u32) -> (u32, u32) {
        let range = (particle::INFLUENCE_FACTOR * RESOLUTION) as f32;
        let floor = pixel_position.floor();
        let mut min = floor - range;
        let mut max = floor + range;
        min = min.clamp(0.0, (scale - 1) as f32);
        max = max.clamp(0.0, (scale - 1) as f32);
        return (min as u32, max as u32);
    }

    fn check_pixel(&mut self, workgroup_data: &mut workgroup::WorkGroupData, x: u32, y: u32) {
        let coord = self.position_to_workgroup_map_index(workgroup_data, x, y);
        let neighbour_id = workgroup_data.pixel_grid[coord].id;

        if neighbour_id == particle::ParticleIDLocal::null() {
            return;
        }

        let neighbour = &workgroup_data.particles[neighbour_id as usize];
        let length = neighbour.position.distance(self.particle.position);
        if length < (particle::INFLUENCE_FACTOR as f32 * particle::PARTICLE_RADIUS) {
            self.store_neighbour(
                workgroup_data,
                particle::ParticleIDLocal { id: neighbour_id },
            );
        }
    }

    fn store_neighbour(
        &mut self,
        workgroup_data: &mut workgroup::WorkGroupData,
        neighbour_id: particle::ParticleIDLocal,
    ) {
        workgroup_data.store_neighbour(self.thread_id, self.count, neighbour_id);
        self.count += 1;
    }

    fn store_neighbour_count(
        &mut self,
        thread_id: usize,
        workgroup_data: &mut workgroup::WorkGroupData,
    ) {
        workgroup_data.neighbours_count[thread_id] = self.count;
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use glam::vec2;
//     use world;
//
//     const O: u32 = 1;
//
//     fn make_particle(x: f32, y: f32) -> particle::ParticleGlobal {
//         particle::ParticleGlobal {
//             position: vec2(x, y),
//             ..Default::default()
//         }
//     }
//
//     fn setup() -> (PixelGridGlobal, particle::ParticlesGlobal) {
//         let map: PixelGridGlobal = Default::default();
//         let mut particles: particle::ParticlesGlobal =
//             [particle::ParticleGlobal::default(); world::NUM_PARTICLES];
//         particles[0] = make_particle(0.2, -0.2);
//         particles[1] = make_particle(0.001, 0.001);
//         particles[2] = make_particle(-0.2, 0.2);
//         particles[3] = make_particle(1.0, 1.0);
//         (map, particles)
//     }
//
//     fn pixelize(map: &mut PixelGridGlobal, particles: &mut particle::ParticlesGlobal) {
//         for i in 0..world::NUM_PARTICLES {
//             NeighbouringParticles::place_particle_in_pixel_grid(
//                 i as particle::ParticleIDGlobal,
//                 particles,
//                 map,
//             );
//         }
//     }
//
//     #[test]
//     fn it_converts_coords() {
//         let bp = make_particle(0.0, 0.0);
//         assert_eq!(bp.pixel_position(), vec2(1.0, 1.0));
//         let bp = make_particle(-1.0, -1.0);
//         assert_eq!(bp.pixel_position(), vec2(0.0, 0.0));
//         let bp = make_particle(1.0, 1.0);
//         assert_eq!(bp.pixel_position(), vec2(2.0, 2.0));
//     }
//
//     #[test]
//     fn it_places_particles_in_map() {
//         let (mut map, mut particles) = setup();
//         pixelize(&mut map, &mut particles);
//         #[rustfmt::skip]
//         assert_eq!(map, [
//             0,   0+O, 0,
//             2+O, 1+O, 0,
//             0,   0,   3+O
//         ]);
//     }
//
//     #[test]
//     fn it_finds_particles_around_the_centre() {
//         let (mut map, mut particles) = setup();
//         pixelize(&mut map, &mut particles);
//         let mut neighbours = NeighbouringParticles::find(1, &mut map, &mut particles);
//         assert_eq!(neighbours.length(), 3);
//         assert_eq!(neighbours.get_neighbour(0).id, 0);
//         assert_eq!(neighbours.get_neighbour(1).id, 2);
//         assert_eq!(neighbours.get_neighbour(2).id, 1);
//         assert_eq!(neighbours.get_neighbour(3).id, 0);
//     }
//
//     #[test]
//     fn it_finds_particles_around_the_bottom_left() {
//         let (mut map, mut particles) = setup();
//         pixelize(&mut map, &mut particles);
//         let mut neighbours = NeighbouringParticles::find(3, &mut map, &mut particles);
//         assert_eq!(neighbours.length(), 1);
//         assert_eq!(neighbours.get_neighbour(0).id, 3);
//         assert_eq!(neighbours.get_neighbour(1).id, 0);
//     }
// }
