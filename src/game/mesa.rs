//! 3D card table rendering with bevy_la_mesa.

use bevy::{color::Srgba, prelude::*, render::alpha::AlphaMode, transform::TransformSystems};
use bevy_la_mesa::events::{
    AlignCardsInHand, CardHover, CardOut, CardPress, DrawToHand, PlaceCardOnTable, RenderDeck,
};
use bevy_la_mesa::{
    Card as MesaCardComponent, CardMetadata, Deck as MesaDeck, DeckArea, Hand as MesaHand,
    HandArea, LaMesaPlugin, LaMesaPluginSettings, PlayArea,
};
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAlign, TextAnchor, TextAtlas};
use bevy_tweening::TweenAnim;
use std::{cmp::Ordering, num::NonZeroU32};

use super::{
    CardEffect, CardId, CardRegistry, Deck, DeckReshuffledMessage, GameResult, Hand, LocalPlayer,
    Opponent, PendingInput, PlayCardMessage,
};
use crate::{AppSystems, input::card_flag, screens::Screen};

/// Marker component for cards that have effect text added.
#[derive(Component)]
struct CardEffectTextAdded;

#[derive(Resource, Clone)]
struct CardTextMaterial(Handle<StandardMaterial>);

#[derive(Component)]
struct HoveredCard;

const LOCAL_PLAYER_INDEX: usize = 1;
const OPPONENT_PLAYER_INDEX: usize = 2;

const CARD_BACK_IMAGE: &str = "images/splash.png";
const CARD_FRONT_DAMAGE: &str = "images/ducky.png";
const CARD_FRONT_HEAL: &str = "images/ducky.png";
const CARD_FRONT_DRAW: &str = "images/ducky.png";
const HAND_FAN_RADIUS: f32 = 12.0;
const HAND_FAN_MAX_SPAN: f32 = 10.0;
const HAND_FAN_BASE_STEP: f32 = 0.15;
const HAND_FAN_MAX_ANGLE: f32 = 1.2;
const HAND_LAYER_STEP: f32 = 0.01;
const HAND_TILT_STEP: f32 = 0.004;
const HAND_HOVER_LIFT: f32 = 0.35;
const CARD_TEXT_LIFT: f32 = 0.002;

#[derive(Clone, Debug)]
struct MesaCard {
    card_id: CardId,
    front: String,
    back: String,
}

impl Default for MesaCard {
    fn default() -> Self {
        Self {
            card_id: CardId(0),
            front: CARD_FRONT_DAMAGE.to_string(),
            back: CARD_BACK_IMAGE.to_string(),
        }
    }
}

impl CardMetadata for MesaCard {
    type Output = CardId;

    fn front_image_filename(&self) -> String {
        self.front.clone()
    }

    fn back_image_filename(&self) -> String {
        self.back.clone()
    }
}

#[derive(Resource, Default)]
struct MesaScene {
    local_deck: Option<Entity>,
    opponent_deck: Option<Entity>,
    local_play_marker: Option<usize>,
    opponent_play_marker: Option<usize>,
}

impl MesaScene {
    fn deck_for(&self, player: usize) -> Option<Entity> {
        match player {
            LOCAL_PLAYER_INDEX => self.local_deck,
            OPPONENT_PLAYER_INDEX => self.opponent_deck,
            _ => None,
        }
    }

    fn play_marker_for(&self, player: usize) -> Option<usize> {
        match player {
            LOCAL_PLAYER_INDEX => self.local_play_marker,
            OPPONENT_PLAYER_INDEX => self.opponent_play_marker,
            _ => None,
        }
    }
}

#[derive(Resource, Default)]
struct MesaDecksRendered {
    local: bool,
    opponent: bool,
}

#[derive(Resource, Default)]
struct MesaHandMap {
    local: Vec<Entity>,
    opponent: Vec<Entity>,
}

