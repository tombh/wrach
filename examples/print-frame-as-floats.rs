use wrach_api::Wrach;

extern crate bevy;
extern crate wrach_api;

fn main() {
    let mut wrach = Wrach::new(3);
    wrach.tick();

    println!("Positions: {:?}", wrach.positions);
    println!("Velocities: {:?}", wrach.velocities);
}
