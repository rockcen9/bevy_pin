use bevy::{
    input_focus::InputFocus,
    text::{EditableText, FontCx, LayoutCx},
};

use crate::manager::entity_filter::fetch::DiscoveredComponents;
use crate::prelude::*;
use crate::ui_layout::theme::widgets::{ScrollableContainer, show_global_message};
use crate::utils::parse_json_value;
use crate::{manager::connection::ServerUrl, prelude::entity_card::*};

use super::layout::render_pincard;

// ── Components ─────────────────────────────────────────────────────────────────

/// Drives periodic BRP polling for a pincard's component data.
#[derive(Component)]
pub(super) struct EntityCardPollTimer(pub(super) Timer);

/// Context stored on a `brp_list_components` request entity.
#[derive(Component)]
pub(super) struct EntityCardListCtx {
    pub(super) entity_id: u64,
}

/// Context stored on a `brp_get_components` request entity.
#[derive(Component)]
pub(super) struct EntityCardGetCtx {
    pub(super) entity_id: u64,
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            init_entity_cards,
            trigger_initial_fetch,
            tick_pincard_polls,
            submit_pincard_field,
            handle_pincard_insert_submit,
            handle_pincard_remove_component_button,
            handle_pincard_despawn_button,
        ),
    );
}

// ── Polling systems ───────────────────────────────────────────────────────────

pub(super) fn init_entity_cards(
    added_cards: Query<(Entity, &EntityCard, &Children), Added<EntityCard>>,
    headers: Query<Entity, With<EntityCardHeader>>,
    known: Res<EntityCardKnownMarkerComponents>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    for (card_entity, entity_card_data, children) in &added_cards {
        init_single_entity_card(
            card_entity,
            entity_card_data,
            children,
            &headers,
            &known,
            &server_url,
            &mut commands,
        );
    }
}

fn init_single_entity_card(
    card_entity: Entity,
    entity_card_data: &EntityCard,
    children: &Children,
    headers: &Query<Entity, With<EntityCardHeader>>,
    known: &Res<EntityCardKnownMarkerComponents>,
    server_url: &Res<ServerUrl>,
    commands: &mut Commands,
) {
    use super::resize::{
        EntityCardResizeCornerBL, EntityCardResizeCornerBR, EntityCardResizeCornerTL,
        EntityCardResizeCornerTR, EntityCardResizeHandle, EntityCardResizeHandleBottom,
        EntityCardResizeHandleLeft, EntityCardResizeHandleTop,
    };
    use crate::ui_layout::theme::widgets::scrollable_list;

    let entity_id = entity_card_data.entity_id;
    let key = entity_card_key(entity_id);

    // Find the EntityCardHeader child and insert PinCardTitle + despawn button.
    if let Some(header) = children.iter().find(|e| headers.contains(*e)) {
        let despawn_btn = commands
            .spawn_scene(remove_button(EntityCardDespawnButton { entity_id }))
            .id();
        commands.entity(header).insert(EntityCardTitle(entity_id));
        commands.entity(header).insert_children(0, &[despawn_btn]);
    }

    // Add the scrollable body as a child scene.
    let scroll = commands.spawn_scene(scrollable_list(key)).id();
    commands.entity(card_entity).add_child(scroll);

    // Add resize handles as plain bundles.
    commands.entity(card_entity).with_children(|p| {
        p.spawn((
            EntityCardResizeHandle,
            Pickable::default(),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(6.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ));
        p.spawn((
            EntityCardResizeHandleBottom,
            Pickable::default(),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(6.0),
                ..default()
            },
        ));
        p.spawn((
            EntityCardResizeHandleLeft,
            Pickable::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(6.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ));
        p.spawn((
            EntityCardResizeHandleTop,
            Pickable::default(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(6.0),
                ..default()
            },
        ));
        p.spawn((
            EntityCardResizeCornerBR,
            Pickable::default(),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(10.0),
                height: Val::Px(10.0),
                ..default()
            },
        ));
        p.spawn((
            EntityCardResizeCornerBL,
            Pickable::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(10.0),
                height: Val::Px(10.0),
                ..default()
            },
        ));
        p.spawn((
            EntityCardResizeCornerTR,
            Pickable::default(),
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(10.0),
                height: Val::Px(10.0),
                ..default()
            },
        ));
        p.spawn((
            EntityCardResizeCornerTL,
            Pickable::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(10.0),
                height: Val::Px(10.0),
                ..default()
            },
        ));
    });

    commands
        .entity(card_entity)
        .insert(EntityCardPollTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )));

    // Populate the known marker components once (on first card add).
    if known.0.is_empty() {
        let req = commands.brp_registry_schema(&server_url.0, json!({}));
        commands
            .entity(req)
            .observe(
                |trigger: On<Add, RpcResponse<BrpSchema>>,
                 query: Query<&RpcResponse<BrpSchema>>,
                 mut known: ResMut<EntityCardKnownMarkerComponents>,
                 mut commands: Commands| {
                    let entity = trigger.entity;
                    let Ok(response) = query.get(entity) else {
                        commands.entity(entity).despawn();
                        return;
                    };
                    if let Ok(data) = &response.data {
                        if let Some(obj) = data.result.as_object() {
                            for (type_path, schema) in obj {
                                let is_marker = schema
                                    .get("properties")
                                    .and_then(|p| p.as_object())
                                    .map(|p| p.is_empty())
                                    .unwrap_or(true);
                                if !is_marker {
                                    continue;
                                }
                                let short = type_path
                                    .split("::")
                                    .last()
                                    .unwrap_or(type_path)
                                    .to_string();
                                known.0.entry(short).or_insert_with(|| type_path.clone());
                                known
                                    .0
                                    .entry(type_path.clone())
                                    .or_insert_with(|| type_path.clone());
                            }
                            debug!(
                                "PinCardKnownMarkerComponents: {} marker types loaded",
                                known.0.len() / 2
                            );
                        }
                    }
                    commands.entity(entity).despawn();
                },
            )
            .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
                commands.entity(trigger.entity).despawn();
            });
    }
}

