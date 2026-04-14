use crate::manager::connection::ServerUrl;
use crate::manager::entity_filter::fetch::{ComponentEntry, DiscoveredComponents};
use crate::manager::pinboard::{
    load_save::{PinboardPendingData, PinboardPendingItem, PinboardSaveData},
    pin_card::spawn_pin_card,
    ui::PinboardContainer,
};
use crate::prelude::entity_card::*;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{COLOR_LABEL_SECONDARY, COLOR_LABEL_TERTIARY};
use crate::ui_layout::theme::widgets::pin_button;

/// Sentinel stored in `EntityCardExpandState` for the top-level children header.
pub(super) const CHILDREN_SENTINEL: &str = "__children__";

/// Sentinel for an individual child entity row (keyed by the child's entity id).
pub(super) fn child_sentinel(child_id: u64) -> String {
    format!("__child__{}", child_id)
}

/// BRP type path for the `ChildOf` relationship component.
const CHILD_OF_TYPE: &str = "bevy_ecs::hierarchy::ChildOf";

/// Maximum tree depth rendered to guard against circular hierarchies.
const MAX_DEPTH: u32 = 8;

/// Query key used when adding child entities to `DiscoveredComponents`.
const CHILDREN_QUERY_KEY: &str = "__entity_card_children__";

/// Cache of direct child entity IDs for **any** entity seen via BRP.
///
/// Pinned cards use this to render the children tree; any entity that appears
/// as a parent in the ChildOf query is included, enabling grandchild look-up.
#[derive(Resource, Default)]
pub struct EntityCardChildrenCache(pub HashMap<u64, Vec<u64>>);

/// Marker on a pending ChildOf BRP world query.
#[derive(Component)]
struct ChildrenQueryCtx;

/// Context on a pending `brp_list_components` request for a child entity's Name.
#[derive(Component)]
struct ChildNameFetchCtx {
    entity_id: u64,
}

/// Payload on a pin button inside a child entity row.
#[derive(Component, Clone)]
pub(super) struct ChildPinButton {
    pub entity_id: u64,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<EntityCardChildrenCache>().add_systems(
        Update,
        (
            fetch_children_on_cache_change,
            fetch_name_for_children,
            on_child_pin_button,
        ),
    );
}

/// Issues one `world.query` for all `ChildOf` entities whenever component
/// data is refreshed.
fn fetch_children_on_cache_change(
    cache: Res<EntityCardDataCache>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    if !cache.is_changed() || cache.0.is_empty() {
        return;
    }
    let req = commands.brp_world_query(
        &server_url.0,
        json!({
            "data": { "components": [CHILD_OF_TYPE] },
            "filter": { "with": [CHILD_OF_TYPE], "without": [] },
            "strict": false
        }),
    );
    commands
        .entity(req)
        .insert(ChildrenQueryCtx)
        .observe(on_children_query_response)
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            commands.entity(trigger.entity).despawn();
        });
}

fn on_children_query_response(
    trigger: On<Add, RpcResponse<BrpWorldQuery>>,
    q: Query<&RpcResponse<BrpWorldQuery>, With<ChildrenQueryCtx>>,
    pinned: Res<EntityCardDataCache>,
    mut children_cache: ResMut<EntityCardChildrenCache>,
    mut commands: Commands,
) {
    let ecs_entity = trigger.entity;
    let Ok(response) = q.get(ecs_entity) else {
        commands.entity(ecs_entity).despawn();
        return;
    };
    if let Ok(data) = &response.data {
        // Clear only the top-level pinned entities so cards with no children
        // show an empty list, while preserving cached data for deeper nodes.
        children_cache.0.clear();
        for entity_id in pinned.0.keys() {
            children_cache.0.entry(*entity_id).or_default();
        }

        // Populate parent → children for every entity in the query result.
        for entry in &data.result {
            let Some(child_of_val) = entry.components.get(CHILD_OF_TYPE) else {
                continue;
            };
            // ChildOf(Entity) may serialize as a plain u64, {"bits": u64}, or {"0": u64}.
            let parent_id = child_of_val
                .as_u64()
                .or_else(|| child_of_val.get("bits").and_then(|v| v.as_u64()))
                .or_else(|| child_of_val.get("0").and_then(|v| v.as_u64()));
            let Some(parent_id) = parent_id else { continue };
            children_cache
                .0
                .entry(parent_id)
                .or_default()
                .push(entry.entity);
        }
    }
    commands.entity(ecs_entity).despawn();
}

