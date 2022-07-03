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
use crate::world;

pub const MAX_NEIGHBOURS: usize = 9;
pub const MAX_NEIGHBOURS_WITH_COUNT: usize = MAX_NEIGHBOURS + 1;
type Neighbourhood = [particle::Particle; MAX_NEIGHBOURS_WITH_COUNT];
pub type NeighbourhoodIDs = [u32; MAX_NEIGHBOURS + 1];
pub type NeighbourhoodIDsBuffer = [NeighbourhoodIDs; world::NUM_PARTICLES];

cfg_if::cfg_if! {
    if #[cfg(not(test))] {
        const GRID_RESOLUTION: u32 = 2;
    } else {
        const GRID_RESOLUTION: u32 = 1;
    }
}
pub const GRID_WIDTH: u32 = world::MAP_WIDTH * GRID_RESOLUTION;
pub const GRID_HEIGHT: u32 = world::MAP_HEIGHT * GRID_RESOLUTION;
pub const GRID_SIZE: usize = (GRID_WIDTH * GRID_HEIGHT) as usize;
pub type PixelMapBasic = [particle::ParticleID; GRID_SIZE];

const NO_PARTICLE_ID: u32 = 0;

#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Default, Copy, Clone)]
pub struct NeighbouringParticles {
    count: usize,
    particle: particle::Particle,
    pub neighbourhood_ids: NeighbourhoodIDs,
    pub neighbourhood: Neighbourhood,
}

impl NeighbouringParticles {
    pub fn place_particle_in_pixel(
        id: particle::ParticleID,
        particles: &mut particle::Particles,
        map: &mut PixelMapBasic,
    ) {
        let pixel_position = particles[id as usize].pixel_position();
        // Remember that pixels are first class citizens and function as grids in themselves
        let coord = Self::linear_pixel_coord(
            pixel_position.x.floor() as u32,
            pixel_position.y.floor() as u32,
        );
        // Add 1 because ID 0 is reserved as the empty particle
        // TODO: use strong typing and From/Into traits
        map[coord] = id + 1;
    }

    pub fn find(
        id: u32,
        map: &PixelMapBasic,
        particles: &mut particle::Particles,
        neighbourhood_ids_buffer: &mut NeighbourhoodIDsBuffer,
    ) {
        let central_particle = particle::Particle::new(id, particles[id as usize]);
        let mut neighbouring = NeighbouringParticles::new(central_particle);
        neighbouring.search_area_of_influence(map, particles);
        neighbouring.neighbourhood_ids[0] = neighbouring.count as u32;
        neighbourhood_ids_buffer[id as usize] = neighbouring.neighbourhood_ids;
    }

    pub fn recruit(
        id: u32,
        particles: &mut particle::Particles,
        neighbourhood_ids_buffer: &NeighbourhoodIDsBuffer,
    ) -> NeighbouringParticles {
        let central_particle = particle::Particle::new(id, particles[id as usize]);
        let mut neighbouring = NeighbouringParticles::new(central_particle);
        let neighbourhood_ids = neighbourhood_ids_buffer[id as usize];
        neighbouring.count = neighbourhood_ids[0] as usize;
        for i in 1..MAX_NEIGHBOURS_WITH_COUNT {
            neighbouring.recruit_neighbour(i - 1, particles, neighbourhood_ids[i]);
        }
        return neighbouring;
    }

    fn new(particle: particle::Particle) -> NeighbouringParticles {
        let mut neighbouring = NeighbouringParticles::default();
        neighbouring.particle = particle;
        return neighbouring;
    }

    fn linear_pixel_coord(x: u32, y: u32) -> usize {
        ((y * GRID_WIDTH as u32) + x) as usize
    }

    fn search_area_of_influence(
        &mut self,
        map: &PixelMapBasic,
        particles: &mut particle::Particles,
    ) {
        self.count = 0;
        let (x_min, x_max) = Self::range(self.particle.pixel_position().x, GRID_WIDTH);
        let (y_min, y_max) = Self::range(self.particle.pixel_position().y, GRID_HEIGHT);
        for y in y_min..(y_max + 1) {
            for x in x_min..(x_max + 1) {
                if self.count > MAX_NEIGHBOURS as usize {
                    return;
                }
                self.check_pixel(x, y, map, particles);
            }
        }
    }

    fn range(pixel_position: f32, scale: u32) -> (u32, u32) {
        let range = (particle::INFLUENCE_FACTOR * GRID_RESOLUTION) as f32;
        let floor = pixel_position.floor();
        let mut min = floor - range;
        let mut max = floor + range;
        min = min.clamp(0.0, (scale - 1) as f32);
        max = max.clamp(0.0, (scale - 1) as f32);
        return (min as u32, max as u32);
    }

