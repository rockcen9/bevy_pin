use bevy::{
    input_focus::{InputFocus, tab_navigation::TabIndex},
    text::{EditableText, FontCx, LayoutCx, TextCursorStyle, TextEdit},
    window::{CursorIcon, SystemCursorIcon},
};

use crate::manager::connection::ServerUrl;
use crate::manager::entity_filter::component_list::RemoveComponentButton;
use crate::manager::entity_filter::component_list::insert_component::KnownMarkerComponents;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_HEADER_BG, COLOR_INPUT_BG, COLOR_INPUT_BORDER, COLOR_INPUT_TEXT, COLOR_LABEL_SECONDARY,
    COLOR_LABEL_TERTIARY, COLOR_PANEL_BG, COLOR_PAUSED, COLOR_ROW_HOVER, COLOR_TITLE,
};
use crate::ui_layout::theme::widgets::{
    ScrollableContainer, close_button, close_button::CloseButtonWidget, scrollable_list,
    show_global_message,
};
use crate::utils::{parse_array_field, parse_fields, parse_json_value, unwrap_newtype};

use super::load_save::PinboardSaveData;

// ── Data ──────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PinCardEntry {
    pub entity_id: u64,
    pub label: String,
    pub left: f32,
    pub top: f32,
    #[serde(default = "default_card_width")]
    pub width: f32,
    #[serde(default = "default_card_height")]
    pub height: f32,
}

fn default_card_width() -> f32 {
    280.0
}

fn default_card_height() -> f32 {
    300.0
}

// ── Components ────────────────────────────────────────────────────────────────

#[derive(Component, Clone, Default, Reflect)]
pub struct DragHandle;

#[derive(Component, Clone, Default, Reflect)]
pub struct PinCard {
    pub entity_id: u64,
}

#[derive(Component, Clone, Default, Reflect)]
pub struct PinCardTitle(pub u64);

#[derive(Component)]
pub struct PinCardHighlight {
    pub timer: Timer,
}

impl PinCardHighlight {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.2, TimerMode::Once),
        }
    }
}

/// Drives periodic BRP polling for a pincard's component data.
#[derive(Component)]
struct PinCardPollTimer(Timer);

/// Context stored on a `brp_list_components` request entity.
#[derive(Component)]
struct PinCardListCtx {
    entity_id: u64,
}

/// Context stored on a `brp_get_components` request entity.
#[derive(Component)]
struct PinCardGetCtx {
    entity_id: u64,
}

/// Button on a component header row that toggles its expanded state.
#[derive(Component, Clone, Default)]
struct PinCardExpandToggle {
    entity_id: u64,
    type_path: String,
}

/// Close button payload — identifies which pincard to remove.
#[derive(Component, Clone)]
struct PinCardCloseButton {
    entity_id: u64,
}

/// Right-edge drag handle for resizing the pincard width.
#[derive(Component, Clone, Default)]
struct PinCardResizeHandle;

/// Left-edge drag handle for resizing the pincard width from the left.
#[derive(Component, Clone, Default)]
struct PinCardResizeHandleLeft;

/// Bottom-edge drag handle for resizing the pincard height.
#[derive(Component, Clone, Default)]
struct PinCardResizeHandleBottom;

/// Top-edge drag handle for resizing the pincard height from the top.
#[derive(Component, Clone, Default)]
struct PinCardResizeHandleTop;

#[derive(Component, Clone, Default)]
struct PinCardResizeCornerBR; // bottom-right

#[derive(Component, Clone, Default)]
struct PinCardResizeCornerBL; // bottom-left

#[derive(Component, Clone, Default)]
struct PinCardResizeCornerTR; // top-right

#[derive(Component, Clone, Default)]
struct PinCardResizeCornerTL; // top-left

/// Marks the outer row node of the scrollable list so height resize can target it.
#[derive(Component, Clone)]
struct PinCardScrollOuter {
    entity_id: u64,
}

/// Marker on an editable field input in a pincard expanded row.
#[derive(Component, Clone, Default)]
struct EditablePinCardField {
    entity_id: u64,
    type_path: String,
    field_key: String,
}

/// Marker on the insert-component text input inside a pincard.
#[derive(Component, Clone, Default)]
struct PinCardInsertField {
    entity_id: u64,
}

/// Sentinel type_path stored in `PinCardExpandState` for the insert-component row.
const INSERT_SENTINEL: &str = "__insert__";

// ── Resources ─────────────────────────────────────────────────────────────────

/// Which component rows are expanded, keyed by `entity_id → set of type_paths`.
#[derive(Resource, Default)]
struct PinCardExpandState(HashMap<u64, HashSet<String>>);

/// Last-received component data per entity, used for instant re-render on expand.
#[derive(Resource, Default)]
struct PinCardDataCache(HashMap<u64, serde_json::Map<String, serde_json::Value>>);

// ── Scene builders ────────────────────────────────────────────────────────────

pub fn pincard_key(entity_id: u64) -> String {
    format!("pin-{}", entity_id)
}

