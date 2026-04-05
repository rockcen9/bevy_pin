# bevy-spot

This is a web based remote inspector for the Bevy engine, built using the Bevy engine (WASM).

Example setup in [./examples/demo_game.rs](./examples/demo_game.rs)

Try it live at <https://rockcen9.github.io/bevy_pin/>!
The default host is `127.0.0.1:15702`.
You can append `?host=192.168.1.100:15702` to the URL to connect to a completely different address.

## Native Alternative

Run `cargo run` from the project directory. By default, it will keep trying to connect to <http://127.0.0.1:15702>.

## Features

### Component Query

Set up custom component queries using `With<T>` and `Without<T>` filters to track specific entities and see their component changes instantly. (Still a work in progress.)

### State Monitor and Switching

It automatically finds the states in your app and provides visual buttons to easily switch between them or trigger a `NextState`.

### Resource Monitor and Modification

Automatically watch resource values update in real-time, and type new values directly into the UI to send them back to the game.

![Resource monitor and modify](./assets/resource_monitor_and_modify.gif)

## Basic Setup

Enable bevy_remote feature for Bevy

```rust
    let cors_headers = Headers::new()
        .insert(
            "Access-Control-Allow-Origin",
            "https://rockcen9.github.io/bevy_pin/",
        )
        .insert("Access-Control-Allow-Headers", "Content-Type");

    // add remote plugin
    app.add_plugins(RemotePlugin::default()); //
    app.add_plugins(RemoteHttpPlugin::default().with_headers(cors_headers));
```

## States Monitor

```rust
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default, Reflect)]
// #[states(scoped_entities)]
pub enum Screen {
    Splash,
    Title,
    Loading,
    #[default]
    Gameplay,
}

```

## Resource Monitor

```rust
app.init_resource::<House>();
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct House {
    address: String,
    number: u32,
}
```

## Component Monitor

```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Bird {
    hobby: String,
}
```

## Roadmap

- [ ] Display component data
- [ ] Modify component data
- [ ] Pin component by entity ID
- [ ] Cache query result
- [ ] Observer debugger

## License

- [MIT License](./LICENSE-MIT.md)
- [Apache License, Version 2.0](./LICENSE-APACHE-2.0.md)

## Credits

- This project is inspired by:

- [bevy-inspector-egui](https://github.com/jakobhellermann/bevy-inspector-egui) - A huge inspiration for Bevy inspector tools and UI patterns.

- [Flecs Explorer](https://www.flecs.dev/explorer/) - Real-time ECS data visualization and debugging.

## Demos

### States

![States](./docs/demos/state.gif)

### Resources

![Resources](./docs/demos/resource.gif)

### Components

![Components](./docs/demos/query.gif)

## Compatible Versions

It is compatible with older versions of Bevy as long as there are no breaking changes in the Bevy Remote Protocol (BRP).

| Bevy version | `bevy_pin` version |
|:-------------|:--------------------------|
| `0.19 dev`       | `0.1`                    |
