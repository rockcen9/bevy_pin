use bevy::{
    input_focus::{InputFocus, tab_navigation::TabIndex},
    text::{EditableText, FontCx, LayoutCx, TextCursorStyle},
};

use crate::manager::connection::ServerUrl;
use crate::manager::entity_filter::component_list::{
    ComponentDataState, ComponentNameRow, SelectedComponent,
};
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_HEADER_BG, COLOR_INPUT_BG, COLOR_INPUT_BORDER, COLOR_INPUT_TEXT,
    COLOR_LABEL as COLOR_FIELD_KEY, COLOR_PANEL_BG, COLOR_TITLE,
};
use crate::ui_layout::theme::widgets::{ScrollableContainer, scrollable_list};

/// Stored on the active watch stream entity so the observer can validate freshness.
#[derive(Component)]
struct WatchComponentCtx {
    entity_id: u64,
    type_path: String,
}

// ── State ──────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
struct InspectorState {
    entity_id: Option<u64>,
    type_path: Option<String>,
    fields: Vec<(String, String)>, // (field_name, value_string)
}

/// Holds the ECS entity for the active `get_components+watch` stream.
/// Inserting `AbortStream` on it cancels and despawns it.
#[derive(Resource, Default)]
struct InspectorStreamEntity(Option<Entity>);

// ── UI Components ──────────────────────────────────────────────────────────

#[derive(Component, Clone, Default)]
#[require(DespawnOnExit::<SidebarState>(SidebarState::Component))]
struct InspectorPanel;

/// Marker on the editable input so we know which entity/component/field to mutate on Enter.
#[derive(Component, Clone, Default)]
struct EditableInspectorField {
    field_key: String,
}

pub fn inspector_panel() -> impl Scene {
    bsn! {
        #InspectorPanel
        InspectorPanel
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
                    Text::new("Inspector")
                    template(|_| Ok(TextFont::from_font_size(18.0)))
                    TextColor(COLOR_TITLE)
                )]
            ),
            scrollable_list("inspector", 300.0),
        ]
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<InspectorState>()
        .init_resource::<InspectorStreamEntity>()
        .add_systems(
            Update,
            (
                fetch_on_component_selection,
                render_inspector.run_if(resource_changed::<InspectorState>),
                submit_inspector_field,
            ),
        );
}

// ── Systems ────────────────────────────────────────────────────────────────

fn fetch_on_component_selection(
    selected: Query<&ComponentNameRow, With<SelectedComponent>>,
    component_data: Res<ComponentDataState>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
    mut state: ResMut<InspectorState>,
    mut stream_entity: ResMut<InspectorStreamEntity>,
    mut last: Local<Option<(u64, String)>>,
) {
    let current = selected.single().ok().and_then(|row| {
        component_data
            .entity_id()
            .map(|id| (id, row.type_path.clone()))
    });

    if current == *last {
        return;
    }
    *last = current.clone();

    // Cancel and despawn the old watch stream.
    if let Some(e) = stream_entity.0.take() {
        debug!("inspector: aborting old stream {:?}", e);
        commands.entity(e).insert(AbortStream);
    }

    state.fields.clear();

    let Some((entity_id, type_path)) = current else {
        debug!("inspector: selection cleared");
        state.entity_id = None;
        state.type_path = None;
        return;
    };

    debug!("inspector: watching entity={entity_id} type_path={type_path}");
    state.entity_id = Some(entity_id);
    state.type_path = Some(type_path.clone());

    // One-shot fetch for the initial snapshot (populates fields before the first watch event).
    let get_req =
        commands.brp_get_components(&server_url.0, entity_id, &[type_path.clone()], false);
    {
        let type_path = type_path.clone();
        commands
            .entity(get_req)
            .observe(
                move |trigger: On<Add, RpcResponse<BrpGetComponents>>,
                      query: Query<&RpcResponse<BrpGetComponents>>,
                      mut state: ResMut<InspectorState>,
                      mut commands: Commands| {
                    let entity = trigger.entity;
                    if let Ok(resp) = query.get(entity) {
                        if state.entity_id != Some(entity_id)
                            || state.type_path.as_deref() != Some(&type_path)
                        {
                            debug!("inspector get: stale, dropping");
                        } else {
                            match resp.data.as_ref() {
                                Ok(gc) => {
                                    let raw = &gc.result["components"][&type_path];
                                    let val = unwrap_newtype(raw);
                                    let new_fields = parse_fields(val);
                                    if new_fields != state.fields {
                                        state.fields = new_fields;
                                    }
                                }
                                Err(e) => warn!("inspector get_components error: {e}"),
                            }
                        }
                    }
                    commands.entity(entity).despawn();
                },
            )
            .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
                warn!("inspector get_components timed out");
                commands.entity(trigger.entity).despawn();
            });
    }

    // Watch stream — fires on every subsequent change.
    let stream =
        commands.brp_watch_components(&server_url.0, entity_id, &[type_path.as_str()], false);
    debug!("inspector: spawned stream entity {:?}", stream);
    commands
        .entity(stream)
        .insert(WatchComponentCtx {
            entity_id,
            type_path,
        })
        .observe(
            |trigger: On<Insert, StreamData<BrpGetComponentsWatch>>,
             query: Query<(&StreamData<BrpGetComponentsWatch>, &WatchComponentCtx)>,
             mut state: ResMut<InspectorState>| {
                let entity = trigger.entity;
                let Ok((data, ctx)) = query.get(entity) else {
                    debug!("inspector stream: query miss on {:?}", entity);
                    return;
                };

                debug!(
                    "inspector stream: {} event(s) for entity={} type_path={}",
                    data.0.len(),
                    ctx.entity_id,
                    ctx.type_path
                );

                if state.entity_id != Some(ctx.entity_id)
                    || state.type_path.as_deref() != Some(&ctx.type_path)
                {
                    debug!(
                        "inspector stream: stale (state={:?}/{:?}), dropping",
                        state.entity_id, state.type_path
                    );
                    return;
                }

                for item in &data.0 {
                    let raw = &item.result["components"][&ctx.type_path];
                    debug!("inspector stream: raw result = {}", raw);
                    let val = unwrap_newtype(raw);
                    let new_fields = parse_fields(val);
                    debug!(
                        "inspector stream: parsed {} field(s): {:?}",
                        new_fields.len(),
                        new_fields.iter().map(|(k, _)| k).collect::<Vec<_>>()
                    );
                    if new_fields != state.fields {
                        state.fields = new_fields;
                    }
                }
            },
        )
        .observe(|trigger: On<Add, StreamDisconnected>| {
            warn!(
                "Inspector stream {:?} disconnected — server closed or network error",
                trigger.entity
            );
        });
    stream_entity.0 = Some(stream);
}

