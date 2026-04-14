use crate::prelude::*;
use crate::ui_layout::theme::palette::{
    COLOR_DESTRUCTIVE_HOVER, COLOR_HEADER_BG, COLOR_INPUT_TEXT,
};

#[derive(Component, Clone, Default)]
pub struct RemoveButtonWidget;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update_remove_button_hover);
}

/// Spawns a styled remove ("-") button. The caller provides their own
/// component (e.g. `RemoveButton(key)`) which is inserted via `payload`,
/// keeping all click logic in the calling module.
pub fn remove_button<B: Bundle + Clone>(payload: B) -> impl Scene {
    bsn! {
        Button
        RemoveButtonWidget
        template(move |_| Ok(payload.clone()))
        Node {
            width: Val::Px(20.0),
            height: Val::Px(20.0),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(COLOR_HEADER_BG)
        Children [(
            Text::new("-")
            template(|_| Ok(TextFont::from_font_size(16.0)))
            TextColor(COLOR_INPUT_TEXT)
        )]
    }
}

fn update_remove_button_hover(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RemoveButtonWidget>),
    >,
) {
    for (interaction, mut color) in &mut buttons {
        color.set_if_neq(BackgroundColor(match interaction {
            Interaction::Hovered => COLOR_DESTRUCTIVE_HOVER,
            _ => COLOR_HEADER_BG,
        }));
    }
}