pub(super) fn trigger_initial_fetch(
    added: Query<(Entity, &ScrollableContainer), Added<ScrollableContainer>>,
    entity_cards: Query<&EntityCard>,
    child_of: Query<&ChildOf>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    for (container_entity, container) in &added {
        let Some(entity_id) = parse_pincard_key(&container.0) else {
            continue;
        };
        if entity_cards.iter().any(|ec| ec.entity_id == entity_id) {
            spawn_pincard_fetch(entity_id, &server_url.0, &mut commands);

            // ScrollableContainer IS the inner column node — one hop up is the scroll outer row.
            if let Ok(scroll_outer) = child_of.get(container_entity) {
                commands
                    .entity(scroll_outer.0)
                    .insert(EntityCardScrollOuter { entity_id });
            }
        }
    }
}

pub(super) fn tick_pincard_polls(
    time: Res<Time>,
    mut cards: Query<(&EntityCard, &mut EntityCardPollTimer)>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    for (card, mut timer) in &mut cards {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            spawn_pincard_fetch(card.entity_id, &server_url.0, &mut commands);
        }
    }
}

pub fn spawn_pincard_fetch(entity_id: u64, url: &str, commands: &mut Commands) {
    let req = commands.brp_list_components(url, entity_id);
    commands
        .entity(req)
        .insert(EntityCardListCtx { entity_id })
        .observe(on_pincard_list_response)
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            commands.entity(trigger.entity).despawn();
        });
}

fn on_pincard_list_response(
    trigger: On<Add, RpcResponse<BrpListComponents>>,
    q: Query<(&RpcResponse<BrpListComponents>, &EntityCardListCtx)>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    let ecs_entity = trigger.entity;
    let Ok((response, ctx)) = q.get(ecs_entity) else {
        commands.entity(ecs_entity).despawn();
        return;
    };
    match &response.data {
        Err(e) => {
            debug!(
                "pincard: list_components failed for entity #{}: {e}",
                ctx.entity_id
            );
        }
        Ok(data) => {
            if !data.result.is_empty() {
                let entity_id = ctx.entity_id;
                let type_paths = data.result.clone();
                let req = commands.brp_get_components(&server_url.0, entity_id, &type_paths, false);
                commands
                    .entity(req)
                    .insert(EntityCardGetCtx { entity_id })
                    .observe(on_pincard_get_response)
                    .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
                        commands.entity(trigger.entity).despawn();
                    });
            }
        }
    }
    commands.entity(ecs_entity).despawn();
}

