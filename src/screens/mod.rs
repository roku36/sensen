//! The game's main screen states and transitions between them.

mod gameplay;
mod loading;
mod splash;
mod title;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.register_type::<Screen>();

    app.add_plugins((
        gameplay::plugin,
        loading::plugin,
        splash::plugin,
        title::plugin,
    ));

    // BRP remote control for testing (dev only)
    #[cfg(feature = "dev")]
    {
        app.register_type::<GotoScreen>();
        app.add_systems(Update, handle_goto_screen);

        // --lobby CLI arg: skip directly to lobby screen
        if check_cli_lobby_arg() {
            app.add_systems(
                OnEnter(Screen::Title),
                |mut next: ResMut<NextState<Screen>>| {
                    next.set(Screen::Lobby);
                },
            );
        }
    }
}

/// The game's main screen states.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Reflect)]
pub enum Screen {
    #[default]
    Splash,
    Title,
    Loading,
    Lobby,
    Gameplay,
}

/// Resource to trigger screen transition via BRP.
/// Insert this resource with the target screen to transition.
#[cfg(feature = "dev")]
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct GotoScreen(pub Screen);

/// Check for --lobby CLI arg to skip directly to lobby
#[cfg(feature = "dev")]
pub fn check_cli_lobby_arg() -> bool {
    std::env::args().any(|arg| arg == "--lobby")
}

#[cfg(feature = "dev")]
fn handle_goto_screen(
    mut commands: Commands,
    goto: Option<Res<GotoScreen>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if let Some(goto) = goto {
        next_screen.set(goto.0);
        commands.remove_resource::<GotoScreen>();
    }
}