pub fn pincard(
    label: String,
    entity_id: u64,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
) -> impl Scene {
    let key = pincard_key(entity_id);
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px({ left }),
            top: Val::Px({ top }),
            flex_direction: FlexDirection::Column,
            width: Val::Px({ width }),
            min_width: Val::Px(180.0),
            overflow: Overflow::clip(),
            border_radius: BorderRadius::all(Val::Px(10.0)),
        }
        BackgroundColor(COLOR_PANEL_BG)
        Children [
            (
                DragHandle
                PinCardTitle({ entity_id })
                Button
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    border_radius: BorderRadius::top(Val::Px(10.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                }
                BackgroundColor(COLOR_HEADER_BG)
                Children [
                    (
                        Text::new( label.clone() )
                        template(|_| Ok(TextFont::from_font_size(18.0)))
                        TextColor(COLOR_TITLE)
                    ),
                    close_button(PinCardCloseButton { entity_id })
                ]
            ),
            scrollable_list( key.clone() , height ),
            (
                PinCardResizeHandle
                Pickable::default()
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(6.0),
                    height: Val::Percent(100.0),
                }
            ),
            (
                PinCardResizeHandleBottom
                Pickable::default()
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(6.0),
                }
            ),
            (
                PinCardResizeHandleLeft
                Pickable::default()
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(6.0),
                    height: Val::Percent(100.0),
                }
            ),
            (
                PinCardResizeHandleTop
                Pickable::default()
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(6.0),
                }
            ),
            (
                PinCardResizeCornerBR
                Pickable::default()
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Px(10.0),
                    height: Val::Px(10.0),
                }
            ),
            (
                PinCardResizeCornerBL
                Pickable::default()
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Px(10.0),
                    height: Val::Px(10.0),
                }
            ),
            (
                PinCardResizeCornerTR
                Pickable::default()
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(10.0),
                    height: Val::Px(10.0),
                }
            ),
            (
                PinCardResizeCornerTL
                Pickable::default()
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(10.0),
                    height: Val::Px(10.0),
                }
            ),
        ]
    }
}

// ── Plugin ────────────────────────────────────────────────────────────────────

pub fn plugin(app: &mut App) {
    app.init_resource::<PinCardExpandState>()
        .init_resource::<PinCardDataCache>()
        .add_observer(on_drag_handle_added)
        .add_observer(on_resize_handle_added)
        .add_observer(on_resize_handle_bottom_added)
        .add_observer(on_resize_handle_left_added)
        .add_observer(on_resize_handle_top_added)
        .add_observer(on_resize_corner_br_added)
        .add_observer(on_resize_corner_bl_added)
        .add_observer(on_resize_corner_tr_added)
        .add_observer(on_resize_corner_tl_added)
        .add_observer(on_pin_card_added)
        .add_systems(
            Update,
            (
                drive_pincard_highlight,
                trigger_initial_fetch,
                tick_pincard_polls,
                update_header_hover,
                handle_expand_toggle,
                render_from_cache_on_expand_change.after(handle_expand_toggle),
                on_pincard_close,
                submit_pincard_field,
                handle_pincard_insert_submit,
                auto_select_on_focus,
                restore_scroll_height,
            ),
        );
}

// ── Polling systems ───────────────────────────────────────────────────────────

fn on_pin_card_added(trigger: On<Add, PinCard>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(PinCardPollTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )));
}

fn trigger_initial_fetch(
    added: Query<(Entity, &ScrollableContainer), Added<ScrollableContainer>>,
    pin_cards: Query<&PinCard>,
    child_of: Query<&ChildOf>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    for (container_entity, container) in &added {
        let Some(entity_id) = parse_pincard_key(&container.0) else {
            continue;
        };
        if pin_cards.iter().any(|pc| pc.entity_id == entity_id) {
            spawn_pincard_fetch(entity_id, &server_url.0, &mut commands);

            // ScrollableContainer IS the inner column node — one hop up is the scroll outer row.
            if let Ok(scroll_outer) = child_of.get(container_entity) {
                commands
                    .entity(scroll_outer.0)
                    .insert(PinCardScrollOuter { entity_id });
            }
        }
    }
}

fn tick_pincard_polls(
    time: Res<Time>,
    mut cards: Query<(&PinCard, &mut PinCardPollTimer)>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
    state: Res<State<SidebarState>>,
) {
    if *state.get() != SidebarState::Pinboard {
        return;
    }
    for (card, mut timer) in &mut cards {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            spawn_pincard_fetch(card.entity_id, &server_url.0, &mut commands);
        }
    }
}

fn spawn_pincard_fetch(entity_id: u64, url: &str, commands: &mut Commands) {
    let req = commands.brp_list_components(url, entity_id);
    commands
        .entity(req)
        .insert(PinCardListCtx { entity_id })
        .observe(on_pincard_list_response)
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            commands.entity(trigger.entity).despawn();
        });
}

