pub mod palette;
pub mod widgets;
use crate::prelude::*;
pub use widgets::*;

pub fn plugin(app: &mut App) {
    widgets::plugin(app);
}