impl MesaHandMap {
    fn hand_mut(&mut self, player: usize) -> Option<&mut Vec<Entity>> {
        match player {
            LOCAL_PLAYER_INDEX => Some(&mut self.local),
            OPPONENT_PLAYER_INDEX => Some(&mut self.opponent),
            _ => None,
        }
    }

    fn hand(&self, player: usize) -> Option<&Vec<Entity>> {
        match player {
            LOCAL_PLAYER_INDEX => Some(&self.local),
            OPPONENT_PLAYER_INDEX => Some(&self.opponent),
            _ => None,
        }
    }
}

/// Tracks the previous hand size for each player to detect draws.
#[derive(Resource, Default)]
struct PreviousHandSizes {
    local: usize,
    opponent: usize,
}

impl PreviousHandSizes {
    fn get(&self, player: usize) -> usize {
        match player {
            LOCAL_PLAYER_INDEX => self.local,
            OPPONENT_PLAYER_INDEX => self.opponent,
            _ => 0,
        }
    }

    fn set(&mut self, player: usize, size: usize) {
        match player {
            LOCAL_PLAYER_INDEX => self.local = size,
            OPPONENT_PLAYER_INDEX => self.opponent = size,
            _ => {}
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_plugins(LaMesaPlugin::<MesaCard>::default());
    app.insert_resource(LaMesaPluginSettings { num_players: 2 });
    app.init_resource::<MesaScene>();
    app.init_resource::<MesaDecksRendered>();
    app.init_resource::<MesaHandMap>();
    app.init_resource::<PreviousHandSizes>();
    app.clear_messages_on_exit::<CardPress>(Screen::Gameplay)
        .clear_messages_on_exit::<RenderDeck<MesaCard>>(Screen::Gameplay)
        .clear_messages_on_exit::<DrawToHand>(Screen::Gameplay)
        .clear_messages_on_exit::<PlaceCardOnTable>(Screen::Gameplay)
        .clear_messages_on_exit::<AlignCardsInHand>(Screen::Gameplay);

    app.add_systems(
        OnEnter(Screen::Gameplay),
        (reset_mesa_state, spawn_mesa_scene).chain(),
    );
    app.add_systems(OnExit(Screen::Gameplay), reset_mesa_state);

    app.add_systems(
        Update,
        (
            render_initial_decks,
            handle_deck_reshuffle,
            sync_hand_to_mesa,
            sync_played_cards,
            add_effect_text_to_cards,
            track_hand_hover,
        )
            .chain()
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        PostUpdate,
        fan_hand_layout
            .before(TransformSystems::Propagate)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_systems(
        Update,
        tag_mesa_cards_for_cleanup
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );

    app.add_systems(
        Update,
        handle_card_press_input
            .in_set(AppSystems::RecordInput)
            .run_if(in_state(Screen::Gameplay))
            .run_if(in_state(GameResult::Playing)),
    );
}

fn reset_mesa_state(
    mut scene: ResMut<MesaScene>,
    mut rendered: ResMut<MesaDecksRendered>,
    mut hand_map: ResMut<MesaHandMap>,
    mut prev_sizes: ResMut<PreviousHandSizes>,
) {
    *scene = MesaScene::default();
    *rendered = MesaDecksRendered::default();
    *hand_map = MesaHandMap::default();
    *prev_sizes = PreviousHandSizes::default();
}

fn rotate_around_origin_y(transform: Transform) -> Transform {
    let rotation = Quat::from_rotation_y(std::f32::consts::PI);
    Transform {
        translation: rotation * transform.translation,
        rotation: rotation * transform.rotation,
        ..transform
    }
}

fn spawn_mesa_scene(
    mut commands: Commands,
    mut scene: ResMut<MesaScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("spawn_mesa_scene called!");

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 800.0,
        affects_lightmapped_meshes: true,
    });

    commands.spawn((
        Name::new("Mesa Sun"),
        DirectionalLight {
            illuminance: 9000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(6.0, 12.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        DespawnOnExit(Screen::Gameplay),
    ));

    commands.spawn((
        Name::new("Mesa Table"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(30.0, 20.0).subdivisions(10))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.09, 0.12),
            perceptual_roughness: 0.9,
            metallic: 0.1,
            ..default()
        })),
        Transform::default(),
        DespawnOnExit(Screen::Gameplay),
    ));

    let text_material = materials.add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        ..default()
    });
    commands.insert_resource(CardTextMaterial(text_material));

    let local_deck_transform = Transform::from_translation(Vec3::new(-6.0, 0.01, 2.5));
    let opponent_deck_transform = rotate_around_origin_y(local_deck_transform);
    let local_hand_transform = Transform::from_translation(Vec3::new(0.0, 1.6, 6.5))
        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_4));
    let opponent_hand_transform = rotate_around_origin_y(local_hand_transform);
    let local_play_transform = Transform::from_translation(Vec3::new(0.0, 0.02, 1.5));
    let opponent_play_transform = rotate_around_origin_y(local_play_transform);

    let local_deck = commands
        .spawn((
            Name::new("Deck Area Local"),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(2.5, 3.5).subdivisions(8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.12, 0.16),
                perceptual_roughness: 0.85,
                ..default()
            })),
            local_deck_transform,
            DeckArea {
                marker: LOCAL_PLAYER_INDEX,
            },
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();

    let opponent_deck = commands
        .spawn((
            Name::new("Deck Area Opponent"),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(2.5, 3.5).subdivisions(8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.12, 0.12, 0.16),
                perceptual_roughness: 0.85,
                ..default()
            })),
            opponent_deck_transform,
            DeckArea {
                marker: OPPONENT_PLAYER_INDEX,
            },
            DespawnOnExit(Screen::Gameplay),
        ))
        .id();

    commands.spawn((
        Name::new("Hand Area Local"),
        local_hand_transform,
        HandArea {
            player: LOCAL_PLAYER_INDEX,
        },
        DespawnOnExit(Screen::Gameplay),
    ));

    commands.spawn((
        Name::new("Hand Area Opponent"),
        opponent_hand_transform,
        HandArea {
            player: OPPONENT_PLAYER_INDEX,
        },
        DespawnOnExit(Screen::Gameplay),
    ));

    commands.spawn((
        Name::new("Play Area Local"),
        local_play_transform,
        PlayArea {
            marker: LOCAL_PLAYER_INDEX,
            player: LOCAL_PLAYER_INDEX,
        },
        DespawnOnExit(Screen::Gameplay),
    ));

    commands.spawn((
        Name::new("Play Area Opponent"),
        opponent_play_transform,
        PlayArea {
            marker: OPPONENT_PLAYER_INDEX,
            player: OPPONENT_PLAYER_INDEX,
        },
        DespawnOnExit(Screen::Gameplay),
    ));

    scene.local_deck = Some(local_deck);
    scene.opponent_deck = Some(opponent_deck);
    scene.local_play_marker = Some(LOCAL_PLAYER_INDEX);
    scene.opponent_play_marker = Some(OPPONENT_PLAYER_INDEX);
}