fn on_pincard_list_response(
    trigger: On<Add, RpcResponse<BrpListComponents>>,
    q: Query<(&RpcResponse<BrpListComponents>, &PinCardListCtx)>,
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
                    .insert(PinCardGetCtx { entity_id })
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
    q: Query<(&RpcResponse<BrpGetComponents>, &PinCardGetCtx)>,
    containers: Query<(Entity, &ScrollableContainer)>,
    expand_state: Res<PinCardExpandState>,
    mut cache: ResMut<PinCardDataCache>,
    input_focus: Res<InputFocus>,
    editable_fields: Query<&EditablePinCardField>,
    insert_fields: Query<&PinCardInsertField>,
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
            let key = pincard_key(ctx.entity_id);
            if let Some((container_entity, _)) = containers.iter().find(|(_, c)| c.0 == key) {
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
                        render_pincard(
                            &mut commands,
                            container_entity,
                            ctx.entity_id,
                            cached,
                            &expand_state,
                        );
                    }
                }
            }
        }
    }
    commands.entity(ecs_entity).despawn();
}

// ── Expand / collapse ─────────────────────────────────────────────────────────

fn handle_expand_toggle(
    toggles: Query<(&Interaction, &PinCardExpandToggle), (Changed<Interaction>, With<Button>)>,
    mut expand_state: ResMut<PinCardExpandState>,
) {
    for (interaction, toggle) in &toggles {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let set = expand_state.0.entry(toggle.entity_id).or_default();
        if set.contains(&toggle.type_path) {
            set.remove(&toggle.type_path);
        } else {
            set.insert(toggle.type_path.clone());
        }
    }
}

fn render_from_cache_on_expand_change(
    expand_state: Res<PinCardExpandState>,
    cache: Res<PinCardDataCache>,
    containers: Query<(Entity, &ScrollableContainer)>,
    input_focus: Res<InputFocus>,
    editable_fields: Query<&EditablePinCardField>,
    insert_fields: Query<&PinCardInsertField>,
    mut commands: Commands,
) {
    if !expand_state.is_changed() {
        return;
    }
    let focused_entity_id = input_focus.get().and_then(|e| {
        editable_fields
            .get(e)
            .map(|f| f.entity_id)
            .ok()
            .or_else(|| insert_fields.get(e).map(|f| f.entity_id).ok())
    });

    for (entity_id, components) in &cache.0 {
        if focused_entity_id == Some(*entity_id) {
            continue;
        }
        let key = pincard_key(*entity_id);
        if let Some((container_entity, _)) = containers.iter().find(|(_, c)| c.0 == key) {
            render_pincard(
                &mut commands,
                container_entity,
                *entity_id,
                components,
                &expand_state,
            );
        }
    }
}

// ── Render helpers ────────────────────────────────────────────────────────────

