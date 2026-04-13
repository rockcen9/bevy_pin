use crate::prelude::*;

pub mod history;
pub mod insert;
pub mod spawned;

pub fn plugin(app: &mut App) {
    app.init_resource::<SpawnedEntityId>();
    history::plugin(app);
    insert::plugin(app);
    spawned::plugin(app);
}
#[derive(Component, Default, Clone, Reflect)]
pub struct NewScenePanelRoot;
#[derive(Component, Default, Clone, Reflect)]
pub struct NewScenePanel;
#[derive(Component, Default, Clone, Reflect)]
pub struct SpawnedEntityPanel;
#[derive(Resource, Default)]
pub struct SpawnedEntityId(Option<u64>);