fn render_initial_decks(
    scene: Res<MesaScene>,
    mut rendered: ResMut<MesaDecksRendered>,
    registry: Res<CardRegistry>,
    local_query: Query<&Deck, With<LocalPlayer>>,
    opponent_query: Query<&Deck, With<Opponent>>,
    deck_cards: Query<(Entity, &MesaDeck)>,
    children_query: Query<&Children>,
    mut render_deck: MessageWriter<RenderDeck<MesaCard>>,
    mut commands: Commands,
) {
    if !rendered.local {
        let Some(deck_entity) = scene.local_deck else {
            info!("render_initial_decks: no local deck entity");
            return;
        };
        let Ok(deck) = local_query.single() else {
            info!("render_initial_decks: no local player deck component");
            return;
        };
        info!(
            "render_initial_decks: rendering local deck with {} cards",
            deck.cards.len()
        );
        rebuild_deck_visual(
            deck_entity,
            LOCAL_PLAYER_INDEX,
            &deck.cards,
            &registry,
            &deck_cards,
            &children_query,
            &mut commands,
            &mut render_deck,
        );
        rendered.local = true;
    }

    if !rendered.opponent {
        let Some(deck_entity) = scene.opponent_deck else {
            return;
        };
        let Ok(deck) = opponent_query.single() else {
            return;
        };
        rebuild_deck_visual(
            deck_entity,
            OPPONENT_PLAYER_INDEX,
            &deck.cards,
            &registry,
            &deck_cards,
            &children_query,
            &mut commands,
            &mut render_deck,
        );
        rendered.opponent = true;
    }
}