fn render_pincard(
    commands: &mut Commands,
    container_entity: Entity,
    entity_id: u64,
    components: &serde_json::Map<String, serde_json::Value>,
    expand_state: &PinCardExpandState,
) {
    commands.entity(container_entity).despawn_children();

    // ── Insert Component row (always at top) ──────────────────────────────────
    let is_insert_expanded = expand_state
        .0
        .get(&entity_id)
        .map(|s| s.contains(INSERT_SENTINEL))
        .unwrap_or(false);
    let insert_header = commands
        .spawn_scene(insert_header(entity_id, is_insert_expanded))
        .id();
    commands.entity(container_entity).add_child(insert_header);
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
        template(move |_| Ok(RemoveComponentButton { entity_id, type_path: remove_type_path.clone() }))
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
                PinCardExpandToggle {
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
        EditablePinCardField {
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

fn insert_header(entity_id: u64, is_expanded: bool) -> impl Scene {
    let icon = if is_expanded { "V" } else { ">" };
    bsn! {
        Button
        PinCardExpandToggle {
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

fn insert_input_row(entity_id: u64) -> impl Scene {
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
            PinCardInsertField { entity_id: { entity_id } }
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

fn handle_pincard_insert_submit(
    mut input_focus: ResMut<InputFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_inputs: Query<(&mut EditableText, &PinCardInsertField)>,
    mut font_cx: ResMut<FontCx>,
    mut layout_cx: ResMut<LayoutCx>,
    known: Res<KnownMarkerComponents>,
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

fn update_header_hover(
    mut headers: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PinCardExpandToggle>),
    >,
) {
    for (interaction, mut bg) in &mut headers {
        bg.set_if_neq(BackgroundColor(match interaction {
            Interaction::Hovered => COLOR_ROW_HOVER,
            _ => Color::NONE,
        }));
    }
}

fn on_pincard_close(
    buttons: Query<(&Interaction, &PinCardCloseButton), (Changed<Interaction>, With<Button>)>,
    pin_cards: Query<(Entity, &PinCard)>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
    mut cache: ResMut<PinCardDataCache>,
    mut expand_state: ResMut<PinCardExpandState>,
    mut commands: Commands,
) {
    for (interaction, btn) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let entity_id = btn.entity_id;

        if let Some((card_entity, _)) = pin_cards.iter().find(|(_, pc)| pc.entity_id == entity_id) {
            commands.entity(card_entity).despawn();
        }

        if let Some(ref mut sd) = save_data {
            sd.cards.retain(|c| c.entity_id != entity_id);
            sd.persist().ok();
        }

        cache.0.remove(&entity_id);
        expand_state.0.remove(&entity_id);
    }
}

fn parse_pincard_key(key: &str) -> Option<u64> {
    key.strip_prefix("pin-")?.parse().ok()
}

fn submit_pincard_field(
    mut input_focus: ResMut<InputFocus>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut text_inputs: Query<(&mut EditableText, &EditablePinCardField)>,
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

fn auto_select_on_focus(
    input_focus: Res<InputFocus>,
    mut text_inputs: Query<&mut EditableText, With<EditablePinCardField>>,
) {
    if !input_focus.is_changed() {
        return;
    }
    let Some(focused) = input_focus.get() else {
        return;
    };
    let Ok(mut text_input) = text_inputs.get_mut(focused) else {
        return;
    };
    text_input.queue_edit(TextEdit::SelectAll);
}

// ── Drag systems ──────────────────────────────────────────────────────────────

fn drive_pincard_highlight(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut BackgroundColor, &mut PinCardHighlight)>,
) {
    for (entity, mut bg, mut highlight) in &mut q {
        highlight.timer.tick(time.delta());
        let t = highlight.timer.fraction();
        let start = COLOR_PAUSED.to_srgba();
        let end = COLOR_HEADER_BG.to_srgba();
        bg.0 = Color::srgba(
            start.red + (end.red - start.red) * t,
            start.green + (end.green - start.green) * t,
            start.blue + (end.blue - start.blue) * t,
            start.alpha + (end.alpha - start.alpha) * t,
        );
        if highlight.timer.just_finished() {
            commands.entity(entity).remove::<PinCardHighlight>();
        }
    }
}

fn on_resize_handle_added(trigger: On<Add, PinCardResizeHandle>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .observe(on_resize_drag)
        .observe(on_resize_drag_end)
        .observe(on_resize_over)
        .observe(on_resize_out);
}

fn on_resize_drag(
    trigger: On<Pointer<Drag>>,
    child_of: Query<&ChildOf>,
    mut nodes: Query<&mut Node>,
) {
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok(mut node) = nodes.get_mut(parent.0) else {
        return;
    };
    let delta = trigger.event.delta.x;
    node.width = Val::Px(match node.width {
        Val::Px(w) => (w + delta).max(180.0),
        _ => 280.0 + delta,
    });
}

fn on_resize_drag_end(
    trigger: On<Pointer<DragEnd>>,
    child_of: Query<&ChildOf>,
    nodes: Query<&Node>,
    pin_cards: Query<&PinCard>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
) {
    let Some(save_data) = save_data.as_mut() else {
        return;
    };
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let card_entity = parent.0;
    let Ok(node) = nodes.get(card_entity) else {
        return;
    };
    let Ok(pin_card) = pin_cards.get(card_entity) else {
        return;
    };
    let width = match node.width {
        Val::Px(w) => w,
        _ => return,
    };
    for entry in save_data.cards.iter_mut() {
        if entry.entity_id == pin_card.entity_id {
            entry.width = width;
            break;
        }
    }
    save_data.persist().ok();
}

fn on_resize_over(
    _trigger: On<Pointer<Over>>,
    mut windows: Query<Entity, With<Window>>,
    mut commands: Commands,
) {
    for entity in &mut windows {
        commands
            .entity(entity)
            .insert(CursorIcon::from(SystemCursorIcon::EwResize));
    }
}

fn on_resize_out(
    _trigger: On<Pointer<Out>>,
    mut windows: Query<Entity, With<Window>>,
    mut commands: Commands,
) {
    for entity in &mut windows {
        commands
            .entity(entity)
            .insert(CursorIcon::from(SystemCursorIcon::Default));
    }
}

fn on_resize_handle_bottom_added(
    trigger: On<Add, PinCardResizeHandleBottom>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.entity)
        .observe(on_resize_bottom_drag)
        .observe(on_resize_bottom_drag_end)
        .observe(on_resize_bottom_over)
        .observe(on_resize_bottom_out);
}

fn on_resize_bottom_drag(
    trigger: On<Pointer<Drag>>,
    child_of: Query<&ChildOf>,
    pin_cards: Query<&PinCard>,
    mut scroll_outers: Query<(&PinCardScrollOuter, &mut Node, &ComputedNode)>,
) {
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok(pin_card) = pin_cards.get(parent.0) else {
        return;
    };
    let delta = trigger.event.delta.y;
    for (outer, mut node, computed) in &mut scroll_outers {
        if outer.entity_id == pin_card.entity_id {
            // Use the actual rendered height as the base so the first drag
            // doesn't jump from content-height to max_height.
            let base = match node.height {
                Val::Px(h) => h,
                _ => computed.size().y,
            };
            node.height = Val::Px((base + delta).max(60.0));
            break;
        }
    }
}

fn on_resize_bottom_drag_end(
    trigger: On<Pointer<DragEnd>>,
    child_of: Query<&ChildOf>,
    pin_cards: Query<&PinCard>,
    scroll_outers: Query<(&PinCardScrollOuter, &Node)>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
) {
    let Some(save_data) = save_data.as_mut() else {
        return;
    };
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok(pin_card) = pin_cards.get(parent.0) else {
        return;
    };
    for (outer, node) in &scroll_outers {
        if outer.entity_id == pin_card.entity_id {
            if let Val::Px(h) = node.height {
                for entry in save_data.cards.iter_mut() {
                    if entry.entity_id == pin_card.entity_id {
                        entry.height = h;
                        break;
                    }
                }
                save_data.persist().ok();
            }
            break;
        }
    }
}

fn on_resize_bottom_over(
    _trigger: On<Pointer<Over>>,
    mut windows: Query<Entity, With<Window>>,
    mut commands: Commands,
) {
    for entity in &mut windows {
        commands
            .entity(entity)
            .insert(CursorIcon::from(SystemCursorIcon::NsResize));
    }
}

fn on_resize_bottom_out(
    _trigger: On<Pointer<Out>>,
    mut windows: Query<Entity, With<Window>>,
    mut commands: Commands,
) {
    for entity in &mut windows {
        commands
            .entity(entity)
            .insert(CursorIcon::from(SystemCursorIcon::Default));
    }
}

// ── Corner resize helpers ─────────────────────────────────────────────────────

/// Resize width from right + height from bottom (no position change).
fn corner_drag_br(
    delta_x: f32,
    delta_y: f32,
    entity_id: u64,
    card_node: &mut Node,
    scroll_outers: &mut Query<(&PinCardScrollOuter, &mut Node, &ComputedNode), Without<PinCard>>,
) {
    card_node.width = Val::Px(match card_node.width {
        Val::Px(w) => (w + delta_x).max(180.0),
        _ => 280.0 + delta_x,
    });
    for (outer, mut sn, computed) in scroll_outers.iter_mut() {
        if outer.entity_id == entity_id {
            let base = match sn.height {
                Val::Px(h) => h,
                _ => computed.size().y,
            };
            sn.height = Val::Px((base + delta_y).max(60.0));
            break;
        }
    }
}

/// Resize width from left (moves card) + height from bottom.
fn corner_drag_bl(
    delta_x: f32,
    delta_y: f32,
    entity_id: u64,
    card_node: &mut Node,
    scroll_outers: &mut Query<(&PinCardScrollOuter, &mut Node, &ComputedNode), Without<PinCard>>,
) {
    let current_w = match card_node.width {
        Val::Px(w) => w,
        _ => 280.0,
    };
    let new_w = (current_w - delta_x).max(180.0);
    let actual_delta = current_w - new_w;
    card_node.width = Val::Px(new_w);
    card_node.left = Val::Px(match card_node.left {
        Val::Px(x) => x + actual_delta,
        _ => actual_delta,
    });
    for (outer, mut sn, computed) in scroll_outers.iter_mut() {
        if outer.entity_id == entity_id {
            let base = match sn.height {
                Val::Px(h) => h,
                _ => computed.size().y,
            };
            sn.height = Val::Px((base + delta_y).max(60.0));
            break;
        }
    }
}

/// Resize width from right + height from top (moves card).
fn corner_drag_tr(
    delta_x: f32,
    delta_y: f32,
    entity_id: u64,
    card_node: &mut Node,
    scroll_outers: &mut Query<(&PinCardScrollOuter, &mut Node, &ComputedNode), Without<PinCard>>,
) {
    card_node.width = Val::Px(match card_node.width {
        Val::Px(w) => (w + delta_x).max(180.0),
        _ => 280.0 + delta_x,
    });
    for (outer, mut sn, computed) in scroll_outers.iter_mut() {
        if outer.entity_id == entity_id {
            let current_h = match sn.height {
                Val::Px(h) => h,
                _ => computed.size().y,
            };
            let new_h = (current_h - delta_y).max(60.0);
            let actual_delta = current_h - new_h;
            sn.height = Val::Px(new_h);
            card_node.top = Val::Px(match card_node.top {
                Val::Px(y) => y + actual_delta,
                _ => actual_delta,
            });
            break;
        }
    }
}

/// Resize width from left (moves card) + height from top (moves card).
fn corner_drag_tl(
    delta_x: f32,
    delta_y: f32,
    entity_id: u64,
    card_node: &mut Node,
    scroll_outers: &mut Query<(&PinCardScrollOuter, &mut Node, &ComputedNode), Without<PinCard>>,
) {
    let current_w = match card_node.width {
        Val::Px(w) => w,
        _ => 280.0,
    };
    let new_w = (current_w - delta_x).max(180.0);
    let actual_dx = current_w - new_w;
    card_node.width = Val::Px(new_w);
    card_node.left = Val::Px(match card_node.left {
        Val::Px(x) => x + actual_dx,
        _ => actual_dx,
    });
    for (outer, mut sn, computed) in scroll_outers.iter_mut() {
        if outer.entity_id == entity_id {
            let current_h = match sn.height {
                Val::Px(h) => h,
                _ => computed.size().y,
            };
            let new_h = (current_h - delta_y).max(60.0);
            let actual_dy = current_h - new_h;
            sn.height = Val::Px(new_h);
            card_node.top = Val::Px(match card_node.top {
                Val::Px(y) => y + actual_dy,
                _ => actual_dy,
            });
            break;
        }
    }
}

fn corner_save(
    entity_id: u64,
    card_node: &Node,
    scroll_outers: &Query<(&PinCardScrollOuter, &Node), Without<PinCard>>,
    save_data: &mut Persistent<PinboardSaveData>,
) {
    let left = match card_node.left {
        Val::Px(v) => v,
        _ => return,
    };
    let top = match card_node.top {
        Val::Px(v) => v,
        _ => return,
    };
    let width = match card_node.width {
        Val::Px(v) => v,
        _ => return,
    };
    let height = scroll_outers
        .iter()
        .find(|(o, _)| o.entity_id == entity_id)
        .and_then(|(_, n)| {
            if let Val::Px(h) = n.height {
                Some(h)
            } else {
                None
            }
        });
    let Some(height) = height else { return };
    for entry in save_data.cards.iter_mut() {
        if entry.entity_id == entity_id {
            entry.left = left;
            entry.top = top;
            entry.width = width;
            entry.height = height;
            break;
        }
    }
    save_data.persist().ok();
}

// ── BR corner ─────────────────────────────────────────────────────────────────

fn on_resize_corner_br_added(trigger: On<Add, PinCardResizeCornerBR>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .observe(
            |trigger: On<Pointer<Drag>>,
             child_of: Query<&ChildOf>,
             pin_cards: Query<&PinCard>,
             mut card_nodes: Query<&mut Node, With<PinCard>>,
             mut scroll_outers: Query<
                (&PinCardScrollOuter, &mut Node, &ComputedNode),
                Without<PinCard>,
            >| {
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok(pc) = pin_cards.get(p.0) else { return };
                let Ok(mut cn) = card_nodes.get_mut(p.0) else {
                    return;
                };
                corner_drag_br(
                    trigger.event.delta.x,
                    trigger.event.delta.y,
                    pc.entity_id,
                    &mut cn,
                    &mut scroll_outers,
                );
            },
        )
        .observe(
            |trigger: On<Pointer<DragEnd>>,
             child_of: Query<&ChildOf>,
             pin_cards: Query<&PinCard>,
             card_nodes: Query<&Node, With<PinCard>>,
             scroll_outers: Query<(&PinCardScrollOuter, &Node), Without<PinCard>>,
             mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>| {
                let Some(sd) = save_data.as_mut() else { return };
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok(pc) = pin_cards.get(p.0) else { return };
                let Ok(cn) = card_nodes.get(p.0) else { return };
                corner_save(pc.entity_id, cn, &scroll_outers, sd);
            },
        )
        .observe(
            |_trigger: On<Pointer<Over>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::SeResize));
                }
            },
        )
        .observe(
            |_trigger: On<Pointer<Out>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::Default));
                }
            },
        );
}

