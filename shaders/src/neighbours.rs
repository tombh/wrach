#[cfg(target_arch = "spirv")]
// For all the glam maths like trigonometry
use spirv_std::num_traits::Float;

use crate::particle;
use crate::particle::ParticleaAsPixel;
use crate::world;

const INFLUENCE_ROW_SIZE: usize = ((particle::INFLUENCE_FACTOR * 2) + 1) as usize;
type NeighbourhoodRow = [particle::Particle; INFLUENCE_ROW_SIZE];
type Neighbourhood = [NeighbourhoodRow; INFLUENCE_ROW_SIZE];

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
    pub fn place_particle_in_pixel(
        id: particle::ParticleID,
        particles: &mut particle::Particles,
        map: &mut world::PixelMapBasic,
    ) {
        let pixel_position = particles[id as usize].pixel_position();
        let coord = Self::linear_pixel_coord(
            pixel_position.x.floor() as u32,
            pixel_position.y.floor() as u32,
        );
        // Add 1 because ID 0 is reserved as the empty particle
        map[coord] = id + 1;
    }

    pub fn find(
        id: particle::ParticleID,
        map: &world::PixelMapBasic,
        particles: &mut particle::Particles,
    ) -> NeighbouringParticles {
        let central_particle = particle::Particle::new(id, particles[id as usize]);
        let mut neighbouring = NeighbouringParticles::new(central_particle);
        neighbouring.search_area_of_influence(map, particles);
        return neighbouring;
    }

    fn new(particle: particle::Particle) -> NeighbouringParticles {
        let mut neighbouring = NeighbouringParticles::default();
        neighbouring.particle = particle;
        return neighbouring;
    }

    fn linear_pixel_coord(x: u32, y: u32) -> usize {
        ((y * world::MAP_WIDTH as u32) + x) as usize
    }

    fn search_area_of_influence(
        &mut self,
        map: &world::PixelMapBasic,
        particles: &mut particle::Particles,
    ) {
        self.count = 0;
        let (x_min, x_max) = Self::range(self.particle.pixel_position().x, world::MAP_WIDTH);
        let (y_min, y_max) = Self::range(self.particle.pixel_position().y, world::MAP_HEIGHT);
        for y in y_min..(y_max + 1) {
            for x in x_min..(x_max + 1) {
                self.check_pixel(x, y, map, particles);
            }
        }
    }

    fn range(pixel_position: f32, scale: u32) -> (u32, u32) {
        let floor = pixel_position.floor();
        let mut min = floor - particle::INFLUENCE_FACTOR as f32;
        let mut max = floor + particle::INFLUENCE_FACTOR as f32;
        min = min.clamp(0.0, (scale - 1) as f32);
        max = max.clamp(0.0, (scale - 1) as f32);
        return (min as u32, max as u32);
    }

    fn check_pixel(
        &mut self,
        x: u32,
        y: u32,
        map: &world::PixelMapBasic,
        particles: &mut particle::Particles,
    ) {
        let coord = Self::linear_pixel_coord(x, y);
        let mut neighbour_id = map[coord];

        // Subtract 1 as all the IDs have been incremented to accomodate particle::NO_PARTICLE_ID
        // TODO: can we use Option<T> instead of hacing ID 0?
        if neighbour_id == particle::NO_PARTICLE_ID {
            return;
        }
        neighbour_id -= 1;

        let particle = particles[neighbour_id as usize];
        let neighbour = particle::Particle::new(neighbour_id, particle);
        self.store_neighbour(neighbour);
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
    use world;

    fn make_particle(x: f32, y: f32) -> particle::ParticleBasic {
        particle::ParticleBasic {
            position: vec2(x, y),
            ..Default::default()
        }
    }

    fn setup() -> (world::PixelMapBasic, particle::Particles) {
        let map: world::PixelMapBasic = Default::default();
        let mut particles: particle::Particles =
            [particle::ParticleBasic::default(); world::NUM_PARTICLES];
        particles[0] = make_particle(0.5, -0.5);
        particles[1] = make_particle(0.001, 0.001);
        particles[2] = make_particle(-0.5, 0.5);
        particles[3] = make_particle(1.0, 1.0);
        (map, particles)
    }

    fn pixelize(map: &mut world::PixelMapBasic, particles: &mut particle::Particles) {
        for i in 0..world::NUM_PARTICLES {
            NeighbouringParticles::place_particle_in_pixel(
                i as particle::ParticleID,
                particles,
                map,
            );
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
        let (mut map, mut particles) = setup();
        pixelize(&mut map, &mut particles);
        #[rustfmt::skip]
        assert_eq!(map, [
            0, 1, 0,
            3, 2, 0,
            0, 0, 4
        ]);
    }

    #[test]
    fn it_finds_particles_around_the_centre() {
        let (mut map, mut particles) = setup();
        pixelize(&mut map, &mut particles);
        let mut neighbours = NeighbouringParticles::find(1, &mut map, &mut particles);
        assert_eq!(neighbours.length(), 4);
        assert_eq!(neighbours.get_neighbour(0).id + 1, 1);
        assert_eq!(neighbours.get_neighbour(1).id + 1, 3);
        assert_eq!(neighbours.get_neighbour(2).id + 1, 2);
        assert_eq!(neighbours.get_neighbour(3).id + 1, 4);
    }

    #[test]
    fn it_finds_particles_around_the_bottom_left() {
        let (mut map, mut particles) = setup();
        pixelize(&mut map, &mut particles);
        let mut neighbours = NeighbouringParticles::find(3, &mut map, &mut particles);
        assert_eq!(neighbours.length(), 2);
        assert_eq!(neighbours.get_neighbour(0).id + 1, 2);
        assert_eq!(neighbours.get_neighbour(1).id + 1, 4);
    }
}
