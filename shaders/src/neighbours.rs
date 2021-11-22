#[cfg(target_arch = "spirv")]
// For all the glam maths like trigonometry
use spirv_std::num_traits::Float;

use crate::particle;
use crate::particle::ParticleGridStartID;
use crate::world;

const INFLUENCE_ROW_SIZE: usize = 3;
type NeighbourhoodRow = [particle::Particle; INFLUENCE_ROW_SIZE];
type Neighbourhood = [NeighbourhoodRow; INFLUENCE_ROW_SIZE];

cfg_if::cfg_if! {
    if #[cfg(not(test))] {
        const GRID_RESOLUTIONISH: f32 = 3.0;
        const MAX_PARTICLES_PER_GRID: usize = (GRID_RESOLUTIONISH * GRID_RESOLUTIONISH) as usize * 2;
    } else {
        const GRID_RESOLUTIONISH: f32 = 2.0;
        const MAX_PARTICLES_PER_GRID: usize = 3;
    }
}

pub const GRIDS_PER_ROW: u32 = (world::MAP_WIDTH as f32 / GRID_RESOLUTIONISH) as u32 + 1;
pub const GRIDS_PER_COL: u32 = (world::MAP_HEIGHT as f32 / GRID_RESOLUTIONISH) as u32 + 1;
pub const GRID_COUNT: usize = (GRIDS_PER_ROW * GRIDS_PER_COL) as usize;
const PER_GRID_STORAGE_SIZE: usize = MAX_PARTICLES_PER_GRID + 1;
pub const TOTAL_GRID_STORAGE_SIZE: usize = GRID_COUNT * PER_GRID_STORAGE_SIZE;
pub type GridBasic = [particle::ParticleID; TOTAL_GRID_STORAGE_SIZE];

pub type GridStartID = u32;

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Copy, Clone)]
pub struct NeighbouringParticles {
    count: usize,
    particle: particle::Particle,
    pub neighbourhood: Neighbourhood,
}

impl Default for NeighbouringParticles {
    fn default() -> NeighbouringParticles {
        NeighbouringParticles {
            count: 0,
            particle: particle::Particle::default(),
            neighbourhood: Default::default(),
        }
    }
}

impl NeighbouringParticles {
    pub fn populate_grid(grid_id: u32, particles: &mut particle::Particles, grid: &mut GridBasic) {
        let grid_start_index = grid_id * PER_GRID_STORAGE_SIZE as u32;
        let max_index = grid_start_index as usize + PER_GRID_STORAGE_SIZE;
        let mut index = grid_start_index;
        for candidate_id in 0..world::NUM_PARTICLES {
            let candidate_grid_start_index = particles[candidate_id].grid_start_index;
            if candidate_grid_start_index == grid_start_index {
                index += 1;
                if index < max_index as u32 {
                    grid[index as usize] = candidate_id as particle::ParticleID;
                }
            }
        }

        let total = index - grid_start_index;
        grid[grid_start_index as usize] = total;
    }

    pub fn find(
        id: particle::ParticleID,
        grid: &GridBasic,
        particles: &mut particle::Particles,
    ) -> NeighbouringParticles {
        let central_particle = particle::Particle::new(id, particles[id as usize]);
        let mut neighbouring = NeighbouringParticles::new(central_particle);
        neighbouring.search_area_of_influence(grid, particles);
        return neighbouring;
    }

    fn new(particle: particle::Particle) -> NeighbouringParticles {
        let mut neighbouring = NeighbouringParticles::default();
        neighbouring.particle = particle;
        return neighbouring;
    }

    pub fn grid_coord_to_grid_start_index(x: u32, y: u32) -> GridStartID {
        ((y * GRIDS_PER_ROW as u32) + x) * PER_GRID_STORAGE_SIZE as u32
    }

    fn search_area_of_influence(&mut self, grid: &GridBasic, particles: &mut particle::Particles) {
        self.count = 0;
        let (grid_x, grid_y) = self.particle.grid_coords_from_particle_coords();
        let (x_min, x_max) = Self::range(grid_x, GRIDS_PER_ROW);
        let (y_min, y_max) = Self::range(grid_y, GRIDS_PER_COL);
        for y in y_min..(y_max + 1) {
            for x in x_min..(x_max + 1) {
                self.check_grid(x, y, grid, particles);
            }
        }
        // for i in 0..world::NUM_PARTICLES {
        //     let candidate = particles[i];
        //     let n = candidate.position - self.particle.position;
        //     if n.length() < particle::PARTICLE_INFLUENCE {
        //         let neighbour = particle::Particle::new(i as u32, candidate);
        //         self.store_neighbour(neighbour);
        //     }
        // }
    }

    fn range(position: u32, scale: u32) -> (u32, u32) {
        let scale_as_index = scale as i32 - 1;
        let mut min: i32 = position as i32 - 1;
        let mut max: i32 = position as i32 + 1;
        if min < 0 {
            min = 0;
        }
        if max > scale_as_index {
            max = scale_as_index;
        }
        return (min as u32, max as u32);
    }

