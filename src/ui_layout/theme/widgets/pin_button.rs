use crate::prelude::*;
use crate::ui_layout::theme::palette::{COLOR_ACTIVE, COLOR_HEADER_BG, COLOR_INPUT_TEXT};

#[derive(Component, Clone, Default)]
pub struct PinButtonWidget;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update_pin_button_hover);
}

pub fn pin_button<B: Bundle + Clone>(payload: B) -> impl Scene {
    bsn! {
        Button
        PinButtonWidget
        template(move |_| Ok(payload.clone()))
        Node {
            width: Val::Px(24.0),
            height: Val::Px(24.0),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(COLOR_HEADER_BG)
        Children [(
            Text::new("O")
            template(|_| Ok(TextFont::from_font_size(16.0)))
            TextColor(COLOR_INPUT_TEXT)
        )]
    }
}

fn update_pin_button_hover(
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PinButtonWidget>),
    >,
) {
    for (interaction, mut color) in &mut buttons {
        color.set_if_neq(BackgroundColor(match interaction {
            Interaction::Hovered => COLOR_ACTIVE,
            _ => COLOR_HEADER_BG,
        }));
    }
}
