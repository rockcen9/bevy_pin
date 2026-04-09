use crate::manager::connection::ServerUrl;
use crate::manager::entity_filter::entity_list::ui::{ComponentEntityRow, SelectedRow};
use crate::manager::entity_filter::fetch::DiscoveredComponents;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_HEADER_BG, COLOR_INPUT_TEXT, COLOR_LABEL_DISABLED as COLOR_NO_DATA,
    COLOR_LABEL_SECONDARY as COLOR_COMPONENT_NAME, COLOR_LABEL_TERTIARY as COLOR_EMPTY,
    COLOR_PANEL_BG, COLOR_ROW_HOVER, COLOR_ROW_SELECTED, COLOR_TITLE,
};
use crate::ui_layout::theme::widgets::{
    ScrollableContainer, close_button::CloseButtonWidget, scrollable_list,
};

#[derive(Component)]
struct ListComponentsCtx {
    entity_id: u64,
}

#[derive(Component)]
struct CheckComponentsCtx {
    entity_id: u64,
}

// ── State ──────────────────────────────────────────────────────────────────

/// Set this to drive the component list panel from any context (entity_query or new_scene).
#[derive(Resource, Default)]
pub struct InspectedEntity(pub Option<u64>);

#[derive(Resource, Default)]
pub enum ComponentDataState {
    #[default]
    Idle,
    /// Waiting for the one-shot `world.list_components` response.
    Fetching { entity_id: u64 },
    /// Component list (and has-data info) is known; watch stream keeps it fresh.
    Ready {
        entity_id: u64,
        type_paths: Vec<String>,
        has_data: HashSet<String>,
    },
}

impl ComponentDataState {
    pub fn entity_id(&self) -> Option<u64> {
        match self {
            Self::Idle => None,
            Self::Fetching { entity_id } | Self::Ready { entity_id, .. } => Some(*entity_id),
        }
    }
}

/// Holds the ECS entity for the active `list_components+watch` stream.
/// Despawning it cancels the stream.
#[derive(Resource, Default)]
struct ComponentListStreamEntity(Option<Entity>);

// ── Marker Components ──────────────────────────────────────────────────────

/// Marks the title text of the Component List panel header.
#[derive(Component, Clone, Default)]
struct ComponentDataTitle;

/// Marks a component name row in the Component List panel.
#[derive(Component, Clone)]
pub struct ComponentNameRow {
    pub type_path: String,
}

/// Marker for the currently selected component name row.
#[derive(Component)]
pub struct SelectedComponent;

/// Button that removes a component from its entity when pressed.
#[derive(Component, Clone)]
pub struct RemoveComponentButton {
    pub entity_id: u64,
    pub type_path: String,
}

// ── UI Components ──────────────────────────────────────────────────────────

#[derive(Component, Clone, Default)]
#[require(DespawnOnExit::<SidebarState>(SidebarState::Component))]
struct ComponentDataPanel;

pub fn component_data_panel() -> impl Scene {
    bsn! {
        #ComponentDataPanel
        ComponentDataPanel
        Node {
            flex_direction: FlexDirection::Column,
            min_width: Val::Px(280.0),
            max_width: Val::Px(280.0),
            border_radius: BorderRadius::all(Val::Px(10.0)),
        }
        BackgroundColor(COLOR_PANEL_BG)
        Children [
            (
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    border_radius: BorderRadius::top(Val::Px(10.0)),
                }
                BackgroundColor(COLOR_HEADER_BG)
                Children [(
                    ComponentDataTitle
                    Text::new("Component List")
                    template(|_| Ok(TextFont::from_font_size(18.0)))
                    TextColor(COLOR_TITLE)
                )]
            ),
            scrollable_list("component-data", 300.0),
        ]
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<InspectedEntity>()
        .init_resource::<ComponentDataState>()
        .init_resource::<ComponentListStreamEntity>()
        .add_observer(on_component_name_row_added)
        .add_systems(
            Update,
            (
                fetch_on_selection,
                render_component_names.run_if(resource_changed::<ComponentDataState>),
                update_panel_title,
                handle_component_row_selection,
                update_component_row_hover,
                handle_remove_component_button,
            ),
        );
}

