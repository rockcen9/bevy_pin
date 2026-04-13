pub mod close_button;
pub mod entity_card;
pub mod global_message;
pub mod pin_button;
pub mod scrollable_panel;
pub mod unpincard;

pub use close_button::close_button;
pub use entity_card::DragHandle;
pub use global_message::show_global_message;
pub use pin_button::pin_button;
pub use scrollable_panel::{ScrollableContainer, scrollable_list, titled_panel};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    close_button::plugin(app);
    entity_card::plugin(app);
    global_message::plugin(app);
    pin_button::plugin(app);
    scrollable_panel::plugin(app);
    unpincard::plugin(app);
}
