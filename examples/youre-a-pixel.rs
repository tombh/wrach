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
use wrach_bevy::{Particle, WrachPlugin, WrachState};

const NUMBER_OF_PARTICLES: u32 = 10000;
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
        .add_systems(Startup, startup)
        .add_systems(PreUpdate, keyboard_events)
        .add_systems(PostUpdate, move_entities)
        .run();
}

#[derive(Component)]
struct PixelEntity(pub usize);

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<WrachState>,
) {
    let mut particles: Vec<Particle> = Vec::new();
    for _ in 0..NUMBER_OF_PARTICLES {
        particles.push(Particle {
            position: Vec2::new(
                random_float(state.config.dimensions.0 as f32).abs(),
                random_float(state.config.dimensions.1 as f32).abs(),
            ),
            velocity: Vec2::new(random_float(1.0), random_float(1.0)),
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

    if state.packed_data.positions.len() < NUMBER_OF_PARTICLES as usize {
        return;
    }

    pixel.par_iter_mut().for_each(|(mut transform, particle)| {
        let world_pos = Vec2::new(
            (state.packed_data.positions[particle.0].x * SCALE) - (window.width() / 2.0),
            (state.packed_data.positions[particle.0].y * SCALE) - (window.height() / 2.0),
        );

        transform.translation = world_pos.extend(0.);
        transform.look_to(Vec3::Z, Vec3::new(0.5, 0.5, 0.0));
    });
}

fn keyboard_events(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    for event in keyboard_input_events.read() {
        if event.state == ButtonState::Released {
            continue;
        }

        // TODO: Update uniform buffer with key presses
        match &event.logical_key {
            Key::ArrowUp => {}
            Key::ArrowDown => {}
            Key::ArrowLeft => {}
            Key::ArrowRight => {}
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
