use super::super::fetch::{DiscoveredComponents, TriggeredDiscoveries};
use crate::manager::connection::ServerUrl;
use crate::manager::entity_query::query::ComponentQueries;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_BUTTON_BG, COLOR_BUTTON_HOVER, COLOR_HEADER_BG, COLOR_INPUT_TEXT, COLOR_PANEL_BG,
    COLOR_ROW_SELECTED, COLOR_TITLE,
};
use crate::ui_layout::theme::widgets::{close_button, scrollable_list, ScrollableContainer};
use bevy::ecs::schedule::common_conditions::resource_changed;

const SCROLL_MAX_HEIGHT: f32 = 300.0;

#[derive(Component, Clone, Default)]
#[require(DespawnOnExit::<SidebarState>(SidebarState::Component))]
struct EntityListPanel;

pub fn entity_list_panel() -> impl Scene {
    bsn! {
        #EntityListPanel
        EntityListPanel
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            flex_wrap: FlexWrap::Wrap,
        }
    }
}

#[derive(Component, Clone, Default)]
#[require(DespawnOnExit::<SidebarState>(SidebarState::Component))]
struct ComponentContainerRoot;

#[derive(Component, Clone, Default)]
pub struct ComponentEntityRow {
    pub entity: u64,
    pub query: String,
}

#[derive(Component)]
pub struct SelectedRow;

#[derive(Component, Clone, Default)]
struct CloseButton(String);

#[derive(Component, Clone)]
struct DeleteEntityButton(u64);

pub fn plugin(app: &mut App) {
    app.add_observer(on_entity_list_panel_added)
        .add_observer(on_entity_row_added)
        .add_systems(
            Update,
            (
                spawn_panels.run_if(resource_changed::<ComponentQueries>),
                spawn_entity_rows,
                update_entity_rows,
                handle_close_button,
                handle_delete_entity_button,
                handle_row_selection,
                update_row_hover,
            ),
        );
}

fn component_panel(title: String) -> impl Scene {
    let close_query = title.clone();
    let list_key = title.clone();
    bsn! {
        #ComponentContainerRoot
        ComponentContainerRoot
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
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                }
                BackgroundColor(COLOR_HEADER_BG)
                Children [
                    (
                        Text::new( title.clone() )
                        template(|_| Ok(TextFont::from_font_size(18.0)))
                        TextColor(COLOR_TITLE)
                    ),
                    close_button(CloseButton(close_query.clone())),
                ]
            ),
            scrollable_list(list_key, SCROLL_MAX_HEIGHT),
        ]
    }
}


fn entity_row(entity_id: u64, query: String, value_str: String) -> impl Scene {
    let index_label = crate::utils::entity_display_label(entity_id);
    let label = if value_str.is_empty() {
        index_label
    } else {
        format!("{index_label}  {value_str}")
    };
    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            column_gap: Val::Px(4.0),
        }
        Children [
            close_button(DeleteEntityButton(entity_id)),
            (
                Button
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    flex_grow: 1.0,
                }
                BackgroundColor(COLOR_BUTTON_BG)
                ComponentEntityRow {
                    entity: { entity_id },
                    query: { query.clone() },
                }
                Children [(
                    template(move |_| Ok(Text::new(label.clone())))
                    template(|_| Ok(TextFont::from_font_size(13.0)))
                    TextColor(COLOR_INPUT_TEXT)
                )]
            ),
        ]
    }
}

fn on_entity_list_panel_added(
    _trigger: On<Add, EntityListPanel>,
    commands: Commands,
    queries: Res<ComponentQueries>,
    root: Query<Entity, With<EntityListPanel>>,
    containers: Query<&ScrollableContainer>,
) {
    debug!(
        "EntityListPanel added — spawning panels for {} queries",
        queries.0.len()
    );
    spawn_panels_inner(commands, queries, root, containers);
}

fn spawn_panels(
    commands: Commands,
    queries: Res<ComponentQueries>,
    root: Query<Entity, With<EntityListPanel>>,
    containers: Query<&ScrollableContainer>,
) {
    spawn_panels_inner(commands, queries, root, containers);
}

fn spawn_panels_inner(
    mut commands: Commands,
    queries: Res<ComponentQueries>,
    root: Query<Entity, With<EntityListPanel>>,
    containers: Query<&ScrollableContainer>,
) {
    let Ok(root_entity) = root.single() else {
        return;
    };

    for entry in &queries.0 {
        if containers.iter().any(|c| c.0 == entry.raw) {
            debug!(
                "spawn_panels: panel for '{}' already exists, skipping",
                entry.raw
            );
            continue;
        }

        debug!("spawn_panels: spawning panel for '{}'", entry.raw);
        let panel = commands
            .spawn_scene(component_panel(entry.raw.clone()))
            .id();
        commands.entity(root_entity).add_child(panel);
    }
}

fn spawn_entity_rows(
    mut commands: Commands,
    components: Res<DiscoveredComponents>,
    containers: Query<(Entity, &ScrollableContainer)>,
    rows: Query<(Entity, &ComponentEntityRow, &ChildOf)>,
) {
    if !components.is_changed() {
        return;
    }

    debug!(
        "spawn_entity_rows: DiscoveredComponents changed ({} entries)",
        components.0.len()
    );

    // Despawn rows for entities no longer in DiscoveredComponents
    let valid: HashSet<(u64, &str)> = components
        .0
        .iter()
        .map(|e| (e.entity, e.query.as_str()))
        .collect();
    for (_, row, child_of) in &rows {
        if !valid.contains(&(row.entity, row.query.as_str())) {
            debug!("spawn_entity_rows: removing stale row entity={} query='{}'", row.entity, row.query);
            commands.entity(child_of.parent()).despawn();
        }
    }

    let existing: HashSet<(u64, &str)> =
        rows.iter().map(|(_, r, _)| (r.entity, r.query.as_str())).collect();

    for entry in &components.0 {
        if existing.contains(&(entry.entity, entry.query.as_str())) {
            continue;
        }

        let Some((container_entity, _)) = containers.iter().find(|(_, c)| c.0 == entry.query)
        else {
            debug!(
                "spawn_entity_rows: no container for query '{}' (entity {}), skipping",
                entry.query, entry.entity
            );
            continue;
        };

        debug!(
            "spawn_entity_rows: spawning row for entity {} in query '{}'",
            entry.entity, entry.query
        );
        let value_str = entry.value.as_ref().map(value_summary).unwrap_or_default();
        let row = commands
            .spawn_scene(entity_row(entry.entity, entry.query.clone(), value_str))
            .id();
        commands.entity(container_entity).add_child(row);
    }
}

