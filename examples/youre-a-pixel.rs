extern crate bevy;
extern crate wrach_bevy;

use bevy::color::palettes::css;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::window::WindowResolution;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};

use rand::Rng;
use wrach_bevy::{GPUUpload, Particle, WrachPlugin, WrachState};

const NUMBER_OF_PARTICLES: u32 = 20000;
const SCALE: f32 = 3.0;

fn main() {
    let wrach = WrachPlugin::default();
    let window_width = wrach.config.dimensions.0 as f32 * SCALE;
    let window_height = wrach.config.dimensions.1 as f32 * SCALE;

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(window_width, window_height)
                        .with_scale_factor_override(1.0),
                    title: "Wrach example: You're a pixel".into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(wrach)
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, keyboard_events)
        .add_systems(PostUpdate, move_entities)
        .run();
}

#[derive(Component)]
struct PixelEntity(pub usize);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<WrachState>,
) {
    let mut particles: Vec<Particle> = Vec::new();
    for _ in 0..NUMBER_OF_PARTICLES {
        particles.push(Particle {
            position: (
                random_float(state.config.dimensions.0 as f32).abs(),
                random_float(state.config.dimensions.1 as f32).abs(),
            ),
            velocity: (random_float(1.0), random_float(1.0)),
        });
    }
    state.add_particles(particles);

    commands.spawn(Camera2dBundle::default());

    let boid_mesh_you = meshes.add(RegularPolygon::new(3.0 * SCALE, 4));

    // A single red "pixel" that you control
    commands.spawn((
        PixelEntity(0),
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(boid_mesh_you),
            material: materials.add(Color::from(css::ORANGE_RED)),
            ..Default::default()
        },
    ));

    let boid_mesh = meshes.add(RegularPolygon::new(1.0 * SCALE, 4));
    let boid_material = materials.add(Color::from(css::ANTIQUE_WHITE));
    for i in 1..NUMBER_OF_PARTICLES {
        commands.spawn((
            PixelEntity(i as usize),
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(boid_mesh.clone()),
                material: boid_material.clone(),
                ..Default::default()
            },
        ));
    }
}

fn random_float(magnitude: f32) -> f32 {
    rand::thread_rng().gen_range(-magnitude..magnitude)
}

fn move_entities(
    state: Res<WrachState>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut pixel: Query<(&mut Transform, &PixelEntity), With<PixelEntity>>,
) {
    let window = window.single();

    if state.positions.len() < NUMBER_OF_PARTICLES as usize {
        return;
    }

    pixel.par_iter_mut().for_each(|(mut transform, particle)| {
        let world_pos = Vec2::new(
            (state.positions[particle.0].x * SCALE) - (window.width() / 2.0),
            (state.positions[particle.0].y * SCALE) - (window.height() / 2.0),
        );

        transform.translation = world_pos.extend(0.);
        transform.look_to(Vec3::Z, Vec3::new(0.5, 0.5, 0.0));
    });
}

fn keyboard_events(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut state: ResMut<WrachState>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    let delta = 0.1;

    for event in keyboard_input_events.read() {
        if event.state == ButtonState::Released {
            continue;
        }

        let mut current_velocity = state.velocities[0];
        let mut gpu_uploads = GPUUpload::default();

        match &event.logical_key {
            Key::ArrowUp => {
                current_velocity.y += delta;
                gpu_uploads.velocities = vec![current_velocity];
                state.gpu_upload(gpu_uploads);
            }
            Key::ArrowDown => {
                current_velocity.y -= delta;
                gpu_uploads.velocities = vec![current_velocity];
                state.gpu_upload(gpu_uploads);
            }
            Key::ArrowLeft => {
                current_velocity.x -= delta;
                gpu_uploads.velocities = vec![current_velocity];
                state.gpu_upload(gpu_uploads);
            }
            Key::ArrowRight => {
                current_velocity.x += delta;
                gpu_uploads.velocities = vec![current_velocity];
                state.gpu_upload(gpu_uploads);
            }
            _ => {}
        }

        #[allow(clippy::single_match)]
        match &event.key_code {
            KeyCode::KeyQ => {
                app_exit_events.send(AppExit::Success);
            }
            _ => {}
        }
    }
}
