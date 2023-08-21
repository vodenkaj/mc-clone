use bevy::{
    core_pipeline::contrast_adaptive_sharpening::ContrastAdaptiveSharpeningSettings,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use mc_clone::plugins::{
    camera::camera::{CameraHandlerPlugin, FlyCamera},
    terrain::terrain::TerrainPlugin,
};

fn main() {
    App::new()
        .add_systems(Startup, setup)
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraHandlerPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Update, log_fps)
        .add_systems(Startup, setup_window)
        .add_plugins(TerrainPlugin)
        .run();
}

fn log_fps(diagnostics: Res<DiagnosticsStore>) {
    diagnostics.iter().for_each(|d| {
        let value = d.value();
        if value.is_some() {
            if d.name.eq("fps") {
                //dbg!(d.name.clone(), value);
            }
        }
    })
}

fn setup_window(mut query: Query<&mut Window>) {
    query.iter_mut().for_each(|mut w| {
        w.cursor.visible = false;
    })
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                ..default()
            },
            ContrastAdaptiveSharpeningSettings {
                enabled: true,
                ..default()
            },
        ))
        .insert(FlyCamera::default());
}
