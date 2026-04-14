use crate::manager::connection::ServerUrl;
use crate::manager::entity_lookup::FoundEntity;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_BUTTON_BG, COLOR_BUTTON_HOVER, COLOR_INPUT_TEXT, COLOR_SEPARATOR,
};
use crate::ui_layout::theme::widgets::{ScrollableContainer, titled_panel};

pub struct LookupEntry {
    pub entity_id: u64,
    pub name: Option<String>,
}

#[derive(Resource, Default)]
pub struct LookupHistory(pub Vec<LookupEntry>);

impl LookupHistory {
    pub fn push(&mut self, entity_id: u64) {
        self.0.retain(|e| e.entity_id != entity_id);
        self.0.insert(
            0,
            LookupEntry {
                entity_id,
                name: None,
            },
        );
    }

    pub fn update_name(&mut self, entity_id: u64, name: String) {
        if let Some(entry) = self.0.iter_mut().find(|e| e.entity_id == entity_id) {
            entry.name = Some(name);
        }
    }
}

#[derive(Component, Clone)]
struct HistoryItem(u64);

#[derive(Resource)]
struct HistoryRefreshTimer(Timer);

impl Default for HistoryRefreshTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(5.0, TimerMode::Repeating))
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<LookupHistory>()
        .init_resource::<HistoryRefreshTimer>()
        .add_systems(
            Update,
            (
                refresh_history,
                rebuild_history_panel,
                handle_history_item_click,
                update_history_item_hover,
            ),
        );
}

pub fn history_panel() -> impl Scene {
    bsn! {
        titled_panel("Lookup History", "lookup-history", 300.0)
        DespawnOnExit::<SidebarState>(SidebarState::EntityLookup)
    }
}

fn refresh_history(
    time: Res<Time>,
    mut timer: ResMut<HistoryRefreshTimer>,
    history: Res<LookupHistory>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() || history.0.is_empty() {
        return;
    }

    let req = commands.brp_world_query(
        &server_url.0,
        json!({
            "data": { "components": [], "option": "all", "has": [] },
            "filter": { "with": [], "without": [] },
            "strict": false
        }),
    );
    commands
        .entity(req)
        .observe(
            |trigger: On<Add, RpcResponse<BrpWorldQuery>>,
             q: Query<&RpcResponse<BrpWorldQuery>>,
             mut history: ResMut<LookupHistory>,
             mut commands: Commands| {
                let ecs_entity = trigger.entity;
                let Ok(response) = q.get(ecs_entity) else {
                    commands.entity(ecs_entity).despawn();
                    return;
                };
                if let Ok(data) = &response.data {
                    let live_ids: Vec<u64> = data.result.iter().map(|e| e.entity).collect();
                    history.0.retain(|e| live_ids.contains(&e.entity_id));
                    for entry in &data.result {
                        let name = entry
                            .components
                            .as_object()
                            .and_then(|m| {
                                let key = m
                                    .keys()
                                    .find(|k| k.split("::").last().unwrap_or("") == "Name")?;
                                m.get(key)
                            })
                            .and_then(|v| match v {
                                serde_json::Value::String(s) => Some(s.clone()),
                                v => v
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .map(|s| s.to_string()),
                            });
                        if let Some(name) = name {
                            history.update_name(entry.entity, name);
                        }
                    }
                }
                commands.entity(ecs_entity).despawn();
            },
        )
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            commands.entity(trigger.entity).despawn();
        });
}

fn rebuild_history_panel(
    history: Res<LookupHistory>,
    containers: Query<(Entity, &ScrollableContainer)>,
    added_containers: Query<&ScrollableContainer, Added<ScrollableContainer>>,
    mut commands: Commands,
) {
    let is_newly_added = added_containers.iter().any(|c| c.0 == "lookup-history");
    if !history.is_changed() && !is_newly_added {
        return;
    }

    for (container, _) in containers.iter().filter(|(_, c)| c.0 == "lookup-history") {
        commands.entity(container).despawn_children();

        if history.0.is_empty() {
            let child = commands
                .spawn((
                    Text::new("No lookups yet"),
                    TextFont::from_font_size(12.0),
                    TextColor(COLOR_SEPARATOR),
                ))
                .id();
            commands.entity(container).add_child(child);
            continue;
        }

        for entry in &history.0 {
            let entity_id = entry.entity_id;
            let label = match &entry.name {
                Some(name) => format!("{} {}", crate::utils::entity_display_label(entity_id), name),
                None => crate::utils::entity_display_label(entity_id),
            };
            let child = commands
                .spawn((
                    Button,
                    HistoryItem(entity_id),
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(COLOR_BUTTON_BG),
                ))
                .with_child((
                    Text::new(label),
                    TextFont::from_font_size(13.0),
                    TextColor(COLOR_INPUT_TEXT),
                ))
                .id();
            commands.entity(container).add_child(child);
        }
    }
}

fn handle_history_item_click(
    items: Query<(&Interaction, &HistoryItem), Changed<Interaction>>,
    mut inspected: ResMut<FoundEntity>,
) {
    for (interaction, item) in &items {
        if *interaction == Interaction::Pressed {
            debug!("EntityLookup history: selecting entity #{}", item.0);
            inspected.0 = Some(item.0);
        }
    }
}

fn update_history_item_hover(
    mut items: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<HistoryItem>),
    >,
) {
    for (interaction, mut color) in &mut items {
        color.set_if_neq(BackgroundColor(match interaction {
            Interaction::Hovered => COLOR_BUTTON_HOVER,
            _ => COLOR_BUTTON_BG,
        }));
    }
}