fn handle_deck_reshuffle(
    mut reshuffle: MessageReader<DeckReshuffledMessage>,
    scene: Res<MesaScene>,
    registry: Res<CardRegistry>,
    deck_cards: Query<(Entity, &MesaDeck)>,
    children_query: Query<&Children>,
    local_query: Query<Entity, With<LocalPlayer>>,
    opponent_query: Query<Entity, With<Opponent>>,
    mut render_deck: MessageWriter<RenderDeck<MesaCard>>,
    mut commands: Commands,
) {
    for message in reshuffle.read() {
        let Some(player_index) =
            player_index_for_entity(message.player, &local_query, &opponent_query)
        else {
            continue;
        };
        let Some(deck_entity) = scene.deck_for(player_index) else {
            continue;
        };

        rebuild_deck_visual(
            deck_entity,
            player_index,
            &message.deck,
            &registry,
            &deck_cards,
            &children_query,
            &mut commands,
            &mut render_deck,
        );
    }
}

/// Sync the game logic Hand component to 3D mesa rendering.
/// This watches for changes in Hand.cards.len() and triggers draws accordingly.
fn sync_hand_to_mesa(
    local_player: Query<&Hand, With<LocalPlayer>>,
    opponent_player: Query<&Hand, With<Opponent>>,
    scene: Res<MesaScene>,
    deck_cards_query: Query<(Entity, &MesaDeck, &Transform)>,
    mut hand_map: ResMut<MesaHandMap>,
    mut prev_sizes: ResMut<PreviousHandSizes>,
    mut draw_to_hand: MessageWriter<DrawToHand>,
) {
    let players: [(usize, Option<&Hand>); 2] = [
        (LOCAL_PLAYER_INDEX, local_player.single().ok()),
        (OPPONENT_PLAYER_INDEX, opponent_player.single().ok()),
    ];

    for (player_index, maybe_hand) in players {
        let Some(hand) = maybe_hand else { continue };
        let current_size = hand.cards.len();
        let previous_size = prev_sizes.get(player_index);

        if current_size > previous_size {
            // Cards were added - need to draw to mesa
            let cards_to_draw = current_size - previous_size;

            let Some(deck_entity) = scene.deck_for(player_index) else {
                continue;
            };

            // Check if we have enough deck cards visually
            let available = gather_deck_cards(&deck_cards_query);
            let available_count = available.get(&player_index).map(|v| v.len()).unwrap_or(0);

            if available_count >= cards_to_draw {
                // Get visual hand card count (from MesaHandMap)
                let visual_hand_size = hand_map.hand(player_index).map(|h| h.len()).unwrap_or(0);

                // Update MesaHandMap with newly drawn card entities
                if let Some(cards) = available.get(&player_index) {
                    if let Some(mesa_hand) = hand_map.hand_mut(player_index) {
                        mesa_hand.extend(cards.iter().take(cards_to_draw).copied());
                    }
                }

                // bevy_la_mesa expects num_cards = target hand size
                draw_to_hand.write(DrawToHand {
                    deck_entity,
                    num_cards: visual_hand_size + cards_to_draw,
                    player: player_index,
                });

                info!(
                    "sync_hand_to_mesa: player {} drew {} cards (hand: {} -> {})",
                    player_index, cards_to_draw, previous_size, current_size
                );
            }
        }

        // Update previous size to current
        prev_sizes.set(player_index, current_size);
    }
}

