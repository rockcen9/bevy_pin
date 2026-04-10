use bevy::input_focus::{InputFocus, tab_navigation::TabIndex};
use bevy::text::{EditableText, TextCursorStyle};

use crate::manager::connection::ServerUrl;
use crate::manager::entity_filter::component_list::InspectedEntity;
use crate::manager::entity_lookup::history::LookupHistory;
use crate::prelude::*;

#[derive(Component)]
struct NameFetchCtx {
    entity_id: u64,
    name_type_path: String,
}
use crate::ui_layout::theme::palette::{
    COLOR_BUTTON_BG, COLOR_BUTTON_HOVER, COLOR_HEADER_BG, COLOR_INPUT_BG, COLOR_INPUT_BORDER,
    COLOR_INPUT_TEXT, COLOR_PANEL_BG, COLOR_TITLE,
};
use crate::ui_layout::theme::widgets::show_global_message;

#[derive(Component, Clone, Default)]
#[require(DespawnOnExit::<SidebarState>(SidebarState::EntityLookup))]
pub struct LookupPanel;

#[derive(Component, Clone, Default)]
struct LookupInput;

#[derive(Component, Clone, Default)]
struct LookupButton;

#[derive(Component)]
struct LookupQueryCtx {
    display_index: u32,
}

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (submit_on_enter, handle_lookup_button, update_button_hover),
    );
}

pub fn lookup_panel() -> impl Scene {
    bsn! {
        #LookupPanel
        LookupPanel
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
                    Text::new("Entity Lookup")
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
                        LookupInput
                        template(|_| {
                            let mut text_input = EditableText {
                                max_characters: Some(12),
                                ..default()
                            };
                            text_input.editor_mut().set_text("");
                            Ok(text_input)
                        })
                        template(|_| Ok(TextFont { font_size: FontSize::Px(13.0), ..default() }))
                        TextColor(COLOR_INPUT_TEXT)
                        TextCursorStyle::default()
                        TabIndex(1)
                    ),
                    (
                        Button
                        LookupButton
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            justify_content: JustifyContent::Center,
                        }
                        BackgroundColor(COLOR_BUTTON_BG)
                        Children [(
                            Text::new("Find")
                            template(|_| Ok(TextFont::from_font_size(13.0)))
                            TextColor(COLOR_INPUT_TEXT)
                        )]
                    ),
                ]
            ),
        ]
    }
}

fn submit_on_enter(
    input_focus: Res<InputFocus>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inputs: Query<&mut EditableText, With<LookupInput>>,
    server_url: Res<ServerUrl>,
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
    match raw.parse::<u32>() {
        Ok(n) => {
            send_lookup(n, &server_url.0, &mut commands);
            text_input.editor_mut().set_text("");
        }
        Err(_) => {
            show_global_message(format!("'{}' is not a valid number", raw), &mut commands);
        }
    }
}

fn handle_lookup_button(
    buttons: Query<&Interaction, (Changed<Interaction>, With<LookupButton>)>,
    mut inputs: Query<&mut EditableText, With<LookupInput>>,
    server_url: Res<ServerUrl>,
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
        match raw.parse::<u32>() {
            Ok(n) => {
                send_lookup(n, &server_url.0, &mut commands);
                text_input.editor_mut().set_text("");
            }
            Err(_) => {
                show_global_message(format!("'{}' is not a valid number", raw), &mut commands);
            }
        }
    }
}

fn update_button_hover(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<LookupButton>),
    >,
) {
    for (interaction, mut color) in &mut buttons {
        color.set_if_neq(BackgroundColor(match interaction {
            Interaction::Hovered => COLOR_BUTTON_HOVER,
            _ => COLOR_BUTTON_BG,
        }));
    }
}

fn send_lookup(display_index: u32, url: &str, commands: &mut Commands) {
    debug!("EntityLookup: searching for display index {}", display_index);
    let req = commands.brp_world_query(
        url,
        json!({
            "data": { "components": [], "option": "all", "has": [] },
            "filter": { "with": [], "without": [] },
            "strict": false
        }),
    );
    commands
        .entity(req)
        .insert(LookupQueryCtx { display_index })
        .observe(
            |trigger: On<Add, RpcResponse<BrpWorldQuery>>,
             q: Query<(&RpcResponse<BrpWorldQuery>, &LookupQueryCtx)>,
             mut inspected: ResMut<InspectedEntity>,
             mut history: ResMut<LookupHistory>,
             server_url: Res<ServerUrl>,
             mut commands: Commands| {
                let ecs_entity = trigger.entity;
                let Ok((response, ctx)) = q.get(ecs_entity) else {
                    commands.entity(ecs_entity).despawn();
                    return;
                };
                let target_label = format!("v{}", ctx.display_index);
                if let Ok(data) = &response.data {
                    let found = data
                        .result
                        .iter()
                        .find(|entry| crate::utils::entity_display_label(entry.entity) == target_label);
                    match found {
                        Some(entry) => {
                            debug!(
                                "EntityLookup: found entity #{} for '{}'",
                                entry.entity, target_label
                            );
                            let entity_id = entry.entity;
                            let name_type_path = entry
                                .components
                                .as_object()
                                .and_then(|m| {
                                    m.keys()
                                        .find(|k| k.split("::").last().unwrap_or("") == "Name")
                                })
                                .map(|s| s.to_string());

                            inspected.0 = Some(entity_id);
                            history.push(entity_id);

                            if let Some(name_type_path) = name_type_path {
                                let req = commands.brp_get_components(
                                    &server_url.0,
                                    entity_id,
                                    &[name_type_path.clone()],
                                    false,
                                );
                                commands
                                    .entity(req)
                                    .insert(NameFetchCtx { entity_id, name_type_path })
                                    .observe(
                                        |trigger: On<Add, RpcResponse<BrpGetComponents>>,
                                         q: Query<(&RpcResponse<BrpGetComponents>, &NameFetchCtx)>,
                                         mut history: ResMut<LookupHistory>,
                                         mut commands: Commands| {
                                            let ecs_entity = trigger.entity;
                                            let Ok((response, ctx)) = q.get(ecs_entity) else {
                                                commands.entity(ecs_entity).despawn();
                                                return;
                                            };
                                            if let Ok(data) = &response.data {
                                                let value = data.result["components"]
                                                    .as_object()
                                                    .and_then(|m| m.get(&ctx.name_type_path));
                                                let name = value.and_then(|v| match v {
                                                    serde_json::Value::String(s) => Some(s.clone()),
                                                    v => v
                                                        .get("name")
                                                        .and_then(|n| n.as_str())
                                                        .map(|s| s.to_string()),
                                                });
                                                if let Some(name) = name {
                                                    debug!(
                                                        "EntityLookup: name for #{} = '{}'",
                                                        ctx.entity_id, name
                                                    );
                                                    history.update_name(ctx.entity_id, name);
                                                }
                                            }
                                            commands.entity(ecs_entity).despawn();
                                        },
                                    )
                                    .observe(
                                        |trigger: On<Add, TimeoutError>, mut commands: Commands| {
                                            commands.entity(trigger.entity).despawn();
                                        },
                                    );
                            }
                        }
                        None => {
                            debug!("EntityLookup: no entity found for '{}'", target_label);
                            show_global_message(
                                format!("No entity found for '{}'", target_label),
                                &mut commands,
                            );
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
