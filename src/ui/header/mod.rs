use crate::prelude::*;
use crate::ui::theme::palette::{
    COLOR_BG_SURFACE, COLOR_LABEL, COLOR_PAUSED, COLOR_PAUSED_HOVER, COLOR_RUNNING,
    COLOR_RUNNING_HOVER, COLOR_SEPARATOR,
};

#[derive(Component, Default, Clone)]
pub struct HeadPanel;

#[derive(Component, Default, Clone)]
pub struct AutoButton;

#[derive(Component, Default, Clone)]
struct PauseButtonLabel;

#[derive(Component, Default, Clone)]
struct GitHubButton;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            on_auto_button_hover,
            on_pause_button_pressed,
            update_pause_button_label,
            update_pause_button_color,
            on_github_button_pressed,
        ),
    );
}

fn on_auto_button_hover(
    pause: Res<State<Pause>>,
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<AutoButton>),
    >,
) {
    let (base, hovered) = if pause.0 {
        (COLOR_PAUSED, COLOR_PAUSED_HOVER)
    } else {
        (COLOR_RUNNING, COLOR_RUNNING_HOVER)
    };
    for (interaction, mut bg) in &mut query {
        match *interaction {
            Interaction::Hovered => {
                bg.set_if_neq(BackgroundColor(hovered));
            }
            Interaction::None => {
                bg.set_if_neq(BackgroundColor(base));
            }
            Interaction::Pressed => {}
        }
    }
}

fn update_pause_button_color(
    pause: Res<State<Pause>>,
    mut query: Query<&mut BackgroundColor, With<AutoButton>>,
) {
    if !pause.is_changed() {
        return;
    }
    let color = if pause.0 { COLOR_PAUSED } else { COLOR_RUNNING };
    for mut bg in &mut query {
        bg.set_if_neq(BackgroundColor(color));
    }
}

fn on_pause_button_pressed(
    query: Query<&Interaction, (Changed<Interaction>, With<AutoButton>)>,
    pause: Res<State<Pause>>,
    mut next_pause: ResMut<NextState<Pause>>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_pause.set(Pause(!pause.0));
        }
    }
}

fn on_github_button_pressed(
    query: Query<&Interaction, (Changed<Interaction>, With<GitHubButton>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            let _ = webbrowser::open("https://github.com/rockcen9/bevy_pin");
        }
    }
}

fn update_pause_button_label(
    pause: Res<State<Pause>>,
    mut query: Query<&mut Text, With<PauseButtonLabel>>,
) {
    if !pause.is_changed() {
        return;
    }
    for mut text in &mut query {
        text.0 = if pause.0 {
            "> Resume".to_string()
        } else {
            "|| Pause".to_string()
        };
    }
}

pub fn head_panel() -> impl Scene {
    bsn! {
        HeadPanel
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            padding: UiRect::horizontal(Val::Px(16.0)),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            border: UiRect::bottom(Val::Px(1.0)),
        }
        BackgroundColor(COLOR_BG_SURFACE)
        BorderColor::all(COLOR_SEPARATOR)
        Children [
            pause_button(),
            github_button()
        ]
    }
}

fn github_button() -> impl Scene {
    bsn! {
        GitHubButton
        Button
        Node {
            width: Val::Px(30.0),
            height: Val::Px(30.0),
            margin: UiRect::left(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(4.0)),
        }
        Children [(
            Node {
                width: Val::Px(28.0),
                height: Val::Px(28.0),
            }
            template(|ctx| Ok(ImageNode::new(ctx.resource::<AssetServer>().load("github_logo.png"))))
        )]
    }
}

fn pause_button() -> impl Scene {
    bsn! {
        AutoButton
        Button
        Node {
            height: Val::Px(30.0),
            padding: UiRect::axes(Val::Px(14.0), Val::Px(0.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(6.0)),
        }
        BackgroundColor(COLOR_RUNNING)
        Children [(
            PauseButtonLabel
            Text("|| Pause")
            template(|_| Ok(TextFont::from_font_size(13.0)))
            TextColor(COLOR_LABEL)
        )]
    }
}