fn update_entity_rows(
    components: Res<DiscoveredComponents>,
    rows: Query<(&ComponentEntityRow, &Children)>,
    mut texts: Query<&mut Text>,
) {
    if !components.is_changed() {
        return;
    }

    for (row, children) in &rows {
        let Some(entry) = components
            .0
            .iter()
            .find(|e| e.entity == row.entity && e.query == row.query)
        else {
            continue;
        };
        let Some(&text_child) = children.get(0) else {
            continue;
        };
        let Ok(mut text) = texts.get_mut(text_child) else {
            continue;
        };
        let index_label = crate::utils::entity_display_label(row.entity);
        let value_str = entry.value.as_ref().map(value_summary).unwrap_or_default();
        let new_label = if value_str.is_empty() {
            index_label
        } else {
            format!("{index_label}  {value_str}")
        };
        text.set_if_neq(Text(new_label));
    }
}

fn handle_close_button(
    mut commands: Commands,
    buttons: Query<(&Interaction, &CloseButton, &ChildOf), Changed<Interaction>>,
    parents: Query<&ChildOf>,
    mut queries: ResMut<ComponentQueries>,
    mut components: ResMut<DiscoveredComponents>,
    mut triggered: ResMut<TriggeredDiscoveries>,
) {
    for (interaction, close, header_child_of) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let query_raw = &close.0;

        queries.0.retain(|e| &e.raw != query_raw);
        components.0.retain(|e| &e.query != query_raw);
        triggered.0.remove(query_raw);

        // button -> header -> panel
        if let Ok(panel_child_of) = parents.get(header_child_of.parent()) {
            commands.entity(panel_child_of.parent()).despawn();
        }
    }
}


fn handle_delete_entity_button(
    buttons: Query<(&Interaction, &DeleteEntityButton), Changed<Interaction>>,
    server_url: Res<ServerUrl>,
    mut components: ResMut<DiscoveredComponents>,
    mut commands: Commands,
) {
    for (interaction, btn) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let entity_id = btn.0;
        debug!("handle_delete_entity_button: despawning entity #{}", entity_id);
        let req = commands.brp_despawn_entity(&server_url.0, entity_id);
        commands
            .entity(req)
            .observe(
                move |trigger: On<Add, RpcResponse<BrpMutate>>,
                      query: Query<&RpcResponse<BrpMutate>>,
                      mut commands: Commands| {
                    let entity = trigger.entity;
                    if let Ok(response) = query.get(entity) {
                        match &response.data {
                            Ok(_) => info!("despawn_entity #{} ok", entity_id),
                            Err(e) => error!("despawn_entity #{} failed: {}", entity_id, e),
                        }
                    }
                    commands.entity(entity).despawn();
                },
            )
            .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
                commands.entity(trigger.entity).despawn();
            });

        components.0.retain(|e| e.entity != entity_id);
    }
}

fn on_entity_row_added(trigger: On<Add, ComponentEntityRow>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .observe(on_row_selected_added)
        .observe(on_row_selected_removed);
}

fn on_row_selected_added(
    trigger: On<Add, SelectedRow>,
    mut backgrounds: Query<&mut BackgroundColor>,
) {
    if let Ok(mut bg) = backgrounds.get_mut(trigger.entity) {
        bg.set_if_neq(BackgroundColor(COLOR_ROW_SELECTED));
    }
}

fn on_row_selected_removed(
    trigger: On<Remove, SelectedRow>,
    mut backgrounds: Query<&mut BackgroundColor>,
) {
    if let Ok(mut bg) = backgrounds.get_mut(trigger.entity) {
        bg.set_if_neq(BackgroundColor(COLOR_BUTTON_BG));
    }
}

fn handle_row_selection(
    mut commands: Commands,
    rows: Query<(Entity, &Interaction), (Changed<Interaction>, With<ComponentEntityRow>)>,
    selected: Query<Entity, With<SelectedRow>>,
) {
    for (entity, interaction) in &rows {
        if *interaction == Interaction::Pressed {
            for prev in &selected {
                commands.entity(prev).remove::<SelectedRow>();
            }
            commands.entity(entity).insert(SelectedRow);
        }
    }
}

fn update_row_hover(
    mut rows: Query<
        (Entity, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ComponentEntityRow>),
    >,
    selected: Query<Entity, With<SelectedRow>>,
) {
    for (entity, interaction, mut bg) in &mut rows {
        if selected.contains(entity) {
            continue;
        }
        let new_color = match interaction {
            Interaction::Hovered => COLOR_BUTTON_HOVER,
            _ => COLOR_BUTTON_BG,
        };
        bg.set_if_neq(BackgroundColor(new_color));
    }
}

fn value_summary(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(k, v)| format!("{}: {}", k, value_to_string(v)))
            .collect::<Vec<_>>()
            .join("  "),
        other => value_to_string(other),
    }
}

fn value_to_string(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}
