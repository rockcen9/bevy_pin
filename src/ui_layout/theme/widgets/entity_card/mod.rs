mod children;
mod highlight;
mod interaction;
mod layout;
mod resize;
mod rpc;

use crate::prelude::*;
pub use children::EntityCardChildrenCache;
pub use core::*;
pub use highlight::*;
pub use interaction::DragHandle;
pub use layout::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<EntityCardExpandState>()
        .init_resource::<EntityCardDataCache>()
        .init_resource::<EntityCardKnownMarkerComponents>()
        .add_plugins((
            children::plugin,
            highlight::plugin,
            interaction::plugin,
            resize::plugin,
            rpc::plugin,
        ));
}
