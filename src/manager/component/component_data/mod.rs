use crate::manager::component::monitor::ui::{ComponentEntityRow, SelectedRow};
use crate::manager::connection::ServerUrl;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_HEADER_BG, COLOR_LABEL_DISABLED as COLOR_NO_DATA,
    COLOR_LABEL_SECONDARY as COLOR_COMPONENT_NAME, COLOR_LABEL_TERTIARY as COLOR_EMPTY,
    COLOR_PANEL_BG, COLOR_ROW_HOVER, COLOR_ROW_SELECTED, COLOR_TITLE,
};

// ── BRP ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ListComponentsResponse {
    result: Vec<String>,
}

#[derive(Deserialize)]
struct CheckComponentsResponse {
    result: serde_json::Value,
}

#[derive(Component)]
struct ListComponentsCtx {
    entity_id: u64,
}

#[derive(Component)]
struct CheckComponentsCtx {
    entity_id: u64,
}

// ── State ──────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct ComponentDataState {
    pub entity_id: Option<u64>,
    pub type_paths: Vec<String>,
    pub has_data: HashSet<String>, // type_paths with inspectable fields
    pub ready: bool,               // true once has_data is resolved
}

// ── Marker Components ──────────────────────────────────────────────────────

/// Marks a component name row in the Component Data panel.
#[derive(Component, Clone)]
pub struct ComponentNameRow {
    pub type_path: String,
    pub short_name: String,
}

/// Marker for the currently selected component name row.
#[derive(Component)]
pub struct SelectedComponent;

// ── UI Components ──────────────────────────────────────────────────────────

#[derive(Component, Clone, Default)]
#[require(DespawnOnExit::<SidebarState>(SidebarState::Component))]
struct ComponentDataPanel;

#[derive(Component, Clone, Default)]
struct ComponentDataContent;

pub fn component_data_panel() -> impl Scene {
    bsn! {
        #ComponentDataPanel
        ComponentDataPanel
        Node {
            flex_direction: FlexDirection::Column,
            min_width: Val::Px(280.0),
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
                    Text::new("Component Data")
                    template(|_| Ok(TextFont::from_font_size(18.0)))
                    TextColor(COLOR_TITLE)
                )]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    padding: UiRect::all(Val::Px(10.0)),
                }
                ComponentDataContent
            ),
        ]
    }
}

pub fn plugin(app: &mut App) {
    app.add_plugins(BrpEndpointPlugin::<ListComponentsResponse>::default())
        .add_plugins(BrpEndpointPlugin::<CheckComponentsResponse>::default())
        .init_resource::<ComponentDataState>()
        .add_observer(on_component_name_row_added)
        .add_systems(
            Update,
            (
                fetch_on_selection,
                render_component_names.run_if(resource_changed::<ComponentDataState>),
                handle_component_row_selection,
                update_component_row_hover,
            ),
        );
}

// ── Systems ────────────────────────────────────────────────────────────────

fn fetch_on_selection(
    selected: Query<&ComponentEntityRow, With<SelectedRow>>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
    mut state: ResMut<ComponentDataState>,
    mut last: Local<Option<u64>>,
) {
    let current_id = selected.single().ok().map(|r| r.entity);
    if current_id == *last {
        return;
    }
    *last = current_id;
    state.entity_id = current_id;
    state.type_paths.clear();
    state.has_data.clear();
    state.ready = false;

    let Some(entity_id) = current_id else {
        return;
    };

    let payload = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "world.list_components",
        "params": { "entity": entity_id }
    }))
    .unwrap();

    commands
        .spawn((
            BrpRequest::<ListComponentsResponse>::new(&server_url.0, payload),
            ListComponentsCtx { entity_id },
        ))
        .observe(
            |trigger: On<Add, BrpResponse<ListComponentsResponse>>,
             q: Query<(&BrpResponse<ListComponentsResponse>, &ListComponentsCtx)>,
             server_url: Res<ServerUrl>,
             mut state: ResMut<ComponentDataState>,
             mut commands: Commands| {
                let ecs_entity = trigger.entity;
                let Ok((response, ctx)) = q.get(ecs_entity) else {
                    commands.entity(ecs_entity).despawn();
                    return;
                };

                if state.entity_id != Some(ctx.entity_id) {
                    commands.entity(ecs_entity).despawn();
                    return;
                }

                if let Ok(data) = &response.data {
                    let type_paths = data.result.clone();
                    state.type_paths = type_paths.clone();

                    if type_paths.is_empty() {
                        state.ready = true;
                    } else {
                        let payload = serde_json::to_vec(&json!({
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "world.get_components",
                            "params": {
                                "entity": ctx.entity_id,
                                "components": type_paths,
                                "strict": false
                            }
                        }))
                        .unwrap();

                        commands
                            .spawn((
                                BrpRequest::<CheckComponentsResponse>::new(&server_url.0, payload),
                                CheckComponentsCtx {
                                    entity_id: ctx.entity_id,
                                },
                            ))
                            .observe(on_check_components_response)
                            .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
                                commands.entity(trigger.entity).despawn();
                            });
                    }
                }

                commands.entity(ecs_entity).despawn();
            },
        )
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            commands.entity(trigger.entity).despawn();
        });
}

