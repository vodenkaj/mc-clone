use bevy::{
    input::mouse::MouseMotion,
    prelude::{
        Component, EventReader, Input, KeyCode, Plugin, Quat, Query, Res, Transform, Update, Vec2,
        Vec3,
    },
    time::Time,
};

#[derive(Component)]
pub struct FlyCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivy: f32,
    pub speed: f32,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            sensitivy: 20.0,
            speed: 0.1,
        }
    }
}

pub struct CameraHandlerPlugin;

impl Plugin for CameraHandlerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, handle_input);
        app.add_systems(Update, handle_mouse_input);
    }
}

fn handle_input(mut query: Query<(&mut Transform, &FlyCamera)>, keys: Res<Input<KeyCode>>) {
    let (mut camera_transform, camera) = query.get_single_mut().unwrap();
    let mut velocity = Vec3::default();
    if keys.pressed(KeyCode::Comma) {
        velocity += camera_transform.forward() * camera.speed;
    }
    if keys.pressed(KeyCode::O) {
        velocity += camera_transform.back() * camera.speed;
    }
    if keys.pressed(KeyCode::A) {
        velocity += camera_transform.left() * camera.speed;
    }
    if keys.pressed(KeyCode::E) {
        velocity += camera_transform.right() * camera.speed;
    }
    camera_transform.translation += velocity;
}

fn handle_mouse_input(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut FlyCamera)>,
    mut mouse: EventReader<MouseMotion>,
) {
    let delta = mouse.iter().fold(Vec2::ZERO, |sum, e| sum + e.delta);
    if delta.is_nan() {
        return;
    }

    let (mut transform, mut camera) = query.get_single_mut().unwrap();

    camera.yaw -= delta.x * camera.sensitivy * time.delta_seconds();
    camera.pitch += (delta.y * camera.sensitivy * time.delta_seconds()).clamp(-89.0, 89.9);

    transform.rotation = Quat::from_axis_angle(Vec3::Y, camera.yaw.to_radians())
        * Quat::from_axis_angle(-Vec3::X, camera.pitch.to_radians());
}
