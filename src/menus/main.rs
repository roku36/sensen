//! The main menu (seen on the title screen).

use bevy::prelude::*;

use crate::{asset_tracking::ResourceHandles, menus::Menu, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);

    #[cfg(not(target_family = "wasm"))]
    {
        app.add_systems(OnEnter(Menu::Main), spawn_update_banner);
        app.add_systems(Update, refresh_update_banner.run_if(in_state(Menu::Main)));
    }
}

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Main Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Main),
        #[cfg(not(target_family = "wasm"))]
        children![
            widget::button("Online", enter_lobby_screen),
            widget::button("Solo", enter_loading_or_gameplay_screen),
            widget::button("Settings", open_settings_menu),
            widget::button("Credits", open_credits_menu),
            widget::button("Exit", exit_app),
        ],
        #[cfg(target_family = "wasm")]
        children![
            widget::button("Online", enter_lobby_screen),
            widget::button("Solo", enter_loading_or_gameplay_screen),
            widget::button("Settings", open_settings_menu),
            widget::button("Credits", open_credits_menu),
        ],
    ));
}

// ── Update banner (native only) ──────────────────────────────────────

#[cfg(not(target_family = "wasm"))]
mod update_ui {
    use bevy::prelude::*;

    use crate::{
        auto_update::{TriggerUpdateDownload, UpdateStatus},
        menus::Menu,
        theme::widget,
    };

    #[derive(Component)]
    pub struct UpdateBanner;

    /// Tracks which status the banner was last built for.
    #[derive(Resource, Default, PartialEq, Eq)]
    pub enum BannerState {
        #[default]
        None,
        Available,
        Downloading,
        RestartRequired,
        Error,
    }

    pub fn spawn_update_banner(mut commands: Commands) {
        commands.init_resource::<BannerState>();
    }

    pub fn refresh_update_banner(
        mut commands: Commands,
        status: Res<UpdateStatus>,
        mut banner_state: ResMut<BannerState>,
        existing_banner: Query<Entity, With<UpdateBanner>>,
    ) {
        let desired = match *status {
            UpdateStatus::Available { .. } => BannerState::Available,
            UpdateStatus::Downloading => BannerState::Downloading,
            UpdateStatus::RestartRequired => BannerState::RestartRequired,
            UpdateStatus::Error(_) => BannerState::Error,
            _ => BannerState::None,
        };

        if *banner_state == desired {
            return;
        }
        *banner_state = desired;

        // Despawn old banner
        for entity in &existing_banner {
            commands.entity(entity).despawn();
        }

        match &*status {
            UpdateStatus::Available { latest_version } => {
                let version_text = format!("v{} → v{latest_version}", env!("CARGO_PKG_VERSION"));
                commands.spawn((
                    Name::new("Update Banner"),
                    UpdateBanner,
                    DespawnOnExit(Menu::Main),
                    GlobalZIndex(3),
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        top: Val::Px(40.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![
                        (
                            Text::new("新しいアップデートがあります！"),
                            TextFont::from_font_size(28.0),
                            TextColor(Color::srgb(1.0, 0.9, 0.3)),
                        ),
                        (
                            Text::new(version_text),
                            TextFont::from_font_size(20.0),
                            TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        ),
                        widget::button("アップデート", on_update_click),
                    ],
                ));
            }
            UpdateStatus::Downloading => {
                commands.spawn((
                    Name::new("Update Banner"),
                    UpdateBanner,
                    DespawnOnExit(Menu::Main),
                    GlobalZIndex(3),
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        top: Val::Px(40.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![(
                        Text::new("ダウンロード中..."),
                        TextFont::from_font_size(28.0),
                        TextColor(Color::srgb(0.7, 0.9, 1.0)),
                    ),],
                ));
            }
            UpdateStatus::RestartRequired => {
                commands.spawn((
                    Name::new("Update Banner"),
                    UpdateBanner,
                    DespawnOnExit(Menu::Main),
                    GlobalZIndex(3),
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        top: Val::Px(40.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![
                        (
                            Text::new("アップデート完了！再起動してください"),
                            TextFont::from_font_size(28.0),
                            TextColor(Color::srgb(0.3, 1.0, 0.5)),
                        ),
                        widget::button("再起動", on_restart_click),
                    ],
                ));
            }
            UpdateStatus::Error(msg) => {
                let display = format!("更新チェック失敗: {msg}");
                commands.spawn((
                    Name::new("Update Banner"),
                    UpdateBanner,
                    DespawnOnExit(Menu::Main),
                    GlobalZIndex(3),
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        bottom: Val::Px(20.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                    children![(
                        Text::new(display),
                        TextFont::from_font_size(14.0),
                        TextColor(Color::srgb(0.8, 0.4, 0.4)),
                    ),],
                ));
            }
            _ => {}
        }
    }

    fn on_update_click(_: On<Pointer<Click>>, mut commands: Commands) {
        commands.insert_resource(TriggerUpdateDownload);
    }

    fn on_restart_click(_: On<Pointer<Click>>, mut app_exit: MessageWriter<AppExit>) {
        // Try to restart the app
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::process::Command::new(exe).spawn();
        }
        app_exit.write(AppExit::Success);
    }
}

#[cfg(not(target_family = "wasm"))]
use update_ui::{refresh_update_banner, spawn_update_banner};

// ── Menu button handlers ─────────────────────────────────────────────

fn enter_lobby_screen(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Lobby);
}

fn enter_loading_or_gameplay_screen(
    _: On<Pointer<Click>>,
    resource_handles: Res<ResourceHandles>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if resource_handles.is_all_done() {
        next_screen.set(Screen::Gameplay);
    } else {
        next_screen.set(Screen::Loading);
    }
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: On<Pointer<Click>>, mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
