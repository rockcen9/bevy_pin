use crate::manager::component::component_data::{
    ComponentDataState, ComponentNameRow, SelectedComponent,
};
use crate::manager::connection::ServerUrl;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_HEADER_BG, COLOR_LABEL as COLOR_FIELD_KEY, COLOR_LABEL_TERTIARY as COLOR_FIELD_VALUE,
    COLOR_PANEL_BG, COLOR_TITLE,
};

// ── BRP ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GetComponentResponse {
    result: serde_json::Value,
}

#[derive(Component)]
struct GetComponentCtx {
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

// ── UI Components ──────────────────────────────────────────────────────────

#[derive(Component, Clone, Default)]
#[require(DespawnOnExit::<SidebarState>(SidebarState::Component))]
struct InspectorPanel;

#[derive(Component, Clone, Default)]
struct InspectorContent;

pub fn inspector_panel() -> impl Scene {
    bsn! {
        #InspectorPanel
        InspectorPanel
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
                    Text::new("Inspector")
                    template(|_| Ok(TextFont::from_font_size(18.0)))
                    TextColor(COLOR_TITLE)
                )]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(10.0)),
                }
                InspectorContent
            ),
        ]
    }
}

pub fn plugin(app: &mut App) {
    app.add_plugins(BrpEndpointPlugin::<GetComponentResponse>::default())
        .init_resource::<InspectorState>()
        .add_systems(
            Update,
            (
                fetch_on_component_selection,
                render_inspector.run_if(resource_changed::<InspectorState>),
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
    mut last: Local<Option<(u64, String)>>,
) {
    let current = selected
        .single()
        .ok()
        .and_then(|row| component_data.entity_id.map(|id| (id, row.type_path.clone())));

    if current == *last {
        return;
    }
    *last = current.clone();
    state.fields.clear();

    let Some((entity_id, type_path)) = current else {
        state.entity_id = None;
        state.type_path = None;
        return;
    };

    state.entity_id = Some(entity_id);
    state.type_path = Some(type_path.clone());

    let payload = serde_json::to_vec(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "world.get_components",
        "params": {
            "entity": entity_id,
            "components": [type_path],
            "strict": false
        }
    }))
    .unwrap();

    commands
        .spawn((
            BrpRequest::<GetComponentResponse>::new(&server_url.0, payload),
            GetComponentCtx {
                entity_id,
                type_path,
            },
        ))
        .observe(
            |trigger: On<Add, BrpResponse<GetComponentResponse>>,
             q: Query<(&BrpResponse<GetComponentResponse>, &GetComponentCtx)>,
             mut state: ResMut<InspectorState>,
             mut commands: Commands| {
                let ecs_entity = trigger.entity;
                let Ok((response, ctx)) = q.get(ecs_entity) else {
                    commands.entity(ecs_entity).despawn();
                    return;
                };

                if state.entity_id != Some(ctx.entity_id)
                    || state.type_path.as_deref() != Some(&ctx.type_path)
                {
                    commands.entity(ecs_entity).despawn();
                    return;
                }

                if let Ok(data) = &response.data {
                    let raw = &data.result["components"][&ctx.type_path];
                    let val = unwrap_newtype(raw);
                    state.fields = match val {
                        serde_json::Value::Object(map) => map
                            .iter()
                            .map(|(k, v)| (k.clone(), value_to_string(v)))
                            .collect(),
                        serde_json::Value::Array(arr) => decompose_affine(arr)
                            .unwrap_or_else(|| {
                                vec![("value".to_string(), value_to_string(val))]
                            }),
                        other if !other.is_null() => {
                            vec![("value".to_string(), value_to_string(other))]
                        }
                        _ => vec![],
                    };
                }

                commands.entity(ecs_entity).despawn();
            },
        )
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            commands.entity(trigger.entity).despawn();
        });
}

fn render_inspector(
    mut commands: Commands,
    state: Res<InspectorState>,
    content: Query<(Entity, Option<&Children>), With<InspectorContent>>,
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
                Text::new("Select a component"),
                TextFont::from_font_size(13.0),
                TextColor(COLOR_FIELD_VALUE),
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
                TextColor(COLOR_FIELD_VALUE),
            ))
            .id();
        commands.entity(content_entity).add_child(loading);
        return;
    }

    for (field, value) in &state.fields {
        let row = commands
            .spawn(Node {
                flex_direction: FlexDirection::Row,
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
        let val = commands
            .spawn((
                Text::new(value.clone()),
                TextFont::from_font_size(13.0),
                TextColor(COLOR_FIELD_VALUE),
            ))
            .id();
        commands.entity(row).add_children(&[key, val]);
        commands.entity(content_entity).add_child(row);
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
    let (r00, r10, r20) = if sx > eps { (m00/sx, m10/sx, m20/sx) } else { (1.0, 0.0, 0.0) };
    let (r01, r11, r21) = if sy > eps { (m01/sy, m11/sy, m21/sy) } else { (0.0, 1.0, 0.0) };
    let (r02, r12, r22) = if sz > eps { (m02/sz, m12/sz, m22/sz) } else { (0.0, 0.0, 1.0) };

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
        ("rotation".to_string(), format!("[{:.1},{:.1},{:.1},{:.1}]", qx, qy, qz, qw)),
        ("scale".to_string(),    format!("[{:.1},{:.1},{:.1}]", sx, sy, sz)),
        ("translation".to_string(), format!("[{:.1},{:.1},{:.1}]", tx, ty, tz)),
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
