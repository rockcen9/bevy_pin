use crate::{
    manager::{
        entity_filter::{
            component_list,
            entity_list::ui,
            inspector,
            query::{history::query_history_panel, insert::insert_panel, query_panel_root},
            ui::{left_query_root, right_info_root},
        },
        new_scene::{NewScenePanelRoot, spawned::spawned_panel, insert::spawn_entity_panel},
        resource::ui::resource_panels_root,
        state::ui::state_panels_root,
    },
    prelude::*,
    ui_layout::theme::palette::COLOR_BG_BASE,
};

#[derive(Component, Default, Clone, Reflect)]
pub struct ContentPanel;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        spawn_state_panel.run_if(in_state(SidebarState::State)),
    );
    app.add_systems(
        Update,
        spawn_resource_panel.run_if(in_state(SidebarState::Resource)),
    );
    app.add_systems(
        Update,
        spawn_component_panel.run_if(in_state(SidebarState::Component)),
    );
    app.add_systems(
        Update,
        spawn_new_scene_panel.run_if(in_state(SidebarState::NewScene)),
    );
}
pub fn content_panel() -> impl Scene {
    bsn! {
        #ContentPanel
        ContentPanel
        Node {
            flex_grow: 1.0,
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
        }
        BackgroundColor(COLOR_BG_BASE)
    }
}
fn spawn_state_panel(
    mut commands: Commands,
    content: Single<(Entity, Option<&Children>), With<ContentPanel>>,
) {
    let (entity, children) = *content;
    if children.map(|c| c.is_empty()).unwrap_or(true) {
        debug!("Spawning state panels root into ContentPanel");
        let child = commands.spawn_scene(state_panels_root()).id();
        commands.entity(entity).add_child(child);
    }
}

#[derive(Component, Default, Clone, Reflect)]
#[require(DespawnOnExit::<SidebarState>(SidebarState::Component))]
pub struct ComponentPanelRoot;

fn spawn_component_panel(
    mut commands: Commands,
    content: Single<(Entity, Option<&Children>), With<ContentPanel>>,
) {
    let (entity, children) = *content;
    if children.map(|c| c.is_empty()).unwrap_or(true) {
        debug!("Spawning component panels root into ContentPanel");
        let scene = bsn! {
            #ComponentPanelRoot
            ComponentPanelRoot
            Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
            }
            BackgroundColor(COLOR_BG_BASE)
            Children [
                (left_query_root()
                Children [
                    (
                        query_panel_root()
                        Children[
                            insert_panel(),
                            query_history_panel(),
                        ]
                    ),
                    ui::entity_list_panel(),
                ]),
                (right_info_root()
                Children [
                    component_list::component_data_panel(),
                    inspector::inspector_panel(),
                ]),
            ]
        };
        let child = commands.spawn_scene(scene).id();
        commands.entity(entity).add_child(child);
    }
}
fn spawn_resource_panel(
    mut commands: Commands,
    content: Single<(Entity, Option<&Children>), With<ContentPanel>>,
) {
    let (entity, children) = *content;
    if children.map(|c| c.is_empty()).unwrap_or(true) {
        debug!("Spawning resource panels root into ContentPanel");
        let child = commands.spawn_scene(resource_panels_root()).id();
        commands.entity(entity).add_child(child);
    }
}

fn spawn_new_scene_panel(
    mut commands: Commands,
    content: Single<(Entity, Option<&Children>), With<ContentPanel>>,
) {
    let (entity, children) = *content;
    if children.map(|c| c.is_empty()).unwrap_or(true) {
        debug!("Spawning new scene panel into ContentPanel");
        let scene = bsn! {
            #NewScenePanelRoot
            NewScenePanelRoot
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(20.0)),
                column_gap: Val::Px(12.0),
            }
            Children [
                (
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                    }
                    Children [
                        spawn_entity_panel(),
                        spawned_panel(),
                    ]
                ),
                component_list::component_data_panel(),
                inspector::inspector_panel(),
            ]
        };
        let child = commands.spawn_scene(scene).id();
        commands.entity(entity).add_child(child);
    }
}
// #[cfg(feature = "dev")]
// mod debug {
//     use bevy::prelude::*;
//     use bevy_inspector_egui::quick::FilterQueryInspectorPlugin;

//     use crate::ui::body::content::ContentPanel;

//     pub fn plugin(app: &mut App) {
//         app.add_plugins(
//             FilterQueryInspectorPlugin::<With<ContentPanel>>::default()
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
