use bevy::{
    input_focus::{InputFocus, tab_navigation::TabIndex},
    text::{EditableText, FontCx, LayoutCx, TextCursorStyle},
};

use super::{ComponentQueries, QueryEntry};
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_BUTTON_BG, COLOR_BUTTON_HOVER, COLOR_HEADER_BG, COLOR_HINT_BG, COLOR_INPUT_BG,
    COLOR_INPUT_BORDER, COLOR_INPUT_TEXT, COLOR_PANEL_BG, COLOR_SEPARATOR, COLOR_SYNTAX_WITH,
    COLOR_SYNTAX_WITHOUT, COLOR_TITLE,
};

#[derive(Component, Clone, Default)]
struct QueryPanelRoot;

#[derive(Component, Clone, Default)]
struct QueryInput;

#[derive(Component, Clone, Default)]
struct AddQueryButton;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (submit_on_enter, handle_add_button, update_button_hover),
    );
}

pub fn query_panel() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            min_width: Val::Px(320.0),
            border_radius: BorderRadius::all(Val::Px(10.0)),
        }
        BackgroundColor(COLOR_PANEL_BG)
        QueryPanelRoot
        Children [
            (
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    border_radius: BorderRadius::top(Val::Px(10.0)),
                }
                BackgroundColor(COLOR_HEADER_BG)
                Children [(
                    Text::new("Component Query")
                    template(|_| Ok(TextFont::from_font_size(18.0)))
                    TextColor(COLOR_TITLE)
                )]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    align_items: AlignItems::Center,
                }
                Children [
                    (
                        Node {
                            flex_grow: 1.0,
                            border: UiRect::all(Val::Px(1.0)),
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BorderColor::all(COLOR_INPUT_BORDER)
                        BackgroundColor(COLOR_INPUT_BG)
                        QueryInput
                        template(|_| {
                            Ok(EditableText {
                                max_characters: Some(256),
                                ..default()
                            })
                        })
                        template(|_| {
                            Ok(TextFont {
                                font_size: FontSize::Px(13.0),
                                ..default()
                            })
                        })
                        TextColor(COLOR_INPUT_TEXT)
                        TextCursorStyle::default()
                        TabIndex(1)
                    ),
                    (
                        Button
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                        }
                        BackgroundColor(COLOR_BUTTON_BG)
                        AddQueryButton
                        Children [(
                            Text::new("Add")
                            template(|_| Ok(TextFont::from_font_size(13.0)))
                            TextColor(COLOR_INPUT_TEXT)
                        )]
                    ),
                ]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(10.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    margin: UiRect::axes(Val::Px(10.0), Val::Px(0.0)),
                }
                BackgroundColor(COLOR_HINT_BG)
                BorderColor::all(COLOR_SEPARATOR)
                Children [
                    (
                        Text::new("Query filters")
                        template(|_| Ok(TextFont::from_font_size(11.0)))
                        TextColor(COLOR_TITLE)
                    ),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(0.0),
                        }
                        Children [
                            (
                                Text::new("With<C>")
                                template(|_| Ok(TextFont::from_font_size(12.0)))
                                TextColor(COLOR_SYNTAX_WITH)
                            ),
                            (
                                Text::new(" = ")
                                template(|_| Ok(TextFont::from_font_size(12.0)))
                                TextColor(COLOR_TITLE)
                            ),
                            (
                                Text::new("C")
                                template(|_| Ok(TextFont::from_font_size(12.0)))
                                TextColor(COLOR_SYNTAX_WITH)
                            ),
                            (
                                Text::new("  |  ")
                                template(|_| Ok(TextFont::from_font_size(12.0)))
                                TextColor(COLOR_TITLE)
                            ),
                            (
                                Text::new("Without<C>")
                                template(|_| Ok(TextFont::from_font_size(12.0)))
                                TextColor(COLOR_SYNTAX_WITHOUT)
                            ),
                            (
                                Text::new(" = ")
                                template(|_| Ok(TextFont::from_font_size(12.0)))
                                TextColor(COLOR_TITLE)
                            ),
                            (
                                Text::new("!C")
                                template(|_| Ok(TextFont::from_font_size(12.0)))
                                TextColor(COLOR_SYNTAX_WITHOUT)
                            ),
                        ]
                    ),
                ]
            ),
        ]
    }
}

fn submit_on_enter(
    input_focus: Res<InputFocus>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query_inputs: Query<&mut EditableText, With<QueryInput>>,
    mut queries: ResMut<ComponentQueries>,
    mut font_cx: ResMut<FontCx>,
    mut layout_cx: ResMut<LayoutCx>,
) {
    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }
    let Some(focused) = input_focus.get() else {
        return;
    };
    let Ok(mut text_input) = query_inputs.get_mut(focused) else {
        return;
    };

    let value = text_input.value().to_string();
    let value = value.trim().to_string();
    if value.is_empty() {
        return;
    }

    queries.insert(QueryEntry::new(value));
    text_input.clear(&mut font_cx.0, &mut layout_cx.0);
}

fn handle_add_button(
    buttons: Query<&Interaction, (Changed<Interaction>, With<AddQueryButton>)>,
    mut query_inputs: Query<&mut EditableText, With<QueryInput>>,
    mut queries: ResMut<ComponentQueries>,
    mut font_cx: ResMut<FontCx>,
    mut layout_cx: ResMut<LayoutCx>,
) {
    for interaction in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Ok(mut text_input) = query_inputs.single_mut() else {
            continue;
        };

        let value = text_input.value().to_string();
        let value = value.trim().to_string();
        if value.is_empty() {
            continue;
        }

        queries.insert(QueryEntry::new(value));
        text_input.clear(&mut font_cx.0, &mut layout_cx.0);
    }
}

fn update_button_hover(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<AddQueryButton>),
    >,
) {
    for (interaction, mut color) in &mut buttons {
        let new_color = match interaction {
            Interaction::Hovered => COLOR_BUTTON_HOVER,
            _ => COLOR_BUTTON_BG,
        };
        color.set_if_neq(BackgroundColor(new_color));
    }
}
