use bevy::{
    input_focus::{InputFocus, tab_navigation::TabIndex},
    text::{EditableText, TextCursorStyle},
};

use crate::manager::connection::ServerUrl;
use crate::manager::entity_filter::component_list::list::InspectedEntity;
use crate::manager::entity_filter::entity_list::ui::{ComponentEntityRow, SelectedRow};
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_BUTTON_BG, COLOR_BUTTON_HOVER, COLOR_HEADER_BG, COLOR_INPUT_BG, COLOR_INPUT_BORDER,
    COLOR_INPUT_TEXT, COLOR_LABEL_SECONDARY, COLOR_PANEL_BG, COLOR_SEPARATOR, COLOR_TITLE,
};
use crate::ui_layout::theme::widgets::show_global_message;

#[derive(Component, Clone, Default)]
#[require(DespawnOnExit::<SidebarState>(SidebarState::EntityFilter))]
pub struct InsertComponentPanel;

#[derive(Component, Clone, Default)]
struct InsertComponentInput;

#[derive(Component, Clone, Default)]
struct InsertComponentButton;

/// short_name -> full_type_path, populated from registry.schema on panel spawn.
#[derive(Resource, Default)]
pub struct KnownMarkerComponents(pub HashMap<String, String>);

pub fn plugin(app: &mut App) {
    app.init_resource::<KnownMarkerComponents>()
        .add_observer(on_panel_spawn)
        .add_systems(
            Update,
            (
                submit_on_enter,
                handle_insert_button,
                update_button_hover,
            ),
        );
}

fn on_panel_spawn(
    _trigger: On<Add, InsertComponentPanel>,
    mut commands: Commands,
    server_url: Res<ServerUrl>,
) {
    let req = commands.brp_registry_schema(&server_url.0, json!({}));
    commands
        .entity(req)
        .observe(
            |trigger: On<Add, RpcResponse<BrpSchema>>,
             query: Query<&RpcResponse<BrpSchema>>,
             mut known: ResMut<KnownMarkerComponents>,
             mut commands: Commands| {
                let entity = trigger.entity;
                let Ok(response) = query.get(entity) else {
                    commands.entity(entity).despawn();
                    return;
                };
                if let Ok(data) = &response.data {
                    if let Some(obj) = data.result.as_object() {
                        for (type_path, schema) in obj {
                            let is_marker = schema
                                .get("properties")
                                .and_then(|p| p.as_object())
                                .map(|p| p.is_empty())
                                .unwrap_or(true);
                            if !is_marker {
                                continue;
                            }
                            let short = type_path
                                .split("::")
                                .last()
                                .unwrap_or(type_path)
                                .to_string();
                            known.0.entry(short).or_insert_with(|| type_path.clone());
                            known.0.entry(type_path.clone()).or_insert_with(|| type_path.clone());
                        }
                        debug!(
                            "InsertComponentPanel: {} marker types loaded",
                            known.0.len() / 2
                        );
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            commands.entity(trigger.entity).despawn();
        });
}

pub fn insert_component_panel() -> impl Scene {
    bsn! {
        #InsertComponentPanel
        InsertComponentPanel
        Node {
            flex_direction: FlexDirection::Column,
            min_width: Val::Px(280.0),
            max_width: Val::Px(280.0),
            border_radius: BorderRadius::all(Val::Px(10.0)),
        }
        BackgroundColor(COLOR_PANEL_BG)
        Children [
            (
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    border_radius: BorderRadius::top(Val::Px(10.0)),
                }
                BackgroundColor(COLOR_HEADER_BG)
                Children [(
                    Text::new("Insert Component")
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
                            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                        }
                        BorderColor::all(COLOR_INPUT_BORDER)
                        BackgroundColor(COLOR_INPUT_BG)
                        InsertComponentInput
                        template(|_| {
                            let mut text_input = EditableText {
                                max_characters: Some(1024),
                                ..default()
                            };
                            text_input.editor_mut().set_text("");
                            Ok(text_input)
                        })
                        template(|_| {
                            Ok(TextFont {
                                font_size: FontSize::Px(13.0),
                                ..default()
                            })
                        })
                        TextColor(COLOR_INPUT_TEXT)
                        TextCursorStyle::default()
                        TabIndex(2)
                    ),
                    (
                        Button
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                        }
                        BackgroundColor(COLOR_BUTTON_BG)
                        InsertComponentButton
                        Children [(
                            Text::new("Insert")
                            template(|_| Ok(TextFont::from_font_size(13.0)))
                            TextColor(COLOR_INPUT_TEXT)
                        )]
                    ),
                ]
            ),
            (
                Node {
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    margin: UiRect::all(Val::Px(10.0)),
                }
                BackgroundColor(COLOR_INPUT_BG)
                BorderColor::all(COLOR_SEPARATOR)
                Children [(
                    Text::new("Marker Component supported only")
                    template(|_| Ok(TextFont::from_font_size(11.0)))
                    TextColor(COLOR_LABEL_SECONDARY)
                )]
            ),
        ]
    }
}

