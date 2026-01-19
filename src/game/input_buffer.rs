//! Shared input buffer and offline input processing.

use bevy::prelude::*;

use crate::{
    AppSystems,
    input::{INPUT_DRAW, card_flag, flags_from_keyboard},
};

use super::{
    CardRegistry, Cost, DRAW_COST, DRAW_COUNT, DrawCardsMessage, GameResult, GameplaySystems, Hand,
    LocalPlayer, MAX_HAND_SIZE, PlayCardMessage, is_offline,
};
use crate::screens::Screen;

/// One-frame input buffer that UI and keyboard systems can write into.
#[derive(Resource, Default)]
pub struct PendingInput {
    flags: u16,
}

impl PendingInput {
    pub fn push_flags(&mut self, flags: u16) {
        self.flags |= flags;
    }

    pub fn take_flags(&mut self) -> u16 {
        let flags = self.flags;
        self.flags = 0;
        flags
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<PendingInput>();
    app.add_systems(
        Update,
        capture_keyboard_input
            .in_set(AppSystems::RecordInput)
            .run_if(is_offline)
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(GameResult::Playing)),
    );
    app.add_systems(
        Update,
        apply_pending_inputs
            .in_set(AppSystems::Update)
            .in_set(GameplaySystems::Input)
            .run_if(is_offline)
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(GameResult::Playing)),
    );
}

fn capture_keyboard_input(keyboard: Res<ButtonInput<KeyCode>>, mut pending: ResMut<PendingInput>) {
    pending.push_flags(flags_from_keyboard(&keyboard));
}

fn apply_pending_inputs(
    mut pending: ResMut<PendingInput>,
    mut player_query: Query<(Entity, &Hand, &mut Cost), With<LocalPlayer>>,
    card_registry: Res<CardRegistry>,
    mut play_messages: MessageWriter<PlayCardMessage>,
    mut draw_messages: MessageWriter<DrawCardsMessage>,
) {
    let flags = pending.take_flags();
    if flags == 0 {
        return;
    }

    let Ok((player_entity, hand, mut cost)) = player_query.single_mut() else {
        return;
    };

    apply_local_input_flags(
        flags,
        player_entity,
        hand,
        &mut cost,
        &card_registry,
        &mut draw_messages,
        &mut play_messages,
    );
}

pub(crate) fn apply_local_input_flags(
    flags: u16,
    player_entity: Entity,
    hand: &Hand,
    cost: &mut Cost,
    card_registry: &CardRegistry,
    draw_messages: &mut MessageWriter<DrawCardsMessage>,
    play_messages: &mut MessageWriter<PlayCardMessage>,
) {
    if flags & INPUT_DRAW != 0 && cost.try_spend(DRAW_COST) {
        draw_messages.write(DrawCardsMessage {
            player: player_entity,
            count: DRAW_COUNT,
        });
    }

    for i in 0..MAX_HAND_SIZE {
        let Some(flag) = card_flag(i) else {
            continue;
        };
        if flags & flag != 0 {
            if let Some(card_id) = hand.cards.get(i).copied() {
                if let Some(card_def) = card_registry.get(card_id) {
                    if cost.try_spend(card_def.cost) {
                        play_messages.write(PlayCardMessage {
                            player: player_entity,
                            hand_index: i,
                        });
                    }
                }
            }
            break;
        }
    }
}
