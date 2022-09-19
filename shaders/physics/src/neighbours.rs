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

use crate::wrach_glam::glam::{vec2, Vec2};

use crate::particle;
use crate::world;

pub const MAX_NEIGHBOURS: usize = 9;
pub const MAX_NEIGHBOURS_WITH_COUNT: usize = MAX_NEIGHBOURS + 1;
type Neighbourhood = [particle::Particle; MAX_NEIGHBOURS_WITH_COUNT];
pub type NeighbourhoodIDs = [particle::ParticleID; MAX_NEIGHBOURS + 1];
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

const NO_PARTICLE_ID: particle::ParticleID = 0;

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
        position: particle::ParticlePosition,
        map: &mut PixelMapBasic,
    ) {
        let pixel_position = position.pixel_position();
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
        id: particle::ParticleID,
        map: &PixelMapBasic,
        positions: &particle::ParticlePositions,
        neighbourhood_ids_buffer: &mut NeighbourhoodIDsBuffer,
    ) -> NeighbourhoodIDs {
        let particle = particle::Particle::new(particle::Particle {
            id,
            position: positions[id as usize],
            ..Default::default()
        });
        let central_particle = particle::Particle::new(particle);
        let mut neighbouring = NeighbouringParticles::new(central_particle);
        neighbouring.search_area_of_influence(map, positions);
        neighbouring.neighbourhood_ids[0] = neighbouring.count as particle::ParticleID;
        neighbourhood_ids_buffer[id as usize] = neighbouring.neighbourhood_ids;
        neighbouring.neighbourhood_ids
    }

    pub fn recruit_from_global(
        id: particle::ParticleID,
        positions: &particle::ParticlePositions,
        velocities: &particle::ParticleVelocities,
        propogations: &particle::ParticlePropogations,
        neighbourhood_ids_buffer: &NeighbourhoodIDsBuffer,
        stage: u32,
    ) -> NeighbouringParticles {
        let central_particle = particle::Particle::new(particle::Particle {
            id,
            position: positions[id as usize],
            ..Default::default()
        });
        let mut neighbouring = NeighbouringParticles::new(central_particle);
        let neighbourhood_ids = neighbourhood_ids_buffer[id as usize];
        neighbouring.count = neighbourhood_ids[0] as usize;
        for i in 1..=neighbouring.count {
            neighbouring.neighbourhood[i - 1] = neighbouring.recruit_neighbour(
                positions,
                velocities,
                propogations,
                neighbourhood_ids[i],
                stage,
            );
        }
        neighbouring
    }

    pub fn recruit_from_ids(
        id: particle::ParticleID,
        positions: &particle::ParticlePositions,
        velocities: &particle::ParticleVelocities,
        propogations: &particle::ParticlePropogations,
        neighbourhood_ids: NeighbourhoodIDs,
        stage: u32,
    ) -> NeighbouringParticles {
        let central_particle = particle::Particle::new(particle::Particle {
            id,
            position: positions[id as usize],
            ..Default::default()
        });
        let mut neighbouring = NeighbouringParticles::new(central_particle);
        neighbouring.count = neighbourhood_ids[0] as usize;
        for i in 1..=neighbouring.count {
            neighbouring.neighbourhood[i - 1] = neighbouring.recruit_neighbour(
                positions,
                velocities,
                propogations,
                neighbourhood_ids[i],
                stage,
            );
        }
        neighbouring
    }

    fn new(particle: particle::Particle) -> NeighbouringParticles {
        NeighbouringParticles {
            particle,
            ..Default::default()
        }
    }

    fn linear_pixel_coord(x: u32, y: u32) -> usize {
        ((y * GRID_WIDTH as u32) + x) as usize
    }

    fn search_area_of_influence(
        &mut self,
        map: &PixelMapBasic,
        positions: &particle::ParticlePositions,
    ) {
        self.count = 0;
        let (x_min, x_max) = Self::range(self.particle.position.pixel_position().x, GRID_WIDTH);
        let (y_min, y_max) = Self::range(self.particle.position.pixel_position().y, GRID_HEIGHT);
        for y in y_min..(y_max + 1) {
            for x in x_min..(x_max + 1) {
                if self.count > MAX_NEIGHBOURS as usize {
                    return;
                }
                self.check_pixel(x, y, map, positions);
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
        (min as u32, max as u32)
    }

    fn check_pixel(
        &mut self,
        x: u32,
        y: u32,
        map: &PixelMapBasic,
        positions: &particle::ParticlePositions,
    ) {
        let coord = Self::linear_pixel_coord(x, y);

        // -------------------------------
        // EXPENSIVE MEMORY ACESS
        let mut neighbour_id = map[coord];
        // -------------------------------

        // Subtract 1 as all the IDs have been incremented to accomodate particle::NO_PARTICLE_ID
        // TODO: can we use Option<T> instead of hacking ID 0?
        if neighbour_id == NO_PARTICLE_ID {
            return;
        }

        neighbour_id -= 1;

        // ---------------------------------------------
        // EXPENSIVE MEMORY ACESS
        let position = positions[neighbour_id as usize];
        // ---------------------------------------------

        let length = position.distance(self.particle.position);
        if length < (particle::INFLUENCE_FACTOR as f32 * particle::PARTICLE_RADIUS) {
            self.note_neighbour_id(neighbour_id);
        };
    }

    fn note_neighbour_id(&mut self, neighbour_id: particle::ParticleID) {
        let index_offset = self.count + 1;
        self.neighbourhood_ids[index_offset] = neighbour_id;
        self.count += 1;
    }

    fn recruit_neighbour(
        &mut self,
        positions: &particle::ParticlePositions,
        velocities: &particle::ParticleVelocities,
        propogations: &particle::ParticlePropogations,
        neighbour_id: particle::ParticleID,
        stage: u32,
    ) -> particle::Particle {
        match stage {
            0 => particle::Particle::new(particle::Particle {
                id: neighbour_id,
                position: positions[neighbour_id as usize],
                velocity: velocities[neighbour_id as usize],
                ..Default::default()
            }),
            1 => particle::Particle::new(particle::Particle {
                id: neighbour_id,
                previous: propogations[neighbour_id as usize].previous,
                position: positions[neighbour_id as usize],
                velocity: velocities[neighbour_id as usize],
                lambda: propogations[neighbour_id as usize].lambda,
                ..Default::default()
            }),
            _ => particle::Particle::default(),
        }
    }

    pub fn get_neighbour(&mut self, index: u32) -> particle::Particle {
        self.neighbourhood[index as usize]
    }

    pub fn length(&self) -> u32 {
        self.count as u32
    }
}

pub trait PositionAsPixel {
    fn pixel_position(&self) -> Vec2;
    fn scale(&self, position: f32, scale: u32) -> f32;
}

impl PositionAsPixel for Vec2 {
    fn pixel_position(&self) -> Vec2 {
        vec2(
            self.scale(self.x, GRID_WIDTH),
            self.scale(self.y, GRID_HEIGHT),
        )
    }
    fn scale(&self, position: f32, scale: u32) -> f32 {
        ((position + 1.0) / 2.0) * (scale - 1) as f32
    }
}

// TODO There's a pointer cast error when you use `for in ..`
// impl Iterator for NeighbouringParticles {}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec2;
    use world;

    const O: particle::ParticleID = 1;

    fn make_position(x: f32, y: f32) -> particle::ParticlePosition {
        vec2(x, y)
    }

    fn setup() -> (
        PixelMapBasic,
        NeighbourhoodIDsBuffer,
        particle::ParticlePositions,
    ) {
        let map: PixelMapBasic = Default::default();
        let neighbourhood_ids_buffer: NeighbourhoodIDsBuffer = Default::default();
        let mut positions: particle::ParticlePositions =
            [particle::ParticlePosition::default(); world::NUM_PARTICLES];
        positions[0] = make_position(0.2, -0.2);
        positions[1] = make_position(0.001, 0.001);
        positions[2] = make_position(-0.2, 0.2);
        positions[3] = make_position(1.0, 1.0);
        (map, neighbourhood_ids_buffer, positions)
    }

    fn pixelize(
        map: &mut PixelMapBasic,
        positions: &particle::ParticlePositions,
        neighbourhood_ids_buffer: &mut NeighbourhoodIDsBuffer,
    ) {
        for i in 0..world::NUM_PARTICLES {
            NeighbouringParticles::place_particle_in_pixel(
                i as particle::ParticleID,
                positions[i],
                map,
            );
        }
        for i in 0..world::NUM_PARTICLES {
            NeighbouringParticles::find(
                i as particle::ParticleID,
                map,
                positions,
                neighbourhood_ids_buffer,
            );
        }
    }

    #[test]
    fn it_converts_coords() {
        let bp = make_position(0.0, 0.0);
        assert_eq!(bp.pixel_position(), vec2(1.0, 1.0));
        let bp = make_position(-1.0, -1.0);
        assert_eq!(bp.pixel_position(), vec2(0.0, 0.0));
        let bp = make_position(1.0, 1.0);
        assert_eq!(bp.pixel_position(), vec2(2.0, 2.0));
    }

    #[test]
    fn it_places_particles_in_map() {
        let (mut map, mut neighbourhood_ids_buffer, particles) = setup();
        pixelize(&mut map, &particles, &mut neighbourhood_ids_buffer);
        #[rustfmt::skip]
        assert_eq!(map, [
            0,   O, 0,
            2+O, 1+O, 0,
            0,   0,   3+O
        ]);
    }

    #[test]
    fn it_finds_particles_around_the_centre() {
        let (mut map, mut neighbourhood_ids_buffer, positions) = setup();
        pixelize(&mut map, &positions, &mut neighbourhood_ids_buffer);
        let velocities = [vec2(0.0, 0.0); world::NUM_PARTICLES];
        let propogations = [particle::ParticlePropogation::default(); world::NUM_PARTICLES];
        let mut neighbours = NeighbouringParticles::recruit_from_global(
            1,
            &positions,
            &velocities,
            &propogations,
            &neighbourhood_ids_buffer,
            0,
        );
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
        let (mut map, mut neighbourhood_ids_buffer, positions) = setup();
        pixelize(&mut map, &positions, &mut neighbourhood_ids_buffer);
        let velocities = [vec2(0.0, 0.0); world::NUM_PARTICLES];
        let propogations = [particle::ParticlePropogation::default(); world::NUM_PARTICLES];
        let mut neighbours = NeighbouringParticles::recruit_from_global(
            3,
            &positions,
            &velocities,
            &propogations,
            &neighbourhood_ids_buffer,
            0,
        );
        assert_eq!(neighbours.length(), 1);
        assert_eq!(neighbours.get_neighbour(0).id, 3);
        assert_eq!(neighbours.get_neighbour(1).id, 0);
    }
}
