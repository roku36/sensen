// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
mod demo;
#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod input;
mod menus;
mod network;
mod screens;
mod theme;

#[cfg(feature = "dev_native")]
use bevy::remote::http::RemoteHttpPlugin;
use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_defer::AsyncPlugin;
use bevy_rich_text3d::{LoadFonts, Text3dPlugin};

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Sensen".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
            MeshPickingPlugin,
            AsyncPlugin::default_settings(),
            Text3dPlugin {
                load_system_fonts: cfg!(not(target_family = "wasm")),
                ..default()
            },
        ));
        // Embed a font for wasm (no system fonts available).
        app.insert_resource(LoadFonts {
            font_embedded: vec![include_bytes!("../assets/fonts/FiraSans-Bold.ttf")],
            ..default()
        });

        app.insert_resource(UiPickingSettings {
            require_markers: true,
        });
        app.insert_resource(MeshPickingSettings {
            require_markers: true,
            ..default()
        });

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            demo::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            game::plugin,
            menus::plugin,
            network::plugin,
            screens::plugin,
            theme::plugin,
        ));

        // Add Bevy Remote Protocol for debugging (native dev only)
        #[cfg(feature = "dev_native")]
        {
            use bevy::remote::RemotePlugin;

            // Parse --brp-port=XXXX from CLI args
            let port = std::env::args()
                .find(|arg| arg.starts_with("--brp-port="))
                .and_then(|arg| arg.strip_prefix("--brp-port=").map(|s| s.to_string()))
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(15702);

            app.add_plugins(RemotePlugin::default());
            app.add_plugins(RemoteHttpPlugin::default().with_port(port));
            info!("BRP listening on port {}", port);
        }

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub(crate) enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;

fn spawn_camera(mut commands: Commands) {
    let camera_transform = Transform::from_xyz(0.0, 12.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        camera_transform,
        IsDefaultUiCamera,
        UiPickingCamera,
        MeshPickingCamera,
    ));
}