fn sync_played_cards(
    mut play_messages: MessageReader<PlayCardMessage>,
    local_query: Query<Entity, With<LocalPlayer>>,
    opponent_query: Query<Entity, With<Opponent>>,
    scene: Res<MesaScene>,
    mut hand_map: ResMut<MesaHandMap>,
    mut place_card: MessageWriter<PlaceCardOnTable>,
    mut align_hand: MessageWriter<AlignCardsInHand>,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    for message in play_messages.read() {
        let Some(player_index) =
            player_index_for_entity(message.player, &local_query, &opponent_query)
        else {
            continue;
        };

        let Some(hand) = hand_map.hand_mut(player_index) else {
            continue;
        };
        if message.hand_index >= hand.len() {
            continue;
        }

        let card_entity = hand.remove(message.hand_index);
        if let Some(marker) = scene.play_marker_for(player_index) {
            place_card.write(PlaceCardOnTable {
                card_entity,
                marker,
                player: player_index,
            });
        } else {
            despawn_entity_recursive(card_entity, &children_query, &mut commands);
        }

        align_hand.write(AlignCardsInHand {
            player: player_index,
        });
    }
}

fn handle_card_press_input(
    mut card_press: MessageReader<CardPress>,
    cards_in_hand: Query<(Entity, &MesaHand, &MesaCardComponent<MesaCard>, &Transform)>,
    parents: Query<&ChildOf>,
    local_hand_query: Query<&Hand, With<LocalPlayer>>,
    hand_map: Res<MesaHandMap>,
    mut pending: ResMut<PendingInput>,
) {
    for press in card_press.read() {
        info!("CardPress received for entity {:?}", press.entity);

        // Check if the pressed entity is in the local player's hand
        let Some(card_entity) = resolve_pressed_card_entity(press.entity, &cards_in_hand, &parents)
        else {
            info!("  -> No matching Mesa card entity found for press target");
            continue;
        };
        let Ok((_, hand, _card, _transform)) = cards_in_hand.get(card_entity) else {
            info!("  -> Resolved entity missing MesaHand component");
            continue;
        };

        // Only handle local player's cards
        if hand.player != LOCAL_PLAYER_INDEX {
            continue;
        }

        // Get the game logic hand to verify index is valid
        let Ok(game_hand) = local_hand_query.single() else {
            info!("  -> No game logic Hand found for local player");
            continue;
        };

        let Some(local_hand_cards) = hand_map.hand(LOCAL_PLAYER_INDEX) else {
            continue;
        };

        // Find the index of the pressed card based on the game-logic order
        let Some(index) = local_hand_cards.iter().position(|e| *e == card_entity) else {
            continue;
        };

        // Verify index is within game logic hand bounds
        if index >= game_hand.cards.len() {
            info!(
                "  -> Card index {} is out of bounds (game hand has {} cards)",
                index,
                game_hand.cards.len()
            );
            continue;
        }

        if let Some(flag) = card_flag(index) {
            info!(
                "  -> Card at index {} pressed, pushing flag {} (game hand: {:?})",
                index, flag, game_hand.cards
            );
            pending.push_flags(flag);
        }
    }
}

fn resolve_pressed_card_entity(
    entity: Entity,
    cards_in_hand: &Query<(Entity, &MesaHand, &MesaCardComponent<MesaCard>, &Transform)>,
    parents: &Query<&ChildOf>,
) -> Option<Entity> {
    let mut current = entity;
    loop {
        if cards_in_hand.get(current).is_ok() {
            return Some(current);
        }

        let Ok(child_of) = parents.get(current) else {
            return None;
        };
        current = child_of.parent();
    }
}

