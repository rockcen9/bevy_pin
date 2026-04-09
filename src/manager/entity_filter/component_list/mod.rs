use crate::prelude::*;

mod list;
pub use list::*;

mod unknown_issue;

pub fn plugin(app: &mut App) {
    app.add_plugins((list::plugin, unknown_issue::plugin));
}
