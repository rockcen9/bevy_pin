mod ext;
pub mod methods;
pub mod types;

pub use ext::BrpCommandsExt;
pub(crate) use json_rpc_helper::{RemoteHelperPlugin, RpcEndpointPlugin};
pub use json_rpc_helper::{RpcResponse, TimeoutError};
pub use types::*;

use bevy::prelude::*;

/// Add once. Registers [`RemoteHelperPlugin`] (timeout systems) and
/// [`RpcEndpointPlugin`] for every built-in BRP response type.
///
/// After this you can use [`BrpCommandsExt`] on [`Commands`] without any
/// additional plugin registration for the built-in methods.
///
/// For custom BRP methods with your own response types, add
/// `RpcEndpointPlugin::<MyResponse>::default()` separately.
pub struct BrpPlugin;

impl Plugin for BrpPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RemoteHelperPlugin)
            .add_plugins(RpcEndpointPlugin::<types::BrpListComponents>::default())
            .add_plugins(RpcEndpointPlugin::<types::BrpListAllComponents>::default())
            .add_plugins(RpcEndpointPlugin::<types::BrpGetComponents>::default())
            .add_plugins(RpcEndpointPlugin::<types::BrpWorldQuery>::default())
            .add_plugins(RpcEndpointPlugin::<types::BrpListResources>::default())
            .add_plugins(RpcEndpointPlugin::<types::BrpGetResources>::default())
            .add_plugins(RpcEndpointPlugin::<types::BrpSpawnEntity>::default())
            .add_plugins(RpcEndpointPlugin::<types::BrpSchema>::default())
            .add_plugins(RpcEndpointPlugin::<types::BrpMutate>::default())
            .add_plugins(RpcEndpointPlugin::<types::BrpHeartbeat>::default());
    }
}
