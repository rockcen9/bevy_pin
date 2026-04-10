use crate::manager::connection::ServerUrl;
use crate::manager::entity_filter::component_list::InspectedEntity;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_BUTTON_BG, COLOR_BUTTON_HOVER, COLOR_DESTRUCTIVE_HOVER, COLOR_INPUT_TEXT,
    COLOR_MENU_NORMAL, COLOR_SEPARATOR,
};
use crate::ui_layout::theme::widgets::{ScrollableContainer, close_button, titled_panel};

#[derive(Clone)]
pub struct SpawnEntry {
    pub type_name: String,
    pub entity_id: u64,
}

#[derive(Resource, Default)]
pub struct SpawnedEntities(pub Vec<SpawnEntry>);

#[derive(Component, Clone)]
pub struct SpawnedItem {
    pub type_name: String,
    pub entity_id: u64,
}

/// Despawns the remote entity via BRP when clicked.
#[derive(Component, Clone)]
pub struct DespawnEntityButton(pub u64);

#[derive(Component, Clone, Default, Reflect)]
struct SpawnedPanelRoot;

pub fn spawned_panel() -> impl Scene {
    bsn! {
        titled_panel("Spawned", "spawned-entities", 300.0)
        #SpawnedPanelRoot
        SpawnedPanelRoot
        DespawnOnExit::<SidebarState>(SidebarState::NewScene)
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<SpawnedEntities>().add_systems(
        Update,
        (
            rebuild_spawned_panel,
            handle_spawned_item_click,
            update_spawned_item_hover,
            handle_despawn_button_click,
            update_despawn_button_hover,
        ),
    );
}

fn rebuild_spawned_panel(
    spawned: Res<SpawnedEntities>,
    containers: Query<(Entity, &ScrollableContainer)>,
    added_containers: Query<&ScrollableContainer, Added<ScrollableContainer>>,
    mut commands: Commands,
) {
    let is_newly_added = added_containers.iter().any(|c| c.0 == "spawned-entities");
    if !spawned.is_changed() && !is_newly_added {
        return;
    }
    debug!(
        "rebuild_spawned_panel: spawned.is_changed={}, is_newly_added={}, count={}",
        spawned.is_changed(),
        is_newly_added,
        spawned.0.len()
    );
    for (container, _) in containers.iter().filter(|(_, c)| c.0 == "spawned-entities") {
        debug!(
            "rebuild_spawned_panel: rebuilding container {:?}",
            container
        );
        commands.entity(container).despawn_children();

        if spawned.0.is_empty() {
            debug!("rebuild_spawned_panel: no entries, spawning empty hint");
            let child = commands.spawn_scene(empty_hint()).id();
            commands.entity(container).add_child(child);
            continue;
        }

        debug!("rebuild_spawned_panel: spawning {} items", spawned.0.len());
        for entry in spawned.0.iter().rev() {
            debug!(
                "rebuild_spawned_panel: spawning item type={} entity_id={}",
                entry.type_name, entry.entity_id
            );
            let child = commands.spawn_scene(spawned_item(entry.clone())).id();
            commands.entity(container).add_child(child);
        }
    }
}

fn spawned_item(entry: SpawnEntry) -> impl Scene {
    let label = format!(
        "{} {}",
        entry.type_name,
        crate::utils::entity_display_label(entry.entity_id)
    );
    let type_name_click = entry.type_name.clone();
    let entity_id_click = entry.entity_id;
    let entity_id = entry.entity_id;
    bsn! {
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            column_gap: Val::Px(4.0),
        }
        Children [
            close_button(DespawnEntityButton(entity_id)),
            (
                template(move |_| Ok(SpawnedItem { type_name: type_name_click.clone(), entity_id: entity_id_click }))
                Button
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    flex_grow: 1.0,
                }
                BackgroundColor(COLOR_BUTTON_BG)
                Children [(
                    template(move |_| Ok(Text::new(label.clone())))
                    template(|_| Ok(TextFont::from_font_size(13.0)))
                    TextColor(COLOR_INPUT_TEXT)
                )]
            ),
        ]
    }
}

fn empty_hint() -> impl Scene {
    bsn! {
        Text::new("No spawns yet")
        template(|_| Ok(TextFont::from_font_size(12.0)))
        TextColor(COLOR_SEPARATOR)
    }
}

fn handle_spawned_item_click(
    items: Query<(&Interaction, &SpawnedItem), Changed<Interaction>>,
    mut inspected: ResMut<InspectedEntity>,
) {
    for (interaction, item) in &items {
        debug!(
            "handle_spawned_item_click: interaction={:?} type={} entity_id={}",
            interaction, item.type_name, item.entity_id
        );
        if *interaction == Interaction::Pressed {
            debug!(
                "spawned item clicked: {} entity #{} -> setting inspected",
                item.type_name, item.entity_id
            );
            inspected.0 = Some(item.entity_id);
        }
    }
}

fn update_spawned_item_hover(
    mut items: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SpawnedItem>),
    >,
) {
    for (interaction, mut color) in &mut items {
        color.set_if_neq(BackgroundColor(match interaction {
            Interaction::Hovered => COLOR_BUTTON_HOVER,
            _ => COLOR_BUTTON_BG,
        }));
    }
}

fn handle_despawn_button_click(
    items: Query<(&Interaction, &DespawnEntityButton), Changed<Interaction>>,
    server_url: Res<ServerUrl>,
    mut spawned: ResMut<SpawnedEntities>,
    mut commands: Commands,
) {
    for (interaction, btn) in &items {
        debug!(
            "handle_despawn_button_click: interaction={:?} entity_id={}",
            interaction, btn.0
        );
        if *interaction != Interaction::Pressed {
            continue;
        }
        let entity_id = btn.0;
        debug!(
            "handle_despawn_button_click: sending BRP despawn for entity_id={} url={}",
            entity_id, server_url.0
        );
        let req = commands.brp_despawn_entity(&server_url.0, entity_id);
        debug!("handle_despawn_button_click: BRP request entity={:?}", req);
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
                error!(
                    "handle_despawn_button_click: timeout waiting for BRP response entity={:?}",
                    trigger.entity
                );
                commands.entity(trigger.entity).despawn();
            });
        debug!(
            "handle_despawn_button_click: removing entity_id={} from spawned list (before: {})",
            entity_id,
            spawned.0.len()
        );
        spawned.0.retain(|e| e.entity_id != entity_id);
        debug!(
            "handle_despawn_button_click: spawned list after retain: {}",
            spawned.0.len()
        );
    }
}

fn update_despawn_button_hover(
    mut items: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<DespawnEntityButton>),
    >,
) {
    for (interaction, mut color) in &mut items {
        color.set_if_neq(BackgroundColor(match interaction {
            Interaction::Hovered => COLOR_DESTRUCTIVE_HOVER,
            _ => COLOR_MENU_NORMAL,
        }));
    }
}