fn track_hand_hover(
    mut commands: Commands,
    mut hover: MessageReader<CardHover>,
    mut out: MessageReader<CardOut>,
    cards_in_hand: Query<(Entity, &MesaHand, &MesaCardComponent<MesaCard>, &Transform)>,
    parents: Query<&ChildOf>,
) {
    for event in hover.read() {
        let Some(card_entity) = resolve_pressed_card_entity(event.entity, &cards_in_hand, &parents)
        else {
            continue;
        };
        let Ok((_, hand, _, _)) = cards_in_hand.get(card_entity) else {
            continue;
        };
        if hand.player != LOCAL_PLAYER_INDEX {
            continue;
        }

        commands
            .entity(card_entity)
            .insert(HoveredCard)
            .remove::<TweenAnim>();
    }

    for event in out.read() {
        let Some(card_entity) = resolve_pressed_card_entity(event.entity, &cards_in_hand, &parents)
        else {
            continue;
        };
        commands
            .entity(card_entity)
            .remove::<HoveredCard>()
            .remove::<TweenAnim>();
    }
}

fn gather_deck_cards(
    deck_cards_query: &Query<(Entity, &MesaDeck, &Transform)>,
) -> std::collections::HashMap<usize, Vec<Entity>> {
    let mut deck_cards: std::collections::HashMap<usize, Vec<(Entity, f32)>> =
        std::collections::HashMap::new();

    for (entity, deck, transform) in deck_cards_query.iter() {
        deck_cards
            .entry(deck.marker)
            .or_default()
            .push((entity, transform.translation.y));
    }

    let mut result = std::collections::HashMap::new();
    for (marker, mut cards) in deck_cards {
        cards.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result.insert(
            marker,
            cards.into_iter().map(|(entity, _)| entity).collect(),
        );
    }

    result
}

fn rebuild_deck_visual(
    deck_entity: Entity,
    marker: usize,
    deck_cards: &[CardId],
    registry: &CardRegistry,
    existing_cards: &Query<(Entity, &MesaDeck)>,
    children_query: &Query<&Children>,
    commands: &mut Commands,
    render_deck: &mut MessageWriter<RenderDeck<MesaCard>>,
) {
    for (entity, deck) in existing_cards.iter() {
        if deck.marker == marker {
            despawn_entity_recursive(entity, children_query, commands);
        }
    }

    let deck = deck_cards
        .iter()
        .map(|card_id| mesa_card_from_id(*card_id, registry))
        .collect::<Vec<_>>();

    render_deck.write(RenderDeck::<MesaCard> { deck_entity, deck });
}

fn despawn_entity_recursive(
    entity: Entity,
    children_query: &Query<&Children>,
    commands: &mut Commands,
) {
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            despawn_entity_recursive(child, children_query, commands);
        }
    }
    commands.entity(entity).despawn();
}

fn mesa_card_from_id(card_id: CardId, registry: &CardRegistry) -> MesaCard {
    let front = registry
        .get(card_id)
        .map(|def| match def.effect {
            CardEffect::Damage(_) => CARD_FRONT_DAMAGE,
            CardEffect::Heal(_) => CARD_FRONT_HEAL,
            CardEffect::Draw(_) => CARD_FRONT_DRAW,
        })
        .unwrap_or(CARD_FRONT_DAMAGE)
        .to_string();

    MesaCard {
        card_id,
        front,
        back: CARD_BACK_IMAGE.to_string(),
    }
}

fn player_index_for_entity(
    entity: Entity,
    local_query: &Query<Entity, With<LocalPlayer>>,
    opponent_query: &Query<Entity, With<Opponent>>,
) -> Option<usize> {
    if local_query.contains(entity) {
        Some(LOCAL_PLAYER_INDEX)
    } else if opponent_query.contains(entity) {
        Some(OPPONENT_PLAYER_INDEX)
    } else {
        None
    }
}

