use crate::prelude::*;

pub mod state;

pub mod resource;

pub mod connection;

pub mod entity_filter;

pub mod new_scene;

pub mod entity_lookup;

pub mod pinboard;

pub fn plugin(app: &mut App) {
    app.add_plugins(BrpPlugin);
    app.add_plugins(BrpStreamPlugin);

    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
    });

    connection::plugin(app);

    app.init_state::<SidebarState>()
        .register_type::<State<SidebarState>>()
        .register_type::<NextState<SidebarState>>();

    state::plugin(app);

    resource::plugin(app);

    entity_filter::plugin(app);

    new_scene::plugin(app);

    entity_lookup::plugin(app);

    pinboard::plugin(app);
}