// ── Systems ────────────────────────────────────────────────────────────────

fn fetch_on_selection(
    selected: Query<&ComponentEntityRow, With<SelectedRow>>,
    inspected: Res<InspectedEntity>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
    mut state: ResMut<ComponentDataState>,
    mut stream_entity: ResMut<ComponentListStreamEntity>,
    mut last: Local<Option<u64>>,
) {
    let from_row = selected.single().ok().map(|r| r.entity);
    let current_id = from_row.or(inspected.0);
    if inspected.is_changed() {
        debug!(
            "fetch_on_selection: InspectedEntity changed -> {:?}",
            inspected.0
        );
    }
    if current_id == *last {
        return;
    }
    debug!(
        "fetch_on_selection: entity changed {:?} -> {:?}",
        *last, current_id
    );
    *last = current_id;

    // Cancel and despawn old watch stream entity.
    if let Some(e) = stream_entity.0.take() {
        debug!("fetch_on_selection: aborting previous stream {:?}", e);
        commands.entity(e).insert(AbortStream);
    }

    let Some(entity_id) = current_id else {
        debug!("fetch_on_selection: cleared (no entity selected)");
        *state = ComponentDataState::Idle;
        return;
    };

    debug!(
        "fetch_on_selection: fetching components for entity #{}",
        entity_id
    );
    *state = ComponentDataState::Fetching { entity_id };

    // One-shot to get the initial list (Fetching → Ready).
    let req = commands.brp_list_components(&server_url.0, entity_id);
    commands
        .entity(req)
        .insert(ListComponentsCtx { entity_id })
        .observe(on_initial_list_response)
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            error!("on_initial_list_response: timed out for entity request");
            commands.entity(trigger.entity).despawn();
        });

    // Watch stream for delta updates once Ready.
    let stream = commands.brp_watch_list_components(&server_url.0, entity_id);
    debug!(
        "fetch_on_selection: spawned watch stream {:?} for entity #{}",
        stream, entity_id
    );
    commands
        .entity(stream)
        .observe(
            |trigger: On<Insert, StreamData<BrpListComponentsWatch>>,
             query: Query<&StreamData<BrpListComponentsWatch>>,
             mut state: ResMut<ComponentDataState>,
             server_url: Res<ServerUrl>,
             mut commands: Commands| {
                let Ok(data) = query.get(trigger.entity) else {
                    return;
                };

                for item in &data.0 {
                    let added = &item.result.added;
                    let removed = &item.result.removed;
                    debug!("watch stream delta — added={added:?} removed={removed:?}");

                    let Some(entity_id) = state.entity_id() else {
                        continue;
                    };

                    let new_type_paths = match state.bypass_change_detection() {
                        ComponentDataState::Ready { type_paths, .. } => {
                            let mut paths = type_paths.clone();
                            for r in removed {
                                paths.retain(|p| p != r);
                            }
                            for a in added {
                                if !paths.contains(a) {
                                    paths.push(a.clone());
                                }
                            }
                            paths
                        }
                        ComponentDataState::Fetching { .. } => {
                            debug!("watch stream: still fetching initial list, ignoring delta");
                            continue;
                        }
                        ComponentDataState::Idle => {
                            error!(
                                "watch stream: delta in Idle state — stream should not be active"
                            );
                            continue;
                        }
                    };

                    let needs_update = match state.bypass_change_detection() {
                        ComponentDataState::Ready { type_paths, .. } => {
                            *type_paths != new_type_paths
                        }
                        _ => false,
                    };
                    if !needs_update {
                        debug!("watch stream: delta produced no change, skipping");
                        continue;
                    }

                    debug!(
                        "watch stream: applying delta → {} components for entity #{}",
                        new_type_paths.len(),
                        entity_id
                    );

                    if let ComponentDataState::Ready {
                        type_paths,
                        has_data,
                        ..
                    } = &mut *state
                    {
                        has_data.retain(|k| new_type_paths.contains(k));
                        *type_paths = new_type_paths.clone();
                    }

                    // One-shot has_data check for newly added paths only.
                    if !added.is_empty() {
                        debug!(
                            "watch stream: has_data check for {} newly added path(s)",
                            added.len()
                        );
                        let req =
                            commands.brp_get_components(&server_url.0, entity_id, added, false);
                        commands
                            .entity(req)
                            .insert(CheckComponentsCtx { entity_id })
                            .observe(on_check_components_response)
                            .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
                                error!("watch stream: has_data check timed out");
                                commands.entity(trigger.entity).despawn();
                            });
                    }
                }
            },
        )
        .observe(|trigger: On<Add, StreamDisconnected>| {
            warn!(
                "Component list stream {:?} disconnected — server closed or network error",
                trigger.entity
            );
        });
    stream_entity.0 = Some(stream);
}

