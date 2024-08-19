extern crate bevy;
extern crate wrach_bevy;

use bevy::color::palettes::css;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};

use wrach_bevy::{WrachPlugin, WrachState};

const NUMBER_OF_PARTICLES: i32 = 10000;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(WrachPlugin {
            size: NUMBER_OF_PARTICLES,
        })
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
) {
    commands.spawn(Camera2dBundle::default());

    let boid_mesh_you = meshes.add(RegularPolygon::new(5., 4));

    // A single red "pixel" that you control
    commands.spawn((
        PixelEntity(0),
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(boid_mesh_you),
            material: materials.add(Color::from(css::ORANGE_RED)),
            ..Default::default()
        },
    ));

    let boid_mesh = meshes.add(RegularPolygon::new(1., 4));
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

fn move_entities(
    state: Res<WrachState>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut pixel: Query<(&mut Transform, &PixelEntity), With<PixelEntity>>,
) {
    let window = window.single();

    if state.positions.is_empty() {
        return;
    }

    pixel.par_iter_mut().for_each(|(mut transform, particle)| {
        let world_pos = Vec2::new(
            (window.width() / 2.) * (state.positions[particle.0].x),
            (window.height() / 2.) * (state.positions[particle.0].y),
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
    let delta = 0.001;
    let mut current_velocity = state.velocities[0];

    for event in keyboard_input_events.read() {
        if event.state == ButtonState::Released {
            continue;
        }

        match &event.logical_key {
            Key::ArrowUp => {
                current_velocity.y += delta;
                state.overwrite = vec![current_velocity];
            }
            Key::ArrowDown => {
                current_velocity.y -= delta;
                state.overwrite = vec![current_velocity];
            }
            Key::ArrowLeft => {
                current_velocity.x -= delta;
                state.overwrite = vec![current_velocity];
            }
            Key::ArrowRight => {
                current_velocity.x += delta;
                state.overwrite = vec![current_velocity];
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
