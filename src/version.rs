use bevy::{prelude::*, ui::Val::*};

use crate::GAME_VERSION;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_version_button);
}

fn spawn_version_button(mut commands: Commands) {
    commands.spawn((
        Name::new("Version Button"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Px(8.0),
            right: Px(8.0),
            padding: UiRect::axes(Px(12.0), Px(6.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![(
            Text(format!("v{}", GAME_VERSION)),
            TextFont {
                font_size: FontSize::Px(14.0),
                ..default()
            },
            // ash from ColorPalette (#b6b6b4)
            TextColor(Color::srgb(
                0xb6 as f32 / 255.0,
                0xb6 as f32 / 255.0,
                0xb4 as f32 / 255.0
            )),
            Pickable::IGNORE,
        )],
    ));
}