    fn check_pixel(
        &mut self,
        x: u32,
        y: u32,
        map: &PixelMapBasic,
        particles: &mut particle::Particles,
    ) {
        let coord = Self::linear_pixel_coord(x, y);
        let mut neighbour_id = map[coord];

        // Subtract 1 as all the IDs have been incremented to accomodate particle::NO_PARTICLE_ID
        // TODO: can we use Option<T> instead of hacking ID 0?
        if neighbour_id == NO_PARTICLE_ID {
            return;
        }

        neighbour_id -= 1;

        let particle = particles[neighbour_id as usize];
        let neighbour = particle::Particle::new(neighbour_id, particle);
        let length = neighbour.position.distance(self.particle.position);
        if length < (particle::INFLUENCE_FACTOR as f32 * particle::PARTICLE_RADIUS) {
            self.note_neighbour_id(neighbour_id);
        };
    }

    fn note_neighbour_id(&mut self, neighbour_id: u32) {
        let index_offset = self.count + 1;
        self.neighbourhood_ids[index_offset] = neighbour_id;
        self.count += 1;
    }

    fn recruit_neighbour(
        &mut self,
        index: usize,
        particles: &mut particle::Particles,
        neighbour_id: u32,
    ) {
        let particle = particle::Particle::new(neighbour_id, particles[neighbour_id as usize]);
        self.neighbourhood[index] = particle;
    }

    pub fn get_neighbour(&mut self, index: u32) -> particle::Particle {
        self.neighbourhood[index as usize]
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
    use world;

    const O: u32 = 1;

    fn make_particle(x: f32, y: f32) -> particle::ParticleBasic {
        particle::ParticleBasic {
            position: vec2(x, y),
            ..Default::default()
        }
    }

    fn setup() -> (PixelMapBasic, NeighbourhoodIDsBuffer, particle::Particles) {
        let map: PixelMapBasic = Default::default();
        let neighbourhood_ids_buffer: NeighbourhoodIDsBuffer = Default::default();
        let mut particles: particle::Particles =
            [particle::ParticleBasic::default(); world::NUM_PARTICLES];
        particles[0] = make_particle(0.2, -0.2);
        particles[1] = make_particle(0.001, 0.001);
        particles[2] = make_particle(-0.2, 0.2);
        particles[3] = make_particle(1.0, 1.0);
        (map, neighbourhood_ids_buffer, particles)
    }

    fn pixelize(
        map: &mut PixelMapBasic,
        particles: &mut particle::Particles,
        neighbourhood_ids_buffer: &mut NeighbourhoodIDsBuffer,
    ) {
        for i in 0..world::NUM_PARTICLES {
            NeighbouringParticles::place_particle_in_pixel(
                i as particle::ParticleID,
                particles,
                map,
            );
        }
        for i in 0..world::NUM_PARTICLES {
            NeighbouringParticles::find(i as u32, map, particles, neighbourhood_ids_buffer);
        }
    }

    #[test]
    fn it_converts_coords() {
        let bp = make_particle(0.0, 0.0);
        assert_eq!(bp.pixel_position(), vec2(1.0, 1.0));
        let bp = make_particle(-1.0, -1.0);
        assert_eq!(bp.pixel_position(), vec2(0.0, 0.0));
        let bp = make_particle(1.0, 1.0);
        assert_eq!(bp.pixel_position(), vec2(2.0, 2.0));
    }

    #[test]
    fn it_places_particles_in_map() {
        let (mut map, mut neighbourhood_ids_buffer, mut particles) = setup();
        pixelize(&mut map, &mut particles, &mut neighbourhood_ids_buffer);
        #[rustfmt::skip]
        assert_eq!(map, [
            0,   0+O, 0,
            2+O, 1+O, 0,
            0,   0,   3+O
        ]);
    }

    #[test]
    fn it_finds_particles_around_the_centre() {
        let (mut map, mut neighbourhood_ids_buffer, mut particles) = setup();
        pixelize(&mut map, &mut particles, &mut neighbourhood_ids_buffer);
        let mut neighbours =
            NeighbouringParticles::recruit(1, &mut particles, &mut neighbourhood_ids_buffer);
        assert_eq!(neighbours.length(), 3);
        for i in 0..MAX_NEIGHBOURS {
            println!("{}", neighbours.get_neighbour(i as u32).id);
        }
        assert_eq!(neighbours.get_neighbour(0).id, 0);
        assert_eq!(neighbours.get_neighbour(1).id, 2);
        assert_eq!(neighbours.get_neighbour(2).id, 1);
        assert_eq!(neighbours.get_neighbour(3).id, 0);
    }

    #[test]
    fn it_finds_particles_around_the_bottom_left() {
        let (mut map, mut neighbourhood_ids_buffer, mut particles) = setup();
        pixelize(&mut map, &mut particles, &mut neighbourhood_ids_buffer);
        let mut neighbours =
            NeighbouringParticles::recruit(3, &mut particles, &mut neighbourhood_ids_buffer);
        assert_eq!(neighbours.length(), 1);
        assert_eq!(neighbours.get_neighbour(0).id, 3);
        assert_eq!(neighbours.get_neighbour(1).id, 0);
    }
}
