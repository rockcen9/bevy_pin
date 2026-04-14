mod components;
mod highlight;
mod interaction;
mod layout;
mod resize;
mod rpc;

use crate::prelude::*;
pub use components::*;
pub use highlight::*;
pub use interaction::DragHandle;
pub use layout::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<components::EntityCardExpandState>()
        .init_resource::<components::EntityCardDataCache>()
        .init_resource::<components::EntityCardKnownMarkerComponents>()
        .add_plugins((highlight::plugin, interaction::plugin, resize::plugin, rpc::plugin));
}
