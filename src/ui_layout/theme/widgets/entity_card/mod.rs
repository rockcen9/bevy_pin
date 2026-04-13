mod components;
mod interaction;
mod layout;
mod resize;
mod rpc;

pub use components::{
    DragHandle, EntityCard, EntityCardDataCache, EntityCardEntry, EntityCardExpandState,
    EntityCardHighlight, EntityCardTitle, entity_card_key,
};
pub use layout::*;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<components::EntityCardExpandState>()
        .init_resource::<components::EntityCardDataCache>()
        .init_resource::<components::EntityCardKnownMarkerComponents>()
        .add_plugins((interaction::plugin, resize::plugin, rpc::plugin));
}
