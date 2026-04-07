use crate::prelude::*;
use crate::ui_layout::theme::palette::{COLOR_HEADER_BG, COLOR_INPUT_TEXT};

const FADE_IN: f32 = 2.0;
const HOLD: f32 = 1.5;
const FADE_OUT: f32 = 0.8;

#[derive(Component)]
pub struct GlobalMessage {
    elapsed: f32,
}

#[derive(Component)]
struct GlobalMessageText;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, tick_global_messages);
}

/// Spawn a toast message that fades in over 2 s, holds, then fades out and despawns.
pub fn show_global_message(text: impl Into<String>, commands: &mut Commands) {
    let text = text.into();
    commands
        .spawn((
            GlobalMessage { elapsed: 0.0 },
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                left: Val::Percent(50.0),
                padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(COLOR_HEADER_BG.with_alpha(0.0)),
            ZIndex(999),
        ))
        .with_children(|parent| {
            parent.spawn((
                GlobalMessageText,
                Text::new(text),
                TextFont::from_font_size(14.0),
                TextColor(COLOR_INPUT_TEXT.with_alpha(0.0)),
            ));
        });
}

fn tick_global_messages(
    mut messages: Query<(Entity, &mut GlobalMessage, &mut BackgroundColor, &Children)>,
    mut texts: Query<&mut TextColor, With<GlobalMessageText>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let total = FADE_IN + HOLD + FADE_OUT;

    for (entity, mut msg, mut bg, children) in &mut messages {
        msg.elapsed += time.delta_secs();

        let alpha = if msg.elapsed < FADE_IN {
            msg.elapsed / FADE_IN
        } else if msg.elapsed < FADE_IN + HOLD {
            1.0
        } else {
            let fade_elapsed = msg.elapsed - FADE_IN - HOLD;
            1.0 - (fade_elapsed / FADE_OUT).min(1.0)
        };

        bg.0 = COLOR_HEADER_BG.with_alpha(alpha * COLOR_HEADER_BG.alpha());

        for &child in children {
            if let Ok(mut text_color) = texts.get_mut(child) {
                text_color.0 = COLOR_INPUT_TEXT.with_alpha(alpha * COLOR_INPUT_TEXT.alpha());
            }
        }

        if msg.elapsed >= total {
            commands.entity(entity).despawn();
        }
    }
}
