use super::AppState;
use crate::prelude::*;
use crate::ui::theme::palette::{
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
struct GithubButton;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            sync_menu_button_colors,
            on_component_button,
            on_resource_button,
            on_state_button,
            on_github_button,
        ),
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
            component_button(),
            resource_button(),
            state_button(),
            (
                Node {
                    flex_grow: 1.0,
                }
            ),
        ]
    }
}

fn component_button() -> impl Scene {
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
            Text("Components")
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
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0))
        Children [(
            Text("Resources")
            template(|_| Ok(TextFont::from_font_size(15.0)))
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85))
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
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.0))
        Children [(
            Text("States")
            template(|_| Ok(TextFont::from_font_size(15.0)))
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85))
        )]
    }
}

fn sync_menu_button_colors(
    app_state: Res<State<AppState>>,
    mut query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&ComponentButton>,
            Option<&ResourceButton>,
        ),
        Or<(
            With<ComponentButton>,
            With<ResourceButton>,
            With<StateButton>,
        )>,
    >,
) {
    for (interaction, mut bg, comp, res) in &mut query {
        let is_active = if comp.is_some() {
            *app_state.get() == AppState::Component
        } else if res.is_some() {
            *app_state.get() == AppState::Resource
        } else {
            *app_state.get() == AppState::State
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
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Component);
        }
    }
}

fn on_resource_button(
    query: Query<&Interaction, (Changed<Interaction>, With<ResourceButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Resource);
        }
    }
}

fn on_state_button(
    query: Query<&Interaction, (Changed<Interaction>, With<StateButton>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::State);
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
