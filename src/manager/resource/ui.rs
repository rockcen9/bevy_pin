use bevy::{
    input_focus::{InputFocus, tab_navigation::TabIndex},
    text::{EditableText, FontCx, LayoutCx, TextCursorStyle},
};

use super::fetch::{DiscoveredResources, ResourceScreenRoot};
use super::update;
use crate::manager::connection::ServerUrl;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_HEADER_BG, COLOR_HEADER_BG_DISABLED, COLOR_INPUT_BG, COLOR_INPUT_BG_DISABLED,
    COLOR_INPUT_BORDER, COLOR_INPUT_TEXT, COLOR_LABEL_DISABLED, COLOR_LABEL_SECONDARY,
    COLOR_PANEL_BG, COLOR_PANEL_BG_DISABLED, COLOR_TITLE,
};
use crate::ui_layout::theme::widgets::{ScrollableContainer, scrollable_list};
use std::sync::Arc;

// Marker on the outer panel node for disabled-state color updates
#[derive(Component, Clone, Default)]
struct ResourcePanelMarker(Arc<str>);

// Marker on the header child node for disabled-state color updates
#[derive(Component, Clone, Default)]
struct ResourcePanelHeader(Arc<str>);

// Use Arc<str> for identification fields
#[derive(Component, Clone, Default)]
struct ResourceFieldRow {
    type_path: Arc<str>,
    key: Arc<str>,
}

// Marker on the EditableText node so submit_field_value knows which resource/field to write
#[derive(Component, Clone, Default)]
struct EditableFieldMarker {
    type_path: Arc<str>,
    key: Arc<str>,
}

// Inserted on submit so update_field_values refreshes the field once even while focused
#[derive(Component)]
struct RefreshOnce;

#[derive(Resource, Default)]
struct SpawnedResourcePanels(HashSet<String>);

#[derive(Resource, Default)]
struct SpawnedFieldRows(HashSet<String>);

pub fn plugin(app: &mut App) {
    app.init_resource::<SpawnedResourcePanels>()
        .init_resource::<SpawnedFieldRows>()
        .add_systems(
            OnExit(SidebarState::Resource),
            (clear_spawned_panels, clear_spawned_rows),
        )
        .add_systems(
            Update,
            (
                spawn_resource_panels,
                spawn_field_rows,
                update_field_values,
                update_panel_disabled_colors,
                submit_field_value,
            ),
        );
}

fn clear_spawned_panels(mut spawned: ResMut<SpawnedResourcePanels>) {
    spawned.0.clear();
}

fn clear_spawned_rows(mut spawned: ResMut<SpawnedFieldRows>) {
    spawned.0.clear();
}

pub fn resource_panels_root() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            flex_wrap: FlexWrap::Wrap
        }
        #ResourcePanelRoot
        ResourceScreenRoot
        DespawnOnExit::<SidebarState>(SidebarState::Resource)
    }
}

fn resource_panel(label: String, type_path: Arc<str>) -> impl Scene {
    let header_type_path = type_path.clone();
    let list_key = type_path.to_string();
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            min_width: Val::Px(280.0),
            max_width: Val::Px(280.0),
            border_radius: BorderRadius::all(Val::Px(10.0)),
        }
        BackgroundColor(COLOR_PANEL_BG)
        ResourcePanelMarker({ type_path.clone() })
        Children [
            (
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    border_radius: BorderRadius::top(Val::Px(10.0)),
                }
                BackgroundColor(COLOR_HEADER_BG)
                ResourcePanelHeader({ header_type_path.clone() })
                Children [(
                    Text::new(label.clone())
                    template(|_| Ok(TextFont::from_font_size(18.0)))
                    TextColor(COLOR_TITLE)
                )]
            ),
            scrollable_list(list_key, 300.0),
        ]
    }
}

