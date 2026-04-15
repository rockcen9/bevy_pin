use super::SidebarState;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_BG_SURFACE, COLOR_LABEL_SECONDARY, COLOR_MENU_ACTIVE, COLOR_MENU_HOVER,
    COLOR_MENU_NORMAL, COLOR_SEPARATOR,
};

#[derive(Component, Default, Clone)]
pub struct MenuPanel;

#[derive(Component, Default, Clone)]
struct ComponentButton;

#[derive(Component, Default, Clone)]
struct ResourceButton;

#[derive(Component, Default, Clone)]
struct StateButton;

#[derive(Component, Default, Clone)]
struct NewScene;

#[derive(Component, Default, Clone)]
struct EntityLookupButton;

#[derive(Component, Default, Clone)]
struct WorkspaceButton;

#[derive(Component, Default, Clone)]
struct GithubButton;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            sync_menu_button_colors,
            on_component_button,
            on_resource_button,
            on_state_button,
            on_new_scene_button,
            on_entity_lookup_button,
            on_pinboard_button,
            on_github_button,
        )
            .run_if(in_state(ConnectionState::Connected)),
    );
}

pub fn menu_panel() -> impl Scene {
    bsn! {
        MenuPanel
        Node {
            width: Val::Px(200.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(12.0)),
            row_gap: Val::Px(2.0),
            border: UiRect::right(Val::Px(1.0)),
        }
        BackgroundColor(COLOR_BG_SURFACE)
        BorderColor::all(COLOR_SEPARATOR)
        Children [
            entity_query_button(),
            entity_lookup_button(),
            new_scene_button(),
            resource_button(),
            state_button(),
            workspace_button(),
       (
                Node {
                    flex_grow: 1.0,
                }
            ),
        ]
    }
}

fn entity_query_button() -> impl Scene {
    bsn! {
        ComponentButton
        Button
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(12.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        BackgroundColor(COLOR_MENU_NORMAL)
        Children [(
            Text("Entity Filter")
            template(|_| Ok(TextFont::from_font_size(15.0)))
            TextColor(COLOR_LABEL_SECONDARY)
        )]
    }
}

fn resource_button() -> impl Scene {
    bsn! {
        ResourceButton
        Button
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(12.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        BackgroundColor(COLOR_MENU_NORMAL)
        Children [(
            Text("Resources")
            template(|_| Ok(TextFont::from_font_size(15.0)))
            TextColor(COLOR_LABEL_SECONDARY)
        )]
    }
}

fn state_button() -> impl Scene {
    bsn! {
        StateButton
        Button
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(12.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        BackgroundColor(COLOR_MENU_NORMAL)
        Children [(
            Text("States")
            template(|_| Ok(TextFont::from_font_size(15.0)))
            TextColor(COLOR_LABEL_SECONDARY)
        )]
    }
}

fn workspace_button() -> impl Scene {
    bsn! {
        WorkspaceButton
        Button
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(12.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        BackgroundColor(COLOR_MENU_NORMAL)
        Children [(
            Text("Workspace")
            template(|_| Ok(TextFont::from_font_size(15.0)))
            TextColor(COLOR_LABEL_SECONDARY)
        )]
    }
}

fn new_scene_button() -> impl Scene {
    bsn! {
        NewScene
        Button
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(12.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        BackgroundColor(COLOR_MENU_NORMAL)
        Children [(
            Text("New Scene")
            template(|_| Ok(TextFont::from_font_size(15.0)))
            TextColor(COLOR_LABEL_SECONDARY)
        )]
    }
}

fn entity_lookup_button() -> impl Scene {
    bsn! {
        EntityLookupButton
        Button
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(Val::Px(12.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
        }
        BackgroundColor(COLOR_MENU_NORMAL)
        Children [(
            Text("Entity Lookup")
            template(|_| Ok(TextFont::from_font_size(15.0)))
            TextColor(COLOR_LABEL_SECONDARY)
        )]
    }
}

fn sync_menu_button_colors(
    app_state: Res<State<SidebarState>>,
    mut query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&ComponentButton>,
            Option<&ResourceButton>,
            Option<&NewScene>,
            Option<&EntityLookupButton>,
            Option<&WorkspaceButton>,
        ),
        Or<(
            With<ComponentButton>,
            With<ResourceButton>,
            With<StateButton>,
            With<NewScene>,
            With<EntityLookupButton>,
            With<WorkspaceButton>,
        )>,
    >,
) {
    for (interaction, mut bg, comp, res, new_scene, lookup, workspace) in &mut query {
        let is_active = if comp.is_some() {
            *app_state.get() == SidebarState::EntityFilter
        } else if res.is_some() {
            *app_state.get() == SidebarState::Resource
        } else if new_scene.is_some() {
            *app_state.get() == SidebarState::NewScene
        } else if lookup.is_some() {
            *app_state.get() == SidebarState::EntityLookup
        } else if workspace.is_some() {
            *app_state.get() == SidebarState::Workspace
        } else {
            *app_state.get() == SidebarState::State
        };

        let color = if is_active {
            BackgroundColor(COLOR_MENU_ACTIVE)
        } else {
            match interaction {
                Interaction::Hovered => BackgroundColor(COLOR_MENU_HOVER),
                _ => BackgroundColor(COLOR_MENU_NORMAL),
            }
        };
        bg.set_if_neq(color);
    }
}

fn on_component_button(
    query: Query<&Interaction, (Changed<Interaction>, With<ComponentButton>)>,
    mut next_state: ResMut<NextState<SidebarState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(SidebarState::EntityFilter);
        }
    }
}

fn on_resource_button(
    query: Query<&Interaction, (Changed<Interaction>, With<ResourceButton>)>,
    mut next_state: ResMut<NextState<SidebarState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(SidebarState::Resource);
        }
    }
}

fn on_state_button(
    query: Query<&Interaction, (Changed<Interaction>, With<StateButton>)>,
    mut next_state: ResMut<NextState<SidebarState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(SidebarState::State);
        }
    }
}

fn on_new_scene_button(
    query: Query<&Interaction, (Changed<Interaction>, With<NewScene>)>,
    mut next_state: ResMut<NextState<SidebarState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(SidebarState::NewScene);
        }
    }
}

fn on_entity_lookup_button(
    query: Query<&Interaction, (Changed<Interaction>, With<EntityLookupButton>)>,
    mut next_state: ResMut<NextState<SidebarState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(SidebarState::EntityLookup);
        }
    }
}

fn on_pinboard_button(
    query: Query<&Interaction, (Changed<Interaction>, With<WorkspaceButton>)>,
    mut next_state: ResMut<NextState<SidebarState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(SidebarState::Workspace);
        }
    }
}

fn on_github_button(query: Query<&Interaction, (Changed<Interaction>, With<GithubButton>)>) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            let _ = webbrowser::open("https://github.com/rockcen9/bevy_pin");
        }
    }
}
