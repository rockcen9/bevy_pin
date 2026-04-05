use super::ConnectionState;
use crate::prelude::*;

#[derive(Component, Clone, Default)]
struct DisconnectedOverlay;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (spawn_overlay, despawn_overlay));
}

fn spawn_overlay(
    mut commands: Commands,
    state: Res<State<ConnectionState>>,
    overlay: Query<(), With<DisconnectedOverlay>>,
) {
    if !state.is_changed() {
        return;
    }
    if *state.get() != ConnectionState::Disconnected {
        return;
    }
    if !overlay.is_empty() {
        return;
    }

    commands.spawn_scene(disconnected_overlay());
}

fn despawn_overlay(
    mut commands: Commands,
    state: Res<State<ConnectionState>>,
    overlay: Query<Entity, With<DisconnectedOverlay>>,
) {
    if !state.is_changed() {
        return;
    }
    if *state.get() != ConnectionState::Connected {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
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
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55))
        ZIndex(999)
        DisconnectedOverlay
        Children [(
            Text::new("Reconnecting...")
            template(|_| Ok(TextFont::from_font_size(48.0)))
            TextColor(Color::srgba(0.75, 0.75, 0.75, 1.0))
        )]
    }
}
