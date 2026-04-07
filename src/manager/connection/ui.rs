use super::ConnectionState;
use crate::prelude::*;
use crate::ui_layout::theme::palette::{COLOR_OVERLAY_BG, COLOR_OVERLAY_TEXT};

#[derive(Component, Clone, Default)]
struct DisconnectedOverlay;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, sync_overlay);
}

fn sync_overlay(
    mut commands: Commands,
    state: Res<State<ConnectionState>>,
    overlay: Query<Entity, With<DisconnectedOverlay>>,
) {
    let should_show = *state.get() == ConnectionState::Disconnected;
    let is_shown = !overlay.is_empty();

    match (should_show, is_shown) {
        (true, false) => {
            commands.spawn_scene(disconnected_overlay());
        }
        (false, true) => {
            for entity in &overlay {
                commands.entity(entity).despawn();
            }
        }
        _ => {}
    }
}

fn disconnected_overlay() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(1920.0),
            height: Val::Px(1080.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(COLOR_OVERLAY_BG)
        ZIndex(999)
        DisconnectedOverlay
        Children [(
            Text::new("Reconnecting...")
            template(|_| Ok(TextFont::from_font_size(48.0)))
            TextColor(COLOR_OVERLAY_TEXT)
        )]
    }
}