// ── BL corner ─────────────────────────────────────────────────────────────────

fn on_resize_corner_bl_added(trigger: On<Add, PinCardResizeCornerBL>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .observe(
            |trigger: On<Pointer<Drag>>,
             child_of: Query<&ChildOf>,
             pin_cards: Query<&PinCard>,
             mut card_nodes: Query<&mut Node, With<PinCard>>,
             mut scroll_outers: Query<
                (&PinCardScrollOuter, &mut Node, &ComputedNode),
                Without<PinCard>,
            >| {
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok(pc) = pin_cards.get(p.0) else { return };
                let Ok(mut cn) = card_nodes.get_mut(p.0) else {
                    return;
                };
                corner_drag_bl(
                    trigger.event.delta.x,
                    trigger.event.delta.y,
                    pc.entity_id,
                    &mut cn,
                    &mut scroll_outers,
                );
            },
        )
        .observe(
            |trigger: On<Pointer<DragEnd>>,
             child_of: Query<&ChildOf>,
             pin_cards: Query<&PinCard>,
             card_nodes: Query<&Node, With<PinCard>>,
             scroll_outers: Query<(&PinCardScrollOuter, &Node), Without<PinCard>>,
             mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>| {
                let Some(sd) = save_data.as_mut() else { return };
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok(pc) = pin_cards.get(p.0) else { return };
                let Ok(cn) = card_nodes.get(p.0) else { return };
                corner_save(pc.entity_id, cn, &scroll_outers, sd);
            },
        )
        .observe(
            |_trigger: On<Pointer<Over>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::SwResize));
                }
            },
        )
        .observe(
            |_trigger: On<Pointer<Out>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::Default));
                }
            },
        );
}