/// When the children cache updates, issue `brp_list_components` for every
/// child entity not yet present in `DiscoveredComponents` so the poll system
/// can resolve their Name values and `display_label` returns a proper name.
fn fetch_name_for_children(
    children_cache: Res<EntityCardChildrenCache>,
    discovered: Res<DiscoveredComponents>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    if !children_cache.is_changed() {
        return;
    }

    let already_known: HashSet<u64> = discovered.0.iter().map(|e| e.entity).collect();

    let mut all_ids: HashSet<u64> = HashSet::default();
    for (parent, children) in &children_cache.0 {
        all_ids.insert(*parent);
        for child in children {
            all_ids.insert(*child);
        }
    }

    for entity_id in all_ids {
        if already_known.contains(&entity_id) {
            continue;
        }
        let req = commands.brp_list_components(&server_url.0, entity_id);
        commands
            .entity(req)
            .insert(ChildNameFetchCtx { entity_id })
            .observe(on_child_name_fetch)
            .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
                commands.entity(trigger.entity).despawn();
            });
    }
}

fn on_child_name_fetch(
    trigger: On<Add, RpcResponse<BrpListComponents>>,
    q: Query<(&RpcResponse<BrpListComponents>, &ChildNameFetchCtx)>,
    mut discovered: ResMut<DiscoveredComponents>,
    mut commands: Commands,
) {
    let ecs_entity = trigger.entity;
    let Ok((response, ctx)) = q.get(ecs_entity) else {
        commands.entity(ecs_entity).despawn();
        return;
    };
    if let Ok(data) = &response.data {
        // Guard against a race where another system already added this entity.
        if !discovered.0.iter().any(|e| e.entity == ctx.entity_id) {
            let name_type_path = data
                .result
                .iter()
                .find(|p| p.split("::").last().unwrap_or("") == "Name")
                .cloned();
            discovered.0.push(ComponentEntry {
                entity: ctx.entity_id,
                query: CHILDREN_QUERY_KEY.to_string(),
                name_type_path,
                value: None,
            });
        }
    }
    commands.entity(ecs_entity).despawn();
}

// ── Recursive tree rendering ──────────────────────────────────────────────────

/// Spawns a child entity row and, if expanded, its entire subtree into
/// `container_entity`. `depth` starts at 0 for direct children.
pub(super) fn render_child_tree(
    commands: &mut Commands,
    container_entity: Entity,
    card_entity_id: u64,
    child_id: u64,
    depth: u32,
    expand_state: &EntityCardExpandState,
    children_cache: &EntityCardChildrenCache,
    components: &Res<DiscoveredComponents>,
) {
    if depth > MAX_DEPTH {
        return;
    }

    let grandchildren = children_cache.0.get(&child_id).cloned().unwrap_or_default();
    let has_children = !grandchildren.is_empty();
    let sentinel = child_sentinel(child_id);
    let is_expanded = expand_state
        .0
        .get(&card_entity_id)
        .map(|s| s.contains(&sentinel))
        .unwrap_or(false);

    let row = spawn_child_row(
        commands,
        card_entity_id,
        child_id,
        depth,
        has_children,
        is_expanded,
        components,
    );
    commands.entity(container_entity).add_child(row);

    if is_expanded && has_children {
        let mut sorted = grandchildren;
        sorted.sort_unstable();
        for grandchild_id in sorted {
            render_child_tree(
                commands,
                container_entity,
                card_entity_id,
                grandchild_id,
                depth + 1,
                expand_state,
                children_cache,
                components,
            );
        }
    }
}

