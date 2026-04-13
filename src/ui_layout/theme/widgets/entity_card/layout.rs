use bevy::{
    input_focus::tab_navigation::TabIndex,
    text::{EditableText, TextCursorStyle},
};

use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_HEADER_BG, COLOR_INPUT_BG, COLOR_INPUT_BORDER, COLOR_INPUT_TEXT, COLOR_LABEL_SECONDARY,
    COLOR_LABEL_TERTIARY, COLOR_PANEL_BG, COLOR_TITLE,
};
use crate::utils::{parse_array_field, parse_fields, unwrap_newtype};

use super::components::{
    EditableEntityCardField, EntityCard, EntityCardExpandState, EntityCardExpandToggle,
    EntityCardHeader, EntityCardInsertField, EntityCardRemoveComponentButton, INSERT_SENTINEL,
};

// ── Scene builders ────────────────────────────────────────────────────────────

/// A general-purpose floating card with a draggable header and no close button.
///
/// - `drag_bundle`: Components placed on the header row (e.g. `DragHandle`).
///   Observers keyed on those components handle drag/pointer behaviour.
/// - The caller is responsible for adding body content and any card-specific
///   actions (close button, resize handles, etc.) after spawn.
pub fn spawn_entity_card<D: Bundle + Clone, H: SceneList>(
    label: String,
    entity_id: u64,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    drag_bundle: D,
    header_children: H,
) -> impl Scene {
    bsn! {
        EntityCard { entity_id: { entity_id }, height: { height } }
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px({ left }),
            top: Val::Px({ top }),
            flex_direction: FlexDirection::Column,
            width: Val::Px({ width }),
            height: Val::Px({ height }),
            min_width: Val::Px(180.0),
            overflow: Overflow::clip(),
            border_radius: BorderRadius::all(Val::Px(10.0)),
        }
        BackgroundColor(COLOR_PANEL_BG)
        Children [
            (
                EntityCardHeader
                Button
                template(move |_| Ok(drag_bundle.clone()))
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    border_radius: BorderRadius::top(Val::Px(10.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                }
                BackgroundColor(COLOR_HEADER_BG)
                EntityCardHighlight::new()
                Children [
                    (
                        template(move |_| Ok(Text::new(label.clone())))
                        template(|_| Ok(TextFont::from_font_size(18.0)))
                        TextColor(COLOR_TITLE)
                    ),
                    {header_children}
                ]
            )
        ]
    }
}

// ── Render helpers ────────────────────────────────────────────────────────────

pub(super) fn render_pincard(
    commands: &mut Commands,
    container_entity: Entity,
    entity_id: u64,
    components: &serde_json::Map<String, serde_json::Value>,
    expand_state: &EntityCardExpandState,
) {
    commands.entity(container_entity).despawn_children();

    // ── Insert Component row (always at top) ──────────────────────────────────
    let is_insert_expanded = expand_state
        .0
        .get(&entity_id)
        .map(|s| s.contains(INSERT_SENTINEL))
        .unwrap_or(false);

    let insert_header_entity = commands
        .spawn_scene(insert_header(entity_id, is_insert_expanded))
        .id();
    commands
        .entity(container_entity)
        .add_child(insert_header_entity);
    if is_insert_expanded {
        let insert_row = commands.spawn_scene(insert_input_row(entity_id)).id();
        commands.entity(container_entity).add_child(insert_row);
    }

    let mut sorted: Vec<(&String, &serde_json::Value)> = components.iter().collect();
    sorted.sort_by_key(|(k, _)| k.as_str());

    for (type_path, value) in sorted {
        let short_name = type_path
            .split("::")
            .last()
            .unwrap_or(type_path)
            .to_string();

        let is_expandable = match value {
            serde_json::Value::Null => false,
            serde_json::Value::Object(m) => !m.is_empty(),
            serde_json::Value::Array(a) => !a.is_empty(),
            _ => true, // string / number / bool → single-value tuple
        };
        let is_expanded = expand_state
            .0
            .get(&entity_id)
            .map(|s| s.contains(type_path))
            .unwrap_or(false);

        let header = spawn_component_header(
            commands,
            entity_id,
            type_path,
            short_name,
            value,
            is_expandable,
            is_expanded,
        );
        commands.entity(container_entity).add_child(header);

        if is_expanded {
            for (field_name, field_value) in parse_fields(unwrap_newtype(value)) {
                let row =
                    spawn_field_row(commands, entity_id, type_path, &field_name, &field_value);
                commands.entity(container_entity).add_child(row);
            }
        }
    }
}

