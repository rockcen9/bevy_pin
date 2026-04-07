pub mod palette;
pub mod widgets;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    widgets::plugin(app);
}