fn on_check_components_response(
    trigger: On<Add, BrpResponse<CheckComponentsResponse>>,
    q: Query<(&BrpResponse<CheckComponentsResponse>, &CheckComponentsCtx)>,
    mut state: ResMut<ComponentDataState>,
    mut commands: Commands,
) {
    let ecs_entity = trigger.entity;
    let Ok((response, ctx)) = q.get(ecs_entity) else {
        commands.entity(ecs_entity).despawn();
        return;
    };

    if state.entity_id != Some(ctx.entity_id) {
        commands.entity(ecs_entity).despawn();
        return;
    }

    if let Ok(data) = &response.data {
        if let Some(components_map) = data.result["components"].as_object() {
            for (type_path, value) in components_map {
                let has_fields = match value {
                    serde_json::Value::Null => false,
                    serde_json::Value::Object(m) => !m.is_empty(),
                    _ => true, // number, bool, string, array all count
                };
                if has_fields {
                    state.has_data.insert(type_path.clone());
                }
            }
        }
    }

    state.ready = true;
    commands.entity(ecs_entity).despawn();
}

fn render_component_names(
    mut commands: Commands,
    state: Res<ComponentDataState>,
    content: Query<(Entity, Option<&Children>), With<ComponentDataContent>>,
) {
    let Ok((content_entity, children)) = content.single() else {
        return;
    };

    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    if state.entity_id.is_none() {
        let placeholder = commands
            .spawn((
                Text::new("Select an entity row"),
                TextFont::from_font_size(13.0),
                TextColor(COLOR_EMPTY),
            ))
            .id();
        commands.entity(content_entity).add_child(placeholder);
        return;
    }

    if !state.ready {
        let loading = commands
            .spawn((
                Text::new("Loading..."),
                TextFont::from_font_size(13.0),
                TextColor(COLOR_EMPTY),
            ))
            .id();
        commands.entity(content_entity).add_child(loading);
        return;
    }

    for type_path in &state.type_paths {
        let short_name = type_path
            .split("::")
            .last()
            .unwrap_or(type_path)
            .to_string();

        let inspectable = state.has_data.contains(type_path);

        let mut row_cmds = commands.spawn((
            Node {
                padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            ComponentNameRow {
                type_path: type_path.clone(),
                short_name: short_name.clone(),
            },
        ));

        if inspectable {
            row_cmds.insert(Button);
        }

        let text_color = if inspectable {
            COLOR_COMPONENT_NAME
        } else {
            COLOR_NO_DATA
        };

        let row = row_cmds
            .with_child((
                Text::new(short_name),
                TextFont::from_font_size(13.0),
                TextColor(text_color),
            ))
            .id();

        commands.entity(content_entity).add_child(row);
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
    rows: Query<(Entity, &Interaction), (Changed<Interaction>, With<ComponentNameRow>)>,
    selected: Query<Entity, With<SelectedComponent>>,
) {
    for (entity, interaction) in &rows {
        if *interaction == Interaction::Pressed {
            for prev in &selected {
                commands.entity(prev).remove::<SelectedComponent>();
            }
            commands.entity(entity).insert(SelectedComponent);
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