fn on_pincard_get_response(
    trigger: On<Add, RpcResponse<BrpGetComponents>>,
    q: Query<(&RpcResponse<BrpGetComponents>, &EntityCardGetCtx)>,
    containers: Query<(Entity, &ScrollableContainer)>,
    expand_state: Res<EntityCardExpandState>,
    children_cache: Res<EntityCardChildrenCache>,
    parent_cache: Res<super::childof::EntityCardParentCache>,
    discovered_components: Res<DiscoveredComponents>,
    mut cache: ResMut<EntityCardDataCache>,
    input_focus: Res<InputFocus>,
    editable_fields: Query<&EditableEntityCardField>,
    insert_fields: Query<&EntityCardInsertField>,
    mut commands: Commands,
) {
    let ecs_entity = trigger.entity;
    let Ok((response, ctx)) = q.get(ecs_entity) else {
        commands.entity(ecs_entity).despawn();
        return;
    };
    match &response.data {
        Err(e) => {
            debug!(
                "pincard: get_components failed for entity #{}: {e}",
                ctx.entity_id
            );
        }
        Ok(data) => {
            let key = entity_card_key(ctx.entity_id);
            let matching: Vec<Entity> = containers
                .iter()
                .filter(|(_, c)| c.0 == key)
                .map(|(e, _)| e)
                .collect();
            if !matching.is_empty() {
                if let Some(map) = data.result["components"].as_object() {
                    let new_data: serde_json::Map<_, _> =
                        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                    cache.0.insert(ctx.entity_id, new_data);

                    // Skip re-render while the user is editing any input on this card.
                    let focused_on_this_card = input_focus
                        .get()
                        .map(|e| {
                            editable_fields
                                .get(e)
                                .map(|f| f.entity_id)
                                .ok()
                                .or_else(|| insert_fields.get(e).map(|f| f.entity_id).ok())
                                == Some(ctx.entity_id)
                        })
                        .unwrap_or(false);

                    if !focused_on_this_card {
                        let cached = cache.0.get(&ctx.entity_id).unwrap();
                        for container_entity in matching {
                            render_pincard(
                                &mut commands,
                                container_entity,
                                ctx.entity_id,
                                cached,
                                &expand_state,
                                &children_cache,
                                &parent_cache,
                                &discovered_components,
                            );
                        }
                    }
                }
            }
        }
    }
    commands.entity(ecs_entity).despawn();
}

pub(super) fn handle_pincard_remove_component_button(
    buttons: Query<
        (&Interaction, &EntityCardRemoveComponentButton),
        (Changed<Interaction>, With<Button>),
    >,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    for (interaction, btn) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        debug!(
            "pincard remove: '{}' from entity #{}",
            btn.type_path, btn.entity_id
        );
        let entity_id = btn.entity_id;
        let url = server_url.0.clone();
        let req =
            commands.brp_remove_components(&server_url.0, btn.entity_id, &[btn.type_path.clone()]);
        commands
            .entity(req)
            .observe(
                move |trigger: On<Add, RpcResponse<BrpMutate>>,
                      q: Query<&RpcResponse<BrpMutate>>,
                      mut commands: Commands| {
                    let ecs_entity = trigger.entity;
                    if let Ok(response) = q.get(ecs_entity) {
                        match &response.data {
                            Ok(_) => {
                                debug!("pincard remove succeeded");
                                spawn_pincard_fetch(entity_id, &url, &mut commands);
                            }
                            Err(e) => error!("pincard remove failed: {e}"),
                        }
                    }
                    commands.entity(ecs_entity).despawn();
                },
            )
            .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
                error!("pincard remove: request timed out");
                commands.entity(trigger.entity).despawn();
            });
    }
}

pub(super) fn handle_pincard_despawn_button(
    buttons: Query<(&Interaction, &EntityCardDespawnButton), (Changed<Interaction>, With<Button>)>,
    server_url: Res<ServerUrl>,
    entity_cards: Query<(Entity, &EntityCard)>,
    mut expand_state: ResMut<EntityCardExpandState>,
    mut cache: ResMut<EntityCardDataCache>,
    mut commands: Commands,
) {
    for (interaction, btn) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let entity_id = btn.entity_id;
        debug!("pincard: despawning entity #{}", entity_id);

        // Despawn the local UI card immediately (optimistic update).
        for (card_entity, card) in &entity_cards {
            if card.entity_id == entity_id {
                commands.entity(card_entity).despawn();
                break;
            }
        }

        // Clean up local caches.
        expand_state.0.remove(&entity_id);
        cache.0.remove(&entity_id);

        // Fire BRP despawn.
        let req = commands.brp_despawn_entity(&server_url.0, entity_id);
        commands
            .entity(req)
            .observe(
                move |trigger: On<Add, RpcResponse<BrpMutate>>,
                      q: Query<&RpcResponse<BrpMutate>>,
                      mut commands: Commands| {
                    let ecs_entity = trigger.entity;
                    if let Ok(response) = q.get(ecs_entity) {
                        match &response.data {
                            Ok(_) => debug!("pincard: entity #{} despawned via BRP", entity_id),
                            Err(e) => {
                                error!("pincard despawn failed for entity #{}: {e}", entity_id)
                            }
                        }
                    }
                    commands.entity(ecs_entity).despawn();
                },
            )
            .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
                error!("pincard despawn: request timed out");
                commands.entity(trigger.entity).despawn();
            });
    }
}