// ── TR corner ─────────────────────────────────────────────────────────────────

fn on_resize_corner_tr_added(trigger: On<Add, PinCardResizeCornerTR>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .observe(
            |trigger: On<Pointer<Drag>>,
             child_of: Query<&ChildOf>,
             pin_cards: Query<&PinCard>,
             mut card_nodes: Query<&mut Node, With<PinCard>>,
             mut scroll_outers: Query<
                (&PinCardScrollOuter, &mut Node, &ComputedNode),
                Without<PinCard>,
            >| {
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok(pc) = pin_cards.get(p.0) else { return };
                let Ok(mut cn) = card_nodes.get_mut(p.0) else {
                    return;
                };
                corner_drag_tr(
                    trigger.event.delta.x,
                    trigger.event.delta.y,
                    pc.entity_id,
                    &mut cn,
                    &mut scroll_outers,
                );
            },
        )
        .observe(
            |trigger: On<Pointer<DragEnd>>,
             child_of: Query<&ChildOf>,
             pin_cards: Query<&PinCard>,
             card_nodes: Query<&Node, With<PinCard>>,
             scroll_outers: Query<(&PinCardScrollOuter, &Node), Without<PinCard>>,
             mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>| {
                let Some(sd) = save_data.as_mut() else { return };
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok(pc) = pin_cards.get(p.0) else { return };
                let Ok(cn) = card_nodes.get(p.0) else { return };
                corner_save(pc.entity_id, cn, &scroll_outers, sd);
            },
        )
        .observe(
            |_trigger: On<Pointer<Over>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::NeResize));
                }
            },
        )
        .observe(
            |_trigger: On<Pointer<Out>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::Default));
                }
            },
        );
}