fn render_inspector(
    mut commands: Commands,
    state: Res<InspectorState>,
    content: Query<(Entity, Option<&Children>, &ScrollableContainer)>,
) {
    let Some((content_entity, children, _)) = content.iter().find(|(_, _, c)| c.0 == "inspector")
    else {
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
                Text::new("Select a component"),
                TextFont::from_font_size(13.0),
                TextColor(COLOR_INPUT_TEXT),
            ))
            .id();
        commands.entity(content_entity).add_child(placeholder);
        return;
    }

    if state.fields.is_empty() {
        let loading = commands
            .spawn((
                Text::new("Loading..."),
                TextFont::from_font_size(13.0),
                TextColor(COLOR_INPUT_TEXT),
            ))
            .id();
        commands.entity(content_entity).add_child(loading);
        return;
    }

    for (field, value) in &state.fields {
        let row = commands
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .id();

        let key = commands
            .spawn((
                Text::new(field.clone()),
                TextFont::from_font_size(13.0),
                TextColor(COLOR_FIELD_KEY),
            ))
            .id();

        let display_val = value.clone();
        let field_key = field.clone();
        let input = commands
            .spawn((
                Node {
                    width: Val::Px(160.0),
                    border: UiRect::all(Val::Px(1.0)),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(COLOR_INPUT_BORDER),
                BackgroundColor(COLOR_INPUT_BG),
                EditableInspectorField { field_key },
                {
                    let mut text_input = EditableText {
                        max_characters: Some(128),
                        ..default()
                    };
                    text_input.editor.set_text(&display_val);
                    text_input
                },
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(COLOR_INPUT_TEXT),
                TextCursorStyle::default(),
                TabIndex(1),
            ))
            .id();

        commands.entity(row).add_children(&[key, input]);
        commands.entity(content_entity).add_child(row);
    }
}

