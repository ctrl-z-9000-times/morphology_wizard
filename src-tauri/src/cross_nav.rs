//! Plugin for cross sectional viewport and navigation
//!
//! Engage movement by clicking on the 3D viewport and using the "WASD" keys.
//!

// TODO: split the main window into four sections and make three axis-aligned
// orthographic cameras.

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use std::f32::consts::PI;

pub struct CrossSection;

impl Plugin for CrossSection {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, CrossSectionState::setup);
        app.add_systems(Update, (mouse_button, mouse_motion, keyboard_input, update_camera));
    }
}

#[derive(Component)]
struct CrossSectionState {
    cursor_position: Option<Vec2>,
    translation: Vec3,
    rotation: Vec3,
    mouse_sensitivity: f32,
    move_speed: f32,
    sprint_multiplier: f32,
}
impl Default for CrossSectionState {
    fn default() -> Self {
        Self {
            cursor_position: Default::default(),
            translation: Default::default(),
            rotation: Default::default(),
            mouse_sensitivity: 0.2,
            move_speed: 40.0,
            sprint_multiplier: 10.0,
        }
    }
}
impl CrossSectionState {
    fn setup(mut commands: Commands) {
        let state = CrossSectionState::default();
        let camera = Camera3dBundle { ..default() };
        commands.spawn((state, camera));
    }
    fn cursor_grab(&mut self, window: &mut Window) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
        self.cursor_position = window.cursor_position();
        let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
        window.set_cursor_position(Some(center));
    }
    fn cursor_ungrab(&mut self, window: &mut Window) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
        window.set_cursor_position(std::mem::take(&mut self.cursor_position));
    }
    fn rotation(&self) -> Transform {
        let Vec3 { x, y, z } = self.rotation;
        let mut transform = Transform::IDENTITY;
        transform.rotate_z(z);
        transform.rotate_local_y(y);
        transform.rotate_local_x(x);
        transform
    }
    fn look_right(&mut self, radians: f32) {
        self.rotation[1] = (self.rotation[1] + radians).rem_euclid(2.0 * PI);
    }
    fn look_up(&mut self, radians: f32) {
        self.rotation[0] = (self.rotation[0] + radians).clamp(-0.5 * PI, 0.5 * PI);
    }
    fn move_up(&mut self, distance: f32) {
        self.translation[1] += distance;
    }
    fn move_forward(&mut self, distance: f32) {
        self.translation += self.rotation().forward() * distance;
    }
    fn move_right(&mut self, distance: f32) {
        self.translation += self.rotation().right() * distance;
    }
}

fn mouse_button(
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
    mut q_views: Query<&mut CrossSectionState>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    let mut primary_window = q_windows.single_mut();
    let mut cross_sections = q_views.single_mut();
    if buttons.just_pressed(MouseButton::Left) {
        cross_sections.cursor_grab(&mut primary_window);
    } else if buttons.just_released(MouseButton::Left) {
        cross_sections.cursor_ungrab(&mut primary_window);
    }
}

fn mouse_motion(mut evr_motion: EventReader<MouseMotion>, mut q_views: Query<&mut CrossSectionState>, time: Res<Time>) {
    for mut cross_section in q_views.iter_mut() {
        if cross_section.cursor_position.is_none() {
            continue;
        }
        let dt = time.delta_seconds();
        for ev in evr_motion.read() {
            let dx = -ev.delta.x as f32 * dt * cross_section.mouse_sensitivity;
            let dy = -ev.delta.y as f32 * dt * cross_section.mouse_sensitivity;
            cross_section.look_up(dy);
            cross_section.look_right(dx);
        }
    }
}

fn keyboard_input(keys: Res<ButtonInput<KeyCode>>, mut q_views: Query<&mut CrossSectionState>, time: Res<Time>) {
    for mut cross_section in q_views.iter_mut() {
        if cross_section.cursor_position.is_none() {
            continue;
        }
        let mut speed = time.delta_seconds() * cross_section.move_speed;
        let mut direction = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyA) {
            direction[0] -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            direction[0] += 1.0;
        }
        if keys.pressed(KeyCode::Space) {
            direction[1] += 1.0;
        }
        if keys.pressed(KeyCode::ControlLeft) {
            direction[1] -= 1.0;
        }
        if keys.pressed(KeyCode::KeyW) {
            direction[2] += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            direction[2] -= 1.0;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            speed *= cross_section.sprint_multiplier;
        }
        direction = speed * direction.normalize_or_zero();
        cross_section.move_up(direction[1]);
        cross_section.move_forward(direction[2]);
        cross_section.move_right(direction[0]);
    }
}

fn update_camera(q_views: Query<&CrossSectionState>, mut q_transform: Query<&mut Transform, With<CrossSectionState>>) {
    let cross_section_state = q_views.single();
    let mut camera_transform = q_transform.single_mut();
    //
    camera_transform.translation = cross_section_state.translation;
    camera_transform.rotation = cross_section_state.rotation().rotation;
}