/// Observer: handles the initial one-shot `world.list_components` response.
/// Transitions state from `Fetching` to `Ready` and fires the first `has_data` check.
fn on_initial_list_response(
    trigger: On<Add, RpcResponse<BrpListComponents>>,
    q: Query<(&RpcResponse<BrpListComponents>, &ListComponentsCtx)>,
    server_url: Res<ServerUrl>,
    mut state: ResMut<ComponentDataState>,
    mut commands: Commands,
) {
    let ecs_entity = trigger.entity;
    let Ok((response, ctx)) = q.get(ecs_entity) else {
        commands.entity(ecs_entity).despawn();
        return;
    };

    if state.entity_id() != Some(ctx.entity_id) {
        debug!(
            "on_initial_list_response: stale (state={:?}, ctx={}), dropping",
            state.entity_id(),
            ctx.entity_id
        );
        commands.entity(ecs_entity).despawn();
        return;
    }

    match &response.data {
        Err(e) => error!(
            "on_initial_list_response: server error for entity #{}: {e}",
            ctx.entity_id
        ),
        Ok(data) => {
            let type_paths = data.result.clone();
            debug!(
                "on_initial_list_response: {} components for entity #{}",
                type_paths.len(),
                ctx.entity_id
            );

            *state = ComponentDataState::Ready {
                entity_id: ctx.entity_id,
                type_paths: type_paths.clone(),
                has_data: HashSet::new(),
            };

            if !type_paths.is_empty() {
                debug!(
                    "on_initial_list_response: firing has_data check for {} path(s)",
                    type_paths.len()
                );
                let req =
                    commands.brp_get_components(&server_url.0, ctx.entity_id, &type_paths, false);
                commands
                    .entity(req)
                    .insert(CheckComponentsCtx {
                        entity_id: ctx.entity_id,
                    })
                    .observe(on_check_components_response)
                    .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
                        error!("on_check_components_response: timed out");
                        commands.entity(trigger.entity).despawn();
                    });
            }
        }
    }

    commands.entity(ecs_entity).despawn();
}

