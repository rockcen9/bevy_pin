use crate::{manager::SidebarState, prelude::*};
use std::sync::Arc;

use super::get::DiscoveredStates;
use crate::manager::connection::ServerUrl;

use crate::ui_layout::theme::palette::{
    COLOR_ACTIVE, COLOR_BUTTON_TEXT, COLOR_DISABLED, COLOR_HOVER, COLOR_INACTIVE,
};
use crate::ui_layout::theme::widgets::{titled_panel, ScrollableContainer};

#[derive(Component, Clone, Default)]
#[require(DespawnOnExit::<SidebarState>(SidebarState::State), Name::new("StatePanelRoot"))]
pub struct StatePanelsRoot;


#[derive(Component, Clone, Default)]
struct StateButton {
    variant: Arc<str>,
    state_type_path: Arc<str>,
}

#[derive(Resource, Default)]
struct SpawnedStatePanels(HashSet<String>);

#[derive(Resource, Default)]
struct SpawnedStateButtons(HashSet<String>);

pub fn plugin(app: &mut App) {
    app.init_resource::<SpawnedStatePanels>()
        .init_resource::<SpawnedStateButtons>()
        .add_systems(
            OnExit(SidebarState::State),
            (clear_spawned_panels, clear_spawned_buttons),
        )
        .add_systems(
            Update,
            (
                spawn_state_panels,
                spawn_state_buttons,
                update_button_colors,
                update_button_hover,
                handle_state_button_press,
            )
                .chain(),
        );
}

fn clear_spawned_panels(mut spawned: ResMut<SpawnedStatePanels>) {
    spawned.0.clear();
}

fn clear_spawned_buttons(mut spawned: ResMut<SpawnedStateButtons>) {
    spawned.0.clear();
}

pub fn state_panels_root() -> impl Scene {
    bsn! {
        #StatePanelRoot
        StatePanelsRoot
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            flex_wrap: FlexWrap::Wrap
        }
    }
}


fn state_button(variant: Arc<str>, state_type_path: Arc<str>) -> impl Scene {
    let label = variant.to_string();

    bsn! {
        Button
        Node {
            padding: UiRect::axes(Val::Px(14.0), Val::Px(7.0)),
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.0),
            border_radius: BorderRadius::all(Val::Px(6.0)),
        }
        BackgroundColor(COLOR_INACTIVE)
        StateButton {
            variant: { variant.clone() },
            state_type_path: { state_type_path.clone() },
        }
        Children [(
            Text::new( label.clone() )
            template(|_| Ok(TextFont::from_font_size(14.0)))
            TextColor(COLOR_BUTTON_TEXT)
        )]
    }
}

fn spawn_state_panels(
    mut commands: Commands,
    states: Res<DiscoveredStates>,
    root: Query<Entity, With<StatePanelsRoot>>,
    mut spawned: ResMut<SpawnedStatePanels>,
) {
    if !states.is_changed() {
        return;
    }

    let Ok(root_entity) = root.single() else {
        return;
    };

    for entry in &states.0 {
        if spawned.0.contains(&entry.state_type_path) {
            continue;
        }
        spawned.0.insert(entry.state_type_path.clone());

        debug!("Spawning panel for state: {}", entry.state_type_path);

        let panel = commands
            .spawn_scene(titled_panel(
                entry.label.clone(),
                entry.state_type_path.clone(),
                300.0,
            ))
            .id();
        commands.entity(root_entity).add_child(panel);
    }
}

fn spawn_state_buttons(
    mut commands: Commands,
    states: Res<DiscoveredStates>,
    containers: Query<(Entity, &ScrollableContainer)>,
    mut spawned: ResMut<SpawnedStateButtons>,
) {
    if !states.is_changed() {
        return;
    }

    for entry in &states.0 {
        if entry.variants.is_empty() || spawned.0.contains(&entry.state_type_path) {
            continue;
        }

        let Some((container_entity, _)) = containers
            .iter()
            .find(|(_, c)| c.0 == entry.state_type_path)
        else {
            continue;
        };

        spawned.0.insert(entry.state_type_path.clone());

        debug!(
            "Spawning {} buttons for state: {}",
            entry.variants.len(),
            entry.state_type_path
        );

        let type_path_arc: Arc<str> = entry.state_type_path.as_str().into();

        for variant in &entry.variants {
            let btn = commands
                .spawn_scene(state_button(variant.as_str().into(), type_path_arc.clone()))
                .id();
            commands.entity(container_entity).add_child(btn);
        }
    }
}

fn button_color(state_exists: bool, is_active: bool, interaction: &Interaction) -> Color {
    if !state_exists {
        return COLOR_DISABLED;
    }
    match (is_active, interaction) {
        (true, _) => COLOR_ACTIVE,
        (false, Interaction::Hovered) => COLOR_HOVER,
        _ => COLOR_INACTIVE,
    }
}

fn update_button_colors(
    states: Res<DiscoveredStates>,
    mut buttons: Query<(&StateButton, &Interaction, &mut BackgroundColor)>,
) {
    if !states.is_changed() {
        return;
    }
    for (button, interaction, mut color) in &mut buttons {
        let entry = states
            .0
            .iter()
            .find(|e| e.state_type_path == &*button.state_type_path);
        let current = entry.and_then(|e| e.current.as_deref());
        let state_exists = entry.map_or(false, |e| e.current.is_some());
        let is_active = current == Some(&*button.variant);
        debug!(
            "[update_button_colors] {} / {} | current={:?} exists={} active={}",
            button.state_type_path, button.variant, current, state_exists, is_active
        );
        color.set_if_neq(BackgroundColor(button_color(
            state_exists,
            is_active,
            interaction,
        )));
    }
}

fn update_button_hover(
    states: Res<DiscoveredStates>,
    mut buttons: Query<(&StateButton, &Interaction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (button, interaction, mut color) in &mut buttons {
        let entry = states
            .0
            .iter()
            .find(|e| e.state_type_path == &*button.state_type_path);
        let current = entry.and_then(|e| e.current.as_deref());
        let state_exists = entry.map_or(false, |e| e.current.is_some());
        let is_active = current == Some(&*button.variant);
        color.set_if_neq(BackgroundColor(button_color(
            state_exists,
            is_active,
            interaction,
        )));
    }
}

fn handle_state_button_press(
    query: Query<(&Interaction, &StateButton), Changed<Interaction>>,
    states: Res<DiscoveredStates>,
    server_url: Res<ServerUrl>,
    mut commands: Commands,
) {
    for (interaction, button) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(path) = states
            .0
            .iter()
            .find(|e| e.state_type_path == &*button.state_type_path)
            .and_then(|e| e.next_state_resource.clone())
        else {
            error!("NextState resource not found — cannot switch state");
            continue;
        };

        let variant = button.variant.to_string();
        let req = commands.brp_insert_resources(
            &server_url.0,
            &path,
            json!({ "Pending": variant }),
        );
        commands
            .entity(req)
            .observe(
                |trigger: On<Add, RpcResponse<BrpMutate>>,
                 query: Query<&RpcResponse<BrpMutate>>,
                 mut commands: Commands| {
                    let entity = trigger.entity;
                    if let Ok(response) = query.get(entity) {
                        match &response.data {
                            Ok(body) => info!("State switch response: {:?}", body.result),
                            Err(e) => error!("State switch failed: {}", e),
                        }
                    }
                    commands.entity(entity).despawn();
                },
            )
            .observe(
                |trigger: On<Add, TimeoutError>, mut commands: Commands| {
                    error!("State switch request timed out");
                    commands.entity(trigger.entity).despawn();
                },
            );
    }
}