fn spawn_component_header(
    commands: &mut Commands,
    entity_id: u64,
    type_path: &str,
    short_name: String,
    value: &serde_json::Value,
    is_expandable: bool,
    is_expanded: bool,
) -> Entity {
    // ── remove button ─────────────────────────────────────────────────────────
    use crate::ui_layout::theme::widgets::close_button::CloseButtonWidget;
    let remove_type_path = type_path.to_string();
    let remove_btn = commands.spawn_scene(bsn! {
        Button
        CloseButtonWidget
        Node {
            width: Val::Px(14.0),
            height: Val::Px(14.0),
            border_radius: BorderRadius::all(Val::Px(3.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
        }
        BackgroundColor(COLOR_HEADER_BG)
        template(move |_| Ok(EntityCardRemoveComponentButton { entity_id, type_path: remove_type_path.clone() }))
        Children [(
            Text::new("X")
            template(|_| Ok(TextFont::from_font_size(9.0)))
            TextColor(COLOR_INPUT_TEXT)
        )]
    }).id();

    // ── expand icon + name ────────────────────────────────────────────────────
    let icon = if is_expandable {
        if is_expanded { "V" } else { ">" }
    } else {
        " "
    };

    let icon_text = commands
        .spawn_scene(bsn! {
            Text::new( icon.to_string() )
            template(|_| Ok(TextFont::from_font_size(9.0)))
            TextColor(COLOR_LABEL_TERTIARY)
        })
        .id();

    let name_text = commands
        .spawn_scene(bsn! {
            Text::new( short_name.clone() )
            template(|_| Ok(TextFont::from_font_size(12.0)))
            TextColor(COLOR_LABEL_SECONDARY)
            TextLayout { linebreak: LineBreak::NoWrap }
        })
        .id();

    // ── value preview (collapsed or non-expandable) ───────────────────────────
    let mut children = vec![remove_btn, icon_text, name_text];

    let show_preview = !is_expandable || !is_expanded;
    if show_preview {
        let preview = format_component_value(value);
        if !preview.is_empty() {
            let val_text = commands
                .spawn_scene(bsn! {
                    Text::new( preview.clone() )
                    template(|_| Ok(TextFont::from_font_size(11.0)))
                    TextColor(COLOR_LABEL_TERTIARY)
                    TextLayout { linebreak: LineBreak::NoWrap }
                })
                .id();
            children.push(val_text);
        }
    }

    // ── header row ────────────────────────────────────────────────────────────
    // Only expandable rows get Button + toggle — non-expandable rows must never
    // enter the expand state machine, otherwise clicking them hides their value.
    let tp_str = type_path.to_string();
    let header = if is_expandable {
        commands
            .spawn_scene(bsn! {
                Button
                EntityCardExpandToggle {
                    entity_id: { entity_id },
                    type_path: { tp_str.clone() },
                }
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                    column_gap: Val::Px(4.0),
                }
                BackgroundColor(Color::NONE)
            })
            .id()
    } else {
        commands
            .spawn_scene(bsn! {
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                    column_gap: Val::Px(4.0),
                }
            })
            .id()
    };

    commands.entity(header).add_children(&children);
    header
}

fn format_component_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Object(m) if m.is_empty() => String::new(),
        v => v.to_string(),
    }
}