fn on_check_components_response(
    trigger: On<Add, RpcResponse<BrpGetComponents>>,
    q: Query<(&RpcResponse<BrpGetComponents>, &CheckComponentsCtx)>,
    mut state: ResMut<ComponentDataState>,
    mut commands: Commands,
) {
    let ecs_entity = trigger.entity;
    let Ok((response, ctx)) = q.get(ecs_entity) else {
        commands.entity(ecs_entity).despawn();
        return;
    };

    if state.entity_id() != Some(ctx.entity_id) {
        commands.entity(ecs_entity).despawn();
        return;
    }

    match &response.data {
        Err(e) => error!(
            "on_check_components_response: server error for entity #{}: {e}",
            ctx.entity_id
        ),
        Ok(data) => {
            let components_key = &data.result["components"];
            if let Some(components_map) = components_key.as_object() {
                let mut new_has_data: HashSet<String> = HashSet::new();
                for (type_path, value) in components_map {
                    let has_fields = match value {
                        serde_json::Value::Null => false,
                        serde_json::Value::Object(m) => !m.is_empty(),
                        _ => true,
                    };
                    if has_fields {
                        new_has_data.insert(type_path.clone());
                    }
                }
                debug!(
                    "on_check_components_response: {}/{} path(s) have data for entity #{}",
                    new_has_data.len(),
                    components_map.len(),
                    ctx.entity_id
                );
                // Merge into has_data rather than replacing, so a delta check
                // (covering only added paths) doesn't wipe previously-known entries.
                let needs_update = matches!(&*state,
                    ComponentDataState::Ready { has_data, .. }
                    if !new_has_data.is_subset(has_data)
                );
                if needs_update {
                    debug!("on_check_components_response: merging new has_data entries");
                    if let ComponentDataState::Ready { has_data, .. } = &mut *state {
                        has_data.extend(new_has_data);
                    }
                } else {
                    debug!("on_check_components_response: no new has_data entries, skipping");
                }
            } else {
                error!(
                    "on_check_components_response: unexpected shape — no 'components' key: {:?}",
                    data.result
                );
            }
        }
    }

    commands.entity(ecs_entity).despawn();
}

fn update_panel_title(
    state: Res<ComponentDataState>,
    discovered: Res<DiscoveredComponents>,
    mut titles: Query<&mut Text, With<ComponentDataTitle>>,
) {
    let Ok(mut text) = titles.single_mut() else {
        return;
    };
    let new_title = match state.entity_id() {
        Some(id) => {
            let display_name = discovered
                .0
                .iter()
                .filter(|e| e.entity == id)
                .find_map(|e| match e.value.as_ref()? {
                    serde_json::Value::String(s) => Some(s.clone()),
                    v => v
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                });
            match display_name {
                Some(name) => format!("{} {}", name, crate::utils::entity_display_label(id)),
                None => format!("Entity {}", crate::utils::entity_display_label(id)),
            }
        }
        None => "Component List".to_string(),
    };
    if text.0 != new_title {
        debug!("update_panel_title: '{}'", new_title);
        text.0 = new_title;
    }
}

fn render_component_names(
    mut commands: Commands,
    state: Res<ComponentDataState>,
    content: Query<(Entity, Option<&Children>, &ScrollableContainer)>,
) {
    let Some((content_entity, children, _)) =
        content.iter().find(|(_, _, c)| c.0 == "component-data")
    else {
        return;
    };
    debug!(
        "render_component_names: state={:?}",
        match &*state {
            ComponentDataState::Idle => "Idle".to_string(),
            ComponentDataState::Fetching { entity_id } => format!("Fetching(#{})", entity_id),
            ComponentDataState::Ready {
                entity_id,
                type_paths,
                has_data,
            } => format!(
                "Ready(#{}, {} paths, {} with data)",
                entity_id,
                type_paths.len(),
                has_data.len()
            ),
        }
    );

    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    match &*state {
        ComponentDataState::Idle => {
            let placeholder = commands
                .spawn((
                    Text::new("Select an entity row"),
                    TextFont::from_font_size(13.0),
                    TextColor(COLOR_EMPTY),
                ))
                .id();
            commands.entity(content_entity).add_child(placeholder);
        }
        ComponentDataState::Fetching { .. } => {
            let loading = commands
                .spawn((
                    Text::new("Loading..."),
                    TextFont::from_font_size(13.0),
                    TextColor(COLOR_EMPTY),
                ))
                .id();
            commands.entity(content_entity).add_child(loading);
        }
        ComponentDataState::Ready {
            entity_id,
            type_paths,
            has_data,
            ..
        } => {
            let entity_id = *entity_id;
            for type_path in type_paths {
                let short_name = type_path
                    .split("::")
                    .last()
                    .unwrap_or(type_path)
                    .to_string();

                let inspectable = has_data.contains(type_path);

                let text_color = if inspectable {
                    COLOR_COMPONENT_NAME
                } else {
                    COLOR_NO_DATA
                };

                let close_btn = commands
                    .spawn((
                        Button,
                        CloseButtonWidget,
                        Node {
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            flex_shrink: 0.0,
                            ..default()
                        },
                        BackgroundColor(COLOR_HEADER_BG),
                        RemoveComponentButton {
                            entity_id,
                            type_path: type_path.clone(),
                        },
                    ))
                    .with_child((
                        Text::new("X"),
                        TextFont::from_font_size(10.0),
                        TextColor(COLOR_INPUT_TEXT),
                    ))
                    .id();

                let label = commands
                    .spawn((
                        Text::new(short_name),
                        TextFont::from_font_size(13.0),
                        TextColor(text_color),
                    ))
                    .id();

                let mut row_cmds = commands.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(6.0),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    ComponentNameRow {
                        type_path: type_path.clone(),
                    },
                ));
                if inspectable {
                    row_cmds.insert(Button);
                }
                let row = row_cmds.add_children(&[close_btn, label]).id();

                commands.entity(content_entity).add_child(row);
            }
        }
    }
}

