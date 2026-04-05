use crate::prelude::*;

pub mod state;

pub mod resource;

pub mod connection;

pub mod component;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
    });

    connection::plugin(app);

    app.add_sub_state::<SidebarState>();

    state::plugin(app);

    resource::plugin(app);

    component::plugin(app);
}
