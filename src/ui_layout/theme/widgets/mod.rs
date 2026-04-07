pub mod close_button;
pub mod scrollable_panel;

pub use close_button::close_button;
pub use scrollable_panel::{scrollable_list, titled_panel, ScrollableContainer};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    close_button::plugin(app);
    scrollable_panel::plugin(app);
}