// ── TL corner ─────────────────────────────────────────────────────────────────

fn on_resize_corner_tl_added(trigger: On<Add, PinCardResizeCornerTL>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .observe(
            |trigger: On<Pointer<Drag>>,
             child_of: Query<&ChildOf>,
             pin_cards: Query<&PinCard>,
             mut card_nodes: Query<&mut Node, With<PinCard>>,
             mut scroll_outers: Query<
                (&PinCardScrollOuter, &mut Node, &ComputedNode),
                Without<PinCard>,
            >| {
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok(pc) = pin_cards.get(p.0) else { return };
                let Ok(mut cn) = card_nodes.get_mut(p.0) else {
                    return;
                };
                corner_drag_tl(
                    trigger.event.delta.x,
                    trigger.event.delta.y,
                    pc.entity_id,
                    &mut cn,
                    &mut scroll_outers,
                );
            },
        )
        .observe(
            |trigger: On<Pointer<DragEnd>>,
             child_of: Query<&ChildOf>,
             pin_cards: Query<&PinCard>,
             card_nodes: Query<&Node, With<PinCard>>,
             scroll_outers: Query<(&PinCardScrollOuter, &Node), Without<PinCard>>,
             mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>| {
                let Some(sd) = save_data.as_mut() else { return };
                let Ok(p) = child_of.get(trigger.entity) else {
                    return;
                };
                let Ok(pc) = pin_cards.get(p.0) else { return };
                let Ok(cn) = card_nodes.get(p.0) else { return };
                corner_save(pc.entity_id, cn, &scroll_outers, sd);
            },
        )
        .observe(
            |_trigger: On<Pointer<Over>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::NwResize));
                }
            },
        )
        .observe(
            |_trigger: On<Pointer<Out>>,
             mut windows: Query<Entity, With<Window>>,
             mut commands: Commands| {
                for e in &mut windows {
                    commands
                        .entity(e)
                        .insert(CursorIcon::from(SystemCursorIcon::Default));
                }
            },
        );
}

/// When a scroll outer node is first tagged, set `height` from save data so the
/// loaded height matches exactly what was saved (not just a `max_height` cap).
fn restore_scroll_height(
    added: Query<(Entity, &PinCardScrollOuter), Added<PinCardScrollOuter>>,
    save_data: Option<Res<Persistent<PinboardSaveData>>>,
    mut nodes: Query<&mut Node>,
) {
    for (entity, outer) in &added {
        let saved_height = save_data
            .as_ref()
            .and_then(|sd| sd.cards.iter().find(|c| c.entity_id == outer.entity_id))
            .map(|c| c.height)
            .unwrap_or(300.0);
        if let Ok(mut node) = nodes.get_mut(entity) {
            node.height = Val::Px(saved_height);
        }
    }
}

fn on_resize_handle_left_added(trigger: On<Add, PinCardResizeHandleLeft>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .observe(on_resize_left_drag)
        .observe(on_resize_left_drag_end)
        .observe(on_resize_over)
        .observe(on_resize_out);
}

fn on_resize_left_drag(
    trigger: On<Pointer<Drag>>,
    child_of: Query<&ChildOf>,
    mut nodes: Query<&mut Node, With<PinCard>>,
) {
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok(mut node) = nodes.get_mut(parent.0) else {
        return;
    };
    let delta = trigger.event.delta.x;
    let current_width = match node.width {
        Val::Px(w) => w,
        _ => 280.0,
    };
    let new_width = (current_width - delta).max(180.0);
    // Only move left by the amount the width actually changed.
    let actual_delta = current_width - new_width;
    node.width = Val::Px(new_width);
    node.left = Val::Px(match node.left {
        Val::Px(x) => x + actual_delta,
        _ => actual_delta,
    });
}