// 1. Rename parameter to display_val to avoid macro shadowing
fn field_row(key: Arc<str>, display_val: String, type_path: Arc<str>) -> impl Scene {
    let key_str = key.to_string();
    // Pre-clone for the EditableFieldMarker before the macro consumes the originals
    let marker_type_path = type_path.clone();
    let marker_key = key.clone();

    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
        }
        ResourceFieldRow {
            type_path: { type_path.clone() },
            key: { key.clone() },
        }
        Children [
            (
                Text::new(key_str.clone() )
                template(|_| Ok(TextFont::from_font_size(13.0)))
                TextColor(COLOR_LABEL_SECONDARY)
            ),
            (
                Node {
                    width: Val::Px(160.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                }
                BorderColor::all(COLOR_INPUT_BORDER)
                BackgroundColor(COLOR_INPUT_BG)
                EditableFieldMarker {
                    type_path: { marker_type_path.clone() },
                    key: { marker_key.clone() },
                }
                template(move |_| {
                    let mut text_input = EditableText {
                        max_characters: Some(128),
                        ..default()
                    };
                    text_input.editor.set_text(&display_val);
                    Ok(text_input)
                })
                template(move |_| {
                    Ok(TextFont {
                        font_size: FontSize::Px(13.0),
                        ..default()
                    })
                })
                TextColor(COLOR_INPUT_TEXT)
                TextCursorStyle::default()
                TabIndex(1)
            )
        ]
    }
}

fn update_panel_disabled_colors(
    resources: Res<DiscoveredResources>,
    mut panels: Query<(&ResourcePanelMarker, &mut BackgroundColor)>,
    mut headers: Query<(&ResourcePanelHeader, &mut BackgroundColor), Without<ResourcePanelMarker>>,
    rows: Query<(&ResourceFieldRow, &Children)>,
    mut key_texts: Query<&mut TextColor, Without<BackgroundColor>>,
    mut input_bgs: Query<
        &mut BackgroundColor,
        (Without<ResourcePanelMarker>, Without<ResourcePanelHeader>),
    >,
) {
    if !resources.is_changed() {
        return;
    }
    for (marker, mut color) in &mut panels {
        let has_value = resources
            .0
            .iter()
            .find(|e| e.type_path == &*marker.0)
            .map_or(false, |e| e.value.is_some());
        debug!(
            "[resource panel] type_path={} value={}",
            marker.0,
            if has_value { "Some" } else { "None" }
        );
        let target = if has_value {
            COLOR_PANEL_BG
        } else {
            COLOR_PANEL_BG_DISABLED
        };
        color.set_if_neq(BackgroundColor(target));
    }
    for (header, mut color) in &mut headers {
        let has_value = resources
            .0
            .iter()
            .find(|e| e.type_path == &*header.0)
            .map_or(false, |e| e.value.is_some());
        let target = if has_value {
            COLOR_HEADER_BG
        } else {
            COLOR_HEADER_BG_DISABLED
        };
        color.set_if_neq(BackgroundColor(target));
    }
    for (row, children) in &rows {
        let has_value = resources
            .0
            .iter()
            .find(|e| e.type_path == &*row.type_path)
            .map_or(false, |e| e.value.is_some());

        if let Some(&key_entity) = children.get(0) {
            if let Ok(mut text_color) = key_texts.get_mut(key_entity) {
                let target = if has_value {
                    COLOR_LABEL_SECONDARY
                } else {
                    COLOR_LABEL_DISABLED
                };
                text_color.set_if_neq(TextColor(target));
            }
        }
        if let Some(&input_entity) = children.get(1) {
            if let Ok(mut bg) = input_bgs.get_mut(input_entity) {
                let target = if has_value {
                    COLOR_INPUT_BG
                } else {
                    COLOR_INPUT_BG_DISABLED
                };
                bg.set_if_neq(BackgroundColor(target));
            }
        }
    }
}

