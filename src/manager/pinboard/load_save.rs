use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_ACTIVE, COLOR_BUTTON_TEXT, COLOR_HEADER_BG, COLOR_LABEL_SECONDARY, COLOR_OVERLAY_BG,
    COLOR_PANEL_BG, COLOR_SEPARATOR, COLOR_TITLE,
};

use super::pincard::{PinCard, PinCardEntry, pincard, pincard_key};
use super::ui::PinboardContainer;

#[derive(Resource, Serialize, Deserialize, Default)]
pub struct PinboardSaveData {
    pub cards: Vec<PinCardEntry>,
}

#[derive(Component, Clone, Default, Reflect)]
pub struct PinboardPendingData {
    pub entity_id: u64,
    pub key: String,
    pub highlight: bool,
}

#[derive(Resource, Default, Reflect, Clone)]
#[reflect(Resource)]
pub struct PinboardPendingItem(pub Vec<PinboardPendingData>);

#[derive(Resource)]
struct PinboardLoadFailed {
    save_path: PathBuf,
}

#[derive(Component, Clone, Default)]
struct PinboardErrorDialog;

#[derive(Component, Clone, Default)]
struct PinboardResetButton;

pub fn plugin(app: &mut App) {
    #[cfg(target_arch = "wasm32")]
    let save_path = PathBuf::from("local/pinboard_save.json");
    #[cfg(not(target_arch = "wasm32"))]
    let save_path = PathBuf::from("pinboard_save.json");

    match Persistent::<PinboardSaveData>::builder()
        .name("pinboard")
        .format(StorageFormat::Json)
        .path(save_path.clone())
        .default(PinboardSaveData::default())
        .build()
    {
        Ok(persistent) => {
            app.insert_resource(persistent);
        }
        Err(_) => {
            app.insert_resource(PinboardLoadFailed {
                save_path: save_path.clone(),
            });
        }
    }

    app.init_resource::<PinboardPendingItem>()
        .add_systems(
            Startup,
            spawn_load_error_dialog.run_if(resource_exists::<PinboardLoadFailed>),
        )
        .add_systems(PostStartup, load_pinboard_data)
        .add_systems(Update, on_reset_button);
}

fn error_dialog() -> impl Scene {
    bsn! {
        PinboardErrorDialog
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(COLOR_OVERLAY_BG)
        ZIndex(999)
        Children [(
            // --- THE MAIN PANEL ---
            Node {
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(Val::Px(12.0)),
                border: UiRect::all(Val::Px(1.0)),
                // THE FIX: Remove min/max width and use a fixed width.
                // This forces Taffy to calculate the text wrap height correctly.
                width: Val::Px(480.0),
            }
            BackgroundColor(COLOR_PANEL_BG)
            template(|_| Ok(BorderColor::all(COLOR_SEPARATOR)))
            Children [
                // SIBLING 1: THE TITLE/HEADER
                (
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(16.0)),
                        border_radius: BorderRadius::top(Val::Px(12.0)),
                    }
                    BackgroundColor(COLOR_HEADER_BG)
                    Children [(
                        Text::new("Save Format Mismatch")
                        template(|_| Ok(TextFont::from_font_size(17.0)))
                        TextColor(COLOR_TITLE)
                    )]
                ),

                // SIBLING 2: THE BODY CONTAINER
                (
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(24.0)),
                        // Because the width is fixed, row_gap will actually work now.
                        row_gap: Val::Px(24.0),
                    }
                    Children [
                        // INNER CHILD A: TEXT BODY
                        (
                            Text::new("The existing save file format is incompatible with this version. Resetting will restore your pinboard to its default state. Would you like to proceed?")
                            template(|_| Ok(TextFont::from_font_size(14.0)))
                            TextColor(COLOR_LABEL_SECONDARY)
                        ),

                        // INNER CHILD B: BUTTON CONTAINER
                        (
                            Node {
                                // Just align this wrapper to the right
                                align_self: AlignSelf::FlexEnd,
                            }
                            Children [(
                                PinboardResetButton
                                Button
                                Node {
                                    padding: UiRect::axes(Val::Px(20.0), Val::Px(9.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border_radius: BorderRadius::all(Val::Px(6.0)),
                                }
                                BackgroundColor(COLOR_ACTIVE)
                                Children [(
                                    Text::new("Reset to Default")
                                    template(|_| Ok(TextFont::from_font_size(13.0)))
                                    TextColor(COLOR_BUTTON_TEXT)
                                )]
                            )]
                        ),
                    ]
                ),
            ]
        )]
    }
}
fn spawn_load_error_dialog(mut commands: Commands) {
    commands.spawn_scene(error_dialog());
}

fn on_reset_button(
    query: Query<&Interaction, (Changed<Interaction>, With<PinboardResetButton>)>,
    dialog: Query<Entity, With<PinboardErrorDialog>>,
    load_failed: Option<Res<PinboardLoadFailed>>,
    mut commands: Commands,
) {
    let Some(load_failed) = load_failed else {
        return;
    };

    for interaction in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Ok(entity) = dialog.single() {
            commands.entity(entity).despawn();
        }

        let save_path = load_failed.save_path.clone();
        #[cfg(not(target_arch = "wasm32"))]
        let _ = std::fs::remove_file(&save_path);

        match Persistent::<PinboardSaveData>::builder()
            .name("pinboard")
            .format(StorageFormat::Json)
            .path(save_path)
            .default(PinboardSaveData::default())
            .build()
        {
            Ok(persistent) => {
                commands.insert_resource(persistent);
            }
            Err(e) => {
                error!("on_reset_button: failed to initialize fresh save data: {e}");
            }
        }

        commands.remove_resource::<PinboardLoadFailed>();
    }
}

fn load_pinboard_data(
    save_data: Option<Res<Persistent<PinboardSaveData>>>,
    pinboard: Query<Entity, With<PinboardContainer>>,
    mut pending: ResMut<PinboardPendingItem>,
    mut commands: Commands,
) {
    let Some(save_data) = save_data else {
        return;
    };
    if save_data.cards.is_empty() {
        return;
    }
    let Ok(pinboard_entity) = pinboard.single() else {
        return;
    };
    for entry in &save_data.cards {
        let key = pincard_key(entry.entity_id);
        let panel = commands
            .spawn_scene(pincard(
                entry.label.clone(),
                entry.entity_id,
                entry.left,
                entry.top,
                entry.width,
                entry.height,
            ))
            .id();
        commands.entity(panel).insert(PinCard {
            entity_id: entry.entity_id,
        });
        commands.entity(pinboard_entity).add_child(panel);
        pending.0.push(PinboardPendingData {
            entity_id: entry.entity_id,
            key,
            highlight: false,
        });
    }
}