pub(super) fn handle_pincard_insert_submit(
    mut input_focus: ResMut<InputFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_inputs: Query<(&mut EditableText, &EntityCardInsertField)>,
    mut font_cx: ResMut<FontCx>,
    mut layout_cx: ResMut<LayoutCx>,
    known: Res<EntityCardKnownMarkerComponents>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    if !keyboard_input.just_pressed(KeyCode::Enter) {
        return;
    }
    let Some(focused) = input_focus.get() else {
        return;
    };
    let Ok((mut text_input, marker)) = text_inputs.get_mut(focused) else {
        return;
    };

    let raw = text_input.value().to_string();
    let raw = raw.trim().to_string();
    if raw.is_empty() {
        return;
    }

    let Some(type_path) = known.0.get(&raw).cloned() else {
        show_global_message(
            format!("'{}' is not a marker component", raw),
            &mut commands,
        );
        return;
    };

    let entity_id = marker.entity_id;
    let url = server_url.0.clone();
    let components = serde_json::json!({ type_path.as_str(): {} });
    let req = commands.brp_insert_components(&server_url.0, entity_id, components);
    commands
        .entity(req)
        .observe(
            move |trigger: On<Add, RpcResponse<BrpMutate>>,
                  query: Query<&RpcResponse<BrpMutate>>,
                  mut commands: Commands| {
                let entity = trigger.entity;
                if let Ok(response) = query.get(entity) {
                    match &response.data {
                        Ok(_) => {
                            info!(
                                "pincard insert: '{}' added to entity #{}",
                                type_path, entity_id
                            );
                            spawn_pincard_fetch(entity_id, &url, &mut commands);
                        }
                        Err(e) => {
                            error!("pincard insert failed: {}", e);
                            show_global_message(
                                "Insert failed — check logs".to_string(),
                                &mut commands,
                            );
                        }
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            commands.entity(trigger.entity).despawn();
        });

    text_input.clear(&mut font_cx.0, &mut layout_cx.0);
    input_focus.clear();
}

pub(super) fn submit_pincard_field(
    mut input_focus: ResMut<InputFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_inputs: Query<(&mut EditableText, &EditableEntityCardField)>,
    mut font_cx: ResMut<FontCx>,
    mut layout_cx: ResMut<LayoutCx>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    if !keyboard_input.just_pressed(KeyCode::Enter) {
        return;
    }
    let Some(focused_entity) = input_focus.get() else {
        return;
    };
    let Ok((mut text_input, marker)) = text_inputs.get_mut(focused_entity) else {
        return;
    };

    let raw = text_input.value().to_string();
    if raw.is_empty() {
        return;
    }

    let json_value = parse_json_value(&raw);
    let field_path = if marker.field_key == "value" {
        String::new()
    } else {
        format!(".{}", marker.field_key)
    };

    let entity_id = marker.entity_id;
    let req = commands.brp_mutate_component(
        &server_url.0,
        entity_id,
        &marker.type_path,
        &field_path,
        json_value,
    );
    commands
        .entity(req)
        .observe(
            move |trigger: On<Add, RpcResponse<BrpMutate>>,
                  query: Query<&RpcResponse<BrpMutate>>,
                  server_url: Res<ServerUrl>,
                  mut commands: Commands| {
                let entity = trigger.entity;
                if let Ok(response) = query.get(entity) {
                    match &response.data {
                        Ok(_) => spawn_pincard_fetch(entity_id, &server_url.0, &mut commands),
                        Err(e) => error!("pincard mutate failed: {}", e),
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            error!("pincard mutate request timed out");
            commands.entity(trigger.entity).despawn();
        });

    text_input.clear(&mut font_cx.0, &mut layout_cx.0);
    input_focus.clear();
}

pub(super) fn parse_pincard_key(key: &str) -> Option<u64> {
    key.strip_prefix("pin-")?.parse().ok()
}
