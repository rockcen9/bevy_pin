use bevy::{
    input_focus::{InputFocus, tab_navigation::TabIndex},
    text::{EditableText, TextCursorStyle},
};

use crate::manager::new_scene::history::{SpawnEntry, SpawnedEntities};
use crate::manager::{connection::ServerUrl, new_scene::SpawnedEntityId};
use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_BUTTON_BG, COLOR_BUTTON_HOVER, COLOR_HEADER_BG, COLOR_HINT_BG, COLOR_INPUT_BG,
    COLOR_INPUT_BORDER, COLOR_INPUT_TEXT, COLOR_LABEL_SECONDARY, COLOR_PANEL_BG, COLOR_SEPARATOR,
    COLOR_TITLE,
};
use crate::ui_layout::theme::widgets::show_global_message;

#[derive(Component, Clone, Default)]
#[require(DespawnOnExit::<SidebarState>(SidebarState::NewScene))]
pub struct SpawnEntityPanel;

#[derive(Component, Clone, Default)]
struct SpawnEntityInput;

#[derive(Component, Clone, Default)]
struct SpawnEntityButton;

/// short_name -> full_type_path, populated from registry.schema on panel open.
#[derive(Resource, Default)]
struct KnownComponents(HashMap<String, String>);

pub fn plugin(app: &mut App) {
    app.init_resource::<KnownComponents>()
        .add_observer(on_panel_spawn);
    app.add_systems(
        Update,
        (submit_on_enter, handle_spawn_button, update_button_hover),
    );
}

fn on_panel_spawn(
    _trigger: On<Add, SpawnEntityPanel>,
    mut commands: Commands,
    server_url: Res<ServerUrl>,
) {
    let req = commands.brp_registry_schema(&server_url.0, json!({}));
    commands
        .entity(req)
        .observe(
            |trigger: On<Add, RpcResponse<BrpSchema>>,
             query: Query<&RpcResponse<BrpSchema>>,
             mut known: ResMut<KnownComponents>,
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
                            known
                                .0
                                .entry(type_path.clone())
                                .or_insert_with(|| type_path.clone());
                        }
                        debug!("KnownComponents: {} marker types loaded", known.0.len() / 2);
                    }
                }
                commands.entity(entity).despawn();
            },
        )
        .observe(|trigger: On<Add, TimeoutError>, mut commands: Commands| {
            commands.entity(trigger.entity).despawn();
        });
}

pub fn spawn_entity_panel() -> impl Scene {
    bsn! {
        #SpawnEntityPanel
        SpawnEntityPanel
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
                    Text::new("New Scene")
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
                        SpawnEntityInput
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
                        SpawnEntityButton
                        Children [(
                            Text::new("Spawn")
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
                BackgroundColor(COLOR_HINT_BG)
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
    mut inputs: Query<&mut EditableText, With<SpawnEntityInput>>,
    server_url: Res<ServerUrl>,
    known: Res<KnownComponents>,
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

    let Some(type_path) = resolve_type_path(&raw, &known) else {
        show_global_message(
            format!("'{}' is not a marker component", raw),
            &mut commands,
        );
        return;
    };
    spawn_entity(type_path, &server_url.0, &mut commands);
    text_input.editor_mut().set_text("");
}

fn handle_spawn_button(
    buttons: Query<&Interaction, (Changed<Interaction>, With<SpawnEntityButton>)>,
    mut inputs: Query<&mut EditableText, With<SpawnEntityInput>>,
    server_url: Res<ServerUrl>,
    known: Res<KnownComponents>,
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

        let Some(type_path) = resolve_type_path(&raw, &known) else {
            show_global_message(
                format!("'{}' is not a marker component", raw),
                &mut commands,
            );
            continue;
        };
        spawn_entity(type_path, &server_url.0, &mut commands);
        text_input.editor_mut().set_text("");
    }
}

fn update_button_hover(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SpawnEntityButton>),
    >,
) {
    for (interaction, mut color) in &mut buttons {
        color.set_if_neq(BackgroundColor(match interaction {
            Interaction::Hovered => COLOR_BUTTON_HOVER,
            _ => COLOR_BUTTON_BG,
        }));
    }
}

/// Resolves the input to a full type path only if it is a known marker component.
fn resolve_type_path(input: &str, known: &KnownComponents) -> Option<String> {
    known.0.get(input).cloned()
}

fn spawn_entity(type_path: String, url: &str, commands: &mut Commands) {
    let components = json!({ type_path.clone(): {} });
    let req = commands.brp_spawn_entity(url, components);
    commands
        .entity(req)
        .observe(
            move |trigger: On<Add, RpcResponse<BrpSpawnEntity>>,
                  query: Query<&RpcResponse<BrpSpawnEntity>>,
                  mut spawned: ResMut<SpawnedEntities>,
                  mut selected: ResMut<SpawnedEntityId>,
                  mut commands: Commands| {
                let entity = trigger.entity;
                if let Ok(response) = query.get(entity) {
                    match &response.data {
                        Ok(data) => {
                            let short = type_path
                                .split("::")
                                .last()
                                .unwrap_or(&type_path)
                                .to_string();
                            info!("spawn_entity: #{} ({})", data.result.entity, short);
                            spawned.0.push(SpawnEntry {
                                type_name: short,
                                entity_id: data.result.entity,
                            });
                            selected.0 = Some(data.result.entity);
                        }
                        Err(e) => {
                            error!("spawn_entity failed: {}", e);
                            show_global_message("Spawn failed — check logs", &mut commands);
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
