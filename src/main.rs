use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_pancam::{DirectionKeys, PanCam, PanCamPlugin};
use iyes_perf_ui::prelude::*;
use nebulon::{player::PlayerPlugin, terrain::TerrainPlugin, BG_COLOR, WH, WW};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        mode: bevy::window::WindowMode::Windowed,
                        resolution: (WW as f32, WH as f32).into(),
                        title: "ProcGen".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgba_u8(
            BG_COLOR.0, BG_COLOR.1, BG_COLOR.2, 0,
        )))
        .add_plugins(PanCamPlugin::default())
        .add_plugins(PerfUiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(TerrainPlugin)
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, setup_perf_ui)
        .run();
}

fn setup_perf_ui(mut commands: Commands) {
    commands.spawn((PerfUiEntryFPS::default(),));
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        PanCam {
            move_keys: DirectionKeys::NONE,
            grab_buttons: vec![],
            zoom_to_cursor: false,
            min_scale: 0.1,
            max_scale: 10.,
            ..default()
        },
        Camera2d::default(),
    ));
}