fn spawn_field_row(
    commands: &mut Commands,
    entity_id: u64,
    type_path: &str,
    field_name: &str,
    field_value: &str,
) -> Entity {
    let sub_values = parse_array_field(field_value);

    let row = commands
        .spawn_scene(bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect {
                    left: Val::Px(32.0),
                    right: Val::Px(6.0),
                    top: Val::Px(1.0),
                    bottom: Val::Px(1.0),
                },
                column_gap: Val::Px(4.0),
            }
        })
        .id();

    let name_str = field_name.to_string();
    let name_text = commands
        .spawn_scene(bsn! {
            Node { width: Val::Px(80.0), flex_shrink: 0.0 }
            Text::new( name_str.clone() )
            template(|_| Ok(TextFont::from_font_size(11.0)))
            TextColor(COLOR_LABEL_TERTIARY)
            TextLayout { linebreak: LineBreak::NoWrap }
        })
        .id();

    let mut children = vec![name_text];

    if let Some(sub_vals) = sub_values {
        // Vec2/Vec3/Vec4/Quat: use struct field access (.x, .y, .z, .w).
        // Larger arrays fall back to index syntax ([0], [1], ...).
        let axes = ["x", "y", "z", "w"];
        let use_axes = sub_vals.len() >= 2 && sub_vals.len() <= 4;

        for (i, val) in sub_vals.iter().enumerate() {
            let field_key = if use_axes {
                format!("{}.{}", field_name, axes[i])
            } else {
                format!("{}[{}]", field_name, i)
            };
            let input = commands
                .spawn_scene(field_input(entity_id, type_path, &field_key, val, true))
                .id();
            children.push(input);
        }
    } else {
        // Scalar / string field: single flex-growing box.
        let input = commands
            .spawn_scene(field_input(
                entity_id,
                type_path,
                field_name,
                field_value,
                false,
            ))
            .id();
        children.push(input);
    }

    commands.entity(row).add_children(&children);
    row
}

fn field_input(
    entity_id: u64,
    type_path: &str,
    field_key: &str,
    value: &str,
    small: bool,
) -> impl Scene {
    let type_path = type_path.to_string();
    let field_key = field_key.to_string();
    let value = value.to_string();
    bsn! {
        template(move |_| {
            Ok(if small {
                Node {
                    width: Val::Px(44.0),
                    min_width: Val::Px(0.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                }
            } else {
                Node {
                    width: Val::Px(160.0),
                    min_width: Val::Px(0.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    overflow: Overflow::clip(),
                    ..default()
                }
            })
        })
        BorderColor::all(COLOR_INPUT_BORDER)
        BackgroundColor(COLOR_INPUT_BG)
        EditableEntityCardField {
            entity_id: { entity_id },
            type_path: { type_path.clone() },
            field_key: { field_key.clone() },
        }
        template(move |_| {
            let mut t = EditableText { max_characters: Some(128), ..default() };
            t.editor.set_text(&value.clone());
            Ok(t)
        })
        template(|_| Ok(TextFont { font_size: FontSize::Px(11.0), ..default() }))
        TextColor(COLOR_INPUT_TEXT)
        TextCursorStyle::default()
        TabIndex(1)
    }
}

pub(super) fn insert_header(entity_id: u64, is_expanded: bool) -> impl Scene {
    let icon = if is_expanded { "V" } else { ">" };
    bsn! {
        Button
        EntityCardExpandToggle {
            entity_id: { entity_id },
            type_path: { INSERT_SENTINEL.to_string() },
        }
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
            column_gap: Val::Px(4.0),
        }
        BackgroundColor(Color::NONE)
        Children [
            (Node { width: Val::Px(14.0), height: Val::Px(14.0), flex_shrink: 0.0 }),
            (
                Text::new( icon.to_string() )
                template(|_| Ok(TextFont::from_font_size(9.0)))
                TextColor(COLOR_LABEL_TERTIARY)
            ),
            (
                Text::new("Insert Component")
                template(|_| Ok(TextFont::from_font_size(12.0)))
                TextColor(COLOR_LABEL_SECONDARY)
                TextLayout { linebreak: LineBreak::NoWrap }
            ),
        ]
    }
}

pub(super) fn insert_input_row(entity_id: u64) -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect {
                left: Val::Px(32.0),
                right: Val::Px(6.0),
                top: Val::Px(1.0),
                bottom: Val::Px(1.0),
            },
        }
        Children [(
            Node {
                min_width: Val::Px(160.0),
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                overflow: Overflow::clip(),
            }
            BorderColor::all(COLOR_INPUT_BORDER)
            BackgroundColor(COLOR_INPUT_BG)
            EntityCardInsertField { entity_id: { entity_id } }
            template(|_| {
                let mut t = EditableText { max_characters: Some(256), ..default() };
                t.editor.set_text("");
                Ok(t)
            })
            template(|_| Ok(TextFont { font_size: FontSize::Px(11.0), ..default() }))
            TextColor(COLOR_INPUT_TEXT)
            TextCursorStyle::default()
            TabIndex(1)
        )]
    }
}
