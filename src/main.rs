mod manager;
mod prelude;
mod ui_layout;
mod utils;
mod version;
use crate::prelude::*;
pub const GAME_WIDTH: f32 = 1920.;
pub const GAME_HEIGHT: f32 = 1080.;
pub const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");
use bevy::input_focus::tab_navigation::TabNavigationPlugin;

use tracing_subscriber::field::MakeExt;
fn main() -> AppExit {
    let mut app = App::new();
    let default_plugins = DefaultPlugins
        .set(bevy::log::LogPlugin {
            level: bevy::log::Level::INFO,
            filter: format!(
                concat!(
                    "{default},",
                    "calloop::loop_logic=error,",
                    "bevy_pin::manager::state::get=warn,",
                    "bevy_pin::manager::state::ui=warn,",
                    "bevy_pin::ui::body::content=warn,",
                    "bevy_pin::ui=warn,",
                    "bevy_pin::manager::resource::get=warn,",
                    "bevy_pin::manager::resource::set=warn,",
                    "bevy_pin::manager::resource::ui=warn,",
                    "bevy_pin::manager::component::get=warn,",
                    "bevy_pin::manager::component::query::history=warn,",
                    "bevy_pin::manager::component::query::insert=warn,",
                    "bevy_pin::manager::component::query=warn,",
                    "bevy_pin::manager::component::entity_list::ui=warn,",
                    "bevy_pin::manager::connection::reconnect=warn,",
                    "bevy_pin::manager::new_scene::spawned=warn,",
                    "bevy_pin::manager::entity_filter::component_list=warn,",
                    "bevy_pin::manager::entity_filter::inspector=warn,",
                    "stream_helper=warn,",
                    "json_rpc_helper=warn,",
                    "bevy_pin::manager::entity_filter::entity_list::ui=warn,",
                    "bevy_pin::manager::entity_filter::fetch::discovery=warn,",
                    "bevy_pin::ui_layout::theme::widgets::unpincard=warn,",
                    "bevy_pin::manager::pinboard::ui=warn,",
                    "bevy_pin::manager::pinboard::pincard=warn,",
                ),
                default = bevy::log::DEFAULT_FILTER
            ),
            fmt_layer: |_| {
                Some(Box::new(
                    bevy::log::tracing_subscriber::fmt::Layer::default()
                        .without_time()
                        .map_fmt_fields(MakeExt::debug_alt)
                        .with_writer(std::io::stderr),
                ))
            },
            ..default()
        })
        .set(AssetPlugin {
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Window {
                visible: false,
                title: "Bevy Pin".to_string(),
                fit_canvas_to_parent: true,
                resolution: bevy::window::WindowResolution::new(
                    GAME_WIDTH as u32,
                    GAME_HEIGHT as u32,
                ),
                ..default()
            }
            .into(),
            ..default()
        });
    #[cfg(feature = "dev")]
    let default_plugins =
        default_plugins.disable::<bevy::dev_tools::render_debug::RenderDebugOverlayPlugin>();

    app.add_plugins(default_plugins);
    #[cfg(feature = "dev_native")]
    dogfooding::plugin(&mut app);
    // Set up the `Pause` state.
    app.init_state::<Pause>();
    app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

    app.add_systems(Update, show_window_after_warmup);
    app.add_plugins(manager::plugin);
    app.add_plugins(ui_layout::plugin);
    app.add_plugins(TabNavigationPlugin);
    app.add_plugins(version::plugin);

    app.run()
}

fn show_window_after_warmup(mut window: Query<&mut Window>, mut warmup_frame_count: Local<u32>) {
    *warmup_frame_count += 1;
    if *warmup_frame_count == 5 {
        if let Ok(mut window) = window.single_mut() {
            window.visible = true;
        }
    }
}
/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PausableSystems;

#[cfg(feature = "dev_native")]
mod dogfooding {
    use bevy::{
        prelude::*,
        remote::{
            RemotePlugin,
            http::{Headers, RemoteHttpPlugin},
        },
    };

    pub fn plugin(app: &mut App) {
        let cors_headers = Headers::new()
            .insert("Access-Control-Allow-Origin", "https://rockcen9.github.io")
            .insert("Access-Control-Allow-Headers", "Content-Type");

        // add remote plugin
        app.add_plugins(RemotePlugin::default());
        app.add_plugins(
            RemoteHttpPlugin::default()
                .with_headers(cors_headers)
                .with_port(15703), // default port number + 1
        );
    }
}
