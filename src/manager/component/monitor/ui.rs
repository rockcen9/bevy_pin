use super::super::fetch::{DiscoveredComponents, TriggeredDiscoveries};
use crate::manager::component::query::ComponentQueries;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_DESTRUCTIVE as COLOR_CLOSE_BG, COLOR_DESTRUCTIVE_HOVER as COLOR_CLOSE_HOVER,
    COLOR_HEADER_BG, COLOR_LABEL as COLOR_INPUT_TEXT, COLOR_LABEL_SECONDARY as COLOR_ENTITY_ID,
    COLOR_LABEL_TERTIARY as COLOR_VALUE, COLOR_PANEL_BG, COLOR_ROW_HOVER, COLOR_ROW_SELECTED,
    COLOR_SCROLLBAR_THUMB, COLOR_SCROLLBAR_TRACK, COLOR_TITLE,
};
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;

#[derive(Component, Clone, Default)]
#[require(DespawnOnExit::<SidebarState>(SidebarState::Component))]
struct MonitorPanel;

pub fn monitor_panel() -> impl Scene {
    bsn! {
        #MonitorPanel
        MonitorPanel
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
struct ComponentListContainer(String);

#[derive(Component, Clone, Default)]
pub struct ComponentEntityRow {
    pub entity: u64,
    pub query: String,
}

#[derive(Component)]
pub struct SelectedRow;

#[derive(Component, Clone, Default)]
struct CloseButton(String);

#[derive(Component, Clone, Default)]
struct ScrollbarThumb(String);

const SCROLL_MAX_HEIGHT: f32 = 300.0;

pub fn plugin(app: &mut App) {
    app.add_observer(on_monitor_panel_added);
    app.add_observer(on_scrollbar_thumb_added);
    app.add_observer(on_entity_row_added);
    app.add_systems(
        Update,
        (
            spawn_panels.run_if(resource_changed::<ComponentQueries>),
            spawn_entity_rows,
            update_entity_rows,
            handle_close_button,
            update_close_hover,
            update_scrollbar,
            scroll_on_mouse_wheel,
            handle_row_selection,
            update_row_hover,
        ),
    );
}
// #[cfg(feature = "dev")]
// mod debug {
//     use bevy::prelude::*;
//     use bevy_inspector_egui::quick::FilterQueryInspectorPlugin;

//     use crate::manager::component::ui::monitor_panel::ComponentContainerRoot;

//     pub fn plugin(app: &mut App) {
//         app.add_plugins(
//             FilterQueryInspectorPlugin::<With<ComponentContainerRoot>>::default()
//                 .run_if(command_key_toggle_active(false, KeyCode::Digit4)),
//         );
//     }
//     pub fn command_key_toggle_active(
//         default: bool,
//         key: KeyCode,
//     ) -> impl FnMut(Res<ButtonInput<KeyCode>>) -> bool + Clone {
//         let mut active = default;
//         move |inputs: Res<ButtonInput<KeyCode>>| {
//             if inputs.pressed(KeyCode::SuperLeft) && inputs.just_pressed(key) {
//                 active = !active;
//             }
//             active
//         }
//     }
// }

fn component_panel(title: String) -> impl Scene {
    let container_title = title.clone();
    let close_query = title.clone();
    let thumb_title = title.clone();
    bsn! {
        #ComponentContainerRoot
        ComponentContainerRoot
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
                    (
                        Button
                        Node {
                            width: Val::Px(20.0),
                            height: Val::Px(20.0),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                        }
                        BackgroundColor(COLOR_CLOSE_BG)
                        CloseButton({ close_query.clone() })
                        Children [(
                            Text::new("X")
                            template(|_| Ok(TextFont::from_font_size(14.0)))
                            TextColor(COLOR_INPUT_TEXT)
                        )]
                    ),
                ]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    max_height: Val::Px(SCROLL_MAX_HEIGHT),
                }
                Children [
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            flex_grow: 1.0,
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollPosition::default()
                        ComponentListContainer({ container_title.clone() })
                    ),
                    (
                        Node {
                            width: Val::Px(6.0),
                            align_self: AlignSelf::Stretch,
                        }
                        BackgroundColor(COLOR_SCROLLBAR_TRACK)
                        Children [(
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Px(6.0),
                                height: Val::Px(SCROLL_MAX_HEIGHT),
                                top: Val::Px(0.0),
                                border_radius: BorderRadius::all(Val::Px(3.0)),
                            }
                            Pickable::default()
                        BackgroundColor(COLOR_SCROLLBAR_THUMB)
                            ScrollbarThumb({ thumb_title.clone() })
                        )]
                    ),
                ]
            ),
        ]
    }
}
fn get_entity_display_label(raw_id: u64) -> String {
    // Extract the 32-bit index from the raw 64-bit ID
    let entity_index = raw_id as u32;

    // If the index is in the "High Range" (Scene Entities),
    // calculate the offset from u32::MAX.
    let display_index = if entity_index > 4_000_000_000 {
        u32::MAX - entity_index
    } else {
        entity_index
    };

    format!("v{}", display_index)
}
fn entity_row(entity_id: u64, query: String, value_str: String) -> impl Scene {
    let index_label = get_entity_display_label(entity_id);

    bsn! {
        Button
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        BackgroundColor(Color::NONE)
        ComponentEntityRow {
            entity: { entity_id },
            query: { query.clone() },
        }
        Children [
            (
                Text::new( index_label.clone() )
                template(|_| Ok(TextFont::from_font_size(13.0)))
                TextColor(COLOR_ENTITY_ID)
            ),
            (
                Text::new( value_str.clone() )
                template(|_| Ok(TextFont::from_font_size(13.0)))
                TextColor(COLOR_VALUE)
            ),
        ]
    }
}

