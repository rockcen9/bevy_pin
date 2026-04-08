use crate::prelude::*;

pub mod state;

pub mod resource;

pub mod connection;

pub mod entity_query;

pub mod new_scene;

pub fn plugin(app: &mut App) {
    app.add_plugins(BrpPlugin);

    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
    });

    connection::plugin(app);

    app.init_state::<SidebarState>()
        .register_type::<State<SidebarState>>()
        .register_type::<NextState<SidebarState>>();

    state::plugin(app);

    resource::plugin(app);

    entity_query::plugin(app);

    new_scene::plugin(app);
}
