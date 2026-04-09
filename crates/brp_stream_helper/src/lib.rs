mod ext;

pub use brp_helper::methods;
pub use brp_helper::types::{
    BrpGetComponents, BrpGetComponentsWatch, BrpListComponentsWatch, BrpWorldQuery,
};
pub use ext::BrpStreamCommandsExt;
pub use stream_helper::{AbortStream, StreamData, StreamDisconnected, StreamPlugin};

use bevy::prelude::*;
use brp_helper::types::{BrpGetComponentsWatch as GetWatch, BrpListComponentsWatch as ListWatch};
use stream_helper::StreamPlugin as SP;

/// Add once. Registers the pump systems for all built-in BRP `+watch` stream types.
///
/// After this you can use [`BrpStreamCommandsExt`] on [`Commands`] and attach
/// observers to the returned entity.
///
/// ```ignore
/// app.add_plugins(BrpStreamPlugin);
/// ```
pub struct BrpStreamPlugin;

impl Plugin for BrpStreamPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SP::<ListWatch>::default())
            .add_plugins(SP::<GetWatch>::default());
    }
}