fn on_resize_left_drag_end(
    trigger: On<Pointer<DragEnd>>,
    child_of: Query<&ChildOf>,
    nodes: Query<&Node, With<PinCard>>,
    pin_cards: Query<&PinCard>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
) {
    let Some(save_data) = save_data.as_mut() else {
        return;
    };
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok(node) = nodes.get(parent.0) else {
        return;
    };
    let Ok(pin_card) = pin_cards.get(parent.0) else {
        return;
    };
    let (Val::Px(left), Val::Px(width)) = (node.left, node.width) else {
        return;
    };
    for entry in save_data.cards.iter_mut() {
        if entry.entity_id == pin_card.entity_id {
            entry.left = left;
            entry.width = width;
            break;
        }
    }
    save_data.persist().ok();
}

fn on_resize_handle_top_added(trigger: On<Add, PinCardResizeHandleTop>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .observe(on_resize_top_drag)
        .observe(on_resize_top_drag_end)
        .observe(on_resize_bottom_over)
        .observe(on_resize_bottom_out);
}

fn on_resize_top_drag(
    trigger: On<Pointer<Drag>>,
    child_of: Query<&ChildOf>,
    pin_cards: Query<&PinCard>,
    mut card_nodes: Query<&mut Node, With<PinCard>>,
    mut scroll_outers: Query<(&PinCardScrollOuter, &mut Node, &ComputedNode), Without<PinCard>>,
) {
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok(pin_card) = pin_cards.get(parent.0) else {
        return;
    };
    let delta = trigger.event.delta.y;

    // Find scroll outer first to know how much height we can give up.
    for (outer, mut scroll_node, computed) in &mut scroll_outers {
        if outer.entity_id == pin_card.entity_id {
            let current_h = match scroll_node.height {
                Val::Px(h) => h,
                _ => computed.size().y,
            };
            let new_h = (current_h - delta).max(60.0);
            let actual_delta = current_h - new_h; // how much height actually changed
            scroll_node.height = Val::Px(new_h);

            // Move the card top by the same amount so the bottom stays fixed.
            if let Ok(mut node) = card_nodes.get_mut(parent.0) {
                node.top = Val::Px(match node.top {
                    Val::Px(y) => y + actual_delta,
                    _ => actual_delta,
                });
            }
            break;
        }
    }
}

fn on_resize_top_drag_end(
    trigger: On<Pointer<DragEnd>>,
    child_of: Query<&ChildOf>,
    pin_cards: Query<&PinCard>,
    card_nodes: Query<&Node, With<PinCard>>,
    scroll_outers: Query<(&PinCardScrollOuter, &Node), Without<PinCard>>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
) {
    let Some(save_data) = save_data.as_mut() else {
        return;
    };
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok(pin_card) = pin_cards.get(parent.0) else {
        return;
    };
    let Ok(card_node) = card_nodes.get(parent.0) else {
        return;
    };
    let Val::Px(top) = card_node.top else {
        return;
    };
    for (outer, scroll_node) in &scroll_outers {
        if outer.entity_id == pin_card.entity_id {
            if let Val::Px(h) = scroll_node.height {
                for entry in save_data.cards.iter_mut() {
                    if entry.entity_id == pin_card.entity_id {
                        entry.top = top;
                        entry.height = h;
                        break;
                    }
                }
                save_data.persist().ok();
            }
            break;
        }
    }
}

fn on_drag_handle_added(trigger: On<Add, DragHandle>, mut commands: Commands) {
    commands.entity(trigger.entity).observe(on_drag);
    commands.entity(trigger.entity).observe(on_drag_end);
}

fn on_drag(trigger: On<Pointer<Drag>>, child_of: Query<&ChildOf>, mut nodes: Query<&mut Node>) {
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let Ok(mut node) = nodes.get_mut(parent.0) else {
        return;
    };
    let delta = trigger.event.delta;
    node.left = Val::Px(match node.left {
        Val::Px(x) => x + delta.x,
        _ => delta.x,
    });
    node.top = Val::Px(match node.top {
        Val::Px(y) => y + delta.y,
        _ => delta.y,
    });
}

fn on_drag_end(
    trigger: On<Pointer<DragEnd>>,
    child_of: Query<&ChildOf>,
    nodes: Query<&Node>,
    pin_cards: Query<&PinCard>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
) {
    let Some(save_data) = save_data.as_mut() else {
        return;
    };
    let Ok(parent) = child_of.get(trigger.entity) else {
        return;
    };
    let card_entity = parent.0;
    let Ok(node) = nodes.get(card_entity) else {
        return;
    };
    let Ok(pin_card) = pin_cards.get(card_entity) else {
        return;
    };

    let left = match node.left {
        Val::Px(x) => x,
        _ => 0.0,
    };
    let top = match node.top {
        Val::Px(y) => y,
        _ => 0.0,
    };

    for entry in save_data.cards.iter_mut() {
        if entry.entity_id == pin_card.entity_id {
            entry.left = left;
            entry.top = top;
            break;
        }
    }
    save_data.persist().ok();
}