fn spawn_child_row(
    commands: &mut Commands,
    card_entity_id: u64,
    child_id: u64,
    depth: u32,
    has_children: bool,
    is_expanded: bool,
    components: &Res<DiscoveredComponents>,
) -> Entity {
    let indent = 42.0 + (depth as f32) * 2.0;
    let icon = if has_children {
        if is_expanded { "V" } else { ">" }
    } else {
        " "
    };
    let label = components.display_label(child_id);

    if has_children {
        commands
            .spawn_scene(bsn! {
                Button
                EntityCardExpandToggle {
                    entity_id: { card_entity_id },
                    type_path: { child_sentinel(child_id) },
                }
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                    padding: UiRect {
                        left: Val::Px({ indent }),
                        right: Val::Px(6.0),
                        top: Val::Px(2.0),
                        bottom: Val::Px(2.0),
                    },
                    column_gap: Val::Px(4.0),
                }
                BackgroundColor(Color::NONE)
                Children [
                    (
                        Text::new( icon.to_string() )
                        template(|_| Ok(TextFont::from_font_size(9.0)))
                        TextColor(COLOR_LABEL_TERTIARY)
                    ),
                    (
                        Text::new( label.clone() )
                        template(|_| Ok(TextFont::from_font_size(11.0)))
                        TextColor(COLOR_LABEL_SECONDARY)
                        TextLayout { linebreak: LineBreak::NoWrap }
                    ),
                    (
                        pin_button::pin_button(ChildPinButton { entity_id: child_id })
                    )
                ]
            })
            .id()
    } else {
        commands
            .spawn_scene(bsn! {
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                    padding: UiRect {
                        left: Val::Px({ indent }),
                        right: Val::Px(6.0),
                        top: Val::Px(2.0),
                        bottom: Val::Px(2.0),
                    },
                    column_gap: Val::Px(4.0),
                }
                Children [
                    (
                        Text::new( icon.to_string() )
                        template(|_| Ok(TextFont::from_font_size(9.0)))
                        TextColor(COLOR_LABEL_TERTIARY)
                    ),
                    (
                        Text::new( label.clone() )
                        template(|_| Ok(TextFont::from_font_size(11.0)))
                        TextColor(COLOR_LABEL_SECONDARY)
                        TextLayout { linebreak: LineBreak::NoWrap }
                    ),
                    (

                        pin_button::pin_button(ChildPinButton { entity_id: child_id })
                    )
                ]
            })
            .id()
    }
}

fn on_child_pin_button(
    buttons: Query<(&Interaction, &ChildPinButton), (Changed<Interaction>, With<Button>)>,
    pinboard: Query<Entity, With<PinboardContainer>>,
    components: Res<DiscoveredComponents>,
    mut save_data: Option<ResMut<Persistent<PinboardSaveData>>>,
    mut pending: ResMut<PinboardPendingItem>,
    mut next_sidebar: ResMut<NextState<SidebarState>>,
    mut commands: Commands,
) {
    for (interaction, btn) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let entity_id = btn.entity_id;

        if save_data.as_ref().map_or(false, |sd| {
            sd.cards.iter().any(|c| c.entity_id == entity_id)
        }) {
            pending.0.push(PinboardPendingData {
                entity_id,
                key: entity_card_key(entity_id),
                highlight: true,
            });
            next_sidebar.set(SidebarState::Pinboard);
            continue;
        }

        let Ok(pinboard_entity) = pinboard.single() else {
            continue;
        };

        let label = components.display_label(entity_id);
        let left = 20.0;
        let top = 20.0;
        let width = 280.0;
        let height = 400.0;
        let key = entity_card_key(entity_id);

        let panel = commands
            .spawn_scene(spawn_pin_card(
                label.clone(),
                entity_id,
                left,
                top,
                width,
                height,
            ))
            .id();
        commands.entity(pinboard_entity).add_child(panel);

        if let Some(sd) = save_data.as_mut() {
            sd.cards.push(EntityCardEntry {
                entity_id,
                label: label.clone(),
                left,
                top,
                width,
                height,
            });
            sd.persist().ok();
        }

        pending.0.push(PinboardPendingData {
            entity_id,
            key,
            highlight: true,
        });
        next_sidebar.set(SidebarState::Pinboard);
    }
}

// ── Top-level header scene ────────────────────────────────────────────────────

pub(super) fn children_header(entity_id: u64, is_expanded: bool, count: usize) -> impl Scene {
    let icon = if is_expanded { "V" } else { ">" };
    let label = format!("Children ({})", count);
    bsn! {
        Button
        EntityCardExpandToggle {
            entity_id: { entity_id },
            type_path: { CHILDREN_SENTINEL.to_string() },
        }
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
            column_gap: Val::Px(4.0),
        }
        BackgroundColor(Color::NONE)
        Children [
            (Node { width: Val::Px(20.0), height: Val::Px(20.0), flex_shrink: 0.0 }),
            (
                Text::new( icon.to_string() )
                template(|_| Ok(TextFont::from_font_size(9.0)))
                TextColor(COLOR_LABEL_TERTIARY)
            ),
            (
                Text::new( label.clone() )
                template(|_| Ok(TextFont::from_font_size(12.0)))
                TextColor(COLOR_LABEL_SECONDARY)
                TextLayout { linebreak: LineBreak::NoWrap }
            ),
        ]
    }
}