fn on_monitor_panel_added(
    _trigger: On<Add, MonitorPanel>,
    commands: Commands,
    queries: Res<ComponentQueries>,
    root: Query<Entity, With<MonitorPanel>>,
    containers: Query<&ComponentListContainer>,
) {
    debug!(
        "MonitorPanel added — spawning panels for {} queries",
        queries.0.len()
    );
    spawn_panels_inner(commands, queries, root, containers);
}

fn spawn_panels(
    commands: Commands,
    queries: Res<ComponentQueries>,
    root: Query<Entity, With<MonitorPanel>>,
    containers: Query<&ComponentListContainer>,
) {
    spawn_panels_inner(commands, queries, root, containers);
}

fn spawn_panels_inner(
    mut commands: Commands,
    queries: Res<ComponentQueries>,
    root: Query<Entity, With<MonitorPanel>>,
    containers: Query<&ComponentListContainer>,
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
    containers: Query<(Entity, &ComponentListContainer)>,
    rows: Query<&ComponentEntityRow>,
) {
    if !components.is_changed() {
        return;
    }

    debug!(
        "spawn_entity_rows: DiscoveredComponents changed ({} entries)",
        components.0.len()
    );

    for entry in &components.0 {
        if rows
            .iter()
            .any(|r| r.entity == entry.entity && r.query == entry.query)
        {
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
        let Some(&value_child) = children.get(1) else {
            continue;
        };
        let Ok(mut text) = texts.get_mut(value_child) else {
            continue;
        };
        let new_val = entry.value.as_ref().map(value_summary).unwrap_or_default();
        text.0 = new_val;
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

fn update_close_hover(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<CloseButton>),
    >,
) {
    for (interaction, mut color) in &mut buttons {
        let new_color = match interaction {
            Interaction::Hovered => COLOR_CLOSE_HOVER,
            _ => COLOR_CLOSE_BG,
        };
        color.set_if_neq(BackgroundColor(new_color));
    }
}

fn scroll_on_mouse_wheel(
    mut mouse_wheel: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    parents: Query<&ChildOf>,
    mut containers: Query<(&mut ScrollPosition, &ComputedNode), With<ComponentListContainer>>,
) {
    let total_delta: f32 = mouse_wheel
        .read()
        .map(|e| match e.unit {
            MouseScrollUnit::Line => -e.y * 20.0,
            MouseScrollUnit::Pixel => -e.y,
        })
        .sum();

    if total_delta == 0.0 {
        return;
    }

    let mut scrolled: HashSet<Entity> = Default::default();

    for hit_map in hover_map.values() {
        for &hovered in hit_map.keys() {
            let mut entity = hovered;
            loop {
                if containers.contains(entity) && scrolled.insert(entity) {
                    if let Ok((mut scroll, computed)) = containers.get_mut(entity) {
                        let scale = computed.inverse_scale_factor();
                        let max_scroll =
                            ((computed.content_size().y - computed.size().y) * scale).max(0.0);
                        scroll.0.y = (scroll.0.y + total_delta).clamp(0.0, max_scroll);
                    }
                    break;
                }
                match parents.get(entity) {
                    Ok(child_of) => entity = child_of.parent(),
                    Err(_) => break,
                }
            }
        }
    }
}

fn on_scrollbar_thumb_added(trigger: On<Add, ScrollbarThumb>, mut commands: Commands) {
    commands.entity(trigger.entity).observe(on_thumb_drag);
}

fn on_thumb_drag(
    drag: On<Pointer<Drag>>,
    thumbs: Query<&ScrollbarThumb>,
    mut containers: Query<(&ComponentListContainer, &mut ScrollPosition, &ComputedNode)>,
) {
    let Ok(thumb) = thumbs.get(drag.entity) else {
        return;
    };
    let Some((_, mut scroll_pos, computed)) =
        containers.iter_mut().find(|(c, _, _)| c.0 == thumb.0)
    else {
        return;
    };

    let scale = computed.inverse_scale_factor();
    let content_h = computed.content_size().y * scale;
    let visible_h = computed.size().y * scale;
    let max_scroll = (content_h - visible_h).max(0.0);
    if max_scroll <= 0.0 {
        return;
    }

    let thumb_h = (visible_h / content_h * SCROLL_MAX_HEIGHT).max(20.0);
    let scroll_range = (SCROLL_MAX_HEIGHT - thumb_h).max(1.0);
    let scroll_delta = drag.delta.y * max_scroll / scroll_range;
    scroll_pos.0.y = (scroll_pos.0.y + scroll_delta).clamp(0.0, max_scroll);
}

fn update_scrollbar(
    containers: Query<(&ComponentListContainer, &ScrollPosition, &ComputedNode)>,
    mut thumbs: Query<(&ScrollbarThumb, &mut Node)>,
) {
    for (container, scroll_pos, computed) in &containers {
        let scale = computed.inverse_scale_factor();
        let content_h = computed.content_size().y * scale;
        let visible_h = computed.size().y * scale;

        let Some((_, mut thumb_node)) = thumbs.iter_mut().find(|(t, _)| t.0 == container.0) else {
            continue;
        };

        if content_h <= visible_h || content_h == 0.0 {
            thumb_node.height = Val::Percent(100.0);
            thumb_node.top = Val::Px(0.0);
            continue;
        }

        let thumb_h = (visible_h / content_h * SCROLL_MAX_HEIGHT).max(20.0);
        let max_scroll = content_h - visible_h;
        let thumb_top = (scroll_pos.0.y / max_scroll) * (SCROLL_MAX_HEIGHT - thumb_h);

        thumb_node.height = Val::Px(thumb_h);
        thumb_node.top = Val::Px(thumb_top.clamp(0.0, SCROLL_MAX_HEIGHT - thumb_h));
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
        bg.set_if_neq(BackgroundColor(Color::NONE));
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
            Interaction::Hovered => COLOR_ROW_HOVER,
            _ => Color::NONE,
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
