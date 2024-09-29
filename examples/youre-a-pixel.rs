extern crate bevy;
extern crate wrach_bevy;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::WindowResolution;

use rand::Rng;
use wrach_bevy::{DrawPlugin, Particle, WrachPlugin, WrachState};

const NUMBER_OF_PARTICLES: u32 = 125_000;
const SCALE: f32 = 4.0;

fn main() {
    let wrach = WrachPlugin::default();
    let window_width = wrach.config.dimensions.0 as f32 * SCALE;
    let window_height = wrach.config.dimensions.1 as f32 * SCALE;

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(window_width, window_height)
                        .with_scale_factor_override(1.0),
                    title: "Wrach example: You're a pixel".into(),
                    ..default()
                }),
                ..default()
            }),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            wrach,
            DrawPlugin,
        ))
        .add_systems(Startup, startup)
        .add_systems(PreUpdate, keyboard_events)
        .run();
}

fn startup(mut state: ResMut<WrachState>) {
    let mut particles: Vec<Particle> = Vec::new();
    for _ in 0..NUMBER_OF_PARTICLES {
        particles.push(Particle {
            position: Vec2::new(
                random_float(state.config.dimensions.0 as f32).abs(),
                random_float(state.config.dimensions.1 as f32).abs(),
            ),
            velocity: Vec2::new(random_float(0.5), random_float(0.5)),
        });
    }
    state.add_particles(particles);
}

fn random_float(magnitude: f32) -> f32 {
    rand::thread_rng().gen_range(-magnitude..magnitude)
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
