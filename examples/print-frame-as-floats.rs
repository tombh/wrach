use rand::Rng;
use wrach_api::Wrach;
use wrach_bevy::Particle;

extern crate bevy;
extern crate wrach_api;

fn main() {
    let mut wrach = Wrach::new(3);
    let mut particles: Vec<Particle> = Vec::new();
    for _ in 0..3 {
        let x = rand::thread_rng().gen_range(-1.0..1.0);
        let y = rand::thread_rng().gen_range(-1.0..1.0);
        particles.push(Particle {
            position: (x, y),
            velocity: (x, y),
        });
    }
    wrach.add_particles(particles);

    for _ in 0..3 {
        wrach.tick();
    }

    println!("Positions: {:?}", wrach.positions);
    println!("Velocities: {:?}", wrach.velocities);
}