fn submit_on_enter(
    input_focus: Res<InputFocus>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inputs: Query<&mut EditableText, With<InsertComponentInput>>,
    server_url: Res<ServerUrl>,
    known: Res<KnownMarkerComponents>,
    selected_row: Query<&ComponentEntityRow, With<SelectedRow>>,
    inspected: Res<InspectedEntity>,
    mut commands: Commands,
) {
    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }
    let Some(focused) = input_focus.get() else {
        return;
    };
    let Ok(mut text_input) = inputs.get_mut(focused) else {
        return;
    };

    let raw = text_input.value().to_string();
    let raw = raw.trim().to_string();
    if raw.is_empty() {
        return;
    }

    let entity_id = selected_row.single().ok().map(|r| r.entity).or(inspected.0);
    let Some(entity_id) = entity_id else {
        show_global_message("No entity selected".to_string(), &mut commands);
        return;
    };

    let Some(type_path) = resolve_type_path(&raw, &known) else {
        show_global_message(
            format!("'{}' is not a marker component", raw),
            &mut commands,
        );
        return;
    };
    insert_component(entity_id, type_path, &server_url.0, &mut commands);
    text_input.editor_mut().set_text("");
}

fn handle_insert_button(
    buttons: Query<&Interaction, (Changed<Interaction>, With<InsertComponentButton>)>,
    mut inputs: Query<&mut EditableText, With<InsertComponentInput>>,
    server_url: Res<ServerUrl>,
    known: Res<KnownMarkerComponents>,
    selected_row: Query<&ComponentEntityRow, With<SelectedRow>>,
    inspected: Res<InspectedEntity>,
    mut commands: Commands,
) {
    for interaction in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Ok(mut text_input) = inputs.single_mut() else {
            continue;
        };

        let raw = text_input.value().to_string();
        let raw = raw.trim().to_string();
        if raw.is_empty() {
            continue;
        }

        let entity_id = selected_row.single().ok().map(|r| r.entity).or(inspected.0);
        let Some(entity_id) = entity_id else {
            show_global_message("No entity selected".to_string(), &mut commands);
            continue;
        };

        let Some(type_path) = resolve_type_path(&raw, &known) else {
            show_global_message(
                format!("'{}' is not a marker component", raw),
                &mut commands,
            );
            continue;
        };
        insert_component(entity_id, type_path, &server_url.0, &mut commands);
        text_input.editor_mut().set_text("");
    }
}

fn update_button_hover(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<InsertComponentButton>),
    >,
) {
    for (interaction, mut color) in &mut buttons {
        color.set_if_neq(BackgroundColor(match interaction {
            Interaction::Hovered => COLOR_BUTTON_HOVER,
            _ => COLOR_BUTTON_BG,
        }));
    }
}

fn resolve_type_path(input: &str, known: &KnownMarkerComponents) -> Option<String> {
    known.0.get(input).cloned()
}

fn insert_component(entity_id: u64, type_path: String, url: &str, commands: &mut Commands) {
    let components = json!({ type_path.clone(): {} });
    let req = commands.brp_insert_components(url, entity_id, components);
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
                            let short =
                                type_path.split("::").last().unwrap_or(&type_path).to_string();
                            info!(
                                "insert_component: '{}' added to entity #{}",
                                short, entity_id
                            );
                        }
                        Err(e) => {
                            error!("insert_component failed: {}", e);
                            show_global_message("Insert failed — check logs", &mut commands);
                        }
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            commands.entity(trigger.entity).despawn();
        });
}
