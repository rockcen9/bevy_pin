use bevy::prelude::*;
use bevy::remote::http::Headers;
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};
fn main() {
    let mut app = App::new();
    let default_plugins = DefaultPlugins.build().set(bevy::log::LogPlugin {
        filter:
            "bevy_diagnostic::system_information_diagnostics_plugin=warn,bevy_render::renderer=warn"
                .to_string(),
        ..default()
    });
    #[cfg(feature = "dev")]
    let default_plugins =
        default_plugins.disable::<bevy::dev_tools::render_debug::RenderDebugOverlayPlugin>();
    app.add_plugins(default_plugins);
    game_manager(&mut app);
    app.run();
}

fn game_manager(app: &mut App) {
    let cors_headers = Headers::new()
        .insert("Access-Control-Allow-Origin", "https://rockcen9.github.io")
        .insert("Access-Control-Allow-Headers", "Content-Type");

    // add remote plugin
    app.add_plugins(RemotePlugin::default()); //
    app.add_plugins(RemoteHttpPlugin::default().with_headers(cors_headers));

    // register state
    app.init_state::<Screen>();
    app.add_sub_state::<GameState>();
    app.register_type::<bevy::prelude::State<GameState>>();
    app.register_type::<bevy::prelude::NextState<GameState>>();
    app.register_type::<bevy::prelude::State<Screen>>();
    app.register_type::<bevy::prelude::NextState<Screen>>();

    // register resource
    app.init_resource::<House>();
    app.init_resource::<Company>();

    // spawn entity
    entity_plugin(app);

    // monitor state
    monitor_state(app);

    // _demo_resource(app);

    _demo_component(app);
}

pub fn entity_plugin(app: &mut App) {
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((Bird::default(),));
    });
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((Bird::default(),));
    });
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((Rat::default(),));
    });
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Reflect)]
#[states(scoped_entities)]
pub enum Screen {
    Splash,
    Title,
    Loading,
    #[default]
    Gameplay,
}
#[derive(SubStates, Debug, Hash, PartialEq, Eq, Clone, Default, Reflect)]
#[source(Screen = Screen::Gameplay)]
pub enum GameState {
    #[default]
    Tutorial,
    Preparing,
    Running,
    UIActive,
    Succeeded,
    Failed,
    Win,
}
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct House {
    address: String,
    number: u32,
}
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct Company {
    address: String,
    number: u32,
}
impl Default for Company {
    fn default() -> Self {
        Self {
            address: "Blue Main St".to_string(),
            number: 222,
        }
    }
}
impl Default for House {
    fn default() -> Self {
        Self {
            address: "Red Main St".to_string(),
            number: 111,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Transform, Name::new("Bird"), Age(5), Fly, Animal)]
pub struct Bird {
    hobby: String,
}
impl Default for Bird {
    fn default() -> Self {
        Self {
            hobby: "Fly".to_string(),
        }
    }
}
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Transform, Name::new("Rat"), Age(1), Run, Animal)]
pub struct Rat {
    hobby: String,
}
impl Default for Rat {
    fn default() -> Self {
        Self {
            hobby: "Chew".to_string(),
        }
    }
}
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Name::new("Age"))]
pub struct Age(u32);

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Name::new("Fly"))]
pub struct Fly;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Name::new("Run"))]
pub struct Run;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Name::new("Animal"))]
pub struct Animal;

fn monitor_state(app: &mut App) {
    app.add_systems(
        Update,
        (
            |mut events: MessageReader<StateTransitionEvent<Screen>>| {
                for e in events.read() {
                    info!("Screen: {:?} -> {:?}", e.exited, e.entered);
                }
            },
            |mut events: MessageReader<StateTransitionEvent<GameState>>| {
                for e in events.read() {
                    info!("GameState: {:?} -> {:?}", e.exited, e.entered);
                }
            },
        ),
    );
}

fn _spam_company(company: Option<Res<Company>>, time: Res<Time>, mut timer: Local<Timer>) {
    timer.tick(time.delta());
    if timer.just_finished() {
        *timer = Timer::from_seconds(1.0, TimerMode::Once);
        info!("Company: {:?}", company.as_deref());
    }
}

fn _demo_resource(app: &mut App) {
    app.add_systems(Update, _spam_company);
    app.add_systems(
        Update,
        |mut house: ResMut<House>, time: Res<Time>, mut timer: Local<Timer>| {
            timer.tick(time.delta());
            if timer.just_finished() {
                *timer = Timer::from_seconds(1.0, TimerMode::Once);
                house.number += 1;
            }
        },
    );
    app.add_systems(
        Update,
        |mut commands: Commands,
         company: Option<Res<Company>>,
         time: Res<Time>,
         mut timer: Local<Timer>| {
            timer.tick(time.delta());
            if !timer.just_finished() {
                return;
            }
            *timer = Timer::from_seconds(3.0, TimerMode::Once);
            if company.is_none() {
                let inserted = Company::default();
                info!("Company: insert {:?}", inserted);
                commands.insert_resource(inserted);
            } else {
                info!("Company: remove");
                commands.remove_resource::<Company>();
            }
        },
    );
}
fn _demo_component(app: &mut App) {
    app.add_systems(Update, spawn_rat_per_sec);
}
fn spawn_rat_per_sec(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: Local<Timer>,
    mut count: Local<u32>,
) {
    if *count >= 20 {
        return;
    }
    if timer.duration().is_zero() {
        *timer = Timer::from_seconds(1.0, TimerMode::Repeating);
    }
    timer.tick(time.delta());
    if timer.just_finished() {
        commands.spawn((Rat::default(),));
        *count += 1;
        println!("spawned rat {} / 10", *count);
    }
}