fn spawn_resource_panels(
    mut commands: Commands,
    resources: Res<DiscoveredResources>,
    root: Query<Entity, With<ResourceScreenRoot>>,
    mut spawned: ResMut<SpawnedResourcePanels>,
) {
    if !resources.is_changed() {
        return;
    }

    let Ok(root_entity) = root.single() else {
        return;
    };

    for entry in &resources.0 {
        if spawned.0.contains(&entry.type_path) {
            continue;
        }
        spawned.0.insert(entry.type_path.clone());

        // Convert String to Arc<str> before passing it down
        let type_path_arc: Arc<str> = entry.type_path.as_str().into();

        let panel = commands
            .spawn_scene(resource_panel(entry.label.clone(), type_path_arc))
            .id();
        commands.entity(root_entity).add_child(panel);
    }
}

fn spawn_field_rows(
    mut commands: Commands,
    resources: Res<DiscoveredResources>,
    containers: Query<(Entity, &ScrollableContainer)>,
    mut spawned: ResMut<SpawnedFieldRows>,
) {
    if !resources.is_changed() {
        return;
    }

    for entry in &resources.0 {
        let Some(value) = &entry.value else { continue };
        let Some(obj) = value.as_object() else {
            continue;
        };
        if spawned.0.contains(&entry.type_path) {
            continue;
        }

        let Some((container_entity, _)) = containers.iter().find(|(_, c)| c.0 == entry.type_path)
        else {
            continue;
        };

        spawned.0.insert(entry.type_path.clone());

        let type_path_arc: Arc<str> = entry.type_path.as_str().into();

        for (key, val) in obj {
            let display = value_to_string(val);
            let key_arc: Arc<str> = key.as_str().into();

            let row = commands
                .spawn_scene(field_row(key_arc, display, type_path_arc.clone()))
                .id();
            commands.entity(container_entity).add_child(row);
        }
    }
}

fn update_field_values(
    mut commands: Commands,
    resources: Res<DiscoveredResources>,
    input_focus: Res<InputFocus>,
    rows: Query<(&ResourceFieldRow, &Children)>,
    mut editable_texts: Query<&mut EditableText>,
    refresh_once: Query<Entity, With<RefreshOnce>>,
) {
    if !resources.is_changed() {
        return;
    }

    for (row, children) in &rows {
        let Some(entry) = resources.0.iter().find(|e| e.type_path == &*row.type_path) else {
            continue;
        };
        let Some(value) = &entry.value else { continue };
        let Some(obj) = value.as_object() else {
            continue;
        };
        let Some(field_val) = obj.get(&*row.key) else {
            continue;
        };

        let Some(&input_entity) = children.get(1) else {
            continue;
        };

        let has_refresh = refresh_once.contains(input_entity);

        // Skip focused fields unless they just submitted (RefreshOnce)
        if input_focus.get() == Some(input_entity) && !has_refresh {
            continue;
        }

        if let Ok(mut editable) = editable_texts.get_mut(input_entity) {
            editable.editor.set_text(&value_to_string(field_val));
        }

        if has_refresh {
            commands.entity(input_entity).remove::<RefreshOnce>();
        }
    }
}

fn submit_field_value(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_inputs: Query<(&mut EditableText, &EditableFieldMarker)>,
    mut font_cx: ResMut<FontCx>,
    mut layout_cx: ResMut<LayoutCx>,
    server_url: Res<ServerUrl>,
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
    let field_path = format!(".{}", marker.key);
    update::mutate_resource_field(
        marker.type_path.to_string(),
        field_path,
        json_value,
        &server_url.0,
        &mut commands,
    );
    text_input.clear(&mut font_cx.0, &mut layout_cx.0);
    commands.entity(focused_entity).insert(RefreshOnce);
}

fn parse_json_value(s: &str) -> serde_json::Value {
    if let Ok(n) = s.parse::<i64>() {
        return json!(n);
    }
    if let Ok(n) = s.parse::<f64>() {
        return json!(n);
    }
    if let Ok(b) = s.parse::<bool>() {
        return json!(b);
    }
    json!(s)
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