    fn check_grid(
        &mut self,
        x: u32,
        y: u32,
        grid: &GridBasic,
        particles: &mut particle::Particles,
    ) {
        let grid_start_index = Self::grid_coord_to_grid_start_index(x, y) as usize;
        let total_particles_in_grid = grid[grid_start_index];
        let start = grid_start_index + 1;
        let end = start + total_particles_in_grid as usize;
        for grid_index in start..end {
            let particle_id = grid[grid_index] as usize;
            let candidate = particles[particle_id];
            let n = candidate.pre_fluid_position - self.particle.pre_fluid_position;
            if n.length() < particle::PARTICLE_INFLUENCE {
                let neighbour = particle::Particle::new(particle_id as u32, candidate);
                self.store_neighbour(neighbour);
            }
        }
    }

    // Because the Spirv compiler doesn't make it easy to init arrays with more then 32 items,
    // we're storing the neighbours in a 2D array
    fn neighbourhood_coord_hack(index: u32) -> (usize, usize) {
        let row = (index as f32 / INFLUENCE_ROW_SIZE as f32).floor();
        let col = index % INFLUENCE_ROW_SIZE as u32;
        (col as usize, row as usize)
    }

    fn store_neighbour(&mut self, neighbour: particle::Particle) {
        let (col, row) = Self::neighbourhood_coord_hack(self.count as u32);
        self.neighbourhood[row][col] = neighbour;
        self.count += 1;
    }

    pub fn get_neighbour(&mut self, index: u32) -> particle::Particle {
        let (col, row) = Self::neighbourhood_coord_hack(index);
        self.neighbourhood[row][col]
    }

    pub fn length(&self) -> u32 {
        self.count as u32
    }
}

// TODO There's a pointer cast error when you use `for in ..`
// impl Iterator for NeighbouringParticles {}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec2;
    use particle::ParticleGridStartID;
    use world;

    /// For a map (x) size of 3x3 and a grid (-|) resolution of 2.0, the map can be placed
    /// in the grid as so:
    /// ------------
    /// |x x |   x |
    /// |    |     |
    /// ------------
    /// |x x |   x |
    /// |x x |   x |
    /// ------------

    fn make_particle(x: f32, y: f32) -> particle::ParticleBasic {
        let mut particle = particle::ParticleBasic {
            position: vec2(x, y),
            ..Default::default()
        };
        particle.grid_start_index = particle.grid_start_index();
        particle
    }

    fn setup() -> (GridBasic, particle::Particles) {
        let map: GridBasic = [0; TOTAL_GRID_STORAGE_SIZE];
        let mut particles: particle::Particles =
            [particle::ParticleBasic::default(); world::NUM_PARTICLES];
        particles[0] = make_particle(0.2, -0.2);
        particles[1] = make_particle(0.001, 0.001);

        particles[2] = make_particle(0.8, 0.8);
        particles[3] = make_particle(1.0, 1.0);

        particles[4] = make_particle(-0.001, -0.001);
        (map, particles)
    }

    fn pixelize(grid: &mut GridBasic, particles: &mut particle::Particles) {
        for grid_id in 0..GRID_COUNT {
            NeighbouringParticles::populate_grid(grid_id as u32, particles, grid);
        }
    }

    #[test]
    fn it_calculates_basic_properties_of_grid() {
        assert_eq!(GRIDS_PER_ROW, 2);
        assert_eq!(GRIDS_PER_COL, 2);
        assert_eq!(GRID_COUNT, 4);
        assert_eq!(TOTAL_GRID_STORAGE_SIZE, 16);
    }

    #[test]
    fn it_converts_coords() {
        let bp = make_particle(-1.0, -1.0);
        assert_eq!(bp.grid_start_index(), 0);

        let bp = make_particle(0.0, 0.0);
        assert_eq!(bp.grid_start_index(), 12);

        let bp = make_particle(0.001, 0.001);
        assert_eq!(bp.grid_start_index(), 12);

        let bp = make_particle(1.0, -1.0);
        assert_eq!(bp.grid_start_index(), 4);

        let bp = make_particle(-1.0, 1.0);
        assert_eq!(bp.grid_start_index(), 8);

        let bp = make_particle(0.99, 0.99);
        assert_eq!(bp.grid_start_index(), 12);

        let bp = make_particle(1.0, 1.0);
        assert_eq!(bp.grid_start_index(), 12);
    }

    #[test]
    fn it_places_particles_in_grids() {
        let (mut grid, mut particles) = setup();
        pixelize(&mut grid, &mut particles);
        #[rustfmt::skip]
        assert_eq!(grid, [
        // count  particles   count  particles
        // bottom row
           1,     4, 0, 0,    1,     0, 0, 0,
        // top row
           0,     0, 0, 0,    3,     1, 2, 3,
        ]);
    }

    #[test]
    fn it_finds_particles_around_the_centre() {
        let (mut grid, mut particles) = setup();
        pixelize(&mut grid, &mut particles);
        let mut neighbours = NeighbouringParticles::find(1, &mut grid, &mut particles);
        assert_eq!(neighbours.length(), 2);
        assert_eq!(neighbours.get_neighbour(0).id, 4);
        assert_eq!(neighbours.get_neighbour(1).id, 0);
        assert_eq!(neighbours.get_neighbour(2).id, 0);
    }

    #[test]
    fn it_finds_particles_around_the_bottom_left() {
        let (mut grid, mut particles) = setup();
        pixelize(&mut grid, &mut particles);
        let mut neighbours = NeighbouringParticles::find(3, &mut grid, &mut particles);
        assert_eq!(neighbours.length(), 1);
        assert_eq!(neighbours.get_neighbour(0).id, 2);
        assert_eq!(neighbours.get_neighbour(1).id, 0);
    }
}