/// Add effect text to cards that don't have it yet.
fn add_effect_text_to_cards(
    mut commands: Commands,
    registry: Res<CardRegistry>,
    text_material: Option<Res<CardTextMaterial>>,
    cards_without_text: Query<(Entity, &MesaCardComponent<MesaCard>), Without<CardEffectTextAdded>>,
) {
    let Some(text_material) = text_material else {
        return;
    };
    let text_material = text_material.0.clone();

    for (entity, card) in cards_without_text.iter() {
        let card_id = card.data.card_id;
        let Some(card_def) = registry.get(card_id) else {
            commands.entity(entity).insert(CardEffectTextAdded);
            continue;
        };

        // Build effect text
        let effect_text = match &card_def.effect {
            CardEffect::Damage(amount) => format!("DMG\n{:.0}", amount),
            CardEffect::Heal(amount) => format!("HEAL\n{:.0}", amount),
            CardEffect::Draw(count) => format!("DRAW\n{}", count),
        };

        let cost_text = format!("{:.0}", card_def.cost);

        // Determine color based on effect type
        let effect_color = match &card_def.effect {
            CardEffect::Damage(_) => Srgba::rgb(1.0, 0.3, 0.3),
            CardEffect::Heal(_) => Srgba::rgb(0.3, 1.0, 0.3),
            CardEffect::Draw(_) => Srgba::rgb(0.3, 0.5, 1.0),
        };

        commands.entity(entity).insert(CardEffectTextAdded);

        // Add effect text as a child entity (positioned on top of the card front face)
        commands.entity(entity).with_children(|parent| {
            // Effect text (center of card)
            parent.spawn((
                Name::new("Card Effect Text"),
                Text3d::new(effect_text),
                Text3dStyling {
                    size: 32.0,
                    color: effect_color,
                    stroke: NonZeroU32::new(3),
                    stroke_color: Srgba::BLACK,
                    align: TextAlign::Center,
                    anchor: TextAnchor::CENTER,
                    line_height: 0.9,
                    world_scale: Some(Vec2::splat(0.6)),
                    layer_offset: 0.001,
                    ..default()
                },
                Mesh3d::default(),
                MeshMaterial3d(text_material.clone()),
                Transform::from_xyz(0.0, CARD_TEXT_LIFT, 0.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ));

            // Cost text (top left corner)
            parent.spawn((
                Name::new("Card Cost Text"),
                Text3d::new(cost_text),
                Text3dStyling {
                    size: 24.0,
                    color: Srgba::rgb(1.0, 0.9, 0.2),
                    stroke: NonZeroU32::new(2),
                    stroke_color: Srgba::BLACK,
                    align: TextAlign::Center,
                    anchor: TextAnchor::CENTER,
                    world_scale: Some(Vec2::splat(0.35)),
                    layer_offset: 0.001,
                    ..default()
                },
                Mesh3d::default(),
                MeshMaterial3d(text_material.clone()),
                Transform::from_xyz(-0.9, CARD_TEXT_LIFT, -1.3)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ));
        });
    }
}

fn tag_mesa_cards_for_cleanup(
    mut commands: Commands,
    new_cards: Query<(Entity, Option<&Pickable>), Added<MesaCardComponent<MesaCard>>>,
) {
    for (entity, pickable) in &new_cards {
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert(DespawnOnExit(Screen::Gameplay));
        if pickable.is_none() {
            entity_commands.insert(Pickable::default());
        }
    }
}

fn fan_hand_layout(
    hand_areas: Query<(&HandArea, &Transform)>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    hand_map: Res<MesaHandMap>,
    mut cards: ParamSet<(
        Query<(Entity, &MesaHand, &Transform), Without<HandArea>>,
        Query<
            (
                Entity,
                &MesaHand,
                &mut Transform,
                &mut MesaCardComponent<MesaCard>,
                Option<&TweenAnim>,
                Option<&HoveredCard>,
            ),
            Without<HandArea>,
        >,
    )>,
) {
    let camera_position = camera_query
        .iter()
        .next()
        .map(|transform| transform.translation())
        .unwrap_or(Vec3::ZERO);

    for (hand_area, hand_transform) in &hand_areas {
        let mut hand_cards: Vec<Entity> = hand_map
            .hand(hand_area.player)
            .map(|order| {
                order
                    .iter()
                    .filter_map(|entity| cards.p0().get(*entity).ok().map(|_| *entity))
                    .collect()
            })
            .unwrap_or_else(|| {
                let mut collected: Vec<(Entity, f32)> = cards
                    .p0()
                    .iter()
                    .filter(|(_, hand, _)| hand.player == hand_area.player)
                    .map(|(entity, _, transform)| (entity, transform.translation.x))
                    .collect();
                collected.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
                collected.into_iter().map(|(entity, _)| entity).collect()
            });

        if hand_cards.is_empty() {
            continue;
        }

        if hand_area.player == OPPONENT_PLAYER_INDEX {
            hand_cards.reverse();
        }

        let card_count = hand_cards.len();
        let desired_angle = HAND_FAN_BASE_STEP * (card_count.saturating_sub(1) as f32);
        let span_ratio = (HAND_FAN_MAX_SPAN * 0.5 / HAND_FAN_RADIUS).clamp(0.0, 1.0);
        let max_angle_from_span = 2.0 * span_ratio.asin();
        let fan_angle = desired_angle
            .min(max_angle_from_span)
            .min(HAND_FAN_MAX_ANGLE);
        let angle_step = if card_count > 1 {
            fan_angle / (card_count as f32 - 1.0)
        } else {
            0.0
        };

        let mut targets = Vec::with_capacity(card_count);
        for (index, entity) in hand_cards.iter().enumerate() {
            let angle = -fan_angle * 0.5 + angle_step * index as f32;
            let local_offset = Vec3::new(
                angle.sin() * HAND_FAN_RADIUS,
                0.0,
                HAND_FAN_RADIUS - angle.cos() * HAND_FAN_RADIUS,
            );
            let world_offset = hand_transform.rotation * local_offset;
            let translation = hand_transform.translation + world_offset;
            let rotation = hand_transform.rotation * Quat::from_rotation_y(-angle);
            targets.push((*entity, translation, rotation));
        }

        let to_camera = (camera_position - hand_transform.translation).normalize_or_zero();
        let stack_direction = if to_camera.length_squared() > 0.0 {
            to_camera
        } else {
            hand_transform.rotation * Vec3::Y
        };

        targets.sort_by(|a, b| a.1.x.partial_cmp(&b.1.x).unwrap_or(Ordering::Equal));

        for (rank, (entity, translation, rotation)) in targets.into_iter().enumerate() {
            let layered_translation =
                translation + stack_direction * (rank as f32 * HAND_LAYER_STEP);
            let tilt = (rank as f32 - (card_count as f32 - 1.0) * 0.5) * HAND_TILT_STEP;
            let final_rotation = rotation * Quat::from_rotation_x(tilt);
            if let Ok((_, _, mut transform, mut card, tween, hovered)) = cards.p1().get_mut(entity)
            {
                let base_transform = Transform {
                    translation: layered_translation,
                    rotation: final_rotation,
                    scale: transform.scale,
                };
                card.transform = Some(base_transform);
                let hover_offset = if hovered.is_some() {
                    stack_direction * HAND_HOVER_LIFT
                } else {
                    Vec3::ZERO
                };
                if tween.is_none() {
                    transform.translation = layered_translation + hover_offset;
                    transform.rotation = final_rotation;
                }
            }
        }
    }
}