fn on_component_name_row_added(trigger: On<Add, ComponentNameRow>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .observe(on_component_selected_added)
        .observe(on_component_selected_removed);
}

fn on_component_selected_added(
    trigger: On<Add, SelectedComponent>,
    mut backgrounds: Query<&mut BackgroundColor>,
) {
    if let Ok(mut bg) = backgrounds.get_mut(trigger.entity) {
        bg.set_if_neq(BackgroundColor(COLOR_ROW_SELECTED));
    }
}

fn on_component_selected_removed(
    trigger: On<Remove, SelectedComponent>,
    mut backgrounds: Query<&mut BackgroundColor>,
) {
    if let Ok(mut bg) = backgrounds.get_mut(trigger.entity) {
        bg.set_if_neq(BackgroundColor(Color::NONE));
    }
}

fn handle_component_row_selection(
    mut commands: Commands,
    rows: Query<
        (Entity, &Interaction, &ComponentNameRow),
        (Changed<Interaction>, With<ComponentNameRow>),
    >,
    selected: Query<Entity, With<SelectedComponent>>,
) {
    for (entity, interaction, row) in &rows {
        if *interaction == Interaction::Pressed {
            debug!(
                "handle_component_row_selection: selected '{}'",
                row.type_path
            );
            for prev in &selected {
                commands.entity(prev).remove::<SelectedComponent>();
            }
            commands.entity(entity).insert(SelectedComponent);
        }
    }
}

fn handle_remove_component_button(
    buttons: Query<(&Interaction, &RemoveComponentButton), (Changed<Interaction>, With<Button>)>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed {
            debug!(
                "handle_remove_component_button: removing '{}' from entity #{}",
                btn.type_path, btn.entity_id
            );
            let req = commands.brp_remove_components(
                &server_url.0,
                btn.entity_id,
                &[btn.type_path.clone()],
            );
            commands
                .entity(req)
                .observe(
                    |trigger: On<Add, RpcResponse<BrpMutate>>,
                     q: Query<&RpcResponse<BrpMutate>>,
                     mut commands: Commands| {
                        let ecs_entity = trigger.entity;
                        if let Ok(response) = q.get(ecs_entity) {
                            if let Err(e) = &response.data {
                                error!("handle_remove_component_button: server error: {e}");
                            } else {
                                debug!("handle_remove_component_button: remove succeeded");
                            }
                        }
                        commands.entity(ecs_entity).despawn();
                    },
                )
                .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
                    error!("handle_remove_component_button: remove request timed out");
                    commands.entity(trigger.entity).despawn();
                });
        }
    }
}

fn update_component_row_hover(
    mut rows: Query<
        (Entity, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ComponentNameRow>),
    >,
    selected: Query<Entity, With<SelectedComponent>>,
) {
    for (entity, interaction, mut bg) in &mut rows {
        if selected.contains(entity) {
            continue;
        }
        let new_color = match interaction {
            Interaction::Hovered => COLOR_ROW_HOVER,
            _ => Color::NONE,
        };
        bg.set_if_neq(BackgroundColor(new_color));
    }
}
