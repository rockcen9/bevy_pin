use crate::prelude::*;

pub mod insert;
pub mod spawned;

pub fn plugin(app: &mut App) {
    spawned::plugin(app);
    insert::plugin(app);
}
#[derive(Component, Default, Clone, Reflect)]
pub struct NewScenePanelRoot;