fn submit_inspector_field(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_inputs: Query<(&mut EditableText, &EditableInspectorField)>,
    mut font_cx: ResMut<FontCx>,
    mut layout_cx: ResMut<LayoutCx>,
    state: Res<InspectorState>,
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
    let (Some(entity_id), Some(type_path)) = (state.entity_id, state.type_path.as_ref()) else {
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

    mutate_component_field(
        entity_id,
        type_path.clone(),
        field_path,
        json_value,
        &server_url.0,
        &mut commands,
    );

    text_input.clear(&mut font_cx.0, &mut layout_cx.0);
}

// ── BRP helpers ────────────────────────────────────────────────────────────

fn mutate_component_field(
    entity_id: u64,
    type_path: String,
    field_path: String,
    value: serde_json::Value,
    url: &str,
    commands: &mut Commands,
) {
    let req = commands.brp_mutate_component(url, entity_id, &type_path, &field_path, value);
    commands
        .entity(req)
        .observe(
            |trigger: On<Add, RpcResponse<BrpMutate>>,
             query: Query<&RpcResponse<BrpMutate>>,
             mut commands: Commands| {
                let entity = trigger.entity;
                if let Ok(response) = query.get(entity) {
                    match &response.data {
                        Ok(body) => info!("mutate_component_field response: {:?}", body.result),
                        Err(e) => error!("mutate_component_field failed: {}", e),
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            error!("mutate_component_field request timed out");
            commands.entity(trigger.entity).despawn();
        });
}

// ── Value helpers ──────────────────────────────────────────────────────────

const TRANSFORM_FIELD_ORDER: &[&str] = &["translation", "rotation", "scale"];

fn parse_fields(val: &serde_json::Value) -> Vec<(String, String)> {
    match val {
        serde_json::Value::Object(map) => {
            let mut fields: Vec<(String, String)> = map
                .iter()
                .map(|(k, v)| (k.clone(), value_to_string(v)))
                .collect();
            let keys: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
            if TRANSFORM_FIELD_ORDER.iter().all(|f| keys.contains(f)) {
                fields.sort_by_key(|(k, _)| {
                    TRANSFORM_FIELD_ORDER
                        .iter()
                        .position(|f| *f == k.as_str())
                        .unwrap_or(usize::MAX)
                });
            }
            fields
        }
        serde_json::Value::Array(arr) => decompose_affine(arr)
            .unwrap_or_else(|| vec![("value".to_string(), value_to_string(val))]),
        other if !other.is_null() => vec![("value".to_string(), value_to_string(other))],
        _ => vec![],
    }
}

/// Decomposes a flat 12-float Affine3A array (column-major) into rotation/scale/translation,
/// matching the display format of Transform.
fn decompose_affine(arr: &[serde_json::Value]) -> Option<Vec<(String, String)>> {
    if arr.len() != 12 {
        return None;
    }
    let f: Vec<f64> = arr.iter().filter_map(|v| v.as_f64()).collect();
    if f.len() != 12 {
        return None;
    }

    // Column-major 3×4: col0=[f0,f1,f2], col1=[f3,f4,f5], col2=[f6,f7,f8], t=[f9,f10,f11]
    let (m00, m10, m20) = (f[0], f[1], f[2]);
    let (m01, m11, m21) = (f[3], f[4], f[5]);
    let (m02, m12, m22) = (f[6], f[7], f[8]);
    let (tx, ty, tz) = (f[9], f[10], f[11]);

    let sx = (m00 * m00 + m10 * m10 + m20 * m20).sqrt();
    let sy = (m01 * m01 + m11 * m11 + m21 * m21).sqrt();
    let sz = (m02 * m02 + m12 * m12 + m22 * m22).sqrt();

    let eps = 1e-10_f64;
    let (r00, r10, r20) = if sx > eps {
        (m00 / sx, m10 / sx, m20 / sx)
    } else {
        (1.0, 0.0, 0.0)
    };
    let (r01, r11, r21) = if sy > eps {
        (m01 / sy, m11 / sy, m21 / sy)
    } else {
        (0.0, 1.0, 0.0)
    };
    let (r02, r12, r22) = if sz > eps {
        (m02 / sz, m12 / sz, m22 / sz)
    } else {
        (0.0, 0.0, 1.0)
    };

    // Rotation matrix → quaternion (Shepperd's method)
    let trace = r00 + r11 + r22;
    let (qx, qy, qz, qw) = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        ((r21 - r12) / s, (r02 - r20) / s, (r10 - r01) / s, 0.25 * s)
    } else if r00 > r11 && r00 > r22 {
        let s = (1.0 + r00 - r11 - r22).sqrt() * 2.0;
        (0.25 * s, (r01 + r10) / s, (r02 + r20) / s, (r21 - r12) / s)
    } else if r11 > r22 {
        let s = (1.0 - r00 + r11 - r22).sqrt() * 2.0;
        ((r01 + r10) / s, 0.25 * s, (r12 + r21) / s, (r02 - r20) / s)
    } else {
        let s = (1.0 - r00 - r11 + r22).sqrt() * 2.0;
        ((r02 + r20) / s, (r12 + r21) / s, 0.25 * s, (r10 - r01) / s)
    };

    Some(vec![
        (
            "translation".to_string(),
            format!("[{:.1},{:.1},{:.1}]", tx, ty, tz),
        ),
        (
            "rotation".to_string(),
            format!("[{:.1},{:.1},{:.1},{:.1}]", qx, qy, qz, qw),
        ),
        (
            "scale".to_string(),
            format!("[{:.1},{:.1},{:.1}]", sx, sy, sz),
        ),
    ])
}

/// Unwraps newtype wrappers so `GlobalTransform([{...}])` renders like `Transform({...})`.
/// Handles: single-element array `[inner]` and single-key object `{"0": inner}`.
fn unwrap_newtype(val: &serde_json::Value) -> &serde_json::Value {
    if let Some(arr) = val.as_array() {
        if arr.len() == 1 {
            return &arr[0];
        }
    }
    if let Some(map) = val.as_object() {
        if map.len() == 1 {
            if let Some(inner) = map.get("0") {
                return inner;
            }
        }
    }
    val
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

fn normalize_bare_decimals(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let chars: Vec<char> = s.chars().collect();
    for i in 0..chars.len() {
        result.push(chars[i]);
        if chars[i] == '.' {
            match chars.get(i + 1).copied() {
                Some(c) if c.is_ascii_digit() => {}
                _ => result.push('0'),
            }
        }
    }
    result
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
    let normalized = normalize_bare_decimals(s);
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&normalized) {
        return v;
    }
    json!(s)
}
